use async_trait::async_trait;
use anyhow::bail;

use crate::{
    types::{ChatCompletionObject, ChatRequest},
};
use super::client::RawRequester;
use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use rand::Rng;
use serde_json::json;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ClaudeModelId {
    Claude2,
    Claude3Opus,
}

impl ClaudeModelId {
    pub fn name(&self) -> String {
        match self {
            ClaudeModelId::Claude2 => "claude-2".to_string(),
            ClaudeModelId::Claude3Opus => "claude-3-opus-20240229".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "claude-2" => Ok(ClaudeModelId::Claude2),
            "claude-3-opus-20240229" => Ok(ClaudeModelId::Claude3Opus),
            _ => Err(Error::InvalidModelName),
        }
    }

    fn all() -> Vec<Self> {
        vec![
            ClaudeModelId::Claude2,
            ClaudeModelId::Claude3Opus,
        ]
    }
}

impl ToJson for ClaudeModelId {
    fn to_json(&self) -> serde_json::Value {
        json!(self.name())
    }
}

impl FromJson for ClaudeModelId {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        Self::from_str(v.as_str().unwrap()).map_err(|_| Error::InvalidModelName)
    }
}

impl Generatable for ClaudeModelId {
    fn gen(context: &mut GeneratorContext) -> Self {
        let all = Self::all();
        let i = context.rng.gen_range(0..all.len());
        all[i]
    }
}

pub struct ClaudeRawRequester {}

#[async_trait]
impl RawRequester for ClaudeRawRequester {
    async fn make_uncached_request(
        &mut self,
        _request: &ChatRequest,
    ) -> anyhow::Result<ChatCompletionObject> {
        bail!("ClaudeRawRequester is not implemented yet")
    }
}