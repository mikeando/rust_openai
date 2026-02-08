use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, bail};
use data_encoding::HEXLOWER;
use inscenerator_xfs::{OsFs, Xfs};
use ring::digest;
use serde_json::json;

use crate::{
    json::{FromJson, ToJson},
    types::{ChatCompletionObject, ChatRequest},
};

pub struct OpenAILLM {
    requester: Arc<dyn RawRequester + Send + Sync>,
    cache: Arc<Mutex<dyn RequestCache + Send>>,
}

impl OpenAILLM {
    pub fn with_defaults(openai_api_key: &str) -> anyhow::Result<OpenAILLM> {
        let openai_api_key = openai_api_key.to_string();
        let requester = OpenAIRawRequester { openai_api_key };
        let requester = Arc::new(requester);
        let fs = OsFs {};
        let fs = Arc::new(Mutex::new(fs));
        let cache = DefaultRequestCache::new(fs, PathBuf::from("cache"))?;
        let cache = Arc::new(Mutex::new(cache));
        Ok(OpenAILLM::new(requester, cache))
    }

    pub fn new(
        requester: Arc<dyn RawRequester + Send + Sync>,
        cache: Arc<Mutex<dyn RequestCache + Send>>,
    ) -> OpenAILLM {
        OpenAILLM { requester, cache }
    }
}

pub trait RawRequester {
    fn make_uncached_request(
        &self,
        request: &ChatRequest,
    ) -> anyhow::Result<ChatCompletionObject>;
}

pub struct OpenAIRawRequester {
    pub openai_api_key: String,
}

impl RawRequester for OpenAIRawRequester {
    fn make_uncached_request(
        &self,
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
        &self,
        request: &ChatRequest,
        response: &ChatCompletionObject,
    ) -> anyhow::Result<()>;
}

pub struct DefaultRequestCache {
    fs: Arc<Mutex<dyn Xfs + Send>>,
    root: PathBuf,
}

impl DefaultRequestCache {
    pub fn new(
        fs: Arc<Mutex<dyn Xfs + Send>>,
        root: PathBuf,
    ) -> anyhow::Result<DefaultRequestCache> {
        let is_dir = fs.lock().unwrap().is_dir(&root);
        if !is_dir {
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

        let fs = self.fs.lock().unwrap();
        if let Ok(reader) = fs.reader(&cache_file_path) {
            let value: serde_json::Value = serde_json::from_reader(reader)?;
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
        &self,
        request: &ChatRequest,
        response: &ChatCompletionObject,
    ) -> anyhow::Result<()> {
        let key = self.key(request);
        let cache_file_path = self.key_to_path(&key);

        let cache_entry = json!({
            "request": request.to_json(),
            "response": response.to_json(),
        });

        let mut fs = self.fs.lock().unwrap();
        let writer = fs
            .writer(&cache_file_path)
            .map_err(|e| anyhow!("Cache write failed: {}", e))?;
        serde_json::to_writer_pretty(writer, &cache_entry)?;

        Ok(())
    }
}

impl OpenAILLM {
    pub fn make_request(
        &self,
        request: &ChatRequest,
    ) -> anyhow::Result<(ChatCompletionObject, bool)> {
        if let Some(v) = self.cache.lock().unwrap().get_response_if_cached(request)? {
            return Ok((v, true));
        }

        let response = self
            .requester
            .make_uncached_request(request)?;

        self.cache
            .lock()
            .unwrap()
            .cache_response(request, &response)?;

        Ok((response, false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::{Generatable, GeneratorContext};
    use crate::types::{Message, ModelId};
    use inscenerator_xfs::mockfs::MockFS;
    use std::path::Path;

    #[test]
    fn test_cache_roundtrip() -> anyhow::Result<()> {
        let mut fs = MockFS::new();
        let root = Path::new("/cache");
        fs.create_dir_all(root)?;

        let fs = Arc::new(Mutex::new(fs));
        let cache = DefaultRequestCache::new(fs, root.to_path_buf())?;

        let request = ChatRequest::new(ModelId::Gpt5(None), vec![Message::user_message("hello")]);

        let mut context = GeneratorContext::new();
        let mut response = ChatCompletionObject::gen(&mut context);
        response.id = "test".to_string();

        // Cache it
        cache.cache_response(&request, &response)?;

        // Retrieve it
        let cached_response = cache.get_response_if_cached(&request)?;
        assert!(cached_response.is_some());
        assert_eq!(cached_response.unwrap().id, "test");

        Ok(())
    }
}
