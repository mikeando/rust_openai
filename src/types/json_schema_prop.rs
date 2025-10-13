use crate::generate::{gen_opt, Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::json_ext::JsonValueExt;
use crate::types::{Error, JSONSchema};
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub struct JsonSchemaProp {
    pub name: String,
    pub description: Option<String>,
    pub schema: JSONSchema,
    pub strict: Option<bool>,
}

impl ToJson for JsonSchemaProp {
    fn to_json(&self) -> serde_json::Value {
        let mut v = serde_json::Map::new();
        v.insert("name".to_string(), json!(self.name));
        if let Some(description) = &self.description {
            v.insert("description".to_string(), json!(description));
        }
        v.insert("schema".to_string(), self.schema.to_json());
        if let Some(strict) = self.strict {
            v.insert("strict".to_string(), json!(strict));
        }
        json!(v)
    }
}

impl FromJson for JsonSchemaProp {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        Ok(JsonSchemaProp {
            name: v["name"].to_string_or_err()?,
            description: v["description"].to_opt_string()?,
            schema: JSONSchema::from_json(&v["schema"])?,
            strict: v["strict"].to_opt_bool()?,
        })
    }
}

impl Generatable for JsonSchemaProp {
    fn gen(context: &mut GeneratorContext) -> Self {
        JsonSchemaProp {
            name: context.gen(),
            description: gen_opt(context, 0.5),
            schema: context.gen(),
            strict: gen_opt(context, 0.5),
        }
    }
}