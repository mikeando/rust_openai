use rust_openai::request::OpenAILLM;
use rust_openai::types::{
    ChatRequest, JsonSchemaProp, Message, ModelId, ResponseFormat, JSONSchema,
};
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let mut llm = OpenAILLM::with_defaults(&api_key).await?;

    let messages = vec![
        Message::system_message("You are a helpful assistant that extracts information from text and returns it in JSON format."),
        Message::user_message("Extract the name and age of the person in the following sentence: 'John Doe is 42 years old.'"),
    ];

    let schema = JsonSchemaProp {
        name: "person".to_string(),
        description: Some("Information about a person".to_string()),
        schema: JSONSchema(json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The name of the person"
                },
                "age": {
                    "type": "number",
                    "description": "The age of the person"
                }
            },
            "required": ["name", "age"]
        })),
        strict: Some(true),
    };

    let request = ChatRequest::new(ModelId::Gpt4o, messages)
        .with_max_tokens(Some(128))
        .with_response_format(ResponseFormat::JsonSchema(schema));

    let (response, is_from_cache) = llm.make_request(&request).await?;

    println!("is from cache: {}", is_from_cache);
    println!("{:#?}", response);

    Ok(())
}