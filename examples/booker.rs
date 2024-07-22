use rust_openai::{
    request::OpenAILLM,
    types::{JSONSchema, Tool},
};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use tokio;

use std::fmt::Write;

use rust_openai::types::{ChatRequest, Message, ModelId};
use std::env;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
struct Outline {
    chapters: Vec<ChapterOutline>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
struct ChapterOutline {
    title: String,
    subtitle: String,
    overview: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
struct ChapterBreakdown {
    chapter_index: u32,
    chapter_title: String,
    sections: Vec<SectionOutline>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
struct SectionOutline {
    title: String,
    key_points: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
    let mut llm = OpenAILLM::with_defaults(&openai_api_key);
    let model_id = ModelId::Gpt4oMini;

    let schema2 = JSONSchema(serde_json::to_value(schema_for!(Outline)).unwrap());

    let request: ChatRequest = ChatRequest::new(
        model_id,
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

    let (response, is_from_cache) = llm.make_request(&request).await?;

    println!("is from cache: {}", is_from_cache);
    println!("{:#?}", response);

    let args: Outline = serde_json::from_str(
        &response.choices[0]
            .message
            .as_assistant_message()
            .as_ref()
            .unwrap()
            .tool_calls
            .as_ref()
            .unwrap()[0]
            .function
            .arguments,
    )
    .unwrap();
    println!("{:#?}", args);

    let mut overview = String::new();
    for (i, c) in args.chapters.iter().enumerate() {
        writeln!(overview, "Chapter {}: {} -- {}", i + 1, c.title, c.subtitle).unwrap();
        writeln!(overview, "{}\n", c.overview).unwrap();
    }
    println!("{}", overview);

    let request: ChatRequest = ChatRequest::new(
        model_id,
        vec![
            Message::system_message("You are a an expert book authoring AI."),
            Message::user_message(format!("Generate a one paragraph description for the following book:\n\nSubject matter: World building for fantasy and science fiction novels.\n\nTarget Audience: Professional and experiences authors looking to improve their world building skills.\n\n{}", overview)),
        ],
    );

    let (response, is_from_cache) = llm.make_request(&request).await?;

    println!("is from cache: {}", is_from_cache);
    println!("{:#?}", response);

    let summary = response.choices[0]
        .message
        .as_assistant_message()
        .as_ref()
        .unwrap()
        .content
        .as_ref()
        .unwrap();
    println!("{}", summary);
    println!();
    println!("{}", overview);

    // Break down the first chapter
    let schema2 = JSONSchema(serde_json::to_value(schema_for!(ChapterBreakdown)).unwrap());

    for chapter_index in 1..=args.chapters.len() {
        println!("processing chapter {}", chapter_index);

        let request: ChatRequest = ChatRequest::new(
            model_id,
            vec![
                Message::system_message("You are a an expert book authoring AI."),
                Message::user_message(format!("Create a list of potential sections to be included in chapter {}, based on the following book overview:\n\n{}\n\n{}\n", chapter_index, summary, overview )),
            ],
        ).with_tools(vec![
            Tool{
                description: Some("Submit a list of sections for a chapter".to_string()), 
                name: "generate_chapter_outline".to_string(),
                parameters: Some(schema2.clone()),
            }]);

        let (response, _is_from_cache) = llm.make_request(&request).await?;

        let rew_response_object = &response.choices[0]
            .message
            .as_assistant_message()
            .as_ref()
            .unwrap()
            .tool_calls
            .as_ref()
            .unwrap()[0]
            .function
            .arguments;
        let args: ChapterBreakdown = serde_json::from_str(rew_response_object).unwrap();
        println!("{:#?}", args);
    }

    Ok(())
}
