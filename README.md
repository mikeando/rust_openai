# Rust OpenAI Client

This is a Rust client for the OpenAI API. It is currently under development and supports the following features:

- GPT-5 models
- The Responses API
- Tool calling

## Installation

To use this client, add the following to your `Cargo.toml` file:

```toml
[dependencies]
rust_openai = { git = "https://github.com/mikeando/rust_openai.git" }
```

## Usage

Here is a basic example of how to use the client to make a request to the API:

```rust
use rust_openai::request::OpenAILLM;
use rust_openai::types::{ChatRequest, Message, ModelId};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
    let mut llm = OpenAILLM::with_defaults(&openai_api_key).await?;

    let request: ChatRequest = ChatRequest::new(
        ModelId::Gpt5Mini,
        vec![
            Message::user_message("Hello!"),
        ],
    ).with_instructions("You are a helpful assistant.".to_string());

    let (response, is_from_cache) = llm.make_request(&request).await?;

    println!("is from cache: {}", is_from_cache);
    println!("{:#?}", response);

    Ok(())
}
```
