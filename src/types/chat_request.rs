use crate::generate::{
    func_gen, gen_opt_vec, gen_vec, opt_gen, Generatable, Generator, GeneratorContext,
};
use crate::json::{FromJson, ToJson};
use crate::json_ext::JsonValueExt;
use crate::types::Error;
use crate::types::{LogitBias, Message, ModelId, ResponseFormat, Tool, ToolChoice};
use rand::Rng;
use serde_json::json;

use std::collections::BTreeMap;

/// TODO: Extract logit_bias as own struct.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatRequest {
    pub model: ModelId,
    pub messages: Vec<Message>,
    pub frequency_penalty: Option<f32>,
    pub logit_bias: Option<LogitBias>,
    pub max_tokens: Option<u32>,
    pub n: Option<u32>,
    pub presence_penalty: Option<f32>,
    pub response_format: Option<ResponseFormat>,
    pub seed: Option<i32>,
    pub stop: Option<Vec<String>>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub tools: Option<Vec<Tool>>,
    pub tool_choice: Option<ToolChoice>,
    pub user: Option<String>,
}

impl ChatRequest {
    pub fn new(model: ModelId, messages: Vec<Message>) -> ChatRequest {
        ChatRequest {
            model,
            messages,
            frequency_penalty: None,
            logit_bias: None,
            max_tokens: None,
            n: None,
            presence_penalty: None,
            response_format: None,
            seed: None,
            stop: None,
            temperature: None,
            top_p: None,
            tools: None,
            tool_choice: None,
            user: None,
        }
    }

    pub fn with_tool_choice(self, v: ToolChoice) -> ChatRequest {
        let mut result = self;
        result.tool_choice = Some(v);
        result
    }

    pub fn with_tools(self, tools: Vec<Tool>) -> ChatRequest {
        let mut result = self;
        result.tools = Some(tools);
        result
    }

    pub fn with_max_tokens(self, max_tokens: Option<u32>) -> ChatRequest {
        let mut result = self;
        result.max_tokens = max_tokens;
        result
    }

    pub fn with_response_format(self, response_format: ResponseFormat) -> ChatRequest {
        let mut result = self;
        result.response_format = Some(response_format);
        result
    }
}

impl ToJson for ChatRequest {
    fn to_json(&self) -> serde_json::Value {
        let mut v: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        v.insert("model".to_string(), json!(self.model.name()));
        v.insert(
            "messages".to_string(),
            serde_json::Value::Array(self.messages.iter().map(|m| m.to_json()).collect()),
        );
        if let Some(frequency_penalty) = &self.frequency_penalty {
            v.insert("frequency_penalty".to_string(), json!(frequency_penalty));
        }
        if let Some(logit_bias) = &self.logit_bias {
            v.insert("logit_bias".to_string(), logit_bias.to_json());
        }
        if let Some(max_tokens) = &self.max_tokens {
            v.insert("max_tokens".to_string(), json!(max_tokens));
        }
        if let Some(n) = &self.n {
            v.insert("n".to_string(), json!(n));
        }
        if let Some(presence_penalty) = &self.presence_penalty {
            v.insert("presence_penalty".to_string(), json!(presence_penalty));
        }
        if let Some(response_format) = &self.response_format {
            v.insert("response_format".to_string(), response_format.to_json());
        }
        if let Some(seed) = &self.seed {
            v.insert("seed".to_string(), json!(seed));
        }
        if let Some(stop) = &self.stop {
            v.insert("stop".to_string(), json!(stop));
        }
        if let Some(temperature) = &self.temperature {
            v.insert("temperature".to_string(), json!(temperature));
        }
        if let Some(top_p) = &self.top_p {
            v.insert("top_p".to_string(), json!(top_p));
        }
        if let Some(tools) = &self.tools {
            v.insert(
                "tools".to_string(),
                serde_json::Value::Array(tools.iter().map(|tool| tool.to_json()).collect()),
            );
        }
        if let Some(tool_choice) = &self.tool_choice {
            v.insert("tool_choice".to_string(), tool_choice.to_json());
        }
        if let Some(user) = &self.user {
            v.insert("user".to_string(), json!(user));
        }

        json!(v)
    }
}

impl FromJson for ChatRequest {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        Ok(ChatRequest {
            model: ModelId::from_json(&v["model"])?,
            messages: v["messages"].flat_map_array(Message::from_json)?,
            frequency_penalty: v["frequency_penalty"].to_opt_f32()?,
            logit_bias: v["logit_bias"].map_opt_obj(LogitBias::from_json)?,
            max_tokens: v["max_tokens"].to_opt_u32()?,
            n: v["n"].to_opt_u32()?,
            presence_penalty: v["presence_penalty"].to_opt_f32()?,
            response_format: v["response_format"].map_opt_obj(ResponseFormat::from_json)?,
            seed: v["seed"].to_opt_i32()?,
            stop: v["stop"].flat_map_opt_array(|v| {
                v.as_str()
                    .map(|s| s.to_string())
                    .ok_or(Error::JsonExpectedString)
            })?,
            temperature: v["temperature"].to_opt_f32()?,
            top_p: v["top_p"].to_opt_f32()?,
            tools: v["tools"].flat_map_opt_array(Tool::from_json)?,
            tool_choice: v["tool_choice"].map_opt(ToolChoice::from_json)?,
            user: v["user"].to_opt_string()?,
        })
    }
}

impl Generatable for ChatRequest {
    fn gen(context: &mut GeneratorContext) -> Self {
        ChatRequest {
            model: context.gen(),
            messages: gen_vec(context, 0, 4),
            frequency_penalty: context.rng.gen(),
            logit_bias: context.gen(),
            max_tokens: opt_gen(0.5, func_gen(|c| c.rng.gen_range(0..100))).gen(context),
            n: opt_gen(0.5, func_gen(|c| c.rng.gen_range(1..=4))).gen(context),
            presence_penalty: opt_gen(0.5, func_gen(|c| c.rng.gen())).gen(context),
            response_format: context.gen(),
            seed: opt_gen(0.25, func_gen(|c| c.rng.gen_range(0..100))).gen(context),
            stop: gen_opt_vec(context, 0.25, 0, 4),
            temperature: opt_gen(0.25, func_gen(|c| c.rng.gen())).gen(context),
            top_p: opt_gen(0.25, func_gen(|c| c.rng.gen())).gen(context),
            tools: gen_opt_vec(context, 0.25, 0, 4),
            tool_choice: context.gen(),
            user: context.gen(),
        }
    }
}
