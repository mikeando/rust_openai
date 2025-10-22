use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;

use rand::Rng;
use serde_json::json;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BaseModelId {
    Gpt35Turbo,
    Gpt4o,
    Gpt4oMini,
    Gpt5,
    Gpt5Mini,
    Gpt5Nano,
    Gpt5Pro,
    Gpt41,
}

impl BaseModelId {
    pub fn name(&self) -> &str {
        match self {
            BaseModelId::Gpt35Turbo => "gpt-3.5-turbo",
            BaseModelId::Gpt4o => "gpt-4o",
            BaseModelId::Gpt4oMini => "gpt-4o-mini",
            BaseModelId::Gpt5 => "gpt-5",
            BaseModelId::Gpt5Mini => "gpt-5-mini",
            BaseModelId::Gpt5Nano => "gpt-5-nano",
            BaseModelId::Gpt5Pro => "gpt-5-pro",
            BaseModelId::Gpt41 => "gpt-4.1",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ModelId {
    pub base_model: BaseModelId,
    pub version: Option<String>,
}

impl ModelId {
    pub fn new(base_model: BaseModelId) -> ModelId {
        ModelId {
            base_model,
            version: None,
        }
    }

    pub fn with_version(mut self, version: &str) -> ModelId {
        self.version = Some(version.to_string());
        self
    }

    pub fn name(&self) -> String {
        let base = self.base_model.name();
        if let Some(version) = &self.version {
            format!("{}-{}", base, version)
        } else {
            base.to_string()
        }
    }

    pub fn from_str(name: &str) -> Result<ModelId, Error> {
        let (base_model, version) = if let Some(version) = name.strip_prefix("gpt-4o-mini-") {
            (BaseModelId::Gpt4oMini, Some(version))
        } else if name == "gpt-4o-mini" {
            (BaseModelId::Gpt4oMini, None)
        } else if let Some(version) = name.strip_prefix("gpt-3.5-turbo-") {
            (BaseModelId::Gpt35Turbo, Some(version))
        } else if name == "gpt-3.5-turbo" {
            (BaseModelId::Gpt35Turbo, None)
        } else if let Some(version) = name.strip_prefix("gpt-4o-") {
            (BaseModelId::Gpt4o, Some(version))
        } else if name == "gpt-4o" {
            (BaseModelId::Gpt4o, None)
        } else if let Some(version) = name.strip_prefix("gpt-5-mini-") {
            (BaseModelId::Gpt5Mini, Some(version))
        } else if name == "gpt-5-mini" {
            (BaseModelId::Gpt5Mini, None)
        } else if let Some(version) = name.strip_prefix("gpt-5-nano-") {
            (BaseModelId::Gpt5Nano, Some(version))
        } else if name == "gpt-5-nano" {
            (BaseModelId::Gpt5Nano, None)
        } else if let Some(version) = name.strip_prefix("gpt-5-pro-") {
            (BaseModelId::Gpt5Pro, Some(version))
        } else if name == "gpt-5-pro" {
            (BaseModelId::Gpt5Pro, None)
        } else if let Some(version) = name.strip_prefix("gpt-5-") {
            (BaseModelId::Gpt5, Some(version))
        } else if name == "gpt-5" {
            (BaseModelId::Gpt5, None)
        } else if name == "gpt-4.1" {
            (BaseModelId::Gpt41, None)
        } else {
            return Err(Error::InvalidModelName);
        };

        Ok(ModelId {
            base_model,
            version: version.map(|s| s.to_string()),
        })
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
        let base_models = [
            BaseModelId::Gpt35Turbo,
            BaseModelId::Gpt4o,
            BaseModelId::Gpt4oMini,
            BaseModelId::Gpt5,
            BaseModelId::Gpt5Mini,
            BaseModelId::Gpt5Nano,
            BaseModelId::Gpt5Pro,
            BaseModelId::Gpt41,
        ];
        let base_model = base_models[context.rng.gen_range(0..base_models.len())];
        let version = if base_model != BaseModelId::Gpt41 && context.rng.gen_bool(0.5) {
            Some(String::gen(context))
        } else {
            None
        };
        ModelId {
            base_model,
            version,
        }
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
            ModelId {
                base_model: BaseModelId::Gpt4o,
                version: Some("2024-05-13".to_string())
            }
        );
    }
}
