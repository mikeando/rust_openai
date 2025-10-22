use crate::json::{FromJson, ToJson};
use serde_json::{json, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct Reasoning {
    pub effort: Option<String>,
    pub summary: Option<String>,
}

impl ToJson for Reasoning {
    fn to_json(&self) -> Value {
        json!({
            "effort": self.effort,
            "summary": self.summary,
        })
    }
}

impl FromJson for Reasoning {
    fn from_json(value: &Value) -> Result<Self, crate::types::Error> {
        Ok(Self {
            effort: value
                .get("effort")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            summary: value
                .get("summary")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        })
    }
}
