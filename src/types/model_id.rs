use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;

use rand::Rng;
use serde_json::json;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ModelId {
    Gpt51(Option<String>),
    Gpt5(Option<String>),
    Gpt5Mini(Option<String>),
    Gpt5Nano(Option<String>),
}

impl ModelId {
    pub fn name(&self) -> String {
        match self {
            ModelId::Gpt51(tag) => Self::with_tag("gpt-5.1", tag),
            ModelId::Gpt5(tag) => Self::with_tag("gpt-5", tag),
            ModelId::Gpt5Mini(tag) => Self::with_tag("gpt-5-mini", tag),
            ModelId::Gpt5Nano(tag) => Self::with_tag("gpt-5-nano", tag),
        }
    }

    fn with_tag(base: &str, tag: &Option<String>) -> String {
        match tag {
            Some(t) => format!("{}-{}", base, t),
            None => base.to_string(),
        }
    }

    pub fn values() -> Vec<ModelId> {
        vec![
            ModelId::Gpt51(None),
            ModelId::Gpt5(None),
            ModelId::Gpt5Mini(None),
            ModelId::Gpt5Nano(None),
        ]
    }

    pub fn from_str(name: &str) -> Result<ModelId, Error> {
        if let Some(tag) = name.strip_prefix("gpt-5-mini") {
            Ok(ModelId::Gpt5Mini(Self::parse_tag(tag)))
        } else if let Some(tag) = name.strip_prefix("gpt-5-nano") {
            Ok(ModelId::Gpt5Nano(Self::parse_tag(tag)))
        } else if let Some(tag) = name.strip_prefix("gpt-5.1") {
            Ok(ModelId::Gpt51(Self::parse_tag(tag)))
        } else if let Some(tag) = name.strip_prefix("gpt-5") {
            Ok(ModelId::Gpt5(Self::parse_tag(tag)))
        } else {
            Err(Error::InvalidModelName)
        }
    }

    fn parse_tag(tag: &str) -> Option<String> {
        let t = tag.trim_start_matches('-');
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_id_name() {
        assert_eq!(ModelId::Gpt51(None).name(), "gpt-5.1");
        assert_eq!(
            ModelId::Gpt51(Some("2025".to_string())).name(),
            "gpt-5.1-2025"
        );
        assert_eq!(ModelId::Gpt5(None).name(), "gpt-5");
        assert_eq!(ModelId::Gpt5(Some("v1".to_string())).name(), "gpt-5-v1");
        assert_eq!(ModelId::Gpt5Mini(None).name(), "gpt-5-mini");
        assert_eq!(
            ModelId::Gpt5Mini(Some("beta".to_string())).name(),
            "gpt-5-mini-beta"
        );
        assert_eq!(ModelId::Gpt5Nano(None).name(), "gpt-5-nano");
        assert_eq!(ModelId::Gpt5Nano(Some("x".to_string())).name(), "gpt-5-nano-x");
    }

    #[test]
    fn test_model_id_from_str() {
        assert_eq!(ModelId::from_str("gpt-5.1").unwrap(), ModelId::Gpt51(None));
        assert_eq!(
            ModelId::from_str("gpt-5.1-2025").unwrap(),
            ModelId::Gpt51(Some("2025".to_string()))
        );
        assert_eq!(ModelId::from_str("gpt-5").unwrap(), ModelId::Gpt5(None));
        assert_eq!(
            ModelId::from_str("gpt-5-v1").unwrap(),
            ModelId::Gpt5(Some("v1".to_string()))
        );
        assert_eq!(
            ModelId::from_str("gpt-5-mini").unwrap(),
            ModelId::Gpt5Mini(None)
        );
        assert_eq!(
            ModelId::from_str("gpt-5-mini-2025-08-07").unwrap(),
            ModelId::Gpt5Mini(Some("2025-08-07".to_string()))
        );
        assert_eq!(
            ModelId::from_str("gpt-5-nano").unwrap(),
            ModelId::Gpt5Nano(None)
        );

        // Edge cases
        assert!(matches!(
            ModelId::from_str("invalid"),
            Err(Error::InvalidModelName)
        ));
    }

    #[test]
    fn test_model_id_round_trip() {
        let models = vec![
            ModelId::Gpt51(None),
            ModelId::Gpt51(Some("2025".to_string())),
            ModelId::Gpt5(None),
            ModelId::Gpt5Mini(Some("x-y-z".to_string())),
        ];

        for model in models {
            let json = model.to_json();
            let back = ModelId::from_json(&json).unwrap();
            assert_eq!(model, back);
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
        let tag = if context.rng.gen_bool(0.5) {
            Some(String::gen(context))
        } else {
            None
        };
        match context.rng.gen_range(0..4) {
            0 => ModelId::Gpt51(tag),
            1 => ModelId::Gpt5(tag),
            2 => ModelId::Gpt5Mini(tag),
            _ => ModelId::Gpt5Nano(tag),
        }
    }
}
