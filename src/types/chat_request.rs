use crate::{
    generate::{Generatable, GeneratorContext},
    json::{FromJson, ToJson},
    types::{Error, Message, ModelId, ResponseFormat, Tool, ToolChoice},
};

use rand::Rng;
use serde_json::{json, Value};

#[derive(Debug, PartialEq, Clone)]
pub struct ChatRequest {
    pub model: ModelId,
    pub input: Vec<Message>,
    pub instructions: Option<String>,
    pub response_format: Option<ResponseFormat>,
    pub seed: Option<i64>,
    pub store: Option<bool>,
    pub tools: Option<Vec<Tool>>,
    pub tool_choice: Option<ToolChoice>,
    pub previous_response_id: Option<String>,
}

impl ChatRequest {
    pub fn new(model: ModelId, input: Vec<Message>) -> Self {
        Self {
            model,
            input,
            instructions: None,
            response_format: None,
            seed: None,
            store: None,
            tools: None,
            tool_choice: None,
            previous_response_id: None,
        }
    }

    pub fn with_instructions(mut self, instructions: String) -> Self {
        self.instructions = Some(instructions);
        self
    }

    pub fn with_response_format(mut self, response_format: ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }

    pub fn with_seed(mut self, seed: i64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn with_store(mut self, store: bool) -> Self {
        self.store = Some(store);
        self
    }

    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    pub fn with_previous_response_id(mut self, previous_response_id: String) -> Self {
        self.previous_response_id = Some(previous_response_id);
        self
    }
}

impl ToJson for ChatRequest {
    fn to_json(&self) -> Value {
        let mut result = json!({
            "model": self.model.to_json(),
            "input": self.input.iter().map(|m| m.to_json()).collect::<Vec<Value>>(),
        });
        if let Some(instructions) = &self.instructions {
            result["instructions"] = json!(instructions);
        }
        if let Some(response_format) = &self.response_format {
            result["response_format"] = response_format.to_json();
        }
        if let Some(seed) = self.seed {
            result["seed"] = json!(seed);
        }
        if let Some(store) = self.store {
            result["store"] = json!(store);
        }
        if let Some(tools) = &self.tools {
            result["tools"] = json!(tools.iter().map(|t| t.to_json()).collect::<Vec<Value>>());
        }
        if let Some(tool_choice) = &self.tool_choice {
            result["tool_choice"] = tool_choice.to_json();
        }
        if let Some(previous_response_id) = &self.previous_response_id {
            result["previous_response_id"] = json!(previous_response_id);
        }
        result
    }
}

impl FromJson for ChatRequest {
    fn from_json(value: &Value) -> Result<Self, Error> {
        let model = ModelId::from_json(&value["model"])?;
        let input = value["input"]
            .as_array()
            .unwrap()
            .iter()
            .map(Message::from_json)
            .collect::<Result<Vec<Message>, Error>>()?;
        let mut result = Self::new(model, input);
        if let Some(instructions) = value.get("instructions") {
            result = result.with_instructions(instructions.as_str().unwrap().to_string());
        }
        if let Some(response_format) = value.get("response_format") {
            result = result.with_response_format(ResponseFormat::from_json(response_format)?);
        }
        if let Some(seed) = value.get("seed") {
            result = result.with_seed(seed.as_i64().unwrap());
        }
        if let Some(store) = value.get("store") {
            result = result.with_store(store.as_bool().unwrap());
        }
        if let Some(tools) = value.get("tools") {
            result = result.with_tools(
                tools
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(Tool::from_json)
                    .collect::<Result<Vec<Tool>, Error>>()?,
            );
        }
        if let Some(tool_choice) = value.get("tool_choice") {
            result = result.with_tool_choice(ToolChoice::from_json(tool_choice)?);
        }
        if let Some(previous_response_id) = value.get("previous_response_id") {
            result = result
                .with_previous_response_id(previous_response_id.as_str().unwrap().to_string());
        }
        Ok(result)
    }
}

impl Generatable for ChatRequest {
    fn gen(context: &mut GeneratorContext) -> Self {
        let mut result = Self::new(
            ModelId::gen(context),
            vec![Message::gen(context), Message::gen(context)],
        );
        if context.rng.gen() {
            result = result.with_response_format(ResponseFormat::gen(context));
        }
        if context.rng.gen() {
            result = result.with_seed(context.rng.gen());
        }
        if context.rng.gen() {
            result = result.with_store(context.rng.gen());
        }
        if context.rng.gen() {
            result = result.with_tools(vec![Tool::gen(context), Tool::gen(context)]);
        }
        if context.rng.gen() {
            result = result.with_tool_choice(ToolChoice::gen(context));
        }
        if context.rng.gen() {
            result = result.with_previous_response_id(String::gen(context));
        }
        result
    }
}
