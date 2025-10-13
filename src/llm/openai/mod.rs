use reqwest::Client;
use anyhow::{anyhow, bail};
use std::sync::Arc;

use crate::{
    json::FromJson,
    types::{ChatCompletionObject, ChatRequest},
};
use crate::json::ToJson;
use crate::generate::{Generatable, GeneratorContext};
use crate::types::Error;
use rand::Rng;
use serde_json::json;
use crate::llm::RequestCache;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OpenAIModelId {
    Gpt35Turbo,
    Gpt35Turbo0613,
    Gpt35Turbo0125,
    Gpt4o,
    Gpt4o20240513,
    Gpt4o20240806,
    Gpt4oMini,
    Gpt4oMini20240718,
}

impl OpenAIModelId {
    pub fn name(&self) -> String {
        match self {
            OpenAIModelId::Gpt35Turbo => "gpt-3.5-turbo".to_string(),
            OpenAIModelId::Gpt35Turbo0613 => "gpt-3.5-turbo-0613".to_string(),
            OpenAIModelId::Gpt35Turbo0125 => "gpt-3.5-turbo-0125".to_string(),
            OpenAIModelId::Gpt4o => "gpt-4o".to_string(),
            OpenAIModelId::Gpt4o20240513 => "gpt-4o-2024-05-13".to_string(),
            OpenAIModelId::Gpt4o20240806 => "gpt-4o-2024-08-06".to_string(),
            OpenAIModelId::Gpt4oMini => "gpt-4o-mini".to_string(),
            OpenAIModelId::Gpt4oMini20240718 => "gpt-4o-mini-2024-07-18".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "gpt-3.5-turbo" => Ok(OpenAIModelId::Gpt35Turbo),
            "gpt-3.5-turbo-0613" => Ok(OpenAIModelId::Gpt35Turbo0613),
            "gpt-3.5-turbo-0125" => Ok(OpenAIModelId::Gpt35Turbo0125),
            "gpt-4o" => Ok(OpenAIModelId::Gpt4o),
            "gpt-4o-2024-05-13" => Ok(OpenAIModelId::Gpt4o20240513),
            "gpt-4o-2024-08-06" => Ok(OpenAIModelId::Gpt4o20240806),
            "gpt-4o-mini" => Ok(OpenAIModelId::Gpt4oMini),
            "gpt-4o-mini-2024-07-18" => Ok(OpenAIModelId::Gpt4oMini20240718),
            _ => Err(Error::InvalidModelName),
        }
    }

    fn all() -> Vec<Self> {
        vec![
            OpenAIModelId::Gpt35Turbo,
            OpenAIModelId::Gpt35Turbo0613,
            OpenAIModelId::Gpt35Turbo0125,
            OpenAIModelId::Gpt4o,
            OpenAIModelId::Gpt4o20240513,
            OpenAIModelId::Gpt4o20240806,
            OpenAIModelId::Gpt4oMini,
            OpenAIModelId::Gpt4oMini20240718,
        ]
    }
}

impl ToJson for OpenAIModelId {
    fn to_json(&self) -> serde_json::Value {
        json!(self.name())
    }
}

impl FromJson for OpenAIModelId {
    fn from_json(v: &serde_json::Value) -> Result<Self, Error> {
        Self::from_str(v.as_str().unwrap()).map_err(|_| Error::InvalidModelName)
    }
}

impl Generatable for OpenAIModelId {
    fn gen(context: &mut GeneratorContext) -> Self {
        let all = Self::all();
        let i = context.rng.gen_range(0..all.len());
        all[i]
    }
}

use crate::llm::{DefaultRequestCache, DefaultFS};

pub struct OpenAILlm {
    model: OpenAIModelId,
    requester: OpenAIRawRequester,
    cache: DefaultRequestCache,
}

impl OpenAILlm {
    pub async fn new(openai_api_key: &str, model: OpenAIModelId) -> anyhow::Result<Self> {
        let requester = OpenAIRawRequester {
            openai_api_key: openai_api_key.to_string(),
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

pub struct OpenAIRawRequester {
    pub openai_api_key: String,
}

impl OpenAIRawRequester {
    pub async fn make_uncached_request(
        &mut self,
        request: &ChatRequest,
        model: &OpenAIModelId,
    ) -> anyhow::Result<ChatCompletionObject> {
        let client = Client::new();
        let mut full_request = request.to_json();
        full_request["model"] = model.to_json();

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.openai_api_key))
            .json(&full_request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let response_text = response.text().await?;
            let v: serde_json::Value = serde_json::from_str(&response_text)?;
            let error_message = &v["error"]["message"];

            bail!(
                "Error making openai request: ({} {}) {}",
                status.as_str(),
                status.canonical_reason().unwrap_or("Unknown"),
                error_message.as_str().unwrap_or("")
            );
        }

        let response_text = response.text().await?;
        let v: serde_json::Value = serde_json::from_str(&response_text)?;
        let response: ChatCompletionObject = ChatCompletionObject::from_json(&v)
            .map_err(|e| anyhow!("Error decoding openai response: {:?}", e))?;
        Ok(response)
    }
}