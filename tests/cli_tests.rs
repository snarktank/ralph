//! CLI integration tests for the Ralph binary
//!
//! These tests verify that the CLI commands work correctly by running
//! the actual compiled binary.

use assert_cmd::Command;
use predicates::prelude::*;

/// Get a Command instance for the ralph binary
#[allow(deprecated)]
fn ralph_cmd() -> Command {
    Command::cargo_bin("ralph").expect("Failed to find ralph binary")
}

// ============================================================================
// --version flag tests
// ============================================================================

#[test]
fn test_version_flag_short() {
    ralph_cmd()
        .arg("-V")
        .assert()
        .success()
        // The styled version uses uppercase RALPH in the banner
        .stdout(predicate::str::contains("RALPH"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_version_flag_long() {
    ralph_cmd()
        .arg("--version")
        .assert()
        .success()
        // The styled version uses uppercase RALPH in the banner
        .stdout(predicate::str::contains("RALPH"))
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

// ============================================================================
// --help flag tests
// ============================================================================

#[test]
fn test_help_flag_short() {
    ralph_cmd()
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Enterprise-ready autonomous AI agent framework",
        ))
        .stdout(predicate::str::contains("USAGE:"))
        .stdout(predicate::str::contains("COMMANDS:"));
}

#[test]
fn test_help_flag_long() {
    ralph_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Enterprise-ready autonomous AI agent framework",
        ))
        .stdout(predicate::str::contains("USAGE:"))
        .stdout(predicate::str::contains("COMMANDS:"));
}

#[test]
fn test_help_shows_quality_command() {
    ralph_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("quality"))
        .stdout(predicate::str::contains("Run quality checks"));
}

#[test]
fn test_help_shows_mcp_server_command() {
    ralph_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("mcp-server"))
        .stdout(predicate::str::contains("Start MCP server mode"));
}

// ============================================================================
// quality command tests
// ============================================================================

#[test]
fn test_quality_command_runs() {
    ralph_cmd()
        .arg("quality")
        .assert()
        .success()
        .stdout(predicate::str::contains("Running quality checks"));
}

#[test]
fn test_quality_help() {
    ralph_cmd()
        .args(["quality", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Run quality checks"));
}

// ============================================================================
// mcp-server command tests
// ============================================================================

#[test]
fn test_mcp_server_help() {
    ralph_cmd()
        .args(["mcp-server", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Start MCP server mode"))
        .stdout(predicate::str::contains("--prd"));
}

// ============================================================================
// Default behavior tests
// ============================================================================

#[test]
fn test_no_args_shows_help_without_prd() {
    // Run in a temp directory without prd.json - should show help
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    ralph_cmd()
        .current_dir(temp_dir.path())
        .assert()
        .success()
        // Output contains "RALPH" in the ASCII banner or "ralph" in usage
        .stdout(predicate::str::contains("RALPH").or(predicate::str::contains("ralph")))
        .stdout(predicate::str::contains("--help"));
}

#[test]
fn test_no_args_with_prd_starts_running() {
    // Run in a temp directory with prd.json - should start executing
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let prd_content = r#"{
        "project": "Test",
        "branchName": "test/branch",
        "description": "Test PRD",
        "userStories": [
            {
                "id": "US-001",
                "title": "Test story",
                "description": "Test",
                "acceptanceCriteria": ["AC1"],
                "priority": 1,
                "passes": true
            }
        ]
    }"#;
    std::fs::write(temp_dir.path().join("prd.json"), prd_content)
        .expect("Failed to write prd.json");

    // With all stories passing, should output COMPLETE
    ralph_cmd()
        .current_dir(temp_dir.path())
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("COMPLETE"));
}

// ============================================================================
// Invalid command tests
// ============================================================================

#[test]
fn test_invalid_command_fails() {
    ralph_cmd()
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_invalid_flag_fails() {
    ralph_cmd()
        .arg("--nonexistent-flag")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}
