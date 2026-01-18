//! Markdown output format for audit reports.
//!
//! This module provides markdown formatting for audit reports,
//! enabling human-readable documentation output.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use thiserror::Error;

use crate::audit::{AuditFinding, AuditReport, Complexity, FeatureOpportunity, Severity};

/// Errors that can occur during markdown output operations.
#[derive(Error, Debug)]
pub enum MarkdownOutputError {
    /// IO error during file operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for markdown output operations.
pub type MarkdownOutputResult<T> = Result<T, MarkdownOutputError>;

/// Writer for markdown-formatted audit reports.
pub struct MarkdownReportWriter;

impl MarkdownReportWriter {
    /// Write an audit report to a markdown file.
    ///
    /// # Arguments
    ///
    /// * `report` - The audit report to format.
    /// * `path` - The file path to write the markdown output.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `MarkdownOutputError` on failure.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    /// use ralphmacchio::audit::{AuditReport, output::MarkdownReportWriter};
    ///
    /// let report = AuditReport::new(PathBuf::from("/project"));
    /// MarkdownReportWriter::write_to_file(&report, "audit.md").unwrap();
    /// ```
    pub fn write_to_file<P: AsRef<Path>>(
        report: &AuditReport,
        path: P,
    ) -> MarkdownOutputResult<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        let markdown = Self::to_markdown_string(report);
        writer.write_all(markdown.as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    /// Format an audit report as a markdown string.
    ///
    /// # Arguments
    ///
    /// * `report` - The audit report to format.
    ///
    /// # Returns
    ///
    /// Returns the formatted markdown string.
    pub fn to_markdown_string(report: &AuditReport) -> String {
        let mut output = String::new();

        // Title
        output.push_str("# Codebase Audit Report\n\n");

        // Table of Contents
        output.push_str(&Self::format_table_of_contents(report));

        // Executive Summary
        output.push_str(&Self::format_executive_summary(report));

        // Findings
        output.push_str(&Self::format_findings(report));

        // Opportunities
        output.push_str(&Self::format_opportunities(report));

        // Metadata
        output.push_str(&Self::format_metadata(report));

        output
    }

    /// Format the table of contents section.
    fn format_table_of_contents(report: &AuditReport) -> String {
        let mut toc = String::from("## Table of Contents\n\n");
        toc.push_str("- [Executive Summary](#executive-summary)\n");
        if !report.findings.is_empty() {
            toc.push_str("- [Findings](#findings)\n");
        }
        if !report.opportunities.is_empty() {
            toc.push_str("- [Opportunities](#opportunities)\n");
        }
        toc.push_str("- [Audit Metadata](#audit-metadata)\n");
        toc.push('\n');
        toc
    }

    /// Format the executive summary section with key metrics.
    fn format_executive_summary(report: &AuditReport) -> String {
        let mut summary = String::from("## Executive Summary\n\n");

        let (critical, high, medium, low) = report.finding_counts();
        let total_findings = critical + high + medium + low;
        let total_opportunities = report.opportunities.len();

        // Overview paragraph
        summary.push_str(&format!(
            "This audit analyzed **{}** and identified **{}** findings and **{}** opportunities for improvement.\n\n",
            report.metadata.project_root.display(),
            total_findings,
            total_opportunities
        ));

        // Key Metrics
        summary.push_str("### Key Metrics\n\n");
        summary.push_str("| Metric | Value |\n");
        summary.push_str("|--------|-------|\n");
        summary.push_str(&format!(
            "| Total Files | {} |\n",
            report.inventory.total_files
        ));
        summary.push_str(&format!(
            "| Total Lines of Code | {} |\n",
            report.inventory.total_loc
        ));
        summary.push_str(&format!(
            "| Dependencies | {} |\n",
            report.dependencies.dependencies.len()
        ));
        summary.push_str(&format!("| Total Findings | {} |\n", total_findings));
        summary.push_str(&format!(
            "| Total Opportunities | {} |\n",
            total_opportunities
        ));
        summary.push('\n');

        // Findings breakdown
        if total_findings > 0 {
            summary.push_str("### Findings by Severity\n\n");
            summary.push_str("| Severity | Count |\n");
            summary.push_str("|----------|-------|\n");
            summary.push_str(&format!(
                "| {} Critical | {} |\n",
                Self::severity_badge(Severity::Critical),
                critical
            ));
            summary.push_str(&format!(
                "| {} High | {} |\n",
                Self::severity_badge(Severity::High),
                high
            ));
            summary.push_str(&format!(
                "| {} Medium | {} |\n",
                Self::severity_badge(Severity::Medium),
                medium
            ));
            summary.push_str(&format!(
                "| {} Low | {} |\n",
                Self::severity_badge(Severity::Low),
                low
            ));
            summary.push('\n');
        }

        summary
    }

    /// Format the findings section.
    fn format_findings(report: &AuditReport) -> String {
        if report.findings.is_empty() {
            return String::new();
        }

        let mut section = String::from("## Findings\n\n");

        // Group findings by severity (Critical first, then High, Medium, Low)
        let mut critical: Vec<&AuditFinding> = Vec::new();
        let mut high: Vec<&AuditFinding> = Vec::new();
        let mut medium: Vec<&AuditFinding> = Vec::new();
        let mut low: Vec<&AuditFinding> = Vec::new();

        for finding in &report.findings {
            match finding.severity {
                Severity::Critical => critical.push(finding),
                Severity::High => high.push(finding),
                Severity::Medium => medium.push(finding),
                Severity::Low => low.push(finding),
            }
        }

        // Format each severity group
        if !critical.is_empty() {
            section.push_str("### Critical Severity\n\n");
            for finding in critical {
                section.push_str(&Self::format_finding(finding));
            }
        }

        if !high.is_empty() {
            section.push_str("### High Severity\n\n");
            for finding in high {
                section.push_str(&Self::format_finding(finding));
            }
        }

        if !medium.is_empty() {
            section.push_str("### Medium Severity\n\n");
            for finding in medium {
                section.push_str(&Self::format_finding(finding));
            }
        }

        if !low.is_empty() {
            section.push_str("### Low Severity\n\n");
            for finding in low {
                section.push_str(&Self::format_finding(finding));
            }
        }

        section
    }

    /// Format a single finding.
    fn format_finding(finding: &AuditFinding) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "#### {} {} - {}\n\n",
            Self::severity_badge(finding.severity),
            finding.id,
            finding.title
        ));

        output.push_str(&format!("**Category:** {}\n\n", finding.category));
        output.push_str(&format!("{}\n\n", finding.description));

        if !finding.affected_files.is_empty() {
            output.push_str("**Affected Files:**\n");
            for file in &finding.affected_files {
                output.push_str(&format!("- `{}`\n", file.display()));
            }
            output.push('\n');
        }

        output.push_str(&format!(
            "**Recommendation:** {}\n\n",
            finding.recommendation
        ));
        output.push_str("---\n\n");

        output
    }

    /// Format the opportunities section.
    fn format_opportunities(report: &AuditReport) -> String {
        if report.opportunities.is_empty() {
            return String::new();
        }

        let mut section = String::from("## Opportunities\n\n");

        for opportunity in &report.opportunities {
            section.push_str(&Self::format_opportunity(opportunity));
        }

        section
    }

    /// Format a single opportunity.
    fn format_opportunity(opportunity: &FeatureOpportunity) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "### {} {} - {}\n\n",
            Self::complexity_indicator(opportunity.complexity),
            opportunity.id,
            opportunity.title
        ));

        output.push_str(&format!("**Rationale:** {}\n\n", opportunity.rationale));
        output.push_str(&format!(
            "**Complexity:** {}\n\n",
            Self::complexity_label(opportunity.complexity)
        ));

        if !opportunity.suggested_stories.is_empty() {
            output.push_str("**Suggested User Stories:**\n\n");
            for story in &opportunity.suggested_stories {
                output.push_str(&format!(
                    "{}. **{}** (Priority: {})\n",
                    story.priority, story.title, story.priority
                ));
                output.push_str(&format!("   {}\n", story.description));
                if !story.acceptance_criteria.is_empty() {
                    output.push_str("   - Acceptance Criteria:\n");
                    for criterion in &story.acceptance_criteria {
                        output.push_str(&format!("     - {}\n", criterion));
                    }
                }
                output.push('\n');
            }
        }

        output.push_str("---\n\n");

        output
    }

    /// Format the metadata section.
    fn format_metadata(report: &AuditReport) -> String {
        let mut section = String::from("## Audit Metadata\n\n");

        section.push_str("| Property | Value |\n");
        section.push_str("|----------|-------|\n");
        section.push_str(&format!(
            "| Audit Version | {} |\n",
            report.metadata.audit_version
        ));
        section.push_str(&format!("| Timestamp | {} |\n", report.metadata.timestamp));
        section.push_str(&format!(
            "| Project Root | `{}` |\n",
            report.metadata.project_root.display()
        ));
        if let Some(ref hash) = report.metadata.commit_hash {
            section.push_str(&format!("| Commit | `{}` |\n", hash));
        }
        if let Some(ref branch) = report.metadata.branch {
            section.push_str(&format!("| Branch | {} |\n", branch));
        }
        section.push_str(&format!(
            "| Duration | {} ms |\n",
            report.metadata.duration_ms
        ));
        section.push('\n');

        section
    }

    /// Get a severity badge emoji.
    fn severity_badge(severity: Severity) -> &'static str {
        match severity {
            Severity::Critical => "🔴",
            Severity::High => "🟠",
            Severity::Medium => "🟡",
            Severity::Low => "🟢",
        }
    }

    /// Get a complexity indicator emoji.
    fn complexity_indicator(complexity: Complexity) -> &'static str {
        match complexity {
            Complexity::Low => "⬜",
            Complexity::Medium => "🟦",
            Complexity::High => "🟪",
        }
    }

    /// Get a human-readable complexity label.
    fn complexity_label(complexity: Complexity) -> &'static str {
        match complexity {
            Complexity::Low => "Low - Quick win, minimal effort required",
            Complexity::Medium => "Medium - Moderate effort, some planning needed",
            Complexity::High => "High - Significant effort, requires careful planning",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{AuditMetadata, DependencyAnalysis, FileInventory, SuggestedStory};
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
            inventory: FileInventory {
                total_files: 42,
                total_loc: 5000,
                ..Default::default()
            },
            dependencies: DependencyAnalysis::default(),
            findings: vec![
                AuditFinding {
                    id: "ARCH-001".to_string(),
                    severity: Severity::Critical,
                    category: "architecture".to_string(),
                    title: "Security vulnerability detected".to_string(),
                    description: "SQL injection vulnerability found in user input handling."
                        .to_string(),
                    affected_files: vec![PathBuf::from("src/db.rs")],
                    recommendation: "Use parameterized queries instead of string concatenation."
                        .to_string(),
                },
                AuditFinding {
                    id: "ARCH-002".to_string(),
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
                AuditFinding {
                    id: "STYLE-001".to_string(),
                    severity: Severity::Low,
                    category: "style".to_string(),
                    title: "Inconsistent naming".to_string(),
                    description: "Some functions use snake_case inconsistently.".to_string(),
                    affected_files: vec![],
                    recommendation: "Follow Rust naming conventions.".to_string(),
                },
            ],
            opportunities: vec![
                FeatureOpportunity {
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
                },
                FeatureOpportunity {
                    id: "FEAT-002".to_string(),
                    title: "Performance monitoring".to_string(),
                    rationale: "No visibility into runtime performance.".to_string(),
                    complexity: Complexity::Low,
                    suggested_stories: vec![],
                },
                FeatureOpportunity {
                    id: "FEAT-003".to_string(),
                    title: "Multi-region support".to_string(),
                    rationale: "Single region deployment limits availability.".to_string(),
                    complexity: Complexity::High,
                    suggested_stories: vec![],
                },
            ],
        }
    }

    #[test]
    fn test_write_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("audit.md");

        let report = create_test_report();
        MarkdownReportWriter::write_to_file(&report, &output_path).unwrap();

        assert!(output_path.exists());

        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("# Codebase Audit Report"));
        assert!(content.contains("## Executive Summary"));
        assert!(content.contains("## Findings"));
        assert!(content.contains("## Opportunities"));
    }

    #[test]
    fn test_table_of_contents() {
        let report = create_test_report();
        let markdown = MarkdownReportWriter::to_markdown_string(&report);

        assert!(markdown.contains("## Table of Contents"));
        assert!(markdown.contains("- [Executive Summary](#executive-summary)"));
        assert!(markdown.contains("- [Findings](#findings)"));
        assert!(markdown.contains("- [Opportunities](#opportunities)"));
        assert!(markdown.contains("- [Audit Metadata](#audit-metadata)"));
    }

    #[test]
    fn test_executive_summary_metrics() {
        let report = create_test_report();
        let markdown = MarkdownReportWriter::to_markdown_string(&report);

        assert!(markdown.contains("## Executive Summary"));
        assert!(markdown.contains("### Key Metrics"));
        assert!(markdown.contains("| Total Files | 42 |"));
        assert!(markdown.contains("| Total Lines of Code | 5000 |"));
        assert!(markdown.contains("| Dependencies | 0 |"));
        assert!(markdown.contains("| Total Findings | 4 |"));
        assert!(markdown.contains("| Total Opportunities | 3 |"));
    }

    #[test]
    fn test_findings_by_severity() {
        let report = create_test_report();
        let markdown = MarkdownReportWriter::to_markdown_string(&report);

        assert!(markdown.contains("### Findings by Severity"));
        assert!(markdown.contains("| 🔴 Critical | 1 |"));
        assert!(markdown.contains("| 🟠 High | 1 |"));
        assert!(markdown.contains("| 🟡 Medium | 1 |"));
        assert!(markdown.contains("| 🟢 Low | 1 |"));
    }

    #[test]
    fn test_findings_section_format() {
        let report = create_test_report();
        let markdown = MarkdownReportWriter::to_markdown_string(&report);

        // Check severity sections exist
        assert!(markdown.contains("### Critical Severity"));
        assert!(markdown.contains("### High Severity"));
        assert!(markdown.contains("### Medium Severity"));
        assert!(markdown.contains("### Low Severity"));

        // Check severity badges in findings
        assert!(markdown.contains("#### 🔴 ARCH-001 - Security vulnerability detected"));
        assert!(markdown.contains("#### 🟠 ARCH-002 - Missing error handling"));
        assert!(markdown.contains("#### 🟡 DOC-001 - Missing module documentation"));
        assert!(markdown.contains("#### 🟢 STYLE-001 - Inconsistent naming"));
    }

    #[test]
    fn test_finding_details() {
        let report = create_test_report();
        let markdown = MarkdownReportWriter::to_markdown_string(&report);

        // Check finding details
        assert!(markdown.contains("**Category:** architecture"));
        assert!(markdown.contains("SQL injection vulnerability found"));
        assert!(markdown.contains("**Affected Files:**"));
        assert!(markdown.contains("- `src/db.rs`"));
        assert!(markdown.contains("**Recommendation:**"));
    }

    #[test]
    fn test_opportunities_section() {
        let report = create_test_report();
        let markdown = MarkdownReportWriter::to_markdown_string(&report);

        assert!(markdown.contains("## Opportunities"));
        assert!(markdown.contains("### 🟦 FEAT-001 - Add caching layer"));
        assert!(markdown.contains("### ⬜ FEAT-002 - Performance monitoring"));
        assert!(markdown.contains("### 🟪 FEAT-003 - Multi-region support"));
    }

    #[test]
    fn test_complexity_indicators() {
        let report = create_test_report();
        let markdown = MarkdownReportWriter::to_markdown_string(&report);

        // Check complexity labels
        assert!(markdown.contains("**Complexity:** Low - Quick win, minimal effort required"));
        assert!(markdown.contains("**Complexity:** Medium - Moderate effort, some planning needed"));
        assert!(markdown
            .contains("**Complexity:** High - Significant effort, requires careful planning"));
    }

    #[test]
    fn test_suggested_stories() {
        let report = create_test_report();
        let markdown = MarkdownReportWriter::to_markdown_string(&report);

        assert!(markdown.contains("**Suggested User Stories:**"));
        assert!(markdown.contains("**Implement cache infrastructure**"));
        assert!(markdown.contains("**Add cache to hot paths**"));
        assert!(markdown.contains("- Acceptance Criteria:"));
        assert!(markdown.contains("- Cache client configured"));
    }

    #[test]
    fn test_metadata_section() {
        let report = create_test_report();
        let markdown = MarkdownReportWriter::to_markdown_string(&report);

        assert!(markdown.contains("## Audit Metadata"));
        assert!(markdown.contains("| Audit Version | 0.1.0 |"));
        assert!(markdown.contains("| Timestamp | 2024-01-15T10:30:00Z |"));
        assert!(markdown.contains("| Project Root | `/test/project` |"));
        assert!(markdown.contains("| Commit | `abc123` |"));
        assert!(markdown.contains("| Branch | main |"));
        assert!(markdown.contains("| Duration | 1500 ms |"));
    }

    #[test]
    fn test_empty_report() {
        let report = AuditReport::new(PathBuf::from("/empty"));
        let markdown = MarkdownReportWriter::to_markdown_string(&report);

        // Should still have basic structure
        assert!(markdown.contains("# Codebase Audit Report"));
        assert!(markdown.contains("## Table of Contents"));
        assert!(markdown.contains("## Executive Summary"));
        assert!(markdown.contains("## Audit Metadata"));

        // Should not have findings or opportunities sections
        assert!(!markdown.contains("## Findings"));
        assert!(!markdown.contains("## Opportunities"));
    }

    #[test]
    fn test_empty_toc_without_findings_and_opportunities() {
        let report = AuditReport::new(PathBuf::from("/empty"));
        let toc = MarkdownReportWriter::format_table_of_contents(&report);

        // Should not include links to empty sections
        assert!(!toc.contains("- [Findings]"));
        assert!(!toc.contains("- [Opportunities]"));
    }

    #[test]
    fn test_severity_badge_function() {
        assert_eq!(
            MarkdownReportWriter::severity_badge(Severity::Critical),
            "🔴"
        );
        assert_eq!(MarkdownReportWriter::severity_badge(Severity::High), "🟠");
        assert_eq!(MarkdownReportWriter::severity_badge(Severity::Medium), "🟡");
        assert_eq!(MarkdownReportWriter::severity_badge(Severity::Low), "🟢");
    }

    #[test]
    fn test_complexity_indicator_function() {
        assert_eq!(
            MarkdownReportWriter::complexity_indicator(Complexity::Low),
            "⬜"
        );
        assert_eq!(
            MarkdownReportWriter::complexity_indicator(Complexity::Medium),
            "🟦"
        );
        assert_eq!(
            MarkdownReportWriter::complexity_indicator(Complexity::High),
            "🟪"
        );
    }

    #[test]
    fn test_no_findings_no_severity_breakdown() {
        let mut report = AuditReport::new(PathBuf::from("/test"));
        report.findings.clear();
        let markdown = MarkdownReportWriter::to_markdown_string(&report);

        // Should not have findings by severity section when no findings
        assert!(!markdown.contains("### Findings by Severity"));
    }
}
