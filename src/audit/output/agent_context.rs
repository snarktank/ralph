//! Agent context output for audit reports.
//!
//! This module generates codebase pattern information suitable for
//! appending to progress.txt, enabling agents to follow existing
//! project conventions.

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use thiserror::Error;

use crate::audit::{
    ArchitectureAnalysis, ArchitecturePattern, AsyncPattern, ErrorHandlingPattern, ModulePattern,
    NamingConvention, PatternAnalysis,
};

/// Errors that can occur during agent context output operations.
#[derive(Error, Debug)]
pub enum AgentContextError {
    /// IO error during file operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for agent context output operations.
pub type AgentContextResult<T> = Result<T, AgentContextError>;

/// Collected context for generating agent-friendly output.
#[derive(Debug, Clone, Default)]
pub struct AgentContext {
    /// Pattern analysis results
    pub patterns: Option<PatternAnalysis>,
    /// Architecture analysis results
    pub architecture: Option<ArchitectureAnalysis>,
    /// Primary language detected
    pub primary_language: Option<String>,
    /// Build commands for the project
    pub build_commands: Vec<String>,
    /// Additional key conventions
    pub key_conventions: Vec<String>,
}

impl AgentContext {
    /// Create a new empty agent context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the pattern analysis.
    pub fn with_patterns(mut self, patterns: PatternAnalysis) -> Self {
        self.patterns = Some(patterns);
        self
    }

    /// Set the architecture analysis.
    pub fn with_architecture(mut self, architecture: ArchitectureAnalysis) -> Self {
        self.architecture = Some(architecture);
        self
    }

    /// Set the primary language.
    pub fn with_primary_language(mut self, language: impl Into<String>) -> Self {
        self.primary_language = Some(language.into());
        self
    }

    /// Add a build command.
    pub fn with_build_command(mut self, command: impl Into<String>) -> Self {
        self.build_commands.push(command.into());
        self
    }

    /// Add a key convention.
    pub fn with_key_convention(mut self, convention: impl Into<String>) -> Self {
        self.key_conventions.push(convention.into());
        self
    }
}

/// Writer for agent context output to progress.txt.
pub struct AgentContextWriter;

impl AgentContextWriter {
    /// Generate the codebase patterns section as a string.
    ///
    /// # Arguments
    ///
    /// * `context` - The agent context containing pattern and architecture info.
    ///
    /// # Returns
    ///
    /// Returns a formatted markdown string for the codebase patterns section.
    pub fn generate_patterns_section(context: &AgentContext) -> String {
        let mut output = String::new();

        output.push_str("## Codebase Patterns\n");

        // Primary language and build commands
        if let Some(ref lang) = context.primary_language {
            output.push_str(&Self::format_language_section(
                lang,
                &context.build_commands,
            ));
        }

        // Naming conventions
        if let Some(ref patterns) = context.patterns {
            output.push_str(&Self::format_naming_conventions(patterns));
        }

        // Architecture pattern
        if let Some(ref arch) = context.architecture {
            output.push_str(&Self::format_architecture_pattern(arch));
        }

        // Module organization
        if let Some(ref patterns) = context.patterns {
            output.push_str(&Self::format_module_pattern(patterns));
        }

        // Error handling pattern
        if let Some(ref patterns) = context.patterns {
            output.push_str(&Self::format_error_handling(patterns));
        }

        // Async pattern
        if let Some(ref patterns) = context.patterns {
            if patterns.uses_async {
                output.push_str(&Self::format_async_pattern(patterns));
            }
        }

        // Key conventions
        for convention in &context.key_conventions {
            output.push_str(&format!("- {}\n", convention));
        }

        output
    }

    /// Append the codebase patterns section to an existing progress.txt file.
    ///
    /// If the file already contains a "## Codebase Patterns" section, it will
    /// be replaced with the new content. Otherwise, the section is appended
    /// after the header.
    ///
    /// # Arguments
    ///
    /// * `context` - The agent context containing pattern and architecture info.
    /// * `path` - The path to the progress.txt file.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an `AgentContextError` on failure.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ralphmacchio::audit::output::AgentContextWriter;
    /// use ralphmacchio::audit::output::AgentContext;
    ///
    /// let context = AgentContext::new()
    ///     .with_primary_language("Rust")
    ///     .with_build_command("cargo check")
    ///     .with_key_convention("Use thiserror for error types");
    ///
    /// AgentContextWriter::append_to_progress(&context, "progress.txt").unwrap();
    /// ```
    pub fn append_to_progress<P: AsRef<Path>>(
        context: &AgentContext,
        path: P,
    ) -> AgentContextResult<()> {
        let path = path.as_ref();

        // Read existing content if file exists
        let existing_content = if path.exists() {
            let mut file = File::open(path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            content
        } else {
            String::new()
        };

        // Generate the new patterns section
        let patterns_section = Self::generate_patterns_section(context);

        // Determine where to insert/replace the patterns section
        let new_content = Self::merge_patterns_section(&existing_content, &patterns_section);

        // Write the updated content
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(new_content.as_bytes())?;
        writer.flush()?;

        Ok(())
    }

    /// Merge the patterns section into existing content.
    ///
    /// If the content already has a "## Codebase Patterns" section, replace it.
    /// Otherwise, insert after the header divider (---).
    fn merge_patterns_section(existing: &str, patterns_section: &str) -> String {
        const PATTERNS_HEADER: &str = "## Codebase Patterns";
        const DIVIDER: &str = "---";

        // Check if patterns section already exists
        if let Some(start_idx) = existing.find(PATTERNS_HEADER) {
            // Find the end of the patterns section (next ## or ---)
            let after_header = &existing[start_idx..];
            let end_offset = Self::find_section_end(after_header);
            let end_idx = start_idx + end_offset;

            // Replace the existing section
            let mut result = String::new();
            result.push_str(&existing[..start_idx]);
            result.push_str(patterns_section);
            result.push('\n');
            result.push_str(DIVIDER);
            result.push('\n');

            // Skip the old divider if present
            let remaining = &existing[end_idx..];
            let remaining = remaining.trim_start_matches(DIVIDER).trim_start();
            if !remaining.is_empty() {
                result.push('\n');
                result.push_str(remaining);
            }

            result
        } else {
            // Find the first --- divider and insert after it
            if let Some(divider_idx) = existing.find(DIVIDER) {
                let after_divider = divider_idx + DIVIDER.len();
                let mut result = String::new();
                result.push_str(&existing[..after_divider]);
                result.push_str("\n\n");
                result.push_str(patterns_section);
                result.push('\n');
                result.push_str(DIVIDER);

                // Append any remaining content after the first divider
                let remaining = &existing[after_divider..];
                // Skip leading whitespace and any immediate divider
                let remaining = remaining.trim_start();
                if !remaining.is_empty() {
                    result.push_str("\n\n");
                    result.push_str(remaining);
                }

                result
            } else {
                // No divider found, just append the section
                let mut result = existing.to_string();
                if !result.is_empty() && !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
                result.push_str(patterns_section);
                result
            }
        }
    }

    /// Find the end of a section (next ## header or ---).
    fn find_section_end(content: &str) -> usize {
        let mut offset = 0;
        let mut first_line = true;

        for line in content.lines() {
            // Skip the first line (the header itself)
            if first_line {
                offset += line.len() + 1; // +1 for newline
                first_line = false;
                continue;
            }

            // Check for section end markers
            if line.starts_with("## ") || line.starts_with("---") {
                return offset;
            }

            offset += line.len() + 1; // +1 for newline
        }

        offset
    }

    /// Format language section with build commands.
    fn format_language_section(language: &str, build_commands: &[String]) -> String {
        let mut output = String::new();

        if build_commands.is_empty() {
            output.push_str(&format!("- This is a {} project\n", language));
        } else {
            let commands = build_commands.join(", ");
            output.push_str(&format!(
                "- This is a {} project - use {}\n",
                language, commands
            ));
        }

        output
    }

    /// Format naming conventions section.
    fn format_naming_conventions(patterns: &PatternAnalysis) -> String {
        let mut output = String::new();
        let mut conventions = Vec::new();

        if patterns.function_naming != NamingConvention::Unknown {
            conventions.push(format!("{} for functions", patterns.function_naming));
        }

        if patterns.type_naming != NamingConvention::Unknown {
            conventions.push(format!("{} for types", patterns.type_naming));
        }

        if patterns.constant_naming != NamingConvention::Unknown {
            conventions.push(format!("{} for constants", patterns.constant_naming));
        }

        if !conventions.is_empty() {
            output.push_str(&format!(
                "- Naming conventions: {}\n",
                conventions.join(", ")
            ));
        }

        output
    }

    /// Format architecture pattern.
    fn format_architecture_pattern(arch: &ArchitectureAnalysis) -> String {
        if arch.pattern == ArchitecturePattern::Unknown {
            return String::new();
        }

        let pattern_name = match arch.pattern {
            ArchitecturePattern::Layered => "layered",
            ArchitecturePattern::Modular => "modular/component-based",
            ArchitecturePattern::Hexagonal => "hexagonal (ports and adapters)",
            ArchitecturePattern::Clean => "clean architecture",
            ArchitecturePattern::Microservices => "microservices",
            ArchitecturePattern::Monolithic => "monolithic",
            ArchitecturePattern::Mvc => "MVC (Model-View-Controller)",
            ArchitecturePattern::Mvvm => "MVVM (Model-View-ViewModel)",
            ArchitecturePattern::EventDriven => "event-driven",
            ArchitecturePattern::Mixed => "mixed architecture patterns",
            ArchitecturePattern::Unknown => return String::new(),
        };

        format!("- Architecture pattern: {}\n", pattern_name)
    }

    /// Format module organization pattern.
    fn format_module_pattern(patterns: &PatternAnalysis) -> String {
        if patterns.module_pattern == ModulePattern::Unknown {
            return String::new();
        }

        let pattern_desc = match patterns.module_pattern {
            ModulePattern::Flat => "flat structure with modules in one directory",
            ModulePattern::FeatureBased => "feature-based organization",
            ModulePattern::LayerBased => "layer-based organization (controllers, services, models)",
            ModulePattern::DomainDriven => "domain-driven design structure",
            ModulePattern::Mixed => "mixed module organization",
            ModulePattern::Unknown => return String::new(),
        };

        format!("- Module organization: {}\n", pattern_desc)
    }

    /// Format error handling pattern.
    fn format_error_handling(patterns: &PatternAnalysis) -> String {
        if patterns.error_handling == ErrorHandlingPattern::Unknown {
            return String::new();
        }

        let pattern_desc = match patterns.error_handling {
            ErrorHandlingPattern::ResultBased => "Result/Option-based error handling",
            ErrorHandlingPattern::ExceptionBased => "exception-based (try/catch)",
            ErrorHandlingPattern::ErrorCodes => "error code returns",
            ErrorHandlingPattern::CustomErrorTypes => {
                "custom error types (thiserror/anyhow pattern)"
            }
            ErrorHandlingPattern::Mixed => "mixed error handling patterns",
            ErrorHandlingPattern::Unknown => return String::new(),
        };

        format!("- Error handling: {}\n", pattern_desc)
    }

    /// Format async pattern.
    fn format_async_pattern(patterns: &PatternAnalysis) -> String {
        let pattern_desc = match patterns.async_pattern {
            AsyncPattern::Tokio => "tokio runtime",
            AsyncPattern::AsyncStd => "async-std runtime",
            AsyncPattern::JsAsync => "JavaScript async/await",
            AsyncPattern::PythonAsyncio => "Python asyncio",
            AsyncPattern::GoRoutines => "Go goroutines",
            AsyncPattern::None | AsyncPattern::Unknown => return String::new(),
        };

        format!("- Async runtime: {}\n", pattern_desc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_patterns() -> PatternAnalysis {
        PatternAnalysis {
            naming_conventions: vec![],
            function_naming: NamingConvention::SnakeCase,
            type_naming: NamingConvention::PascalCase,
            constant_naming: NamingConvention::ScreamingSnakeCase,
            module_pattern: ModulePattern::Flat,
            error_handling: ErrorHandlingPattern::CustomErrorTypes,
            async_pattern: AsyncPattern::Tokio,
            uses_async: true,
            additional_patterns: vec![],
        }
    }

    fn create_test_architecture() -> ArchitectureAnalysis {
        ArchitectureAnalysis {
            pattern: ArchitecturePattern::Modular,
            secondary_patterns: vec![],
            confidence: 0.85,
            layers: vec![],
            couplings: vec![],
            boundary_violations: vec![],
            coupling_score: 0.3,
            observations: vec![],
        }
    }

    #[test]
    fn test_agent_context_builder() {
        let context = AgentContext::new()
            .with_primary_language("Rust")
            .with_build_command("cargo check")
            .with_build_command("cargo clippy")
            .with_key_convention("Use thiserror for error types");

        assert_eq!(context.primary_language, Some("Rust".to_string()));
        assert_eq!(context.build_commands.len(), 2);
        assert_eq!(context.key_conventions.len(), 1);
    }

    #[test]
    fn test_generate_patterns_section_basic() {
        let context = AgentContext::new()
            .with_primary_language("Rust")
            .with_build_command("`cargo check` for typechecking")
            .with_build_command("`cargo clippy -- -D warnings` for linting");

        let output = AgentContextWriter::generate_patterns_section(&context);

        assert!(output.contains("## Codebase Patterns"));
        assert!(output.contains("Rust project"));
        assert!(output.contains("cargo check"));
        assert!(output.contains("cargo clippy"));
    }

    #[test]
    fn test_generate_patterns_section_with_patterns() {
        let patterns = create_test_patterns();
        let context = AgentContext::new()
            .with_primary_language("Rust")
            .with_patterns(patterns);

        let output = AgentContextWriter::generate_patterns_section(&context);

        assert!(output.contains("Naming conventions"));
        assert!(output.contains("snake_case for functions"));
        assert!(output.contains("PascalCase for types"));
        assert!(output.contains("Error handling"));
        assert!(output.contains("custom error types"));
        assert!(output.contains("tokio runtime"));
    }

    #[test]
    fn test_generate_patterns_section_with_architecture() {
        let arch = create_test_architecture();
        let context = AgentContext::new()
            .with_primary_language("Rust")
            .with_architecture(arch);

        let output = AgentContextWriter::generate_patterns_section(&context);

        assert!(output.contains("Architecture pattern"));
        assert!(output.contains("modular"));
    }

    #[test]
    fn test_generate_patterns_section_with_key_conventions() {
        let context = AgentContext::new()
            .with_key_convention("Use mod.rs with re-exports")
            .with_key_convention("Derive Serialize, Deserialize on all public types");

        let output = AgentContextWriter::generate_patterns_section(&context);

        assert!(output.contains("Use mod.rs with re-exports"));
        assert!(output.contains("Derive Serialize, Deserialize"));
    }

    #[test]
    fn test_append_to_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let progress_path = temp_dir.path().join("progress.txt");

        let context = AgentContext::new()
            .with_primary_language("Rust")
            .with_build_command("`cargo check`");

        AgentContextWriter::append_to_progress(&context, &progress_path).unwrap();

        let content = std::fs::read_to_string(&progress_path).unwrap();
        assert!(content.contains("## Codebase Patterns"));
        assert!(content.contains("Rust project"));
    }

    #[test]
    fn test_append_to_existing_file_with_header() {
        let temp_dir = TempDir::new().unwrap();
        let progress_path = temp_dir.path().join("progress.txt");

        // Create initial file with header
        std::fs::write(
            &progress_path,
            "# Ralph Progress Log\nStarted: 2024-01-15\n---\n\n## 2024-01-15 - US-001\n- Work done\n",
        )
        .unwrap();

        let context = AgentContext::new()
            .with_primary_language("Rust")
            .with_build_command("`cargo check`");

        AgentContextWriter::append_to_progress(&context, &progress_path).unwrap();

        let content = std::fs::read_to_string(&progress_path).unwrap();

        // Should contain original header
        assert!(content.contains("# Ralph Progress Log"));
        // Should contain patterns section
        assert!(content.contains("## Codebase Patterns"));
        // Should preserve story entries
        assert!(content.contains("## 2024-01-15 - US-001"));
    }

    #[test]
    fn test_replace_existing_patterns_section() {
        let temp_dir = TempDir::new().unwrap();
        let progress_path = temp_dir.path().join("progress.txt");

        // Create file with existing patterns section
        std::fs::write(
            &progress_path,
            "# Ralph Progress Log\n---\n\n## Codebase Patterns\n- Old pattern info\n\n---\n\n## US-001\n- Work\n",
        )
        .unwrap();

        let context = AgentContext::new()
            .with_primary_language("TypeScript")
            .with_build_command("`npm run build`");

        AgentContextWriter::append_to_progress(&context, &progress_path).unwrap();

        let content = std::fs::read_to_string(&progress_path).unwrap();

        // Should have new patterns
        assert!(content.contains("TypeScript project"));
        // Should NOT have old patterns
        assert!(!content.contains("Old pattern info"));
        // Should preserve story entries
        assert!(content.contains("## US-001"));
    }

    #[test]
    fn test_format_language_section_with_commands() {
        let output = AgentContextWriter::format_language_section(
            "Rust",
            &[
                "`cargo check` for typechecking".to_string(),
                "`cargo clippy` for linting".to_string(),
            ],
        );

        assert!(output.contains("Rust project"));
        assert!(output.contains("cargo check"));
        assert!(output.contains("cargo clippy"));
    }

    #[test]
    fn test_format_language_section_no_commands() {
        let output = AgentContextWriter::format_language_section("Python", &[]);

        assert!(output.contains("Python project"));
        assert!(!output.contains("use"));
    }

    #[test]
    fn test_format_naming_conventions() {
        let patterns = PatternAnalysis {
            function_naming: NamingConvention::SnakeCase,
            type_naming: NamingConvention::PascalCase,
            constant_naming: NamingConvention::ScreamingSnakeCase,
            ..Default::default()
        };

        let output = AgentContextWriter::format_naming_conventions(&patterns);

        assert!(output.contains("snake_case for functions"));
        assert!(output.contains("PascalCase for types"));
        assert!(output.contains("SCREAMING_SNAKE_CASE for constants"));
    }

    #[test]
    fn test_format_naming_conventions_unknown() {
        let patterns = PatternAnalysis::default();

        let output = AgentContextWriter::format_naming_conventions(&patterns);

        // Should be empty when all conventions are unknown
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_architecture_patterns() {
        let test_cases = vec![
            (ArchitecturePattern::Layered, "layered"),
            (ArchitecturePattern::Modular, "modular"),
            (ArchitecturePattern::Hexagonal, "hexagonal"),
            (ArchitecturePattern::Clean, "clean"),
            (ArchitecturePattern::Mvc, "MVC"),
            (ArchitecturePattern::EventDriven, "event-driven"),
        ];

        for (pattern, expected) in test_cases {
            let arch = ArchitectureAnalysis {
                pattern: pattern.clone(),
                ..Default::default()
            };
            let output = AgentContextWriter::format_architecture_pattern(&arch);
            assert!(
                output.contains(expected),
                "Expected '{}' in output for {:?}",
                expected,
                pattern
            );
        }
    }

    #[test]
    fn test_format_architecture_unknown() {
        let arch = ArchitectureAnalysis::default();
        let output = AgentContextWriter::format_architecture_pattern(&arch);
        assert!(output.is_empty());
    }

    #[test]
    fn test_format_module_patterns() {
        let patterns = PatternAnalysis {
            module_pattern: ModulePattern::LayerBased,
            ..Default::default()
        };

        let output = AgentContextWriter::format_module_pattern(&patterns);

        assert!(output.contains("layer-based"));
        assert!(output.contains("controllers, services, models"));
    }

    #[test]
    fn test_format_error_handling() {
        let patterns = PatternAnalysis {
            error_handling: ErrorHandlingPattern::ResultBased,
            ..Default::default()
        };

        let output = AgentContextWriter::format_error_handling(&patterns);

        assert!(output.contains("Result/Option-based"));
    }

    #[test]
    fn test_format_async_patterns() {
        let test_cases = vec![
            (AsyncPattern::Tokio, "tokio"),
            (AsyncPattern::AsyncStd, "async-std"),
            (AsyncPattern::JsAsync, "JavaScript"),
            (AsyncPattern::PythonAsyncio, "asyncio"),
            (AsyncPattern::GoRoutines, "goroutines"),
        ];

        for (pattern, expected) in test_cases {
            let patterns = PatternAnalysis {
                async_pattern: pattern.clone(),
                uses_async: true,
                ..Default::default()
            };
            let output = AgentContextWriter::format_async_pattern(&patterns);
            assert!(
                output.contains(expected),
                "Expected '{}' in output for {:?}",
                expected,
                pattern
            );
        }
    }

    #[test]
    fn test_format_async_none() {
        let patterns = PatternAnalysis {
            async_pattern: AsyncPattern::None,
            ..Default::default()
        };

        let output = AgentContextWriter::format_async_pattern(&patterns);
        assert!(output.is_empty());
    }

    #[test]
    fn test_find_section_end() {
        let content = "## Codebase Patterns\n- Pattern 1\n- Pattern 2\n\n---\n\n## Stories";
        let end = AgentContextWriter::find_section_end(content);

        // Should find position just before ---
        let before_end = &content[..end];
        assert!(!before_end.contains("---"));
        assert!(before_end.contains("Pattern 2"));
    }

    #[test]
    fn test_find_section_end_with_next_header() {
        let content = "## Codebase Patterns\n- Pattern 1\n\n## Next Section\n- More";
        let end = AgentContextWriter::find_section_end(content);

        let before_end = &content[..end];
        assert!(before_end.contains("Pattern 1"));
        assert!(!before_end.contains("Next Section"));
    }

    #[test]
    fn test_merge_patterns_insert_after_divider() {
        let existing = "# Header\nLine 2\n---\n\n## Story 1\n- Work";
        let patterns = "## Codebase Patterns\n- New pattern\n";

        let result = AgentContextWriter::merge_patterns_section(existing, patterns);

        // Header should come first
        assert!(result.starts_with("# Header"));
        // Patterns section should be present
        assert!(result.contains("## Codebase Patterns"));
        assert!(result.contains("New pattern"));
        // Stories should be preserved
        assert!(result.contains("## Story 1"));
    }

    #[test]
    fn test_merge_patterns_replace_existing() {
        let existing =
            "# Header\n---\n\n## Codebase Patterns\n- Old pattern\n\n---\n\n## Story 1\n- Work";
        let patterns = "## Codebase Patterns\n- New pattern\n";

        let result = AgentContextWriter::merge_patterns_section(existing, patterns);

        // Should have new patterns
        assert!(result.contains("New pattern"));
        // Should NOT have old patterns
        assert!(!result.contains("Old pattern"));
        // Stories should be preserved
        assert!(result.contains("## Story 1"));
    }

    #[test]
    fn test_full_integration() {
        let temp_dir = TempDir::new().unwrap();
        let progress_path = temp_dir.path().join("progress.txt");

        // Create realistic progress.txt
        std::fs::write(
            &progress_path,
            r#"# Ralph Progress Log
Started: Sat Jan 18 2026
Feature: Codebase Audit
---

---


## 2026-01-18 11:26 - US-001
- **What was implemented**: Create audit module structure
- **Files changed**:
  - src/audit/
- **Iterations used**: 1
---
"#,
        )
        .unwrap();

        let patterns = create_test_patterns();
        let arch = create_test_architecture();

        let context = AgentContext::new()
            .with_primary_language("Rust")
            .with_build_command("`cargo check` for typechecking")
            .with_build_command("`cargo clippy -- -D warnings` for linting")
            .with_build_command("`cargo fmt` for formatting")
            .with_patterns(patterns)
            .with_architecture(arch)
            .with_key_convention("Modules follow pattern: `mod.rs` with re-exports")
            .with_key_convention("Error handling uses `thiserror` crate");

        AgentContextWriter::append_to_progress(&context, &progress_path).unwrap();

        let content = std::fs::read_to_string(&progress_path).unwrap();

        // Verify structure
        assert!(content.contains("# Ralph Progress Log"));
        assert!(content.contains("## Codebase Patterns"));
        assert!(content.contains("Rust project"));
        assert!(content.contains("cargo check"));
        assert!(content.contains("snake_case for functions"));
        assert!(content.contains("modular"));
        assert!(content.contains("tokio runtime"));
        assert!(content.contains("mod.rs"));
        assert!(content.contains("## 2026-01-18 11:26 - US-001"));
    }
}
