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
                "name": self.name,
            },
            "type": "function",
        });
        if let Some(description) = &self.description {
            v["function"]
                .as_object_mut()
                .unwrap()
                .insert("description".to_string(), json!(description));
        }
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
        let f = v
            .get("function")
            .and_then(|v| v.as_object())
            .with_context(|| "missing function in tool")?;
        Ok(Tool {
            description: f
                .get("description")
                .unwrap_or(&serde_json::Value::Null)
                .to_opt_string()
                .with_context(|| "invalid function.description in tool")?,
            name: f
                .get("name")
                .and_then(|v| v.as_str())
                .context("missing or invalid function.name field in tool")?
                .to_string(),
            parameters: f
                .get("parameters")
                .unwrap_or(&serde_json::Value::Null)
                .map_opt(JSONSchema::from_json)
                .context("invalid function.parameters field in tool")?,
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


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::generate::GeneratorContext;

    #[test]
    fn test_tool_ping_pong() {
        let tool = Tool {
            description: Some("A test tool".to_string()),
            name: "test_tool".to_string(),
            parameters: Some(JSONSchema(json!({"type": "object"}))),
        };
        let value = tool.to_json();
        let tool_copy = Tool::from_json(&value).unwrap();
        assert_eq!(tool, tool_copy);
    }

    #[test]
    fn test_tool_ping_pong_no_description() {
        let tool = Tool {
            description: None,
            name: "test_tool".to_string(),
            parameters: Some(JSONSchema(json!({"type": "object"}))),
        };
        let value = tool.to_json();
        let tool_copy = Tool::from_json(&value).unwrap();
        assert_eq!(tool, tool_copy);
    }

    #[test]
    fn test_tool_ping_pong_no_schema() {
        let tool = Tool {
            description: Some("No schema".to_string()),
            name: "test_tool".to_string(),
            parameters: None,
        };
        let value = tool.to_json();
        let tool_copy = Tool::from_json(&value).unwrap();
        assert_eq!(tool, tool_copy);
    }

    #[test]
    fn test_deserialize_missing_name() {
        let v = json!({
            "type": "function",
            "function": {
                "description": "A tool without a name",
                "parameters": { "type": "object" }
            }
        });
        let err = Tool::from_json(&v).unwrap_err();
        assert!(err.to_string().contains("missing or invalid function.name field in tool"));
    }

    #[test]
    fn test_deserialize_missing_description() {
        let v = json!({
            "type": "function",
            "function": {
                "name": "test_tool",
                "parameters": { "type": "object" }
            }
        });
        let tool = Tool::from_json(&v).unwrap();
        assert_eq!(tool.description, None);
    }

    #[test]
    fn test_deserialize_missing_parameters() {
        let v = json!({
            "type": "function",
            "function": {
                "name": "test_tool",
                "description": "A tool without parameters"
            }
        });
        let tool = Tool::from_json(&v).unwrap();
        assert_eq!(tool.parameters, None);
    }


    #[test]
    fn test_tool_generatable() {
        let mut ctx = GeneratorContext::new();

        for _i in 0..10 {
            let tool = Tool::gen(&mut ctx);

            // Check we can serialize and deserialize the generated tool
            let json_value = tool.to_json();
            let deserialized_tool = Tool::from_json(&json_value).unwrap();
            assert_eq!(tool, deserialized_tool);
        }
    }
}