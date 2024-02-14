use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use rand::Rng;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub enum ResponseFormat {
    JSON,
    Text,
}

// OpenAI api spec says this should be one of
// { "type": "json_object" }
// { "type": "text"}
impl ToJson for ResponseFormat {
    fn to_json(&self) -> serde_json::Value {
        let v = match self {
            ResponseFormat::JSON => json!("json"),
            ResponseFormat::Text => json!("text"),
        };
        json!({"type": v})
    }
}

impl FromJson for ResponseFormat {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        match v["type"].as_str() {
            Some("json") => Ok(ResponseFormat::JSON),
            Some("text") => Ok(ResponseFormat::Text),
            _ => Err(Error::InvalidResponseFormat),
        }
    }
}

impl Generatable for ResponseFormat {
    fn gen(context: &mut GeneratorContext) -> Self {
        match context.rng.gen_range(0..=1) {
            0 => ResponseFormat::JSON,
            1 => ResponseFormat::Text,
            _ => unreachable!(),
        }
    }
}
