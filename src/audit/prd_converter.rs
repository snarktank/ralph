//! PRD-to-prd.json converter for Ralph.
//!
//! This module converts generated PRD markdown files into the prd.json format
//! that Ralph uses to execute user stories. This allows users to immediately
//! run Ralph after an audit without manual conversion.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

/// A user story in prd.json format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrdUserStory {
    /// Story ID (e.g., "US-001")
    pub id: String,
    /// Story title
    pub title: String,
    /// Story description
    pub description: String,
    /// List of acceptance criteria
    pub acceptance_criteria: Vec<String>,
    /// Priority (lower = higher priority)
    pub priority: u32,
    /// Whether the story passes (always false for newly generated)
    pub passes: bool,
    /// Optional notes
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
}

/// The prd.json file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrdJson {
    /// Project name
    pub project: String,
    /// Branch name for the work
    pub branch_name: String,
    /// Description of the PRD
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// List of user stories
    pub user_stories: Vec<PrdUserStory>,
}

impl PrdJson {
    /// Create a new empty PrdJson
    pub fn new(project: String, branch_name: String) -> Self {
        Self {
            project,
            branch_name,
            description: String::new(),
            user_stories: Vec::new(),
        }
    }

    /// Add a description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add a user story
    pub fn add_story(&mut self, story: PrdUserStory) {
        self.user_stories.push(story);
    }
}

/// Configuration for PRD conversion
#[derive(Debug, Clone, Default)]
pub struct PrdConverterConfig {
    /// Skip the user confirmation prompt
    pub skip_prompt: bool,
    /// Project name (extracted from PRD if not provided)
    pub project_name: Option<String>,
    /// Branch name for the generated prd.json
    pub branch_name: Option<String>,
    /// Output directory for prd.json (defaults to current directory)
    pub output_dir: PathBuf,
}

impl PrdConverterConfig {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        Self {
            skip_prompt: false,
            project_name: None,
            branch_name: None,
            output_dir: PathBuf::from("."),
        }
    }

    /// Set skip_prompt flag
    pub fn with_skip_prompt(mut self, skip: bool) -> Self {
        self.skip_prompt = skip;
        self
    }

    /// Set the project name
    pub fn with_project_name(mut self, name: impl Into<String>) -> Self {
        self.project_name = Some(name.into());
        self
    }

    /// Set the branch name
    pub fn with_branch_name(mut self, name: impl Into<String>) -> Self {
        self.branch_name = Some(name.into());
        self
    }

    /// Set the output directory
    pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }
}

/// Result of PRD conversion
#[derive(Debug, Clone)]
pub struct PrdConversionResult {
    /// Path to the generated prd.json file
    pub prd_json_path: PathBuf,
    /// Number of user stories extracted
    pub story_count: usize,
    /// Project name used
    pub project_name: String,
    /// Branch name used
    pub branch_name: String,
}

/// Converter for PRD markdown to prd.json
pub struct PrdConverter {
    config: PrdConverterConfig,
}

impl Default for PrdConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl PrdConverter {
    /// Create a new PRD converter with default configuration
    pub fn new() -> Self {
        Self {
            config: PrdConverterConfig::new(),
        }
    }

    /// Create a new PRD converter with custom configuration
    pub fn with_config(config: PrdConverterConfig) -> Self {
        Self { config }
    }

    /// Get the configuration
    pub fn config(&self) -> &PrdConverterConfig {
        &self.config
    }

    /// Prompt the user for confirmation before converting
    ///
    /// Returns true if the user confirms, false otherwise.
    /// If `skip_prompt` is set in config, always returns true.
    pub fn prompt_user_confirmation(&self) -> io::Result<bool> {
        if self.config.skip_prompt {
            return Ok(true);
        }

        self.prompt_with_reader_writer(&mut io::stdin().lock(), &mut io::stdout())
    }

    /// Prompt with custom reader/writer (for testing)
    pub fn prompt_with_reader_writer<R: BufRead, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> io::Result<bool> {
        writeln!(writer)?;
        writeln!(
            writer,
            "╭─────────────────────────────────────────────────────────────╮"
        )?;
        writeln!(
            writer,
            "│  📋 PRD to prd.json Conversion                              │"
        )?;
        writeln!(
            writer,
            "│  Convert the generated PRD to prd.json for Ralph?          │"
        )?;
        writeln!(
            writer,
            "╰─────────────────────────────────────────────────────────────╯"
        )?;
        writeln!(writer)?;
        writeln!(
            writer,
            "This will create a prd.json file that Ralph can use to execute"
        )?;
        writeln!(writer, "the user stories from the PRD.")?;
        writeln!(writer)?;
        write!(writer, "Convert to prd.json? [Y/n]: ")?;
        writer.flush()?;

        let mut input = String::new();
        reader.read_line(&mut input)?;

        let response = input.trim().to_lowercase();
        Ok(response.is_empty() || response == "y" || response == "yes")
    }

    /// Convert a PRD markdown file to prd.json
    pub fn convert(&self, prd_path: &Path) -> io::Result<PrdConversionResult> {
        let content = fs::read_to_string(prd_path)?;
        self.convert_from_string(&content)
    }

    /// Convert PRD markdown content to prd.json
    pub fn convert_from_string(&self, content: &str) -> io::Result<PrdConversionResult> {
        // Extract project name from title
        let project_name = self
            .config
            .project_name
            .clone()
            .unwrap_or_else(|| extract_project_name(content));

        // Generate branch name
        let branch_name = self.config.branch_name.clone().unwrap_or_else(|| {
            format!("ralph/{}-improvements", sanitize_branch_name(&project_name))
        });

        // Extract description from introduction
        let description = extract_description(content);

        // Parse user stories from markdown
        let stories = parse_user_stories(content);

        // Create prd.json structure
        let mut prd_json = PrdJson::new(project_name.clone(), branch_name.clone());
        if !description.is_empty() {
            prd_json = prd_json.with_description(description);
        }

        for story in &stories {
            prd_json.add_story(story.clone());
        }

        // Write prd.json file
        let prd_json_path = self.config.output_dir.join("prd.json");
        let json_content = serde_json::to_string_pretty(&prd_json)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&prd_json_path, json_content)?;

        Ok(PrdConversionResult {
            prd_json_path,
            story_count: stories.len(),
            project_name,
            branch_name,
        })
    }
}

/// Extract project name from PRD title
fn extract_project_name(content: &str) -> String {
    // Look for "# PRD: <project> Improvements" pattern
    let title_re = Regex::new(r"^#\s+PRD:\s+(.+?)\s+Improvements").unwrap();

    for line in content.lines() {
        if let Some(caps) = title_re.captures(line) {
            return caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "Project".to_string());
        }
    }

    "Project".to_string()
}

/// Extract description from the Introduction section
fn extract_description(content: &str) -> String {
    let mut in_intro = false;
    let mut description_lines = Vec::new();

    for line in content.lines() {
        if line.starts_with("## Introduction") {
            in_intro = true;
            continue;
        }
        if in_intro {
            if line.starts_with("## ") {
                break;
            }
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("**Audit Summary:**") {
                description_lines.push(trimmed);
            }
        }
    }

    description_lines.join(" ")
}

/// Parse user stories from PRD markdown
fn parse_user_stories(content: &str) -> Vec<PrdUserStory> {
    let mut stories = Vec::new();

    // Regex patterns for parsing
    let story_header_re = Regex::new(r"^####\s+(US-\d+):\s+(.+?)(?:\s+\[.*\])?$").unwrap();
    let description_re = Regex::new(r"^\*\*Description:\*\*\s+(.+)$").unwrap();
    let criterion_re = Regex::new(r"^-\s+\[\s*[xX ]?\s*\]\s+(.+)$").unwrap();

    let mut current_story: Option<PrdUserStory> = None;
    let mut in_acceptance_criteria = false;
    let mut priority_counter = 1u32;

    for line in content.lines() {
        // Check for story header
        if let Some(caps) = story_header_re.captures(line) {
            // Save previous story if exists
            if let Some(story) = current_story.take() {
                stories.push(story);
            }

            let id = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let title = caps
                .get(2)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();

            current_story = Some(PrdUserStory {
                id,
                title,
                description: String::new(),
                acceptance_criteria: Vec::new(),
                priority: priority_counter,
                passes: false,
                notes: String::new(),
            });
            priority_counter += 1;
            in_acceptance_criteria = false;
            continue;
        }

        // Check for description
        if let Some(caps) = description_re.captures(line) {
            if let Some(ref mut story) = current_story {
                story.description = caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
            }
            continue;
        }

        // Check for acceptance criteria section
        if line.contains("**Acceptance Criteria:**") {
            in_acceptance_criteria = true;
            continue;
        }

        // Check for criterion
        if in_acceptance_criteria {
            if let Some(caps) = criterion_re.captures(line) {
                if let Some(ref mut story) = current_story {
                    let criterion = caps
                        .get(1)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default();
                    story.acceptance_criteria.push(criterion);
                }
                continue;
            }
            // End of acceptance criteria section (empty line or new section)
            if line.trim().is_empty() || line.starts_with("##") || line.starts_with("####") {
                in_acceptance_criteria = false;
            }
        }
    }

    // Save last story if exists
    if let Some(story) = current_story.take() {
        stories.push(story);
    }

    stories
}

/// Sanitize a string for use in a branch name
fn sanitize_branch_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn sample_prd_content() -> &'static str {
        r#"# PRD: TestProject Improvements

## Introduction

This PRD was auto-generated from a codebase audit of `/test/project`. It contains user stories derived from audit findings and identified feature opportunities.

**Audit Summary:** 3 findings (0 critical, 1 high, 1 medium, 1 low) and 1 opportunities identified.

## Goals

- Address critical and high-severity audit findings
- Implement identified feature opportunities

## User Stories

### Audit Findings

#### US-001: Address: Missing abstraction layer [High]
**Description:** As a developer, I want to add a service layer to separate concerns so that this is a high-priority issue affecting code quality

**Acceptance Criteria:**
- [ ] Add a service layer to separate concerns
- [ ] Address issue in src/api.rs
- [ ] Typecheck passes
- [ ] Tests pass

#### US-002: Address: TODO comments found [Medium]
**Description:** As a developer, I want to address todo comments or convert to issues so that this improves code maintainability and quality

**Acceptance Criteria:**
- [ ] Address TODO comments or convert to issues
- [ ] Address issues in 5 affected files
- [ ] Typecheck passes
- [ ] Tests pass

### Feature Opportunities

#### US-003: Implement /health endpoint [Low Complexity]
**Description:** As a developer, I want to add get /health endpoint returning service status because api exists but lacks health check for monitoring

**Acceptance Criteria:**
- [ ] GET /health returns 200 when healthy
- [ ] Includes database connectivity check

## Functional Requirements

- FR-01: Address: Missing abstraction layer
- FR-02: Address: TODO comments found
- FR-03: Implement /health endpoint

## Non-Goals

- Changes outside the scope of identified findings and opportunities

## Technical Considerations

**File types detected:**
- rs (10 files)
- toml (2 files)

## Success Metrics

- All critical and high-severity findings addressed
- Typecheck passes for all changes

---

*Generated on 2024-01-15T12:00:00Z from audit results.*
"#
    }

    #[test]
    fn test_extract_project_name() {
        let content = sample_prd_content();
        assert_eq!(extract_project_name(content), "TestProject");
    }

    #[test]
    fn test_extract_project_name_not_found() {
        let content = "# Some Other Title\n\nContent here";
        assert_eq!(extract_project_name(content), "Project");
    }

    #[test]
    fn test_extract_description() {
        let content = sample_prd_content();
        let description = extract_description(content);
        assert!(description.contains("auto-generated from a codebase audit"));
        assert!(!description.contains("Audit Summary"));
    }

    #[test]
    fn test_parse_user_stories() {
        let content = sample_prd_content();
        let stories = parse_user_stories(content);

        assert_eq!(stories.len(), 3);

        // Check first story
        assert_eq!(stories[0].id, "US-001");
        assert_eq!(stories[0].title, "Address: Missing abstraction layer");
        assert!(stories[0].description.contains("service layer"));
        assert_eq!(stories[0].acceptance_criteria.len(), 4);
        assert_eq!(stories[0].priority, 1);
        assert!(!stories[0].passes);

        // Check second story
        assert_eq!(stories[1].id, "US-002");
        assert_eq!(stories[1].title, "Address: TODO comments found");

        // Check third story
        assert_eq!(stories[2].id, "US-003");
        assert_eq!(stories[2].title, "Implement /health endpoint");
    }

    #[test]
    fn test_sanitize_branch_name() {
        assert_eq!(sanitize_branch_name("My Project"), "my-project");
        assert_eq!(sanitize_branch_name("test_project"), "test-project");
        assert_eq!(sanitize_branch_name("foo--bar"), "foo-bar");
        assert_eq!(sanitize_branch_name("--test--"), "test");
    }

    #[test]
    fn test_prd_json_serialization() {
        let mut prd = PrdJson::new("TestProject".to_string(), "test/branch".to_string());
        prd = prd.with_description("Test description");
        prd.add_story(PrdUserStory {
            id: "US-001".to_string(),
            title: "Test story".to_string(),
            description: "Test description".to_string(),
            acceptance_criteria: vec!["Criterion 1".to_string()],
            priority: 1,
            passes: false,
            notes: String::new(),
        });

        let json = serde_json::to_string_pretty(&prd).unwrap();
        assert!(json.contains("\"project\": \"TestProject\""));
        assert!(json.contains("\"branchName\": \"test/branch\""));
        assert!(json.contains("\"userStories\""));
        assert!(json.contains("\"acceptanceCriteria\""));
        assert!(json.contains("\"passes\": false"));
    }

    #[test]
    fn test_prd_converter_config_builder() {
        let config = PrdConverterConfig::new()
            .with_skip_prompt(true)
            .with_project_name("my-project")
            .with_branch_name("feature/test")
            .with_output_dir("/output");

        assert!(config.skip_prompt);
        assert_eq!(config.project_name, Some("my-project".to_string()));
        assert_eq!(config.branch_name, Some("feature/test".to_string()));
        assert_eq!(config.output_dir, PathBuf::from("/output"));
    }

    #[test]
    fn test_prompt_confirmation_skipped() {
        let config = PrdConverterConfig::new().with_skip_prompt(true);
        let converter = PrdConverter::with_config(config);

        assert!(converter.prompt_user_confirmation().unwrap());
    }

    #[test]
    fn test_prompt_confirmation_yes() {
        let converter = PrdConverter::new();

        let input = "y\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = converter
            .prompt_with_reader_writer(&mut reader, &mut writer)
            .unwrap();

        assert!(result);
    }

    #[test]
    fn test_prompt_confirmation_empty_defaults_yes() {
        let converter = PrdConverter::new();

        let input = "\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = converter
            .prompt_with_reader_writer(&mut reader, &mut writer)
            .unwrap();

        assert!(result); // Empty input defaults to yes
    }

    #[test]
    fn test_prompt_confirmation_no() {
        let converter = PrdConverter::new();

        let input = "n\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = converter
            .prompt_with_reader_writer(&mut reader, &mut writer)
            .unwrap();

        assert!(!result);
    }

    #[test]
    fn test_convert_from_string() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let content = sample_prd_content();

        let config = PrdConverterConfig::new()
            .with_skip_prompt(true)
            .with_output_dir(temp_dir.path().to_path_buf());

        let converter = PrdConverter::with_config(config);
        let result = converter.convert_from_string(content).unwrap();

        // Check result
        assert_eq!(result.story_count, 3);
        assert_eq!(result.project_name, "TestProject");
        assert!(result.branch_name.contains("testproject"));

        // Check file was created
        assert!(result.prd_json_path.exists());

        // Check content
        let json_content = fs::read_to_string(&result.prd_json_path).unwrap();
        let prd: PrdJson = serde_json::from_str(&json_content).unwrap();

        assert_eq!(prd.project, "TestProject");
        assert_eq!(prd.user_stories.len(), 3);
        assert!(prd.user_stories.iter().all(|s| !s.passes));
    }

    #[test]
    fn test_convert_with_custom_names() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let content = sample_prd_content();

        let config = PrdConverterConfig::new()
            .with_skip_prompt(true)
            .with_project_name("CustomProject")
            .with_branch_name("custom/branch-name")
            .with_output_dir(temp_dir.path().to_path_buf());

        let converter = PrdConverter::with_config(config);
        let result = converter.convert_from_string(content).unwrap();

        assert_eq!(result.project_name, "CustomProject");
        assert_eq!(result.branch_name, "custom/branch-name");

        let json_content = fs::read_to_string(&result.prd_json_path).unwrap();
        let prd: PrdJson = serde_json::from_str(&json_content).unwrap();

        assert_eq!(prd.project, "CustomProject");
        assert_eq!(prd.branch_name, "custom/branch-name");
    }

    #[test]
    fn test_convert_empty_prd() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let content = "# Some document\n\nNo user stories here.";

        let config = PrdConverterConfig::new()
            .with_skip_prompt(true)
            .with_project_name("EmptyProject")
            .with_output_dir(temp_dir.path().to_path_buf());

        let converter = PrdConverter::with_config(config);
        let result = converter.convert_from_string(content).unwrap();

        assert_eq!(result.story_count, 0);
        assert!(result.prd_json_path.exists());

        let json_content = fs::read_to_string(&result.prd_json_path).unwrap();
        let prd: PrdJson = serde_json::from_str(&json_content).unwrap();

        assert_eq!(prd.user_stories.len(), 0);
    }

    #[test]
    fn test_acceptance_criteria_checkbox_parsing() {
        let content = r#"# PRD: Test Improvements

## User Stories

#### US-001: Test story
**Description:** Test description

**Acceptance Criteria:**
- [ ] Unchecked criterion
- [x] Checked criterion
- [X] Also checked criterion
"#;

        let stories = parse_user_stories(content);
        assert_eq!(stories.len(), 1);
        assert_eq!(stories[0].acceptance_criteria.len(), 3);
        assert_eq!(stories[0].acceptance_criteria[0], "Unchecked criterion");
        assert_eq!(stories[0].acceptance_criteria[1], "Checked criterion");
        assert_eq!(stories[0].acceptance_criteria[2], "Also checked criterion");
    }

    #[test]
    fn test_story_priority_ordering() {
        let content = r#"# PRD: Test Improvements

## User Stories

#### US-001: First story
**Description:** First

**Acceptance Criteria:**
- [ ] Criterion

#### US-002: Second story
**Description:** Second

**Acceptance Criteria:**
- [ ] Criterion

#### US-003: Third story
**Description:** Third

**Acceptance Criteria:**
- [ ] Criterion
"#;

        let stories = parse_user_stories(content);
        assert_eq!(stories.len(), 3);
        assert_eq!(stories[0].priority, 1);
        assert_eq!(stories[1].priority, 2);
        assert_eq!(stories[2].priority, 3);
    }

    #[test]
    fn test_prd_user_story_notes_serialization() {
        // Notes should be omitted when empty
        let story = PrdUserStory {
            id: "US-001".to_string(),
            title: "Test".to_string(),
            description: "Desc".to_string(),
            acceptance_criteria: vec![],
            priority: 1,
            passes: false,
            notes: String::new(),
        };

        let json = serde_json::to_string(&story).unwrap();
        assert!(!json.contains("notes"));

        // Notes should be included when non-empty
        let story_with_notes = PrdUserStory {
            id: "US-001".to_string(),
            title: "Test".to_string(),
            description: "Desc".to_string(),
            acceptance_criteria: vec![],
            priority: 1,
            passes: false,
            notes: "Some notes".to_string(),
        };

        let json_with_notes = serde_json::to_string(&story_with_notes).unwrap();
        assert!(json_with_notes.contains("notes"));
        assert!(json_with_notes.contains("Some notes"));
    }

    #[test]
    fn test_convert_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let prd_path = temp_dir.path().join("test-prd.md");
        let content = sample_prd_content();
        fs::write(&prd_path, content).unwrap();

        let config = PrdConverterConfig::new()
            .with_skip_prompt(true)
            .with_output_dir(temp_dir.path().to_path_buf());

        let converter = PrdConverter::with_config(config);
        let result = converter.convert(&prd_path).unwrap();

        assert_eq!(result.story_count, 3);
        assert!(result.prd_json_path.exists());
    }
}
