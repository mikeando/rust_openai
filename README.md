# Rust OpenAI Client

This is a Rust client for the OpenAI API. It is currently under development and supports the following features:

- GPT-5 models
- The Responses API
- Tool calling
- Embeddings

## Installation

To use this client, add the following to your `Cargo.toml` file:

```toml
[dependencies]
rust_openai = { git = "https://github.com/example/rust_openai.git" }
```

## Usage

### Chat API

Here is a basic example of how to use the client to make a Chat API request:

```rust
use rust_openai::request::OpenAILLM;
use rust_openai::types::{ChatRequest, Message, ModelId};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let openai_api_key = env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable not set");
    let mut llm = OpenAILLM::with_defaults(&openai_api_key).await?;

    let request: ChatRequest = ChatRequest::new(
        ModelId::Gpt5Mini,
        vec![
            Message::user_message("Hello!"),
        ],
    ).with_instructions("You are a helpful assistant.".to_string());

    let (response, is_from_cache) = llm.make_request(&request).await?;

    println!("Response is from cache: {}", is_from_cache);

    // You can inspect the response object for details
    println!("Response: {:#?}", response);

    // Or get the content of the first message
    if let Some(content) = response.choices.get(0).and_then(|c| c.message.content.as_ref()) {
        println!("Assistant's message: {}", content);
    }

    Ok(())
}
```

### Embedding API

You can also use the client to create text embeddings:

```rust
use rust_openai::embedding;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let openai_api_key = env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable not set");

    let text = "This is an example text to embed.";
    let embedding_vector = embedding::make_uncached_embedding_request(text, &openai_api_key).await?;

    println!("Embedding vector: {:?}", embedding_vector);

    Ok(())
}
```

### Tool Calling

The client supports tool calling, allowing you to define functions that the model can call.

```rust
use rust_openai::{
    request::OpenAILLM,
    types::{JSONSchema, Tool},
};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use rust_openai::types::{ChatRequest, Message, ModelId};
use std::env;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct MyToolArgs {
    /// The name of the person to greet.
    name: String,
    /// The number of times to greet the person.
    repetitions: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let openai_api_key = env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable not set");
    let mut llm = OpenAILLM::with_defaults(&openai_api_key).await?;
    let model_id = ModelId::Gpt5Mini;

    let schema = JSONSchema(serde_json::to_value(schema_for!(MyToolArgs)).unwrap());
    let tools = vec![Tool {
        description: Some("Greets a person a specified number of times.".to_string()),
        name: "greet_person".to_string(),
        parameters: Some(schema),
    }];

    let request: ChatRequest = ChatRequest::new(
        model_id,
        vec![
            Message::user_message("Please greet John Doe 3 times."),
        ],
    )
    .with_instructions("You are a helpful assistant.".to_string())
    .with_tools(tools);

    let (response, _is_from_cache) = llm.make_request(&request).await?;

    if let Some(tool_call) = response.choices.get(0).and_then(|c| c.message.tool_calls.as_ref().and_then(|tc| tc.get(0))) {
        if tool_call.function.name == "greet_person" {
            let args: MyToolArgs = serde_json::from_str(&tool_call.function.arguments).unwrap();
            for _ in 0..args.repetitions {
                println!("Hello, {}!", args.name);
            }
        }
    }

    Ok(())
}
```
