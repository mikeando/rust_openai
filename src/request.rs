use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::anyhow;
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
    pub fn with_defaults(openai_api_key: &str) -> OpenAILLM {
        let openai_api_key = openai_api_key.to_string();
        let requester = OpenAIRawRequester { openai_api_key };
        let requester = Arc::new(AsyncMutex::new(requester));
        let fs = DefaultFS {};
        let fs = Arc::new(AsyncMutex::new(fs));
        let cache = DefaultRequestCache {
            fs,
            root: PathBuf::from("cache"),
        };
        let cache = Arc::new(AsyncMutex::new(cache));
        OpenAILLM::new(requester, cache)
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
    async fn make_uncached_request(&mut self, request: &ChatRequest) -> ChatCompletionObject;
}

pub struct OpenAIRawRequester {
    pub openai_api_key: String,
}

#[async_trait]
impl RawRequester for OpenAIRawRequester {
    async fn make_uncached_request(&mut self, request: &ChatRequest) -> ChatCompletionObject {
        let client = Client::new();

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.openai_api_key))
            .json(&request.to_json())
            .send()
            .await
            .unwrap();

        let response_text = response.text().await.unwrap();
        // eprintln!("---- raw response ---\n{}\n", response_text);
        let v: serde_json::Value = serde_json::from_str(&response_text).unwrap();
        let response: ChatCompletionObject = ChatCompletionObject::from_json(&v).unwrap();
        response
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

#[async_trait]
pub trait TrivialFS {
    async fn read_to_string(&self, p: &Path) -> anyhow::Result<String>;
    async fn write(&self, p: &Path, value: &str) -> anyhow::Result<()>;
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
}

pub struct DefaultRequestCache {
    pub fs: Arc<AsyncMutex<dyn TrivialFS + Send>>,
    pub root: PathBuf,
}

impl DefaultRequestCache {
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
    pub async fn make_request(&mut self, request: &ChatRequest) -> (ChatCompletionObject, bool) {
        // First check if we have a cached result
        if let Some(v) = self
            .cache
            .lock()
            .await
            .get_response_if_cached(request)
            .await
            .unwrap()
        {
            return (v, true);
        }

        // There is no cache value!
        // Make the request
        let response = self
            .requester
            .lock()
            .await
            .make_uncached_request(request)
            .await;

        // Cache it for next time
        self.cache
            .lock()
            .await
            .cache_response(request, &response)
            .await
            .unwrap();

        (response, false)
    }
}
