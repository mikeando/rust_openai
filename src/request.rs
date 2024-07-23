use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, bail};
use async_trait::async_trait;
use data_encoding::HEXLOWER;
use reqwest::Client;
use ring::digest;
use serde_json::json;

type AsyncMutex<T> = tokio::sync::Mutex<T>;

use crate::{
    json::{FromJson, ToJson},
    types::{ChatCompletionObject, ChatRequest},
};

pub struct OpenAILLM {
    requester: Arc<AsyncMutex<dyn RawRequester + Send>>,
    cache: Arc<AsyncMutex<dyn RequestCache + Send>>,
}

impl OpenAILLM {
    pub async fn with_defaults(openai_api_key: &str) -> anyhow::Result<OpenAILLM> {
        let openai_api_key = openai_api_key.to_string();
        let requester = OpenAIRawRequester { openai_api_key };
        let requester = Arc::new(AsyncMutex::new(requester));
        let fs = DefaultFS {};
        let fs = Arc::new(AsyncMutex::new(fs));
        let cache = DefaultRequestCache::new(fs, PathBuf::from("cache")).await?;
        let cache = Arc::new(AsyncMutex::new(cache));
        Ok(OpenAILLM::new(requester, cache))
    }

    pub fn new(
        requester: Arc<AsyncMutex<dyn RawRequester + Send>>,
        cache: Arc<AsyncMutex<dyn RequestCache + Send>>,
    ) -> OpenAILLM {
        OpenAILLM { requester, cache }
    }
}

#[async_trait]
pub trait RawRequester {
    async fn make_uncached_request(
        &mut self,
        request: &ChatRequest,
    ) -> anyhow::Result<ChatCompletionObject>;
}

pub struct OpenAIRawRequester {
    pub openai_api_key: String,
}

#[async_trait]
impl RawRequester for OpenAIRawRequester {
    async fn make_uncached_request(
        &mut self,
        request: &ChatRequest,
    ) -> anyhow::Result<ChatCompletionObject> {
        let client = Client::new();

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.openai_api_key))
            .json(&request.to_json())
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            // Try and see if we can divine the reason from the JSON,
            // if we get any...
            // TODO: If this doesn't work we should at least return what we do know
            let response_text = response.text().await?;
            let v: serde_json::Value = serde_json::from_str(&response_text)?;
            let error_message = &v["error"]["message"];

            bail!(
                "Error making openai request: ({} {}) {}",
                status.as_str(),
                status.canonical_reason().unwrap_or("Unknown"),
                error_message.as_str().unwrap_or("")
            );
        }

        let response_text = response.text().await?;
        let v: serde_json::Value = serde_json::from_str(&response_text)?;
        let response: ChatCompletionObject = ChatCompletionObject::from_json(&v)
            .map_err(|e| anyhow!("Error decoding openai response: {:?}", e))?;
        Ok(response)
    }
}

#[async_trait]
pub trait RequestCache {
    //TODO: Use a better Result type!
    async fn get_response_if_cached(
        &self,
        request: &ChatRequest,
    ) -> anyhow::Result<Option<ChatCompletionObject>>;
    async fn cache_response(
        &mut self,
        request: &ChatRequest,
        response: &ChatCompletionObject,
    ) -> anyhow::Result<()>;
}

#[derive(Debug, PartialEq)]
pub enum TrivialFSPathType {
    NoSuchPath,
    File,
    Directory,
}

#[async_trait]
pub trait TrivialFS {
    async fn read_to_string(&self, p: &Path) -> anyhow::Result<String>;
    async fn write(&self, p: &Path, value: &str) -> anyhow::Result<()>;
    async fn path_type(&self, p: &Path) -> anyhow::Result<TrivialFSPathType>;
}

pub struct DefaultFS {}

#[async_trait]
impl TrivialFS for DefaultFS {
    async fn read_to_string(&self, p: &Path) -> anyhow::Result<String> {
        Ok(std::fs::read_to_string(p)?)
    }
    async fn write(&self, p: &Path, value: &str) -> anyhow::Result<()> {
        std::fs::write(p, value)?;
        Ok(())
    }
    async fn path_type(&self, p: &Path) -> anyhow::Result<TrivialFSPathType> {
        use std::io::ErrorKind;

        let r = std::fs::metadata(p);
        match r {
            Ok(metadata) => {
                if metadata.is_file() {
                    Ok(TrivialFSPathType::File)
                } else if metadata.is_dir() {
                    Ok(TrivialFSPathType::Directory)
                } else {
                    Err(anyhow!(
                        "path_type failed: '{}' is an invalid path type",
                        p.display()
                    ))
                }
            }
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(TrivialFSPathType::NoSuchPath),
            Err(e) => Err(anyhow!(
                "path_type failed when stating {}:  {}",
                p.display(),
                e
            )),
        }
    }
}

pub struct DefaultRequestCache {
    fs: Arc<AsyncMutex<dyn TrivialFS + Send>>,
    root: PathBuf,
}

impl DefaultRequestCache {
    pub async fn new(
        fs: Arc<AsyncMutex<dyn TrivialFS + Send>>,
        root: PathBuf,
    ) -> anyhow::Result<DefaultRequestCache> {
        let r = fs.lock().await.path_type(&root).await?;
        if r != TrivialFSPathType::Directory {
            bail!(
                "DefaultRrequestCache::new failed - '{}' is not a directory",
                root.display()
            );
        }
        Ok(DefaultRequestCache { fs, root })
    }

    fn key(&self, value: &ChatRequest) -> String {
        let request_json = value.to_json();
        let request_str = request_json.to_string();
        let digest = digest::digest(&digest::SHA256, request_str.as_bytes());
        // The key length is way too big for what we want.
        let full_key = HEXLOWER.encode(digest.as_ref());
        let key = &full_key[0..32];
        key.to_string()
    }

    fn key_to_path(&self, key: &str) -> PathBuf {
        self.root.join(format!("{}.json", key))
    }
}

#[async_trait]
impl RequestCache for DefaultRequestCache {
    async fn get_response_if_cached(
        &self,
        request: &ChatRequest,
    ) -> anyhow::Result<Option<ChatCompletionObject>> {
        // First check if we have a cached result
        let key = self.key(request);

        let cache_file_path = self.key_to_path(&key);

        // Open and read the cache file if it exists
        if let Ok(content) = self.fs.lock().await.read_to_string(&cache_file_path).await {
            // Convert the content to json
            let value: serde_json::Value = serde_json::from_str(&content)?;
            // Get the request
            let cached_request = ChatRequest::from_json(&value["request"])
                .map_err(|_e| anyhow!("unable to decode request"))?;
            let cached_response = ChatCompletionObject::from_json(&value["response"])
                .map_err(|_e| anyhow!("unabled to decode resposne"))?;
            if cached_request != *request {
                anyhow::bail!("Cached request does not match!");
            }
            Ok(Some(cached_response))
        } else {
            Ok(None)
        }
    }

    async fn cache_response(
        &mut self,
        request: &ChatRequest,
        response: &ChatCompletionObject,
    ) -> anyhow::Result<()> {
        let key = self.key(request);

        let cache_file_path = self.key_to_path(&key);

        let cache_entry = json!({
            "request": request.to_json(),
            "response": response.to_json(),
        });

        self.fs
            .lock()
            .await
            .write(
                &cache_file_path,
                &serde_json::to_string_pretty(&cache_entry).unwrap(),
            )
            .await
            .unwrap();

        Ok(())
    }
}

impl OpenAILLM {
    pub async fn make_request(
        &mut self,
        request: &ChatRequest,
    ) -> anyhow::Result<(ChatCompletionObject, bool)> {
        // First check if we have a cached result
        if let Some(v) = self
            .cache
            .lock()
            .await
            .get_response_if_cached(request)
            .await
            .unwrap()
        {
            return Ok((v, true));
        }

        // There is no cache value!
        // Make the request
        let response = self
            .requester
            .lock()
            .await
            .make_uncached_request(request)
            .await?;

        // Cache it for next time
        self.cache
            .lock()
            .await
            .cache_response(request, &response)
            .await?;

        Ok((response, false))
    }
}
