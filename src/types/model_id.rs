use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;

use rand::Rng;
use serde_json::json;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ModelId {
    pub name: String,
}

impl ModelId {
    pub fn new(name: &str) -> ModelId {
        ModelId {
            name: name.to_string(),
        }
    }
}

impl FromJson for ModelId {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        Ok(ModelId {
            name: v.as_str().unwrap().to_string(),
        })
    }
}

impl ToJson for ModelId {
    fn to_json(&self) -> serde_json::Value {
        json!(self.name)
    }
}

impl Generatable for ModelId {
    fn gen(context: &mut GeneratorContext) -> Self {
        let values = &["gpt-3.5-turbo", "gpt-4o", "claude-2", "grok-1"];
        let i = context.rng.gen_range(0..values.len());
        ModelId::new(values[i])
    }
}
