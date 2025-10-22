use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use rand::Rng;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub struct UsageStats {
    pub input_tokens: u32,
    pub input_tokens_details: Option<InputTokensDetails>,
    pub output_tokens: u32,
    pub output_tokens_details: Option<OutputTokensDetails>,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputTokensDetails {
    pub cached_tokens: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputTokensDetails {
    pub reasoning_tokens: Option<u32>,
}

impl FromJson for UsageStats {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        Ok(UsageStats {
            input_tokens: v["input_tokens"].as_i64().unwrap() as u32,
            input_tokens_details: v.get("input_tokens_details").map(|d| InputTokensDetails {
                cached_tokens: d
                    .get("cached_tokens")
                    .and_then(|x| x.as_i64())
                    .map(|x| x as u32),
            }),
            output_tokens: v["output_tokens"].as_i64().unwrap() as u32,
            output_tokens_details: v.get("output_tokens_details").map(|d| OutputTokensDetails {
                reasoning_tokens: d
                    .get("reasoning_tokens")
                    .and_then(|x| x.as_i64())
                    .map(|x| x as u32),
            }),
            total_tokens: v["total_tokens"].as_i64().unwrap() as u32,
        })
    }
}

impl ToJson for UsageStats {
    fn to_json(&self) -> serde_json::Value {
        let mut obj = serde_json::Map::new();
        obj.insert("input_tokens".to_string(), json!(self.input_tokens));
        if let Some(ref details) = self.input_tokens_details {
            let mut details_obj = serde_json::Map::new();
            if let Some(cached) = details.cached_tokens {
                details_obj.insert("cached_tokens".to_string(), json!(cached));
            }
            obj.insert(
                "input_tokens_details".to_string(),
                serde_json::Value::Object(details_obj),
            );
        }
        obj.insert("output_tokens".to_string(), json!(self.output_tokens));
        if let Some(ref details) = self.output_tokens_details {
            let mut details_obj = serde_json::Map::new();
            if let Some(reasoning) = details.reasoning_tokens {
                details_obj.insert("reasoning_tokens".to_string(), json!(reasoning));
            }
            obj.insert(
                "output_tokens_details".to_string(),
                serde_json::Value::Object(details_obj),
            );
        }
        obj.insert("total_tokens".to_string(), json!(self.total_tokens));
        serde_json::Value::Object(obj)
    }
}

impl Generatable for UsageStats {
    fn gen(context: &mut GeneratorContext) -> Self {
        UsageStats {
            input_tokens: context.rng.gen(),
            input_tokens_details: Some(InputTokensDetails {
                cached_tokens: Some(context.rng.gen()),
            }),
            output_tokens: context.rng.gen(),
            output_tokens_details: Some(OutputTokensDetails {
                reasoning_tokens: Some(context.rng.gen()),
            }),
            total_tokens: context.rng.gen(),
        }
    }
}
