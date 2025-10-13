
use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use rand::Rng;
use serde_json::json;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ModelId {
    pub name: String,
}

impl ModelId {
    pub fn new(name: &str) -> ModelId {
        ModelId {
            name: name.to_string(),
        }
    }

    // Well-known models
    pub fn gpt_3_5_turbo() -> Self { Self::new("gpt-3.5-turbo") }
    pub fn gpt_3_5_turbo_0613() -> Self { Self::new("gpt-3.5-turbo-0613") }
    pub fn gpt_3_5_turbo_0125() -> Self { Self::new("gpt-3.5-turbo-0125") }
    pub fn gpt_4o() -> Self { Self::new("gpt-4o") }
    pub fn gpt_4o_2024_05_13() -> Self { Self::new("gpt-4o-2024-05-13") }
    pub fn gpt_4o_2024_08_06() -> Self { Self::new("gpt-4o-2024-08-06") }
    pub fn gpt_4o_mini() -> Self { Self::new("gpt-4o-mini") }
    pub fn gpt_4o_mini_2024_07_18() -> Self { Self::new("gpt-4o-mini-2024-07-18") }

    fn all_known() -> Vec<Self> {
        vec![
            Self::gpt_3_5_turbo(),
            Self::gpt_3_5_turbo_0613(),
            Self::gpt_3_5_turbo_0125(),
            Self::gpt_4o(),
            Self::gpt_4o_2024_05_13(),
            Self::gpt_4o_2024_08_06(),
            Self::gpt_4o_mini(),
            Self::gpt_4o_mini_2024_07_18(),
        ]
    }
}

impl FromJson for ModelId {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        Ok(ModelId {
            name: v.as_str().unwrap().to_string(),
        })
    }
}

impl ToJson for ModelId {
    fn to_json(&self) -> serde_json::Value {
        json!(self.name)
    }
}

impl Generatable for ModelId {
    fn gen(context: &mut GeneratorContext) -> Self {
        let values = Self::all_known();
        let i = context.rng.gen_range(0..values.len());
        values[i].clone()
    }
}
