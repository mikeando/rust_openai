use reqwest::Client;
use serde_json::json;

use crate::json::ToJson;

struct EmbeddingRequest {
    input: String,
    model: String,
}

impl ToJson for EmbeddingRequest {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "input": self.input,
            "model": self.model,
        })
    }
}

//TODO: Add a caching layer like we use for completion requests.
pub async fn make_uncached_embedding_request(text: &str, openai_api_key: &str) -> Vec<f32> {
    let client = Client::new();

    let request = EmbeddingRequest {
        input: text.to_string(),
        model: "text-embedding-3-small".to_string(),
    };

    let response = client
        .post("https://api.openai.com/v1/embeddings")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", openai_api_key))
        .json(&request.to_json())
        .send()
        .await
        .unwrap();

    let response_text = response.text().await.unwrap();
    // eprintln!("---- raw response ---\n{}\n", response_text);
    let v: serde_json::Value = serde_json::from_str(&response_text).unwrap();
    let r: Vec<f32> = v["data"][0]["embedding"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect();
    r
}
