use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub struct JSONSchema(pub serde_json::Value);

impl ToJson for JSONSchema {
    fn to_json(&self) -> serde_json::Value {
        self.0.clone()
    }
}

impl FromJson for JSONSchema {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        Ok(JSONSchema(v.clone()))
    }
}

/// TODO: Make this nicer.
impl Generatable for JSONSchema {
    fn gen(_context: &mut GeneratorContext) -> Self {
        JSONSchema(json!({}))
    }
}
