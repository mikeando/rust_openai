use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::json_ext::JsonValueExt;
use crate::types::Error;
use crate::types::JSONSchema;
use anyhow::Context;
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
        let f = v["function"]
            .as_object()
            .with_context(|| "missing function in tool")?;
        Ok(Tool {
            description: f["description"]
                .to_opt_string()
                .with_context(|| "missing or invalid function.description in tool")?,
            name: f["name"]
                .as_str()
                .context("missing or invalid function.name field in tool")?
                .to_string(),
            parameters: f["parameters"]
                .map_opt(JSONSchema::from_json)
                .context("missing or invalid function.parameters field in tool")?,
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
