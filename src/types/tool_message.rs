use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub struct ToolMessage {
    pub content: String,
    pub tool_call_id: String,
}
impl ToJson for ToolMessage {
    fn to_json(&self) -> serde_json::Value {
        // Responses API format: function_call_output item (not role:"tool")
        json!({
            "type": "function_call_output",
            "call_id": self.tool_call_id,
            "output": self.content,
        })
    }
}

impl FromJson for ToolMessage {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        // Support both Responses API format (type/call_id/output) and legacy (role/tool_call_id/content)
        let content = v["output"].as_str()
            .or_else(|| v["content"].as_str())
            .unwrap_or("")
            .to_string();
        let tool_call_id = v["call_id"].as_str()
            .or_else(|| v["tool_call_id"].as_str())
            .unwrap_or("")
            .to_string();
        Ok(ToolMessage { content, tool_call_id })
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
