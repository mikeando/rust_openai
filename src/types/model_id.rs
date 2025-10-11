use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;

use rand::Rng;
use serde_json::json;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ModelId {
    // $0.5/1.5 per M in/out
    Gpt35Turbo,
    Gpt35Turbo0613,
    Gpt35Turbo0125,
    // $5.00/15.00 per M in/out
    Gpt4o,
    Gpt4o20240513,
    Gpt4o20240806,
    // $0.15/0.60 per M in/out
    Gpt4oMini,
    Gpt4oMini20240718,
    // Frontier Models
    Gpt5,
    Gpt5Mini,
    Gpt5Nano,
    Gpt5Pro,
    Gpt41,
}

impl ModelId {
    pub fn name(&self) -> String {
        match self {
            ModelId::Gpt35Turbo => String::from("gpt-3.5-turbo"),
            ModelId::Gpt35Turbo0613 => String::from("gpt-3.5-turbo-0613"),
            ModelId::Gpt35Turbo0125 => String::from("gpt-3.5-turbo-0125"),
            ModelId::Gpt4o => String::from("gpt-4o"),
            ModelId::Gpt4o20240513 => String::from("gpt-4o-2024-05-13"),
            ModelId::Gpt4o20240806 => String::from("gpt-4o-2024-08-06"),
            ModelId::Gpt4oMini => String::from("gpt-4o-mini"),
            ModelId::Gpt4oMini20240718 => String::from("gpt-4o-mini-2024-07-18"),
            ModelId::Gpt5 => String::from("gpt-5"),
            ModelId::Gpt5Mini => String::from("gpt-5-mini"),
            ModelId::Gpt5Nano => String::from("gpt-5-nano"),
            ModelId::Gpt5Pro => String::from("gpt-5-pro"),
            ModelId::Gpt41 => String::from("gpt-4.1"),
        }
    }

    pub fn values() -> Vec<ModelId> {
        vec![
            ModelId::Gpt35Turbo,
            ModelId::Gpt35Turbo0613,
            ModelId::Gpt35Turbo0125,
            ModelId::Gpt4o,
            ModelId::Gpt4o20240513,
            ModelId::Gpt4o20240806,
            ModelId::Gpt4oMini,
            ModelId::Gpt4oMini20240718,
            ModelId::Gpt5,
            ModelId::Gpt5Mini,
            ModelId::Gpt5Nano,
            ModelId::Gpt5Pro,
            ModelId::Gpt41,
        ]
    }

    pub fn from_str(name: &str) -> Result<ModelId, Error> {
        match name {
            "gpt-3.5-turbo" => Ok(ModelId::Gpt35Turbo),
            "gpt-3.5-turbo-0613" => Ok(ModelId::Gpt35Turbo0613),
            "gpt-3.5-turbo-0125" => Ok(ModelId::Gpt35Turbo0125),
            "gpt-4o" => Ok(ModelId::Gpt4o),
            "gpt-4o-2024-05-13" => Ok(ModelId::Gpt4o20240513),
            "gpt-4o-2024-08-06" => Ok(ModelId::Gpt4o20240806),
            "gpt-4o-mini" => Ok(ModelId::Gpt4oMini),
            "gpt-4o-mini-2024-07-18" => Ok(ModelId::Gpt4oMini20240718),
            "gpt-5" => Ok(ModelId::Gpt5),
            "gpt-5-mini" => Ok(ModelId::Gpt5Mini),
            "gpt-5-nano" => Ok(ModelId::Gpt5Nano),
            "gpt-5-pro" => Ok(ModelId::Gpt5Pro),
            "gpt-4.1" => Ok(ModelId::Gpt41),
            _ => Err(Error::InvalidModelName),
        }
    }
}

impl FromJson for ModelId {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        ModelId::from_str(v.as_str().unwrap())
    }
}

impl ToJson for ModelId {
    fn to_json(&self) -> serde_json::Value {
        json!(self.name())
    }
}

impl Generatable for ModelId {
    fn gen(context: &mut GeneratorContext) -> Self {
        let values = Self::values();
        let i = context.rng.gen_range(0..values.len());
        values[i]
    }
}
