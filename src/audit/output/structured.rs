//! Structured output formats for audit reports.
//!
//! This module provides JSON serialization for audit reports,
//! enabling programmatic consumption by external tools.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use thiserror::Error;

use crate::audit::AuditReport;

/// Errors that can occur during JSON output operations.
#[derive(Error, Debug)]
pub enum JsonOutputError {
    /// IO error during file operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error.
    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for JSON output operations.
pub type JsonOutputResult<T> = Result<T, JsonOutputError>;

/// Writer for JSON-formatted audit reports.
pub struct JsonReportWriter;

impl JsonReportWriter {
    /// Write an audit report to a JSON file.
    ///
    /// # Arguments
    ///
    /// * `report` - The audit report to serialize.
    /// * `path` - The file path to write the JSON output.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `JsonOutputError` on failure.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    /// use ralphmacchio::audit::{AuditReport, output::JsonReportWriter};
    ///
    /// let report = AuditReport::new(PathBuf::from("/project"));
    /// JsonReportWriter::write_to_file(&report, "audit.json").unwrap();
    /// ```
    pub fn write_to_file<P: AsRef<Path>>(report: &AuditReport, path: P) -> JsonOutputResult<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        let json = serde_json::to_string_pretty(report)?;
        writer.write_all(json.as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    /// Serialize an audit report to a JSON string.
    ///
    /// # Arguments
    ///
    /// * `report` - The audit report to serialize.
    ///
    /// # Returns
    ///
    /// Returns the JSON string on success, or a `JsonOutputError` on failure.
    pub fn to_json_string(report: &AuditReport) -> JsonOutputResult<String> {
        let json = serde_json::to_string_pretty(report)?;
        Ok(json)
    }

    /// Serialize an audit report to a compact JSON string (no whitespace).
    ///
    /// # Arguments
    ///
    /// * `report` - The audit report to serialize.
    ///
    /// # Returns
    ///
    /// Returns the compact JSON string on success, or a `JsonOutputError` on failure.
    pub fn to_json_compact(report: &AuditReport) -> JsonOutputResult<String> {
        let json = serde_json::to_string(report)?;
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{
        AuditFinding, AuditMetadata, Complexity, DependencyAnalysis, FeatureOpportunity,
        FileInventory, Severity, SuggestedStory,
    };
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_report() -> AuditReport {
        AuditReport {
            metadata: AuditMetadata {
                audit_version: "0.1.0".to_string(),
                timestamp: "2024-01-15T10:30:00Z".to_string(),
                project_root: PathBuf::from("/test/project"),
                commit_hash: Some("abc123".to_string()),
                branch: Some("main".to_string()),
                duration_ms: 1500,
            },
            inventory: FileInventory::default(),
            dependencies: DependencyAnalysis::default(),
            findings: vec![
                AuditFinding {
                    id: "ARCH-001".to_string(),
                    severity: Severity::High,
                    category: "architecture".to_string(),
                    title: "Missing error handling".to_string(),
                    description: "Several functions lack proper error handling.".to_string(),
                    affected_files: vec![PathBuf::from("src/lib.rs"), PathBuf::from("src/main.rs")],
                    recommendation: "Add Result types and proper error propagation.".to_string(),
                },
                AuditFinding {
                    id: "DOC-001".to_string(),
                    severity: Severity::Medium,
                    category: "documentation".to_string(),
                    title: "Missing module documentation".to_string(),
                    description: "Public modules lack documentation comments.".to_string(),
                    affected_files: vec![PathBuf::from("src/utils/mod.rs")],
                    recommendation: "Add module-level documentation with //! comments.".to_string(),
                },
            ],
            opportunities: vec![FeatureOpportunity {
                id: "FEAT-001".to_string(),
                title: "Add caching layer".to_string(),
                rationale: "Repeated database queries could benefit from caching.".to_string(),
                complexity: Complexity::Medium,
                suggested_stories: vec![
                    SuggestedStory {
                        title: "Implement cache infrastructure".to_string(),
                        description: "Set up Redis or in-memory caching.".to_string(),
                        acceptance_criteria: vec![
                            "Cache client configured".to_string(),
                            "Connection pooling implemented".to_string(),
                        ],
                        priority: 1,
                    },
                    SuggestedStory {
                        title: "Add cache to hot paths".to_string(),
                        description: "Cache frequently accessed data.".to_string(),
                        acceptance_criteria: vec![
                            "User queries cached".to_string(),
                            "Cache invalidation works".to_string(),
                        ],
                        priority: 2,
                    },
                ],
            }],
        }
    }

    #[test]
    fn test_write_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("audit.json");

        let report = create_test_report();
        JsonReportWriter::write_to_file(&report, &output_path).unwrap();

        assert!(output_path.exists());

        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("\"audit_version\""));
        assert!(content.contains("\"metadata\""));
        assert!(content.contains("\"inventory\""));
        assert!(content.contains("\"dependencies\""));
        assert!(content.contains("\"findings\""));
        assert!(content.contains("\"opportunities\""));
    }

    #[test]
    fn test_to_json_string() {
        let report = create_test_report();
        let json = JsonReportWriter::to_json_string(&report).unwrap();

        // Verify it's valid JSON by parsing
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Check metadata section
        assert!(parsed.get("metadata").is_some());
        assert_eq!(parsed["metadata"]["audit_version"].as_str(), Some("0.1.0"));
        assert_eq!(parsed["metadata"]["commit_hash"].as_str(), Some("abc123"));
        assert_eq!(parsed["metadata"]["branch"].as_str(), Some("main"));
        assert_eq!(parsed["metadata"]["duration_ms"].as_u64(), Some(1500));

        // Check inventory section
        assert!(parsed.get("inventory").is_some());

        // Check dependencies section
        assert!(parsed.get("dependencies").is_some());

        // Check findings section
        assert!(parsed.get("findings").is_some());
        let findings = parsed["findings"].as_array().unwrap();
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0]["id"].as_str(), Some("ARCH-001"));
        assert_eq!(findings[0]["severity"].as_str(), Some("high"));
        assert_eq!(findings[1]["id"].as_str(), Some("DOC-001"));
        assert_eq!(findings[1]["severity"].as_str(), Some("medium"));

        // Check opportunities section
        assert!(parsed.get("opportunities").is_some());
        let opportunities = parsed["opportunities"].as_array().unwrap();
        assert_eq!(opportunities.len(), 1);
        assert_eq!(opportunities[0]["id"].as_str(), Some("FEAT-001"));
        assert_eq!(opportunities[0]["complexity"].as_str(), Some("medium"));

        // Check suggested stories
        let stories = opportunities[0]["suggested_stories"].as_array().unwrap();
        assert_eq!(stories.len(), 2);
        assert_eq!(stories[0]["priority"].as_u64(), Some(1));
    }

    #[test]
    fn test_to_json_compact() {
        let report = create_test_report();
        let compact = JsonReportWriter::to_json_compact(&report).unwrap();
        let pretty = JsonReportWriter::to_json_string(&report).unwrap();

        // Compact should be shorter (no whitespace formatting)
        assert!(compact.len() < pretty.len());

        // Both should be valid JSON with same content
        let compact_parsed: serde_json::Value = serde_json::from_str(&compact).unwrap();
        let pretty_parsed: serde_json::Value = serde_json::from_str(&pretty).unwrap();
        assert_eq!(compact_parsed, pretty_parsed);
    }

    #[test]
    fn test_json_round_trip() {
        let report = create_test_report();
        let json = JsonReportWriter::to_json_string(&report).unwrap();

        // Parse back to AuditReport
        let deserialized: AuditReport = serde_json::from_str(&json).unwrap();

        // Verify key fields
        assert_eq!(deserialized.metadata.audit_version, "0.1.0");
        assert_eq!(
            deserialized.metadata.commit_hash,
            Some("abc123".to_string())
        );
        assert_eq!(deserialized.findings.len(), 2);
        assert_eq!(deserialized.opportunities.len(), 1);
    }

    #[test]
    fn test_empty_report() {
        let report = AuditReport::new(PathBuf::from("/empty"));
        let json = JsonReportWriter::to_json_string(&report).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["findings"].as_array().unwrap().is_empty());
        assert!(parsed["opportunities"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_write_to_file_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("subdir").join("audit.json");

        // Create parent directory first (since write_to_file doesn't create dirs)
        std::fs::create_dir_all(output_path.parent().unwrap()).unwrap();

        let report = create_test_report();
        JsonReportWriter::write_to_file(&report, &output_path).unwrap();

        assert!(output_path.exists());
    }

    #[test]
    fn test_severity_serialization() {
        let report = create_test_report();
        let json = JsonReportWriter::to_json_string(&report).unwrap();

        // Verify severity is serialized as lowercase string
        assert!(json.contains("\"severity\": \"high\""));
        assert!(json.contains("\"severity\": \"medium\""));
    }

    #[test]
    fn test_complexity_serialization() {
        let report = create_test_report();
        let json = JsonReportWriter::to_json_string(&report).unwrap();

        // Verify complexity is serialized as lowercase string
        assert!(json.contains("\"complexity\": \"medium\""));
    }

    #[test]
    fn test_affected_files_serialization() {
        let report = create_test_report();
        let json = JsonReportWriter::to_json_string(&report).unwrap();

        // Verify affected files are included
        assert!(json.contains("src/lib.rs"));
        assert!(json.contains("src/main.rs"));
    }

    #[test]
    fn test_acceptance_criteria_serialization() {
        let report = create_test_report();
        let json = JsonReportWriter::to_json_string(&report).unwrap();

        // Verify acceptance criteria are included
        assert!(json.contains("Cache client configured"));
        assert!(json.contains("Connection pooling implemented"));
    }
}
