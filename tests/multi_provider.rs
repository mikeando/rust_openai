use rust_openai::llm::claude::ClaudeRawRequester;
use rust_openai::llm::openai::OpenAIRawRequester;
use rust_openai::llm::provider::LLMProvider;
use rust_openai::llm::GenericLLM;
use rust_openai::llm::{RequestCache, TrivialFS, TrivialFSPathType};
use rust_openai::types::{ChatRequest, Message, ModelId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

struct MockFS {
    files: Arc<Mutex<HashMap<String, String>>>,
}

#[async_trait::async_trait]
impl TrivialFS for MockFS {
    async fn read_to_string(&self, p: &std::path::Path) -> anyhow::Result<String> {
        self.files
            .lock()
            .await
            .get(p.to_str().unwrap())
            .map(|s| s.clone())
            .ok_or(anyhow::anyhow!("File not found"))
    }

    async fn write(&self, p: &std::path::Path, value: &str) -> anyhow::Result<()> {
        self.files
            .lock()
            .await
            .insert(p.to_str().unwrap().to_string(), value.to_string());
        Ok(())
    }

    async fn path_type(&self, p: &std::path::Path) -> anyhow::Result<TrivialFSPathType> {
        if self.files.lock().await.contains_key(p.to_str().unwrap()) {
            Ok(TrivialFSPathType::File)
        } else {
            Ok(TrivialFSPathType::NoSuchPath)
        }
    }
}

struct MockCache {}

#[async_trait::async_trait]
impl RequestCache for MockCache {
    async fn get_response_if_cached(
        &self,
        _request: &ChatRequest,
    ) -> anyhow::Result<Option<rust_openai::types::ChatCompletionObject>> {
        Ok(None)
    }

    async fn cache_response(
        &mut self,
        _request: &ChatRequest,
        _response: &rust_openai::types::ChatCompletionObject,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

use rust_openai::llm::client::RawRequester;
use rust_openai::types::ChatCompletionObject;

struct MockRawRequester {
    response: anyhow::Result<ChatCompletionObject>,
}

#[async_trait::async_trait]
impl RawRequester for MockRawRequester {
    async fn make_uncached_request(
        &mut self,
        _request: &ChatRequest,
    ) -> anyhow::Result<ChatCompletionObject> {
        self.response.as_ref().cloned().map_err(|e| anyhow::anyhow!(e.to_string()))
    }
}

#[tokio::test]
async fn test_multi_provider() {
    let openai_response = ChatCompletionObject {
        id: "chatcmpl-123".to_string(),
        choices: vec![],
        created: 1677652288,
        model: ModelId::new("gpt-4"),
        system_fingerprint: None,
        object: "chat.completion".to_string(),
        usage: rust_openai::types::UsageStats {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
    };

    let claude_response = ChatCompletionObject {
        id: "chatcmpl-456".to_string(),
        choices: vec![],
        created: 1677652288,
        model: ModelId::new("claude-2"),
        system_fingerprint: None,
        object: "chat.completion".to_string(),
        usage: rust_openai::types::UsageStats {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
    };

    let openai_provider = LLMProvider::OpenAI(Box::new(MockRawRequester {
        response: Ok(openai_response.clone()),
    }));
    let claude_provider = LLMProvider::Claude(Box::new(MockRawRequester {
        response: Ok(claude_response.clone()),
    }));

    let mut llm = GenericLLM::new(
        openai_provider,
        Arc::new(Mutex::new(MockCache {})),
    );

    let request = ChatRequest::new(
        ModelId::new("gpt-4"),
        vec![Message::user_message("Hello")],
    );

    let (result, _) = llm.make_request(&request).await.unwrap();
    assert_eq!(result.id, "chatcmpl-123");

    llm.set_provider(claude_provider);

    let request = ChatRequest::new(
        ModelId::new("claude-2"),
        vec![Message::user_message("Hello")],
    );

    let (result, _) = llm.make_request(&request).await.unwrap();
    assert_eq!(result.id, "chatcmpl-456");
}