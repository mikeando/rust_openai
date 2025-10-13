use async_trait::async_trait;
use anyhow::bail;

use crate::{
    types::{ChatCompletionObject, ChatRequest},
};
use super::client::RawRequester;

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