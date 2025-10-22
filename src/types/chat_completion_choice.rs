use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub struct ChatCompletionChoice {
    pub id: Option<String>,
    pub output_type: Option<String>,
    pub status: Option<String>,
    pub name: Option<String>,
    pub arguments: Option<String>,
    pub content: Option<serde_json::Value>,
    pub summary: Option<serde_json::Value>,
    pub role: Option<String>,
    // Add other fields as needed for reasoning, etc.
}

impl FromJson for ChatCompletionChoice {
    fn from_json(v: &serde_json::Value) -> Result<ChatCompletionChoice, Error> {
        Ok(ChatCompletionChoice {
            id: v.get("id").and_then(|x| x.as_str()).map(|s| s.to_string()),
            output_type: v
                .get("type")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            status: v
                .get("status")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            name: v
                .get("name")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            arguments: v
                .get("arguments")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
            content: v.get("content").cloned(),
            summary: v.get("summary").cloned(),
            role: v
                .get("role")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string()),
        })
    }
}

impl ToJson for ChatCompletionChoice {
    fn to_json(&self) -> serde_json::Value {
        let mut v = serde_json::Map::new();
        if let Some(id) = &self.id {
            v.insert("id".to_string(), json!(id));
        }
        if let Some(output_type) = &self.output_type {
            v.insert("type".to_string(), json!(output_type));
        }
        if let Some(status) = &self.status {
            v.insert("status".to_string(), json!(status));
        }
        if let Some(name) = &self.name {
            v.insert("name".to_string(), json!(name));
        }
        if let Some(arguments) = &self.arguments {
            v.insert("arguments".to_string(), json!(arguments));
        }
        if let Some(content) = &self.content {
            v.insert("content".to_string(), content.clone());
        }
        if let Some(summary) = &self.summary {
            v.insert("summary".to_string(), summary.clone());
        }
        if let Some(role) = &self.role {
            v.insert("role".to_string(), json!(role));
        }
        serde_json::Value::Object(v)
    }
}

impl Generatable for ChatCompletionChoice {
    fn gen(context: &mut GeneratorContext) -> Self {
        ChatCompletionChoice {
            id: Some(String::gen(context)),
            output_type: Some("function_call".to_string()),
            status: Some("completed".to_string()),
            name: Some("test_function".to_string()),
            arguments: Some("{}".to_string()),
            content: Some(json!("Hello!")),
            summary: Some(json!([])),
            role: Some("assistant".to_string()),
        }
    }
}
