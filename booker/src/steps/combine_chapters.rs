use crate::{BookOutline, ChapterOutline, ProjectData, StepAction, StepFile, StepState};

pub struct CombineChapters {
    pub chapter_count: usize,
}

impl StepAction for CombineChapters {
    fn input_files(&self, _key: &str) -> anyhow::Result<Vec<String>> {
        let mut files = vec!["book_outline_with_spine.json".to_string()];
        for i in 1..=self.chapter_count {
            files.push(format!(".booker/chapter_{}.json", i));
        }
        Ok(files)
    }

    fn execute(&self, key: &str, _proj: &mut ProjectData) -> anyhow::Result<StepState> {
        let outline_content = std::fs::read("book_outline_with_spine.json")?;
        let mut args: BookOutline = serde_json::from_slice(&outline_content)?;

        let mut chapter_breakdowns = Vec::new();
        for i in 1..=self.chapter_count {
            let chapter_filename = format!(".booker/chapter_{}.json", i);
            let chapter_content = std::fs::read(&chapter_filename)?;
            let chapter_outline: ChapterOutline = serde_json::from_slice(&chapter_content)?;
            chapter_breakdowns.push(chapter_outline);
        }
        args.chapters = Some(chapter_breakdowns);

        std::fs::write(
            "book_output_with_chapters.json",
            serde_json::to_string_pretty(&args)?,
        )?;
        std::fs::write("book_output_with_chapters.md", &args.render_to_markdown())?;

        let mut inputs = vec![StepFile::from_file("book_outline_with_spine.json")?];
        for i in 1..=self.chapter_count {
            inputs.push(StepFile::from_file(&format!(
                ".booker/chapter_{}.json",
                i
            ))?);
        }

        Ok(StepState {
            key: key.to_string(),
            inputs,
            outputs: vec![
                StepFile::from_file("book_output_with_chapters.json")?,
                StepFile::from_file("book_output_with_chapters.md")?,
            ],
        })
    }
}
