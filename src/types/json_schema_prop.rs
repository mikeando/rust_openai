use crate::generate::{gen_opt, Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::json_ext::JsonValueExt;
use crate::types::{Error, JSONSchema};
use serde_json::json;

/// Represents the properties of a JSON schema response format.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonSchemaProp {
    /// The name of the response format. Must be a-z, A-Z, 0-9, or contain underscores and hyphens, with a maximum length of 64.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::FromJson;
    use serde_json::json;

    #[test]
    fn test_json_schema_prop_from_json() {
        let json = json!({
            "name": "test",
            "description": "a test schema",
            "schema": {
                "type": "object",
                "properties": {
                    "foo": {
                        "type": "string"
                    }
                }
            },
            "strict": true
        });

        let prop = JsonSchemaProp::from_json(&json).unwrap();
        assert_eq!(prop.name, "test");
        assert_eq!(prop.description, Some("a test schema".to_string()));
        assert_eq!(prop.strict, Some(true));
    }
}