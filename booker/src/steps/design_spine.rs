use crate::steps::rebuild_outline_json::RebuildBookOutlineState;
use crate::write_step_state_general;
use crate::{BookOutline, ProjectData, StepAction, StepFile, StepState, TypedTool};
use rust_openai::types::{ChatRequest, Message, ModelId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Response structure for the design spine statement generation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
struct DesignSpineResponse {
    /// The design spine statement (1-2 sentences)
    design_spine: String,
}

/// Generate a design spine statement that captures the core thesis of the book.
///
/// A design spine is a concise 1-2 sentence statement that distills the book's
/// essential purpose and perspective. It serves as a north star for all subsequent
/// chapter development, ensuring coherence and focus.
///
/// # Inputs
/// - `book_outline_with_summary.json` - Book outline with summary paragraph
///
/// # Outputs
/// - `book_outline_with_spine.json` - Book outline with design spine statement added
/// - `book_outline_with_spine.md` - Markdown representation with spine statement
///
/// # AI Model
/// Uses GPT-5 Mini for design spine generation.
///
/// # Process
/// 1. Loads the book outline with summary
/// 2. Prompts AI to distill the core thesis into 1-2 sentences
/// 3. Augments the outline with the design spine statement
/// 4. Outputs both JSON and markdown versions
pub struct DesignSpine;

impl StepAction for DesignSpine {
    fn input_files(&self, _key: &str) -> anyhow::Result<Vec<String>> {
        Ok(vec!["book_outline_with_summary.json".to_string()])
    }

    fn execute(&self, key: &str, proj: &ProjectData) -> anyhow::Result<StepState> {
        let model_id = ModelId::Gpt5Mini;

        // Load the outline with summary
        let outline_content = std::fs::read("book_outline_with_summary.json")?;
        let mut outline: BookOutline = serde_json::from_slice(&outline_content)?;

        // Create the tool for design spine submission
        let design_spine_tool = TypedTool::<DesignSpineResponse>::create(
            "submit_design_spine",
            "Submit the design spine statement for the book - a concise 1-2 sentence statement capturing the core thesis.",
        );

        // Build the prompt
        let prompt = format!(
            r#"You are analyzing a book outline to distill its core thesis into a concise design spine statement.

{}

Create a design spine statement of 1-2 sentences that captures:
1. The core subject matter and its essential nature
2. The key insight, transformation, or perspective the book offers
3. What readers should understand or be able to do after reading

The statement should be concise, clear, and serve as a north star for all subsequent chapter development.

Examples of good design spine statements:
- "Worldbuilding is not just creating details, but designing coherent systems. This book shows experienced authors how to build interconnected worlds that enhance rather than distract from narrative."
- "Traditional productivity advice fails because it ignores human psychology. We demonstrate how aligning tasks with natural cognitive patterns creates sustainable high performance."
- "Software architecture is about making decisions that maximize future flexibility. This book teaches you to identify and defer the decisions that matter most."

Submit your design spine statement using the provided function."#,
            outline.render_to_markdown()
        );

        let request = design_spine_tool.create_request(
            ChatRequest::new(model_id, vec![Message::user_message(prompt)])
                .with_instructions(proj.config.ai_instruction.clone()),
        );

        let response: DesignSpineResponse = request.make_request(&proj.llm)?;
        
        println!("Generated design spine: {}", response.design_spine);

        // Add the design spine to the outline
        outline.design_spine = Some(response.design_spine);

        // Write the updated outline
        let outline_json = serde_json::to_string_pretty(&outline)?;
        std::fs::write("book_outline_with_spine.json", &outline_json)?;
        
        let outline_markdown = outline.render_to_markdown();
        std::fs::write("book_outline_with_spine.md", &outline_markdown)?;

        // Update the rebuild state for the next rebuild step
        let rebuild_state = RebuildBookOutlineState {
            input_markdown_hash: crate::get_file_hash("book_outline_with_spine.md")?,
            output_json_hash: crate::get_file_hash("book_outline_with_spine.json")?,
        };
        write_step_state_general("rebuild_outline_json_3_custom", &rebuild_state)?;

        Ok(StepState {
            key: key.to_string(),
            inputs: vec![StepFile::from_file("book_outline_with_summary.json")?],
            outputs: vec![
                StepFile::from_file("book_outline_with_spine.json")?,
                StepFile::from_file("book_outline_with_spine.md")?,
            ],
        })
    }
}
