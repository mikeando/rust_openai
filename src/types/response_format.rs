use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::{Error, JsonSchemaProp};
use rand::Rng;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub enum ResponseFormat {
    JSON,
    Text,
    JsonSchema(JsonSchemaProp),
}

// OpenAI api spec says this should be one of
// { "type": "json_object" }
// { "type": "text"}
// { "type": "json_schema", "json_schema": { ... } }
impl ToJson for ResponseFormat {
    fn to_json(&self) -> serde_json::Value {
        match self {
            ResponseFormat::JSON => json!({ "type": "json_object" }),
            ResponseFormat::Text => json!({ "type": "text" }),
            ResponseFormat::JsonSchema(prop) => json!({
                "type": "json_schema",
                "json_schema": prop.to_json(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::{FromJson, ToJson};
    use crate::types::JSONSchema;
    use serde_json::json;

    #[test]
    fn test_response_format_to_json() {
        let format = ResponseFormat::JSON;
        assert_eq!(format.to_json(), json!({"type": "json_object"}));

        let format = ResponseFormat::Text;
        assert_eq!(format.to_json(), json!({"type": "text"}));

        let schema = JsonSchemaProp {
            name: "test".to_string(),
            description: None,
            schema: JSONSchema(json!({"type": "string"})),
            strict: None,
        };
        let format = ResponseFormat::JsonSchema(schema);
        assert_eq!(
            format.to_json(),
            json!({
                "type": "json_schema",
                "json_schema": {
                    "name": "test",
                    "schema": {
                        "type": "string"
                    }
                }
            })
        );
    }

    #[test]
    fn test_response_format_from_json() {
        let json = json!({"type": "json_object"});
        let format = ResponseFormat::from_json(&json).unwrap();
        assert_eq!(format, ResponseFormat::JSON);

        let json = json!({"type": "text"});
        let format = ResponseFormat::from_json(&json).unwrap();
        assert_eq!(format, ResponseFormat::Text);

        let json = json!({
            "type": "json_schema",
            "json_schema": {
                "name": "test",
                "schema": {
                    "type": "string"
                }
            }
        });
        let format = ResponseFormat::from_json(&json).unwrap();
        let expected_schema = JsonSchemaProp {
            name: "test".to_string(),
            description: None,
            schema: JSONSchema(json!({"type": "string"})),
            strict: None,
        };
        assert_eq!(format, ResponseFormat::JsonSchema(expected_schema));
    }
}

impl FromJson for ResponseFormat {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        match v.get("type").and_then(|t| t.as_str()) {
            Some("json_object") => Ok(ResponseFormat::JSON),
            Some("text") => Ok(ResponseFormat::Text),
            Some("json_schema") => {
                let prop = JsonSchemaProp::from_json(&v["json_schema"])?;
                Ok(ResponseFormat::JsonSchema(prop))
            }
            _ => Err(Error::InvalidResponseFormat),
        }
    }
}

impl Generatable for ResponseFormat {
    fn gen(context: &mut GeneratorContext) -> Self {
        match context.rng.gen_range(0..=2) {
            0 => ResponseFormat::JSON,
            1 => ResponseFormat::Text,
            2 => ResponseFormat::JsonSchema(JsonSchemaProp::gen(context)),
            _ => unreachable!(),
        }
    }
}