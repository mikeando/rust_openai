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
        let f = v["function"].as_object().ok_or(Error::InvalidJsonStructure)?;
        Ok(Tool {
            description: f.get("description")
                .map(|v| v.to_opt_string())
                .transpose()?
                .flatten(),
            name: f.get("name")
                .and_then(|v| v.as_str())
                .ok_or(Error::InvalidJsonStructure)?
                .to_string(),
            parameters: f.get("parameters")
                .map(|v| JSONSchema::from_json(v))
                .transpose()?,
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
