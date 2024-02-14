use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String, // Usually JSON, but could be malformed or hallucinated
}

impl ToJson for ToolFunction {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "name": self.name,
            "arguments": self.arguments,
        })
    }
}

impl FromJson for ToolFunction {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        Ok(ToolFunction {
            name: v["name"].as_str().unwrap().to_string(),
            arguments: v["arguments"].as_str().unwrap().to_string(),
        })
    }
}

impl Generatable for ToolFunction {
    fn gen(context: &mut GeneratorContext) -> Self {
        ToolFunction {
            name: String::gen(context),
            arguments: String::gen(context),
        }
    }
}
