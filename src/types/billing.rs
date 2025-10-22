use crate::json::{FromJson, ToJson};
use serde_json::{json, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct Billing {
    pub payer: Option<String>,
}

impl ToJson for Billing {
    fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        if let Some(payer) = &self.payer {
            obj.insert("payer".to_string(), json!(payer));
        }
        Value::Object(obj)
    }
}

impl FromJson for Billing {
    fn from_json(value: &Value) -> Result<Self, crate::types::Error> {
        Ok(Self {
            payer: value
                .get("payer")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        })
    }
}
