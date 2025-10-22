use crate::json::{FromJson, ToJson};
use serde_json::{json, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct TextFormat {
    pub format: Option<TextFormatType>,
    pub verbosity: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TextFormatType {
    pub r#type: Option<String>,
}

impl ToJson for TextFormat {
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        if let Some(format) = &self.format {
            obj.insert("format".to_string(), format.to_json());
        }
        if let Some(verbosity) = &self.verbosity {
            obj.insert("verbosity".to_string(), json!(verbosity));
        }
        Value::Object(obj)
    }
}

impl FromJson for TextFormat {
    fn from_json(value: &Value) -> Result<Self, crate::types::Error> {
        Ok(Self {
            format: value
                .get("format")
                .map(|v| TextFormatType::from_json(v).unwrap()),
            verbosity: value
                .get("verbosity")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        })
    }
}

impl ToJson for TextFormatType {
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        if let Some(t) = &self.r#type {
            obj.insert("type".to_string(), json!(t));
        }
        Value::Object(obj)
    }
}

impl FromJson for TextFormatType {
    fn from_json(value: &Value) -> Result<Self, crate::types::Error> {
        Ok(Self {
            r#type: value
                .get("type")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        })
    }
}
