use crate::{
    generate::{Generatable, GeneratorContext},
    json::{FromJson, ToJson},
    types::{Billing, Metadata, Reasoning, TextFormat},
    types::{ChatCompletionChoice, Error, ModelId, Tool, UsageStats},
};

use rand::Rng;
use serde_json::{json, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct ChatCompletionObject {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub model: ModelId,
    pub output: Vec<ChatCompletionChoice>,
    pub system_fingerprint: Option<String>,
    pub usage: UsageStats,
    pub instructions: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
    pub previous_response_id: Option<String>,
    pub user: Option<String>,
    pub tool_choice: Option<String>,
    pub tools: Option<Vec<Tool>>,
    pub max_output_tokens: Option<u32>,
    pub max_tool_calls: Option<u32>,
    pub parallel_tool_calls: Option<bool>,
    pub store: Option<bool>,
    pub background: Option<bool>,
    pub service_tier: Option<String>,
    pub billing: Option<Billing>,
    pub prompt_cache_key: Option<String>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_logprobs: Option<u32>,
    pub truncation: Option<String>,
    pub metadata: Option<Metadata>,
    pub reasoning: Option<Reasoning>,
    pub safety_identifier: Option<String>,
    pub text: Option<TextFormat>,
    pub incomplete_details: Option<String>,
}

impl ToJson for ChatCompletionObject {
    fn to_json(&self) -> Value {
        let mut v = json!({
          "id": self.id,
          "object": self.object,
          "created_at": self.created_at,
          "model": self.model.to_json(),
          "output": self.output.iter().map(|c| c.to_json()).collect::<Vec<Value>>(),
          "usage": self.usage.to_json(),
          "error": self.error,
          "incomplete_details": self.incomplete_details,
          "max_output_tokens": self.max_output_tokens,
          "max_tool_calls": self.max_tool_calls,
          "previous_response_id": self.previous_response_id,
          "prompt_cache_key": self.prompt_cache_key,
          "safety_identifier": self.safety_identifier,
          "user": self.user,
        });
        macro_rules! opt_insert {
            ($field:expr, $name:expr) => {
                if let Some(val) = &$field {
                    v.as_object_mut()
                        .unwrap()
                        .insert($name.to_string(), json!(val));
                }
            };
        }
        opt_insert!(self.system_fingerprint, "system_fingerprint");
        opt_insert!(self.instructions, "instructions");
        opt_insert!(self.status, "status");
        opt_insert!(self.tool_choice, "tool_choice");
        if let Some(tools) = &self.tools {
            v.as_object_mut().unwrap().insert(
                "tools".to_string(),
                json!(tools.iter().map(|t| t.to_json()).collect::<Vec<Value>>()),
            );
        }
        opt_insert!(self.parallel_tool_calls, "parallel_tool_calls");
        opt_insert!(self.store, "store");
        opt_insert!(self.background, "background");
        opt_insert!(self.service_tier, "service_tier");
        if let Some(billing) = &self.billing {
            v.as_object_mut()
                .unwrap()
                .insert("billing".to_string(), billing.to_json());
        }
        opt_insert!(self.temperature, "temperature");
        opt_insert!(self.top_p, "top_p");
        opt_insert!(self.top_logprobs, "top_logprobs");
        opt_insert!(self.truncation, "truncation");
        if let Some(metadata) = &self.metadata {
            v.as_object_mut()
                .unwrap()
                .insert("metadata".to_string(), metadata.to_json());
        }
        if let Some(reasoning) = &self.reasoning {
            v.as_object_mut()
                .unwrap()
                .insert("reasoning".to_string(), reasoning.to_json());
        }
        if let Some(text) = &self.text {
            v.as_object_mut()
                .unwrap()
                .insert("text".to_string(), text.to_json());
        }
        v
    }
}

impl FromJson for ChatCompletionObject {
    fn from_json(value: &Value) -> Result<Self, Error> {
        Ok(Self {
            id: value["id"].as_str().unwrap().to_string(),
            object: value["object"].as_str().unwrap().to_string(),
            created_at: value["created_at"].as_i64().unwrap(),
            model: ModelId::from_json(&value["model"])?,
            output: value["output"]
                .as_array()
                .unwrap()
                .iter()
                .map(ChatCompletionChoice::from_json)
                .collect::<Result<Vec<ChatCompletionChoice>, Error>>()?,
            system_fingerprint: value
                .get("system_fingerprint")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            usage: UsageStats::from_json(&value["usage"])?,
            instructions: value
                .get("instructions")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            status: value
                .get("status")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            error: value
                .get("error")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            previous_response_id: value
                .get("previous_response_id")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            user: value
                .get("user")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            tool_choice: value
                .get("tool_choice")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            tools: value.get("tools").and_then(|v| {
                v.as_array()
                    .map(|arr| arr.iter().map(|t| Tool::from_json(t).unwrap()).collect())
            }),
            max_output_tokens: value
                .get("max_output_tokens")
                .and_then(|v| v.as_u64().map(|n| n as u32)),
            max_tool_calls: value
                .get("max_tool_calls")
                .and_then(|v| v.as_u64().map(|n| n as u32)),
            parallel_tool_calls: value.get("parallel_tool_calls").and_then(|v| v.as_bool()),
            store: value.get("store").and_then(|v| v.as_bool()),
            background: value.get("background").and_then(|v| v.as_bool()),
            service_tier: value
                .get("service_tier")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            billing: value
                .get("billing")
                .and_then(|v| Billing::from_json(v).ok()),
            prompt_cache_key: value
                .get("prompt_cache_key")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            temperature: value
                .get("temperature")
                .and_then(|v| v.as_f64().map(|f| f as f32)),
            top_p: value
                .get("top_p")
                .and_then(|v| v.as_f64().map(|f| f as f32)),
            top_logprobs: value
                .get("top_logprobs")
                .and_then(|v| v.as_u64().map(|n| n as u32)),
            truncation: value
                .get("truncation")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            metadata: value
                .get("metadata")
                .and_then(|v| Metadata::from_json(v).ok()),
            reasoning: value
                .get("reasoning")
                .and_then(|v| Reasoning::from_json(v).ok()),
            safety_identifier: value
                .get("safety_identifier")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            text: value
                .get("text")
                .and_then(|v| TextFormat::from_json(v).ok()),
            incomplete_details: value
                .get("incomplete_details")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
        })
    }
}

impl Generatable for ChatCompletionObject {
    fn gen(context: &mut GeneratorContext) -> Self {
        Self {
            id: String::gen(context),
            object: "chat.completion".to_string(),
            created_at: context.rng.gen(),
            model: ModelId::gen(context),
            output: vec![
                ChatCompletionChoice::gen(context),
                ChatCompletionChoice::gen(context),
            ],
            system_fingerprint: Some(String::gen(context)),
            usage: UsageStats::gen(context),
            instructions: None,
            status: None,
            error: None,
            previous_response_id: None,
            user: None,
            tool_choice: None,
            tools: None,
            max_output_tokens: None,
            max_tool_calls: None,
            parallel_tool_calls: None,
            store: None,
            background: None,
            service_tier: None,
            billing: None,
            prompt_cache_key: None,
            temperature: None,
            top_p: None,
            top_logprobs: None,
            truncation: None,
            metadata: None,
            reasoning: None,
            safety_identifier: None,
            text: None,
            incomplete_details: None,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    pub fn test_against_actual_response() {
        let json_str = r#"
{
  "id": "resp_00e03a4359cd7072006900408509448193a8bb9abcb078a820",
  "object": "response",
  "created_at": 1761624197,
  "status": "completed",
  "background": false,
  "billing": {
    "payer": "developer"
  },
  "error": null,
  "incomplete_details": null,
  "instructions": "You are a an expert book authoring AI.",
  "max_output_tokens": null,
  "max_tool_calls": null,
  "model": "gpt-5-mini-2025-08-07",
  "output": [
    {
      "id": "rs_00e03a4359cd70720069004086efbc819389003fd658cb2c1e",
      "type": "reasoning",
      "summary": []
    },
    {
      "id": "msg_00e03a4359cd7072006900408a53508193b9faef237382e8aa",
      "type": "message",
      "status": "completed",
      "content": [
        {
          "type": "output_text",
          "annotations": [],
          "logprobs": [],
          "text": "Aimed at professional and experienced novelists, this handbook reframes worldbuilding as a disciplined narrative craft\u2014less d\u00e9cor than engine\u2014teaching you how to make ecology, economy, politics, technology/magic and culture interlock so setting drives character, conflict and theme. Organized into modular chapters you can dip into as-needed, it combines systems-thinking frameworks, plausibility rules, cartography and timeline techniques with practical tools\u2014checklists, reproducible workflows, templates, software recommendations, case-study breakdowns and workshop-ready exercises\u2014to speed iteration and preserve continuity across novels, series or shared worlds. Rigorous about testing, editing, collaboration and ethical sourcing, it equips seasoned authors to build believable, narratively useful worlds and to sustain and scale them across a career."
        }
      ],
      "role": "assistant"
    }
  ],
  "parallel_tool_calls": true,
  "previous_response_id": null,
  "prompt_cache_key": null,
  "reasoning": {
    "effort": "medium",
    "summary": null
  },
  "safety_identifier": null,
  "service_tier": "default",
  "store": true,
  "temperature": 1.0,
  "text": {
    "format": {
      "type": "text"
    },
    "verbosity": "medium"
  },
  "tool_choice": "auto",
  "tools": [],
  "top_logprobs": 0,
  "top_p": 1.0,
  "truncation": "disabled",
  "usage": {
    "input_tokens": 2301,
    "input_tokens_details": {
      "cached_tokens": 0
    },
    "output_tokens": 414,
    "output_tokens_details": {
      "reasoning_tokens": 256
    },
    "total_tokens": 2715
  },
  "user": null,
  "metadata": {}
}        "#;

        let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let response = ChatCompletionObject::from_json(&v).unwrap();

        // Check that we can round-trip the JSON serialization
        let serialized = response.to_json();
        assert_eq!(serialized, v);
    }
}
