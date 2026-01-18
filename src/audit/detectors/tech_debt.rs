//! Technical debt detection for identifying code quality issues.
//!
//! This module analyzes the codebase to find:
//! - TODO/FIXME/HACK comments
//! - Outdated dependencies
//! - Dead code indicators

use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::audit::dependencies::DependencyAnalysis;
use crate::audit::{AuditFinding, AuditResult, Severity};

/// Type of technical debt detected
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechDebtType {
    /// TODO comment indicating incomplete work
    TodoComment,
    /// FIXME comment indicating known bug
    FixmeComment,
    /// HACK comment indicating workaround
    HackComment,
    /// XXX comment indicating questionable code
    XxxComment,
    /// Outdated dependency
    OutdatedDependency,
    /// Deprecated code usage
    DeprecatedCode,
    /// Dead code indicator (unused imports, functions, etc.)
    DeadCode,
    /// Commented out code
    CommentedOutCode,
    /// Temporary or debug code
    TemporaryCode,
}

impl std::fmt::Display for TechDebtType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TechDebtType::TodoComment => write!(f, "todo_comment"),
            TechDebtType::FixmeComment => write!(f, "fixme_comment"),
            TechDebtType::HackComment => write!(f, "hack_comment"),
            TechDebtType::XxxComment => write!(f, "xxx_comment"),
            TechDebtType::OutdatedDependency => write!(f, "outdated_dependency"),
            TechDebtType::DeprecatedCode => write!(f, "deprecated_code"),
            TechDebtType::DeadCode => write!(f, "dead_code"),
            TechDebtType::CommentedOutCode => write!(f, "commented_out_code"),
            TechDebtType::TemporaryCode => write!(f, "temporary_code"),
        }
    }
}

/// A technical debt item found in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechDebtItem {
    /// Type of tech debt
    pub debt_type: TechDebtType,
    /// File where the debt was found
    pub file: PathBuf,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// The content or context of the debt
    pub content: String,
    /// Severity of the debt
    pub severity: Severity,
    /// Recommendation for addressing the debt
    pub recommendation: String,
}

/// Complete technical debt analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TechDebtAnalysis {
    /// List of detected tech debt items
    pub items: Vec<TechDebtItem>,
    /// Count of items by type
    pub type_counts: HashMap<String, usize>,
    /// Total number of tech debt items
    pub total_items: usize,
    /// High severity items count
    pub high_severity_count: usize,
    /// Medium severity items count
    pub medium_severity_count: usize,
    /// Low severity items count
    pub low_severity_count: usize,
    /// Observations about the technical debt
    pub observations: Vec<String>,
}

/// Detector for technical debt
pub struct TechDebtDetector {
    root: PathBuf,
}

impl TechDebtDetector {
    /// Create a new technical debt detector
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Get the root path
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Analyze the codebase for technical debt
    pub fn analyze(
        &self,
        dependency_analysis: Option<&DependencyAnalysis>,
    ) -> AuditResult<TechDebtAnalysis> {
        let mut analysis = TechDebtAnalysis::default();

        // Detect TODO/FIXME/HACK/XXX comments
        self.detect_comment_markers(&mut analysis)?;

        // Detect outdated dependencies
        self.detect_outdated_dependencies(&mut analysis, dependency_analysis)?;

        // Detect dead code indicators
        self.detect_dead_code_indicators(&mut analysis)?;

        // Detect commented out code
        self.detect_commented_out_code(&mut analysis)?;

        // Detect temporary/debug code
        self.detect_temporary_code(&mut analysis)?;

        // Calculate statistics
        analysis.total_items = analysis.items.len();
        analysis.high_severity_count = analysis
            .items
            .iter()
            .filter(|i| i.severity == Severity::High || i.severity == Severity::Critical)
            .count();
        analysis.medium_severity_count = analysis
            .items
            .iter()
            .filter(|i| i.severity == Severity::Medium)
            .count();
        analysis.low_severity_count = analysis
            .items
            .iter()
            .filter(|i| i.severity == Severity::Low)
            .count();

        // Count by type
        for item in &analysis.items {
            *analysis
                .type_counts
                .entry(item.debt_type.to_string())
                .or_insert(0) += 1;
        }

        // Generate observations
        analysis.observations = self.generate_observations(&analysis);

        Ok(analysis)
    }

    /// Convert tech debt items to AuditFindings
    pub fn to_findings(&self, analysis: &TechDebtAnalysis) -> Vec<AuditFinding> {
        let mut findings = Vec::new();
        let mut id_counter = 1;

        for item in &analysis.items {
            findings.push(AuditFinding {
                id: format!("DEBT-{:03}", id_counter),
                severity: item.severity,
                category: "tech_debt".to_string(),
                title: self.debt_type_to_title(&item.debt_type),
                description: item.content.clone(),
                affected_files: vec![item.file.clone()],
                recommendation: item.recommendation.clone(),
            });
            id_counter += 1;
        }

        findings
    }

    /// Detect TODO/FIXME/HACK/XXX comments in source files
    fn detect_comment_markers(&self, analysis: &mut TechDebtAnalysis) -> AuditResult<()> {
        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .build();

        // Patterns for comment markers
        let todo_re = Regex::new(r"(?i)\b(TODO|@todo)\b[:\s]*(.*)").unwrap();
        let fixme_re = Regex::new(r"(?i)\b(FIXME|@fixme)\b[:\s]*(.*)").unwrap();
        let hack_re = Regex::new(r"(?i)\b(HACK|@hack)\b[:\s]*(.*)").unwrap();
        let xxx_re = Regex::new(r"(?i)\bXXX\b[:\s]*(.*)").unwrap();

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Only check source files
            if !Self::is_source_file(&ext) {
                continue;
            }

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let relative_path = path.strip_prefix(&self.root).unwrap_or(path);

            for (line_num, line) in content.lines().enumerate() {
                // Check for TODO
                if let Some(caps) = todo_re.captures(line) {
                    let message = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                    analysis.items.push(TechDebtItem {
                        debt_type: TechDebtType::TodoComment,
                        file: relative_path.to_path_buf(),
                        line: Some(line_num + 1),
                        content: if message.is_empty() {
                            "TODO comment without description".to_string()
                        } else {
                            format!("TODO: {}", message)
                        },
                        severity: Severity::Low,
                        recommendation: "Address the TODO or create a tracked issue.".to_string(),
                    });
                }

                // Check for FIXME
                if let Some(caps) = fixme_re.captures(line) {
                    let message = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                    analysis.items.push(TechDebtItem {
                        debt_type: TechDebtType::FixmeComment,
                        file: relative_path.to_path_buf(),
                        line: Some(line_num + 1),
                        content: if message.is_empty() {
                            "FIXME comment without description".to_string()
                        } else {
                            format!("FIXME: {}", message)
                        },
                        severity: Severity::Medium,
                        recommendation: "Fix the issue or create a bug report.".to_string(),
                    });
                }

                // Check for HACK
                if let Some(caps) = hack_re.captures(line) {
                    let message = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                    analysis.items.push(TechDebtItem {
                        debt_type: TechDebtType::HackComment,
                        file: relative_path.to_path_buf(),
                        line: Some(line_num + 1),
                        content: if message.is_empty() {
                            "HACK comment without description".to_string()
                        } else {
                            format!("HACK: {}", message)
                        },
                        severity: Severity::Medium,
                        recommendation: "Refactor the hack into a proper solution when possible."
                            .to_string(),
                    });
                }

                // Check for XXX
                if let Some(caps) = xxx_re.captures(line) {
                    let message = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                    analysis.items.push(TechDebtItem {
                        debt_type: TechDebtType::XxxComment,
                        file: relative_path.to_path_buf(),
                        line: Some(line_num + 1),
                        content: if message.is_empty() {
                            "XXX comment indicating questionable code".to_string()
                        } else {
                            format!("XXX: {}", message)
                        },
                        severity: Severity::Low,
                        recommendation: "Review and address the concern marked by XXX.".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Detect outdated dependencies
    fn detect_outdated_dependencies(
        &self,
        analysis: &mut TechDebtAnalysis,
        dependency_analysis: Option<&DependencyAnalysis>,
    ) -> AuditResult<()> {
        let Some(deps) = dependency_analysis else {
            return Ok(());
        };

        for dep in &deps.dependencies {
            if let Some(outdated) = &dep.outdated {
                let severity = if outdated.security_advisory.is_some() {
                    Severity::Critical
                } else if outdated.is_major_bump {
                    Severity::Medium
                } else {
                    Severity::Low
                };

                let content = if let Some(advisory) = &outdated.security_advisory {
                    format!(
                        "Dependency '{}' is outdated ({} -> {}) and has security advisory: {}",
                        dep.name, dep.version, outdated.latest_version, advisory
                    )
                } else {
                    format!(
                        "Dependency '{}' is outdated: {} -> {}",
                        dep.name, dep.version, outdated.latest_version
                    )
                };

                let recommendation = if outdated.security_advisory.is_some() {
                    "Update immediately to address security vulnerability.".to_string()
                } else if outdated.is_major_bump {
                    "Plan migration to the new major version.".to_string()
                } else {
                    "Consider updating to the latest version.".to_string()
                };

                analysis.items.push(TechDebtItem {
                    debt_type: TechDebtType::OutdatedDependency,
                    file: dep.manifest_path.clone(),
                    line: None,
                    content,
                    severity,
                    recommendation,
                });
            }
        }

        Ok(())
    }

    /// Detect dead code indicators
    fn detect_dead_code_indicators(&self, analysis: &mut TechDebtAnalysis) -> AuditResult<()> {
        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .build();

        // Patterns for dead code indicators
        let allow_dead_code_re = Regex::new(r#"#\[allow\(dead_code\)\]"#).unwrap();
        let unused_import_re = Regex::new(r"(?i)#\[allow\(unused_imports?\)\]").unwrap();
        let unused_variables_re = Regex::new(r"(?i)#\[allow\(unused_variables?\)\]").unwrap();
        let unused_mut_re = Regex::new(r"(?i)#\[allow\(unused_mut\)\]").unwrap();
        let unreachable_code_re = Regex::new(r"(?i)#\[allow\(unreachable_code\)\]").unwrap();

        // Python/JS unused indicators
        let noqa_unused_re = Regex::new(r"#\s*noqa:\s*F401").unwrap(); // Flake8 unused import
        let eslint_disable_unused_re = Regex::new(r"//\s*eslint-disable.*no-unused-vars").unwrap();
        let ts_ignore_re = Regex::new(r"//\s*@ts-ignore").unwrap();
        let underscore_prefix_re = Regex::new(r"^\s*(?:let|const|var)\s+_\w+\s*=").unwrap();

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !Self::is_source_file(&ext) {
                continue;
            }

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let relative_path = path.strip_prefix(&self.root).unwrap_or(path);

            for (line_num, line) in content.lines().enumerate() {
                // Check for Rust dead code allows
                if allow_dead_code_re.is_match(line) {
                    analysis.items.push(TechDebtItem {
                        debt_type: TechDebtType::DeadCode,
                        file: relative_path.to_path_buf(),
                        line: Some(line_num + 1),
                        content: "Suppressed dead_code warning indicates potentially unused code."
                            .to_string(),
                        severity: Severity::Low,
                        recommendation: "Remove dead code or document why it needs to be kept."
                            .to_string(),
                    });
                }

                // Check for unused imports allow
                if unused_import_re.is_match(line) {
                    analysis.items.push(TechDebtItem {
                        debt_type: TechDebtType::DeadCode,
                        file: relative_path.to_path_buf(),
                        line: Some(line_num + 1),
                        content: "Suppressed unused_imports warning.".to_string(),
                        severity: Severity::Low,
                        recommendation: "Remove unused imports.".to_string(),
                    });
                }

                // Check for unused variables allow
                if unused_variables_re.is_match(line) {
                    analysis.items.push(TechDebtItem {
                        debt_type: TechDebtType::DeadCode,
                        file: relative_path.to_path_buf(),
                        line: Some(line_num + 1),
                        content: "Suppressed unused_variables warning.".to_string(),
                        severity: Severity::Low,
                        recommendation: "Remove or use the unused variables.".to_string(),
                    });
                }

                // Check for unused_mut allow
                if unused_mut_re.is_match(line) {
                    analysis.items.push(TechDebtItem {
                        debt_type: TechDebtType::DeadCode,
                        file: relative_path.to_path_buf(),
                        line: Some(line_num + 1),
                        content: "Suppressed unused_mut warning.".to_string(),
                        severity: Severity::Low,
                        recommendation: "Remove unnecessary mut keyword.".to_string(),
                    });
                }

                // Check for unreachable_code allow
                if unreachable_code_re.is_match(line) {
                    analysis.items.push(TechDebtItem {
                        debt_type: TechDebtType::DeadCode,
                        file: relative_path.to_path_buf(),
                        line: Some(line_num + 1),
                        content: "Suppressed unreachable_code warning indicates dead code paths."
                            .to_string(),
                        severity: Severity::Medium,
                        recommendation: "Remove unreachable code or fix the control flow."
                            .to_string(),
                    });
                }

                // Check for Python noqa F401
                if noqa_unused_re.is_match(line) {
                    analysis.items.push(TechDebtItem {
                        debt_type: TechDebtType::DeadCode,
                        file: relative_path.to_path_buf(),
                        line: Some(line_num + 1),
                        content: "Suppressed unused import warning (noqa: F401).".to_string(),
                        severity: Severity::Low,
                        recommendation: "Remove unused import or document why it's needed."
                            .to_string(),
                    });
                }

                // Check for eslint-disable no-unused-vars
                if eslint_disable_unused_re.is_match(line) {
                    analysis.items.push(TechDebtItem {
                        debt_type: TechDebtType::DeadCode,
                        file: relative_path.to_path_buf(),
                        line: Some(line_num + 1),
                        content: "ESLint no-unused-vars disabled.".to_string(),
                        severity: Severity::Low,
                        recommendation: "Remove unused variables.".to_string(),
                    });
                }

                // Check for @ts-ignore
                if ts_ignore_re.is_match(line) {
                    analysis.items.push(TechDebtItem {
                        debt_type: TechDebtType::DeadCode,
                        file: relative_path.to_path_buf(),
                        line: Some(line_num + 1),
                        content: "@ts-ignore directive suppresses TypeScript errors.".to_string(),
                        severity: Severity::Medium,
                        recommendation: "Fix the TypeScript error instead of ignoring it."
                            .to_string(),
                    });
                }

                // Check for underscore-prefixed variables (often unused)
                if underscore_prefix_re.is_match(line) && ext == "ts" || ext == "js" {
                    // Only flag if not destructuring
                    if !line.contains('{') && !line.contains('[') {
                        analysis.items.push(TechDebtItem {
                            debt_type: TechDebtType::DeadCode,
                            file: relative_path.to_path_buf(),
                            line: Some(line_num + 1),
                            content:
                                "Underscore-prefixed variable may indicate intentionally unused code."
                                    .to_string(),
                            severity: Severity::Low,
                            recommendation:
                                "Remove if unused or rename if actually used.".to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Detect commented out code
    fn detect_commented_out_code(&self, analysis: &mut TechDebtAnalysis) -> AuditResult<()> {
        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .build();

        // Patterns that suggest commented out code rather than documentation
        let commented_code_patterns = [
            Regex::new(r"^\s*//\s*(if|for|while|fn|let|const|var|function|class|return)\s")
                .unwrap(),
            Regex::new(r"^\s*//\s*\w+\s*\([^)]*\)\s*[{;]?\s*$").unwrap(), // function call
            Regex::new(r"^\s*//\s*\w+\s*=\s*.+;?\s*$").unwrap(),          // assignment
            Regex::new(r"^\s*#\s*(if|for|while|def|class|return)\s").unwrap(), // Python
            Regex::new(r"^\s*/\*[\s\S]*?(if|for|while|function|class)").unwrap(), // Block comment with code
        ];

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !Self::is_source_file(&ext) {
                continue;
            }

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let relative_path = path.strip_prefix(&self.root).unwrap_or(path);

            // Track consecutive commented lines
            let mut consecutive_commented = 0;
            let mut first_commented_line = 0;

            for (line_num, line) in content.lines().enumerate() {
                let is_commented_code = commented_code_patterns.iter().any(|p| p.is_match(line));

                if is_commented_code {
                    if consecutive_commented == 0 {
                        first_commented_line = line_num + 1;
                    }
                    consecutive_commented += 1;
                } else {
                    // Report if we had 3+ consecutive lines of commented code
                    if consecutive_commented >= 3 {
                        analysis.items.push(TechDebtItem {
                            debt_type: TechDebtType::CommentedOutCode,
                            file: relative_path.to_path_buf(),
                            line: Some(first_commented_line),
                            content: format!(
                                "Block of {} lines of commented out code.",
                                consecutive_commented
                            ),
                            severity: Severity::Low,
                            recommendation:
                                "Remove commented out code. Use version control to preserve history."
                                    .to_string(),
                        });
                    }
                    consecutive_commented = 0;
                }
            }

            // Check end of file
            if consecutive_commented >= 3 {
                analysis.items.push(TechDebtItem {
                    debt_type: TechDebtType::CommentedOutCode,
                    file: relative_path.to_path_buf(),
                    line: Some(first_commented_line),
                    content: format!(
                        "Block of {} lines of commented out code.",
                        consecutive_commented
                    ),
                    severity: Severity::Low,
                    recommendation:
                        "Remove commented out code. Use version control to preserve history."
                            .to_string(),
                });
            }
        }

        Ok(())
    }

    /// Detect temporary or debug code
    fn detect_temporary_code(&self, analysis: &mut TechDebtAnalysis) -> AuditResult<()> {
        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .build();

        // Patterns for temporary/debug code
        let debug_patterns = [
            Regex::new(r"(?i)\bconsole\.(log|debug|info|warn|error)\s*\(").unwrap(),
            Regex::new(r"(?i)\bprint\s*\(").unwrap(), // Python print (might be intentional)
            Regex::new(r"(?i)\bdbg!\s*\(").unwrap(),  // Rust dbg! macro
            Regex::new(r"(?i)\bprintln!\s*\(.*debug").unwrap(), // Rust println with debug
            Regex::new(r"(?i)\bdebugger\s*;?").unwrap(), // JS debugger statement
            Regex::new(r#"(?i)sleep\s*\(\s*\d+\s*\)"#).unwrap(), // Sleep calls (often temporary)
        ];

        let temp_patterns = [
            Regex::new(r"(?i)\btemp\b").unwrap(),
            Regex::new(r"(?i)\btest123\b").unwrap(),
            Regex::new(r"(?i)\bfoo\b").unwrap(),
            Regex::new(r"(?i)\bbar\b").unwrap(),
            Regex::new(r"(?i)\bbaz\b").unwrap(),
            Regex::new(r#"(?i)"TODO:?\s*remove"#).unwrap(),
        ];

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !Self::is_source_file(&ext) {
                continue;
            }

            // Skip test files for debug patterns (they often have intentional debug code)
            let path_str = path.to_string_lossy().to_lowercase();
            let is_test_file = path_str.contains("test")
                || path_str.contains("spec")
                || path_str.contains("_test.");

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let relative_path = path.strip_prefix(&self.root).unwrap_or(path);

            for (line_num, line) in content.lines().enumerate() {
                // Skip if line is in a comment
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("*")
                {
                    continue;
                }

                // Check for debug patterns (skip in test files)
                if !is_test_file {
                    for pattern in &debug_patterns {
                        if pattern.is_match(line) {
                            // Skip Python print in non-debug context
                            if line.contains("print(") && !line.contains("debug") {
                                continue;
                            }

                            analysis.items.push(TechDebtItem {
                                debt_type: TechDebtType::TemporaryCode,
                                file: relative_path.to_path_buf(),
                                line: Some(line_num + 1),
                                content: "Debug/logging statement that may be temporary."
                                    .to_string(),
                                severity: Severity::Low,
                                recommendation: "Remove debug statements or use proper logging."
                                    .to_string(),
                            });
                            break;
                        }
                    }
                }

                // Check for temp variable names (skip in tests)
                if !is_test_file {
                    for pattern in &temp_patterns {
                        if pattern.is_match(line) {
                            // Avoid false positives for common patterns
                            if line.contains("foobar")
                                || line.contains("template")
                                || line.contains("temporary")
                            {
                                continue;
                            }

                            analysis.items.push(TechDebtItem {
                                debt_type: TechDebtType::TemporaryCode,
                                file: relative_path.to_path_buf(),
                                line: Some(line_num + 1),
                                content: "Temporary/placeholder naming suggests unfinished code."
                                    .to_string(),
                                severity: Severity::Low,
                                recommendation: "Replace temporary names with meaningful ones."
                                    .to_string(),
                            });
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a file extension indicates a source file
    fn is_source_file(ext: &str) -> bool {
        matches!(
            ext,
            "rs" | "js"
                | "ts"
                | "tsx"
                | "jsx"
                | "py"
                | "go"
                | "java"
                | "kt"
                | "swift"
                | "c"
                | "cpp"
                | "h"
                | "hpp"
                | "cs"
                | "rb"
                | "php"
                | "scala"
                | "vue"
                | "svelte"
        )
    }

    /// Generate observations about the technical debt
    fn generate_observations(&self, analysis: &TechDebtAnalysis) -> Vec<String> {
        let mut observations = Vec::new();

        if analysis.total_items == 0 {
            observations.push("No significant technical debt detected.".to_string());
            return observations;
        }

        // Summary observation
        observations.push(format!(
            "Found {} technical debt item(s): {} high severity, {} medium, {} low.",
            analysis.total_items,
            analysis.high_severity_count,
            analysis.medium_severity_count,
            analysis.low_severity_count
        ));

        // Comment-based debt
        let todo_count = analysis.type_counts.get("todo_comment").unwrap_or(&0);
        let fixme_count = analysis.type_counts.get("fixme_comment").unwrap_or(&0);
        let hack_count = analysis.type_counts.get("hack_comment").unwrap_or(&0);

        if *todo_count > 0 || *fixme_count > 0 || *hack_count > 0 {
            observations.push(format!(
                "Comment markers: {} TODO(s), {} FIXME(s), {} HACK(s). Consider creating tracked issues.",
                todo_count, fixme_count, hack_count
            ));
        }

        // Outdated dependencies
        if let Some(count) = analysis.type_counts.get("outdated_dependency") {
            if *count > 0 {
                observations.push(format!(
                    "{} outdated dependency(ies). Regular updates reduce security risk.",
                    count
                ));
            }
        }

        // Dead code
        if let Some(count) = analysis.type_counts.get("dead_code") {
            if *count > 0 {
                observations.push(format!(
                    "{} dead code indicator(s). Consider cleanup to improve maintainability.",
                    count
                ));
            }
        }

        // Commented out code
        if let Some(count) = analysis.type_counts.get("commented_out_code") {
            if *count > 0 {
                observations.push(format!(
                    "{} block(s) of commented out code. Use version control instead.",
                    count
                ));
            }
        }

        observations
    }

    /// Convert debt type to human-readable title
    fn debt_type_to_title(&self, debt_type: &TechDebtType) -> String {
        match debt_type {
            TechDebtType::TodoComment => "TODO Comment".to_string(),
            TechDebtType::FixmeComment => "FIXME Comment".to_string(),
            TechDebtType::HackComment => "HACK Workaround".to_string(),
            TechDebtType::XxxComment => "XXX Comment".to_string(),
            TechDebtType::OutdatedDependency => "Outdated Dependency".to_string(),
            TechDebtType::DeprecatedCode => "Deprecated Code Usage".to_string(),
            TechDebtType::DeadCode => "Dead Code Indicator".to_string(),
            TechDebtType::CommentedOutCode => "Commented Out Code".to_string(),
            TechDebtType::TemporaryCode => "Temporary/Debug Code".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::dependencies::{Dependency, DependencyEcosystem, OutdatedInfo};
    use tempfile::TempDir;

    #[test]
    fn test_tech_debt_type_display() {
        assert_eq!(format!("{}", TechDebtType::TodoComment), "todo_comment");
        assert_eq!(format!("{}", TechDebtType::FixmeComment), "fixme_comment");
        assert_eq!(format!("{}", TechDebtType::HackComment), "hack_comment");
        assert_eq!(format!("{}", TechDebtType::XxxComment), "xxx_comment");
        assert_eq!(
            format!("{}", TechDebtType::OutdatedDependency),
            "outdated_dependency"
        );
        assert_eq!(format!("{}", TechDebtType::DeadCode), "dead_code");
        assert_eq!(
            format!("{}", TechDebtType::CommentedOutCode),
            "commented_out_code"
        );
        assert_eq!(format!("{}", TechDebtType::TemporaryCode), "temporary_code");
    }

    #[test]
    fn test_detector_new() {
        let detector = TechDebtDetector::new(PathBuf::from("/test"));
        assert_eq!(detector.root(), &PathBuf::from("/test"));
    }

    #[test]
    fn test_analyze_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        assert_eq!(analysis.total_items, 0);
        assert!(analysis
            .observations
            .iter()
            .any(|o| o.contains("No significant technical debt")));
    }

    #[test]
    fn test_detect_todo_comments() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("test.rs"),
            r#"
fn main() {
    // TODO: implement this feature
    // TODO implement that feature
    // @todo: another one
    println!("Hello");
}
"#,
        )
        .unwrap();

        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        let todo_items: Vec<_> = analysis
            .items
            .iter()
            .filter(|i| i.debt_type == TechDebtType::TodoComment)
            .collect();

        assert_eq!(todo_items.len(), 3);
        assert!(todo_items
            .iter()
            .any(|i| i.content.contains("implement this feature")));
    }

    #[test]
    fn test_detect_fixme_comments() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("test.rs"),
            r#"
fn buggy() {
    // FIXME: this causes a crash
    // FIXME crashes on edge case
    // @fixme: another bug
}
"#,
        )
        .unwrap();

        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        let fixme_items: Vec<_> = analysis
            .items
            .iter()
            .filter(|i| i.debt_type == TechDebtType::FixmeComment)
            .collect();

        assert_eq!(fixme_items.len(), 3);
        assert!(fixme_items.iter().all(|i| i.severity == Severity::Medium));
    }

    #[test]
    fn test_detect_hack_comments() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("test.rs"),
            r#"
fn workaround() {
    // HACK: workaround for upstream bug
    // @hack temporary fix
}
"#,
        )
        .unwrap();

        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        let hack_items: Vec<_> = analysis
            .items
            .iter()
            .filter(|i| i.debt_type == TechDebtType::HackComment)
            .collect();

        assert_eq!(hack_items.len(), 2);
    }

    #[test]
    fn test_detect_xxx_comments() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("test.rs"),
            r#"
fn questionable() {
    // XXX: is this correct?
    // XXX review this logic
}
"#,
        )
        .unwrap();

        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        let xxx_items: Vec<_> = analysis
            .items
            .iter()
            .filter(|i| i.debt_type == TechDebtType::XxxComment)
            .collect();

        assert_eq!(xxx_items.len(), 2);
    }

    #[test]
    fn test_detect_outdated_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());

        let deps = DependencyAnalysis {
            dependencies: vec![
                Dependency {
                    name: "serde".to_string(),
                    version: "1.0".to_string(),
                    ecosystem: DependencyEcosystem::Cargo,
                    is_dev: false,
                    manifest_path: PathBuf::from("Cargo.toml"),
                    outdated: Some(OutdatedInfo {
                        latest_version: "1.1".to_string(),
                        is_major_bump: false,
                        security_advisory: None,
                    }),
                },
                Dependency {
                    name: "tokio".to_string(),
                    version: "0.2".to_string(),
                    ecosystem: DependencyEcosystem::Cargo,
                    is_dev: false,
                    manifest_path: PathBuf::from("Cargo.toml"),
                    outdated: Some(OutdatedInfo {
                        latest_version: "1.0".to_string(),
                        is_major_bump: true,
                        security_advisory: None,
                    }),
                },
                Dependency {
                    name: "vulnerable".to_string(),
                    version: "1.0".to_string(),
                    ecosystem: DependencyEcosystem::Cargo,
                    is_dev: false,
                    manifest_path: PathBuf::from("Cargo.toml"),
                    outdated: Some(OutdatedInfo {
                        latest_version: "1.1".to_string(),
                        is_major_bump: false,
                        security_advisory: Some("CVE-2023-1234".to_string()),
                    }),
                },
            ],
            ecosystem_counts: vec![(DependencyEcosystem::Cargo, 3)],
            outdated_count: 3,
            vulnerable_count: 1,
        };

        let analysis = detector.analyze(Some(&deps)).unwrap();

        let outdated_items: Vec<_> = analysis
            .items
            .iter()
            .filter(|i| i.debt_type == TechDebtType::OutdatedDependency)
            .collect();

        assert_eq!(outdated_items.len(), 3);

        // Check severities
        let serde_item = outdated_items.iter().find(|i| i.content.contains("serde"));
        assert!(serde_item.is_some());
        assert_eq!(serde_item.unwrap().severity, Severity::Low);

        let tokio_item = outdated_items.iter().find(|i| i.content.contains("tokio"));
        assert!(tokio_item.is_some());
        assert_eq!(tokio_item.unwrap().severity, Severity::Medium);

        let vuln_item = outdated_items
            .iter()
            .find(|i| i.content.contains("vulnerable"));
        assert!(vuln_item.is_some());
        assert_eq!(vuln_item.unwrap().severity, Severity::Critical);
    }

    #[test]
    fn test_detect_dead_code_rust() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("test.rs"),
            r#"
#[allow(dead_code)]
fn unused_fn() {}

#[allow(unused_imports)]
use std::collections::HashMap;

#[allow(unused_variables)]
fn with_unused(x: i32) {}

#[allow(unreachable_code)]
fn unreachable() {
    return;
    println!("never runs");
}
"#,
        )
        .unwrap();

        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        let dead_code_items: Vec<_> = analysis
            .items
            .iter()
            .filter(|i| i.debt_type == TechDebtType::DeadCode)
            .collect();

        assert_eq!(dead_code_items.len(), 4);
    }

    #[test]
    fn test_detect_dead_code_python() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("test.py"),
            r#"
import unused  # noqa: F401
from typing import List

def function():
    pass
"#,
        )
        .unwrap();

        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        assert!(analysis
            .items
            .iter()
            .any(|i| i.debt_type == TechDebtType::DeadCode && i.content.contains("noqa: F401")));
    }

    #[test]
    fn test_detect_dead_code_typescript() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("test.ts"),
            r#"
// eslint-disable-next-line no-unused-vars
const unused = 'test';

// @ts-ignore
const bad: number = "string";
"#,
        )
        .unwrap();

        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        assert!(analysis
            .items
            .iter()
            .any(|i| i.debt_type == TechDebtType::DeadCode && i.content.contains("ESLint")));

        assert!(analysis
            .items
            .iter()
            .any(|i| i.debt_type == TechDebtType::DeadCode && i.content.contains("@ts-ignore")));
    }

    #[test]
    fn test_detect_commented_out_code() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("test.rs"),
            r#"
fn main() {
    // if condition {
    //     do_something();
    //     do_another();
    // }
    println!("active code");
}
"#,
        )
        .unwrap();

        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        assert!(analysis
            .items
            .iter()
            .any(|i| i.debt_type == TechDebtType::CommentedOutCode));
    }

    #[test]
    fn test_detect_temporary_code() {
        let temp_dir = TempDir::new().unwrap();

        // Create non-test file
        fs::write(
            temp_dir.path().join("main.js"),
            r#"
function process() {
    console.log("debug output");
    debugger;
    const temp = getValue();
    dbg!(value);
}
"#,
        )
        .unwrap();

        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        let temp_items: Vec<_> = analysis
            .items
            .iter()
            .filter(|i| i.debt_type == TechDebtType::TemporaryCode)
            .collect();

        // Should detect console.log and debugger
        assert!(temp_items.len() >= 2);
    }

    #[test]
    fn test_skip_test_files_for_debug() {
        let temp_dir = TempDir::new().unwrap();

        // Create test file
        fs::write(
            temp_dir.path().join("test_main.js"),
            r#"
function testProcess() {
    console.log("test output");
    debugger;
}
"#,
        )
        .unwrap();

        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        // Debug patterns should be skipped in test files
        let temp_items: Vec<_> = analysis
            .items
            .iter()
            .filter(|i| i.debt_type == TechDebtType::TemporaryCode)
            .collect();

        assert_eq!(temp_items.len(), 0);
    }

    #[test]
    fn test_to_findings() {
        let temp_dir = TempDir::new().unwrap();
        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());

        let analysis = TechDebtAnalysis {
            items: vec![
                TechDebtItem {
                    debt_type: TechDebtType::TodoComment,
                    file: PathBuf::from("src/main.rs"),
                    line: Some(10),
                    content: "TODO: implement feature".to_string(),
                    severity: Severity::Low,
                    recommendation: "Address the TODO".to_string(),
                },
                TechDebtItem {
                    debt_type: TechDebtType::FixmeComment,
                    file: PathBuf::from("src/lib.rs"),
                    line: Some(20),
                    content: "FIXME: bug here".to_string(),
                    severity: Severity::Medium,
                    recommendation: "Fix the bug".to_string(),
                },
            ],
            total_items: 2,
            high_severity_count: 0,
            medium_severity_count: 1,
            low_severity_count: 1,
            ..Default::default()
        };

        let findings = detector.to_findings(&analysis);

        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].id, "DEBT-001");
        assert_eq!(findings[0].category, "tech_debt");
        assert_eq!(findings[0].severity, Severity::Low);
        assert_eq!(findings[0].title, "TODO Comment");

        assert_eq!(findings[1].id, "DEBT-002");
        assert_eq!(findings[1].severity, Severity::Medium);
        assert_eq!(findings[1].title, "FIXME Comment");
    }

    #[test]
    fn test_analysis_serialization() {
        let analysis = TechDebtAnalysis {
            items: vec![TechDebtItem {
                debt_type: TechDebtType::TodoComment,
                file: PathBuf::from("test.rs"),
                line: Some(42),
                content: "TODO: test".to_string(),
                severity: Severity::Low,
                recommendation: "Fix it".to_string(),
            }],
            type_counts: [("todo_comment".to_string(), 1)].into_iter().collect(),
            total_items: 1,
            high_severity_count: 0,
            medium_severity_count: 0,
            low_severity_count: 1,
            observations: vec!["Test observation".to_string()],
        };

        let json = serde_json::to_string(&analysis).unwrap();
        let deserialized: TechDebtAnalysis = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_items, 1);
        assert_eq!(deserialized.items.len(), 1);
        assert_eq!(deserialized.items[0].debt_type, TechDebtType::TodoComment);
    }

    #[test]
    fn test_is_source_file() {
        assert!(TechDebtDetector::is_source_file("rs"));
        assert!(TechDebtDetector::is_source_file("js"));
        assert!(TechDebtDetector::is_source_file("ts"));
        assert!(TechDebtDetector::is_source_file("tsx"));
        assert!(TechDebtDetector::is_source_file("py"));
        assert!(TechDebtDetector::is_source_file("go"));
        assert!(TechDebtDetector::is_source_file("java"));

        assert!(!TechDebtDetector::is_source_file("md"));
        assert!(!TechDebtDetector::is_source_file("txt"));
        assert!(!TechDebtDetector::is_source_file("json"));
        assert!(!TechDebtDetector::is_source_file("yaml"));
    }

    #[test]
    fn test_generate_observations_no_debt() {
        let temp_dir = TempDir::new().unwrap();
        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());

        let analysis = TechDebtAnalysis::default();
        let observations = detector.generate_observations(&analysis);

        assert!(observations
            .iter()
            .any(|o| o.contains("No significant technical debt")));
    }

    #[test]
    fn test_generate_observations_with_debt() {
        let temp_dir = TempDir::new().unwrap();
        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());

        let mut type_counts = HashMap::new();
        type_counts.insert("todo_comment".to_string(), 5);
        type_counts.insert("fixme_comment".to_string(), 3);
        type_counts.insert("hack_comment".to_string(), 2);
        type_counts.insert("outdated_dependency".to_string(), 4);
        type_counts.insert("dead_code".to_string(), 6);

        let analysis = TechDebtAnalysis {
            total_items: 20,
            high_severity_count: 2,
            medium_severity_count: 8,
            low_severity_count: 10,
            type_counts,
            items: vec![],
            observations: vec![],
        };

        let observations = detector.generate_observations(&analysis);

        assert!(observations
            .iter()
            .any(|o| o.contains("20 technical debt item(s)")));
        assert!(observations
            .iter()
            .any(|o| o.contains("5 TODO(s), 3 FIXME(s), 2 HACK(s)")));
        assert!(observations
            .iter()
            .any(|o| o.contains("4 outdated dependency(ies)")));
        assert!(observations
            .iter()
            .any(|o| o.contains("6 dead code indicator(s)")));
    }

    #[test]
    fn test_debt_type_to_title() {
        let temp_dir = TempDir::new().unwrap();
        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());

        assert_eq!(
            detector.debt_type_to_title(&TechDebtType::TodoComment),
            "TODO Comment"
        );
        assert_eq!(
            detector.debt_type_to_title(&TechDebtType::FixmeComment),
            "FIXME Comment"
        );
        assert_eq!(
            detector.debt_type_to_title(&TechDebtType::HackComment),
            "HACK Workaround"
        );
        assert_eq!(
            detector.debt_type_to_title(&TechDebtType::OutdatedDependency),
            "Outdated Dependency"
        );
        assert_eq!(
            detector.debt_type_to_title(&TechDebtType::DeadCode),
            "Dead Code Indicator"
        );
    }

    #[test]
    fn test_multiple_comment_types_same_file() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("mixed.rs"),
            r#"
// TODO: add feature
// FIXME: fix this bug
// HACK: workaround for issue
// XXX: questionable logic
fn mixed_function() {}
"#,
        )
        .unwrap();

        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        assert!(analysis
            .items
            .iter()
            .any(|i| i.debt_type == TechDebtType::TodoComment));
        assert!(analysis
            .items
            .iter()
            .any(|i| i.debt_type == TechDebtType::FixmeComment));
        assert!(analysis
            .items
            .iter()
            .any(|i| i.debt_type == TechDebtType::HackComment));
        assert!(analysis
            .items
            .iter()
            .any(|i| i.debt_type == TechDebtType::XxxComment));
    }

    #[test]
    fn test_case_insensitive_detection() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("case.rs"),
            r#"
// todo: lowercase
// Todo: titlecase
// TODO: uppercase
// FIXME: uppercase
// fixme: lowercase
"#,
        )
        .unwrap();

        let detector = TechDebtDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        let todo_count = analysis
            .items
            .iter()
            .filter(|i| i.debt_type == TechDebtType::TodoComment)
            .count();
        let fixme_count = analysis
            .items
            .iter()
            .filter(|i| i.debt_type == TechDebtType::FixmeComment)
            .count();

        assert_eq!(todo_count, 3);
        assert_eq!(fixme_count, 2);
    }
}
