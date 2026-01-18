//! Language detection and analysis.

use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use super::AuditResult;

/// Level of language support in the codebase
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LanguageSupport {
    Primary,
    Secondary,
    #[default]
    Minimal,
}

/// Information about a detected language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    /// Language name
    pub name: String,
    /// File extensions associated
    pub extensions: Vec<String>,
    /// Number of files
    pub file_count: usize,
    /// Lines of code
    pub loc: usize,
    /// Percentage of total LOC
    pub percentage: f64,
    /// Support level in the project
    pub support: LanguageSupport,
}

/// Language definition for detection
#[derive(Debug, Clone)]
struct LanguageDefinition {
    name: &'static str,
    extensions: &'static [&'static str],
    manifest_files: &'static [&'static str],
}

/// Known language definitions
const LANGUAGE_DEFINITIONS: &[LanguageDefinition] = &[
    LanguageDefinition {
        name: "Rust",
        extensions: &["rs"],
        manifest_files: &["Cargo.toml"],
    },
    LanguageDefinition {
        name: "TypeScript",
        extensions: &["ts", "tsx", "mts", "cts"],
        manifest_files: &["tsconfig.json"],
    },
    LanguageDefinition {
        name: "JavaScript",
        extensions: &["js", "jsx", "mjs", "cjs"],
        manifest_files: &["package.json"],
    },
    LanguageDefinition {
        name: "Python",
        extensions: &["py", "pyw", "pyi"],
        manifest_files: &["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"],
    },
    LanguageDefinition {
        name: "Go",
        extensions: &["go"],
        manifest_files: &["go.mod"],
    },
    LanguageDefinition {
        name: "Java",
        extensions: &["java"],
        manifest_files: &["pom.xml", "build.gradle", "build.gradle.kts"],
    },
    LanguageDefinition {
        name: "Ruby",
        extensions: &["rb", "rake"],
        manifest_files: &["Gemfile"],
    },
    LanguageDefinition {
        name: "C",
        extensions: &["c", "h"],
        manifest_files: &[],
    },
    LanguageDefinition {
        name: "C++",
        extensions: &["cpp", "hpp", "cc", "cxx", "hxx"],
        manifest_files: &["CMakeLists.txt"],
    },
    LanguageDefinition {
        name: "C#",
        extensions: &["cs"],
        manifest_files: &[],
    },
    LanguageDefinition {
        name: "Swift",
        extensions: &["swift"],
        manifest_files: &["Package.swift"],
    },
    LanguageDefinition {
        name: "Kotlin",
        extensions: &["kt", "kts"],
        manifest_files: &[],
    },
    LanguageDefinition {
        name: "PHP",
        extensions: &["php"],
        manifest_files: &["composer.json"],
    },
    LanguageDefinition {
        name: "Scala",
        extensions: &["scala", "sc"],
        manifest_files: &["build.sbt"],
    },
    LanguageDefinition {
        name: "Shell",
        extensions: &["sh", "bash", "zsh", "fish"],
        manifest_files: &[],
    },
];

/// Detector for identifying languages in files
pub struct LanguageDetector {
    /// Extension to language name mapping
    extension_map: HashMap<&'static str, &'static str>,
    /// Language definitions for more complex detection
    definitions: Vec<&'static LanguageDefinition>,
}

impl LanguageDetector {
    /// Create a new language detector
    pub fn new() -> Self {
        let mut extension_map = HashMap::new();
        let definitions: Vec<_> = LANGUAGE_DEFINITIONS.iter().collect();

        for def in LANGUAGE_DEFINITIONS {
            for ext in def.extensions {
                extension_map.insert(*ext, def.name);
            }
        }

        Self {
            extension_map,
            definitions,
        }
    }

    /// Detect language from file extension
    pub fn detect_from_extension(&self, extension: &str) -> Option<String> {
        self.extension_map
            .get(extension.to_lowercase().as_str())
            .map(|s| s.to_string())
    }

    /// Get manifest files for a language
    pub fn manifest_files_for(&self, language: &str) -> Vec<&'static str> {
        self.definitions
            .iter()
            .find(|d| d.name == language)
            .map(|d| d.manifest_files.to_vec())
            .unwrap_or_default()
    }

    /// Get extensions for a language
    pub fn extensions_for(&self, language: &str) -> Vec<&'static str> {
        self.definitions
            .iter()
            .find(|d| d.name == language)
            .map(|d| d.extensions.to_vec())
            .unwrap_or_default()
    }

    /// Check if a manifest file indicates a specific language
    pub fn detect_from_manifest(&self, filename: &str) -> Option<String> {
        for def in &self.definitions {
            if def.manifest_files.contains(&filename) {
                return Some(def.name.to_string());
            }
        }
        None
    }
}

impl Default for LanguageDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for a single language
#[derive(Debug, Default)]
struct LanguageStats {
    file_count: usize,
    loc: usize,
    extensions: Vec<String>,
}

/// Analyzer for aggregating language statistics
pub struct LanguageAnalyzer {
    root: PathBuf,
    detector: LanguageDetector,
}

impl LanguageAnalyzer {
    /// Create a new language analyzer
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            detector: LanguageDetector::new(),
        }
    }

    /// Analyze languages in the project
    pub fn analyze(&self) -> AuditResult<Vec<LanguageInfo>> {
        let mut stats: HashMap<String, LanguageStats> = HashMap::new();
        let mut total_loc = 0usize;

        // Walk the directory tree respecting .gitignore
        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if let Some(lang) = self.detector.detect_from_extension(ext) {
                        let loc = self.count_lines(path);
                        total_loc += loc;

                        let entry = stats.entry(lang).or_default();
                        entry.file_count += 1;
                        entry.loc += loc;

                        let ext_string = ext.to_lowercase();
                        if !entry.extensions.contains(&ext_string) {
                            entry.extensions.push(ext_string);
                        }
                    }
                }
            }
        }

        // Also check for manifest files at root
        self.detect_manifest_languages(&mut stats);

        // Convert stats to LanguageInfo with percentages and support levels
        let mut languages: Vec<LanguageInfo> = stats
            .into_iter()
            .map(|(name, stat)| {
                let percentage = if total_loc > 0 {
                    (stat.loc as f64 / total_loc as f64) * 100.0
                } else if stat.file_count > 0 {
                    // If no LOC but files exist, assign minimal percentage
                    0.1
                } else {
                    0.0
                };

                LanguageInfo {
                    name,
                    extensions: stat.extensions,
                    file_count: stat.file_count,
                    loc: stat.loc,
                    percentage,
                    support: LanguageSupport::Minimal, // Will be updated below
                }
            })
            .collect();

        // Sort by LOC descending
        languages.sort_by(|a, b| b.loc.cmp(&a.loc));

        // Assign support levels based on percentage
        self.assign_support_levels(&mut languages);

        Ok(languages)
    }

    /// Count lines in a file
    fn count_lines(&self, path: &Path) -> usize {
        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return 0,
        };

        let reader = BufReader::new(file);
        reader.lines().count()
    }

    /// Detect languages from manifest files at root
    fn detect_manifest_languages(&self, stats: &mut HashMap<String, LanguageStats>) {
        // Check for Rust
        if self.root.join("Cargo.toml").exists() {
            stats.entry("Rust".to_string()).or_default();
        }

        // Check for TypeScript/JavaScript
        if self.root.join("package.json").exists() {
            // Check if tsconfig.json exists to determine TypeScript vs JavaScript
            if self.root.join("tsconfig.json").exists() {
                stats.entry("TypeScript".to_string()).or_default();
            } else {
                stats.entry("JavaScript".to_string()).or_default();
            }
        }

        // Check for Python
        if self.root.join("pyproject.toml").exists()
            || self.root.join("requirements.txt").exists()
            || self.root.join("setup.py").exists()
            || self.root.join("Pipfile").exists()
        {
            stats.entry("Python".to_string()).or_default();
        }

        // Check for Go
        if self.root.join("go.mod").exists() {
            stats.entry("Go".to_string()).or_default();
        }
    }

    /// Assign support levels based on percentages
    fn assign_support_levels(&self, languages: &mut [LanguageInfo]) {
        if languages.is_empty() {
            return;
        }

        // Primary: >= 50% of total LOC OR the top language if it has >= 30%
        // Secondary: >= 10% of total LOC
        // Minimal: < 10% of total LOC

        for (i, lang) in languages.iter_mut().enumerate() {
            let is_primary = (i == 0 && lang.percentage >= 30.0) || lang.percentage >= 50.0;

            if is_primary {
                lang.support = LanguageSupport::Primary;
            } else if lang.percentage >= 10.0 {
                lang.support = LanguageSupport::Secondary;
            } else {
                lang.support = LanguageSupport::Minimal;
            }
        }
    }

    /// Get the root path
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Get the detector
    pub fn detector(&self) -> &LanguageDetector {
        &self.detector
    }

    /// Get primary languages (support level = Primary)
    pub fn primary_languages(&self) -> AuditResult<Vec<LanguageInfo>> {
        let all = self.analyze()?;
        Ok(all
            .into_iter()
            .filter(|l| l.support == LanguageSupport::Primary)
            .collect())
    }

    /// Get secondary languages (support level = Secondary)
    pub fn secondary_languages(&self) -> AuditResult<Vec<LanguageInfo>> {
        let all = self.analyze()?;
        Ok(all
            .into_iter()
            .filter(|l| l.support == LanguageSupport::Secondary)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_language_support_default() {
        assert_eq!(LanguageSupport::default(), LanguageSupport::Minimal);
    }

    #[test]
    fn test_language_detector_rust() {
        let detector = LanguageDetector::new();
        assert_eq!(
            detector.detect_from_extension("rs"),
            Some("Rust".to_string())
        );
    }

    #[test]
    fn test_language_detector_typescript() {
        let detector = LanguageDetector::new();
        assert_eq!(
            detector.detect_from_extension("ts"),
            Some("TypeScript".to_string())
        );
        assert_eq!(
            detector.detect_from_extension("tsx"),
            Some("TypeScript".to_string())
        );
    }

    #[test]
    fn test_language_detector_javascript() {
        let detector = LanguageDetector::new();
        assert_eq!(
            detector.detect_from_extension("js"),
            Some("JavaScript".to_string())
        );
        assert_eq!(
            detector.detect_from_extension("jsx"),
            Some("JavaScript".to_string())
        );
        assert_eq!(
            detector.detect_from_extension("mjs"),
            Some("JavaScript".to_string())
        );
    }

    #[test]
    fn test_language_detector_python() {
        let detector = LanguageDetector::new();
        assert_eq!(
            detector.detect_from_extension("py"),
            Some("Python".to_string())
        );
        assert_eq!(
            detector.detect_from_extension("pyi"),
            Some("Python".to_string())
        );
    }

    #[test]
    fn test_language_detector_go() {
        let detector = LanguageDetector::new();
        assert_eq!(detector.detect_from_extension("go"), Some("Go".to_string()));
    }

    #[test]
    fn test_language_detector_unknown() {
        let detector = LanguageDetector::new();
        assert_eq!(detector.detect_from_extension("xyz"), None);
    }

    #[test]
    fn test_language_detector_case_insensitive() {
        let detector = LanguageDetector::new();
        assert_eq!(
            detector.detect_from_extension("RS"),
            Some("Rust".to_string())
        );
        assert_eq!(
            detector.detect_from_extension("Py"),
            Some("Python".to_string())
        );
    }

    #[test]
    fn test_detect_from_manifest_cargo() {
        let detector = LanguageDetector::new();
        assert_eq!(
            detector.detect_from_manifest("Cargo.toml"),
            Some("Rust".to_string())
        );
    }

    #[test]
    fn test_detect_from_manifest_package_json() {
        let detector = LanguageDetector::new();
        assert_eq!(
            detector.detect_from_manifest("package.json"),
            Some("JavaScript".to_string())
        );
    }

    #[test]
    fn test_detect_from_manifest_pyproject() {
        let detector = LanguageDetector::new();
        assert_eq!(
            detector.detect_from_manifest("pyproject.toml"),
            Some("Python".to_string())
        );
        assert_eq!(
            detector.detect_from_manifest("requirements.txt"),
            Some("Python".to_string())
        );
    }

    #[test]
    fn test_detect_from_manifest_go_mod() {
        let detector = LanguageDetector::new();
        assert_eq!(
            detector.detect_from_manifest("go.mod"),
            Some("Go".to_string())
        );
    }

    #[test]
    fn test_manifest_files_for_rust() {
        let detector = LanguageDetector::new();
        let manifests = detector.manifest_files_for("Rust");
        assert!(manifests.contains(&"Cargo.toml"));
    }

    #[test]
    fn test_extensions_for_python() {
        let detector = LanguageDetector::new();
        let exts = detector.extensions_for("Python");
        assert!(exts.contains(&"py"));
        assert!(exts.contains(&"pyi"));
    }

    #[test]
    fn test_analyze_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = LanguageAnalyzer::new(temp_dir.path().to_path_buf());
        let languages = analyzer.analyze().unwrap();

        assert!(languages.is_empty());
    }

    #[test]
    fn test_analyze_rust_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create Cargo.toml
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        // Create src directory with Rust files
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(
            src_dir.join("main.rs"),
            "fn main() {\n    println!(\"Hello\");\n}\n",
        )
        .unwrap();
        fs::write(
            src_dir.join("lib.rs"),
            "pub fn greet() {\n    println!(\"Hi\");\n}\n",
        )
        .unwrap();

        let analyzer = LanguageAnalyzer::new(temp_dir.path().to_path_buf());
        let languages = analyzer.analyze().unwrap();

        assert!(!languages.is_empty());
        let rust = languages.iter().find(|l| l.name == "Rust").unwrap();
        assert_eq!(rust.file_count, 2);
        assert_eq!(rust.loc, 6); // 3 lines each
        assert!(rust.percentage > 0.0);
        assert_eq!(rust.support, LanguageSupport::Primary);
    }

    #[test]
    fn test_analyze_typescript_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create package.json and tsconfig.json
        fs::write(
            temp_dir.path().join("package.json"),
            "{\"name\": \"test\"}\n",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("tsconfig.json"),
            "{\"compilerOptions\": {}}\n",
        )
        .unwrap();

        // Create TypeScript files
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(
            src_dir.join("index.ts"),
            "const greeting: string = 'Hello';\nconsole.log(greeting);\n",
        )
        .unwrap();
        fs::write(
            src_dir.join("App.tsx"),
            "const App = () => <div>Hello</div>;\nexport default App;\n",
        )
        .unwrap();

        let analyzer = LanguageAnalyzer::new(temp_dir.path().to_path_buf());
        let languages = analyzer.analyze().unwrap();

        let typescript = languages.iter().find(|l| l.name == "TypeScript").unwrap();
        assert_eq!(typescript.file_count, 2);
        assert!(typescript.extensions.contains(&"ts".to_string()));
        assert!(typescript.extensions.contains(&"tsx".to_string()));
        assert_eq!(typescript.support, LanguageSupport::Primary);
    }

    #[test]
    fn test_analyze_javascript_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create package.json (no tsconfig)
        fs::write(
            temp_dir.path().join("package.json"),
            "{\"name\": \"test\"}\n",
        )
        .unwrap();

        // Create JavaScript files
        fs::write(
            temp_dir.path().join("index.js"),
            "const greeting = 'Hello';\nconsole.log(greeting);\n",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("utils.mjs"),
            "export const add = (a, b) => a + b;\n",
        )
        .unwrap();

        let analyzer = LanguageAnalyzer::new(temp_dir.path().to_path_buf());
        let languages = analyzer.analyze().unwrap();

        let javascript = languages.iter().find(|l| l.name == "JavaScript").unwrap();
        assert_eq!(javascript.file_count, 2);
        assert!(javascript.extensions.contains(&"js".to_string()));
        assert!(javascript.extensions.contains(&"mjs".to_string()));
    }

    #[test]
    fn test_analyze_python_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create requirements.txt
        fs::write(
            temp_dir.path().join("requirements.txt"),
            "flask==2.0.0\nrequests==2.28.0\n",
        )
        .unwrap();

        // Create Python files
        fs::write(
            temp_dir.path().join("app.py"),
            "from flask import Flask\napp = Flask(__name__)\n\n@app.route('/')\ndef hello():\n    return 'Hello'\n",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("utils.py"),
            "def add(a, b):\n    return a + b\n",
        )
        .unwrap();

        let analyzer = LanguageAnalyzer::new(temp_dir.path().to_path_buf());
        let languages = analyzer.analyze().unwrap();

        let python = languages.iter().find(|l| l.name == "Python").unwrap();
        assert_eq!(python.file_count, 2);
        assert!(python.extensions.contains(&"py".to_string()));
        assert_eq!(python.support, LanguageSupport::Primary);
    }

    #[test]
    fn test_analyze_go_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create go.mod
        fs::write(
            temp_dir.path().join("go.mod"),
            "module example.com/test\n\ngo 1.21\n",
        )
        .unwrap();

        // Create Go files
        fs::write(
            temp_dir.path().join("main.go"),
            "package main\n\nimport \"fmt\"\n\nfunc main() {\n    fmt.Println(\"Hello\")\n}\n",
        )
        .unwrap();
        fs::write(
            temp_dir.path().join("utils.go"),
            "package main\n\nfunc Add(a, b int) int {\n    return a + b\n}\n",
        )
        .unwrap();

        let analyzer = LanguageAnalyzer::new(temp_dir.path().to_path_buf());
        let languages = analyzer.analyze().unwrap();

        let go = languages.iter().find(|l| l.name == "Go").unwrap();
        assert_eq!(go.file_count, 2);
        assert!(go.extensions.contains(&"go".to_string()));
        assert_eq!(go.support, LanguageSupport::Primary);
    }

    #[test]
    fn test_analyze_mixed_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create Rust files (60% of code)
        let rust_dir = temp_dir.path().join("rust");
        fs::create_dir(&rust_dir).unwrap();
        fs::write(
            rust_dir.join("main.rs"),
            "fn main() {\n    println!(\"Hello\");\n    println!(\"World\");\n    println!(\"!\");\n}\n\nfn foo() {\n    let x = 1;\n    let y = 2;\n    let z = x + y;\n}\n",
        )
        .unwrap();

        // Create Python files (30% of code)
        let py_dir = temp_dir.path().join("python");
        fs::create_dir(&py_dir).unwrap();
        fs::write(
            py_dir.join("app.py"),
            "def main():\n    print('Hello')\n    print('World')\n",
        )
        .unwrap();

        // Create Shell script (10% of code)
        fs::write(
            temp_dir.path().join("build.sh"),
            "#!/bin/bash\necho 'Building...'\n",
        )
        .unwrap();

        let analyzer = LanguageAnalyzer::new(temp_dir.path().to_path_buf());
        let languages = analyzer.analyze().unwrap();

        assert!(languages.len() >= 2);

        // Rust should be primary (highest LOC)
        let rust = languages.iter().find(|l| l.name == "Rust").unwrap();
        assert_eq!(rust.support, LanguageSupport::Primary);

        // Python should be secondary
        let python = languages.iter().find(|l| l.name == "Python").unwrap();
        assert_eq!(python.support, LanguageSupport::Secondary);
    }

    #[test]
    fn test_percentages_add_up() {
        let temp_dir = TempDir::new().unwrap();

        // Create files for multiple languages
        fs::write(temp_dir.path().join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(temp_dir.path().join("app.py"), "print('hi')\n").unwrap();
        fs::write(temp_dir.path().join("index.js"), "console.log('hi');\n").unwrap();

        let analyzer = LanguageAnalyzer::new(temp_dir.path().to_path_buf());
        let languages = analyzer.analyze().unwrap();

        let total_percentage: f64 = languages.iter().map(|l| l.percentage).sum();
        // Should be approximately 100% (allowing for floating point errors)
        assert!((total_percentage - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_primary_languages() {
        let temp_dir = TempDir::new().unwrap();

        // Create enough Rust code to be primary
        fs::write(
            temp_dir.path().join("main.rs"),
            "fn main() {\n    println!(\"Hello\");\n}\n",
        )
        .unwrap();

        let analyzer = LanguageAnalyzer::new(temp_dir.path().to_path_buf());
        let primary = analyzer.primary_languages().unwrap();

        assert!(!primary.is_empty());
        assert!(primary
            .iter()
            .all(|l| l.support == LanguageSupport::Primary));
    }

    #[test]
    fn test_secondary_languages() {
        let temp_dir = TempDir::new().unwrap();

        // Create primary language (Rust - 80%)
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        for i in 0..8 {
            fs::write(
                src_dir.join(format!("file{}.rs", i)),
                "fn test() {\n    let x = 1;\n}\n",
            )
            .unwrap();
        }

        // Create secondary language (Python - 20%)
        for i in 0..2 {
            fs::write(
                temp_dir.path().join(format!("script{}.py", i)),
                "def test():\n    x = 1\n",
            )
            .unwrap();
        }

        let analyzer = LanguageAnalyzer::new(temp_dir.path().to_path_buf());
        let secondary = analyzer.secondary_languages().unwrap();

        // Python should be secondary (10-50%)
        let python = secondary.iter().find(|l| l.name == "Python");
        assert!(python.is_some());
    }

    #[test]
    fn test_respects_gitignore() {
        let temp_dir = TempDir::new().unwrap();

        // Initialize git repo
        fs::create_dir(temp_dir.path().join(".git")).unwrap();

        // Create .gitignore
        fs::write(
            temp_dir.path().join(".gitignore"),
            "ignored/\n*.generated.rs\n",
        )
        .unwrap();

        // Create files that should be counted
        fs::write(temp_dir.path().join("main.rs"), "fn main() {}\n").unwrap();

        // Create files that should be ignored
        fs::write(
            temp_dir.path().join("auto.generated.rs"),
            "fn generated() {}\n",
        )
        .unwrap();

        let ignored_dir = temp_dir.path().join("ignored");
        fs::create_dir(&ignored_dir).unwrap();
        fs::write(ignored_dir.join("lib.rs"), "fn ignored() {}\n").unwrap();

        let analyzer = LanguageAnalyzer::new(temp_dir.path().to_path_buf());
        let languages = analyzer.analyze().unwrap();

        let rust = languages.iter().find(|l| l.name == "Rust");
        // Should only count main.rs
        assert!(rust.is_some());
        assert_eq!(rust.unwrap().file_count, 1);
    }

    #[test]
    fn test_language_info_serialization() {
        let info = LanguageInfo {
            name: "Rust".to_string(),
            extensions: vec!["rs".to_string()],
            file_count: 10,
            loc: 500,
            percentage: 75.5,
            support: LanguageSupport::Primary,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"name\":\"Rust\""));
        assert!(json.contains("\"support\":\"primary\""));

        let deserialized: LanguageInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Rust");
        assert_eq!(deserialized.support, LanguageSupport::Primary);
    }
}
