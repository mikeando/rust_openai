use crate::llm::claude::ClaudeModelId;
use crate::llm::openai::OpenAIModelId;
use crate::json::{ToJson, FromJson};
use crate::types::Error;
use serde_json::json;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ModelId {
    OpenAI(OpenAIModelId),
    Claude(ClaudeModelId),
}

impl ToJson for ModelId {
    fn to_json(&self) -> serde_json::Value {
        match self {
            ModelId::OpenAI(m) => m.to_json(),
            ModelId::Claude(m) => m.to_json(),
        }
    }
}

impl FromJson for ModelId {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        let s = v.as_str().unwrap();
        if let Ok(m) = OpenAIModelId::from_str(s) {
            return Ok(ModelId::OpenAI(m));
        }
        if let Ok(m) = ClaudeModelId::from_str(s) {
            return Ok(ModelId::Claude(m));
        }
        Err(Error::InvalidModelName)
    }
}