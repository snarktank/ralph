//! Dependency graph construction and analysis

use crate::mcp::tools::load_prd::PrdUserStory;

/// Represents a story node in the dependency graph.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct StoryNode {
    /// Unique story identifier (e.g., "US-001")
    pub id: String,
    /// Story priority (lower values = higher priority)
    pub priority: u32,
    /// Whether the story has already passed
    pub passes: bool,
    /// IDs of stories this story depends on
    pub depends_on: Vec<String>,
    /// Files that this story will modify (for conflict detection)
    pub target_files: Vec<String>,
}

impl From<&PrdUserStory> for StoryNode {
    fn from(story: &PrdUserStory) -> Self {
        StoryNode {
            id: story.id.clone(),
            priority: story.priority,
            passes: story.passes,
            depends_on: story.depends_on.clone(),
            target_files: story.target_files.clone(),
        }
    }
}
