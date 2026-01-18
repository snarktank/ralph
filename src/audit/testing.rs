//! Test coverage analysis for codebases.

use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use super::AuditResult;

/// Type of test pattern detected
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestPattern {
    /// Unit tests (testing individual functions/modules)
    Unit,
    /// Integration tests (testing multiple components together)
    Integration,
    /// End-to-end tests (testing full user flows)
    E2e,
    /// Property-based/fuzz tests
    Property,
    /// Benchmark tests
    Benchmark,
    /// Snapshot tests
    Snapshot,
    /// Unknown test pattern
    #[default]
    Unknown,
}

impl std::fmt::Display for TestPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestPattern::Unit => write!(f, "unit"),
            TestPattern::Integration => write!(f, "integration"),
            TestPattern::E2e => write!(f, "e2e"),
            TestPattern::Property => write!(f, "property"),
            TestPattern::Benchmark => write!(f, "benchmark"),
            TestPattern::Snapshot => write!(f, "snapshot"),
            TestPattern::Unknown => write!(f, "unknown"),
        }
    }
}

/// Information about a test file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFile {
    /// Path to the test file
    pub path: PathBuf,
    /// Number of test functions in this file
    pub test_count: usize,
    /// Detected test pattern(s) for this file
    pub patterns: Vec<TestPattern>,
    /// Module or source file this test appears to cover
    pub covers_module: Option<String>,
}

/// A source module that may or may not have tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceModule {
    /// Module name
    pub name: String,
    /// Path to the module
    pub path: PathBuf,
    /// Whether this module has corresponding tests
    pub has_tests: bool,
    /// Paths to related test files
    pub test_files: Vec<PathBuf>,
}

/// Information about detected test patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPatternInfo {
    /// The test pattern
    pub pattern: TestPattern,
    /// Number of tests using this pattern
    pub count: usize,
    /// Example test files using this pattern
    pub examples: Vec<PathBuf>,
}

/// Complete test coverage analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestAnalysis {
    /// Total number of test files
    pub test_file_count: usize,
    /// Total number of test functions
    pub test_function_count: usize,
    /// List of test files with details
    pub test_files: Vec<TestFile>,
    /// Source modules and their test coverage status
    pub source_modules: Vec<SourceModule>,
    /// Modules that appear to lack tests
    pub untested_modules: Vec<SourceModule>,
    /// Detected test patterns with statistics
    pub test_patterns: Vec<TestPatternInfo>,
    /// Estimated test coverage percentage (by module count)
    pub coverage_percentage: f64,
    /// Test observations and recommendations
    pub observations: Vec<String>,
}

/// Analyzer for detecting test coverage
pub struct TestAnalyzer {
    root: PathBuf,
}

impl TestAnalyzer {
    /// Create a new test analyzer
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Get the root path
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Analyze test coverage in the codebase
    pub fn analyze(&self) -> AuditResult<TestAnalysis> {
        let mut analysis = TestAnalysis::default();

        // Collect test files
        let test_files = self.collect_test_files()?;
        analysis.test_files = test_files;
        analysis.test_file_count = analysis.test_files.len();
        analysis.test_function_count = analysis.test_files.iter().map(|f| f.test_count).sum();

        // Collect source modules
        let source_modules = self.collect_source_modules()?;

        // Match tests to modules
        let (modules_with_tests, untested_modules) =
            self.match_tests_to_modules(source_modules, &analysis.test_files);

        analysis.source_modules = modules_with_tests.clone();
        analysis.source_modules.extend(untested_modules.clone());
        analysis.untested_modules = untested_modules;

        // Calculate coverage percentage
        let total_modules = analysis.source_modules.len();
        let tested_modules = modules_with_tests.len();
        analysis.coverage_percentage = if total_modules > 0 {
            (tested_modules as f64 / total_modules as f64) * 100.0
        } else {
            0.0
        };

        // Analyze test patterns
        analysis.test_patterns = self.analyze_test_patterns(&analysis.test_files);

        // Generate observations
        analysis.observations = self.generate_observations(&analysis);

        Ok(analysis)
    }

    /// Collect all test files in the codebase
    fn collect_test_files(&self) -> AuditResult<Vec<TestFile>> {
        let mut test_files = Vec::new();

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

            // Only analyze source code files
            if !matches!(
                ext.as_str(),
                "rs" | "js" | "ts" | "tsx" | "jsx" | "py" | "go" | "java"
            ) {
                continue;
            }

            // Check if this is a test file
            if self.is_test_file(path, &ext) {
                let content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let test_count = self.count_test_functions(&content, &ext);
                let patterns = self.detect_test_patterns_in_file(&content, path, &ext);
                let covers_module = self.infer_covered_module(path);

                let relative_path = path.strip_prefix(&self.root).unwrap_or(path).to_path_buf();

                test_files.push(TestFile {
                    path: relative_path,
                    test_count,
                    patterns,
                    covers_module,
                });
            }
        }

        Ok(test_files)
    }

    /// Check if a file is a test file
    fn is_test_file(&self, path: &std::path::Path, ext: &str) -> bool {
        let file_name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        let path_str = path.to_string_lossy().to_lowercase();

        // Check common test directory patterns
        let in_test_dir = path_str.contains("/tests/")
            || path_str.contains("/test/")
            || path_str.contains("/__tests__/")
            || path_str.contains("/spec/")
            || path_str.contains("/specs/")
            || path_str.contains("/e2e/")
            || path_str.contains("/integration/");

        // Check file naming patterns
        let has_test_name = file_name.starts_with("test_")
            || file_name.ends_with("_test")
            || file_name.ends_with(".test")
            || file_name.ends_with(".spec")
            || file_name.ends_with("_spec")
            || file_name.ends_with("_tests")
            || file_name == "tests"
            || file_name == "test";

        // Language-specific patterns
        match ext {
            "rs" => {
                // Rust: check for #[cfg(test)] or #[test] in file
                if in_test_dir || has_test_name {
                    return true;
                }
                // Also check file content for inline tests
                if let Ok(content) = fs::read_to_string(path) {
                    return content.contains("#[cfg(test)]") || content.contains("#[test]");
                }
            }
            "go" => {
                // Go: *_test.go files
                return file_name.ends_with("_test");
            }
            "py" => {
                // Python: test_*.py or *_test.py or in tests/ directory
                return in_test_dir
                    || file_name.starts_with("test_")
                    || file_name.ends_with("_test")
                    || file_name == "tests"
                    || file_name == "conftest";
            }
            "js" | "ts" | "tsx" | "jsx" => {
                // JS/TS: *.test.js, *.spec.js, or in __tests__/
                return in_test_dir || has_test_name;
            }
            "java" => {
                // Java: *Test.java, *Tests.java, or in test/ directory
                return in_test_dir || file_name.ends_with("test") || file_name.ends_with("tests");
            }
            _ => {}
        }

        in_test_dir || has_test_name
    }

    /// Count test functions in a file
    fn count_test_functions(&self, content: &str, ext: &str) -> usize {
        match ext {
            "rs" => {
                // Rust: #[test] or #[tokio::test] or #[async_std::test]
                let test_attr_re =
                    Regex::new(r"#\[(?:tokio::)?(?:async_std::)?test(?:\s*\([^)]*\))?\]").unwrap();
                test_attr_re.find_iter(content).count()
            }
            "go" => {
                // Go: func Test*(t *testing.T)
                let test_fn_re = Regex::new(r"func\s+Test[A-Z][a-zA-Z0-9_]*\s*\(").unwrap();
                test_fn_re.find_iter(content).count()
            }
            "py" => {
                // Python: def test_* or unittest methods
                let test_fn_re = Regex::new(r"def\s+test_[a-zA-Z0-9_]*\s*\(").unwrap();
                let unittest_re = Regex::new(r"def\s+test[A-Z][a-zA-Z0-9_]*\s*\(self").unwrap();
                test_fn_re.find_iter(content).count() + unittest_re.find_iter(content).count()
            }
            "js" | "ts" | "tsx" | "jsx" => {
                // JS/TS: it(, test(, describe(
                let it_re = Regex::new(r#"\bit\s*\(\s*['"]"#).unwrap();
                let test_re = Regex::new(r#"\btest\s*\(\s*['"]"#).unwrap();
                it_re.find_iter(content).count() + test_re.find_iter(content).count()
            }
            "java" => {
                // Java: @Test annotation
                let test_re = Regex::new(r"@Test\b").unwrap();
                test_re.find_iter(content).count()
            }
            _ => 0,
        }
    }

    /// Detect test patterns in a file
    fn detect_test_patterns_in_file(
        &self,
        content: &str,
        path: &std::path::Path,
        ext: &str,
    ) -> Vec<TestPattern> {
        let mut patterns = HashSet::new();
        let path_str = path.to_string_lossy().to_lowercase();
        let content_lower = content.to_lowercase();

        // E2E patterns
        if path_str.contains("/e2e/")
            || path_str.contains("e2e")
            || path_str.contains("end-to-end")
            || path_str.contains("end_to_end")
            || content_lower.contains("cypress")
            || content_lower.contains("playwright")
            || content_lower.contains("selenium")
            || content_lower.contains("puppeteer")
            || content_lower.contains("webdriver")
        {
            patterns.insert(TestPattern::E2e);
        }

        // Integration patterns
        if path_str.contains("/integration/")
            || path_str.contains("integration")
            || content_lower.contains("integration")
            || content_lower.contains("testcontainers")
            || content_lower.contains("docker")
            || (ext == "rs" && content_lower.contains("spawn"))
        {
            patterns.insert(TestPattern::Integration);
        }

        // Benchmark patterns
        if path_str.contains("/bench/")
            || path_str.contains("benchmark")
            || content_lower.contains("#[bench]")
            || content_lower.contains("criterion")
            || content_lower.contains("func benchmark")
            || content_lower.contains("@benchmark")
        {
            patterns.insert(TestPattern::Benchmark);
        }

        // Property/fuzz testing patterns
        if content_lower.contains("proptest")
            || content_lower.contains("quickcheck")
            || content_lower.contains("hypothesis")
            || content_lower.contains("fuzz")
            || content_lower.contains("property-based")
            || content_lower.contains("arbitary")
        {
            patterns.insert(TestPattern::Property);
        }

        // Snapshot patterns
        if content_lower.contains("snapshot")
            || content_lower.contains("insta::")
            || content_lower.contains("tomatch")
            || content_lower.contains("expect_file")
        {
            patterns.insert(TestPattern::Snapshot);
        }

        // Default to unit test if no other pattern detected and it's a test file
        if patterns.is_empty() {
            patterns.insert(TestPattern::Unit);
        }

        patterns.into_iter().collect()
    }

    /// Infer which module a test file covers
    fn infer_covered_module(&self, path: &std::path::Path) -> Option<String> {
        let file_name = path.file_stem().and_then(|n| n.to_str())?;
        let file_name_lower = file_name.to_lowercase();

        // Remove test prefixes/suffixes
        let module_name = file_name_lower
            .strip_prefix("test_")
            .or_else(|| file_name_lower.strip_suffix("_test"))
            .or_else(|| file_name_lower.strip_suffix("_tests"))
            .or_else(|| file_name_lower.strip_suffix(".test"))
            .or_else(|| file_name_lower.strip_suffix(".spec"))
            .or_else(|| file_name_lower.strip_suffix("_spec"))
            .unwrap_or(&file_name_lower);

        if module_name.is_empty() || module_name == "test" || module_name == "tests" {
            return None;
        }

        Some(module_name.to_string())
    }

    /// Collect source modules (non-test code files)
    fn collect_source_modules(&self) -> AuditResult<Vec<SourceModule>> {
        let mut modules = Vec::new();

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

            // Only analyze source code files
            if !matches!(
                ext.as_str(),
                "rs" | "js" | "ts" | "tsx" | "jsx" | "py" | "go" | "java"
            ) {
                continue;
            }

            // Skip test files
            if self.is_test_file(path, &ext) {
                continue;
            }

            // Skip common non-code files
            let file_name = path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            if file_name == "mod"
                || file_name == "lib"
                || file_name == "main"
                || file_name == "index"
            {
                // Include but note these are entry points
            }

            let relative_path = path.strip_prefix(&self.root).unwrap_or(path).to_path_buf();

            modules.push(SourceModule {
                name: file_name,
                path: relative_path,
                has_tests: false,
                test_files: Vec::new(),
            });
        }

        Ok(modules)
    }

    /// Match test files to source modules
    fn match_tests_to_modules(
        &self,
        mut modules: Vec<SourceModule>,
        test_files: &[TestFile],
    ) -> (Vec<SourceModule>, Vec<SourceModule>) {
        // Build a map of module names to indices
        let module_names: HashMap<String, Vec<usize>> = modules
            .iter()
            .enumerate()
            .map(|(i, m)| (m.name.clone(), i))
            .fold(HashMap::new(), |mut acc, (name, idx)| {
                acc.entry(name).or_default().push(idx);
                acc
            });

        // Match tests to modules
        for test_file in test_files {
            if let Some(covered_module) = &test_file.covers_module {
                if let Some(indices) = module_names.get(covered_module) {
                    for &idx in indices {
                        modules[idx].has_tests = true;
                        modules[idx].test_files.push(test_file.path.clone());
                    }
                }
            }
        }

        // Also check for inline tests (Rust #[cfg(test)])
        for module in &mut modules {
            if !module.has_tests {
                let full_path = self.root.join(&module.path);
                if let Ok(content) = fs::read_to_string(&full_path) {
                    if content.contains("#[cfg(test)]") && content.contains("#[test]") {
                        module.has_tests = true;
                        module.test_files.push(module.path.clone());
                    }
                }
            }
        }

        // Separate tested and untested modules
        let (tested, untested): (Vec<_>, Vec<_>) = modules.into_iter().partition(|m| m.has_tests);

        (tested, untested)
    }

    /// Analyze test patterns across all test files
    fn analyze_test_patterns(&self, test_files: &[TestFile]) -> Vec<TestPatternInfo> {
        let mut pattern_counts: HashMap<TestPattern, (usize, Vec<PathBuf>)> = HashMap::new();

        for test_file in test_files {
            for pattern in &test_file.patterns {
                let entry = pattern_counts.entry(pattern.clone()).or_default();
                entry.0 += test_file.test_count.max(1);
                if entry.1.len() < 3 {
                    entry.1.push(test_file.path.clone());
                }
            }
        }

        let mut patterns: Vec<TestPatternInfo> = pattern_counts
            .into_iter()
            .map(|(pattern, (count, examples))| TestPatternInfo {
                pattern,
                count,
                examples,
            })
            .collect();

        // Sort by count descending
        patterns.sort_by(|a, b| b.count.cmp(&a.count));

        patterns
    }

    /// Generate observations about test coverage
    fn generate_observations(&self, analysis: &TestAnalysis) -> Vec<String> {
        let mut observations = Vec::new();

        // Coverage observation
        if analysis.coverage_percentage < 20.0 {
            observations.push(format!(
                "Low test coverage: {:.1}% of modules have tests. Consider adding tests for critical modules.",
                analysis.coverage_percentage
            ));
        } else if analysis.coverage_percentage < 50.0 {
            observations.push(format!(
                "Moderate test coverage: {:.1}% of modules have tests.",
                analysis.coverage_percentage
            ));
        } else if analysis.coverage_percentage >= 80.0 {
            observations.push(format!(
                "Good test coverage: {:.1}% of modules have tests.",
                analysis.coverage_percentage
            ));
        } else {
            observations.push(format!(
                "Test coverage: {:.1}% of modules have tests.",
                analysis.coverage_percentage
            ));
        }

        // Test count observation
        if analysis.test_function_count == 0 {
            observations.push(
                "No test functions found. Consider adding tests to ensure code quality."
                    .to_string(),
            );
        } else {
            observations.push(format!(
                "Found {} test function(s) across {} test file(s).",
                analysis.test_function_count, analysis.test_file_count
            ));
        }

        // Untested modules observation
        if !analysis.untested_modules.is_empty() {
            let count = analysis.untested_modules.len();
            if count <= 5 {
                let names: Vec<_> = analysis
                    .untested_modules
                    .iter()
                    .map(|m| m.name.as_str())
                    .collect();
                observations.push(format!("Untested modules: {}", names.join(", ")));
            } else {
                observations.push(format!(
                    "{} modules appear to lack tests. Consider prioritizing tests for critical functionality.",
                    count
                ));
            }
        }

        // Test pattern observations
        let has_unit = analysis
            .test_patterns
            .iter()
            .any(|p| p.pattern == TestPattern::Unit);
        let has_integration = analysis
            .test_patterns
            .iter()
            .any(|p| p.pattern == TestPattern::Integration);
        let has_e2e = analysis
            .test_patterns
            .iter()
            .any(|p| p.pattern == TestPattern::E2e);

        if has_unit && !has_integration && !has_e2e {
            observations.push(
                "Only unit tests detected. Consider adding integration or e2e tests for comprehensive coverage."
                    .to_string(),
            );
        } else if has_unit && has_integration && has_e2e {
            observations
                .push("Good test pyramid: unit, integration, and e2e tests detected.".to_string());
        }

        observations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_test_pattern_default() {
        assert_eq!(TestPattern::default(), TestPattern::Unknown);
    }

    #[test]
    fn test_test_pattern_display() {
        assert_eq!(format!("{}", TestPattern::Unit), "unit");
        assert_eq!(format!("{}", TestPattern::Integration), "integration");
        assert_eq!(format!("{}", TestPattern::E2e), "e2e");
        assert_eq!(format!("{}", TestPattern::Property), "property");
        assert_eq!(format!("{}", TestPattern::Benchmark), "benchmark");
        assert_eq!(format!("{}", TestPattern::Snapshot), "snapshot");
    }

    #[test]
    fn test_test_analyzer_new() {
        let analyzer = TestAnalyzer::new(PathBuf::from("/test"));
        assert_eq!(analyzer.root(), &PathBuf::from("/test"));
    }

    #[test]
    fn test_is_test_file_rust() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = TestAnalyzer::new(temp_dir.path().to_path_buf());

        // Test in tests directory
        let tests_dir = temp_dir.path().join("tests");
        fs::create_dir(&tests_dir).unwrap();
        let test_file = tests_dir.join("test_foo.rs");
        fs::write(&test_file, "#[test]\nfn test_something() {}").unwrap();

        assert!(analyzer.is_test_file(&test_file, "rs"));
    }

    #[test]
    fn test_is_test_file_go() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = TestAnalyzer::new(temp_dir.path().to_path_buf());

        let test_file = temp_dir.path().join("foo_test.go");
        fs::write(&test_file, "package main\nfunc TestFoo(t *testing.T) {}").unwrap();

        assert!(analyzer.is_test_file(&test_file, "go"));

        let non_test_file = temp_dir.path().join("foo.go");
        fs::write(&non_test_file, "package main\nfunc main() {}").unwrap();

        assert!(!analyzer.is_test_file(&non_test_file, "go"));
    }

    #[test]
    fn test_is_test_file_javascript() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = TestAnalyzer::new(temp_dir.path().to_path_buf());

        let test_file = temp_dir.path().join("foo.test.js");
        fs::write(&test_file, "test('foo', () => {});").unwrap();

        assert!(analyzer.is_test_file(&test_file, "js"));

        let spec_file = temp_dir.path().join("foo.spec.js");
        fs::write(&spec_file, "it('should work', () => {});").unwrap();

        assert!(analyzer.is_test_file(&spec_file, "js"));
    }

    #[test]
    fn test_is_test_file_python() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = TestAnalyzer::new(temp_dir.path().to_path_buf());

        let test_file = temp_dir.path().join("test_foo.py");
        fs::write(&test_file, "def test_something(): pass").unwrap();

        assert!(analyzer.is_test_file(&test_file, "py"));

        let suffix_file = temp_dir.path().join("foo_test.py");
        fs::write(&suffix_file, "def test_something(): pass").unwrap();

        assert!(analyzer.is_test_file(&suffix_file, "py"));
    }

    #[test]
    fn test_count_test_functions_rust() {
        let analyzer = TestAnalyzer::new(PathBuf::from("/test"));

        let content = r#"
            #[test]
            fn test_one() {}

            #[test]
            fn test_two() {}

            #[tokio::test]
            async fn test_async() {}
        "#;

        assert_eq!(analyzer.count_test_functions(content, "rs"), 3);
    }

    #[test]
    fn test_count_test_functions_go() {
        let analyzer = TestAnalyzer::new(PathBuf::from("/test"));

        let content = r#"
            func TestOne(t *testing.T) {}
            func TestTwo(t *testing.T) {}
            func BenchmarkFoo(b *testing.B) {}
        "#;

        assert_eq!(analyzer.count_test_functions(content, "go"), 2);
    }

    #[test]
    fn test_count_test_functions_python() {
        let analyzer = TestAnalyzer::new(PathBuf::from("/test"));

        let content = r#"
            def test_one():
                pass

            def test_two():
                pass

            class TestClass(unittest.TestCase):
                def testMethod(self):
                    pass
        "#;

        assert_eq!(analyzer.count_test_functions(content, "py"), 3);
    }

    #[test]
    fn test_count_test_functions_javascript() {
        let analyzer = TestAnalyzer::new(PathBuf::from("/test"));

        let content = r#"
            describe('MyComponent', () => {
                it('should render', () => {});
                test('should handle click', () => {});
                it('should update state', () => {});
            });
        "#;

        assert_eq!(analyzer.count_test_functions(content, "js"), 3);
    }

    #[test]
    fn test_count_test_functions_java() {
        let analyzer = TestAnalyzer::new(PathBuf::from("/test"));

        let content = r#"
            public class FooTest {
                @Test
                public void testOne() {}

                @Test
                public void testTwo() {}
            }
        "#;

        assert_eq!(analyzer.count_test_functions(content, "java"), 2);
    }

    #[test]
    fn test_detect_test_patterns_unit() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = TestAnalyzer::new(temp_dir.path().to_path_buf());

        let test_file = temp_dir.path().join("test_foo.rs");
        let content = "#[test]\nfn test_something() { assert!(true); }";

        let patterns = analyzer.detect_test_patterns_in_file(content, &test_file, "rs");

        assert!(patterns.contains(&TestPattern::Unit));
    }

    #[test]
    fn test_detect_test_patterns_e2e() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = TestAnalyzer::new(temp_dir.path().to_path_buf());

        let e2e_dir = temp_dir.path().join("e2e");
        fs::create_dir(&e2e_dir).unwrap();
        let test_file = e2e_dir.join("test_flow.js");

        let content =
            "const { chromium } = require('playwright');\ntest('user flow', async () => {});";

        let patterns = analyzer.detect_test_patterns_in_file(content, &test_file, "js");

        assert!(patterns.contains(&TestPattern::E2e));
    }

    #[test]
    fn test_detect_test_patterns_integration() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = TestAnalyzer::new(temp_dir.path().to_path_buf());

        let integration_dir = temp_dir.path().join("integration");
        fs::create_dir(&integration_dir).unwrap();
        let test_file = integration_dir.join("test_api.rs");

        let content = "#[test]\nfn test_api_integration() { /* testcontainers */ }";

        let patterns = analyzer.detect_test_patterns_in_file(content, &test_file, "rs");

        assert!(patterns.contains(&TestPattern::Integration));
    }

    #[test]
    fn test_detect_test_patterns_benchmark() {
        let analyzer = TestAnalyzer::new(PathBuf::from("/test"));

        let test_file = PathBuf::from("/test/benches/bench_foo.rs");
        let content = "#[bench]\nfn bench_something(b: &mut Bencher) {}\nuse criterion::*;";

        let patterns = analyzer.detect_test_patterns_in_file(content, &test_file, "rs");

        assert!(patterns.contains(&TestPattern::Benchmark));
    }

    #[test]
    fn test_detect_test_patterns_property() {
        let analyzer = TestAnalyzer::new(PathBuf::from("/test"));

        let test_file = PathBuf::from("/test/tests/prop_test.rs");
        let content = "use proptest::prelude::*;\nproptest! { fn test_prop(x: u32) { } }";

        let patterns = analyzer.detect_test_patterns_in_file(content, &test_file, "rs");

        assert!(patterns.contains(&TestPattern::Property));
    }

    #[test]
    fn test_detect_test_patterns_snapshot() {
        let analyzer = TestAnalyzer::new(PathBuf::from("/test"));

        let test_file = PathBuf::from("/test/tests/snapshot_test.rs");
        let content = "use insta::assert_snapshot;\n#[test]\nfn test_snapshot() { insta::assert_snapshot!(output); }";

        let patterns = analyzer.detect_test_patterns_in_file(content, &test_file, "rs");

        assert!(patterns.contains(&TestPattern::Snapshot));
    }

    #[test]
    fn test_infer_covered_module() {
        let analyzer = TestAnalyzer::new(PathBuf::from("/test"));

        assert_eq!(
            analyzer.infer_covered_module(std::path::Path::new("/test/tests/test_foo.rs")),
            Some("foo".to_string())
        );

        assert_eq!(
            analyzer.infer_covered_module(std::path::Path::new("/test/tests/bar_test.rs")),
            Some("bar".to_string())
        );

        assert_eq!(
            analyzer.infer_covered_module(std::path::Path::new("/test/tests/baz.test.js")),
            Some("baz".to_string())
        );

        assert_eq!(
            analyzer.infer_covered_module(std::path::Path::new("/test/tests/qux.spec.ts")),
            Some("qux".to_string())
        );
    }

    #[test]
    fn test_analyze_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = TestAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        assert_eq!(analysis.test_file_count, 0);
        assert_eq!(analysis.test_function_count, 0);
        assert!(analysis.test_files.is_empty());
        assert_eq!(analysis.coverage_percentage, 0.0);
    }

    #[test]
    fn test_analyze_rust_project_with_tests() {
        let temp_dir = TempDir::new().unwrap();

        // Create source directory
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create source file
        fs::write(src_dir.join("foo.rs"), "pub fn foo() -> i32 { 42 }").unwrap();

        fs::write(src_dir.join("bar.rs"), "pub fn bar() -> i32 { 21 }").unwrap();

        // Create tests directory
        let tests_dir = temp_dir.path().join("tests");
        fs::create_dir(&tests_dir).unwrap();

        // Create test file for foo
        fs::write(
            tests_dir.join("test_foo.rs"),
            r#"
            #[test]
            fn test_foo_returns_42() {
                assert_eq!(42, 42);
            }

            #[test]
            fn test_foo_is_positive() {
                assert!(42 > 0);
            }
            "#,
        )
        .unwrap();

        let analyzer = TestAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        assert_eq!(analysis.test_file_count, 1);
        assert_eq!(analysis.test_function_count, 2);

        // Check that foo has tests and bar doesn't
        let foo_module = analysis.source_modules.iter().find(|m| m.name == "foo");
        assert!(foo_module.is_some());
        assert!(foo_module.unwrap().has_tests);

        let bar_module = analysis.untested_modules.iter().find(|m| m.name == "bar");
        assert!(bar_module.is_some());
    }

    #[test]
    fn test_analyze_rust_inline_tests() {
        let temp_dir = TempDir::new().unwrap();

        // Create source directory
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create source file with inline tests
        fs::write(
            src_dir.join("inline.rs"),
            r#"
            pub fn add(a: i32, b: i32) -> i32 {
                a + b
            }

            #[cfg(test)]
            mod tests {
                use super::*;

                #[test]
                fn test_add() {
                    assert_eq!(add(2, 3), 5);
                }
            }
            "#,
        )
        .unwrap();

        let analyzer = TestAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        // File should be detected as having tests
        assert!(analysis.test_file_count >= 1);
        assert!(analysis.test_function_count >= 1);
    }

    #[test]
    fn test_analyze_javascript_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create source directory
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        fs::write(
            src_dir.join("utils.js"),
            "export function add(a, b) { return a + b; }",
        )
        .unwrap();

        // Create __tests__ directory
        let tests_dir = src_dir.join("__tests__");
        fs::create_dir(&tests_dir).unwrap();

        fs::write(
            tests_dir.join("utils.test.js"),
            r#"
            import { add } from '../utils';

            test('add returns sum', () => {
                expect(add(2, 3)).toBe(5);
            });

            it('handles negative numbers', () => {
                expect(add(-1, 1)).toBe(0);
            });
            "#,
        )
        .unwrap();

        let analyzer = TestAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        assert_eq!(analysis.test_file_count, 1);
        assert_eq!(analysis.test_function_count, 2);
    }

    #[test]
    fn test_analyze_python_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create source file
        fs::write(
            temp_dir.path().join("calculator.py"),
            "def add(a, b): return a + b",
        )
        .unwrap();

        // Create test file
        fs::write(
            temp_dir.path().join("test_calculator.py"),
            r#"
            from calculator import add

            def test_add_positive():
                assert add(2, 3) == 5

            def test_add_negative():
                assert add(-1, 1) == 0
            "#,
        )
        .unwrap();

        let analyzer = TestAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        assert_eq!(analysis.test_file_count, 1);
        assert_eq!(analysis.test_function_count, 2);
    }

    #[test]
    fn test_analyze_go_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create source file
        fs::write(
            temp_dir.path().join("math.go"),
            "package math\n\nfunc Add(a, b int) int { return a + b }",
        )
        .unwrap();

        // Create test file
        fs::write(
            temp_dir.path().join("math_test.go"),
            r#"
            package math

            import "testing"

            func TestAdd(t *testing.T) {
                if Add(2, 3) != 5 {
                    t.Error("Expected 5")
                }
            }

            func TestAddNegative(t *testing.T) {
                if Add(-1, 1) != 0 {
                    t.Error("Expected 0")
                }
            }
            "#,
        )
        .unwrap();

        let analyzer = TestAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        assert_eq!(analysis.test_file_count, 1);
        assert_eq!(analysis.test_function_count, 2);
    }

    #[test]
    fn test_coverage_percentage_calculation() {
        let temp_dir = TempDir::new().unwrap();

        // Create 4 source files
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        for name in ["a", "b", "c", "d"] {
            fs::write(
                src_dir.join(format!("{}.rs", name)),
                format!("pub fn {}() {{}}", name),
            )
            .unwrap();
        }

        // Create tests for only 2 of them
        let tests_dir = temp_dir.path().join("tests");
        fs::create_dir(&tests_dir).unwrap();

        fs::write(tests_dir.join("test_a.rs"), "#[test] fn test_a() {}").unwrap();

        fs::write(tests_dir.join("test_b.rs"), "#[test] fn test_b() {}").unwrap();

        let analyzer = TestAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        // 2 out of 4 modules have tests = 50%
        assert!((analysis.coverage_percentage - 50.0).abs() < 0.1);
        assert_eq!(analysis.untested_modules.len(), 2);
    }

    #[test]
    fn test_test_pattern_info() {
        let temp_dir = TempDir::new().unwrap();

        // Create unit tests
        let tests_dir = temp_dir.path().join("tests");
        fs::create_dir(&tests_dir).unwrap();

        fs::write(tests_dir.join("test_unit.rs"), "#[test] fn test_unit() {}").unwrap();

        // Create integration tests
        let integration_dir = temp_dir.path().join("integration");
        fs::create_dir(&integration_dir).unwrap();

        fs::write(
            integration_dir.join("test_integration.rs"),
            "#[test] fn test_integration() {}",
        )
        .unwrap();

        let analyzer = TestAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        assert!(analysis
            .test_patterns
            .iter()
            .any(|p| p.pattern == TestPattern::Unit));
        assert!(analysis
            .test_patterns
            .iter()
            .any(|p| p.pattern == TestPattern::Integration));
    }

    #[test]
    fn test_observations_low_coverage() {
        let temp_dir = TempDir::new().unwrap();

        // Create many source files
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        for i in 0..10 {
            fs::write(
                src_dir.join(format!("module{}.rs", i)),
                format!("pub fn func{}() {{}}", i),
            )
            .unwrap();
        }

        // Create test for only one
        let tests_dir = temp_dir.path().join("tests");
        fs::create_dir(&tests_dir).unwrap();

        fs::write(
            tests_dir.join("test_module0.rs"),
            "#[test] fn test_module0() {}",
        )
        .unwrap();

        let analyzer = TestAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        // Should have low coverage observation
        assert!(
            analysis
                .observations
                .iter()
                .any(|o| o.contains("Low test coverage")
                    || o.contains("modules appear to lack tests"))
        );
    }

    #[test]
    fn test_test_analysis_serialization() {
        let analysis = TestAnalysis {
            test_file_count: 5,
            test_function_count: 20,
            test_files: vec![TestFile {
                path: PathBuf::from("tests/test_foo.rs"),
                test_count: 4,
                patterns: vec![TestPattern::Unit],
                covers_module: Some("foo".to_string()),
            }],
            source_modules: vec![SourceModule {
                name: "foo".to_string(),
                path: PathBuf::from("src/foo.rs"),
                has_tests: true,
                test_files: vec![PathBuf::from("tests/test_foo.rs")],
            }],
            untested_modules: vec![SourceModule {
                name: "bar".to_string(),
                path: PathBuf::from("src/bar.rs"),
                has_tests: false,
                test_files: vec![],
            }],
            test_patterns: vec![TestPatternInfo {
                pattern: TestPattern::Unit,
                count: 20,
                examples: vec![PathBuf::from("tests/test_foo.rs")],
            }],
            coverage_percentage: 50.0,
            observations: vec!["Moderate test coverage".to_string()],
        };

        let json = serde_json::to_string(&analysis).unwrap();
        let deserialized: TestAnalysis = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.test_file_count, 5);
        assert_eq!(deserialized.test_function_count, 20);
        assert_eq!(deserialized.coverage_percentage, 50.0);
        assert_eq!(deserialized.test_files.len(), 1);
        assert_eq!(deserialized.source_modules.len(), 1);
        assert_eq!(deserialized.untested_modules.len(), 1);
    }
}
