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
        Self::with_cache_dir(openai_api_key, PathBuf::from("cache"))
    }

    /// Create an `OpenAILLM` with the response cache stored at `cache_dir`.
    /// The directory is created if it does not yet exist.
    pub fn with_cache_dir(openai_api_key: &str, cache_dir: PathBuf) -> anyhow::Result<OpenAILLM> {
        std::fs::create_dir_all(&cache_dir)?;
        let openai_api_key = openai_api_key.to_string();
        let requester = OpenAIRawRequester { openai_api_key };
        let requester = Arc::new(requester);
        let fs = OsFs {};
        let fs = Arc::new(Mutex::new(fs));
        let cache = DefaultRequestCache::new(fs, cache_dir)?;
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
        let request_json = normalize_call_ids(value.to_json());
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

/// Replace call_id values in the `input` array with stable sequential
/// placeholders so that cache keys are independent of the ephemeral
/// identifiers assigned by different LLM API runs or versions.
///
/// The same call_id string always maps to the same placeholder within one
/// request (so function_call and function_call_output items that share a
/// call_id remain linked after normalisation).
fn normalize_call_ids(mut request: serde_json::Value) -> serde_json::Value {
    let input = match request.get_mut("input").and_then(|v| v.as_array_mut()) {
        Some(arr) => arr,
        None => return request,
    };

    let mut mapping: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut counter = 0usize;

    for item in input.iter_mut() {
        if let Some(id) = item.get("call_id").and_then(|v| v.as_str()).map(|s| s.to_string()) {
            let placeholder = mapping
                .entry(id)
                .or_insert_with(|| {
                    let p = format!("call_{}", counter);
                    counter += 1;
                    p
                })
                .clone();
            item["call_id"] = serde_json::Value::String(placeholder);
        }
    }

    request
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
            // Compare after normalising call IDs so cache hits are independent
            // of ephemeral identifiers assigned by different API runs.
            if normalize_call_ids(cached_request.to_json())
                != normalize_call_ids(request.to_json())
            {
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
    use crate::types::{FunctionCallItem, Message, ModelId, ToolMessage};
    use inscenerator_xfs::mockfs::MockFS;
    use std::path::Path;

    fn make_cache() -> DefaultRequestCache {
        let mut fs = MockFS::new();
        fs.create_dir_all(Path::new("/cache")).unwrap();
        let fs = Arc::new(Mutex::new(fs));
        DefaultRequestCache::new(fs, Path::new("/cache").to_path_buf()).unwrap()
    }

    fn stub_response(id: &str) -> ChatCompletionObject {
        let mut context = GeneratorContext::new();
        let mut r = ChatCompletionObject::gen(&mut context);
        r.id = id.to_string();
        r
    }

    #[test]
    fn test_cache_roundtrip() -> anyhow::Result<()> {
        let cache = make_cache();
        let request = ChatRequest::new(ModelId::Gpt5(None), vec![Message::user_message("hello")]);
        let response = stub_response("test");

        cache.cache_response(&request, &response)?;

        let cached = cache.get_response_if_cached(&request)?;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().id, "test");
        Ok(())
    }

    /// Cache a request whose input contains function_call / function_call_output
    /// items with one set of call IDs. Verify that looking up the same logical
    /// request with *different* call IDs still produces a cache hit.
    #[test]
    fn test_cache_hit_with_different_call_ids() -> anyhow::Result<()> {
        let cache = make_cache();

        let request_a = ChatRequest::new(
            ModelId::Gpt5(None),
            vec![
                Message::user_message("write me a poem"),
                Message::FunctionCallItem(FunctionCallItem {
                    call_id: "call_OLD_abc".to_string(),
                    name: "search".to_string(),
                    arguments: r#"{"q":"poems"}"#.to_string(),
                }),
                Message::ToolMessage(ToolMessage {
                    tool_call_id: "call_OLD_abc".to_string(),
                    content: "found some poems".to_string(),
                }),
            ],
        );

        cache.cache_response(&request_a, &stub_response("cached"))?;

        // Same conversation, different call IDs (e.g. from a different API run).
        let request_b = ChatRequest::new(
            ModelId::Gpt5(None),
            vec![
                Message::user_message("write me a poem"),
                Message::FunctionCallItem(FunctionCallItem {
                    call_id: "fc_NEW_xyz".to_string(),
                    name: "search".to_string(),
                    arguments: r#"{"q":"poems"}"#.to_string(),
                }),
                Message::ToolMessage(ToolMessage {
                    tool_call_id: "fc_NEW_xyz".to_string(),
                    content: "found some poems".to_string(),
                }),
            ],
        );

        let cached = cache.get_response_if_cached(&request_b)?;
        assert!(cached.is_some(), "expected cache hit despite different call IDs");
        assert_eq!(cached.unwrap().id, "cached");
        Ok(())
    }

    /// A request with different *content* must not hit the cache even if the
    /// call IDs happen to be the same.
    #[test]
    fn test_cache_miss_on_different_content() -> anyhow::Result<()> {
        let cache = make_cache();

        let request_a = ChatRequest::new(
            ModelId::Gpt5(None),
            vec![
                Message::user_message("write me a poem"),
                Message::FunctionCallItem(FunctionCallItem {
                    call_id: "call_1".to_string(),
                    name: "search".to_string(),
                    arguments: r#"{"q":"poems"}"#.to_string(),
                }),
                Message::ToolMessage(ToolMessage {
                    tool_call_id: "call_1".to_string(),
                    content: "found some poems".to_string(),
                }),
            ],
        );

        cache.cache_response(&request_a, &stub_response("cached"))?;

        let request_b = ChatRequest::new(
            ModelId::Gpt5(None),
            vec![
                Message::user_message("write me a poem"),
                Message::FunctionCallItem(FunctionCallItem {
                    call_id: "call_1".to_string(),
                    name: "search".to_string(),
                    arguments: r#"{"q":"sonnets"}"#.to_string(), // different args
                }),
                Message::ToolMessage(ToolMessage {
                    tool_call_id: "call_1".to_string(),
                    content: "found some poems".to_string(),
                }),
            ],
        );

        let cached = cache.get_response_if_cached(&request_b)?;
        assert!(cached.is_none(), "expected cache miss on different content");
        Ok(())
    }
}
