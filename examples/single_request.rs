use data_encoding::HEXLOWER;
use reqwest::Client;
use ring::digest;
use serde_json::json;
use tokio;

use rust_openai::json::{FromJson, ToJson};
use rust_openai::types::{ChatCompletionObject, ChatRequest, Message, ModelId};
use std::env;
use std::fs::read_to_string;

pub trait Keyer<T, K> {
    fn key(&self, value: &T) -> K;
}

struct DefaultSHA256FromChatRequest {}

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

struct DefaultFilesystemCache {}
struct CacheEntry {
    request: ChatRequest,
    response: ChatCompletionObject,
}

impl DefaultFilesystemCache {
    fn key_to_path(&self, key: &str) -> String {
        format!("cache/{}.json", key)
    }
}

impl Cache<String, CacheEntry> for DefaultFilesystemCache {
    fn get_value_if_cached(&self, key: &String) -> Option<CacheEntry> {
        let cache_file_path = self.key_to_path(&key);

        // Open and read the cache file if it exists
        if let Ok(content) = read_to_string(&cache_file_path) {
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

async fn make_uncached_request(
    request: &ChatRequest,
    openai_api_key: &str,
) -> ChatCompletionObject {
    let client = Client::new();

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", openai_api_key))
        .json(&request.to_json())
        .send()
        .await
        .unwrap();

    let response_text = response.text().await.unwrap();
    let v: serde_json::Value = serde_json::from_str(&response_text).unwrap();
    let response: ChatCompletionObject = ChatCompletionObject::from_json(&v).unwrap();
    response
}

async fn make_request(request: &ChatRequest, openai_api_key: &str) -> (ChatCompletionObject, bool) {
    // First check if we have a cached result
    let key = DefaultSHA256FromChatRequest {}.key(request);

    let cache_value = DefaultFilesystemCache {}.get_value_if_cached(&key);
    if let Some(cache_value) = cache_value {
        if cache_value.request != *request {
            panic!("Cached request does not match!");
        }
        return (cache_value.response, true);
    }

    // There is no cache value!
    // Make the request
    let response = make_uncached_request(request, openai_api_key).await;

    // Now we've got a response. Cache it
    let cache_entry = CacheEntry {
        request: request.clone(),
        response: response.clone(),
    };
    DefaultFilesystemCache {}.cache_value(&key, &cache_entry);

    (response, false)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
    eprintln!("{:?}", openai_api_key);

    let request: ChatRequest = ChatRequest::new(
        ModelId::Gpt35Turbo,
        vec![
            Message::system_message("You are a helpful assistant."),
            Message::user_message("Hello!"),
        ],
    );

    let (response, is_from_cache) = make_request(&request, &openai_api_key).await;

    println!("is from cache: {}", is_from_cache);
    println!("{:#?}", response);

    Ok(())
}
