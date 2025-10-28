use crate::{
    generate::{Generatable, GeneratorContext},
    json::{FromJson, ToJson},
    types::{ChatCompletionChoice, Error, ModelId, UsageStats},
};

use rand::Rng;
use serde_json::{json, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct ChatCompletionObject {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: ModelId,
    pub choices: Vec<ChatCompletionChoice>,
    pub system_fingerprint: Option<String>,
    pub usage: UsageStats,
}

impl ToJson for ChatCompletionObject {
    fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "object": self.object,
            "created": self.created,
            "model": self.model.to_json(),
            "choices": self.choices.iter().map(|c| c.to_json()).collect::<Vec<Value>>(),
            "system_fingerprint": self.system_fingerprint,
            "usage": self.usage.to_json(),
        })
    }
}

impl FromJson for ChatCompletionObject {
    fn from_json(value: &Value) -> Result<Self, Error> {
        Ok(Self {
            id: value["id"].as_str().unwrap().to_string(),
            object: value["object"].as_str().unwrap().to_string(),
            created: value["created"].as_i64().unwrap(),
            model: ModelId::from_json(&value["model"])?,
            choices: value["choices"]
                .as_array()
                .unwrap()
                .iter()
                .map(ChatCompletionChoice::from_json)
                .collect::<Result<Vec<ChatCompletionChoice>, Error>>()?,
            system_fingerprint: value["system_fingerprint"].as_str().map(|s| s.to_string()),
            usage: UsageStats::from_json(&value["usage"])?,
        })
    }
}

impl Generatable for ChatCompletionObject {
    fn gen(context: &mut GeneratorContext) -> Self {
        Self {
            id: String::gen(context),
            object: "chat.completion".to_string(),
            created: context.rng.gen(),
            model: ModelId::gen(context),
            choices: vec![ChatCompletionChoice::gen(context), ChatCompletionChoice::gen(context)],
            system_fingerprint: Some(String::gen(context)),
            usage: UsageStats::gen(context),
        }
    }
}
