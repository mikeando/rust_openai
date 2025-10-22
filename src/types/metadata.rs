use crate::json::{FromJson, ToJson};
use serde_json::{json, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct Metadata {
    // Add fields as needed
}

impl ToJson for Metadata {
    fn to_json(&self) -> Value {
        json!({})
    }
}

impl FromJson for Metadata {
    fn from_json(_value: &Value) -> Result<Self, crate::types::Error> {
        Ok(Self {})
    }
}
