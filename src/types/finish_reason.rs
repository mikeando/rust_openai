use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use rand::Rng;
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinishReason {
    Stop,
    ToolCalls,
}

impl ToJson for FinishReason {
    fn to_json(&self) -> serde_json::Value {
        match self {
            FinishReason::Stop => json!("stop"),
            FinishReason::ToolCalls => json!("tool_calls"),
        }
    }
}

impl FromJson for FinishReason {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        match v.as_str() {
            Some("stop") => Ok(FinishReason::Stop),
            Some("tool_calls") => Ok(FinishReason::ToolCalls),
            _ => Err(Error::InvalidFinishReason),
        }
    }
}

impl Generatable for FinishReason {
    fn gen(context: &mut GeneratorContext) -> Self {
        match context.rng.gen_range(0..=1) {
            0 => FinishReason::Stop,
            1 => FinishReason::ToolCalls,
            _ => unreachable!(),
        }
    }
}
