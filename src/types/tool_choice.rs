use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use rand::Rng;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub enum ToolChoice {
    Auto,
    Required,
    None,
}

impl ToJson for ToolChoice {
    fn to_json(&self) -> serde_json::Value {
        match self {
            ToolChoice::Auto => json!("auto"),
            ToolChoice::Required => json!("required"),
            ToolChoice::None => json!("none"),
        }
    }
}

impl FromJson for ToolChoice {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        match v.as_str() {
            Some("auto") => Ok(ToolChoice::Auto),
            Some("required") => Ok(ToolChoice::Required),
            Some("none") => Ok(ToolChoice::None),
            _ => Err(Error::InvalidToolChoice),
        }
    }
}

impl Generatable for ToolChoice {
    fn gen(context: &mut GeneratorContext) -> Self {
        match context.rng.gen_range(0..=0) {
            0 => ToolChoice::Auto,
            _ => unreachable!(),
        }
    }
}
