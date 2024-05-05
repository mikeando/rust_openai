use rust_openai::request::OpenAILLM;
use tokio;

use rust_openai::types::{ChatRequest, Message, ModelId};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
    eprintln!("{:?}", openai_api_key);
    let mut llm = OpenAILLM::with_defaults(&openai_api_key);

    let request: ChatRequest = ChatRequest::new(
        ModelId::Gpt35Turbo,
        vec![
            Message::system_message("You are a helpful assistant."),
            Message::user_message("Hello!"),
        ],
    );

    let (response, is_from_cache) = llm.make_request(&request).await;

    println!("is from cache: {}", is_from_cache);
    println!("{:#?}", response);

    Ok(())
}
