use anyhow::Result;
use rust_openai::types::{ChatRequest, Message, ModelId};
use serde::{Deserialize, Serialize};

use crate::{
    create_book_outline_tool, get_file_hash, load_step_state_general, try_get_file_hash,
    write_step_state_general, ProjectData, StepAction, StepFile, StepLifecycle, StepState,
};

/// State tracking for markdown-to-JSON rebuild operations.
///
/// Stores file hashes to detect when input markdown has changed and
/// determine if the JSON output needs to be regenerated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebuildBookOutlineState {
    /// Hash of the input markdown file
    pub input_markdown_hash: String,
    /// Hash of the output JSON file
    pub output_json_hash: String,
}

/// Convert markdown book outline to structured JSON format.
///
/// This step uses AI to parse a markdown representation of a book outline
/// and convert it into a structured JSON format. It's designed to be reusable
/// for multiple markdown-to-JSON conversion scenarios.
///
/// # Use Cases
/// - Regenerate JSON after manual edits to markdown files
/// - Convert human-readable outlines into machine-processable format
/// - Maintain synchronization between markdown and JSON representations
///
/// # Smart Caching
/// Uses file hash comparison to avoid unnecessary regeneration:
/// - Tracks input markdown hash and output JSON hash
/// - Only reruns if markdown has changed or JSON is missing/modified
/// - Returns `CompleteRunnable` if hashes match (no action needed)
///
/// # AI Model
/// Uses GPT-5 Nano (faster, cheaper) since this is a straightforward conversion task.
pub struct RebuildBookOutlineJson {
    input_md: String,
    output_json: String,
    state_key: String,
}

impl RebuildBookOutlineJson {
    /// Create a new markdown-to-JSON rebuild step.
    ///
    /// # Arguments
    /// - `input_md` - Path to the input markdown file
    /// - `output_json` - Path to the output JSON file
    /// - `state_key` - Unique key for tracking this step's state
    ///
    /// # Example
    /// ```ignore
    /// let step = RebuildBookOutlineJson::new(
    ///     "book_outline.md",
    ///     "book_outline.json",
    ///     "rebuild_outline_json"
    /// );
    /// ```
    pub fn new(input_md: &str, output_json: &str, state_key: &str) -> Self {
        Self {
            input_md: input_md.to_string(),
            output_json: output_json.to_string(),
            state_key: state_key.to_string(),
        }
    }
}

impl StepAction for RebuildBookOutlineJson {
    fn input_files(&self, _key: &str) -> Result<Vec<String>> {
        Ok(vec![self.input_md.clone()])
    }

    fn execute(&self, key: &str, proj: &ProjectData) -> anyhow::Result<StepState> {
        let model_id = ModelId::Gpt5Nano;
        let outline_tool = create_book_outline_tool();
        let content = std::fs::read(&self.input_md)?;

        let prompt = [
            "Submit the partial book summary below to the subnission function. Do not make any changes, just resubmit it as-is:",
            "",
            "---",
            "",
            std::str::from_utf8(&content)?,
            "",
            "---",
            "",
            "NOTE: Not all fields of the outline need to be filled in yet - only those actually present in the markdown.",
        ]
        .join("\n");

        let request = outline_tool.create_request(
            ChatRequest::new(model_id, vec![Message::user_message(prompt)])
                .with_instructions(proj.config.ai_instruction.clone()),
        );

        let args = request.make_request(&proj.llm)?;
        std::fs::write(&self.output_json, serde_json::to_string_pretty(&args)?)?;
        let rebuild_state = RebuildBookOutlineState {
            input_markdown_hash: get_file_hash(&self.input_md)?,
            output_json_hash: get_file_hash(&self.output_json)?,
        };
        write_step_state_general(&self.state_key, &rebuild_state)?;

        Ok(StepState {
            key: key.to_string(),
            inputs: vec![StepFile::from_file(&self.input_md)?],
            outputs: vec![StepFile::from_file(&self.output_json)?],
        })
    }

    fn get_lifecycle(&self, _key: &str) -> Result<StepLifecycle> {
        // This step is a bit different...
        // it is done if
        //   * the input and output files exist and their hashes match the stored values.
        // otherwise it is not done.

        // check if the input/output files exist
        let markdown_hash = try_get_file_hash(&self.input_md)?;
        let json_hash = try_get_file_hash(&self.output_json)?;

        // Load the step metadata
        let step_state: Option<RebuildBookOutlineState> =
            load_step_state_general(&self.state_key)?;

        match (step_state, markdown_hash, json_hash) {
            (None, None, None) => Ok(StepLifecycle::NotRunnable(vec![self.input_md.clone()])),
            (Some(state), Some(md_hash), Some(j_hash)) => {
                if state.input_markdown_hash == md_hash && state.output_json_hash == j_hash {
                    // TODO: Differentiate these states better?
                    Ok(StepLifecycle::CompleteRunnable)
                } else {
                    Ok(StepLifecycle::Runnable)
                }
            }
            (_, Some(_), _) => Ok(StepLifecycle::Runnable),
            (_, None, _) => Ok(StepLifecycle::NotRunnable(vec![self.input_md.clone()])),
        }
    }
}
