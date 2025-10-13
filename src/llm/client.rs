use async_trait::async_trait;
use crate::types::{ChatCompletionObject, ChatRequest};

#[async_trait]
pub trait RawRequester {
    async fn make_uncached_request(
        &mut self,
        request: &ChatRequest,
    ) -> anyhow::Result<ChatCompletionObject>;
}