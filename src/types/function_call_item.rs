use crate::json::{FromJson, ToJson};
use crate::types::Error;
use serde_json::json;

/// A `function_call` input item for the Responses API.
///
/// Used to replay a previous tool call in the conversation history.
/// The Responses API does not accept Chat Completions `tool_calls` on
/// assistant messages; each prior tool invocation must appear as a
/// separate `function_call` item in the `input` array.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallItem {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

impl ToJson for FunctionCallItem {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "type": "function_call",
            "id": self.id,
            "name": self.name,
            "arguments": self.arguments,
        })
    }
}

impl FromJson for FunctionCallItem {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        Ok(FunctionCallItem {
            id: v["id"].as_str().unwrap_or("").to_string(),
            name: v["name"].as_str().unwrap_or("").to_string(),
            arguments: v["arguments"].as_str().unwrap_or("{}").to_string(),
        })
    }
}
