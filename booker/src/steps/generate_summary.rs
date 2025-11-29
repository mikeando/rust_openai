use crate::{BookOutline, ProjectData, StepAction, StepFile, StepState};
use rust_openai::types::{ChatRequest, Message, ModelId};
use std::fmt::Write;

/// Generate a one-paragraph book summary.
///
/// Uses AI to create a concise, professional summary of the book based on
/// the high-level description and the chapter outline. This summary provides
/// an overview of what the book will cover.
///
/// # Inputs
/// - `book_highlevel.txt` - Original high-level book description
/// - `book_outline.json` - Structured chapter outline
///
/// # Outputs
/// - `book_outline_with_summary.json` - Outline augmented with the overview paragraph
/// - `book_outline_with_summary.md` - Markdown representation with summary
///
/// # AI Model
/// Uses GPT-5 Mini for summary generation.
///
/// # Process
/// 1. Loads the existing book outline
/// 2. Combines high-level description with chapter overview
/// 3. Prompts AI to generate a one-paragraph description
/// 4. Augments the outline with the generated summary
pub struct GenerateSummaryParagraph;

impl StepAction for GenerateSummaryParagraph {
    fn input_files(&self, _key: &str) -> anyhow::Result<Vec<String>> {
        Ok(vec![
            "book_highlevel.txt".to_string(),
            "book_outline.json".to_string(),
        ])
    }

    fn execute(&self, key: &str, proj: &ProjectData) -> anyhow::Result<StepState> {
        let model_id = ModelId::Gpt5Mini;

        // Load the outline from file
        let outline_content = std::fs::read("book_outline.json")?;
        let args: BookOutline = serde_json::from_slice(&outline_content)?;

        let highlevel_content = String::from_utf8(std::fs::read("book_highlevel.txt")?)?;

        let mut overview = args.render_to_markdown();
        writeln!(overview, "").unwrap();

        let request: ChatRequest = ChatRequest::new(
            model_id,
            vec![Message::user_message(format!(
                "Generate a one paragraph description for the following book:\n\n{}\n\n{}",
                highlevel_content, overview
            ))],
        )
        .with_instructions(proj.config.ai_instruction.clone());

        let (response, _is_from_cache) = proj.llm.make_request(&request)?;

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

        let mut args = args;
        args.overview = Some(summary.clone());
        std::fs::write(
            "book_outline_with_summary.json",
            serde_json::to_string_pretty(&args)?,
        )?;
        std::fs::write("book_outline_with_summary.md", args.render_to_markdown())?;
        Ok(StepState {
            key: key.to_string(),
            inputs: vec![StepFile::from_file("book_outline.json")?],
            outputs: vec![
                StepFile::from_file("book_outline_with_summary.json")?,
                StepFile::from_file("book_outline_with_summary.md")?,
            ],
        })
    }
}
