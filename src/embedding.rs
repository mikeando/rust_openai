use reqwest::Client;
use serde_json::json;
use snafu::{OptionExt, ResultExt, Snafu};

use crate::json::ToJson;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not make http request to OpenAI"))]
    Request { source: reqwest::Error },

    #[snafu(display("Bad HTTP response status from OpenAI ({status})"))]
    Http { response_text: String, status: u16 },

    #[snafu(display("Returned JSON was not of the expected from: {reason}"))]
    Structure { reason: String },
}

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
pub async fn make_uncached_embedding_request(
    text: &str,
    openai_api_key: &str,
) -> Result<Vec<f32>, Error> {
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
        .context(RequestSnafu)?;

    let response_status = response.status();
    if !response_status.is_success() {
        eprintln!("Error performing request!");
        let response_text = response.text().await.unwrap();
        eprintln!("---- raw response ---\n{}\n", response_text);
        return HttpSnafu {
            response_text,
            status: response_status.as_u16(),
        }
        .fail();
    }

    let response_text = response.text().await.unwrap();
    let v: serde_json::Value = serde_json::from_str(&response_text).unwrap();

    let values = v["data"][0]["embedding"]
        .as_array()
        .with_context(|| StructureSnafu {
            reason: "no data[0].embedding entry",
        })?;

    let r: Option<Vec<f32>> = values
        .iter()
        .map(|v| v.as_f64().map(|v| v as f32))
        .collect();
    let r = r.with_context(|| StructureSnafu {
        reason: "no data[0].embedding contained non-numeric values",
    })?;
    Ok(r)
}
