use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use rand::Rng;
use serde_json::json;

use crate::types::ToolCall;

#[derive(Debug, Clone, PartialEq)]
pub struct AssistantMessage {
    pub content: Option<String>,
    pub name: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl ToJson for AssistantMessage {
    fn to_json(&self) -> serde_json::Value {
        let mut v = json!({"role":"assistant"});
        if let Some(content) = &self.content {
            v["content"] = json!(content);
        }
        if let Some(name) = &self.name {
            v["name"] = json!(name);
        }
        if let Some(tool_calls) = &self.tool_calls {
            let tool_calls = tool_calls.iter().map(|t| t.to_json()).collect::<Vec<_>>();
            v["tool_calls"] = json!(tool_calls);
        }
        v
    }
}

impl FromJson for AssistantMessage {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        let tool_calls = v["tool_calls"].as_array().map(|a| {
            a.iter()
                .map(ToolCall::from_json)
                .collect::<Result<Vec<_>, _>>()
        });
        let tool_calls = match tool_calls {
            None => None,
            Some(v) => Some(v.unwrap()),
        };
        Ok(AssistantMessage {
            name: v["name"].as_str().map(|s| s.to_string()),
            content: v["content"].as_str().map(|s| s.to_string()),
            tool_calls,
        })
    }
}

impl Generatable for AssistantMessage {
    fn gen(context: &mut GeneratorContext) -> Self {
        // are there tool calls
        let tool_calls = match context.rng.gen_bool(0.2) {
            true => {
                // How many
                let n_calls = context.rng.gen_range(0..=4);
                Some((0..n_calls).map(|_| ToolCall::gen(context)).collect())
            }
            false => None,
        };

        AssistantMessage {
            content: context.gen(),
            name: context.gen(),
            tool_calls,
        }
    }
}
