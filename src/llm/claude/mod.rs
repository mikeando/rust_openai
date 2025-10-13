use anyhow::bail;

use crate::{
    types::{ChatCompletionObject, ChatRequest},
};
use crate::generate::{Generatable, GeneratorContext};
use crate::json::{FromJson, ToJson};
use crate::types::Error;
use rand::Rng;
use serde_json::json;
use crate::llm::RequestCache;

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

use crate::llm::{DefaultRequestCache, DefaultFS};
use std::sync::Arc;

pub struct ClaudeLlm {
    model: ClaudeModelId,
    requester: ClaudeRawRequester,
    cache: DefaultRequestCache,
}

impl ClaudeLlm {
    pub async fn new(claude_api_key: &str, model: ClaudeModelId) -> anyhow::Result<Self> {
        let requester = ClaudeRawRequester {
            claude_api_key: claude_api_key.to_string(),
        };
        let fs = DefaultFS {};
        let cache = DefaultRequestCache::new(Arc::new(fs), std::path::PathBuf::from("cache")).await?;
        Ok(Self {
            model,
            requester,
            cache,
        })
    }

    pub async fn make_request(
        &mut self,
        request: &ChatRequest,
    ) -> anyhow::Result<(ChatCompletionObject, bool)> {
        if let Some(v) = self.cache.get_response_if_cached(request).await? {
            return Ok((v, true));
        }

        let response = self.requester.make_uncached_request(request, &self.model).await?;
        self.cache.cache_response(request, &response).await?;
        Ok((response, false))
    }
}

pub struct ClaudeRawRequester {
    pub claude_api_key: String,
}

impl ClaudeRawRequester {
    pub async fn make_uncached_request(
        &mut self,
        _request: &ChatRequest,
        _model: &ClaudeModelId,
    ) -> anyhow::Result<ChatCompletionObject> {
        bail!("ClaudeRawRequester is not implemented yet")
    }
}