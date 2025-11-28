// --- CLI argument parsing (clap) ---
use clap::{Parser, Subcommand};

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
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, marker::PhantomData};

use rust_openai::types::ChatRequest;
use std::env;

mod types;
pub use types::{BookOutline, ChapterOutline, ReviewResult, SectionOutline};

mod steps;
use steps::{
    BookStatement, CombineChapters, DesignSpine, GenerateChapter, GenerateSummaryParagraph,
    ProjectInit, RebuildBookOutlineJson,
};

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

pub fn get_tool_response<T: serde::de::DeserializeOwned>(
    chat_completion_object: ChatCompletionObject,
    tool_name: &str,
) -> anyhow::Result<T> {
    let function_call_response = chat_completion_object
        .output
        .iter()
        .find(|c| {
            c.output_type.as_deref() == Some("function_call")
                && c.name.as_deref() == Some(tool_name)
        })
        .with_context(|| format!("No function_call output found for tool: {}", tool_name))?;

    let args = function_call_response.arguments.as_ref().with_context(|| {
        format!(
            "No arguments found in function_call output for tool: {}",
            tool_name
        )
    })?;
    let args: T = serde_json::from_str(&args)
        .with_context(|| format!("Failed to parse arguments for tool: {}", tool_name))?;
    Ok(args)
}

pub struct TypedTool<T> {
    _t: PhantomData<T>,
    tool: Tool,
}

impl<T: JsonSchema + serde::de::DeserializeOwned> TypedTool<T> {
    pub fn create(name: &str, description: &str) -> TypedTool<T> {
        let schema = JSONSchema(serde_json::to_value(schema_for!(T)).unwrap());

        let tool = Tool {
            description: Some(description.to_string()),
            name: name.to_string(),
            parameters: Some(schema),
        };
        TypedTool {
            _t: PhantomData,
            tool,
        }
    }

    pub fn create_request(&self, request: ChatRequest) -> ModelToolRequest<T> {
        ModelToolRequest::with_tool(request, self)
    }
}

/// Factory function to create the standard BookOutline tool used across multiple steps.
/// This ensures consistency in tool naming and description.
pub fn create_book_outline_tool() -> TypedTool<BookOutline> {
    TypedTool::<BookOutline>::create(
        "submit_outline",
        "Submit the outline for a new book as a list of chapters. Note: Do not include chapter numbers in the chapter name.",
    )
}

struct ModelToolRequest<T> {
    _t: PhantomData<T>,
    tool_name: String,
    request: ChatRequest,
}

impl<T: JsonSchema + serde::de::DeserializeOwned> ModelToolRequest<T> {
    pub fn make_request(&self, llm: &mut OpenAILLM) -> anyhow::Result<T> {
        let (response, _is_from_cache) = llm.make_request(&self.request)?;
        let result: T = get_tool_response(response, &self.tool_name)?;
        Ok(result)
    }

    pub fn with_tool(request: ChatRequest, tool: &TypedTool<T>) -> ModelToolRequest<T> {
        let tools = vec![tool.tool.clone()];
        let request = request.with_tools(tools);
        ModelToolRequest {
            _t: PhantomData::default(),
            request,
            tool_name: tool.tool.name.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepFile {
    filename: String,
    hash: String,
}

impl StepFile {
    fn from_file(filename: &str) -> anyhow::Result<StepFile> {
        Ok(StepFile {
            filename: filename.to_string(),
            hash: get_file_hash(filename)?,
        })
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

    pub fn get_lifecycle_by_files_and_state_json(
        input_files: &[String],
        key: &str,
    ) -> anyhow::Result<StepLifecycle> {
        let mut missing_files = vec![];
        let mut existing_files = HashMap::new();
        for file in input_files {
            if !std::path::Path::new(file).exists() {
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
    get_file_hash(&s.filename)
        .map(|hash| hash == s.hash)
        .unwrap_or(false)
}

pub fn load_step_state_general<T: serde::de::DeserializeOwned>(
    key: &str,
) -> anyhow::Result<Option<T>> {
    let step_state_file = format!(".booker/{}.stepstate.json", key);
    if !std::path::Path::new(&step_state_file).exists() {
        return Ok(None);
    }
    Ok(serde_json::from_reader(std::fs::File::open(
        &step_state_file,
    )?)?)
}

pub fn write_step_state_general<T: serde::ser::Serialize>(
    key: &str,
    step_state: &T,
) -> anyhow::Result<()> {
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

fn all_steps() -> anyhow::Result<Vec<Step>> {
    let mut steps = vec![
        step("Initialize the project", "init", Box::new(ProjectInit)),
        step(
            "Initialize the book statement",
            "initialize",
            Box::new(BookStatement),
        ),
        step(
            "Rebuild book_outline JSON from markdown",
            "rebuild_outline_json",
            Box::new(RebuildBookOutlineJson::new(
                "book_outline.md",
                "book_outline.json",
                "rebuild_outline_json_custom",
            )),
        ),
        step(
            "Generate summary paragraph",
            "generate_summary",
            Box::new(GenerateSummaryParagraph {}),
        ),
        step(
            "Rebuild book_outline_with_summary JSON from markdown",
            "rebuild_outline_json_2",
            Box::new(RebuildBookOutlineJson::new(
                "book_outline_with_summary.md",
                "book_outline_with_summary.json",
                "rebuild_outline_json_2_custom",
            )),
        ),
        step(
            "Generate design spine statement",
            "design_spine",
            Box::new(DesignSpine),
        ),
        step(
            "Rebuild book_outline_with_spine JSON from markdown",
            "rebuild_outline_json_3",
            Box::new(RebuildBookOutlineJson::new(
                "book_outline_with_spine.md",
                "book_outline_with_spine.json",
                "rebuild_outline_json_3_custom",
            )),
        ),
    ];

    let chapter_count = if std::path::Path::new("book_outline_with_spine.json").exists() {
        let outline_content = std::fs::read("book_outline_with_spine.json")?;
        let outline: BookOutline = serde_json::from_slice(&outline_content)?;
        outline.chapters.as_ref().map_or(0, |c| c.len())
    } else {
        0
    };

    if chapter_count > 0 {
        for i in 1..=chapter_count {
            steps.push(step(
                &format!("Generate chapter {}", i),
                &format!("generate_chapter_{}", i),
                Box::new(GenerateChapter { chapter_index: i }),
            ));
        }

        steps.push(step(
            "Combine chapter outlines",
            "combine_chapters",
            Box::new(CombineChapters { chapter_count }),
        ));
    }

    Ok(steps)
}

fn list_steps() -> anyhow::Result<()> {
    let steps = all_steps()?;
    for step in &steps {
        let lifecycle = step.get_lifecycle()?;
        let (symbol, missing_files) = match lifecycle {
            StepLifecycle::NotRunnable(items) => (".", items),
            StepLifecycle::Runnable => (">", vec![]),
            StepLifecycle::CompleteRunnable => ("✓", vec![]),
            StepLifecycle::CompleteNotRunnable(items) => ("?", items),
        };
        println!("{} {} : {}", symbol, step.key, step.description);
        for f in missing_files {
            println!("  - {}", f);
        }
    }
    Ok(())
}

fn run_step_action(step: &Step, proj: &mut ProjectData) -> anyhow::Result<()> {
    println!("Running '{}'", step.key);
    let step_state = step.action.execute(&step.key, proj)?;
    write_step_state(&step_state)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: Cli = Cli::parse();

    dotenv::dotenv().ok();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
    let llm = OpenAILLM::with_defaults(&openai_api_key)?;
    let config = ProjectConfig::load()?;

    let mut proj = ProjectData { llm, config };
    match cli.command {
        None => {
            println!("No command provided.");
        }
        Some(Commands::List) => {
            println!("Listing all step states:");
            list_steps()?;
        }
        Some(Commands::Run { step }) => {
            println!("Running step '{}'", step);
            let steps = all_steps()?;
            let step = steps
                .iter()
                .find(|s| s.key == step)
                .with_context(|| format!("Step '{}' not found", step))?;
            run_step_action(step, &mut proj)?;
        }
    }

    Ok(())
}
