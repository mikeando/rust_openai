use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub struct ToolMessage {
    content: String,
    tool_call_id: String,
}
impl ToJson for ToolMessage {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "role": "tool",
            "content": self.content,
            "tool_call_id": self.tool_call_id,
        })
    }
}

impl FromJson for ToolMessage {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        Ok(ToolMessage {
            content: v["content"].as_str().unwrap().to_string(),
            tool_call_id: v["tool_call_id"].as_str().unwrap().to_string(),
        })
    }
}

impl Generatable for ToolMessage {
    fn gen(context: &mut GeneratorContext) -> Self {
        ToolMessage {
            content: context.gen(),
            tool_call_id: context.gen(),
        }
    }
}
