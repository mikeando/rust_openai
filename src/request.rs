use reqwest::Client;

use crate::{
    cache::{Cache, CacheEntry, DefaultFilesystemCache, DefaultSHA256FromChatRequest, Keyer},
    json::{FromJson, ToJson},
    types::{ChatCompletionObject, ChatRequest},
};

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
    // eprintln!("---- raw response ---\n{}\n", response_text);
    let v: serde_json::Value = serde_json::from_str(&response_text).unwrap();
    let response: ChatCompletionObject = ChatCompletionObject::from_json(&v).unwrap();
    response
}

pub async fn make_request(
    request: &ChatRequest,
    openai_api_key: &str,
) -> (ChatCompletionObject, bool) {
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
