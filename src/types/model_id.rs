use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;

use rand::Rng;
use serde_json::json;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ModelId {
    // $0.5/1.5 per M in/out
    Gpt35Turbo(Option<String>),
    // $5.00/15.00 per M in/out
    Gpt4o(Option<String>),
    // $0.15/0.60 per M in/out
    Gpt4oMini(Option<String>),
    // Frontier Models
    Gpt5(Option<String>),
    Gpt5Mini(Option<String>),
    Gpt5Nano(Option<String>),
    Gpt5Pro(Option<String>),
    Gpt41,
}

impl ModelId {
    pub fn name(&self) -> String {
        let (base, version) = match self {
            ModelId::Gpt35Turbo(v) => ("gpt-3.5-turbo", v),
            ModelId::Gpt4o(v) => ("gpt-4o", v),
            ModelId::Gpt4oMini(v) => ("gpt-4o-mini", v),
            ModelId::Gpt5(v) => ("gpt-5", v),
            ModelId::Gpt5Mini(v) => ("gpt-5-mini", v),
            ModelId::Gpt5Nano(v) => ("gpt-5-nano", v),
            ModelId::Gpt5Pro(v) => ("gpt-5-pro", v),
            ModelId::Gpt41 => ("gpt-4.1", &None),
        };

        if let Some(version) = version {
            format!("{}-{}", base, version)
        } else {
            base.to_string()
        }
    }

    pub fn values() -> Vec<ModelId> {
        vec![
            ModelId::Gpt35Turbo(None),
            ModelId::Gpt35Turbo(Some("0613".to_string())),
            ModelId::Gpt35Turbo(Some("0125".to_string())),
            ModelId::Gpt4o(None),
            ModelId::Gpt4o(Some("2024-05-13".to_string())),
            ModelId::Gpt4o(Some("2024-08-06".to_string())),
            ModelId::Gpt4oMini(None),
            ModelId::Gpt4oMini(Some("2024-07-18".to_string())),
            ModelId::Gpt5(None),
            ModelId::Gpt5Mini(None),
            ModelId::Gpt5Nano(None),
            ModelId::Gpt5Pro(None),
            ModelId::Gpt41,
        ]
    }

    pub fn from_str(name: &str) -> Result<ModelId, Error> {
        if let Some(version) = name.strip_prefix("gpt-4o-mini-") {
            Ok(ModelId::Gpt4oMini(Some(version.to_string())))
        } else if name == "gpt-4o-mini" {
            Ok(ModelId::Gpt4oMini(None))
        } else if let Some(version) = name.strip_prefix("gpt-3.5-turbo-") {
            Ok(ModelId::Gpt35Turbo(Some(version.to_string())))
        } else if name == "gpt-3.5-turbo" {
            Ok(ModelId::Gpt35Turbo(None))
        } else if let Some(version) = name.strip_prefix("gpt-4o-") {
            Ok(ModelId::Gpt4o(Some(version.to_string())))
        } else if name == "gpt-4o" {
            Ok(ModelId::Gpt4o(None))
        } else if let Some(version) = name.strip_prefix("gpt-5-mini-") {
            Ok(ModelId::Gpt5Mini(Some(version.to_string())))
        } else if name == "gpt-5-mini" {
            Ok(ModelId::Gpt5Mini(None))
        } else if let Some(version) = name.strip_prefix("gpt-5-nano-") {
            Ok(ModelId::Gpt5Nano(Some(version.to_string())))
        } else if name == "gpt-5-nano" {
            Ok(ModelId::Gpt5Nano(None))
        } else if let Some(version) = name.strip_prefix("gpt-5-pro-") {
            Ok(ModelId::Gpt5Pro(Some(version.to_string())))
        } else if name == "gpt-5-pro" {
            Ok(ModelId::Gpt5Pro(None))
        } else if let Some(version) = name.strip_prefix("gpt-5-") {
            Ok(ModelId::Gpt5(Some(version.to_string())))
        } else if name == "gpt-5" {
            Ok(ModelId::Gpt5(None))
        } else if name == "gpt-4.1" {
            Ok(ModelId::Gpt41)
        } else {
            Err(Error::InvalidModelName)
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
        values[i].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_from_str() {
        assert_eq!(
            ModelId::from_str("gpt-4o-2024-05-13").unwrap(),
            ModelId::Gpt4o(Some("2024-05-13".to_string()))
        );
    }
}
