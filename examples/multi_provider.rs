use rust_openai::llm::claude::{ClaudeLlm, ClaudeModelId};
use rust_openai::llm::openai::{OpenAILlm, OpenAIModelId};
use rust_openai::types::{ChatRequest, Message};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
    let claude_api_key = env::var("CLAUDE_API_KEY").unwrap();

    let mut openai_llm = OpenAILlm::new(&openai_api_key, OpenAIModelId::Gpt4oMini).await?;
    let mut claude_llm = ClaudeLlm::new(&claude_api_key, ClaudeModelId::Claude2).await?;

    let openai_request = ChatRequest::new(vec![Message::user_message("Hello from OpenAI!")]);
    let (openai_response, from_cache) = openai_llm.make_request(&openai_request).await?;
    println!("OpenAI response (from cache: {}): {:#?}", from_cache, openai_response);

    let claude_request = ChatRequest::new(vec![Message::user_message("Hello from Claude!")]);
    let (claude_response, from_cache) = claude_llm.make_request(&claude_request).await?;
    println!("Claude response (from cache: {}): {:#?}", from_cache, claude_response);

    Ok(())
}