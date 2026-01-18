//! PRD generation from audit findings and opportunities.
//!
//! This module converts audit findings and feature opportunities into a
//! Product Requirements Document (PRD) that follows the existing /prd skill format.

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use crate::audit::interactive::UserAnswers;
use crate::audit::{AuditFinding, AuditReport, Complexity, FeatureOpportunity, Severity};

/// Configuration for PRD generation
#[derive(Debug, Clone, Default)]
pub struct PrdGeneratorConfig {
    /// Skip the user confirmation prompt
    pub skip_prompt: bool,
    /// Project name (derived from directory name if not provided)
    pub project_name: Option<String>,
    /// Output directory for the PRD (defaults to "tasks/")
    pub output_dir: PathBuf,
    /// User answers from the interactive Q&A session
    pub user_answers: Option<UserAnswers>,
}

impl PrdGeneratorConfig {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        Self {
            skip_prompt: false,
            project_name: None,
            output_dir: PathBuf::from("tasks"),
            user_answers: None,
        }
    }

    /// Set skip_prompt flag (for --generate-prd CLI flag)
    pub fn with_skip_prompt(mut self, skip: bool) -> Self {
        self.skip_prompt = skip;
        self
    }

    /// Set the project name
    pub fn with_project_name(mut self, name: impl Into<String>) -> Self {
        self.project_name = Some(name.into());
        self
    }

    /// Set the output directory
    pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }

    /// Set user answers from interactive session
    pub fn with_user_answers(mut self, answers: UserAnswers) -> Self {
        self.user_answers = Some(answers);
        self
    }
}

/// A user story generated from findings or opportunities
#[derive(Debug, Clone)]
pub struct GeneratedUserStory {
    /// Story ID (e.g., "US-001")
    pub id: String,
    /// Story title
    pub title: String,
    /// Story description
    pub description: String,
    /// Acceptance criteria
    pub acceptance_criteria: Vec<String>,
    /// Priority (lower = higher priority)
    pub priority: u32,
    /// Source of the story (finding or opportunity)
    pub source: StorySource,
}

/// Source of a generated user story
#[derive(Debug, Clone)]
pub enum StorySource {
    /// Generated from an audit finding
    Finding {
        id: String,
        severity: Severity,
        category: String,
    },
    /// Generated from a feature opportunity
    Opportunity { id: String, complexity: Complexity },
}

/// Result of PRD generation
#[derive(Debug, Clone)]
pub struct PrdGenerationResult {
    /// Path to the generated PRD file
    pub prd_path: PathBuf,
    /// Number of user stories generated
    pub story_count: usize,
    /// Number of findings converted
    pub findings_converted: usize,
    /// Number of opportunities converted
    pub opportunities_converted: usize,
}

/// Generator for PRD documents from audit results
pub struct PrdGenerator {
    config: PrdGeneratorConfig,
}

impl Default for PrdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl PrdGenerator {
    /// Create a new PRD generator with default configuration
    pub fn new() -> Self {
        Self {
            config: PrdGeneratorConfig::new(),
        }
    }

    /// Create a new PRD generator with custom configuration
    pub fn with_config(config: PrdGeneratorConfig) -> Self {
        Self { config }
    }

    /// Get the configuration
    pub fn config(&self) -> &PrdGeneratorConfig {
        &self.config
    }

    /// Prompt the user for confirmation before generating the PRD
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
            "│  📝 PRD Generation                                          │"
        )?;
        writeln!(
            writer,
            "│  Generate a PRD from audit findings and opportunities?     │"
        )?;
        writeln!(
            writer,
            "╰─────────────────────────────────────────────────────────────╯"
        )?;
        writeln!(writer)?;
        writeln!(
            writer,
            "This will create a PRD file with user stories based on the audit results."
        )?;
        writeln!(
            writer,
            "The PRD can then be used with Ralph to implement improvements."
        )?;
        writeln!(writer)?;
        write!(writer, "Generate PRD? [Y/n]: ")?;
        writer.flush()?;

        let mut input = String::new();
        reader.read_line(&mut input)?;

        let response = input.trim().to_lowercase();
        Ok(response.is_empty() || response == "y" || response == "yes")
    }

    /// Generate a PRD from the audit report
    pub fn generate(&self, report: &AuditReport) -> io::Result<PrdGenerationResult> {
        // Convert findings to user stories
        let finding_stories = self.findings_to_stories(&report.findings);
        let findings_converted = finding_stories.len();

        // Convert opportunities to user stories
        let opportunity_stories = self.opportunities_to_stories(&report.opportunities);
        let opportunities_converted = opportunity_stories.len();

        // Combine and sort by priority
        let mut all_stories = finding_stories;
        all_stories.extend(opportunity_stories);
        all_stories.sort_by_key(|s| s.priority);

        // Renumber stories sequentially
        let all_stories: Vec<GeneratedUserStory> = all_stories
            .into_iter()
            .enumerate()
            .map(|(i, mut s)| {
                s.id = format!("US-{:03}", i + 1);
                s
            })
            .collect();

        let story_count = all_stories.len();

        // Generate PRD markdown
        let prd_content = self.generate_prd_markdown(report, &all_stories);

        // Determine project name
        let project_name = self.config.project_name.clone().unwrap_or_else(|| {
            report
                .metadata
                .project_root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project")
                .to_string()
        });

        // Create output directory if it doesn't exist
        fs::create_dir_all(&self.config.output_dir)?;

        // Write PRD file
        let prd_filename = format!("prd-{}-improvements.md", sanitize_filename(&project_name));
        let prd_path = self.config.output_dir.join(&prd_filename);
        fs::write(&prd_path, prd_content)?;

        Ok(PrdGenerationResult {
            prd_path,
            story_count,
            findings_converted,
            opportunities_converted,
        })
    }

    /// Convert audit findings to user stories
    pub fn findings_to_stories(&self, findings: &[AuditFinding]) -> Vec<GeneratedUserStory> {
        findings
            .iter()
            .filter(|f| f.severity >= Severity::Medium) // Only Medium and above
            .map(|finding| {
                let priority = match finding.severity {
                    Severity::Critical => 1,
                    Severity::High => 2,
                    Severity::Medium => 3,
                    Severity::Low => 4,
                };

                let acceptance_criteria = self.finding_to_acceptance_criteria(finding);

                GeneratedUserStory {
                    id: finding.id.clone(),
                    title: format!("Address: {}", finding.title),
                    description: format!(
                        "As a developer, I want to {} so that {}",
                        finding.recommendation.to_lowercase().trim_end_matches('.'),
                        self.finding_rationale(finding)
                    ),
                    acceptance_criteria,
                    priority,
                    source: StorySource::Finding {
                        id: finding.id.clone(),
                        severity: finding.severity,
                        category: finding.category.clone(),
                    },
                }
            })
            .collect()
    }

    /// Convert feature opportunities to user stories
    pub fn opportunities_to_stories(
        &self,
        opportunities: &[FeatureOpportunity],
    ) -> Vec<GeneratedUserStory> {
        let mut stories = Vec::new();

        for opportunity in opportunities {
            // If the opportunity has suggested stories, use them
            if !opportunity.suggested_stories.is_empty() {
                for suggested in &opportunity.suggested_stories {
                    let priority = match opportunity.complexity {
                        Complexity::Low => 3 + suggested.priority,
                        Complexity::Medium => 4 + suggested.priority,
                        Complexity::High => 5 + suggested.priority,
                    };

                    stories.push(GeneratedUserStory {
                        id: format!("{}-{}", opportunity.id, suggested.priority),
                        title: suggested.title.clone(),
                        description: format!(
                            "As a developer, I want to {} because {}",
                            suggested.description.to_lowercase().trim_end_matches('.'),
                            opportunity.rationale.to_lowercase()
                        ),
                        acceptance_criteria: suggested.acceptance_criteria.clone(),
                        priority,
                        source: StorySource::Opportunity {
                            id: opportunity.id.clone(),
                            complexity: opportunity.complexity,
                        },
                    });
                }
            } else {
                // Create a generic story for the opportunity
                let priority = match opportunity.complexity {
                    Complexity::Low => 3,
                    Complexity::Medium => 4,
                    Complexity::High => 5,
                };

                stories.push(GeneratedUserStory {
                    id: opportunity.id.clone(),
                    title: format!("Implement: {}", opportunity.title),
                    description: format!(
                        "As a developer, I want to implement {} because {}",
                        opportunity.title.to_lowercase(),
                        opportunity.rationale.to_lowercase()
                    ),
                    acceptance_criteria: vec![
                        "Feature is implemented".to_string(),
                        "Tests pass".to_string(),
                        "Documentation is updated".to_string(),
                    ],
                    priority,
                    source: StorySource::Opportunity {
                        id: opportunity.id.clone(),
                        complexity: opportunity.complexity,
                    },
                });
            }
        }

        stories
    }

    /// Generate acceptance criteria from a finding
    fn finding_to_acceptance_criteria(&self, finding: &AuditFinding) -> Vec<String> {
        let mut criteria = Vec::new();

        // Base criteria from the recommendation
        criteria.push(finding.recommendation.clone());

        // Add affected files as criteria if specific
        if !finding.affected_files.is_empty() && finding.affected_files.len() <= 3 {
            for file in &finding.affected_files {
                criteria.push(format!("Address issue in {}", file.display()));
            }
        } else if finding.affected_files.len() > 3 {
            criteria.push(format!(
                "Address issues in {} affected files",
                finding.affected_files.len()
            ));
        }

        // Standard quality criteria
        criteria.push("Typecheck passes".to_string());
        criteria.push("Tests pass".to_string());

        criteria
    }

    /// Generate rationale text from a finding
    fn finding_rationale(&self, finding: &AuditFinding) -> String {
        match finding.severity {
            Severity::Critical => "this is a critical issue that requires immediate attention",
            Severity::High => "this is a high-priority issue affecting code quality",
            Severity::Medium => "this improves code maintainability and quality",
            Severity::Low => "this is a minor improvement to consider",
        }
        .to_string()
    }

    /// Generate the PRD markdown content
    fn generate_prd_markdown(
        &self,
        report: &AuditReport,
        stories: &[GeneratedUserStory],
    ) -> String {
        let project_name = self.config.project_name.clone().unwrap_or_else(|| {
            report
                .metadata
                .project_root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Project")
                .to_string()
        });

        let mut md = String::new();

        // Title
        md.push_str(&format!(
            "# PRD: {} Improvements\n\n",
            capitalize(&project_name)
        ));

        // Introduction
        md.push_str("## Introduction\n\n");
        md.push_str(&format!(
            "This PRD was auto-generated from a codebase audit of `{}`. ",
            report.metadata.project_root.display()
        ));
        md.push_str("It contains user stories derived from audit findings and identified feature opportunities.\n\n");

        // Summary of audit
        let (critical, high, medium, low) = report.finding_counts();
        md.push_str(&format!(
            "**Audit Summary:** {} findings ({} critical, {} high, {} medium, {} low) and {} opportunities identified.\n\n",
            report.findings.len(),
            critical,
            high,
            medium,
            low,
            report.opportunities.len()
        ));

        // Goals
        md.push_str("## Goals\n\n");
        md.push_str("- Address critical and high-severity audit findings\n");
        md.push_str("- Implement identified feature opportunities\n");
        md.push_str("- Improve overall code quality and maintainability\n");
        if critical > 0 || high > 0 {
            md.push_str("- Resolve security and stability concerns\n");
        }
        md.push('\n');

        // User Stories
        md.push_str("## User Stories\n\n");

        // Group stories by source type
        let finding_stories: Vec<_> = stories
            .iter()
            .filter(|s| matches!(s.source, StorySource::Finding { .. }))
            .collect();
        let opportunity_stories: Vec<_> = stories
            .iter()
            .filter(|s| matches!(s.source, StorySource::Opportunity { .. }))
            .collect();

        // Findings section
        if !finding_stories.is_empty() {
            md.push_str("### Audit Findings\n\n");
            for story in &finding_stories {
                md.push_str(&self.format_user_story(story));
            }
        }

        // Opportunities section
        if !opportunity_stories.is_empty() {
            md.push_str("### Feature Opportunities\n\n");
            for story in &opportunity_stories {
                md.push_str(&self.format_user_story(story));
            }
        }

        // Functional Requirements
        md.push_str("## Functional Requirements\n\n");
        for (i, story) in stories.iter().enumerate() {
            md.push_str(&format!("- FR-{:02}: {}\n", i + 1, story.title));
        }
        md.push('\n');

        // Non-Goals
        md.push_str("## Non-Goals\n\n");
        md.push_str("- Changes outside the scope of identified findings and opportunities\n");
        md.push_str("- Major architectural refactoring beyond what's recommended\n");
        md.push_str("- New features not related to audit findings\n\n");

        // Technical Considerations
        md.push_str("## Technical Considerations\n\n");
        if !report.inventory.files_by_extension.is_empty() {
            md.push_str("**File types detected:**\n");
            let mut extensions: Vec<_> = report.inventory.files_by_extension.iter().collect();
            extensions.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
            for (ext, count) in extensions.iter().take(5) {
                let ext_display = if ext.is_empty() {
                    "(no extension)"
                } else {
                    ext.as_str()
                };
                md.push_str(&format!("- {} ({} files)\n", ext_display, count));
            }
            md.push('\n');
        }

        // Success Metrics
        md.push_str("## Success Metrics\n\n");
        md.push_str("- All critical and high-severity findings addressed\n");
        md.push_str("- Typecheck passes for all changes\n");
        md.push_str("- All existing tests continue to pass\n");
        md.push_str("- Code coverage maintained or improved\n\n");

        // Generated timestamp
        md.push_str("---\n\n");
        md.push_str(&format!(
            "*Generated on {} from audit results.*\n",
            report.metadata.timestamp
        ));

        md
    }

    /// Format a single user story in markdown
    fn format_user_story(&self, story: &GeneratedUserStory) -> String {
        let mut md = String::new();

        // Story header with source info
        let source_badge = match &story.source {
            StorySource::Finding { severity, .. } => {
                format!(" [{}]", severity_badge(*severity))
            }
            StorySource::Opportunity { complexity, .. } => {
                format!(" [{}]", complexity_badge(*complexity))
            }
        };

        md.push_str(&format!(
            "#### {}: {}{}\n",
            story.id, story.title, source_badge
        ));
        md.push_str(&format!("**Description:** {}\n\n", story.description));

        md.push_str("**Acceptance Criteria:**\n");
        for criterion in &story.acceptance_criteria {
            md.push_str(&format!("- [ ] {}\n", criterion));
        }
        md.push('\n');

        md
    }
}

/// Sanitize a string for use in a filename
fn sanitize_filename(name: &str) -> String {
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

/// Capitalize the first letter of a string
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Get a severity badge string
fn severity_badge(severity: Severity) -> &'static str {
    match severity {
        Severity::Critical => "🔴 Critical",
        Severity::High => "🟠 High",
        Severity::Medium => "🟡 Medium",
        Severity::Low => "🟢 Low",
    }
}

/// Get a complexity badge string
fn complexity_badge(complexity: Complexity) -> &'static str {
    match complexity {
        Complexity::Low => "✅ Low Complexity",
        Complexity::Medium => "⚡ Medium Complexity",
        Complexity::High => "🔧 High Complexity",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{AuditMetadata, FileInventory, SuggestedStory};
    use std::collections::HashMap;
    use std::io::Cursor;

    fn create_test_report() -> AuditReport {
        let mut files_by_extension = HashMap::new();
        files_by_extension.insert("rs".to_string(), 10);
        files_by_extension.insert("toml".to_string(), 2);

        AuditReport {
            metadata: AuditMetadata {
                audit_version: "0.1.0".to_string(),
                timestamp: "2024-01-15T12:00:00Z".to_string(),
                project_root: PathBuf::from("/test/project"),
                commit_hash: None,
                branch: None,
                duration_ms: 1000,
            },
            inventory: FileInventory {
                files_by_extension,
                total_files: 12,
                total_loc: 1000,
                ..Default::default()
            },
            dependencies: Default::default(),
            findings: vec![
                AuditFinding {
                    id: "ARCH-001".to_string(),
                    severity: Severity::High,
                    category: "architecture".to_string(),
                    title: "Missing abstraction layer".to_string(),
                    description: "No service layer between API and database".to_string(),
                    affected_files: vec![PathBuf::from("src/api.rs")],
                    recommendation: "Add a service layer to separate concerns".to_string(),
                },
                AuditFinding {
                    id: "DEBT-001".to_string(),
                    severity: Severity::Medium,
                    category: "tech_debt".to_string(),
                    title: "TODO comments found".to_string(),
                    description: "5 TODO comments in the codebase".to_string(),
                    affected_files: vec![PathBuf::from("src/main.rs"), PathBuf::from("src/lib.rs")],
                    recommendation: "Address TODO comments or convert to issues".to_string(),
                },
                AuditFinding {
                    id: "LOW-001".to_string(),
                    severity: Severity::Low,
                    category: "style".to_string(),
                    title: "Minor style issue".to_string(),
                    description: "Some style inconsistencies".to_string(),
                    affected_files: vec![],
                    recommendation: "Run formatter".to_string(),
                },
            ],
            opportunities: vec![FeatureOpportunity {
                id: "FEAT-001".to_string(),
                title: "Add health check endpoint".to_string(),
                rationale: "API exists but lacks health check for monitoring".to_string(),
                complexity: Complexity::Low,
                suggested_stories: vec![SuggestedStory {
                    title: "Implement /health endpoint".to_string(),
                    description: "Add GET /health endpoint returning service status".to_string(),
                    acceptance_criteria: vec![
                        "GET /health returns 200 when healthy".to_string(),
                        "Includes database connectivity check".to_string(),
                    ],
                    priority: 1,
                }],
            }],
        }
    }

    #[test]
    fn test_prd_generator_config_builder() {
        let config = PrdGeneratorConfig::new()
            .with_skip_prompt(true)
            .with_project_name("my-project")
            .with_output_dir("/output");

        assert!(config.skip_prompt);
        assert_eq!(config.project_name, Some("my-project".to_string()));
        assert_eq!(config.output_dir, PathBuf::from("/output"));
    }

    #[test]
    fn test_findings_to_stories_filters_low_severity() {
        let report = create_test_report();
        let generator = PrdGenerator::new();

        let stories = generator.findings_to_stories(&report.findings);

        // Should only include high and medium severity findings
        assert_eq!(stories.len(), 2);
        assert!(stories.iter().all(|s| {
            if let StorySource::Finding { severity, .. } = &s.source {
                *severity >= Severity::Medium
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_findings_to_stories_priority_order() {
        let report = create_test_report();
        let generator = PrdGenerator::new();

        let stories = generator.findings_to_stories(&report.findings);

        // Critical/High should have lower priority number (higher priority)
        let high_story = stories.iter().find(|s| {
            if let StorySource::Finding { severity, .. } = &s.source {
                *severity == Severity::High
            } else {
                false
            }
        });

        let medium_story = stories.iter().find(|s| {
            if let StorySource::Finding { severity, .. } = &s.source {
                *severity == Severity::Medium
            } else {
                false
            }
        });

        assert!(high_story.unwrap().priority < medium_story.unwrap().priority);
    }

    #[test]
    fn test_opportunities_to_stories() {
        let report = create_test_report();
        let generator = PrdGenerator::new();

        let stories = generator.opportunities_to_stories(&report.opportunities);

        // Should create stories from suggested stories
        assert!(!stories.is_empty());
        assert!(stories[0].title.contains("health"));
    }

    #[test]
    fn test_opportunities_to_stories_without_suggestions() {
        let opportunities = vec![FeatureOpportunity {
            id: "FEAT-002".to_string(),
            title: "Add caching".to_string(),
            rationale: "Improve performance".to_string(),
            complexity: Complexity::Medium,
            suggested_stories: vec![], // No suggestions
        }];

        let generator = PrdGenerator::new();
        let stories = generator.opportunities_to_stories(&opportunities);

        // Should create a generic story
        assert_eq!(stories.len(), 1);
        assert!(stories[0].title.contains("Implement:"));
    }

    #[test]
    fn test_generate_creates_prd_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let report = create_test_report();

        let config = PrdGeneratorConfig::new()
            .with_skip_prompt(true)
            .with_project_name("test-project")
            .with_output_dir(temp_dir.path().to_path_buf());

        let generator = PrdGenerator::with_config(config);
        let result = generator.generate(&report).unwrap();

        // Check file was created
        assert!(result.prd_path.exists());
        assert!(result.prd_path.to_string_lossy().contains("test-project"));

        // Check content
        let content = fs::read_to_string(&result.prd_path).unwrap();
        assert!(content.contains("# PRD:"));
        assert!(content.contains("## User Stories"));
        assert!(content.contains("US-001"));
    }

    #[test]
    fn test_generate_result_counts() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let report = create_test_report();

        let config = PrdGeneratorConfig::new()
            .with_skip_prompt(true)
            .with_output_dir(temp_dir.path().to_path_buf());

        let generator = PrdGenerator::with_config(config);
        let result = generator.generate(&report).unwrap();

        // 2 findings (medium+high) + 1 opportunity
        assert_eq!(result.findings_converted, 2);
        assert_eq!(result.opportunities_converted, 1);
        assert_eq!(result.story_count, 3);
    }

    #[test]
    fn test_prompt_confirmation_skipped() {
        let config = PrdGeneratorConfig::new().with_skip_prompt(true);
        let generator = PrdGenerator::with_config(config);

        assert!(generator.prompt_user_confirmation().unwrap());
    }

    #[test]
    fn test_prompt_confirmation_yes() {
        let generator = PrdGenerator::new();

        let input = "y\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = generator
            .prompt_with_reader_writer(&mut reader, &mut writer)
            .unwrap();

        assert!(result);
    }

    #[test]
    fn test_prompt_confirmation_empty_defaults_yes() {
        let generator = PrdGenerator::new();

        let input = "\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = generator
            .prompt_with_reader_writer(&mut reader, &mut writer)
            .unwrap();

        assert!(result); // Empty input defaults to yes
    }

    #[test]
    fn test_prompt_confirmation_no() {
        let generator = PrdGenerator::new();

        let input = "n\n";
        let mut reader = Cursor::new(input);
        let mut writer = Vec::new();

        let result = generator
            .prompt_with_reader_writer(&mut reader, &mut writer)
            .unwrap();

        assert!(!result);
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("My Project"), "my-project");
        assert_eq!(sanitize_filename("test_project"), "test-project");
        assert_eq!(sanitize_filename("foo--bar"), "foo-bar");
        assert_eq!(sanitize_filename("--test--"), "test");
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize("HELLO"), "HELLO");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("a"), "A");
    }

    #[test]
    fn test_severity_badge() {
        assert!(severity_badge(Severity::Critical).contains("Critical"));
        assert!(severity_badge(Severity::High).contains("High"));
        assert!(severity_badge(Severity::Medium).contains("Medium"));
        assert!(severity_badge(Severity::Low).contains("Low"));
    }

    #[test]
    fn test_complexity_badge() {
        assert!(complexity_badge(Complexity::Low).contains("Low"));
        assert!(complexity_badge(Complexity::Medium).contains("Medium"));
        assert!(complexity_badge(Complexity::High).contains("High"));
    }

    #[test]
    fn test_prd_markdown_structure() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let report = create_test_report();

        let config = PrdGeneratorConfig::new()
            .with_skip_prompt(true)
            .with_project_name("test-project")
            .with_output_dir(temp_dir.path().to_path_buf());

        let generator = PrdGenerator::with_config(config);
        let result = generator.generate(&report).unwrap();

        let content = fs::read_to_string(&result.prd_path).unwrap();

        // Check all required sections exist
        assert!(content.contains("# PRD:"));
        assert!(content.contains("## Introduction"));
        assert!(content.contains("## Goals"));
        assert!(content.contains("## User Stories"));
        assert!(content.contains("### Audit Findings"));
        assert!(content.contains("### Feature Opportunities"));
        assert!(content.contains("## Functional Requirements"));
        assert!(content.contains("## Non-Goals"));
        assert!(content.contains("## Technical Considerations"));
        assert!(content.contains("## Success Metrics"));
    }

    #[test]
    fn test_acceptance_criteria_from_finding() {
        let generator = PrdGenerator::new();
        let finding = AuditFinding {
            id: "TEST-001".to_string(),
            severity: Severity::High,
            category: "test".to_string(),
            title: "Test finding".to_string(),
            description: "Test description".to_string(),
            affected_files: vec![PathBuf::from("src/test.rs")],
            recommendation: "Fix the issue".to_string(),
        };

        let criteria = generator.finding_to_acceptance_criteria(&finding);

        assert!(criteria.contains(&"Fix the issue".to_string()));
        assert!(criteria.iter().any(|c| c.contains("src/test.rs")));
        assert!(criteria.contains(&"Typecheck passes".to_string()));
        assert!(criteria.contains(&"Tests pass".to_string()));
    }

    #[test]
    fn test_acceptance_criteria_many_files() {
        let generator = PrdGenerator::new();
        let finding = AuditFinding {
            id: "TEST-001".to_string(),
            severity: Severity::High,
            category: "test".to_string(),
            title: "Test finding".to_string(),
            description: "Test description".to_string(),
            affected_files: vec![
                PathBuf::from("src/a.rs"),
                PathBuf::from("src/b.rs"),
                PathBuf::from("src/c.rs"),
                PathBuf::from("src/d.rs"),
                PathBuf::from("src/e.rs"),
            ],
            recommendation: "Fix the issue".to_string(),
        };

        let criteria = generator.finding_to_acceptance_criteria(&finding);

        // Should mention count instead of listing all files
        assert!(criteria.iter().any(|c| c.contains("5 affected files")));
    }

    #[test]
    fn test_story_source_matching() {
        let report = create_test_report();
        let generator = PrdGenerator::new();

        let finding_stories = generator.findings_to_stories(&report.findings);
        let opportunity_stories = generator.opportunities_to_stories(&report.opportunities);

        // All finding stories should have Finding source
        for story in &finding_stories {
            assert!(matches!(story.source, StorySource::Finding { .. }));
        }

        // All opportunity stories should have Opportunity source
        for story in &opportunity_stories {
            assert!(matches!(story.source, StorySource::Opportunity { .. }));
        }
    }

    #[test]
    fn test_empty_report() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let report = AuditReport {
            metadata: AuditMetadata {
                audit_version: "0.1.0".to_string(),
                timestamp: "2024-01-15T12:00:00Z".to_string(),
                project_root: PathBuf::from("/test/empty"),
                commit_hash: None,
                branch: None,
                duration_ms: 100,
            },
            inventory: FileInventory::default(),
            dependencies: Default::default(),
            findings: vec![],
            opportunities: vec![],
        };

        let config = PrdGeneratorConfig::new()
            .with_skip_prompt(true)
            .with_output_dir(temp_dir.path().to_path_buf());

        let generator = PrdGenerator::with_config(config);
        let result = generator.generate(&report).unwrap();

        assert_eq!(result.story_count, 0);
        assert_eq!(result.findings_converted, 0);
        assert_eq!(result.opportunities_converted, 0);
        assert!(result.prd_path.exists());
    }

    #[test]
    fn test_project_name_from_path() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let report = AuditReport {
            metadata: AuditMetadata {
                audit_version: "0.1.0".to_string(),
                timestamp: "2024-01-15T12:00:00Z".to_string(),
                project_root: PathBuf::from("/home/user/my-awesome-project"),
                commit_hash: None,
                branch: None,
                duration_ms: 100,
            },
            inventory: FileInventory::default(),
            dependencies: Default::default(),
            findings: vec![],
            opportunities: vec![],
        };

        let config = PrdGeneratorConfig::new()
            .with_skip_prompt(true)
            .with_output_dir(temp_dir.path().to_path_buf());

        let generator = PrdGenerator::with_config(config);
        let result = generator.generate(&report).unwrap();

        // Should derive name from path
        assert!(result
            .prd_path
            .to_string_lossy()
            .contains("my-awesome-project"));
    }
}
