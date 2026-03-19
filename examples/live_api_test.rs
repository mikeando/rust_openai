use std::env;

use rust_openai::{
    json::FromJson,
    request::OpenAILLM,
    types::{ChatRequest, Message, ModelId, Tool, ToolChoice},
};
use serde_json::json;

// To run this example, you must have an OPENAI_API_KEY environment variable set.
//
// This example is an integration test because it makes real calls to the OpenAI backend.
// It is designed to be run manually to verify that the client can successfully
// interact with the API for all supported models.
//
// To run this example, use the following command:
// OPENAI_API_KEY="YOUR_KEY" cargo run --example live_api_test
//
// This application will panic if the OPENAI_API_KEY is not set.
#[tokio::main]
async fn main() {
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            panic!("Error: OPENAI_API_KEY environment variable not set.");
        }
    };

    let mut llm = OpenAILLM::with_defaults(&api_key)
        .await
        .expect("Failed to create OpenAILLM");

    let parameters = json!({
        "type": "object",
        "properties": {
            "location": {
                "type": "string",
                "description": "The city and state, e.g. San Francisco, CA"
            },
            "unit": {
                "type": "string",
                "enum": ["celsius", "fahrenheit"]
            }
        },
        "required": ["location"]
    });

    let weather_tool = Tool {
        description: Some("Get the current weather in a given location".to_string()),
        name: "get_current_weather".to_string(),
        parameters: Some(rust_openai::types::JSONSchema::from_json(&parameters).unwrap()),
    };

    for model in ModelId::values() {
        println!("\n--- Testing model: {} ---", model.name());
        println!("1. Testing simple chat completion...");

        let chat_request = ChatRequest::new(
            model,
            vec![Message::user_message("What is the capital of France?")],
        );

        match llm.make_request(&chat_request).await {
            Ok((response, cached)) => {
                assert!(!cached, "Expected a live API call, not a cached response");
                assert_eq!(
                    response.model, model,
                    "Response model does not match request model"
                );
                assert!(!response.choices.is_empty(), "Expected at least one choice");
                let choice = &response.choices[0];
                if let Some(content) = choice
                    .message
                    .as_assistant_message()
                    .unwrap()
                    .content
                    .as_ref()
                {
                    println!("  Response: {}", content.trim());
                    assert!(
                        content.to_lowercase().contains("paris"),
                        "Expected response to contain 'paris'"
                    );
                } else {
                    println!("  Model returned no content.");
                }
            }
            Err(e) => {
                println!(
                    "  [WARN] Simple chat request failed for model {}: {:?}",
                    model.name(),
                    e
                );
            }
        }

        println!("2. Testing tool calling...");

        let tool_request = ChatRequest::new(
            model,
            vec![Message::user_message(
                "What is the weather like in Boston?",
            )],
        )
        .with_tool_choice(ToolChoice::Auto)
        .with_tools(vec![weather_tool.clone()]);

        match llm.make_request(&tool_request).await {
            Ok((response, cached)) => {
                assert!(!cached, "Expected a live API call, not a cached response");
                let choice = &response.choices[0];
                let assistant_message = choice.message.as_assistant_message().unwrap();
                if let Some(tool_calls) = assistant_message.tool_calls.as_ref() {
                    if !tool_calls.is_empty() {
                        println!("  Model supports tool calling. Response: {:?}", tool_calls);
                    } else {
                        println!("  Model does not support tool calling (empty tool_calls).");
                    }
                } else {
                    println!("  Model does not support tool calling (no tool_calls field).");
                }
            }
            Err(e) => {
                println!(
                    "  [WARN] Tool calling request failed for model {}: {:?}",
                    model.name(),
                    e
                );
            }
        }
    }
}