use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use crate::types::{ChatCompletionChoice, UsageStats};
use rand::Rng;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub struct ChatCompletionObject {
    pub id: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub created: i64,
    pub model: String,
    pub system_fingerprint: Option<String>,
    pub object: String,
    pub usage: UsageStats,
}
impl FromJson for ChatCompletionObject {
    fn from_json(v: &serde_json::Value) -> Result<ChatCompletionObject, Error> {
        if !v.is_object() {
            return Err(Error::InvalidJsonStructure);
        }
        let id = v.get("id").unwrap().as_str().unwrap().to_string();
        let choices = v
            .get("choices")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(ChatCompletionChoice::from_json)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let created = v.get("created").unwrap().as_i64().unwrap();
        let model = v.get("model").unwrap().as_str().unwrap().to_string();
        let system_fingerprint = v
            .get("system_fingerprint")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let object = v.get("object").unwrap().as_str().unwrap().to_string();

        let usage = UsageStats {
            prompt_tokens: v["usage"]["prompt_tokens"].as_i64().unwrap() as u32,
            completion_tokens: v["usage"]["completion_tokens"].as_i64().unwrap() as u32,
            total_tokens: v["usage"]["total_tokens"].as_i64().unwrap() as u32,
        };

        Ok(ChatCompletionObject {
            id,
            choices,
            created,
            model,
            system_fingerprint,
            object,
            usage,
        })
    }
}

impl ToJson for ChatCompletionObject {
    fn to_json(&self) -> serde_json::Value {
        let choices: Vec<serde_json::Value> = self.choices.iter().map(|v| v.to_json()).collect();
        let mut v = json!({
            "id": self.id,
            "choices": choices,
            "created": self.created,
            "model": self.model,
            "object": self.object,
            "usage": self.usage.to_json(),
        });
        if let Some(system_fingerprint) = &self.system_fingerprint {
            v["system_fingerprint"] = json!(system_fingerprint);
        }
        v
    }
}

impl Generatable for ChatCompletionObject {
    fn gen(context: &mut GeneratorContext) -> Self {
        let choices = (0..context.rng.gen_range(0..4))
            .map(|_| ChatCompletionChoice::gen(context))
            .collect();
        ChatCompletionObject {
            id: context.gen(),
            choices,
            created: context.rng.gen(),
            model: context.gen(),
            system_fingerprint: context.gen(),
            object: context.gen(),
            usage: context.gen(),
        }
    }
}
