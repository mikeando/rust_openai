use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::json_ext::JsonValueExt;
use crate::types::Error;
use crate::types::JSONSchema;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub struct Tool {
    // type: String = "function"
    pub description: Option<String>,
    pub name: String,
    pub parameters: Option<JSONSchema>,
}

impl ToJson for Tool {
    fn to_json(&self) -> serde_json::Value {
        let mut v = json!({
            "function": {
                "description": self.description,
                "name": self.name,
            },
            "type": "function",
        });
        if let Some(parameters) = &self.parameters {
            v["function"]
                .as_object_mut()
                .unwrap()
                .insert("parameters".to_string(), parameters.to_json());
        }
        v
    }
}

impl FromJson for Tool {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        Ok(Tool {
            description: v["function"]["description"].to_opt_string()?,
            name: v["function"]["name"].as_str().unwrap().to_string(),
            parameters: v["function"]["parameters"].map_opt(JSONSchema::from_json)?,
        })
    }
}

impl Generatable for Tool {
    fn gen(context: &mut GeneratorContext) -> Self {
        Tool {
            description: context.gen(),
            name: context.gen(),
            parameters: context.gen(),
        }
    }
}
