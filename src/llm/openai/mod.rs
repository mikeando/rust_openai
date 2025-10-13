use async_trait::async_trait;
use reqwest::Client;
use anyhow::{anyhow, bail};

use crate::{
    json::FromJson,
    types::{ChatCompletionObject, ChatRequest},
};
use crate::json::ToJson;
use super::client::RawRequester;

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