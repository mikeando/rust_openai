use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

/// Breakdown of a chapter into sections with overview, key points and notes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ChapterOutline {
    /// chapter title, not including number.
    pub title: String,
    /// chapter subtitle
    pub subtitle: String,
    /// chapter overview
    pub overview: Option<String>,
    /// sections in the chapter
    pub sections: Option<Vec<SectionOutline>>,
    /// notes for the chapter
    pub notes: Option<Vec<String>>,
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

/// Outline for a single section of a book within a chapter
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SectionOutline {
    /// section title
    pub title: String,
    /// key points in the section
    pub key_points: Vec<String>,
}

/// Response from a review submission
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReviewResult {
    /// overall summary of strengths and weaknesses
    pub summary: String,
    /// individual concrete review suggestions
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BookOutline {
    /// title of the book
    pub title: Option<String>,

    /// subtitle of the book
    pub subtitle: Option<String>,

    /// high-level overview, in markdown format
    pub overview: Option<String>,

    /// design spine statement - 1-2 sentences capturing the core thesis of the book
    pub design_spine: Option<String>,

    /// additional notes, each as a markdown paragraph.
    pub notes: Option<Vec<String>>,

    /// individual concrete review suggestions
    pub chapters: Option<Vec<ChapterOutline>>,
}

impl BookOutline {
    pub fn render_to_markdown(&self) -> String {
        let mut markdown = String::new();
        write!(
            markdown,
            "# {}",
            self.title.as_deref().unwrap_or("Untitled book")
        )
        .unwrap();
        if let Some(subtitle) = &self.subtitle {
            write!(markdown, ": {}", subtitle).unwrap();
        }
        writeln!(markdown, "\n").unwrap();
        if let Some(overview) = &self.overview {
            markdown.push_str("## Overview\n\n");
            markdown.push_str(overview);
            markdown.push_str("\n\n");
        }

        if let Some(design_spine) = &self.design_spine {
            markdown.push_str("## Design Spine\n\n");
            markdown.push_str(design_spine);
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
