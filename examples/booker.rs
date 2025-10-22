use rust_openai::{
    request::OpenAILLM,
    types::{JSONSchema, Tool},
};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

use std::fmt::Write;

use rust_openai::json::ToJson;
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
    let mut llm = OpenAILLM::with_defaults(&openai_api_key).await?;
    let model_id = ModelId::Gpt5Mini;

    let schema2 = JSONSchema(serde_json::to_value(schema_for!(Outline)).unwrap());

    let tools = vec![Tool {
        description: Some("Submit the outline for a new book as a list of chapters".to_string()),
        name: "submit_outline".to_string(),
        parameters: Some(schema2),
    }];
    for tool in &tools {
        assert!(!tool.name.is_empty(), "Tool name must not be empty");
    }
    let request: ChatRequest = ChatRequest::new(
        model_id,
        vec![
            Message::user_message("Generate a outline for the following book, then submit it with the provided function:\n\nSubject matter: World building for fantasy and science fiction novels.\n\nTarget Audience: Professional and experiences authors looking to improve their world building skills."),
        ],
    ).with_instructions("You are a an expert book authoring AI.".to_string())
    .with_tools(tools);

    let (response, _is_from_cache) = llm.make_request(&request).await?;

    let function_call_response = &response.output.iter().find(|c| {
        c.output_type.as_deref() == Some("function_call")
            && c.name.as_deref() == Some("submit_outline")
    });
    let args: Outline = serde_json::from_str(
        function_call_response
            .and_then(|c| c.arguments.as_ref())
            .expect("No function_call output with arguments found"),
    )
    .unwrap();
    println!("=== Generated Outline:");
    println!("{:#?}", args);

    let mut overview = String::new();
    for (i, c) in args.chapters.iter().enumerate() {
        writeln!(overview, "Chapter {}: {} -- {}", i + 1, c.title, c.subtitle).unwrap();
        writeln!(overview, "{}\n", c.overview).unwrap();
    }
    println!("=== Requesting book summary based on outline:");
    println!("{}", overview);

    let request: ChatRequest = ChatRequest::new(
        model_id,
        vec![
            Message::user_message(format!("Generate a one paragraph description for the following book:\n\nSubject matter: World building for fantasy and science fiction novels.\n\nTarget Audience: Professional and experiences authors looking to improve their world building skills.\n\n{}", overview)),
        ],
    ).with_instructions("You are a an expert book authoring AI.".to_string());

    let (response, _is_from_cache) = llm.make_request(&request).await?;

    let summary_response = &response
        .output
        .iter()
        .find(|c| c.output_type.as_deref() == Some("message"))
        .unwrap();

    let summary_pieces = summary_response.content.as_ref().unwrap();
    // This Value should be a list, containing entries with "text" fields.
    // We just want to join them together to get the complete summary.
    let mut summary = String::new();
    if let serde_json::Value::Array(pieces) = summary_pieces {
        for piece in pieces {
            if let Some(text) = piece.get("text").and_then(|t| t.as_str()) {
                summary.push_str(text);
            }
        }
    }

    println!("=== Generated Book Summary:");
    println!("{}", summary);
    println!();
    println!("{}", overview);

    // Break down the first chapter
    let schema2 = JSONSchema(serde_json::to_value(schema_for!(ChapterBreakdown)).unwrap());

    for chapter_index in 1..=args.chapters.len() {
        println!("=== processing chapter {}", chapter_index);

        let tools = vec![Tool {
            description: Some("Submit a list of sections for a chapter".to_string()),
            name: "submit_chapter_outline".to_string(),
            parameters: Some(schema2.clone()),
        }];
        for tool in &tools {
            assert!(!tool.name.is_empty(), "Tool name must not be empty");
        }
        let request: ChatRequest = ChatRequest::new(
            model_id,
            vec![
                Message::user_message(format!("Create and submit a list of potential sections to be included in chapter {}, based on the following book overview:\n\n{}\n\n{}\n", chapter_index, summary, overview )),
            ],
        ).with_instructions("You are a an expert book authoring AI.".to_string())
        .with_tools(tools);

        let (response, _is_from_cache) = llm.make_request(&request).await?;

        let function_call_response = &response.output.iter().find(|c| {
            c.output_type.as_deref() == Some("function_call")
                && c.name.as_deref() == Some("submit_chapter_outline")
        });
        let args: ChapterBreakdown = serde_json::from_str(
            function_call_response
                .and_then(|c| c.arguments.as_ref())
                .expect("No function_call output with arguments found"),
        )
        .unwrap();
        println!("{:#?}", args);
    }

    Ok(())
}
