use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub struct UserMessage {
    pub content: String,
    pub name: Option<String>,
}

impl UserMessage {
    pub fn new<T: Into<String>>(content: T) -> UserMessage {
        UserMessage {
            content: content.into(),
            name: None,
        }
    }
}

impl ToJson for UserMessage {
    fn to_json(&self) -> serde_json::Value {
        let mut v = json!({
            "role": "user",
            "content": self.content
        });
        if let Some(name) = &self.name {
            v.as_object_mut()
                .unwrap()
                .insert("name".to_string(), json!(name));
        }
        v
    }
}

impl FromJson for UserMessage {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        Ok(UserMessage {
            content: v["content"].as_str().unwrap().to_string(),
            name: v["name"].as_str().map(|s| s.to_string()),
        })
    }
}

impl Generatable for UserMessage {
    fn gen(context: &mut GeneratorContext) -> Self {
        UserMessage {
            content: context.gen(),
            name: context.gen(),
        }
    }
}
