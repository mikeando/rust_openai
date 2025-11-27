use anyhow::Result;
use serde::{Deserialize, Serialize};
use rust_openai::types::{ChatRequest, Message, ModelId};

use crate::{
    BookOutline, ProjectData, StepAction, StepFile, StepLifecycle, StepState,
    TypedTool, get_file_hash, load_step_state_general, try_get_file_hash,
    write_step_state_general,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebuildBookOutlineState {
    pub input_markdown_hash: String,
    pub output_json_hash: String,
}

pub struct RebuildBookOutlineJson {
    input_md: String,
    output_json: String,
    state_key: String,
}

impl RebuildBookOutlineJson {
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

    fn execute(&self, key: &str, proj: &mut ProjectData) -> Result<StepState> {
        let model_id = ModelId::Gpt5Nano;
        let outline_tool = TypedTool::<BookOutline>::create(
            "submit_outline",
            "Submit the outline for a new book as a list of chapters. Note: Do not include chapter numbers in the chapter name."
        );
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
        ].join("\n");

        let request = outline_tool.create_request(ChatRequest::new(
            model_id,
            vec![
                Message::user_message(prompt),
            ],
        ).with_instructions("You are a an expert book authoring AI.".to_string())
        );

        let args = request.make_request(&mut proj.llm)?;
        std::fs::write(&self.output_json, serde_json::to_string_pretty(&args)?)?;
        let rebuild_state = RebuildBookOutlineState {
            input_markdown_hash: get_file_hash(&self.input_md)?,
            output_json_hash: get_file_hash(&self.output_json)?,
        };
        write_step_state_general(&self.state_key, &rebuild_state)?;

        Ok(
            StepState { 
                key: key.to_string(), 
                inputs: vec![
                    StepFile::from_file(&self.input_md)?
                ], 
                outputs: vec![
                    StepFile::from_file(&self.output_json)?,
                ] 
            }
        )
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
        let step_state: Option<RebuildBookOutlineState> = load_step_state_general(&self.state_key)?;

        match (step_state, markdown_hash, json_hash) {
            (None, None, None) => Ok(StepLifecycle::NotRunnable(vec![
                self.input_md.clone(),
            ])),
            (Some(state), Some(md_hash), Some(j_hash)) => {
                if state.input_markdown_hash == md_hash && state.output_json_hash == j_hash {
                    // TODO: Differentiate these states better?
                    Ok(StepLifecycle::CompleteRunnable)
                } else {
                    Ok(StepLifecycle::Runnable)
                }
            },
            (_, Some(_), _) => Ok(StepLifecycle::Runnable),
            (_, None, _) => Ok(StepLifecycle::NotRunnable(vec![
                self.input_md.clone(),
            ]))
        }
    }
}
