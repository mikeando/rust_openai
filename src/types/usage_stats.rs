use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use rand::Rng;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub struct UsageStats {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl FromJson for UsageStats {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        Ok(UsageStats {
            prompt_tokens: v["prompt_tokens"].as_i64().unwrap() as u32,
            completion_tokens: v["completion_tokens"].as_i64().unwrap() as u32,
            total_tokens: v["total_tokens"].as_i64().unwrap() as u32,
        })
    }
}

impl ToJson for UsageStats {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "prompt_tokens": self.prompt_tokens,
            "completion_tokens": self.completion_tokens,
            "total_tokens": self.total_tokens,
        })
    }
}

impl Generatable for UsageStats {
    fn gen(context: &mut GeneratorContext) -> Self {
        UsageStats {
            prompt_tokens: context.rng.gen(),
            completion_tokens: context.rng.gen(),
            total_tokens: context.rng.gen(),
        }
    }
}
