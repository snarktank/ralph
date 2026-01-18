//! Dependency graph construction and analysis

use crate::mcp::tools::load_prd::PrdUserStory;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

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

/// A directed acyclic graph representing story dependencies.
///
/// Each node in the graph is a `StoryNode`, and edges represent
/// dependencies (an edge from A to B means A depends on B).
#[allow(dead_code)]
pub struct DependencyGraph {
    /// The underlying directed graph structure
    graph: DiGraph<StoryNode, ()>,
    /// Maps story IDs to their node indices for O(1) lookup
    id_to_node: HashMap<String, NodeIndex>,
}

#[allow(dead_code)]
impl DependencyGraph {
    /// Constructs a dependency graph from a list of user stories.
    ///
    /// This method:
    /// 1. Creates a node for each story
    /// 2. Builds a lookup table mapping story IDs to node indices
    /// 3. Adds directed edges based on `dependsOn` relationships
    ///
    /// # Arguments
    ///
    /// * `stories` - A slice of `PrdUserStory` to build the graph from
    ///
    /// # Returns
    ///
    /// A new `DependencyGraph` containing all stories and their dependencies
    pub fn from_stories(stories: &[PrdUserStory]) -> Self {
        let mut graph = DiGraph::new();
        let mut id_to_node = HashMap::new();

        // First pass: add all nodes
        for story in stories {
            let node = StoryNode::from(story);
            let idx = graph.add_node(node);
            id_to_node.insert(story.id.clone(), idx);
        }

        // Second pass: add edges based on dependsOn relationships
        for story in stories {
            if let Some(&dependent_idx) = id_to_node.get(&story.id) {
                for dep_id in &story.depends_on {
                    if let Some(&dependency_idx) = id_to_node.get(dep_id) {
                        // Edge direction: dependent -> dependency
                        // This means "story depends on dep_id"
                        graph.add_edge(dependent_idx, dependency_idx, ());
                    }
                }
            }
        }

        DependencyGraph { graph, id_to_node }
    }

    /// Returns a reference to the underlying graph.
    pub fn graph(&self) -> &DiGraph<StoryNode, ()> {
        &self.graph
    }

    /// Returns a reference to the ID-to-node mapping.
    pub fn id_to_node(&self) -> &HashMap<String, NodeIndex> {
        &self.id_to_node
    }

    /// Looks up a node index by story ID in O(1) time.
    pub fn get_node_index(&self, id: &str) -> Option<NodeIndex> {
        self.id_to_node.get(id).copied()
    }

    /// Returns the number of stories in the graph.
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Returns the number of dependency edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}
