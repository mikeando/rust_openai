use std::env;

use rust_openai::{
    request::OpenAILLM,
    types::{ChatRequest, Message, ModelId},
};

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
}