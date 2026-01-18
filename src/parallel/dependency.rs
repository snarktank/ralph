//! Dependency graph construction and analysis

use crate::mcp::tools::load_prd::PrdUserStory;
use crate::parallel::inference::infer_from_files;
use petgraph::algo::{is_cyclic_directed, toposort};
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

    /// Looks up a story node by ID in O(1) time.
    pub fn get_story(&self, id: &str) -> Option<&StoryNode> {
        self.id_to_node.get(id).map(|&idx| &self.graph[idx])
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

    /// Returns story IDs in a valid execution order (dependencies before dependents).
    ///
    /// Uses topological sorting to determine an order where all dependencies
    /// of a story are executed before the story itself.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` containing story IDs in execution order
    /// * `Err(DependencyError::CycleDetected)` if the graph contains cycles
    pub fn topological_order(&self) -> Result<Vec<String>, DependencyError> {
        // toposort returns nodes in dependency order (dependencies first)
        // Since our edges point from dependent -> dependency, we need to reverse the result
        match toposort(&self.graph, None) {
            Ok(indices) => {
                // Reverse to get dependencies before dependents
                let order: Vec<String> = indices
                    .into_iter()
                    .rev()
                    .map(|idx| self.graph[idx].id.clone())
                    .collect();
                Ok(order)
            }
            Err(_) => {
                // Cycle detected - collect participants for error message
                let cycle_stories = self.find_cycle_participants();
                Err(DependencyError::CycleDetected(cycle_stories))
            }
        }
    }

    /// Returns stories that are ready to execute.
    ///
    /// A story is ready when:
    /// - It has not already passed (`passes == false`)
    /// - It is not in the `completed` set
    /// - All its dependencies are in the `completed` set
    ///
    /// # Arguments
    ///
    /// * `completed` - Set of story IDs that have been completed in this execution
    ///
    /// # Returns
    ///
    /// A vector of references to `StoryNode`s that are ready to execute
    pub fn get_ready_stories(
        &self,
        completed: &std::collections::HashSet<String>,
    ) -> Vec<&StoryNode> {
        self.graph
            .node_indices()
            .filter_map(|idx| {
                let node = &self.graph[idx];
                // Skip if already passed or already completed
                if node.passes || completed.contains(&node.id) {
                    return None;
                }
                // Check if all dependencies are completed
                let all_deps_completed = node.depends_on.iter().all(|dep| completed.contains(dep));
                if all_deps_completed {
                    Some(node)
                } else {
                    None
                }
            })
            .collect()
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

    /// Infers dependencies from target file overlaps and adds them to the graph.
    ///
    /// This method analyzes the `target_files` patterns of all stories in the graph
    /// and adds edges for stories that have overlapping patterns. Higher-priority
    /// stories (lower priority number) become dependencies of lower-priority stories.
    ///
    /// Explicit `dependsOn` relationships take precedence - this method will not
    /// duplicate edges that already exist in the graph.
    pub fn infer_dependencies(&mut self) {
        // Collect all story nodes for inference
        let stories: Vec<StoryNode> = self
            .graph
            .node_indices()
            .map(|idx| self.graph[idx].clone())
            .collect();

        // Get inferred dependencies
        let inferred = infer_from_files(&stories);

        // Add edges for inferred dependencies, avoiding duplicates
        for (dependent_id, dependency_id) in inferred {
            if let (Some(&dependent_idx), Some(&dependency_idx)) = (
                self.id_to_node.get(&dependent_id),
                self.id_to_node.get(&dependency_id),
            ) {
                // Check if edge already exists (explicit dependsOn takes precedence)
                let edge_exists = self
                    .graph
                    .edges(dependent_idx)
                    .any(|e| e.target() == dependency_idx);

                if !edge_exists {
                    self.graph.add_edge(dependent_idx, dependency_idx, ());
                    // Also update the node's depends_on list for consistency
                    if let Some(node) = self.graph.node_weight_mut(dependent_idx) {
                        if !node.depends_on.contains(&dependency_id) {
                            node.depends_on.push(dependency_id);
                        }
                    }
                }
            }
        }
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
    fn test_from_stories_simple_3_story_dag() {
        // Create a simple 3-story DAG:
        //
        //   US-001 (no deps)
        //      |
        //   US-002 (depends on US-001)
        //      |
        //   US-003 (depends on US-002)
        //
        let stories = vec![
            make_story("US-001", vec![]),
            make_story("US-002", vec!["US-001"]),
            make_story("US-003", vec!["US-002"]),
        ];

        let graph = DependencyGraph::from_stories(&stories);

        // Verify node count
        assert_eq!(graph.node_count(), 3, "Should have 3 nodes");

        // Verify edge count (2 edges: US-002->US-001, US-003->US-002)
        assert_eq!(graph.edge_count(), 2, "Should have 2 edges");

        // Verify all nodes are accessible by ID
        assert!(graph.get_node_index("US-001").is_some());
        assert!(graph.get_node_index("US-002").is_some());
        assert!(graph.get_node_index("US-003").is_some());

        // Verify node contents
        let node1 = graph.get_story("US-001").unwrap();
        assert_eq!(node1.id, "US-001");
        assert!(node1.depends_on.is_empty());

        let node2 = graph.get_story("US-002").unwrap();
        assert_eq!(node2.id, "US-002");
        assert_eq!(node2.depends_on, vec!["US-001"]);

        let node3 = graph.get_story("US-003").unwrap();
        assert_eq!(node3.id, "US-003");
        assert_eq!(node3.depends_on, vec!["US-002"]);

        // Verify edges are correctly formed
        let us002_idx = graph.get_node_index("US-002").unwrap();
        let us001_idx = graph.get_node_index("US-001").unwrap();
        let us003_idx = graph.get_node_index("US-003").unwrap();

        // US-002 should have an edge to US-001 (dependency)
        let us002_has_edge_to_us001 = graph
            .graph()
            .edges(us002_idx)
            .any(|e| e.target() == us001_idx);
        assert!(us002_has_edge_to_us001, "US-002 should depend on US-001");

        // US-003 should have an edge to US-002 (dependency)
        let us003_has_edge_to_us002 = graph
            .graph()
            .edges(us003_idx)
            .any(|e| e.target() == us002_idx);
        assert!(us003_has_edge_to_us002, "US-003 should depend on US-002");

        // The graph should be a valid DAG
        assert!(graph.validate().is_ok(), "Graph should be a valid DAG");
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

    #[test]
    fn test_topological_order_multi_level_dependencies() {
        // Create a multi-level dependency graph:
        //
        //   US-001 (no deps)
        //      |
        //   US-002 (depends on US-001)
        //      |
        //   US-003 (depends on US-002)
        //      |
        //   US-004 (depends on US-003)
        //
        // Also add US-005 which depends on both US-001 and US-003 (diamond pattern)
        //
        // Valid orders must have: US-001 before US-002 before US-003 before US-004
        // And: US-001 before US-005, US-003 before US-005
        let stories = vec![
            make_story("US-004", vec!["US-003"]),
            make_story("US-002", vec!["US-001"]),
            make_story("US-005", vec!["US-001", "US-003"]),
            make_story("US-001", vec![]),
            make_story("US-003", vec!["US-002"]),
        ];

        let graph = DependencyGraph::from_stories(&stories);
        let result = graph.topological_order();

        assert!(result.is_ok());
        let order = result.unwrap();

        // Verify all stories are present
        assert_eq!(order.len(), 5);

        // Helper to get position in order
        let pos = |id: &str| order.iter().position(|s| s == id).unwrap();

        // Verify dependency ordering constraints
        // US-001 must come before US-002
        assert!(
            pos("US-001") < pos("US-002"),
            "US-001 should come before US-002"
        );
        // US-002 must come before US-003
        assert!(
            pos("US-002") < pos("US-003"),
            "US-002 should come before US-003"
        );
        // US-003 must come before US-004
        assert!(
            pos("US-003") < pos("US-004"),
            "US-003 should come before US-004"
        );
        // US-001 must come before US-005
        assert!(
            pos("US-001") < pos("US-005"),
            "US-001 should come before US-005"
        );
        // US-003 must come before US-005
        assert!(
            pos("US-003") < pos("US-005"),
            "US-003 should come before US-005"
        );
    }

    #[test]
    fn test_topological_order_cycle_fails() {
        // Cycle: US-001 -> US-002 -> US-001
        let stories = vec![
            make_story("US-001", vec!["US-002"]),
            make_story("US-002", vec!["US-001"]),
        ];

        let graph = DependencyGraph::from_stories(&stories);
        let result = graph.topological_order();

        assert!(result.is_err());
        match result {
            Err(DependencyError::CycleDetected(_)) => {}
            _ => panic!("Expected CycleDetected error"),
        }
    }

    #[test]
    fn test_topological_order_empty_graph() {
        let stories: Vec<PrdUserStory> = vec![];
        let graph = DependencyGraph::from_stories(&stories);

        let result = graph.topological_order();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    /// Helper function to create a test story with passes flag
    fn make_story_with_passes(id: &str, depends_on: Vec<&str>, passes: bool) -> PrdUserStory {
        PrdUserStory {
            id: id.to_string(),
            title: format!("Story {}", id),
            description: String::new(),
            acceptance_criteria: vec![],
            priority: 1,
            passes,
            depends_on: depends_on.into_iter().map(String::from).collect(),
            target_files: vec![],
        }
    }

    #[test]
    fn test_get_ready_stories_partially_completed() {
        use std::collections::HashSet;

        // Create a dependency graph:
        //   US-001 (no deps, passes=true - already passed)
        //   US-002 (depends on US-001)
        //   US-003 (depends on US-001)
        //   US-004 (depends on US-002 and US-003)
        //   US-005 (no deps)
        let stories = vec![
            make_story_with_passes("US-001", vec![], true), // Already passed
            make_story_with_passes("US-002", vec!["US-001"], false),
            make_story_with_passes("US-003", vec!["US-001"], false),
            make_story_with_passes("US-004", vec!["US-002", "US-003"], false),
            make_story_with_passes("US-005", vec![], false), // No deps
        ];

        let graph = DependencyGraph::from_stories(&stories);

        // Scenario 1: Nothing completed yet
        // - US-001 passes=true, so not ready
        // - US-002 depends on US-001, but US-001 not in completed set -> not ready
        // - US-003 depends on US-001, but US-001 not in completed set -> not ready
        // - US-004 depends on US-002, US-003 -> not ready
        // - US-005 has no deps and passes=false -> ready
        let completed: HashSet<String> = HashSet::new();
        let ready = graph.get_ready_stories(&completed);
        let ready_ids: Vec<&str> = ready.iter().map(|n| n.id.as_str()).collect();
        assert_eq!(ready_ids, vec!["US-005"]);

        // Scenario 2: US-001 completed (but it already passed, so it's in completed for other stories)
        // - US-002 depends on US-001 which is now completed -> ready
        // - US-003 depends on US-001 which is now completed -> ready
        // - US-004 deps not all completed -> not ready
        // - US-005 no deps -> ready
        let mut completed: HashSet<String> = HashSet::new();
        completed.insert("US-001".to_string());
        let ready = graph.get_ready_stories(&completed);
        let mut ready_ids: Vec<&str> = ready.iter().map(|n| n.id.as_str()).collect();
        ready_ids.sort();
        assert_eq!(ready_ids, vec!["US-002", "US-003", "US-005"]);

        // Scenario 3: US-001 and US-002 completed
        // - US-003 -> ready (US-001 completed)
        // - US-004 deps are US-002 (completed) and US-003 (not completed) -> not ready
        // - US-005 -> ready
        let mut completed: HashSet<String> = HashSet::new();
        completed.insert("US-001".to_string());
        completed.insert("US-002".to_string());
        let ready = graph.get_ready_stories(&completed);
        let mut ready_ids: Vec<&str> = ready.iter().map(|n| n.id.as_str()).collect();
        ready_ids.sort();
        assert_eq!(ready_ids, vec!["US-003", "US-005"]);

        // Scenario 4: US-001, US-002, US-003 completed
        // - US-004 -> ready (all deps completed)
        // - US-005 -> ready
        let mut completed: HashSet<String> = HashSet::new();
        completed.insert("US-001".to_string());
        completed.insert("US-002".to_string());
        completed.insert("US-003".to_string());
        let ready = graph.get_ready_stories(&completed);
        let mut ready_ids: Vec<&str> = ready.iter().map(|n| n.id.as_str()).collect();
        ready_ids.sort();
        assert_eq!(ready_ids, vec!["US-004", "US-005"]);

        // Scenario 5: All completed
        // - No stories ready (all in completed set)
        let mut completed: HashSet<String> = HashSet::new();
        completed.insert("US-001".to_string());
        completed.insert("US-002".to_string());
        completed.insert("US-003".to_string());
        completed.insert("US-004".to_string());
        completed.insert("US-005".to_string());
        let ready = graph.get_ready_stories(&completed);
        assert!(ready.is_empty());
    }

    /// Helper function to create a test story with target files
    fn make_story_with_files(
        id: &str,
        priority: u32,
        depends_on: Vec<&str>,
        target_files: Vec<&str>,
    ) -> PrdUserStory {
        PrdUserStory {
            id: id.to_string(),
            title: format!("Story {}", id),
            description: String::new(),
            acceptance_criteria: vec![],
            priority,
            passes: false,
            depends_on: depends_on.into_iter().map(String::from).collect(),
            target_files: target_files.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn test_infer_dependencies_adds_edges_correctly() {
        // Create stories with overlapping target files but no explicit dependencies
        // US-001 (priority 1): targets src/lib.rs
        // US-002 (priority 2): targets src/lib.rs (overlaps with US-001)
        // US-003 (priority 3): targets tests/test.rs (no overlap)
        let stories = vec![
            make_story_with_files("US-001", 1, vec![], vec!["src/lib.rs"]),
            make_story_with_files("US-002", 2, vec![], vec!["src/lib.rs"]),
            make_story_with_files("US-003", 3, vec![], vec!["tests/test.rs"]),
        ];

        let mut graph = DependencyGraph::from_stories(&stories);

        // Before inference: no edges
        assert_eq!(graph.edge_count(), 0, "Should have no edges initially");

        // Infer dependencies
        graph.infer_dependencies();

        // After inference: US-002 should depend on US-001 due to overlapping target files
        assert_eq!(
            graph.edge_count(),
            1,
            "Should have one inferred edge from US-002 to US-001"
        );

        // Verify the edge is correct: US-002 -> US-001 (US-002 depends on US-001)
        let us002_idx = graph.get_node_index("US-002").unwrap();
        let us001_idx = graph.get_node_index("US-001").unwrap();
        let has_edge = graph
            .graph()
            .edges(us002_idx)
            .any(|e| e.target() == us001_idx);
        assert!(
            has_edge,
            "US-002 should have an edge to US-001 (dependency)"
        );

        // Verify the node's depends_on was updated
        let us002_node = &graph.graph()[us002_idx];
        assert!(
            us002_node.depends_on.contains(&"US-001".to_string()),
            "US-002's depends_on should include US-001"
        );

        // US-003 should have no dependencies (no overlap with others)
        let us003_idx = graph.get_node_index("US-003").unwrap();
        let us003_edges: Vec<_> = graph.graph().edges(us003_idx).collect();
        assert!(
            us003_edges.is_empty(),
            "US-003 should have no dependency edges"
        );
    }

    #[test]
    fn test_infer_dependencies_does_not_duplicate_explicit_edges() {
        // Create stories where US-002 already explicitly depends on US-001
        // and they also have overlapping target files
        let stories = vec![
            make_story_with_files("US-001", 1, vec![], vec!["src/lib.rs"]),
            make_story_with_files("US-002", 2, vec!["US-001"], vec!["src/lib.rs"]),
        ];

        let mut graph = DependencyGraph::from_stories(&stories);

        // Before inference: one explicit edge (US-002 -> US-001)
        assert_eq!(graph.edge_count(), 1, "Should have one explicit edge");

        // Infer dependencies
        graph.infer_dependencies();

        // After inference: still only one edge (no duplicate added)
        assert_eq!(
            graph.edge_count(),
            1,
            "Should still have only one edge (no duplicate)"
        );
    }
}
