//! Dependency graph construction and analysis

use crate::mcp::tools::load_prd::PrdUserStory;
use petgraph::algo::is_cyclic_directed;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during dependency graph operations.
#[derive(Debug, Error, PartialEq)]
pub enum DependencyError {
    /// A circular dependency was detected in the graph.
    #[error("Cycle detected involving stories: {}", .0.join(", "))]
    CycleDetected(Vec<String>),
}

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

    /// Validates the dependency graph by checking for cycles.
    ///
    /// A valid dependency graph must be a directed acyclic graph (DAG).
    /// Circular dependencies are invalid because they create unresolvable
    /// execution order.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the graph is a valid DAG
    /// * `Err(DependencyError::CycleDetected)` if circular dependencies exist,
    ///   containing the IDs of all stories involved in cycles
    pub fn validate(&self) -> Result<(), DependencyError> {
        if is_cyclic_directed(&self.graph) {
            // Collect all story IDs involved in cycles
            let cycle_stories = self.find_cycle_participants();
            Err(DependencyError::CycleDetected(cycle_stories))
        } else {
            Ok(())
        }
    }

    /// Finds all story IDs that participate in cycles.
    ///
    /// Uses a simple approach: a node is in a cycle if it can reach itself
    /// through the graph edges.
    fn find_cycle_participants(&self) -> Vec<String> {
        use petgraph::algo::has_path_connecting;

        let mut cycle_ids: Vec<String> = Vec::new();

        for node_idx in self.graph.node_indices() {
            // Check if any successor can reach back to this node
            for edge in self.graph.edges(node_idx) {
                let target = edge.target();
                if has_path_connecting(&self.graph, target, node_idx, None) {
                    let story_id = &self.graph[node_idx].id;
                    if !cycle_ids.contains(story_id) {
                        cycle_ids.push(story_id.clone());
                    }
                    break;
                }
            }
        }

        cycle_ids.sort();
        cycle_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a test story with dependencies
    fn make_story(id: &str, depends_on: Vec<&str>) -> PrdUserStory {
        PrdUserStory {
            id: id.to_string(),
            title: format!("Story {}", id),
            description: String::new(),
            acceptance_criteria: vec![],
            priority: 1,
            passes: false,
            depends_on: depends_on.into_iter().map(String::from).collect(),
            target_files: vec![],
        }
    }

    #[test]
    fn test_validate_cyclic_dependencies_fails() {
        // Create a cycle: US-001 -> US-002 -> US-003 -> US-001
        let stories = vec![
            make_story("US-001", vec!["US-002"]),
            make_story("US-002", vec!["US-003"]),
            make_story("US-003", vec!["US-001"]),
        ];

        let graph = DependencyGraph::from_stories(&stories);
        let result = graph.validate();

        assert!(result.is_err());
        match result {
            Err(DependencyError::CycleDetected(ids)) => {
                assert_eq!(ids.len(), 3);
                assert!(ids.contains(&"US-001".to_string()));
                assert!(ids.contains(&"US-002".to_string()));
                assert!(ids.contains(&"US-003".to_string()));
            }
            _ => panic!("Expected CycleDetected error"),
        }
    }

    #[test]
    fn test_validate_valid_dag_passes() {
        // Create a valid DAG: US-003 -> US-002 -> US-001 (linear chain)
        let stories = vec![
            make_story("US-001", vec![]),
            make_story("US-002", vec!["US-001"]),
            make_story("US-003", vec!["US-002"]),
        ];

        let graph = DependencyGraph::from_stories(&stories);
        let result = graph.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_empty_graph_passes() {
        let stories: Vec<PrdUserStory> = vec![];
        let graph = DependencyGraph::from_stories(&stories);

        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_validate_no_dependencies_passes() {
        // Stories with no dependencies are a valid DAG
        let stories = vec![
            make_story("US-001", vec![]),
            make_story("US-002", vec![]),
            make_story("US-003", vec![]),
        ];

        let graph = DependencyGraph::from_stories(&stories);
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_validate_self_cycle_fails() {
        // A story that depends on itself
        let stories = vec![make_story("US-001", vec!["US-001"])];

        let graph = DependencyGraph::from_stories(&stories);
        let result = graph.validate();

        assert!(result.is_err());
        match result {
            Err(DependencyError::CycleDetected(ids)) => {
                assert!(ids.contains(&"US-001".to_string()));
            }
            _ => panic!("Expected CycleDetected error"),
        }
    }
}
