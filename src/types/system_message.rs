use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub struct SystemMessage {
    pub content: String,
    pub name: Option<String>,
}

impl SystemMessage {
    pub fn new<T: Into<String>>(content: T) -> SystemMessage {
        SystemMessage {
            content: content.into(),
            name: None,
        }
    }
}

impl ToJson for SystemMessage {
    fn to_json(&self) -> serde_json::Value {
        let mut v = json!({
            "role": "system",
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

impl FromJson for SystemMessage {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        Ok(SystemMessage {
            content: v["content"].as_str().unwrap().to_string(),
            name: v["name"].as_str().map(|s| s.to_string()),
        })
    }
}

impl Generatable for SystemMessage {
    fn gen(context: &mut GeneratorContext) -> Self {
        SystemMessage {
            content: context.gen(),
            name: context.gen(),
        }
    }
}
