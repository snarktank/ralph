//! Integration tests to validate test fixtures are parseable
//!
//! These tests ensure that the test fixtures in tests/fixtures/ directory
//! are valid and can be parsed correctly by the application.

use std::fs;
use std::path::PathBuf;

/// Get the path to the fixtures directory
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

// ============================================================================
// test_prd.json fixture tests
// ============================================================================

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct TestPrd {
    project: String,
    #[serde(rename = "branchName")]
    branch_name: String,
    description: String,
    #[serde(rename = "userStories")]
    user_stories: Vec<TestUserStory>,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct TestUserStory {
    id: String,
    title: String,
    description: String,
    #[serde(rename = "acceptanceCriteria")]
    acceptance_criteria: Vec<String>,
    priority: u32,
    passes: bool,
    notes: String,
}

#[test]
fn test_prd_fixture_exists() {
    let prd_path = fixtures_dir().join("test_prd.json");
    assert!(prd_path.exists(), "test_prd.json fixture should exist");
}

#[test]
fn test_prd_fixture_is_valid_json() {
    let prd_path = fixtures_dir().join("test_prd.json");
    let content = fs::read_to_string(&prd_path).expect("Failed to read test_prd.json");
    let _: serde_json::Value =
        serde_json::from_str(&content).expect("test_prd.json should be valid JSON");
}

#[test]
fn test_prd_fixture_has_correct_structure() {
    let prd_path = fixtures_dir().join("test_prd.json");
    let content = fs::read_to_string(&prd_path).expect("Failed to read test_prd.json");
    let prd: TestPrd =
        serde_json::from_str(&content).expect("test_prd.json should match PRD structure");

    // Verify project fields
    assert!(!prd.project.is_empty(), "Project name should not be empty");
    assert!(
        !prd.branch_name.is_empty(),
        "Branch name should not be empty"
    );

    // Verify we have 2-3 stories
    assert!(
        prd.user_stories.len() >= 2 && prd.user_stories.len() <= 3,
        "PRD should have 2-3 stories, got {}",
        prd.user_stories.len()
    );

    // Verify each story has required fields
    for story in &prd.user_stories {
        assert!(!story.id.is_empty(), "Story ID should not be empty");
        assert!(!story.title.is_empty(), "Story title should not be empty");
        assert!(
            !story.acceptance_criteria.is_empty(),
            "Story should have at least one acceptance criterion"
        );
    }
}

#[test]
fn test_prd_fixture_has_passing_and_failing_stories() {
    let prd_path = fixtures_dir().join("test_prd.json");
    let content = fs::read_to_string(&prd_path).expect("Failed to read test_prd.json");
    let prd: TestPrd =
        serde_json::from_str(&content).expect("test_prd.json should match PRD structure");

    let passing = prd.user_stories.iter().filter(|s| s.passes).count();
    let failing = prd.user_stories.iter().filter(|s| !s.passes).count();

    assert!(passing > 0, "PRD should have at least one passing story");
    assert!(failing > 0, "PRD should have at least one failing story");
}

// ============================================================================
// test_quality_config.toml fixture tests
// ============================================================================

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct TestQualityConfig {
    profiles: std::collections::HashMap<String, TestProfile>,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct TestProfile {
    description: String,
    documentation: TestDocumentationConfig,
    testing: TestTestingConfig,
    ci: TestCiConfig,
    security: TestSecurityConfig,
    blog: TestBlogConfig,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct TestDocumentationConfig {
    required: bool,
    readme: bool,
    inline_comments: bool,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct TestTestingConfig {
    unit_tests: bool,
    integration_tests: bool,
    coverage_threshold: u32,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct TestCiConfig {
    required: bool,
    format_check: bool,
    lint_check: bool,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct TestSecurityConfig {
    cargo_audit: bool,
    cargo_deny: bool,
    sast: bool,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct TestBlogConfig {
    generate: bool,
    #[serde(default)]
    template: Option<String>,
}

#[test]
fn test_quality_config_fixture_exists() {
    let config_path = fixtures_dir().join("test_quality_config.toml");
    assert!(
        config_path.exists(),
        "test_quality_config.toml fixture should exist"
    );
}

#[test]
fn test_quality_config_fixture_is_valid_toml() {
    let config_path = fixtures_dir().join("test_quality_config.toml");
    let content =
        fs::read_to_string(&config_path).expect("Failed to read test_quality_config.toml");
    let _: toml::Value =
        toml::from_str(&content).expect("test_quality_config.toml should be valid TOML");
}

#[test]
fn test_quality_config_fixture_has_correct_structure() {
    let config_path = fixtures_dir().join("test_quality_config.toml");
    let content =
        fs::read_to_string(&config_path).expect("Failed to read test_quality_config.toml");
    let config: TestQualityConfig = toml::from_str(&content)
        .expect("test_quality_config.toml should match QualityConfig structure");

    // Verify we have profiles
    assert!(
        !config.profiles.is_empty(),
        "Config should have at least one profile"
    );

    // Verify each profile has all required sections
    for (name, profile) in &config.profiles {
        assert!(
            !profile.description.is_empty(),
            "Profile '{}' should have a description",
            name
        );
    }
}

#[test]
fn test_quality_config_fixture_has_multiple_profiles() {
    let config_path = fixtures_dir().join("test_quality_config.toml");
    let content =
        fs::read_to_string(&config_path).expect("Failed to read test_quality_config.toml");
    let config: TestQualityConfig = toml::from_str(&content)
        .expect("test_quality_config.toml should match QualityConfig structure");

    // Verify we have at least 2 profiles for meaningful testing
    assert!(
        config.profiles.len() >= 2,
        "Config should have at least 2 profiles for testing, got {}",
        config.profiles.len()
    );
}

#[test]
fn test_quality_config_fixture_has_varying_coverage_thresholds() {
    let config_path = fixtures_dir().join("test_quality_config.toml");
    let content =
        fs::read_to_string(&config_path).expect("Failed to read test_quality_config.toml");
    let config: TestQualityConfig = toml::from_str(&content)
        .expect("test_quality_config.toml should match QualityConfig structure");

    let thresholds: Vec<u32> = config
        .profiles
        .values()
        .map(|p| p.testing.coverage_threshold)
        .collect();

    let min = thresholds.iter().min().unwrap();
    let max = thresholds.iter().max().unwrap();

    assert!(
        min != max,
        "Config should have varying coverage thresholds for testing different scenarios"
    );
}
