use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;

use rand::Rng;
use serde_json::json;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ModelId {
    Gpt5,
    Gpt5Mini,
    Gpt5Nano,
    Gpt5Pro,
}

impl ModelId {
    pub fn name(&self) -> String {
        match self {
            ModelId::Gpt5 => String::from("gpt-5"),
            ModelId::Gpt5Mini => String::from("gpt-5-mini"),
            ModelId::Gpt5Nano => String::from("gpt-5-nano"),
            ModelId::Gpt5Pro => String::from("gpt-5-pro"),
        }
    }

    pub fn values() -> Vec<ModelId> {
        vec![
            ModelId::Gpt5,
            ModelId::Gpt5Mini,
            ModelId::Gpt5Nano,
            ModelId::Gpt5Pro,
        ]
    }

    pub fn from_str(name: &str) -> Result<ModelId, Error> {
        match name {
            "gpt-5" => Ok(ModelId::Gpt5),
            "gpt-5-mini" => Ok(ModelId::Gpt5Mini),
            "gpt-5-nano" => Ok(ModelId::Gpt5Nano),
            "gpt-5-pro" => Ok(ModelId::Gpt5Pro),
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
