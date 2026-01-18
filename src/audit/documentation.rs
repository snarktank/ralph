//! Documentation gap analysis for codebases.

use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::api::ApiAnalysis;
use super::AuditResult;

/// Type of documentation gap
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocGapType {
    /// Missing README file
    MissingReadme,
    /// README exists but is incomplete
    IncompleteReadme,
    /// Missing doc comment on public item
    MissingDocComment,
    /// Missing API documentation
    MissingApiDoc,
    /// Missing module documentation
    MissingModuleDoc,
    /// Missing example in documentation
    MissingExample,
}

impl std::fmt::Display for DocGapType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocGapType::MissingReadme => write!(f, "missing_readme"),
            DocGapType::IncompleteReadme => write!(f, "incomplete_readme"),
            DocGapType::MissingDocComment => write!(f, "missing_doc_comment"),
            DocGapType::MissingApiDoc => write!(f, "missing_api_doc"),
            DocGapType::MissingModuleDoc => write!(f, "missing_module_doc"),
            DocGapType::MissingExample => write!(f, "missing_example"),
        }
    }
}

/// A documentation gap found in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocGap {
    /// Type of gap
    pub gap_type: DocGapType,
    /// File where the gap was found
    pub file: PathBuf,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// Name of the undocumented item (if applicable)
    pub item_name: Option<String>,
    /// Description of the gap
    pub description: String,
    /// Severity (high for public APIs, medium for internal)
    pub severity: DocSeverity,
}

/// Severity of documentation gap
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocSeverity {
    /// Low severity (nice to have)
    Low,
    /// Medium severity (should be documented)
    #[default]
    Medium,
    /// High severity (must be documented)
    High,
}

impl std::fmt::Display for DocSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocSeverity::Low => write!(f, "low"),
            DocSeverity::Medium => write!(f, "medium"),
            DocSeverity::High => write!(f, "high"),
        }
    }
}

/// README completeness analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReadmeAnalysis {
    /// Whether README exists
    pub exists: bool,
    /// Path to README (if exists)
    pub path: Option<PathBuf>,
    /// Whether README has a title/heading
    pub has_title: bool,
    /// Whether README has a description
    pub has_description: bool,
    /// Whether README has installation instructions
    pub has_installation: bool,
    /// Whether README has usage instructions
    pub has_usage: bool,
    /// Whether README has examples
    pub has_examples: bool,
    /// Whether README has license information
    pub has_license: bool,
    /// Whether README has contribution guidelines
    pub has_contributing: bool,
    /// Completeness score (0.0 - 1.0)
    pub completeness_score: f64,
    /// Missing sections
    pub missing_sections: Vec<String>,
}

/// A public item that should be documented
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndocumentedItem {
    /// Item name
    pub name: String,
    /// Item type (function, struct, enum, trait, etc.)
    pub item_type: String,
    /// File path
    pub file: PathBuf,
    /// Line number
    pub line: usize,
    /// Whether it's a public API
    pub is_public_api: bool,
}

/// Complete documentation analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentationAnalysis {
    /// README analysis
    pub readme: ReadmeAnalysis,
    /// List of documentation gaps
    pub gaps: Vec<DocGap>,
    /// List of undocumented public items
    pub undocumented_items: Vec<UndocumentedItem>,
    /// Total public items
    pub total_public_items: usize,
    /// Documented public items
    pub documented_public_items: usize,
    /// Documentation coverage percentage
    pub doc_coverage_percentage: f64,
    /// Undocumented API endpoints
    pub undocumented_endpoints: Vec<String>,
    /// Observations about documentation
    pub observations: Vec<String>,
}

/// Analyzer for documentation gaps
pub struct DocAnalyzer {
    root: PathBuf,
}

impl DocAnalyzer {
    /// Create a new documentation analyzer
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Get the root path
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Analyze documentation in the codebase
    pub fn analyze(
        &self,
        api_analysis: Option<&ApiAnalysis>,
    ) -> AuditResult<DocumentationAnalysis> {
        // Analyze README first
        let readme = self.analyze_readme()?;

        let mut analysis = DocumentationAnalysis {
            readme,
            ..Default::default()
        };

        // Add README gaps
        if !analysis.readme.exists {
            analysis.gaps.push(DocGap {
                gap_type: DocGapType::MissingReadme,
                file: self.root.join("README.md"),
                line: None,
                item_name: None,
                description: "No README file found in the project root.".to_string(),
                severity: DocSeverity::High,
            });
        } else if analysis.readme.completeness_score < 0.5 {
            analysis.gaps.push(DocGap {
                gap_type: DocGapType::IncompleteReadme,
                file: analysis.readme.path.clone().unwrap_or_default(),
                line: None,
                item_name: None,
                description: format!(
                    "README is incomplete. Missing sections: {}",
                    analysis.readme.missing_sections.join(", ")
                ),
                severity: DocSeverity::Medium,
            });
        }

        // Analyze doc comments in source files
        self.analyze_doc_comments(&mut analysis)?;

        // Analyze API documentation
        if let Some(api) = api_analysis {
            self.analyze_api_documentation(&mut analysis, api)?;
        }

        // Calculate coverage percentage
        if analysis.total_public_items > 0 {
            analysis.doc_coverage_percentage = (analysis.documented_public_items as f64
                / analysis.total_public_items as f64)
                * 100.0;
        }

        // Generate observations
        analysis.observations = self.generate_observations(&analysis);

        Ok(analysis)
    }

    /// Analyze README file
    fn analyze_readme(&self) -> AuditResult<ReadmeAnalysis> {
        let mut readme = ReadmeAnalysis::default();

        // Look for README files
        let readme_patterns = [
            "README.md",
            "README",
            "readme.md",
            "Readme.md",
            "README.txt",
        ];

        for pattern in readme_patterns {
            let path = self.root.join(pattern);
            if path.exists() {
                readme.exists = true;
                readme.path = Some(path.strip_prefix(&self.root).unwrap_or(&path).to_path_buf());

                if let Ok(content) = fs::read_to_string(&path) {
                    self.analyze_readme_content(&mut readme, &content);
                }
                break;
            }
        }

        // Calculate completeness score
        let sections = [
            readme.has_title,
            readme.has_description,
            readme.has_installation,
            readme.has_usage,
            readme.has_examples,
            readme.has_license,
            readme.has_contributing,
        ];

        let present = sections.iter().filter(|&&x| x).count();
        readme.completeness_score = present as f64 / sections.len() as f64;

        // Identify missing sections
        if !readme.has_title {
            readme.missing_sections.push("title".to_string());
        }
        if !readme.has_description {
            readme.missing_sections.push("description".to_string());
        }
        if !readme.has_installation {
            readme.missing_sections.push("installation".to_string());
        }
        if !readme.has_usage {
            readme.missing_sections.push("usage".to_string());
        }
        if !readme.has_examples {
            readme.missing_sections.push("examples".to_string());
        }
        if !readme.has_license {
            readme.missing_sections.push("license".to_string());
        }
        if !readme.has_contributing {
            readme.missing_sections.push("contributing".to_string());
        }

        Ok(readme)
    }

    /// Analyze README content for completeness
    fn analyze_readme_content(&self, readme: &mut ReadmeAnalysis, content: &str) {
        let content_lower = content.to_lowercase();
        let lines: Vec<&str> = content.lines().collect();

        // Check for title (first heading)
        let heading_re = Regex::new(r"^#\s+.+").unwrap();
        readme.has_title = lines.iter().any(|line| heading_re.is_match(line));

        // Check for description (content after title, before first section)
        readme.has_description = content.len() > 100;

        // Check for installation section
        readme.has_installation = content_lower.contains("## install")
            || content_lower.contains("## installation")
            || content_lower.contains("### install")
            || content_lower.contains("## getting started")
            || content_lower.contains("## setup");

        // Check for usage section
        readme.has_usage = content_lower.contains("## usage")
            || content_lower.contains("## how to use")
            || content_lower.contains("### usage")
            || content_lower.contains("## quick start");

        // Check for examples
        readme.has_examples = content_lower.contains("## example")
            || content_lower.contains("### example")
            || content.contains("```");

        // Check for license
        readme.has_license = content_lower.contains("## license")
            || content_lower.contains("### license")
            || content_lower.contains("licensed under");

        // Check for contributing
        readme.has_contributing = content_lower.contains("## contribut")
            || content_lower.contains("### contribut")
            || content_lower.contains("pull request");
    }

    /// Analyze doc comments in source files
    fn analyze_doc_comments(&self, analysis: &mut DocumentationAnalysis) -> AuditResult<()> {
        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

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

            // Only analyze Rust files for doc comments (per acceptance criteria)
            if ext != "rs" {
                continue;
            }

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let relative_path = path.strip_prefix(&self.root).unwrap_or(path).to_path_buf();

            // Analyze Rust doc comments
            self.analyze_rust_doc_comments(&content, &relative_path, analysis);
        }

        Ok(())
    }

    /// Analyze Rust doc comments
    fn analyze_rust_doc_comments(
        &self,
        content: &str,
        file: &Path,
        analysis: &mut DocumentationAnalysis,
    ) {
        let lines: Vec<&str> = content.lines().collect();

        // Regex for public items
        let pub_fn_re = Regex::new(r"^\s*pub\s+(?:async\s+)?fn\s+(\w+)").unwrap();
        let pub_struct_re = Regex::new(r"^\s*pub\s+struct\s+(\w+)").unwrap();
        let pub_enum_re = Regex::new(r"^\s*pub\s+enum\s+(\w+)").unwrap();
        let pub_trait_re = Regex::new(r"^\s*pub\s+trait\s+(\w+)").unwrap();
        let pub_type_re = Regex::new(r"^\s*pub\s+type\s+(\w+)").unwrap();
        let pub_const_re = Regex::new(r"^\s*pub\s+const\s+(\w+)").unwrap();
        let pub_mod_re = Regex::new(r"^\s*pub\s+mod\s+(\w+)").unwrap();

        // Check for module-level documentation
        let has_module_doc =
            content.starts_with("//!") || lines.first().is_some_and(|l| l.starts_with("//!"));

        if !has_module_doc
            && file
                .file_name()
                .is_some_and(|n| n == "mod.rs" || n == "lib.rs")
        {
            analysis.gaps.push(DocGap {
                gap_type: DocGapType::MissingModuleDoc,
                file: file.to_path_buf(),
                line: Some(1),
                item_name: None,
                description: "Module-level documentation (//!) is missing.".to_string(),
                severity: DocSeverity::Medium,
            });
        }

        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;

            // Check for public items
            let (item_name, item_type) = if let Some(cap) = pub_fn_re.captures(line) {
                (cap.get(1).map(|m| m.as_str().to_string()), "function")
            } else if let Some(cap) = pub_struct_re.captures(line) {
                (cap.get(1).map(|m| m.as_str().to_string()), "struct")
            } else if let Some(cap) = pub_enum_re.captures(line) {
                (cap.get(1).map(|m| m.as_str().to_string()), "enum")
            } else if let Some(cap) = pub_trait_re.captures(line) {
                (cap.get(1).map(|m| m.as_str().to_string()), "trait")
            } else if let Some(cap) = pub_type_re.captures(line) {
                (cap.get(1).map(|m| m.as_str().to_string()), "type")
            } else if let Some(cap) = pub_const_re.captures(line) {
                (cap.get(1).map(|m| m.as_str().to_string()), "const")
            } else if let Some(cap) = pub_mod_re.captures(line) {
                (cap.get(1).map(|m| m.as_str().to_string()), "module")
            } else {
                continue;
            };

            if let Some(name) = item_name {
                analysis.total_public_items += 1;

                // Check if the previous line(s) have doc comments
                let has_doc = self.has_doc_comment(&lines, line_idx);

                if has_doc {
                    analysis.documented_public_items += 1;
                } else {
                    analysis.undocumented_items.push(UndocumentedItem {
                        name: name.clone(),
                        item_type: item_type.to_string(),
                        file: file.to_path_buf(),
                        line: line_num,
                        is_public_api: true,
                    });

                    analysis.gaps.push(DocGap {
                        gap_type: DocGapType::MissingDocComment,
                        file: file.to_path_buf(),
                        line: Some(line_num),
                        item_name: Some(name),
                        description: format!("Public {} is missing documentation.", item_type),
                        severity: DocSeverity::Medium,
                    });
                }
            }
        }
    }

    /// Check if lines before the current line have doc comments
    fn has_doc_comment(&self, lines: &[&str], line_idx: usize) -> bool {
        if line_idx == 0 {
            return false;
        }

        // Look backwards for doc comments (/// or /** */)
        let mut idx = line_idx - 1;
        loop {
            let line = lines[idx].trim();

            if line.starts_with("///") || line.starts_with("//!") {
                return true;
            }

            if line.starts_with("*/") || line.ends_with("*/") {
                // End of block comment, check if it's a doc comment
                for i in (0..idx).rev() {
                    let check_line = lines[i].trim();
                    if check_line.starts_with("/**") {
                        return true;
                    }
                    if check_line.starts_with("/*") && !check_line.starts_with("/**") {
                        return false;
                    }
                }
            }

            // Skip attributes
            if line.starts_with("#[") || line.starts_with("#![") {
                if idx == 0 {
                    break;
                }
                idx -= 1;
                continue;
            }

            // If we hit a non-comment, non-attribute line, stop
            if !line.is_empty() && !line.starts_with("//") {
                break;
            }

            if idx == 0 {
                break;
            }
            idx -= 1;
        }

        false
    }

    /// Analyze API documentation
    fn analyze_api_documentation(
        &self,
        analysis: &mut DocumentationAnalysis,
        api: &ApiAnalysis,
    ) -> AuditResult<()> {
        // Check for undocumented endpoints
        // An endpoint is considered undocumented if:
        // 1. It has no handler name (can't verify docs)
        // 2. The handler exists but has no doc comment

        for endpoint in &api.endpoints {
            // If we can identify the handler, we already check it via rust doc comments
            // Here we just track endpoints that might be undocumented
            if endpoint.handler.is_none() {
                // Can't verify documentation for endpoints without handler names
                let path = format!("{} {}", endpoint.method, endpoint.path);
                analysis.undocumented_endpoints.push(path.clone());

                analysis.gaps.push(DocGap {
                    gap_type: DocGapType::MissingApiDoc,
                    file: endpoint.file.clone(),
                    line: endpoint.line,
                    item_name: Some(path),
                    description: "API endpoint may be missing documentation.".to_string(),
                    severity: DocSeverity::Medium,
                });
            }
        }

        Ok(())
    }

    /// Generate observations about documentation
    fn generate_observations(&self, analysis: &DocumentationAnalysis) -> Vec<String> {
        let mut observations = Vec::new();

        // README observations
        if !analysis.readme.exists {
            observations.push(
                "No README file found. Consider adding one to help new contributors.".to_string(),
            );
        } else if analysis.readme.completeness_score < 0.5 {
            observations.push(format!(
                "README is {:.0}% complete. Consider adding: {}",
                analysis.readme.completeness_score * 100.0,
                analysis.readme.missing_sections.join(", ")
            ));
        } else if analysis.readme.completeness_score >= 0.8 {
            observations
                .push("README is well-documented with most essential sections.".to_string());
        }

        // Doc coverage observations
        if analysis.total_public_items > 0 {
            if analysis.doc_coverage_percentage < 30.0 {
                observations.push(format!(
                    "Low documentation coverage: {:.1}% of public items are documented.",
                    analysis.doc_coverage_percentage
                ));
            } else if analysis.doc_coverage_percentage < 70.0 {
                observations.push(format!(
                    "Moderate documentation coverage: {:.1}% of public items are documented.",
                    analysis.doc_coverage_percentage
                ));
            } else {
                observations.push(format!(
                    "Good documentation coverage: {:.1}% of public items are documented.",
                    analysis.doc_coverage_percentage
                ));
            }
        }

        // Undocumented items observation
        let undoc_count = analysis.undocumented_items.len();
        if undoc_count > 0 {
            if undoc_count <= 5 {
                let names: Vec<_> = analysis
                    .undocumented_items
                    .iter()
                    .map(|i| i.name.as_str())
                    .collect();
                observations.push(format!("Undocumented public items: {}", names.join(", ")));
            } else {
                observations.push(format!(
                    "{} public items are missing documentation.",
                    undoc_count
                ));
            }
        }

        // API documentation observation
        if !analysis.undocumented_endpoints.is_empty() {
            observations.push(format!(
                "{} API endpoint(s) may lack documentation.",
                analysis.undocumented_endpoints.len()
            ));
        }

        // Gap summary
        let high_gaps = analysis
            .gaps
            .iter()
            .filter(|g| g.severity == DocSeverity::High)
            .count();
        let medium_gaps = analysis
            .gaps
            .iter()
            .filter(|g| g.severity == DocSeverity::Medium)
            .count();

        if high_gaps > 0 || medium_gaps > 0 {
            observations.push(format!(
                "Found {} high-severity and {} medium-severity documentation gaps.",
                high_gaps, medium_gaps
            ));
        }

        observations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_doc_gap_type_display() {
        assert_eq!(format!("{}", DocGapType::MissingReadme), "missing_readme");
        assert_eq!(
            format!("{}", DocGapType::IncompleteReadme),
            "incomplete_readme"
        );
        assert_eq!(
            format!("{}", DocGapType::MissingDocComment),
            "missing_doc_comment"
        );
        assert_eq!(format!("{}", DocGapType::MissingApiDoc), "missing_api_doc");
        assert_eq!(
            format!("{}", DocGapType::MissingModuleDoc),
            "missing_module_doc"
        );
    }

    #[test]
    fn test_doc_severity_default() {
        assert_eq!(DocSeverity::default(), DocSeverity::Medium);
    }

    #[test]
    fn test_doc_severity_display() {
        assert_eq!(format!("{}", DocSeverity::Low), "low");
        assert_eq!(format!("{}", DocSeverity::Medium), "medium");
        assert_eq!(format!("{}", DocSeverity::High), "high");
    }

    #[test]
    fn test_doc_analyzer_new() {
        let analyzer = DocAnalyzer::new(PathBuf::from("/test"));
        assert_eq!(analyzer.root(), &PathBuf::from("/test"));
    }

    #[test]
    fn test_analyze_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = DocAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze(None).unwrap();

        assert!(!analysis.readme.exists);
        assert!(analysis
            .gaps
            .iter()
            .any(|g| g.gap_type == DocGapType::MissingReadme));
    }

    #[test]
    fn test_analyze_with_readme() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("README.md"),
            r#"# My Project

A description of the project.

## Installation

```bash
cargo install myproject
```

## Usage

```bash
myproject --help
```

## Examples

Here's an example:

```rust
fn main() {}
```

## License

MIT License

## Contributing

PRs welcome!
"#,
        )
        .unwrap();

        let analyzer = DocAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze(None).unwrap();

        assert!(analysis.readme.exists);
        assert!(analysis.readme.has_title);
        assert!(analysis.readme.has_description);
        assert!(analysis.readme.has_installation);
        assert!(analysis.readme.has_usage);
        assert!(analysis.readme.has_examples);
        assert!(analysis.readme.has_license);
        assert!(analysis.readme.has_contributing);
        assert!(analysis.readme.completeness_score > 0.9);
    }

    #[test]
    fn test_analyze_incomplete_readme() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("README.md"),
            r#"# My Project

A short description.
"#,
        )
        .unwrap();

        let analyzer = DocAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze(None).unwrap();

        assert!(analysis.readme.exists);
        assert!(analysis.readme.has_title);
        assert!(!analysis.readme.has_installation);
        assert!(!analysis.readme.has_usage);
        assert!(analysis.readme.completeness_score < 0.5);
        assert!(!analysis.readme.missing_sections.is_empty());
    }

    #[test]
    fn test_analyze_rust_documented_items() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        fs::write(
            src.join("lib.rs"),
            r#"//! My library module

/// A documented function
pub fn documented_fn() {}

/// A documented struct
pub struct DocumentedStruct;

/// A documented enum
pub enum DocumentedEnum {
    A,
    B,
}

/// A documented trait
pub trait DocumentedTrait {}
"#,
        )
        .unwrap();

        let analyzer = DocAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze(None).unwrap();

        assert_eq!(analysis.total_public_items, 4);
        assert_eq!(analysis.documented_public_items, 4);
        assert_eq!(analysis.doc_coverage_percentage, 100.0);
        assert!(analysis.undocumented_items.is_empty());
    }

    #[test]
    fn test_analyze_rust_undocumented_items() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        fs::write(
            src.join("lib.rs"),
            r#"//! My library

/// Documented function
pub fn documented() {}

pub fn undocumented() {}

pub struct UndocumentedStruct;

/// Documented struct
pub struct DocumentedStruct;
"#,
        )
        .unwrap();

        let analyzer = DocAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze(None).unwrap();

        assert_eq!(analysis.total_public_items, 4);
        assert_eq!(analysis.documented_public_items, 2);
        assert_eq!(analysis.undocumented_items.len(), 2);
        assert!(analysis
            .undocumented_items
            .iter()
            .any(|i| i.name == "undocumented"));
        assert!(analysis
            .undocumented_items
            .iter()
            .any(|i| i.name == "UndocumentedStruct"));
    }

    #[test]
    fn test_analyze_missing_module_doc() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        fs::write(
            src.join("mod.rs"),
            r#"
pub fn some_function() {}
"#,
        )
        .unwrap();

        let analyzer = DocAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze(None).unwrap();

        assert!(analysis
            .gaps
            .iter()
            .any(|g| g.gap_type == DocGapType::MissingModuleDoc));
    }

    #[test]
    fn test_analyze_with_module_doc() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        fs::write(
            src.join("lib.rs"),
            r#"//! This module does something

pub fn some_function() {}
"#,
        )
        .unwrap();

        let analyzer = DocAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze(None).unwrap();

        assert!(!analysis
            .gaps
            .iter()
            .any(|g| g.gap_type == DocGapType::MissingModuleDoc));
    }

    #[test]
    fn test_analyze_with_attributes() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        fs::write(
            src.join("lib.rs"),
            r#"//! Module doc

/// Documented function with attributes
#[inline]
#[must_use]
pub fn with_attrs() -> i32 { 42 }

#[derive(Debug)]
pub struct UndocumentedWithDerive;
"#,
        )
        .unwrap();

        let analyzer = DocAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze(None).unwrap();

        assert_eq!(analysis.total_public_items, 2);
        assert_eq!(analysis.documented_public_items, 1);
        assert!(analysis
            .undocumented_items
            .iter()
            .any(|i| i.name == "UndocumentedWithDerive"));
    }

    #[test]
    fn test_analyze_pub_types() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        fs::write(
            src.join("lib.rs"),
            r#"//! Types

/// Documented type alias
pub type MyType = i32;

pub type UndocumentedType = String;

/// Documented const
pub const DOCUMENTED: i32 = 42;

pub const UNDOCUMENTED: i32 = 0;

/// Documented module
pub mod documented_mod {}

pub mod undocumented_mod {}
"#,
        )
        .unwrap();

        let analyzer = DocAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze(None).unwrap();

        assert_eq!(analysis.total_public_items, 6);
        assert_eq!(analysis.documented_public_items, 3);
    }

    #[test]
    fn test_analyze_async_functions() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        fs::write(
            src.join("lib.rs"),
            r#"//! Async lib

/// Documented async function
pub async fn documented_async() {}

pub async fn undocumented_async() {}
"#,
        )
        .unwrap();

        let analyzer = DocAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze(None).unwrap();

        assert_eq!(analysis.total_public_items, 2);
        assert_eq!(analysis.documented_public_items, 1);
        assert!(analysis
            .undocumented_items
            .iter()
            .any(|i| i.name == "undocumented_async"));
    }

    #[test]
    fn test_readme_variations() {
        // Test README (no extension)
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("README"), "# Title\n\nDescription").unwrap();

        let analyzer = DocAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze(None).unwrap();

        assert!(analysis.readme.exists);
    }

    #[test]
    fn test_readme_lowercase() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("readme.md"), "# Title\n\nDescription").unwrap();

        let analyzer = DocAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze(None).unwrap();

        assert!(analysis.readme.exists);
    }

    #[test]
    fn test_documentation_analysis_serialization() {
        let analysis = DocumentationAnalysis {
            readme: ReadmeAnalysis {
                exists: true,
                path: Some(PathBuf::from("README.md")),
                has_title: true,
                has_description: true,
                has_installation: true,
                has_usage: true,
                has_examples: true,
                has_license: true,
                has_contributing: false,
                completeness_score: 0.85,
                missing_sections: vec!["contributing".to_string()],
            },
            gaps: vec![DocGap {
                gap_type: DocGapType::MissingDocComment,
                file: PathBuf::from("src/lib.rs"),
                line: Some(10),
                item_name: Some("my_function".to_string()),
                description: "Missing documentation".to_string(),
                severity: DocSeverity::Medium,
            }],
            undocumented_items: vec![UndocumentedItem {
                name: "my_function".to_string(),
                item_type: "function".to_string(),
                file: PathBuf::from("src/lib.rs"),
                line: 10,
                is_public_api: true,
            }],
            total_public_items: 10,
            documented_public_items: 8,
            doc_coverage_percentage: 80.0,
            undocumented_endpoints: vec![],
            observations: vec!["Good documentation".to_string()],
        };

        let json = serde_json::to_string(&analysis).unwrap();
        let deserialized: DocumentationAnalysis = serde_json::from_str(&json).unwrap();

        assert!(deserialized.readme.exists);
        assert_eq!(deserialized.gaps.len(), 1);
        assert_eq!(deserialized.undocumented_items.len(), 1);
        assert_eq!(deserialized.doc_coverage_percentage, 80.0);
    }

    #[test]
    fn test_coverage_percentage_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        // Create 5 items, 3 documented
        fs::write(
            src.join("lib.rs"),
            r#"//! Module

/// Doc 1
pub fn f1() {}

/// Doc 2
pub fn f2() {}

/// Doc 3
pub fn f3() {}

pub fn f4() {}

pub fn f5() {}
"#,
        )
        .unwrap();

        let analyzer = DocAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze(None).unwrap();

        assert_eq!(analysis.total_public_items, 5);
        assert_eq!(analysis.documented_public_items, 3);
        assert!((analysis.doc_coverage_percentage - 60.0).abs() < 0.1);
    }

    #[test]
    fn test_observations_low_coverage() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        // Create 10 items, only 1 documented
        let mut content = String::from("//! Module\n\n/// Documented\npub fn f0() {}\n");
        for i in 1..10 {
            content.push_str(&format!("pub fn f{}() {{}}\n", i));
        }

        fs::write(src.join("lib.rs"), &content).unwrap();

        let analyzer = DocAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze(None).unwrap();

        assert!(analysis
            .observations
            .iter()
            .any(|o| o.contains("Low documentation coverage")));
    }

    #[test]
    fn test_observations_good_coverage() {
        let temp_dir = TempDir::new().unwrap();
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();

        // Create 10 items, 8 documented
        let mut content = String::from("//! Module\n\n");
        for i in 0..8 {
            content.push_str(&format!("/// Documented\npub fn f{}() {{}}\n", i));
        }
        content.push_str("pub fn f8() {}\npub fn f9() {}\n");

        fs::write(src.join("lib.rs"), &content).unwrap();

        let analyzer = DocAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze(None).unwrap();

        assert!(analysis
            .observations
            .iter()
            .any(|o| o.contains("Good documentation coverage")));
    }

    #[test]
    fn test_has_doc_comment_simple() {
        let analyzer = DocAnalyzer::new(PathBuf::from("/test"));

        let lines: Vec<&str> = vec!["/// This is documented", "pub fn foo() {}"];

        assert!(analyzer.has_doc_comment(&lines, 1));
    }

    #[test]
    fn test_has_doc_comment_with_attribute() {
        let analyzer = DocAnalyzer::new(PathBuf::from("/test"));

        let lines: Vec<&str> = vec!["/// This is documented", "#[inline]", "pub fn foo() {}"];

        assert!(analyzer.has_doc_comment(&lines, 2));
    }

    #[test]
    fn test_has_doc_comment_none() {
        let analyzer = DocAnalyzer::new(PathBuf::from("/test"));

        let lines: Vec<&str> = vec!["", "pub fn foo() {}"];

        assert!(!analyzer.has_doc_comment(&lines, 1));
    }

    #[test]
    fn test_undocumented_api_endpoints() {
        let temp_dir = TempDir::new().unwrap();

        let analyzer = DocAnalyzer::new(temp_dir.path().to_path_buf());

        // Create a mock API analysis with endpoints without handlers
        let api = ApiAnalysis {
            endpoints: vec![super::super::api::HttpEndpoint {
                method: super::super::api::HttpMethod::Get,
                path: "/users".to_string(),
                handler: None,
                file: PathBuf::from("src/main.rs"),
                line: Some(10),
                framework: super::super::api::ApiFramework::Axum,
            }],
            ..Default::default()
        };

        let analysis = analyzer.analyze(Some(&api)).unwrap();

        assert!(!analysis.undocumented_endpoints.is_empty());
        assert!(analysis
            .gaps
            .iter()
            .any(|g| g.gap_type == DocGapType::MissingApiDoc));
    }
}
