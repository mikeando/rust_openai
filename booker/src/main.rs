// --- CLI argument parsing (clap) ---
use clap::{Parser, Subcommand};
use std::fmt::Write as _;
use std::io::Write as _;


#[derive(Parser, Debug)]
#[command(name = "booker", about = "Book authoring workflow tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all step states
    List,
    /// Run the next logical step
    Next,
    /// Run an action (prep or run) on a step
    Run {
        /// Step key (e.g. "initialize", "generate_outline")
        step: String,
    },
}
use anyhow::Context;
use data_encoding::HEXLOWER;
use ring::digest;
use rust_openai::{
    request::OpenAILLM,
    types::{ChatCompletionObject, JSONSchema, Tool},
};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, marker::PhantomData};

use rust_openai::types::{ChatRequest, Message, ModelId};
use std::env;

mod steps;
use steps::{RebuildBookOutlineJson, RebuildBookOutlineState};

/// Configuration for the book authoring project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// AI system instruction for the book authoring assistant
    pub ai_instruction: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            ai_instruction: "You are an expert book authoring AI.".to_string(),
        }
    }
}

impl ProjectConfig {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = ".booker/config.json";
        if !std::path::Path::new(config_path).exists() {
            return Ok(Self::default());
        }
        let config: ProjectConfig = serde_json::from_reader(std::fs::File::open(config_path)?)?;
        Ok(config)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(".booker")?;
        let config_path = ".booker/config.json";
        serde_json::to_writer_pretty(std::fs::File::create(config_path)?, self)?;
        Ok(())
    }
}

/// Breakdown of a chapter into sections with overview, key points and notes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
struct ChapterOutline {
    /// chapter title, not including number.
    title: String,
    /// chapter subtitle
    subtitle: String,
    /// chapter overview
    overview: Option<String>,
    /// sections in the chapter
    sections: Option<Vec<SectionOutline>>,
    /// notes for the chapter
    notes: Option<Vec<String>>,
}

/// Outline for a single section of a book within a chapter
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
struct SectionOutline {
    /// section title
    title: String,
    /// key points in the section
    key_points: Vec<String>,
}

/// Response from a review submission
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
struct ReviewResult {
    // overall summary of strengths and weaknesses
    summary: String,
    // individual concrete review suggestions
    suggestions: Vec<String>
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
struct BookOutline {
    /// title of the book
    title: Option<String>,
    
    /// subtitle of the book
    subtitle: Option<String>,

    /// high-level overview, in markdown format
    overview: Option<String>,
    
    /// additional notes, each as a markdown paragraph.
    notes: Option<Vec<String>>,

    /// individual concrete review suggestions
    chapters: Option<Vec<ChapterOutline>>,
} 


impl BookOutline {
    pub fn render_to_markdown(&self) -> String {
        let mut markdown = String::new();
        write!(markdown, "# {}", self.title.as_deref().unwrap_or("Untitled book")).unwrap();
        if let Some(subtitle) = &self.subtitle {
            write!(markdown, ": {}", subtitle).unwrap();
        }
        writeln!(markdown, "\n").unwrap();
        if let Some(overview) = &self.overview {
            markdown.push_str("## Overview\n\n");
            markdown.push_str(overview);
            markdown.push_str("\n\n");
        }

        if let Some(notes) = &self.notes {
            if notes.len() > 0 {
                markdown.push_str("## Additional Notes\n\n");
                for note in notes {
                    markdown.push_str(&format!("{}\n\n", note));
                }
            }
        }

        if let Some(chapters) = &self.chapters {
            for (i, chapter) in chapters.iter().enumerate() {
                let chapter_markdown = chapter.render_to_markdown(i);
                markdown.push_str(&chapter_markdown);
            }
        }
        markdown
    }
}

impl ChapterOutline {
    pub fn render_to_markdown(&self, chapter_index: usize) -> String {
        let mut markdown = String::new();
        markdown.push_str(&format!(
            "## Chapter {}: {} - {}\n\n",
            chapter_index + 1,
            self.title,
            self.subtitle
        ));
        if let Some(overview) = &self.overview {
            markdown.push_str("### Overview\n\n");
            markdown.push_str(overview);
            markdown.push_str("\n\n");
        }
        if let Some(sections) = &self.sections {
            if sections.len() > 0 {
                markdown.push_str("### Sections\n\n");
                for section in sections {
                    markdown.push_str(&format!("#### {}\n", section.title));
                    for point in &section.key_points {
                        markdown.push_str(&format!("- {}\n", point));
                    }
                    markdown.push_str("\n");
                }
            }
        }
        if let Some(notes) = &self.notes {
            if notes.len() > 0 {
                markdown.push_str("### Notes\n\n");
                for note in notes {
                    markdown.push_str(&format!("{}\n\n", note));
                }
            }
        }
        markdown
    }
}

pub fn get_tool_response<T: serde::de::DeserializeOwned>(chat_completion_object: ChatCompletionObject, tool_name: &str) -> anyhow::Result<T> {
    let function_call_response = chat_completion_object.output.iter().find(|c| {
        c.output_type.as_deref() == Some("function_call")
            && c.name.as_deref() == Some(tool_name)
    })
    .with_context(|| format!("No function_call output found for tool: {}", tool_name))?;

    let args =  function_call_response.arguments.as_ref().with_context(|| format!("No arguments found in function_call output for tool: {}", tool_name))?;
    let args: T = serde_json::from_str(&args).with_context(|| format!("Failed to parse arguments for tool: {}", tool_name))?;
    Ok(args)
}

struct TypedTool<T> {
    _t: PhantomData<T>,
    tool: Tool,
}

impl<T: JsonSchema + serde::de::DeserializeOwned> TypedTool<T> {
    pub fn create(
        name: &str,
        description: &str,
    ) -> TypedTool<T> {
        let schema = JSONSchema(serde_json::to_value(schema_for!(T)).unwrap());

        let tool = Tool {
            description: Some(description.to_string()),
            name: name.to_string(),
            parameters: Some(schema),
        };
        TypedTool { _t: PhantomData, tool }
    }

    pub fn create_request(&self, request: ChatRequest) -> ModelToolRequest<T> {
        ModelToolRequest::with_tool(request, self  )
    }
}

struct ModelToolRequest<T> {
    _t: PhantomData<T>,
    tool_name: String,
    request: ChatRequest,
}

impl <T: JsonSchema + serde::de::DeserializeOwned> ModelToolRequest<T> {
    pub fn make_request(&self, llm: &mut OpenAILLM) -> anyhow::Result<T> {
        let (response, _is_from_cache) = llm.make_request(&self.request)?;
        let result: T = get_tool_response(response, &self.tool_name)?;
        Ok(result)
    }

    pub fn with_tool(request: ChatRequest, tool: &TypedTool<T>) -> ModelToolRequest<T> {
        let tools = vec![tool.tool.clone()];
        let request = request.with_tools(tools);
        ModelToolRequest { _t: PhantomData::default(), request, tool_name: tool.tool.name.clone()}
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepFile {
    filename: String,
    hash: String,
}

impl StepFile {
    fn from_file(filename: &str) -> anyhow::Result<StepFile> {
        Ok(
            StepFile { filename: filename.to_string(), hash: get_file_hash(filename)? }
        )
    }
}

pub struct ProjectData {
    llm: OpenAILLM,
    pub config: ProjectConfig,
}

pub trait StepAction {
    fn input_files(&self, key: &str) -> anyhow::Result<Vec<String>>;
    fn execute(&self, key: &str, proj: &mut ProjectData) -> anyhow::Result<StepState>;
    
    /// Default implementation based on input files and step state JSON.
    /// Override this method for steps that need custom lifecycle logic.
    fn get_lifecycle(&self, key: &str) -> anyhow::Result<StepLifecycle> {
        Step::get_lifecycle_by_files_and_state_json(&self.input_files(key)?, key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepState {
    key: String, 
    inputs: Vec<StepFile>,
    outputs: Vec<StepFile>,
}

pub struct Step {
    description: String,
    key: String,
    action: Box<dyn StepAction>,
}
impl Step {

    fn get_lifecycle(&self) -> anyhow::Result<StepLifecycle> {
        self.action.get_lifecycle(&self.key)
    }

    pub fn get_lifecycle_by_files_and_state_json(input_files: &[String], key: &str) -> anyhow::Result<StepLifecycle> {
        let mut missing_files = vec![];
        let mut existing_files = HashMap::new();
        for file in input_files {
            if ! std::path::Path::new(file).exists() {
                missing_files.push(file.to_string());
            } else {
                existing_files.insert(file, get_file_hash(file)?);
            }
        }

        let step_state = load_step_state(key)?;

        match (step_state, missing_files.is_empty()) {
            (None, true) => Ok(StepLifecycle::Runnable),
            (None, false) => Ok(StepLifecycle::NotRunnable(missing_files)),
            (Some(_), true) => Ok(StepLifecycle::CompleteRunnable),
            (Some(_), false) => Ok(StepLifecycle::CompleteNotRunnable(missing_files)),
        }
    }

}

#[derive(Debug)]
pub enum FileState {
    Matching,
    Missing,
    Changed,
}

pub fn try_get_file_hash(filename: &str) -> anyhow::Result<Option<String>> {
    if !std::path::Path::new(filename).exists() {
        return Ok(None);
    }
    let hash = get_file_hash(filename)?;
    Ok(Some(hash))
}

pub fn get_file_hash(filename: &str) -> anyhow::Result<String> {
    let content = std::fs::read(filename)?;
    let digest = digest::digest(&digest::SHA256, content.as_slice());
    let full_hash = HEXLOWER.encode(digest.as_ref());
    let actual_hash = &full_hash[0..32];
    Ok(actual_hash.to_string())
}

pub fn get_file_state(filename: &str, expected_hash: &str) -> anyhow::Result<FileState> {
    if !std::path::Path::new(filename).exists() {
        return Ok(FileState::Missing);
    }
    let actual_hash = get_file_hash(filename)?;
    if actual_hash == expected_hash {
        Ok(FileState::Matching)
    } else {
        Ok(FileState::Changed)
    }
}

// Priority is Missing > Changed > Matching
pub fn get_input_state(inputs: &Vec<StepFile>) -> anyhow::Result<FileState> {
    let mut any_missing = false;
    let mut any_changed = false;
    for input in inputs {
        match get_file_state(&input.filename, &input.hash)? {
            FileState::Missing => any_missing = true,
            FileState::Changed => any_changed = true,
            FileState::Matching => {}
        }
    }
    match (any_missing, any_changed) {
        (true, _) => Ok(FileState::Missing),
        (false, true) => Ok(FileState::Changed),
        (false, false) => Ok(FileState::Matching),
    }
}

pub fn step(description: &str, key: &str, action: Box<dyn StepAction>) -> Step {
    Step {
        description: description.to_string(),
        key: key.to_string(),
        action,
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepLifecycle {
    NotRunnable(Vec<String>),
    Runnable,
    CompleteRunnable,
    CompleteNotRunnable(Vec<String>),
}

pub fn is_file_entry_clean(s: &StepFile) -> bool {
    get_file_hash(&s.filename).map(|hash| hash == s.hash ).unwrap_or(false)
}

pub fn load_step_state_general<T: serde::de::DeserializeOwned>(key: &str) -> anyhow::Result<Option<T>> {
    let step_state_file = format!(".booker/{}.stepstate.json", key);
    if ! std::path::Path::new(&step_state_file).exists() {
        return Ok(None)
    }
    Ok(serde_json::from_reader(std::fs::File::open(&step_state_file)?)?)
}

pub fn write_step_state_general<T: serde::ser::Serialize>(key: &str, step_state: &T) -> anyhow::Result<()> {
    let step_state_file = format!(".booker/{}.stepstate.json", key);
    serde_json::to_writer_pretty(std::fs::File::create(step_state_file)?, step_state)?;
    Ok(())
}

pub fn load_step_state(key: &str) -> anyhow::Result<Option<StepState>> {
    load_step_state_general(key)
}

pub fn write_step_state(step_state: &StepState) -> anyhow::Result<()> {
    write_step_state_general(&step_state.key, step_state)
}



struct ProjectInit;

impl StepAction for ProjectInit {
    fn input_files(&self, key: &str) -> anyhow::Result<Vec<String>> {
        Ok(vec![])
    }

    fn execute(&self, _key: &str, proj: &mut ProjectData) -> anyhow::Result<StepState> {
        // Create the config file
        let config = ProjectConfig::default();
        config.save()?;
        
        // Create the book highlevel file
        let p = "book_highlevel.txt";
        let mut f = std::fs::OpenOptions::new().create(true).write(true).open(p)?;
        writeln!(f,"Subject matter: World building for fantasy and science fiction novels.")?;
        writeln!(f)?;
        writeln!(f, "Target Audience: Professional and experienced authors looking to improve their world building skills.")?;
        drop(f);
        
        // Update proj config
        proj.config = config;
        
        Ok(
            StepState { key: "init".to_string(), inputs: vec![], outputs: vec![
                StepFile::from_file(p)?,
                StepFile::from_file(".booker/config.json")?
            ] }
        )
    }
}

struct BookStatement;

impl StepAction for BookStatement {
    fn input_files(&self, key: &str) -> anyhow::Result<Vec<String>> {
        Ok(vec!["book_highlevel.txt".to_string()])
    }

    fn execute(&self, key: &str, proj: &mut ProjectData) -> anyhow::Result<StepState> {

        let model_id = ModelId::Gpt5Mini;

        let outline_tool = TypedTool::<BookOutline>::create(
            "submit_outline",
            "Submit the outline for a new book as a list of chapters. Note: Do not include chapter numbers in the chapter name."
        );

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

struct GenerateSummaryParagraph;

impl StepAction for GenerateSummaryParagraph {
    fn input_files(&self, key: &str) -> anyhow::Result<Vec<String>> {
        Ok(vec![
            "book_highlevel.txt".to_string(),
            "book_outline.json".to_string()
            ])
    }

    fn execute(&self, key: &str, proj: &mut ProjectData) -> anyhow::Result<StepState> {
        let model_id = ModelId::Gpt5Mini;

        // Load the outline from file
        let outline_content = std::fs::read("book_outline.json")?;
        let args: BookOutline = serde_json::from_slice(&outline_content)?;

        let highlevel_content = String::from_utf8(std::fs::read("book_highlevel.txt")?)?;

        let mut overview = args.render_to_markdown();
        writeln!(overview, "").unwrap();

        let request: ChatRequest = ChatRequest::new(
            model_id,
            vec![
                Message::user_message(format!("Generate a one paragraph description for the following book:\n\n{}\n\n{}", highlevel_content,overview)),
            ],
        ).with_instructions(proj.config.ai_instruction.clone());

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
        std::fs::write("book_outline_with_summary.json", serde_json::to_string_pretty(&args)?)?;
        std::fs::write("book_outline_with_summary.md", args.render_to_markdown())?;
        Ok(
            StepState { key: key.to_string(), inputs: vec![
                StepFile::from_file("book_outline.json")?
            ], outputs: vec![
                StepFile::from_file("book_outline_with_summary.json")?,
                StepFile::from_file("book_outline_with_summary.md")?,
            ] }
        )
    }
}

struct GenerateChapterOutlines;

impl GenerateChapterOutlines {
    fn generate_chapter_outline(&self, proj: &mut ProjectData, args: &BookOutline, chapter_index: usize) -> anyhow::Result<ChapterOutline> {
        let model_id = ModelId::Gpt5Mini;

        println!("=== processing chapter {}", chapter_index);

        let chapter_outline_tool = TypedTool::<ChapterOutline>::create(
            "submit_chapter_outline",
            "Submit a breakdown of a chapter into sections with key points."
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
    fn input_files(&self, key: &str) -> anyhow::Result<Vec<String>> {
        Ok(vec![
            "book_outline_with_summary.json".to_string()
            ])
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

        std::fs::write("book_output_with_chapters.json", serde_json::to_string_pretty(&args)?)?;
        std::fs::write("book_output_with_chapters.md", &args.render_to_markdown())?;
        Ok(
            StepState { key: key.to_string(), inputs: vec![
                StepFile::from_file("book_outline_with_summary.json")?
            ], outputs: vec![
                StepFile::from_file("book_output_with_chapters.json")?,
                StepFile::from_file("book_output_with_chapters.md")?,
            ] }
        )
    }


}



fn all_steps() -> Vec<Step> {
    vec![
        step("Initialize the project", "init", Box::new(ProjectInit)),
        step("Initialize the book statement", "initialize", Box::new(BookStatement)),
        step(
            "Rebuild book_outline JSON from markdown",
            "rebuild_outline_json",
            Box::new(RebuildBookOutlineJson::new(
                "book_outline.md",
                "book_outline.json",
                "rebuild_outline_json_custom"
            ))
        ),
        step("Generate summary paragraph", "generate_summary", Box::new(GenerateSummaryParagraph {})),
        step(
            "Rebuild book_outline_with_summary JSON from markdown",
            "rebuild_outline_json_2",
            Box::new(RebuildBookOutlineJson::new(
                "book_outline_with_summary.md",
                "book_outline_with_summary.json",
                "rebuild_outline_json_2_custom"
            ))
        ),
        step("Generate chapter outlines", "generate_chapter_outlines", Box::new(GenerateChapterOutlines {})),
    ]
}

fn list_steps() -> anyhow::Result<()> {
    let steps = all_steps();
    for step in &steps {
        let lifecycle = step.get_lifecycle()?;
        let (symbol, missing_files) = match lifecycle {
            StepLifecycle::NotRunnable(items) => {
                (".", items)
            },
            StepLifecycle::Runnable => {
                (">", vec![])
            },
            StepLifecycle::CompleteRunnable => {
                ("✓", vec![])
            },
            StepLifecycle::CompleteNotRunnable(items) => {
                ("?", items)
            }
        };
        println!("{} {} : {}", symbol, step.key, step.description);
        for f in missing_files {
            println!("  - {}", f);
        }
    }
    Ok(())
}

fn run_next_logical_step(proj: &mut ProjectData) -> anyhow::Result<()> {
    let steps = all_steps();
    let step = find_next_logical_step(&steps)?;
    run_step_action(&step, proj)?;
    Ok(())
}

fn find_next_logical_step(steps: &Vec<Step>) -> anyhow::Result<Step> {
    todo!("Find the next logical step to run based on step lifecycles");
}

fn run_step_action(step: &Step, proj: &mut ProjectData) -> anyhow::Result<()> {
    println!("Running '{}'", step.key);
    let step_state = step.action.execute(&step.key, proj)?;
    write_step_state(&step_state)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- CLI argument parsing stub ---
    // The following block parses CLI arguments, but does not yet change program flow.
    // Integration with the main logic can be done by branching here in the future.
    let cli: Cli = Cli::parse();

    dotenv::dotenv().ok();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
    let llm = OpenAILLM::with_defaults(&openai_api_key)?;
    let config = ProjectConfig::load()?;

    let mut proj = ProjectData { llm, config };
    match cli.command {
        None => {
            println!("No command provided.");
        },
        Some(Commands::List) => {
            println!("Listing all step states:");
            list_steps()?;
        },
        Some(Commands::Next) => {
            println!("Running next logical step");
            run_next_logical_step(&mut proj)?;
        },
        Some(Commands::Run { step }) => {
            println!("Running step '{}'", step);
            let steps = all_steps();
            let step = steps.iter().find(|s| s.key == step).with_context(|| format!("Step '{}' not found", step))?;
            run_step_action(step, &mut proj)?;
        },
    }

    return Ok(());


    // //
    // // Our next job is to crcitique the book summary in a variety of ways.
    // // We will then rank, combine and apply the top N fixes.
    // //
    // // These axes were generated by asking chatGPT the following:
    // //
    // // > You are going to be given a document containing a book summary with a detailed 
    // // > chapter/section break down for that book. It is a non-fiction book on world building 
    // // > for experienced authors. Your role is to assist the author to improve the final result 
    // // > of his writing process. Consider several different axes on which you could critique the 
    // // > summary document. Note: You will not have access to the final text, just the overall 
    // // > detailed structure of the document and the section/chapter names.
    // //
    // // We could do this iteratively too.
    // let review_axes = vec![
    //     "Structural coherence - logical flow between sections; escalation of concepts; consistent depth per chapter.",
    //     "Conceptual hierarchy - balance between high-level theory and actionable technique; redundancy or missing transitional content.",
    //     "Topical coverage - completeness of worldbuilding domains (sociology, ecology, economics, linguistics, metaphysics, etc.); detection of bias toward one discipline.",
    //     "Reader progression - how concepts scaffold for “experienced authors”; complexity ramp; avoidance of elementary recaps.",
    //     "Originality and perspective - novelty of framing; differentiation from common craft manuals (e.g., The Writer's Guide to Worldbuilding tropes).",
    //     "Integration with creative workflow - linkage between theory and practice; whether sections align with real authorial processes.",
    //     "Pedagogical design - balance of abstract ideas vs. illustrative models, exercises, or frameworks.",
    //     "Internal symmetry - parallel structure between analogous sections (e.g., “building cultures” vs. “building politics”).",
    //     "Tone and audience calibration - assumes authorial sophistication; avoids didactic or condescending tone.",
    //     "Interdisciplinary rigor - application of history, anthropology, or systems theory; whether claims rest on coherent conceptual models.",
    //     "Actionability - how each section yields usable outcomes for a working author.",
    //     "Cohesion of thematic arc - whether book sustains a central thesis about what worldbuilding is or for.",
    // ];

    // let critique_tool = TypedTool::<ReviewResult>::create(
    //     "submit_review",
    //     "Submit a review of a document"
    // );
    // // We're going to use a lighter model here, since there are a lot of tokens.
    // // We could bump this back up later if we find it gives better results.
    // let review_model_id = ModelId::Gpt5Nano;

    // let mut review_entries: Vec<ReviewResult> = Vec::new();

    // println!("=== Generating reviews for the outline");
    // for (i,axis) in review_axes.iter().enumerate() {
    //     println!("=== Requesting review {}/{} based on '{}'", i + 1, review_axes.len(), axis);

    //     // Build the prompt - We structure it a little oddly, with content then task for two reasons.
    //     // 1. LLMs can sometimes forget commands given at the start
    //     // 2. The command will change each iteration, by putting unchanging content at the start
    //     //    we allow prompt-caching to cut in, which for openAI reduces input token costs by a factor 
    //     //    of 10 for the cached tokens.

    //     let mut prompt = String::new();
    //     writeln!(prompt, "The following is a document outline you will be asked to review.").unwrap();
    //     writeln!(prompt, "").unwrap();
    //     writeln!(prompt, "---").unwrap();
    //     writeln!(prompt, "").unwrap();
    //     writeln!(prompt, "{}", args.render_to_markdown()).unwrap();
    //     writeln!(prompt, "").unwrap();
    //     writeln!(prompt, "---").unwrap();
    //     writeln!(prompt, "").unwrap();
    //     writeln!(prompt, "Please review the structure of the outline focusing on '{axis}' and then submit the review using the provided function.").unwrap();
    //     writeln!(prompt, "Ensure you provide a brief overall view of the outline's strengths and weaknesses, as well as concrete actionable suggestions to improve the outline.").unwrap();

    //     // println!("{}", prompt);

    //     let request: ChatRequest = ChatRequest::new(
    //         review_model_id,
    //         vec![
    //             Message::user_message(prompt),
    //         ],
    //     ).with_instructions("Act as an expert book editor. Keep your responses succinct and actionable.".to_string());
    //     let request = critique_tool.create_request(request);
    //     let review: ReviewResult = request.make_request(&mut llm)?;
    //     println!("*** review\n{:#?}", review);

    //     review_entries.push(review);
    // } 

    // // Write all the reviews to a file for later analysis
    // let mut review_markdown = String::new();
    // review_markdown.push_str("# Review Results\n");
    // for (i, review) in review_entries.iter().enumerate() {
    //     review_markdown.push_str(&format!("\n## Review {}\n\n", i + 1));
    //     review_markdown.push_str(&format!("Focus Area: {}\n\n", review_axes[i]));
    //     review_markdown.push_str("### Summary\n\n");
    //     review_markdown.push_str(&format!("{}\n\n", review.summary));
    //     review_markdown.push_str("### Suggestions\n\n");
    //     for suggestion in &review.suggestions {
    //         review_markdown.push_str(&format!("- {}\n", suggestion));
    //     }
    // }
    // std::fs::write("review_results.md", &review_markdown)?;
    // println!("Reviews written to review_results.md");

    // // Combine reviews into a list of unique suggestions
    // let review_suggestions: ReviewResult  = {
    //     let mut prompt: Vec<String> = vec![
    //         "The following are review suggestions for a book summary and chapter breakdown document. ",
    //         "Your task is to combine these into a single list of unique, actionable suggestions, removing duplicates and merging similar points. ",
    //         "Ensure the final list is clear and concise, suitable for guiding improvements to the document outline.",
    //     ].iter().map(|s| s.to_string()).collect();

    //     for(i, review) in review_entries.iter().enumerate() {
    //         prompt.push(String::new());
    //         prompt.push(format!("# Review {}\n\nFocus: {}\n\n{}", i + 1, review_axes[i], review.summary));
    //         prompt.push("\nSuggestions:\n".to_string());
    //         for suggestion in &review.suggestions {
    //             prompt.push(format!("- {}", suggestion));
    //         }
    //     }
    //     let prompt = prompt.join("\n");
    //     println!("=== Combining review suggestions:\n{}", prompt);

    //     let review_combine_tool = TypedTool::<ReviewResult>::create(
    //         "submit_review_suggestions",
    //         "Submit a list of actionable review suggestions"
    //     );

    //     let request: ChatRequest = ChatRequest::new(
    //         review_model_id,
    //         vec![
    //             Message::user_message(prompt),
    //         ],
    //     ).with_instructions("You are an expert book editor.".to_string());
    //     let request = review_combine_tool.create_request(request);

    //     let review: ReviewResult = request.make_request(&mut llm)?;
    //     // This Value should be a list, containing entries with "text" fields.
    //     // We just want to join them together to get the complete summary.
    //     print!("{:#?}", review);
    //     review
    // };

    /* 
    // Now we want to rank these and select the top 5 fixes.
    // We want to focus on high-impact and high-level suggestions.
    let mut prompt = String::new();
    writeln!(prompt, "The following are review suggestions for a book summary and chapter breakdown document:").unwrap();
    writeln!(prompt, "").unwrap();
    for suggestion in &review_suggestions.suggestions {
        writeln!(prompt, "- {}", suggestion).unwrap();
    }
    writeln!(prompt, "").unwrap();
    writeln!(prompt, "Please rank these suggestions based on their potential impact on improving the overall quality of the book summary and chapter breakdown. ").unwrap();
    writeln!(prompt, "Focus on high-level, strategic changes rather than minor edits. ").unwrap();
    writeln!(prompt, "Return the top 5 most impactful suggestions in order of importance. ").unwrap();

    let review_suggestions_schema = JSONSchema(serde_json::to_value(schema_for!(ReviewSuggestions)).unwrap());
    let review_tools = vec![Tool {
        description: Some("Submit a list of actionable review suggestions".to_string()),
        name: "submit_review_suggestions".to_string(),
        parameters: Some(review_suggestions_schema.clone()),
    }];

    let request: ChatRequest = ChatRequest::new(
        model_id,
        vec![
            Message::user_message(prompt),
        ],
    ).with_instructions("You are an expert book editor.".to_string())
    .with_tools(review_tools.clone());
    let (response, _is_from_cache) = llm.make_request(&request)?;
    let review: ReviewSuggestions = get_tool_response(response, "submit_review_suggestions")?;
    println!("RANKED SUGGESTIONS:\n");
    for(i, suggestion) in review.suggestions.iter().enumerate() {
        print!("{}. {}\n", i + 1, suggestion);
    }

    {

        let update_overview_tool = TypedTool::<RevisedOutline>::create(
            "submit_revised_outline",
            "Submits a reviesed book outline based on review suggestions"
        );



        // Apply the suggestions to the outline
        println!("\n=== Applying Top Review Suggestions to Outline ===\n");
        let mut markdown = markdown.clone();
        for suggestion in review.suggestions.iter() {
            let mut prompt = String::new();
            writeln!(prompt, "The following is a book outline:").unwrap();
            writeln!(prompt, "").unwrap();
            writeln!(prompt, "---").unwrap();
            writeln!(prompt, "").unwrap();
            writeln!(prompt, "{markdown}").unwrap();
            writeln!(prompt, "").unwrap();
            writeln!(prompt, "---").unwrap();
            writeln!(prompt, "").unwrap();
            writeln!(prompt, "Please revise the outline based on the following suggestion:").unwrap();
            writeln!(prompt, "{}", suggestion).unwrap();
            writeln!(prompt, "").unwrap();
            writeln!(prompt, "Return the revised outline, including an updated high-level overview in markdown format, a breakdown of chapters and sections, and any additional notes. Use the provided function to submit the revised outline.").unwrap();
            writeln!(prompt, "Be sure that you include the complete revised outline and chapters. Do not just describe the changes you have made. Do not state a section is unchanged - include its original data.").unwrap();
            let request: ChatRequest = ChatRequest::new(
                model_id,
            vec![
                    Message::user_message(prompt),
                ],
            ).with_instructions("You are an expert book editing AI.".to_string());
            let request = update_overview_tool.create_request(request);

            let revised_outline: RevisedOutline = request.make_request(&mut llm)?;
            println!("*** revised outline\n{:#?}", revised_outline);
            // Rebuild the markdown from the revised outline        
            todo!("Rebuild the markdown from the revised outline");    

            println!("=== Revised Outline after applying suggestion:\n{}", markdown);
        }
        std::fs::write("book_output_revised.md", &markdown)?;
    }
    */
    Ok(())
}
