use crate::{BookOutline, ChapterOutline, ProjectData, StepAction, StepFile, StepState, TypedTool};
use rust_openai::types::{ChatRequest, Message, ModelId};

pub struct GenerateChapter {
    pub chapter_index: usize,
}

impl GenerateChapter {
    fn generate_chapter_outline(
        &self,
        proj: &ProjectData,
        args: &BookOutline,
    ) -> anyhow::Result<ChapterOutline> {
        let model_id = ModelId::Gpt5Mini;

        println!("=== processing chapter {}", self.chapter_index);

        let chapter_outline_tool = TypedTool::<ChapterOutline>::create(
            "submit_chapter_outline",
            "Submit a breakdown of a chapter into sections with key points.",
        );

        let overview = args.render_to_markdown();

        let request: ChatRequest = ChatRequest::new(
            model_id,
            vec![
                Message::user_message(format!("Create and submit a list of potential sections to be included in chapter {}, based on the following book overview:\n\n{}", self.chapter_index, overview)),
            ],
        ).with_instructions(proj.config.ai_instruction.clone());
        let request = chapter_outline_tool.create_request(request);

        let breakdown = request.make_request(&proj.llm)?;
        Ok(breakdown)
    }
}

impl StepAction for GenerateChapter {
    fn input_files(&self, _key: &str) -> anyhow::Result<Vec<String>> {
        Ok(vec!["book_outline_with_spine.json".to_string()])
    }

    fn execute(&self, key: &str, proj: &ProjectData) -> anyhow::Result<StepState> {
        let outline_content = std::fs::read("book_outline_with_spine.json")?;
        let args: BookOutline = serde_json::from_slice(&outline_content)?;

        let chapter_breakdown = self.generate_chapter_outline(proj, &args)?;

        let output_filename = format!(".booker/chapter_{}.json", self.chapter_index);
        std::fs::write(
            &output_filename,
            serde_json::to_string_pretty(&chapter_breakdown)?,
        )?;

        Ok(StepState {
            key: key.to_string(),
            inputs: vec![StepFile::from_file("book_outline_with_spine.json")?],
            outputs: vec![StepFile::from_file(&output_filename)?],
        })
    }
}
