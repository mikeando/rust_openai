use serde_json::json;
use snafu::{OptionExt, ResultExt, Snafu};

use crate::json::ToJson;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not make http request to OpenAI"))]
    Request { error: Box<ureq::Error> },

    #[snafu(display("Bad HTTP response status from OpenAI ({status})"))]
    Http { response_text: String, status: u16 },

    #[snafu(display("Returned JSON was not of the expected from: {reason}"))]
    Structure { reason: String },

    #[snafu(display("Could not parse JSON response"))]
    Json { source: std::io::Error },
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
pub fn make_uncached_embedding_request(
    text: &str,
    openai_api_key: &str,
) -> Result<Vec<f32>, Error> {
    let request = EmbeddingRequest {
        input: text.to_string(),
        model: "text-embedding-3-small".to_string(),
    };

    let response = match ureq::post("https://api.openai.com/v1/embeddings")
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {}", openai_api_key))
        .send_json(request.to_json())
    {
        Ok(r) => r,
        Err(ureq::Error::Status(status, response)) => {
            return HttpSnafu {
                response_text: response.into_string().unwrap_or_default(),
                status,
            }
            .fail()
        }
        Err(e) => return RequestSnafu { error: e }.fail(),
    };

    let v: serde_json::Value = response.into_json().context(JsonSnafu)?;

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
