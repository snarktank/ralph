//! Code pattern detection and analysis.

use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::AuditResult;

/// Naming convention style
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamingConvention {
    SnakeCase,
    CamelCase,
    PascalCase,
    ScreamingSnakeCase,
    KebabCase,
    Mixed,
    #[default]
    Unknown,
}

impl std::fmt::Display for NamingConvention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NamingConvention::SnakeCase => write!(f, "snake_case"),
            NamingConvention::CamelCase => write!(f, "camelCase"),
            NamingConvention::PascalCase => write!(f, "PascalCase"),
            NamingConvention::ScreamingSnakeCase => write!(f, "SCREAMING_SNAKE_CASE"),
            NamingConvention::KebabCase => write!(f, "kebab-case"),
            NamingConvention::Mixed => write!(f, "mixed"),
            NamingConvention::Unknown => write!(f, "unknown"),
        }
    }
}

/// Module organization pattern
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModulePattern {
    /// Flat structure with all modules in one directory
    Flat,
    /// Feature-based organization (e.g., feature/module.rs)
    FeatureBased,
    /// Layer-based organization (e.g., controllers/, services/, models/)
    LayerBased,
    /// Domain-driven design structure
    DomainDriven,
    /// Mixed or unclear organization
    Mixed,
    /// Unknown pattern
    #[default]
    Unknown,
}

impl std::fmt::Display for ModulePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModulePattern::Flat => write!(f, "flat"),
            ModulePattern::FeatureBased => write!(f, "feature-based"),
            ModulePattern::LayerBased => write!(f, "layer-based"),
            ModulePattern::DomainDriven => write!(f, "domain-driven"),
            ModulePattern::Mixed => write!(f, "mixed"),
            ModulePattern::Unknown => write!(f, "unknown"),
        }
    }
}

/// Error handling pattern
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorHandlingPattern {
    /// Result/Option-based (Rust idiomatic)
    ResultBased,
    /// Exception-based (try/catch)
    ExceptionBased,
    /// Error code returns
    ErrorCodes,
    /// Custom error types with thiserror/anyhow
    CustomErrorTypes,
    /// Mixed patterns
    Mixed,
    /// Unknown pattern
    #[default]
    Unknown,
}

impl std::fmt::Display for ErrorHandlingPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorHandlingPattern::ResultBased => write!(f, "result-based"),
            ErrorHandlingPattern::ExceptionBased => write!(f, "exception-based"),
            ErrorHandlingPattern::ErrorCodes => write!(f, "error-codes"),
            ErrorHandlingPattern::CustomErrorTypes => write!(f, "custom-error-types"),
            ErrorHandlingPattern::Mixed => write!(f, "mixed"),
            ErrorHandlingPattern::Unknown => write!(f, "unknown"),
        }
    }
}

/// Async pattern detected in the codebase
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AsyncPattern {
    /// Tokio runtime
    Tokio,
    /// async-std runtime
    AsyncStd,
    /// JavaScript/TypeScript async/await
    JsAsync,
    /// Python asyncio
    PythonAsyncio,
    /// Go goroutines
    GoRoutines,
    /// No async patterns detected
    #[default]
    None,
    /// Unknown async pattern
    Unknown,
}

impl std::fmt::Display for AsyncPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AsyncPattern::Tokio => write!(f, "tokio"),
            AsyncPattern::AsyncStd => write!(f, "async-std"),
            AsyncPattern::JsAsync => write!(f, "js-async"),
            AsyncPattern::PythonAsyncio => write!(f, "python-asyncio"),
            AsyncPattern::GoRoutines => write!(f, "go-routines"),
            AsyncPattern::None => write!(f, "none"),
            AsyncPattern::Unknown => write!(f, "unknown"),
        }
    }
}

/// Detected naming convention with usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingConventionInfo {
    /// The naming convention
    pub convention: NamingConvention,
    /// Context where this convention is used (functions, variables, types, etc.)
    pub context: String,
    /// Number of occurrences
    pub count: usize,
    /// Example identifiers
    pub examples: Vec<String>,
}

/// Complete pattern analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternAnalysis {
    /// Detected naming conventions by context
    pub naming_conventions: Vec<NamingConventionInfo>,
    /// Primary naming convention for functions
    pub function_naming: NamingConvention,
    /// Primary naming convention for types/structs
    pub type_naming: NamingConvention,
    /// Primary naming convention for constants
    pub constant_naming: NamingConvention,
    /// Detected module organization pattern
    pub module_pattern: ModulePattern,
    /// Detected error handling patterns
    pub error_handling: ErrorHandlingPattern,
    /// Detected async patterns
    pub async_pattern: AsyncPattern,
    /// Whether the codebase uses async/await
    pub uses_async: bool,
    /// Additional detected patterns
    pub additional_patterns: Vec<String>,
}

/// Analyzer for detecting code patterns
pub struct PatternAnalyzer {
    root: PathBuf,
}

impl PatternAnalyzer {
    /// Create a new pattern analyzer
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Analyze patterns in the codebase
    pub fn analyze(&self) -> AuditResult<PatternAnalysis> {
        let mut analysis = PatternAnalysis::default();
        let mut function_names: Vec<String> = Vec::new();
        let mut type_names: Vec<String> = Vec::new();
        let mut constant_names: Vec<String> = Vec::new();
        let mut dir_names: Vec<String> = Vec::new();

        // Error handling indicators
        let mut has_result_pattern = false;
        let mut has_thiserror = false;
        let mut has_anyhow = false;
        let mut has_try_catch = false;

        // Async indicators
        let mut has_tokio = false;
        let mut has_async_std = false;
        let mut has_async_fn = false;
        let mut has_js_async = false;
        let mut has_python_async = false;
        let mut has_goroutines = false;

        // Walk the directory tree
        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();

            // Collect directory names for module pattern analysis
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !name.starts_with('.') {
                        dir_names.push(name.to_string());
                    }
                }
                continue;
            }

            if !path.is_file() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Only analyze source code files
            if !matches!(
                ext.as_str(),
                "rs" | "js" | "ts" | "tsx" | "py" | "go" | "java"
            ) {
                continue;
            }

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Extract identifiers based on file type
            match ext.as_str() {
                "rs" => {
                    self.extract_rust_identifiers(
                        &content,
                        &mut function_names,
                        &mut type_names,
                        &mut constant_names,
                    );
                    has_result_pattern |= content.contains("Result<");
                    has_thiserror |= content.contains("thiserror");
                    has_anyhow |= content.contains("anyhow");
                    has_async_fn |= content.contains("async fn");
                    has_tokio |= content.contains("tokio::");
                    has_async_std |= content.contains("async_std::");
                }
                "js" | "ts" | "tsx" => {
                    self.extract_js_identifiers(
                        &content,
                        &mut function_names,
                        &mut type_names,
                        &mut constant_names,
                    );
                    has_try_catch |= content.contains("try {") || content.contains("catch (");
                    has_js_async |= content.contains("async ") || content.contains("await ");
                }
                "py" => {
                    self.extract_python_identifiers(
                        &content,
                        &mut function_names,
                        &mut type_names,
                        &mut constant_names,
                    );
                    has_try_catch |= content.contains("try:") || content.contains("except ");
                    has_python_async |= content.contains("async def") || content.contains("await ");
                }
                "go" => {
                    self.extract_go_identifiers(
                        &content,
                        &mut function_names,
                        &mut type_names,
                        &mut constant_names,
                    );
                    has_goroutines |= content.contains("go ") || content.contains("chan ");
                }
                _ => {}
            }
        }

        // Analyze naming conventions
        analysis.function_naming = Self::detect_primary_convention(&function_names);
        analysis.type_naming = Self::detect_primary_convention(&type_names);
        analysis.constant_naming = Self::detect_primary_convention(&constant_names);

        // Build naming convention info
        analysis.naming_conventions =
            self.build_naming_info(&function_names, &type_names, &constant_names);

        // Detect module organization pattern
        analysis.module_pattern = Self::detect_module_pattern(&dir_names);

        // Detect error handling pattern
        analysis.error_handling = Self::detect_error_handling(
            has_result_pattern,
            has_thiserror,
            has_anyhow,
            has_try_catch,
        );

        // Detect async pattern
        analysis.async_pattern = Self::detect_async_pattern(
            has_tokio,
            has_async_std,
            has_async_fn,
            has_js_async,
            has_python_async,
            has_goroutines,
        );
        analysis.uses_async = has_async_fn || has_js_async || has_python_async || has_goroutines;

        // Detect additional patterns
        analysis.additional_patterns = self.detect_additional_patterns(&dir_names);

        Ok(analysis)
    }

    /// Get the root path
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Extract identifiers from Rust code
    fn extract_rust_identifiers(
        &self,
        content: &str,
        functions: &mut Vec<String>,
        types: &mut Vec<String>,
        constants: &mut Vec<String>,
    ) {
        // Function names: fn name(
        let fn_re =
            Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*[<(]").unwrap();
        for cap in fn_re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                functions.push(name.as_str().to_string());
            }
        }

        // Type names: struct/enum/trait Name
        let type_re =
            Regex::new(r"(?:pub\s+)?(?:struct|enum|trait)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
        for cap in type_re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                types.push(name.as_str().to_string());
            }
        }

        // Constants: const NAME: or static NAME:
        let const_re = Regex::new(r"(?:pub\s+)?(?:const|static)\s+([A-Z][A-Z0-9_]*)\s*:").unwrap();
        for cap in const_re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                constants.push(name.as_str().to_string());
            }
        }
    }

    /// Extract identifiers from JavaScript/TypeScript code
    fn extract_js_identifiers(
        &self,
        content: &str,
        functions: &mut Vec<String>,
        types: &mut Vec<String>,
        constants: &mut Vec<String>,
    ) {
        // Function names: function name( or const name = (
        let fn_re = Regex::new(r"function\s+([a-zA-Z_$][a-zA-Z0-9_$]*)\s*\(").unwrap();
        for cap in fn_re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                functions.push(name.as_str().to_string());
            }
        }

        // Arrow functions: const name = (
        let arrow_re =
            Regex::new(r"(?:const|let|var)\s+([a-zA-Z_$][a-zA-Z0-9_$]*)\s*=\s*(?:async\s*)?\(")
                .unwrap();
        for cap in arrow_re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                functions.push(name.as_str().to_string());
            }
        }

        // Type/Interface/Class names
        let type_re = Regex::new(r"(?:class|interface|type)\s+([a-zA-Z_$][a-zA-Z0-9_$]*)").unwrap();
        for cap in type_re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                types.push(name.as_str().to_string());
            }
        }

        // Constants (UPPER_CASE)
        let const_re = Regex::new(r"const\s+([A-Z][A-Z0-9_]*)\s*=").unwrap();
        for cap in const_re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                constants.push(name.as_str().to_string());
            }
        }
    }

    /// Extract identifiers from Python code
    fn extract_python_identifiers(
        &self,
        content: &str,
        functions: &mut Vec<String>,
        types: &mut Vec<String>,
        constants: &mut Vec<String>,
    ) {
        // Function names: def name(
        let fn_re = Regex::new(r"(?:async\s+)?def\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").unwrap();
        for cap in fn_re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                functions.push(name.as_str().to_string());
            }
        }

        // Class names
        let class_re = Regex::new(r"class\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
        for cap in class_re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                types.push(name.as_str().to_string());
            }
        }

        // Constants (UPPER_CASE at module level)
        let const_re = Regex::new(r"^([A-Z][A-Z0-9_]*)\s*=").unwrap();
        for line in content.lines() {
            if let Some(cap) = const_re.captures(line.trim()) {
                if let Some(name) = cap.get(1) {
                    constants.push(name.as_str().to_string());
                }
            }
        }
    }

    /// Extract identifiers from Go code
    fn extract_go_identifiers(
        &self,
        content: &str,
        functions: &mut Vec<String>,
        types: &mut Vec<String>,
        constants: &mut Vec<String>,
    ) {
        // Function names: func Name( or func (r *Type) Name(
        let fn_re = Regex::new(r"func\s+(?:\([^)]*\)\s+)?([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").unwrap();
        for cap in fn_re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                functions.push(name.as_str().to_string());
            }
        }

        // Type names: type Name struct/interface
        let type_re =
            Regex::new(r"type\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+(?:struct|interface)").unwrap();
        for cap in type_re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                types.push(name.as_str().to_string());
            }
        }

        // Constants
        let const_re = Regex::new(r"const\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=").unwrap();
        for cap in const_re.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                constants.push(name.as_str().to_string());
            }
        }
    }

    /// Detect the naming convention of an identifier
    fn detect_convention(name: &str) -> NamingConvention {
        // Skip very short names
        if name.len() < 2 {
            return NamingConvention::Unknown;
        }

        // Check for SCREAMING_SNAKE_CASE (all uppercase with underscores)
        if name
            .chars()
            .all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
            && name.contains('_')
        {
            return NamingConvention::ScreamingSnakeCase;
        }

        // Check for snake_case (lowercase with underscores)
        if name
            .chars()
            .all(|c| c.is_lowercase() || c == '_' || c.is_numeric())
            && name.contains('_')
        {
            return NamingConvention::SnakeCase;
        }

        // Check for kebab-case (lowercase with hyphens)
        if name
            .chars()
            .all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
            && name.contains('-')
        {
            return NamingConvention::KebabCase;
        }

        // Check for PascalCase (starts with uppercase, no underscores/hyphens)
        if name.chars().next().is_some_and(|c| c.is_uppercase())
            && !name.contains('_')
            && !name.contains('-')
        {
            return NamingConvention::PascalCase;
        }

        // Check for camelCase (starts with lowercase, has uppercase, no underscores)
        if name.chars().next().is_some_and(|c| c.is_lowercase())
            && name.chars().any(|c| c.is_uppercase())
            && !name.contains('_')
            && !name.contains('-')
        {
            return NamingConvention::CamelCase;
        }

        // Pure lowercase without separators
        if name.chars().all(|c| c.is_lowercase() || c.is_numeric()) {
            return NamingConvention::SnakeCase; // Treat as snake_case variant
        }

        NamingConvention::Unknown
    }

    /// Detect the primary naming convention from a list of names
    fn detect_primary_convention(names: &[String]) -> NamingConvention {
        if names.is_empty() {
            return NamingConvention::Unknown;
        }

        let mut counts: HashMap<NamingConvention, usize> = HashMap::new();
        for name in names {
            let conv = Self::detect_convention(name);
            *counts.entry(conv).or_insert(0) += 1;
        }

        // Remove Unknown from consideration
        counts.remove(&NamingConvention::Unknown);

        if counts.is_empty() {
            return NamingConvention::Unknown;
        }

        // Find the most common convention
        let (primary, primary_count) = counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(conv, count)| (conv.clone(), *count))
            .unwrap_or((NamingConvention::Unknown, 0));

        // Check if it's dominant (more than 60% of total)
        let total: usize = counts.values().sum();
        if total > 0 && (primary_count as f64 / total as f64) < 0.6 {
            return NamingConvention::Mixed;
        }

        primary
    }

    /// Build detailed naming convention info
    fn build_naming_info(
        &self,
        functions: &[String],
        types: &[String],
        constants: &[String],
    ) -> Vec<NamingConventionInfo> {
        let mut info = Vec::new();

        // Functions
        let fn_conv = Self::detect_primary_convention(functions);
        if fn_conv != NamingConvention::Unknown {
            let examples: Vec<String> = functions
                .iter()
                .filter(|n| Self::detect_convention(n) == fn_conv)
                .take(3)
                .cloned()
                .collect();
            info.push(NamingConventionInfo {
                convention: fn_conv,
                context: "functions".to_string(),
                count: functions.len(),
                examples,
            });
        }

        // Types
        let type_conv = Self::detect_primary_convention(types);
        if type_conv != NamingConvention::Unknown {
            let examples: Vec<String> = types
                .iter()
                .filter(|n| Self::detect_convention(n) == type_conv)
                .take(3)
                .cloned()
                .collect();
            info.push(NamingConventionInfo {
                convention: type_conv,
                context: "types".to_string(),
                count: types.len(),
                examples,
            });
        }

        // Constants
        let const_conv = Self::detect_primary_convention(constants);
        if const_conv != NamingConvention::Unknown {
            let examples: Vec<String> = constants
                .iter()
                .filter(|n| Self::detect_convention(n) == const_conv)
                .take(3)
                .cloned()
                .collect();
            info.push(NamingConventionInfo {
                convention: const_conv,
                context: "constants".to_string(),
                count: constants.len(),
                examples,
            });
        }

        info
    }

    /// Detect module organization pattern
    fn detect_module_pattern(dir_names: &[String]) -> ModulePattern {
        let dir_lower: Vec<String> = dir_names.iter().map(|s| s.to_lowercase()).collect();

        // Layer-based indicators
        let layer_indicators = [
            "controllers",
            "services",
            "models",
            "views",
            "repositories",
            "handlers",
            "middleware",
            "routes",
            "api",
        ];
        let layer_count = layer_indicators
            .iter()
            .filter(|i| dir_lower.contains(&i.to_string()))
            .count();

        // Domain-driven indicators
        let ddd_indicators = [
            "domain",
            "infrastructure",
            "application",
            "interfaces",
            "entities",
        ];
        let ddd_count = ddd_indicators
            .iter()
            .filter(|i| dir_lower.contains(&i.to_string()))
            .count();

        // Feature-based indicators (generic feature-like names)
        let has_features = dir_lower.contains(&"features".to_string())
            || dir_lower.contains(&"modules".to_string());

        // Check for flat structure
        let has_src = dir_lower.contains(&"src".to_string());
        let non_standard_count = dir_names
            .iter()
            .filter(|n| {
                let lower = n.to_lowercase();
                !["src", "tests", "docs", "bin", "target", "build", "dist"]
                    .contains(&lower.as_str())
            })
            .count();

        if ddd_count >= 3 {
            return ModulePattern::DomainDriven;
        }

        if layer_count >= 3 {
            return ModulePattern::LayerBased;
        }

        if has_features {
            return ModulePattern::FeatureBased;
        }

        if has_src && non_standard_count <= 2 {
            return ModulePattern::Flat;
        }

        if layer_count >= 1 && ddd_count >= 1 {
            return ModulePattern::Mixed;
        }

        ModulePattern::Unknown
    }

    /// Detect error handling pattern
    fn detect_error_handling(
        has_result: bool,
        has_thiserror: bool,
        has_anyhow: bool,
        has_try_catch: bool,
    ) -> ErrorHandlingPattern {
        if has_thiserror || has_anyhow {
            return ErrorHandlingPattern::CustomErrorTypes;
        }

        if has_result && !has_try_catch {
            return ErrorHandlingPattern::ResultBased;
        }

        if has_try_catch && !has_result {
            return ErrorHandlingPattern::ExceptionBased;
        }

        if has_result && has_try_catch {
            return ErrorHandlingPattern::Mixed;
        }

        ErrorHandlingPattern::Unknown
    }

    /// Detect async pattern
    fn detect_async_pattern(
        has_tokio: bool,
        has_async_std: bool,
        has_async_fn: bool,
        has_js_async: bool,
        has_python_async: bool,
        has_goroutines: bool,
    ) -> AsyncPattern {
        if has_tokio {
            return AsyncPattern::Tokio;
        }

        if has_async_std {
            return AsyncPattern::AsyncStd;
        }

        if has_js_async {
            return AsyncPattern::JsAsync;
        }

        if has_python_async {
            return AsyncPattern::PythonAsyncio;
        }

        if has_goroutines {
            return AsyncPattern::GoRoutines;
        }

        if has_async_fn {
            // Generic async without specific runtime detection
            return AsyncPattern::Unknown;
        }

        AsyncPattern::None
    }

    /// Detect additional patterns
    fn detect_additional_patterns(&self, dir_names: &[String]) -> Vec<String> {
        let mut patterns = Vec::new();
        let dir_lower: Vec<String> = dir_names.iter().map(|s| s.to_lowercase()).collect();

        if dir_lower.contains(&"migrations".to_string()) {
            patterns.push("database-migrations".to_string());
        }

        if dir_lower.contains(&"fixtures".to_string())
            || dir_lower.contains(&"testdata".to_string())
        {
            patterns.push("test-fixtures".to_string());
        }

        if dir_lower.contains(&"scripts".to_string()) {
            patterns.push("build-scripts".to_string());
        }

        if dir_lower.contains(&"proto".to_string()) || dir_lower.contains(&"protos".to_string()) {
            patterns.push("protobuf-definitions".to_string());
        }

        if dir_lower.contains(&"generated".to_string()) || dir_lower.contains(&"gen".to_string()) {
            patterns.push("code-generation".to_string());
        }

        if dir_lower.contains(&"examples".to_string()) {
            patterns.push("example-code".to_string());
        }

        if dir_lower.contains(&"benches".to_string())
            || dir_lower.contains(&"benchmarks".to_string())
        {
            patterns.push("benchmarks".to_string());
        }

        patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_naming_convention_default() {
        assert_eq!(NamingConvention::default(), NamingConvention::Unknown);
    }

    #[test]
    fn test_naming_convention_display() {
        assert_eq!(format!("{}", NamingConvention::SnakeCase), "snake_case");
        assert_eq!(format!("{}", NamingConvention::CamelCase), "camelCase");
        assert_eq!(format!("{}", NamingConvention::PascalCase), "PascalCase");
        assert_eq!(
            format!("{}", NamingConvention::ScreamingSnakeCase),
            "SCREAMING_SNAKE_CASE"
        );
    }

    #[test]
    fn test_module_pattern_default() {
        assert_eq!(ModulePattern::default(), ModulePattern::Unknown);
    }

    #[test]
    fn test_module_pattern_display() {
        assert_eq!(format!("{}", ModulePattern::Flat), "flat");
        assert_eq!(format!("{}", ModulePattern::LayerBased), "layer-based");
        assert_eq!(format!("{}", ModulePattern::FeatureBased), "feature-based");
    }

    #[test]
    fn test_error_handling_pattern_default() {
        assert_eq!(
            ErrorHandlingPattern::default(),
            ErrorHandlingPattern::Unknown
        );
    }

    #[test]
    fn test_async_pattern_default() {
        assert_eq!(AsyncPattern::default(), AsyncPattern::None);
    }

    #[test]
    fn test_pattern_analyzer_new() {
        let analyzer = PatternAnalyzer::new(PathBuf::from("/test"));
        assert_eq!(analyzer.root(), &PathBuf::from("/test"));
    }

    #[test]
    fn test_detect_convention_snake_case() {
        assert_eq!(
            PatternAnalyzer::detect_convention("my_function_name"),
            NamingConvention::SnakeCase
        );
        assert_eq!(
            PatternAnalyzer::detect_convention("get_user_by_id"),
            NamingConvention::SnakeCase
        );
    }

    #[test]
    fn test_detect_convention_camel_case() {
        assert_eq!(
            PatternAnalyzer::detect_convention("myFunctionName"),
            NamingConvention::CamelCase
        );
        assert_eq!(
            PatternAnalyzer::detect_convention("getUserById"),
            NamingConvention::CamelCase
        );
    }

    #[test]
    fn test_detect_convention_pascal_case() {
        assert_eq!(
            PatternAnalyzer::detect_convention("MyStructName"),
            NamingConvention::PascalCase
        );
        assert_eq!(
            PatternAnalyzer::detect_convention("UserRepository"),
            NamingConvention::PascalCase
        );
    }

    #[test]
    fn test_detect_convention_screaming_snake_case() {
        assert_eq!(
            PatternAnalyzer::detect_convention("MAX_BUFFER_SIZE"),
            NamingConvention::ScreamingSnakeCase
        );
        assert_eq!(
            PatternAnalyzer::detect_convention("DEFAULT_TIMEOUT"),
            NamingConvention::ScreamingSnakeCase
        );
    }

    #[test]
    fn test_detect_primary_convention() {
        let names = vec![
            "get_user".to_string(),
            "set_name".to_string(),
            "find_by_id".to_string(),
            "calculate_total".to_string(),
        ];
        assert_eq!(
            PatternAnalyzer::detect_primary_convention(&names),
            NamingConvention::SnakeCase
        );
    }

    #[test]
    fn test_detect_primary_convention_mixed() {
        let names = vec![
            "get_user".to_string(),
            "setName".to_string(),
            "find_by_id".to_string(),
            "calculateTotal".to_string(),
        ];
        assert_eq!(
            PatternAnalyzer::detect_primary_convention(&names),
            NamingConvention::Mixed
        );
    }

    #[test]
    fn test_detect_module_pattern_layer_based() {
        let dirs = vec![
            "controllers".to_string(),
            "services".to_string(),
            "models".to_string(),
            "middleware".to_string(),
        ];
        assert_eq!(
            PatternAnalyzer::detect_module_pattern(&dirs),
            ModulePattern::LayerBased
        );
    }

    #[test]
    fn test_detect_module_pattern_ddd() {
        let dirs = vec![
            "domain".to_string(),
            "infrastructure".to_string(),
            "application".to_string(),
            "interfaces".to_string(),
        ];
        assert_eq!(
            PatternAnalyzer::detect_module_pattern(&dirs),
            ModulePattern::DomainDriven
        );
    }

    #[test]
    fn test_detect_module_pattern_feature_based() {
        let dirs = vec![
            "features".to_string(),
            "src".to_string(),
            "tests".to_string(),
        ];
        assert_eq!(
            PatternAnalyzer::detect_module_pattern(&dirs),
            ModulePattern::FeatureBased
        );
    }

    #[test]
    fn test_detect_error_handling_result() {
        assert_eq!(
            PatternAnalyzer::detect_error_handling(true, false, false, false),
            ErrorHandlingPattern::ResultBased
        );
    }

    #[test]
    fn test_detect_error_handling_custom() {
        assert_eq!(
            PatternAnalyzer::detect_error_handling(true, true, false, false),
            ErrorHandlingPattern::CustomErrorTypes
        );
        assert_eq!(
            PatternAnalyzer::detect_error_handling(true, false, true, false),
            ErrorHandlingPattern::CustomErrorTypes
        );
    }

    #[test]
    fn test_detect_error_handling_exception() {
        assert_eq!(
            PatternAnalyzer::detect_error_handling(false, false, false, true),
            ErrorHandlingPattern::ExceptionBased
        );
    }

    #[test]
    fn test_detect_async_pattern_tokio() {
        assert_eq!(
            PatternAnalyzer::detect_async_pattern(true, false, true, false, false, false),
            AsyncPattern::Tokio
        );
    }

    #[test]
    fn test_detect_async_pattern_js() {
        assert_eq!(
            PatternAnalyzer::detect_async_pattern(false, false, false, true, false, false),
            AsyncPattern::JsAsync
        );
    }

    #[test]
    fn test_detect_async_pattern_none() {
        assert_eq!(
            PatternAnalyzer::detect_async_pattern(false, false, false, false, false, false),
            AsyncPattern::None
        );
    }

    #[test]
    fn test_analyze_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = PatternAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        assert_eq!(analysis.function_naming, NamingConvention::Unknown);
        assert_eq!(analysis.module_pattern, ModulePattern::Unknown);
        assert!(!analysis.uses_async);
    }

    #[test]
    fn test_analyze_rust_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create src directory
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create a Rust file with patterns
        fs::write(
            src_dir.join("lib.rs"),
            r#"
use thiserror::Error;

pub const MAX_SIZE: usize = 1024;
pub const DEFAULT_TIMEOUT: u64 = 30;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct UserRepository {
    connection: String,
}

pub struct DataStore {
    items: Vec<String>,
}

pub fn get_user_by_id(id: u32) -> Result<String, MyError> {
    Ok("user".to_string())
}

pub fn find_all_users() -> Result<Vec<String>, MyError> {
    Ok(vec![])
}

pub async fn fetch_data() -> Result<String, MyError> {
    Ok("data".to_string())
}
"#,
        )
        .unwrap();

        let analyzer = PatternAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        assert_eq!(analysis.function_naming, NamingConvention::SnakeCase);
        assert_eq!(analysis.type_naming, NamingConvention::PascalCase);
        assert_eq!(
            analysis.constant_naming,
            NamingConvention::ScreamingSnakeCase
        );
        assert_eq!(
            analysis.error_handling,
            ErrorHandlingPattern::CustomErrorTypes
        );
        assert!(analysis.uses_async);
    }

    #[test]
    fn test_analyze_javascript_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create src directory
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create a JavaScript file with patterns
        fs::write(
            src_dir.join("index.js"),
            r#"
const MAX_RETRIES = 3;

class UserService {
    constructor() {}
}

interface UserData {
    name: string;
}

function getUserById(id) {
    return { id };
}

const fetchAllUsers = async () => {
    try {
        const result = await fetch('/users');
        return result;
    } catch (error) {
        console.error(error);
    }
};
"#,
        )
        .unwrap();

        let analyzer = PatternAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        assert_eq!(analysis.function_naming, NamingConvention::CamelCase);
        assert_eq!(analysis.type_naming, NamingConvention::PascalCase);
        assert_eq!(
            analysis.error_handling,
            ErrorHandlingPattern::ExceptionBased
        );
        assert!(analysis.uses_async);
        assert_eq!(analysis.async_pattern, AsyncPattern::JsAsync);
    }

    #[test]
    fn test_detect_additional_patterns() {
        let temp_dir = TempDir::new().unwrap();

        // Create directories
        fs::create_dir(temp_dir.path().join("migrations")).unwrap();
        fs::create_dir(temp_dir.path().join("fixtures")).unwrap();
        fs::create_dir(temp_dir.path().join("scripts")).unwrap();
        fs::create_dir(temp_dir.path().join("examples")).unwrap();

        let analyzer = PatternAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        assert!(analysis
            .additional_patterns
            .contains(&"database-migrations".to_string()));
        assert!(analysis
            .additional_patterns
            .contains(&"test-fixtures".to_string()));
        assert!(analysis
            .additional_patterns
            .contains(&"build-scripts".to_string()));
        assert!(analysis
            .additional_patterns
            .contains(&"example-code".to_string()));
    }

    #[test]
    fn test_pattern_analysis_serialization() {
        let analysis = PatternAnalysis {
            naming_conventions: vec![NamingConventionInfo {
                convention: NamingConvention::SnakeCase,
                context: "functions".to_string(),
                count: 10,
                examples: vec!["get_user".to_string()],
            }],
            function_naming: NamingConvention::SnakeCase,
            type_naming: NamingConvention::PascalCase,
            constant_naming: NamingConvention::ScreamingSnakeCase,
            module_pattern: ModulePattern::Flat,
            error_handling: ErrorHandlingPattern::ResultBased,
            async_pattern: AsyncPattern::Tokio,
            uses_async: true,
            additional_patterns: vec!["test-fixtures".to_string()],
        };

        let json = serde_json::to_string(&analysis).unwrap();
        let deserialized: PatternAnalysis = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.function_naming, NamingConvention::SnakeCase);
        assert_eq!(deserialized.async_pattern, AsyncPattern::Tokio);
        assert!(deserialized.uses_async);
    }

    #[test]
    fn test_extract_rust_identifiers() {
        let analyzer = PatternAnalyzer::new(PathBuf::from("/test"));
        let mut functions = Vec::new();
        let mut types = Vec::new();
        let mut constants = Vec::new();

        let content = r#"
pub fn process_data() {}
async fn fetch_items<T>() {}
pub struct DataStore {}
enum Status {}
const MAX_SIZE: usize = 100;
static DEFAULT_VALUE: i32 = 0;
"#;

        analyzer.extract_rust_identifiers(content, &mut functions, &mut types, &mut constants);

        assert!(functions.contains(&"process_data".to_string()));
        assert!(functions.contains(&"fetch_items".to_string()));
        assert!(types.contains(&"DataStore".to_string()));
        assert!(types.contains(&"Status".to_string()));
        assert!(constants.contains(&"MAX_SIZE".to_string()));
        assert!(constants.contains(&"DEFAULT_VALUE".to_string()));
    }

    #[test]
    fn test_extract_python_identifiers() {
        let analyzer = PatternAnalyzer::new(PathBuf::from("/test"));
        let mut functions = Vec::new();
        let mut types = Vec::new();
        let mut constants = Vec::new();

        let content = r#"
MAX_CONNECTIONS = 100
DEFAULT_TIMEOUT = 30

class UserService:
    pass

def get_user_by_id(user_id):
    pass

async def fetch_all_users():
    pass
"#;

        analyzer.extract_python_identifiers(content, &mut functions, &mut types, &mut constants);

        assert!(functions.contains(&"get_user_by_id".to_string()));
        assert!(functions.contains(&"fetch_all_users".to_string()));
        assert!(types.contains(&"UserService".to_string()));
        assert!(constants.contains(&"MAX_CONNECTIONS".to_string()));
        assert!(constants.contains(&"DEFAULT_TIMEOUT".to_string()));
    }

    #[test]
    fn test_extract_go_identifiers() {
        let analyzer = PatternAnalyzer::new(PathBuf::from("/test"));
        let mut functions = Vec::new();
        let mut types = Vec::new();
        let mut constants = Vec::new();

        let content = r#"
const MaxRetries = 3

type UserService struct {
    db *sql.DB
}

type Config interface {
    Get(key string) string
}

func GetUserById(id int) *User {
    return nil
}

func (s *UserService) FindAll() []User {
    return nil
}
"#;

        analyzer.extract_go_identifiers(content, &mut functions, &mut types, &mut constants);

        assert!(functions.contains(&"GetUserById".to_string()));
        assert!(functions.contains(&"FindAll".to_string()));
        assert!(types.contains(&"UserService".to_string()));
        assert!(types.contains(&"Config".to_string()));
        assert!(constants.contains(&"MaxRetries".to_string()));
    }
}
