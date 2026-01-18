//! Codebase audit module for Ralph.
//!
//! This module provides comprehensive codebase analysis including:
//! - File structure inventory and language detection
//! - Dependency parsing across multiple ecosystems
//! - Architecture pattern analysis
//! - Gap detection and opportunity identification
//! - Report generation in multiple formats

// Allow dead code for scaffolding - will be used in future stories
#![allow(dead_code)]

pub mod api;
pub mod architecture;
pub mod dependencies;
pub mod detectors;
pub mod documentation;
pub mod interactive;
pub mod inventory;
pub mod languages;
pub mod output;
pub mod patterns;
pub mod prd_converter;
pub mod prd_generator;
pub mod testing;

// Re-exports for convenience
#[allow(unused_imports)]
pub use api::{
    ApiAnalysis, ApiFramework, ApiInventory, CliCommand, HttpEndpoint, HttpMethod, McpTool,
};
#[allow(unused_imports)]
pub use architecture::{
    ArchitectureAnalysis, ArchitectureAnalyzer, ArchitectureLayer, ArchitecturePattern,
    BoundaryViolation, CouplingStrength, LayerType, ModuleCoupling,
};
#[allow(unused_imports)]
pub use dependencies::{
    Dependency, DependencyAnalysis, DependencyEcosystem, DependencyParser, OutdatedInfo,
};
#[allow(unused_imports)]
pub use detectors::{
    ArchitectureGap, ArchitectureGapType, ArchitectureGapsAnalysis, ArchitectureGapsDetector,
    OpportunityAnalysis, OpportunityContext, OpportunityDetector, OpportunityPattern,
    OpportunityType, TechDebtAnalysis, TechDebtDetector, TechDebtItem, TechDebtType,
};
#[allow(unused_imports)]
pub use documentation::{
    DocAnalyzer, DocGap, DocGapType, DocSeverity, DocumentationAnalysis, ReadmeAnalysis,
    UndocumentedItem,
};
#[allow(unused_imports)]
pub use interactive::{
    InteractiveConfig, InteractiveSession, ProjectPriority, ProjectPurpose, ProjectStage, Question,
    QuestionOption, TargetUsers, UserAnswers,
};
#[allow(unused_imports)]
pub use inventory::{
    DirectoryNode, DirectoryPurpose, FileInventory, InventoryScanner, KeyFile, ProjectType,
};
#[allow(unused_imports)]
pub use languages::{LanguageAnalyzer, LanguageDetector, LanguageInfo, LanguageSupport};
#[allow(unused_imports)]
pub use output::{
    AgentContext, AgentContextError, AgentContextWriter, JsonOutputError, JsonReportWriter,
    MarkdownOutputError, MarkdownReportWriter,
};
#[allow(unused_imports)]
pub use patterns::{
    AsyncPattern, ErrorHandlingPattern, ModulePattern, NamingConvention, NamingConventionInfo,
    PatternAnalysis, PatternAnalyzer,
};
#[allow(unused_imports)]
pub use prd_converter::{
    PrdConversionResult, PrdConverter, PrdConverterConfig, PrdJson, PrdUserStory,
};
#[allow(unused_imports)]
pub use prd_generator::{
    GeneratedUserStory, PrdGenerationResult, PrdGenerator, PrdGeneratorConfig, StorySource,
};
#[allow(unused_imports)]
pub use testing::{
    SourceModule, TestAnalysis, TestAnalyzer, TestFile, TestPattern, TestPatternInfo,
};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during audit operations
#[derive(Error, Debug)]
pub enum AuditError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Invalid project structure: {0}")]
    InvalidStructure(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}

/// Result type for audit operations
pub type AuditResult<T> = Result<T, AuditError>;

/// Severity level for audit findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "low"),
            Severity::Medium => write!(f, "medium"),
            Severity::High => write!(f, "high"),
            Severity::Critical => write!(f, "critical"),
        }
    }
}

/// A finding from the audit process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    /// Unique identifier (e.g., "ARCH-001", "DEP-002")
    pub id: String,
    /// Severity level
    pub severity: Severity,
    /// Category (architecture, dependencies, testing, documentation, etc.)
    pub category: String,
    /// Short title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Files affected by this finding
    pub affected_files: Vec<PathBuf>,
    /// Recommendation for addressing the finding
    pub recommendation: String,
}

/// A feature opportunity identified during audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureOpportunity {
    /// Unique identifier (e.g., "FEAT-001")
    pub id: String,
    /// Feature title
    pub title: String,
    /// Why this feature would be valuable
    pub rationale: String,
    /// Estimated complexity
    pub complexity: Complexity,
    /// Suggested user stories to implement this feature
    pub suggested_stories: Vec<SuggestedStory>,
}

/// Complexity level for feature opportunities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Complexity {
    Low,
    Medium,
    High,
}

/// A suggested user story for a feature opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedStory {
    /// Story title
    pub title: String,
    /// Story description
    pub description: String,
    /// Acceptance criteria
    pub acceptance_criteria: Vec<String>,
    /// Suggested priority (1 = highest)
    pub priority: u32,
}

/// Metadata about the audit run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetadata {
    /// Version of the audit module
    pub audit_version: String,
    /// Timestamp of the audit
    pub timestamp: String,
    /// Root path that was audited
    pub project_root: PathBuf,
    /// Git commit hash if available
    pub commit_hash: Option<String>,
    /// Git branch if available
    pub branch: Option<String>,
    /// Duration of the audit in milliseconds
    pub duration_ms: u64,
}

/// Complete audit report containing all analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    /// Audit metadata
    pub metadata: AuditMetadata,
    /// File structure analysis
    pub inventory: FileInventory,
    /// Dependency analysis
    pub dependencies: DependencyAnalysis,
    /// Detected findings
    pub findings: Vec<AuditFinding>,
    /// Identified opportunities
    pub opportunities: Vec<FeatureOpportunity>,
}

impl AuditReport {
    /// Create a new audit report with metadata
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            metadata: AuditMetadata {
                audit_version: env!("CARGO_PKG_VERSION").to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                project_root,
                commit_hash: None,
                branch: None,
                duration_ms: 0,
            },
            inventory: FileInventory::default(),
            dependencies: DependencyAnalysis::default(),
            findings: Vec::new(),
            opportunities: Vec::new(),
        }
    }

    /// Get findings by severity
    pub fn findings_by_severity(&self, severity: Severity) -> Vec<&AuditFinding> {
        self.findings
            .iter()
            .filter(|f| f.severity == severity)
            .collect()
    }

    /// Get count of findings by severity
    pub fn finding_counts(&self) -> (usize, usize, usize, usize) {
        let critical = self.findings_by_severity(Severity::Critical).len();
        let high = self.findings_by_severity(Severity::High).len();
        let medium = self.findings_by_severity(Severity::Medium).len();
        let low = self.findings_by_severity(Severity::Low).len();
        (critical, high, medium, low)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    #[test]
    fn test_audit_report_finding_counts() {
        let mut report = AuditReport::new(PathBuf::from("/test"));
        report.findings.push(AuditFinding {
            id: "TEST-001".to_string(),
            severity: Severity::Critical,
            category: "test".to_string(),
            title: "Critical issue".to_string(),
            description: "A critical issue".to_string(),
            affected_files: vec![],
            recommendation: "Fix it".to_string(),
        });
        report.findings.push(AuditFinding {
            id: "TEST-002".to_string(),
            severity: Severity::High,
            category: "test".to_string(),
            title: "High issue".to_string(),
            description: "A high issue".to_string(),
            affected_files: vec![],
            recommendation: "Fix it".to_string(),
        });

        let (critical, high, medium, low) = report.finding_counts();
        assert_eq!(critical, 1);
        assert_eq!(high, 1);
        assert_eq!(medium, 0);
        assert_eq!(low, 0);
    }
}
