use crate::{BookOutline, ChapterOutline, ProjectData, StepAction, StepFile, StepState, TypedTool};
use rust_openai::types::{ChatRequest, Message, ModelId};

/// Generate detailed section breakdowns for each chapter.
///
/// For each chapter in the book outline, this step uses AI to create
/// a detailed breakdown of sections and key points that should be covered.
/// This provides a more granular structure for the book content.
///
/// # Inputs
/// - `book_outline_with_summary.json` - Book outline with summary paragraph
///
/// # Outputs
/// - `book_output_with_chapters.json` - Complete outline with chapter section breakdowns
/// - `book_output_with_chapters.md` - Markdown representation with all chapter details
///
/// # AI Model
/// Uses GPT-5 Mini for chapter breakdown generation.
///
/// # Process
/// 1. Loads the book outline with summary
/// 2. Iterates through each chapter sequentially
/// 3. For each chapter, prompts AI to create a section breakdown
/// 4. Augments the outline with detailed chapter information
///
/// # TODO
/// - Parallelize chapter processing for better performance
/// - Better structure prompts for token reuse across chapters
pub struct GenerateChapterOutlines;

impl GenerateChapterOutlines {
    /// Generate a detailed section breakdown for a single chapter.
    ///
    /// # Arguments
    /// - `proj` - Project data containing LLM client and configuration
    /// - `args` - Complete book outline for context
    /// - `chapter_index` - 1-based chapter number to process
    ///
    /// # Returns
    /// A `ChapterOutline` containing sections and key points for the chapter.
    fn generate_chapter_outline(
        &self,
        proj: &mut ProjectData,
        args: &BookOutline,
        chapter_index: usize,
    ) -> anyhow::Result<ChapterOutline> {
        let model_id = ModelId::Gpt5Mini;

        println!("=== processing chapter {}", chapter_index);

        let chapter_outline_tool = TypedTool::<ChapterOutline>::create(
            "submit_chapter_outline",
            "Submit a breakdown of a chapter into sections with key points.",
        );

        let overview = args.render_to_markdown();

        // TODO: Better structure the prompt for more reuse of the tokens.
        let request: ChatRequest = ChatRequest::new(
            model_id,
            vec![
                Message::user_message(format!("Create and submit a list of potential sections to be included in chapter {}, based on the following book overview:\n\n{}", chapter_index, overview)),
            ],
        ).with_instructions(proj.config.ai_instruction.clone());
        let request = chapter_outline_tool.create_request(request);

        let breakdown = request.make_request(&mut proj.llm)?;
        Ok(breakdown)
    }
}

impl StepAction for GenerateChapterOutlines {
    fn input_files(&self, _key: &str) -> anyhow::Result<Vec<String>> {
        Ok(vec!["book_outline_with_summary.json".to_string()])
    }

    fn execute(&self, key: &str, proj: &mut ProjectData) -> anyhow::Result<StepState> {
        // let model_id = ModelId::Gpt5Mini;
        // Load the outline from file
        let outline_content = std::fs::read("book_outline_with_summary.json")?;
        let mut args: BookOutline = serde_json::from_slice(&outline_content)?;

        // Break down the chapters
        // TODO: Parallelize this
        let mut chapter_breakdowns = Vec::new();
        let chapters = args.chapters.clone().unwrap();
        for chapter_index in 1..=chapters.len() {
            chapter_breakdowns.push(self.generate_chapter_outline(proj, &args, chapter_index)?);
        }
        args.chapters = Some(chapter_breakdowns);

        std::fs::write(
            "book_output_with_chapters.json",
            serde_json::to_string_pretty(&args)?,
        )?;
        std::fs::write("book_output_with_chapters.md", &args.render_to_markdown())?;
        Ok(StepState {
            key: key.to_string(),
            inputs: vec![StepFile::from_file("book_outline_with_summary.json")?],
            outputs: vec![
                StepFile::from_file("book_output_with_chapters.json")?,
                StepFile::from_file("book_output_with_chapters.md")?,
            ],
        })
    }
}
