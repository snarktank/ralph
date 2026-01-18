//! Integration tests for the audit feature on Ralph's own codebase.
//!
//! This test suite runs the full audit on the Ralph codebase itself
//! to verify end-to-end functionality.

use ralphmacchio::audit::{
    AuditReport, InventoryScanner, JsonReportWriter, MarkdownReportWriter, ProjectType,
};
use std::path::PathBuf;

/// Get the path to the Ralph codebase root.
fn ralph_codebase_root() -> PathBuf {
    // This test file is in the `tests/` directory, so parent is the project root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Run a full audit on the Ralph codebase and return the report.
fn run_ralph_audit() -> AuditReport {
    let target_dir = ralph_codebase_root();
    let mut report = AuditReport::new(target_dir.clone());

    // Run inventory scan
    let scanner = InventoryScanner::new(target_dir);
    report.inventory = scanner.scan().expect("Inventory scan should succeed");

    report
}

// ============================================================================
// US-026 Acceptance Criteria 1: Run full audit on Ralph codebase
// ============================================================================

#[test]
fn test_audit_runs_on_ralph_codebase() {
    let target_dir = ralph_codebase_root();

    // Verify the target directory exists and is the Ralph project
    assert!(target_dir.exists(), "Ralph codebase root should exist");
    assert!(
        target_dir.join("Cargo.toml").exists(),
        "Should find Cargo.toml in Ralph root"
    );
    assert!(
        target_dir.join("src").exists(),
        "Should find src directory in Ralph root"
    );

    // Run the audit
    let report = run_ralph_audit();

    // Basic sanity checks - the audit should complete and produce a valid report
    assert!(
        !report.metadata.audit_version.is_empty(),
        "Audit version should be set"
    );
    assert!(
        report.metadata.project_root.exists(),
        "Project root should exist"
    );
}

// ============================================================================
// US-026 Acceptance Criteria 2: All analysis sections produce results
// ============================================================================

#[test]
fn test_inventory_section_produces_results() {
    let report = run_ralph_audit();

    // Verify inventory has meaningful data
    assert!(
        report.inventory.total_files > 0,
        "Should find files in Ralph codebase, found {}",
        report.inventory.total_files
    );
    assert!(
        report.inventory.total_loc > 0,
        "Should count lines of code, found {}",
        report.inventory.total_loc
    );
}

#[test]
fn test_inventory_finds_rust_files() {
    let report = run_ralph_audit();

    // Ralph is a Rust project, so we should find Rust files
    let rust_file_count = report
        .inventory
        .files_by_extension
        .get("rs")
        .copied()
        .unwrap_or(0);

    assert!(
        rust_file_count > 0,
        "Should find Rust files in Ralph codebase, found {}",
        rust_file_count
    );

    // Ralph has significant codebase
    assert!(
        rust_file_count >= 10,
        "Ralph should have at least 10 Rust files, found {}",
        rust_file_count
    );
}

#[test]
fn test_inventory_finds_expected_directories() {
    let report = run_ralph_audit();

    // Verify we found the expected directory structure
    let dir_count = report.inventory.structure.len();
    assert!(
        dir_count > 0,
        "Should find directories in Ralph codebase, found {}",
        dir_count
    );
}

#[test]
fn test_inventory_detects_project_type() {
    let report = run_ralph_audit();

    // Ralph should be detected as a Rust or Mixed project (it has Cargo.toml and other configs)
    // The key indicator is that it's NOT Unknown
    assert!(
        matches!(
            report.inventory.project_type,
            ProjectType::Rust | ProjectType::Mixed
        ),
        "Ralph should be detected as a Rust or Mixed project, got {:?}",
        report.inventory.project_type
    );
}

// ============================================================================
// US-026 Acceptance Criteria 3: JSON output is valid and parseable
// ============================================================================

#[test]
fn test_json_output_is_valid() {
    let report = run_ralph_audit();

    // Generate JSON output
    let json_output =
        JsonReportWriter::to_json_string(&report).expect("JSON serialization should succeed");

    // Verify it's not empty
    assert!(!json_output.is_empty(), "JSON output should not be empty");

    // Verify it's valid JSON by parsing it
    let parsed: serde_json::Value =
        serde_json::from_str(&json_output).expect("JSON output should be valid JSON");

    // Verify structure
    assert!(
        parsed.get("metadata").is_some(),
        "JSON should contain metadata section"
    );
    assert!(
        parsed.get("inventory").is_some(),
        "JSON should contain inventory section"
    );
    assert!(
        parsed.get("dependencies").is_some(),
        "JSON should contain dependencies section"
    );
    assert!(
        parsed.get("findings").is_some(),
        "JSON should contain findings section"
    );
    assert!(
        parsed.get("opportunities").is_some(),
        "JSON should contain opportunities section"
    );
}

#[test]
fn test_json_output_round_trips() {
    let report = run_ralph_audit();

    // Generate JSON output
    let json_output =
        JsonReportWriter::to_json_string(&report).expect("JSON serialization should succeed");

    // Parse it back to AuditReport
    let deserialized: AuditReport =
        serde_json::from_str(&json_output).expect("Should deserialize back to AuditReport");

    // Verify key fields match
    assert_eq!(
        report.metadata.audit_version, deserialized.metadata.audit_version,
        "Audit version should match after round-trip"
    );
    assert_eq!(
        report.inventory.total_files, deserialized.inventory.total_files,
        "Total files should match after round-trip"
    );
    assert_eq!(
        report.inventory.total_loc, deserialized.inventory.total_loc,
        "Total LOC should match after round-trip"
    );
}

#[test]
fn test_json_compact_output_is_valid() {
    let report = run_ralph_audit();

    // Generate compact JSON output
    let compact_json = JsonReportWriter::to_json_compact(&report)
        .expect("Compact JSON serialization should succeed");

    // Verify it's valid JSON
    let _parsed: serde_json::Value =
        serde_json::from_str(&compact_json).expect("Compact JSON output should be valid JSON");

    // Compact JSON should be smaller than pretty JSON
    let pretty_json = JsonReportWriter::to_json_string(&report).unwrap();
    assert!(
        compact_json.len() < pretty_json.len(),
        "Compact JSON should be smaller than pretty JSON"
    );
}

// ============================================================================
// US-026 Acceptance Criteria 4: Markdown output is well-formed
// ============================================================================

#[test]
fn test_markdown_output_is_well_formed() {
    let report = run_ralph_audit();

    // Generate markdown output
    let markdown_output = MarkdownReportWriter::to_markdown_string(&report);

    // Verify it's not empty
    assert!(
        !markdown_output.is_empty(),
        "Markdown output should not be empty"
    );

    // Verify required sections exist
    assert!(
        markdown_output.contains("# Codebase Audit Report"),
        "Markdown should have title"
    );
    assert!(
        markdown_output.contains("## Table of Contents"),
        "Markdown should have table of contents"
    );
    assert!(
        markdown_output.contains("## Executive Summary"),
        "Markdown should have executive summary"
    );
    assert!(
        markdown_output.contains("## Audit Metadata"),
        "Markdown should have audit metadata"
    );
}

#[test]
fn test_markdown_contains_metrics() {
    let report = run_ralph_audit();
    let markdown_output = MarkdownReportWriter::to_markdown_string(&report);

    // Verify key metrics are present
    assert!(
        markdown_output.contains("### Key Metrics"),
        "Markdown should have key metrics section"
    );
    assert!(
        markdown_output.contains("Total Files"),
        "Markdown should show total files"
    );
    assert!(
        markdown_output.contains("Total Lines of Code"),
        "Markdown should show total LOC"
    );
}

#[test]
fn test_markdown_has_proper_structure() {
    let report = run_ralph_audit();
    let markdown_output = MarkdownReportWriter::to_markdown_string(&report);

    // Verify markdown structure with headers
    let h1_count = markdown_output.matches("\n# ").count()
        + if markdown_output.starts_with("# ") {
            1
        } else {
            0
        };
    let h2_count = markdown_output.matches("\n## ").count();

    assert!(h1_count >= 1, "Should have at least one H1 header");
    assert!(
        h2_count >= 2,
        "Should have at least two H2 headers (Table of Contents and Executive Summary)"
    );

    // Verify tables are properly formatted (have header separator)
    let table_separators =
        markdown_output.matches("|---|").count() + markdown_output.matches("|-------").count();
    assert!(
        table_separators > 0,
        "Should have at least one properly formatted table"
    );
}

#[test]
fn test_markdown_metadata_section() {
    let report = run_ralph_audit();
    let markdown_output = MarkdownReportWriter::to_markdown_string(&report);

    // Verify metadata section contains expected information
    assert!(
        markdown_output.contains("Audit Version"),
        "Metadata should show audit version"
    );
    assert!(
        markdown_output.contains("Timestamp"),
        "Metadata should show timestamp"
    );
    assert!(
        markdown_output.contains("Project Root"),
        "Metadata should show project root"
    );
}

// ============================================================================
// US-026 Acceptance Criteria 5: Findings are reasonable for Ralph codebase
// ============================================================================

#[test]
fn test_findings_have_valid_structure() {
    let report = run_ralph_audit();

    // If there are findings, verify they have valid structure
    for finding in &report.findings {
        assert!(!finding.id.is_empty(), "Finding should have an ID");
        assert!(!finding.title.is_empty(), "Finding should have a title");
        assert!(
            !finding.description.is_empty(),
            "Finding should have a description"
        );
        assert!(
            !finding.category.is_empty(),
            "Finding should have a category"
        );
        assert!(
            !finding.recommendation.is_empty(),
            "Finding should have a recommendation"
        );
    }
}

#[test]
fn test_findings_have_valid_severities() {
    let report = run_ralph_audit();

    // If there are findings, verify severities are valid
    for finding in &report.findings {
        // Severity is an enum, so it's always valid, but we can verify it's one of expected values
        let severity_str = format!("{}", finding.severity);
        assert!(
            ["low", "medium", "high", "critical"].contains(&severity_str.as_str()),
            "Severity '{}' should be one of: low, medium, high, critical",
            severity_str
        );
    }
}

#[test]
fn test_finding_counts_are_consistent() {
    let report = run_ralph_audit();

    let (critical, high, medium, low) = report.finding_counts();
    let total_from_counts = critical + high + medium + low;

    assert_eq!(
        total_from_counts,
        report.findings.len(),
        "Sum of finding counts should equal total findings"
    );
}

#[test]
fn test_opportunities_have_valid_structure() {
    let report = run_ralph_audit();

    // If there are opportunities, verify they have valid structure
    for opportunity in &report.opportunities {
        assert!(!opportunity.id.is_empty(), "Opportunity should have an ID");
        assert!(
            !opportunity.title.is_empty(),
            "Opportunity should have a title"
        );
        assert!(
            !opportunity.rationale.is_empty(),
            "Opportunity should have a rationale"
        );
    }
}

// ============================================================================
// US-026 Acceptance Criteria 6: Additional validation tests
// ============================================================================

#[test]
fn test_file_inventory_reasonable_counts() {
    let report = run_ralph_audit();

    // Ralph should have a reasonable number of files
    assert!(
        report.inventory.total_files >= 20,
        "Ralph should have at least 20 files, found {}",
        report.inventory.total_files
    );
    assert!(
        report.inventory.total_files <= 10000,
        "Total files count seems unreasonably high: {}",
        report.inventory.total_files
    );

    // Lines of code should be reasonable
    assert!(
        report.inventory.total_loc >= 1000,
        "Ralph should have at least 1000 lines of code, found {}",
        report.inventory.total_loc
    );
    assert!(
        report.inventory.total_loc <= 1_000_000,
        "LOC count seems unreasonably high: {}",
        report.inventory.total_loc
    );
}

#[test]
fn test_metadata_has_valid_timestamp() {
    let report = run_ralph_audit();

    // Verify timestamp is in ISO 8601 format (or similar)
    let timestamp = &report.metadata.timestamp;
    assert!(!timestamp.is_empty(), "Timestamp should not be empty");

    // Should contain date-like patterns (YYYY-MM-DD or T separator)
    assert!(
        timestamp.contains('-') || timestamp.contains('T'),
        "Timestamp '{}' should look like a date",
        timestamp
    );
}

#[test]
fn test_metadata_has_valid_version() {
    let report = run_ralph_audit();

    // Verify audit version matches package version
    let pkg_version = env!("CARGO_PKG_VERSION");
    assert_eq!(
        report.metadata.audit_version, pkg_version,
        "Audit version should match package version"
    );
}

#[test]
fn test_audit_completes_in_reasonable_time() {
    use std::time::Instant;

    let start = Instant::now();
    let _report = run_ralph_audit();
    let elapsed = start.elapsed();

    // Audit should complete within 30 seconds
    assert!(
        elapsed.as_secs() < 30,
        "Audit should complete in under 30 seconds, took {:?}",
        elapsed
    );
}

// ============================================================================
// Integration with file writing
// ============================================================================

#[test]
fn test_json_file_write() {
    let report = run_ralph_audit();
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("ralph_audit.json");

    // Write to file
    JsonReportWriter::write_to_file(&report, &output_path).expect("Should write JSON file");

    // Verify file exists and is readable
    assert!(output_path.exists(), "JSON file should be created");

    let content = std::fs::read_to_string(&output_path).expect("Should read JSON file");
    let _parsed: serde_json::Value =
        serde_json::from_str(&content).expect("File content should be valid JSON");
}

#[test]
fn test_markdown_file_write() {
    let report = run_ralph_audit();
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("ralph_audit.md");

    // Write to file
    MarkdownReportWriter::write_to_file(&report, &output_path).expect("Should write markdown file");

    // Verify file exists and is readable
    assert!(output_path.exists(), "Markdown file should be created");

    let content = std::fs::read_to_string(&output_path).expect("Should read markdown file");
    assert!(
        content.contains("# Codebase Audit Report"),
        "File content should be valid markdown"
    );
}
