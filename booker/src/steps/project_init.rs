use crate::{ProjectConfig, ProjectData, StepAction, StepFile, StepState};
use std::io::Write;

/// Initial project setup step.
///
/// Creates the necessary project configuration files and initializes
/// the workspace for a new book authoring project.
///
/// # Outputs
/// - `.booker/config.json` - Project configuration with AI instructions
/// - `book_highlevel.txt` - Initial high-level book description with subject matter and target audience
///
/// # Example
/// This step is typically run first when starting a new book project.
pub struct ProjectInit;

impl StepAction for ProjectInit {
    fn input_files(&self, _key: &str) -> anyhow::Result<Vec<String>> {
        Ok(vec![])
    }

    fn execute(&self, _key: &str, proj: &mut ProjectData) -> anyhow::Result<StepState> {
        // Create the config file
        let config = ProjectConfig::default();
        config.save()?;

        // Create the book highlevel file
        let p = "book_highlevel.txt";
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(p)?;
        writeln!(
            f,
            "Subject matter: World building for fantasy and science fiction novels."
        )?;
        writeln!(f)?;
        writeln!(
            f,
            "Target Audience: Professional and experienced authors looking to improve their world building skills."
        )?;
        drop(f);

        // Update proj config
        proj.config = config;

        Ok(StepState {
            key: "init".to_string(),
            inputs: vec![],
            outputs: vec![
                StepFile::from_file(p)?,
                StepFile::from_file(".booker/config.json")?,
            ],
        })
    }
}
