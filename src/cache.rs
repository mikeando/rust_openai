use data_encoding::HEXLOWER;
use ring::digest;
use serde_json::json;

use crate::{json::{FromJson, ToJson}, types::{ChatCompletionObject, ChatRequest}};


pub trait Keyer<T, K> {
    fn key(&self, value: &T) -> K;
}

pub struct DefaultSHA256FromChatRequest {}

impl Keyer<ChatRequest, String> for DefaultSHA256FromChatRequest {
    fn key(&self, value: &ChatRequest) -> String {
        let request_json = value.to_json();
        let request_str = request_json.to_string();
        let digest = digest::digest(&digest::SHA256, request_str.as_bytes());
        // The key length is way too big for what we want.
        let full_key = HEXLOWER.encode(digest.as_ref());
        let key = &full_key[0..32];
        key.to_string()
    }
}

pub trait Cache<K, V> {
    fn get_value_if_cached(&self, key: &K) -> Option<V>;
    fn cache_value(&mut self, key: &K, value: &V);
}

pub struct DefaultFilesystemCache {}

impl DefaultFilesystemCache {
    fn key_to_path(&self, key: &str) -> String {
        format!("cache/{}.json", key)
    }
}

pub struct CacheEntry {
    pub request: ChatRequest,
    pub response: ChatCompletionObject,
}

impl Cache<String, CacheEntry> for DefaultFilesystemCache {
    fn get_value_if_cached(&self, key: &String) -> Option<CacheEntry> {
        let cache_file_path = self.key_to_path(&key);

        // Open and read the cache file if it exists
        if let Ok(content) = std::fs::read_to_string(&cache_file_path) {
            // Convert the content to json
            let value: serde_json::Value = serde_json::from_str(&content).unwrap();
            // Get the request
            let request = ChatRequest::from_json(&value["request"]).unwrap();
            let response = ChatCompletionObject::from_json(&value["response"]).unwrap();
            Some(CacheEntry { request, response })
        } else {
            None
        }
    }

    fn cache_value(&mut self, key: &String, value: &CacheEntry) {
        let cache_file_path = self.key_to_path(&key);

        let cache_entry = json!({
            "request": value.request.to_json(),
            "response": value.response.to_json(),
        });

        std::fs::write(
            cache_file_path,
            serde_json::to_string_pretty(&cache_entry).unwrap(),
        )
        .unwrap();
    }
}