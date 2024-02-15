use rust_openai::{json::ToJson, request::make_request, types::{JSONSchema, Tool}};
use serde::{Deserialize, Serialize};
use serde_json::json;
use schemars::{schema_for, JsonSchema};
use tokio;

use std::fmt::Write;

use rust_openai::types::{ChatRequest, Message, ModelId};
use std::env;

#[derive(Debug,Clone,PartialEq,Eq)]
#[derive(Serialize,Deserialize)]
#[derive(JsonSchema)]
struct Outline {
    chapters: Vec<ChapterOutline>
}

#[derive(Debug,Clone,PartialEq,Eq)]
#[derive(Serialize,Deserialize)]
#[derive(JsonSchema)]
struct ChapterOutline {
    title: String,
    subtitle: String,
    overview: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();

     let schema2 = JSONSchema(serde_json::to_value(schema_for!(Outline)).unwrap());

    let request: ChatRequest = ChatRequest::new(
        ModelId::Gpt35Turbo,
        vec![
            Message::system_message("You are a an expert book authoring AI."),
            Message::user_message("Generate a outline for the following book:\n\nSubject matter: World building for fantasy and science fiction novels.\n\nTarget Audience: Professional and experiences authors looking to improve their world building skills."),
        ],
    ).with_tools(vec![
        Tool{ 
            description: Some("Create the outline for a new book as a list of chapters".to_string()), 
            name: "generate_outline".to_string(),
            parameters: Some(schema2),
        }]);

    let (response, is_from_cache) = make_request(&request, &openai_api_key).await;

    println!("is from cache: {}", is_from_cache);
    println!("{:#?}", response);

    let args: Outline = serde_json::from_str(&response.choices[0].message.as_assistant_message().as_ref().unwrap().tool_calls.as_ref().unwrap()[0].function.arguments).unwrap();
    println!("{:#?}", args);

    let mut overview = String::new();
    for (i, c) in args.chapters.iter().enumerate() {
        writeln!(overview, "Chapter {}: {} -- {}", i+1, c.title, c.subtitle);
        writeln!(overview, "{}\n", c.overview);
    }
    println!("{}", overview);

    let request: ChatRequest = ChatRequest::new(
        ModelId::Gpt35Turbo,
        vec![
            Message::system_message("You are a an expert book authoring AI."),
            Message::user_message(format!("Generate a one paragraph description for the following book:\n\nSubject matter: World building for fantasy and science fiction novels.\n\nTarget Audience: Professional and experiences authors looking to improve their world building skills.\n\n{}", overview)),
        ],
    );

    let (response, is_from_cache) = make_request(&request, &openai_api_key).await;

    println!("is from cache: {}", is_from_cache);
    println!("{:#?}", response);

    let summary = response.choices[0].message.as_assistant_message().as_ref().unwrap().content.as_ref().unwrap();
    println!("{}", summary);
    println!();
    println!("{}", overview);

    Ok(())
}