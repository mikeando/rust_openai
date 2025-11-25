use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, bail};
use data_encoding::HEXLOWER;
use ring::digest;
use serde_json::json;

use crate::{
    json::{FromJson, ToJson},
    types::{ChatCompletionObject, ChatRequest},
};

pub struct OpenAILLM {
    requester: Arc<Mutex<dyn RawRequester + Send>>,
    cache: Arc<Mutex<dyn RequestCache + Send>>,
}

impl OpenAILLM {
    pub fn with_defaults(openai_api_key: &str) -> anyhow::Result<OpenAILLM> {
        let openai_api_key = openai_api_key.to_string();
        let requester = OpenAIRawRequester { openai_api_key };
        let requester = Arc::new(Mutex::new(requester));
        let fs = DefaultFS {};
        let fs = Arc::new(Mutex::new(fs));
        let cache = DefaultRequestCache::new(fs, PathBuf::from("cache"))?;
        let cache = Arc::new(Mutex::new(cache));
        Ok(OpenAILLM::new(requester, cache))
    }

    pub fn new(
        requester: Arc<Mutex<dyn RawRequester + Send>>,
        cache: Arc<Mutex<dyn RequestCache + Send>>,
    ) -> OpenAILLM {
        OpenAILLM { requester, cache }
    }
}

pub trait RawRequester {
    fn make_uncached_request(
        &mut self,
        request: &ChatRequest,
    ) -> anyhow::Result<ChatCompletionObject>;
}

pub struct OpenAIRawRequester {
    pub openai_api_key: String,
}

impl RawRequester for OpenAIRawRequester {
    fn make_uncached_request(
        &mut self,
        request: &ChatRequest,
    ) -> anyhow::Result<ChatCompletionObject> {
        let response = ureq::post("https://api.openai.com/v1/responses")
            .set("Content-Type", "application/json")
            .set("Authorization", &format!("Bearer {}", self.openai_api_key))
            .send_json(request.to_json());

        let response = match response {
            Ok(response) => response,
            Err(ureq::Error::Status(status, response)) => {
                let response_text = response.into_string()?;
                let v: serde_json::Value = serde_json::from_str(&response_text)?;
                let error_message = &v["error"]["message"];
                bail!(
                    "Error making openai request: ({} {}) {}",
                    status,
                    "Unknown",
                    error_message.as_str().unwrap_or("")
                );
            }
            Err(e) => {
                return Err(anyhow!("Error making request: {}", e));
            }
        };

        let response_text = response.into_string()?;
        println!("DEBUG: Raw OpenAI response JSON: {}", response_text);
        let v: serde_json::Value = serde_json::from_str(&response_text)?;
        let response: ChatCompletionObject = ChatCompletionObject::from_json(&v)
            .map_err(|e| anyhow!("Error decoding openai response: {:?}", e))?;
        Ok(response)
    }
}

pub trait RequestCache {
    fn get_response_if_cached(
        &self,
        request: &ChatRequest,
    ) -> anyhow::Result<Option<ChatCompletionObject>>;
    fn cache_response(
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

pub trait TrivialFS {
    fn read_to_string(&self, p: &Path) -> anyhow::Result<String>;
    fn write(&self, p: &Path, value: &str) -> anyhow::Result<()>;
    fn path_type(&self, p: &Path) -> anyhow::Result<TrivialFSPathType>;
}

pub struct DefaultFS {}

impl TrivialFS for DefaultFS {
    fn read_to_string(&self, p: &Path) -> anyhow::Result<String> {
        Ok(std::fs::read_to_string(p)?)
    }
    fn write(&self, p: &Path, value: &str) -> anyhow::Result<()> {
        std::fs::write(p, value)?;
        Ok(())
    }
    fn path_type(&self, p: &Path) -> anyhow::Result<TrivialFSPathType> {
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
    fs: Arc<Mutex<dyn TrivialFS + Send>>,
    root: PathBuf,
}

impl DefaultRequestCache {
    pub fn new(
        fs: Arc<Mutex<dyn TrivialFS + Send>>,
        root: PathBuf,
    ) -> anyhow::Result<DefaultRequestCache> {
        let r = fs.lock().unwrap().path_type(&root)?;
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
        let full_key = HEXLOWER.encode(digest.as_ref());
        let key = &full_key[0..32];
        key.to_string()
    }

    fn key_to_path(&self, key: &str) -> PathBuf {
        self.root.join(format!("{}.json", key))
    }
}

impl RequestCache for DefaultRequestCache {
    fn get_response_if_cached(
        &self,
        request: &ChatRequest,
    ) -> anyhow::Result<Option<ChatCompletionObject>> {
        let key = self.key(request);
        let cache_file_path = self.key_to_path(&key);

        if let Ok(content) = self.fs.lock().unwrap().read_to_string(&cache_file_path) {
            let value: serde_json::Value = serde_json::from_str(&content)?;
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

    fn cache_response(
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

        self.fs.lock().unwrap().write(
            &cache_file_path,
            &serde_json::to_string_pretty(&cache_entry).unwrap(),
        )?;

        Ok(())
    }
}

impl OpenAILLM {
    pub fn make_request(
        &mut self,
        request: &ChatRequest,
    ) -> anyhow::Result<(ChatCompletionObject, bool)> {
        if let Some(v) = self.cache.lock().unwrap().get_response_if_cached(request)? {
            return Ok((v, true));
        }

        let response = self
            .requester
            .lock()
            .unwrap()
            .make_uncached_request(request)?;

        self.cache
            .lock()
            .unwrap()
            .cache_response(request, &response)?;

        Ok((response, false))
    }
}
