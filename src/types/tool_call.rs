use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use crate::types::ToolFunction;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall {
    pub id: String,
    // type: String - always "function"
    pub function: ToolFunction,
}

impl ToJson for ToolCall {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "function": self.function.to_json(),
        })
    }
}

impl FromJson for ToolCall {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        Ok(ToolCall {
            id: v["id"].as_str().unwrap().to_string(),
            function: ToolFunction::from_json(&v["function"])?,
        })
    }
}

impl Generatable for ToolCall {
    fn gen(context: &mut GeneratorContext) -> Self {
        ToolCall {
            id: String::gen(context),
            function: ToolFunction::gen(context),
        }
    }
}
