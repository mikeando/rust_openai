use crate::json::{FromJson, ToJson};
use crate::types::Error;
use serde_json::json;

/// A `function_call` input item for the Responses API.
///
/// Used to replay a previous tool call in the conversation history.
/// The Responses API does not accept Chat Completions `tool_calls` on
/// assistant messages; each prior tool invocation must appear as a
/// separate `function_call` item in the `input` array.
///
/// Note: the linking identifier is `call_id` (not `id`) — `call_id` is
/// what the corresponding `function_call_output` item references.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallItem {
    pub call_id: String,
    pub name: String,
    pub arguments: String,
}

impl ToJson for FunctionCallItem {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "type": "function_call",
            "call_id": self.call_id,
            "name": self.name,
            "arguments": self.arguments,
        })
    }
}

impl FromJson for FunctionCallItem {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        let call_id = v["call_id"].as_str()
            .or_else(|| v["id"].as_str())  // tolerate old `id` field
            .unwrap_or("")
            .to_string();
        Ok(FunctionCallItem {
            call_id,
            name: v["name"].as_str().unwrap_or("").to_string(),
            arguments: v["arguments"].as_str().unwrap_or("{}").to_string(),
        })
    }
}
