//! Conflict detection for parallel execution
//!
//! This module provides infrastructure for detecting when parallel story executions
//! modify the same files, which could lead to merge conflicts or inconsistent state.

use std::collections::HashSet;

use crate::mcp::tools::executor::ExecutionResult;
use crate::parallel::scheduler::ConflictStrategy;

/// Represents a file conflict between two stories.
///
/// A conflict occurs when two stories both modify the same file,
/// which could lead to merge conflicts or inconsistent state.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    /// The file path that both stories modified.
    pub file: String,
    /// The ID of the first story involved in the conflict.
    pub story_a: String,
    /// The ID of the second story involved in the conflict.
    pub story_b: String,
}

#[allow(dead_code)]
impl Conflict {
    /// Creates a new conflict.
    pub fn new(file: String, story_a: String, story_b: String) -> Self {
        Self {
            file,
            story_a,
            story_b,
        }
    }
}

/// Detects conflicts between parallel story executions.
///
/// The `ConflictDetector` uses a configurable strategy to determine
/// when two stories are in conflict. Currently supports file-based
/// conflict detection, where stories modifying the same files are
/// considered to be in conflict.
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct ConflictDetector {
    /// The strategy used for conflict detection.
    pub strategy: ConflictStrategy,
}

#[allow(dead_code)]
impl ConflictDetector {
    /// Creates a new conflict detector with the given strategy.
    pub fn new(strategy: ConflictStrategy) -> Self {
        Self { strategy }
    }

    /// Creates a new conflict detector with file-based strategy (default).
    pub fn file_based() -> Self {
        Self {
            strategy: ConflictStrategy::FileBased,
        }
    }
}

/// Detects file conflicts between two execution results.
///
/// Compares the `files_changed` sets of two execution results and returns
/// a `Conflict` for each file that appears in both sets.
///
/// # Arguments
///
/// * `a` - The first execution result with its story ID
/// * `b` - The second execution result with its story ID
///
/// # Returns
///
/// A vector of `Conflict` structs, one for each file that was modified
/// by both stories. Returns an empty vector if there are no overlapping files.
///
/// # Example
///
/// ```ignore
/// let result_a = ExecutionResult {
///     files_changed: vec!["src/main.rs".to_string(), "src/lib.rs".to_string()],
///     ..Default::default()
/// };
/// let result_b = ExecutionResult {
///     files_changed: vec!["src/lib.rs".to_string(), "src/utils.rs".to_string()],
///     ..Default::default()
/// };
///
/// let conflicts = detect_file_conflicts(
///     (&result_a, "US-001"),
///     (&result_b, "US-002"),
/// );
///
/// // conflicts contains one Conflict for "src/lib.rs"
/// ```
#[allow(dead_code)]
pub fn detect_file_conflicts(
    a: (&ExecutionResult, &str),
    b: (&ExecutionResult, &str),
) -> Vec<Conflict> {
    let (result_a, story_a) = a;
    let (result_b, story_b) = b;

    // Convert files_changed to HashSets for efficient intersection
    let files_a: HashSet<&String> = result_a.files_changed.iter().collect();
    let files_b: HashSet<&String> = result_b.files_changed.iter().collect();

    // Find overlapping files
    files_a
        .intersection(&files_b)
        .map(|file| Conflict::new((*file).clone(), story_a.to_string(), story_b.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_execution_result(files: Vec<&str>) -> ExecutionResult {
        ExecutionResult {
            success: true,
            commit_hash: None,
            error: None,
            iterations_used: 1,
            gate_results: Vec::new(),
            files_changed: files.into_iter().map(String::from).collect(),
            futility_verdict: None,
            iteration_context: None,
            needs_guidance: false,
        }
    }

    #[test]
    fn test_conflict_new() {
        let conflict = Conflict::new(
            "src/main.rs".to_string(),
            "US-001".to_string(),
            "US-002".to_string(),
        );

        assert_eq!(conflict.file, "src/main.rs");
        assert_eq!(conflict.story_a, "US-001");
        assert_eq!(conflict.story_b, "US-002");
    }

    #[test]
    fn test_conflict_detector_new() {
        let detector = ConflictDetector::new(ConflictStrategy::FileBased);
        assert_eq!(detector.strategy, ConflictStrategy::FileBased);
    }

    #[test]
    fn test_conflict_detector_default() {
        let detector = ConflictDetector::default();
        assert_eq!(detector.strategy, ConflictStrategy::FileBased);
    }

    #[test]
    fn test_conflict_detector_file_based() {
        let detector = ConflictDetector::file_based();
        assert_eq!(detector.strategy, ConflictStrategy::FileBased);
    }

    #[test]
    fn test_detect_file_conflicts_no_overlap() {
        let result_a = make_execution_result(vec!["src/main.rs", "src/lib.rs"]);
        let result_b = make_execution_result(vec!["src/utils.rs", "src/config.rs"]);

        let conflicts = detect_file_conflicts((&result_a, "US-001"), (&result_b, "US-002"));

        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_detect_file_conflicts_single_overlap() {
        let result_a = make_execution_result(vec!["src/main.rs", "src/lib.rs"]);
        let result_b = make_execution_result(vec!["src/lib.rs", "src/utils.rs"]);

        let conflicts = detect_file_conflicts((&result_a, "US-001"), (&result_b, "US-002"));

        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].file, "src/lib.rs");
        assert_eq!(conflicts[0].story_a, "US-001");
        assert_eq!(conflicts[0].story_b, "US-002");
    }

    #[test]
    fn test_detect_file_conflicts_multiple_overlaps() {
        let result_a = make_execution_result(vec!["src/main.rs", "src/lib.rs", "src/config.rs"]);
        let result_b = make_execution_result(vec!["src/lib.rs", "src/config.rs", "src/utils.rs"]);

        let conflicts = detect_file_conflicts((&result_a, "US-001"), (&result_b, "US-002"));

        assert_eq!(conflicts.len(), 2);

        let conflict_files: HashSet<&str> = conflicts.iter().map(|c| c.file.as_str()).collect();
        assert!(conflict_files.contains("src/lib.rs"));
        assert!(conflict_files.contains("src/config.rs"));
    }

    #[test]
    fn test_detect_file_conflicts_empty_files_a() {
        let result_a = make_execution_result(vec![]);
        let result_b = make_execution_result(vec!["src/lib.rs", "src/utils.rs"]);

        let conflicts = detect_file_conflicts((&result_a, "US-001"), (&result_b, "US-002"));

        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_detect_file_conflicts_empty_files_b() {
        let result_a = make_execution_result(vec!["src/main.rs", "src/lib.rs"]);
        let result_b = make_execution_result(vec![]);

        let conflicts = detect_file_conflicts((&result_a, "US-001"), (&result_b, "US-002"));

        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_detect_file_conflicts_both_empty() {
        let result_a = make_execution_result(vec![]);
        let result_b = make_execution_result(vec![]);

        let conflicts = detect_file_conflicts((&result_a, "US-001"), (&result_b, "US-002"));

        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_detect_file_conflicts_complete_overlap() {
        let result_a = make_execution_result(vec!["src/main.rs", "src/lib.rs"]);
        let result_b = make_execution_result(vec!["src/main.rs", "src/lib.rs"]);

        let conflicts = detect_file_conflicts((&result_a, "US-001"), (&result_b, "US-002"));

        assert_eq!(conflicts.len(), 2);
    }

    #[test]
    fn test_conflict_equality() {
        let conflict1 = Conflict::new(
            "src/main.rs".to_string(),
            "US-001".to_string(),
            "US-002".to_string(),
        );
        let conflict2 = Conflict::new(
            "src/main.rs".to_string(),
            "US-001".to_string(),
            "US-002".to_string(),
        );

        assert_eq!(conflict1, conflict2);
    }

    #[test]
    fn test_conflict_clone() {
        let conflict = Conflict::new(
            "src/main.rs".to_string(),
            "US-001".to_string(),
            "US-002".to_string(),
        );
        let cloned = conflict.clone();

        assert_eq!(conflict, cloned);
    }
}
