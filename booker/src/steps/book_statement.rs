use rust_openai::types::{ChatRequest, Message, ModelId};
use crate::{StepAction, StepState, StepFile, ProjectData, create_book_outline_tool, get_file_hash};
use crate::steps::rebuild_outline_json::RebuildBookOutlineState;
use crate::write_step_state_general;

pub struct BookStatement;

impl StepAction for BookStatement {
    fn input_files(&self, _key: &str) -> anyhow::Result<Vec<String>> {
        Ok(vec!["book_highlevel.txt".to_string()])
    }

    fn execute(&self, key: &str, proj: &mut ProjectData) -> anyhow::Result<StepState> {

        let model_id = ModelId::Gpt5Mini;

        let outline_tool = create_book_outline_tool();

        let content = std::fs::read("book_highlevel.txt")?;

        let prompt = [
            "Generate a chapter list for the following book, then submit it with the provided function:",
            "",
            std::str::from_utf8(&content)?,
            "",
            "Only provide the chapter titles and subtitles in your response, other fields will be filled in later.",
        ].join("\n");
        let request = outline_tool.create_request(ChatRequest::new(
            model_id,
            vec![
                Message::user_message(prompt),
            ],
        ).with_instructions(proj.config.ai_instruction.clone())
        );

        let args = request.make_request(&mut proj.llm)?;
        // write the outline to file as markdown, and as JSON

        let outline_markdown = args.render_to_markdown();
        std::fs::write("book_outline.md", &outline_markdown)?;
        std::fs::write("book_outline.json", serde_json::to_string_pretty(&args)?)?;
        // TODO: Write a note associating the JSON with the outline so that we don't need to run the outline.md -> outline.json step.

        let rebuild_state = RebuildBookOutlineState {
            input_markdown_hash: get_file_hash("book_outline.md")?,
            output_json_hash: get_file_hash("book_outline.json")?,
        };
        write_step_state_general("rebuild_outline_json_custom", &rebuild_state)?;

        Ok(
            StepState { key: key.to_string(), inputs: vec![
                StepFile::from_file("book_highlevel.txt")?
            ], outputs: vec![
                StepFile::from_file("book_outline.md")?,
                StepFile::from_file("book_outline.json")?,
            ] }
        )
    }
}
