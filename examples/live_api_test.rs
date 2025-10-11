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

    for model in ModelId::values() {
        println!("Testing model: {}", model.name());

        let request = ChatRequest::new(
            model,
            vec![Message::user_message("What is the capital of France?")],
        );

        let (response, cached) = llm
            .make_request(&request)
            .await
            .expect("Failed to make request");

        assert!(!cached, "Expected a live API call, not a cached response");
        assert_eq!(
            response.model, model,
            "Response model does not match request model"
        );
        assert!(!response.choices.is_empty(), "Expected at least one choice");

        let choice = &response.choices[0];
        let content = choice
            .message
            .as_assistant_message()
            .unwrap()
            .content
            .as_ref()
            .unwrap();

        println!("Response: {}", content);
        assert!(
            content.to_lowercase().contains("paris"),
            "Expected response to contain 'paris'"
        );
    }

    test_tool_calling(&mut llm).await;
}

async fn test_tool_calling(llm: &mut OpenAILLM) {
    println!("Testing tool calling");

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

    let request: ChatRequest = ChatRequest::new(
        ModelId::Gpt4o,
        vec![Message::user_message(
            "What is the weather like in Boston?",
        )],
    )
    .with_tool_choice(ToolChoice::Auto)
    .with_tools(vec![Tool {
        description: Some("Get the current weather in a given location".to_string()),
        name: "get_current_weather".to_string(),
        parameters: Some(
            rust_openai::types::JSONSchema::from_json(&parameters).unwrap(),
        ),
    }]);

    let (response, cached) = llm
        .make_request(&request)
        .await
        .expect("Failed to make request");

    assert!(!cached, "Expected a live API call, not a cached response");
    assert_eq!(
        response.model,
        ModelId::Gpt4o,
        "Response model does not match request model"
    );
    assert!(
        !response.choices.is_empty(),
        "Expected at least one choice"
    );

    let choice = &response.choices[0];
    let assistant_message = choice.message.as_assistant_message().unwrap();
    let tool_calls = assistant_message.tool_calls.as_ref().unwrap();
    assert_eq!(tool_calls.len(), 1, "Expected one tool call");
    let tool_call = &tool_calls[0];
    assert_eq!(
        tool_call.function.name, "get_current_weather",
        "Unexpected tool call"
    );
    println!("Tool call response: {:?}", tool_call);
}