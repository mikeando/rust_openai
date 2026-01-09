use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;

use rand::Rng;
use serde_json::json;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ModelId {
    Gpt5,
    Gpt5Mini,
    Gpt5Mini20250807,
    Gpt5Nano,
    Gpt5Pro,
    Gpt5_1,
    Gpt5_2,
    Gpt5ChatLatest,
    Gpt5_1ChatLatest,
    Gpt5_2ChatLatest,
    Gpt5Codex,
    Gpt5_1Codex,
    Gpt5_1CodexMax,
    Gpt5_1CodexMini,
    Gpt5_2Pro,
    Gpt5SearchApi,
}

impl ModelId {
    pub fn name(&self) -> String {
        match self {
            ModelId::Gpt5 => String::from("gpt-5"),
            ModelId::Gpt5Mini => String::from("gpt-5-mini"),
            ModelId::Gpt5Mini20250807 => String::from("gpt-5-mini-2025-08-07"),
            ModelId::Gpt5Nano => String::from("gpt-5-nano"),
            ModelId::Gpt5Pro => String::from("gpt-5-pro"),
            ModelId::Gpt5_1 => String::from("gpt-5.1"),
            ModelId::Gpt5_2 => String::from("gpt-5.2"),
            ModelId::Gpt5ChatLatest => String::from("gpt-5-chat-latest"),
            ModelId::Gpt5_1ChatLatest => String::from("gpt-5.1-chat-latest"),
            ModelId::Gpt5_2ChatLatest => String::from("gpt-5.2-chat-latest"),
            ModelId::Gpt5Codex => String::from("gpt-5-codex"),
            ModelId::Gpt5_1Codex => String::from("gpt-5.1-codex"),
            ModelId::Gpt5_1CodexMax => String::from("gpt-5.1-codex-max"),
            ModelId::Gpt5_1CodexMini => String::from("gpt-5.1-codex-mini"),
            ModelId::Gpt5_2Pro => String::from("gpt-5.2-pro"),
            ModelId::Gpt5SearchApi => String::from("gpt-5-search-api"),
        }
    }

    pub fn values() -> Vec<ModelId> {
        vec![
            ModelId::Gpt5,
            ModelId::Gpt5Mini,
            ModelId::Gpt5Mini20250807,
            ModelId::Gpt5Nano,
            ModelId::Gpt5Pro,
            ModelId::Gpt5_1,
            ModelId::Gpt5_2,
            ModelId::Gpt5ChatLatest,
            ModelId::Gpt5_1ChatLatest,
            ModelId::Gpt5_2ChatLatest,
            ModelId::Gpt5Codex,
            ModelId::Gpt5_1Codex,
            ModelId::Gpt5_1CodexMax,
            ModelId::Gpt5_1CodexMini,
            ModelId::Gpt5_2Pro,
            ModelId::Gpt5SearchApi,
        ]
    }

    pub fn from_str(name: &str) -> Result<ModelId, Error> {
        match name {
            "gpt-5" => Ok(ModelId::Gpt5),
            "gpt-5-mini" => Ok(ModelId::Gpt5Mini),
            "gpt-5-mini-2025-08-07" => Ok(ModelId::Gpt5Mini20250807),
            "gpt-5-nano" => Ok(ModelId::Gpt5Nano),
            "gpt-5-pro" => Ok(ModelId::Gpt5Pro),
            "gpt-5.1" => Ok(ModelId::Gpt5_1),
            "gpt-5.2" => Ok(ModelId::Gpt5_2),
            "gpt-5-chat-latest" => Ok(ModelId::Gpt5ChatLatest),
            "gpt-5.1-chat-latest" => Ok(ModelId::Gpt5_1ChatLatest),
            "gpt-5.2-chat-latest" => Ok(ModelId::Gpt5_2ChatLatest),
            "gpt-5-codex" => Ok(ModelId::Gpt5Codex),
            "gpt-5.1-codex" => Ok(ModelId::Gpt5_1Codex),
            "gpt-5.1-codex-max" => Ok(ModelId::Gpt5_1CodexMax),
            "gpt-5.1-codex-mini" => Ok(ModelId::Gpt5_1CodexMini),
            "gpt-5.2-pro" => Ok(ModelId::Gpt5_2Pro),
            "gpt-5-search-api" => Ok(ModelId::Gpt5SearchApi),
            _ => Err(Error::InvalidModelName),
        }
    }
}

impl FromJson for ModelId {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        ModelId::from_str(v.as_str().unwrap())
    }
}

impl ToJson for ModelId {
    fn to_json(&self) -> serde_json::Value {
        json!(self.name())
    }
}

impl Generatable for ModelId {
    fn gen(context: &mut GeneratorContext) -> Self {
        let values = Self::values();
        let i = context.rng.gen_range(0..values.len());
        values[i]
    }
}
