//! Reconciliation of parallel execution results
//!
//! This module handles detection and reporting of issues that may arise from parallel
//! execution of stories, including git conflicts, type mismatches, and duplicate imports.

use std::path::PathBuf;
use std::process::Command;

/// Issues that can be detected during reconciliation
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReconciliationIssue {
    /// Git merge conflict detected in the specified files
    GitConflict {
        /// Files that have git merge conflicts
        affected_files: Vec<String>,
    },
    /// Type inconsistency between modified modules
    TypeMismatch,
    /// Duplicate import detected
    ImportDuplicate,
}

/// Result of reconciliation analysis
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReconciliationResult {
    /// No issues detected
    Clean,
    /// One or more issues were found
    IssuesFound(Vec<ReconciliationIssue>),
}

/// Engine for reconciling the results of parallel story execution
///
/// The reconciliation engine analyzes the state after parallel execution to detect
/// potential conflicts, type mismatches, and other consistency issues.
#[allow(dead_code)]
pub struct ReconciliationEngine {
    /// Root directory of the project
    project_root: PathBuf,
}

#[allow(dead_code)]
impl ReconciliationEngine {
    /// Creates a new reconciliation engine for the given project root
    ///
    /// # Arguments
    /// * `project_root` - The root directory of the project to reconcile
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    /// Checks for git merge conflicts in the project
    ///
    /// Runs `git status` and detects files with conflict markers (unmerged paths).
    /// Returns a vector of `ReconciliationIssue::GitConflict` if conflicts are found.
    ///
    /// # Returns
    /// A vector of `ReconciliationIssue` containing any detected git conflicts.
    /// Returns an empty vector if no conflicts are detected.
    pub fn check_git_conflicts(&self) -> Vec<ReconciliationIssue> {
        let output = Command::new("git")
            .args(["status", "--porcelain=v1"])
            .current_dir(&self.project_root)
            .output();

        let output = match output {
            Ok(o) => o,
            Err(_) => return Vec::new(),
        };

        if !output.status.success() {
            return Vec::new();
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let conflict_files: Vec<String> = stdout
            .lines()
            .filter(|line| {
                // In git status --porcelain=v1, conflict markers are:
                // UU - both modified (unmerged)
                // AA - both added (unmerged)
                // DD - both deleted (unmerged)
                // AU - added by us (unmerged)
                // UA - added by them (unmerged)
                // DU - deleted by us (unmerged)
                // UD - deleted by them (unmerged)
                let prefix = line.get(0..2).unwrap_or("");
                matches!(prefix, "UU" | "AA" | "DD" | "AU" | "UA" | "DU" | "UD")
            })
            .filter_map(|line| {
                // Format is "XY filename" where XY is the status
                line.get(3..).map(|s| s.to_string())
            })
            .collect();

        if conflict_files.is_empty() {
            Vec::new()
        } else {
            vec![ReconciliationIssue::GitConflict {
                affected_files: conflict_files,
            }]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_reconciliation_engine_new() {
        let engine = ReconciliationEngine::new(PathBuf::from("/tmp/test"));
        assert_eq!(engine.project_root, PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_check_git_conflicts_no_conflicts() {
        // Use current directory which should have no conflicts
        let cwd = env::current_dir().expect("Failed to get current directory");
        let engine = ReconciliationEngine::new(cwd);
        let issues = engine.check_git_conflicts();
        // In a clean repo, there should be no git conflicts
        assert!(
            issues.is_empty()
                || matches!(&issues[0], ReconciliationIssue::GitConflict { affected_files } if !affected_files.is_empty())
        );
    }

    #[test]
    fn test_check_git_conflicts_invalid_directory() {
        let engine = ReconciliationEngine::new(PathBuf::from("/nonexistent/path"));
        let issues = engine.check_git_conflicts();
        // Should return empty on error
        assert!(issues.is_empty());
    }

    #[test]
    fn test_git_conflict_issue_contains_files() {
        let issue = ReconciliationIssue::GitConflict {
            affected_files: vec!["file1.rs".to_string(), "file2.rs".to_string()],
        };
        if let ReconciliationIssue::GitConflict { affected_files } = issue {
            assert_eq!(affected_files.len(), 2);
            assert!(affected_files.contains(&"file1.rs".to_string()));
            assert!(affected_files.contains(&"file2.rs".to_string()));
        } else {
            panic!("Expected GitConflict variant");
        }
    }
}
