use async_trait::async_trait;
use reqwest::Client;
use anyhow::{anyhow, bail};

use crate::{
    json::FromJson,
    types::{ChatCompletionObject, ChatRequest},
};
use crate::json::ToJson;
use super::client::RawRequester;
use crate::generate::{Generatable, GeneratorContext};
use crate::types::Error;
use rand::Rng;
use serde_json::json;

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

pub struct OpenAIRawRequester {
    pub openai_api_key: String,
}

#[async_trait]
impl RawRequester for OpenAIRawRequester {
    async fn make_uncached_request(
        &mut self,
        request: &ChatRequest,
    ) -> anyhow::Result<ChatCompletionObject> {
        let client = Client::new();

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.openai_api_key))
            .json(&request.to_json())
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            // Try and see if we can divine the reason from the JSON,
            // if we get any...
            // TODO: If this doesn't work we should at least return what we do know
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