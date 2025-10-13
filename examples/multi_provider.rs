use rust_openai::llm::claude::ClaudeModelId;
use rust_openai::llm::openai::OpenAIModelId;
use rust_openai::llm::provider::LLMProvider;
use rust_openai::llm::GenericLLM;
use rust_openai::types::{ChatRequest, Message, ModelId};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
    let claude_api_key = env::var("CLAUDE_API_KEY").unwrap();

    let openai_provider = LLMProvider::OpenAI(Box::new(rust_openai::llm::openai::OpenAIRawRequester {
        openai_api_key,
    }));
    let claude_provider = LLMProvider::Claude(Box::new(rust_openai::llm::claude::ClaudeRawRequester {}));

    let mut llm = GenericLLM::new(
        openai_provider,
        Arc::new(Mutex::new(rust_openai::llm::DefaultRequestCache::new(
            Arc::new(Mutex::new(rust_openai::llm::DefaultFS {})),
            std::path::PathBuf::from("cache"),
        ).await?)),
    );

    let openai_request = ChatRequest::new(
        ModelId::OpenAI(OpenAIModelId::Gpt4oMini),
        vec![Message::user_message("Hello from OpenAI!")],
    );

    let (openai_response, from_cache) = llm.make_request(&openai_request).await?;
    println!("OpenAI response (from cache: {}): {:#?}", from_cache, openai_response);

    llm.set_provider(claude_provider);

    let claude_request = ChatRequest::new(
        ModelId::Claude(ClaudeModelId::Claude2),
        vec![Message::user_message("Hello from Claude!")],
    );

    let (claude_response, from_cache) = llm.make_request(&claude_request).await?;
    println!("Claude response (from cache: {}): {:#?}", from_cache, claude_response);

    Ok(())
}