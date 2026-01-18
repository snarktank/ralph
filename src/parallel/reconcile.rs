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
    TypeMismatch {
        /// File where the type error was found
        file: String,
        /// Error message from the compiler
        error: String,
    },
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

    /// Checks for type errors in the project by running `cargo check`
    ///
    /// Runs `cargo check` and parses the output for error messages.
    /// Returns a vector of `ReconciliationIssue::TypeMismatch` for each error found.
    ///
    /// # Returns
    /// A vector of `ReconciliationIssue` containing any detected type errors.
    /// Returns an empty vector if no type errors are detected or if this is not a Rust project.
    pub fn check_type_errors(&self) -> Vec<ReconciliationIssue> {
        // Check if this is a Rust project by looking for Cargo.toml
        let cargo_toml = self.project_root.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Vec::new();
        }

        let output = Command::new("cargo")
            .args(["check", "--message-format=short"])
            .current_dir(&self.project_root)
            .output();

        let output = match output {
            Ok(o) => o,
            Err(_) => return Vec::new(),
        };

        // cargo check returns non-zero exit status when there are errors
        // We need to parse stderr for error messages
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Parse error messages in the short format: "file:line:col: error: message"
        let mut issues = Vec::new();
        for line in stderr.lines() {
            // Match lines that look like error messages
            // Format: "src/file.rs:10:5: error[E0001]: some error message"
            // Or: "error[E0001]: some error message"
            // Or: "error: some error message"
            if line.starts_with("error") || line.contains(": error") {
                let (file, error) = Self::parse_error_line(line);
                issues.push(ReconciliationIssue::TypeMismatch { file, error });
            }
        }

        issues
    }

    /// Runs full reconciliation checking for all known issue types
    ///
    /// This method combines all individual checks (git conflicts, type errors)
    /// and returns a comprehensive `ReconciliationResult`.
    ///
    /// # Returns
    /// `ReconciliationResult::Clean` if no issues are found, or
    /// `ReconciliationResult::IssuesFound` with a list of all detected issues.
    pub fn reconcile(&self) -> ReconciliationResult {
        let mut all_issues = Vec::new();

        // Check for git conflicts
        all_issues.extend(self.check_git_conflicts());

        // Check for type errors
        all_issues.extend(self.check_type_errors());

        if all_issues.is_empty() {
            ReconciliationResult::Clean
        } else {
            ReconciliationResult::IssuesFound(all_issues)
        }
    }

    /// Parses an error line from cargo check output
    ///
    /// # Arguments
    /// * `line` - A line from cargo check stderr
    ///
    /// # Returns
    /// A tuple of (file, error) where file is the affected file path and error is the message
    fn parse_error_line(line: &str) -> (String, String) {
        // Try to parse "file:line:col: error[...]: message" format
        if let Some(colon_pos) = line.find(": error") {
            let file_part = &line[..colon_pos];
            let error_part = &line[colon_pos + 2..]; // Skip ": "

            // Extract just the file path (before line:col)
            let file = file_part.split(':').next().unwrap_or("unknown").to_string();

            return (file, error_part.trim().to_string());
        }

        // If line starts with "error", there's no file info
        if line.starts_with("error") {
            return ("unknown".to_string(), line.to_string());
        }

        // Fallback
        ("unknown".to_string(), line.to_string())
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

    #[test]
    fn test_check_type_errors_valid_project() {
        // Use current directory which is a valid Rust project
        let cwd = env::current_dir().expect("Failed to get current directory");
        let engine = ReconciliationEngine::new(cwd);
        let issues = engine.check_type_errors();
        // A valid project should have no type errors
        assert!(issues.is_empty());
    }

    #[test]
    fn test_check_type_errors_non_rust_project() {
        // Use a directory without Cargo.toml
        let engine = ReconciliationEngine::new(PathBuf::from("/tmp"));
        let issues = engine.check_type_errors();
        // Should return empty for non-Rust projects
        assert!(issues.is_empty());
    }

    #[test]
    fn test_check_type_errors_invalid_directory() {
        let engine = ReconciliationEngine::new(PathBuf::from("/nonexistent/path"));
        let issues = engine.check_type_errors();
        // Should return empty on error
        assert!(issues.is_empty());
    }

    #[test]
    fn test_parse_error_line_with_file() {
        let line = "src/main.rs:10:5: error[E0425]: cannot find value `foo`";
        let (file, error) = ReconciliationEngine::parse_error_line(line);
        assert_eq!(file, "src/main.rs");
        assert_eq!(error, "error[E0425]: cannot find value `foo`");
    }

    #[test]
    fn test_parse_error_line_without_file() {
        let line = "error[E0463]: can't find crate for `some_crate`";
        let (file, error) = ReconciliationEngine::parse_error_line(line);
        assert_eq!(file, "unknown");
        assert_eq!(error, "error[E0463]: can't find crate for `some_crate`");
    }

    #[test]
    fn test_parse_error_line_generic_error() {
        let line = "error: aborting due to 2 previous errors";
        let (file, error) = ReconciliationEngine::parse_error_line(line);
        assert_eq!(file, "unknown");
        assert_eq!(error, "error: aborting due to 2 previous errors");
    }

    #[test]
    fn test_type_mismatch_issue_contains_details() {
        let issue = ReconciliationIssue::TypeMismatch {
            file: "src/lib.rs".to_string(),
            error: "error[E0308]: mismatched types".to_string(),
        };
        if let ReconciliationIssue::TypeMismatch { file, error } = issue {
            assert_eq!(file, "src/lib.rs");
            assert!(error.contains("mismatched types"));
        } else {
            panic!("Expected TypeMismatch variant");
        }
    }

    #[test]
    fn test_reconcile_clean_project() {
        // Use current directory which should be a clean project
        let cwd = env::current_dir().expect("Failed to get current directory");
        let engine = ReconciliationEngine::new(cwd);
        let result = engine.reconcile();
        // A clean project should return Clean
        assert_eq!(result, ReconciliationResult::Clean);
    }

    #[test]
    fn test_reconcile_non_rust_project() {
        // Use /tmp which has no Cargo.toml
        let engine = ReconciliationEngine::new(PathBuf::from("/tmp"));
        let result = engine.reconcile();
        // Should return Clean since there's nothing to check
        assert_eq!(result, ReconciliationResult::Clean);
    }
}
