//! File inventory and project structure analysis.

use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use super::AuditResult;

/// Detected project type
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Java,
    Mixed,
    #[default]
    Unknown,
}

/// Purpose classification for directories
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectoryPurpose {
    Source,
    Test,
    Documentation,
    Configuration,
    Build,
    Dependencies,
    Assets,
    #[default]
    Unknown,
}

/// A node in the directory tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryNode {
    /// Directory name
    pub name: String,
    /// Full path
    pub path: PathBuf,
    /// Detected purpose
    pub purpose: DirectoryPurpose,
    /// Child directories
    pub children: Vec<DirectoryNode>,
    /// Number of files in this directory (not recursive)
    pub file_count: usize,
}

/// Key file identified in the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyFile {
    /// File path relative to project root
    pub path: PathBuf,
    /// File type/purpose
    pub file_type: String,
    /// Why this file is considered key
    pub significance: String,
}

/// Complete file inventory for a project
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileInventory {
    /// Detected project type
    pub project_type: ProjectType,
    /// Total file count
    pub total_files: usize,
    /// Total lines of code (estimated)
    pub total_loc: usize,
    /// Files grouped by extension
    pub files_by_extension: HashMap<String, usize>,
    /// Directory structure
    pub structure: Vec<DirectoryNode>,
    /// Key files identified
    pub key_files: Vec<KeyFile>,
}

/// Scanner for building file inventories
pub struct InventoryScanner {
    root: PathBuf,
}

/// Key file patterns and their descriptions
const KEY_FILE_PATTERNS: &[(&str, &str, &str)] = &[
    (
        "README.md",
        "documentation",
        "Primary project documentation",
    ),
    ("README", "documentation", "Primary project documentation"),
    (
        "Cargo.toml",
        "rust_manifest",
        "Rust project configuration and dependencies",
    ),
    (
        "package.json",
        "npm_manifest",
        "Node.js project configuration and dependencies",
    ),
    (
        "pyproject.toml",
        "python_manifest",
        "Python project configuration (PEP 518)",
    ),
    ("setup.py", "python_manifest", "Python package setup script"),
    (
        "requirements.txt",
        "python_deps",
        "Python dependencies list",
    ),
    ("go.mod", "go_manifest", "Go module definition"),
    ("pom.xml", "maven_manifest", "Maven project configuration"),
    (
        "build.gradle",
        "gradle_manifest",
        "Gradle build configuration",
    ),
    ("Makefile", "build_config", "Build automation script"),
    (
        "Dockerfile",
        "container_config",
        "Docker container definition",
    ),
    (
        "docker-compose.yml",
        "container_config",
        "Docker Compose orchestration",
    ),
    (
        "docker-compose.yaml",
        "container_config",
        "Docker Compose orchestration",
    ),
    (".gitignore", "git_config", "Git ignore patterns"),
    (
        ".github/workflows",
        "ci_config",
        "GitHub Actions CI/CD workflows",
    ),
    ("LICENSE", "legal", "Project license"),
    ("LICENSE.md", "legal", "Project license"),
    ("CHANGELOG.md", "documentation", "Project change history"),
    (
        "CONTRIBUTING.md",
        "documentation",
        "Contribution guidelines",
    ),
    (
        ".env.example",
        "config_template",
        "Environment configuration template",
    ),
    (
        "tsconfig.json",
        "typescript_config",
        "TypeScript compiler configuration",
    ),
    (".eslintrc", "linter_config", "ESLint configuration"),
    (".prettierrc", "formatter_config", "Prettier configuration"),
    (
        "rustfmt.toml",
        "formatter_config",
        "Rust formatter configuration",
    ),
    (
        "clippy.toml",
        "linter_config",
        "Clippy linter configuration",
    ),
];

/// Extensions that count as code files for LOC calculation
const CODE_EXTENSIONS: &[&str] = &[
    "rs", "js", "ts", "jsx", "tsx", "py", "go", "java", "c", "h", "cpp", "hpp", "cc", "cxx", "cs",
    "swift", "kt", "kts", "rb", "php", "scala", "clj", "ex", "exs", "erl", "hs", "ml", "mli", "fs",
    "fsi", "lua", "pl", "pm", "r", "R", "jl", "nim", "zig", "v", "sql", "sh", "bash", "zsh",
    "fish", "ps1", "bat", "cmd",
];

impl InventoryScanner {
    /// Create a new inventory scanner
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Scan the project and build inventory
    pub fn scan(&self) -> AuditResult<FileInventory> {
        let mut inventory = FileInventory::default();
        let mut files_by_extension: HashMap<String, usize> = HashMap::new();
        let mut total_loc = 0usize;
        let mut total_files = 0usize;
        let mut dir_file_counts: HashMap<PathBuf, usize> = HashMap::new();

        // Walk the directory tree respecting .gitignore
        let walker = WalkBuilder::new(&self.root)
            .hidden(false) // Include hidden files
            .git_ignore(true) // Respect .gitignore
            .git_global(true) // Respect global gitignore
            .git_exclude(true) // Respect .git/info/exclude
            .build();

        for entry in walker.flatten() {
            let path = entry.path();

            if path.is_file() {
                total_files += 1;

                // Count files by extension
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("(no extension)")
                    .to_lowercase();
                *files_by_extension.entry(ext.clone()).or_insert(0) += 1;

                // Count files per directory
                if let Some(parent) = path.parent() {
                    *dir_file_counts.entry(parent.to_path_buf()).or_insert(0) += 1;
                }

                // Calculate LOC for code files
                if CODE_EXTENSIONS.contains(&ext.as_str()) {
                    total_loc += self.count_lines(path);
                }
            }
        }

        // Identify key files
        let key_files = self.identify_key_files();

        // Detect project type
        let project_type = self.detect_project_type(&files_by_extension, &key_files);

        // Build directory structure
        let structure = self.build_directory_structure(&dir_file_counts)?;

        inventory.project_type = project_type;
        inventory.total_files = total_files;
        inventory.total_loc = total_loc;
        inventory.files_by_extension = files_by_extension;
        inventory.structure = structure;
        inventory.key_files = key_files;

        Ok(inventory)
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

    /// Identify key files in the project
    fn identify_key_files(&self) -> Vec<KeyFile> {
        let mut key_files = Vec::new();

        for (pattern, file_type, significance) in KEY_FILE_PATTERNS {
            let full_path = self.root.join(pattern);
            if full_path.exists() {
                // Store relative path
                let relative_path = PathBuf::from(pattern);
                key_files.push(KeyFile {
                    path: relative_path,
                    file_type: file_type.to_string(),
                    significance: significance.to_string(),
                });
            }
        }

        // Also check for common variations
        self.check_github_workflows(&mut key_files);

        key_files
    }

    /// Check for GitHub workflow files
    fn check_github_workflows(&self, key_files: &mut Vec<KeyFile>) {
        let workflows_dir = self.root.join(".github").join("workflows");
        if workflows_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&workflows_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .extension()
                        .map(|e| e == "yml" || e == "yaml")
                        .unwrap_or(false)
                    {
                        let relative = path.strip_prefix(&self.root).unwrap_or(&path).to_path_buf();
                        key_files.push(KeyFile {
                            path: relative,
                            file_type: "ci_workflow".to_string(),
                            significance: "GitHub Actions workflow definition".to_string(),
                        });
                    }
                }
            }
        }
    }

    /// Detect project type based on files and extensions
    fn detect_project_type(
        &self,
        files_by_extension: &HashMap<String, usize>,
        key_files: &[KeyFile],
    ) -> ProjectType {
        let has_cargo = key_files.iter().any(|f| f.file_type == "rust_manifest");
        let has_package_json = key_files.iter().any(|f| f.file_type == "npm_manifest");
        let has_pyproject = key_files
            .iter()
            .any(|f| f.file_type == "python_manifest" || f.file_type == "python_deps");
        let has_go_mod = key_files.iter().any(|f| f.file_type == "go_manifest");
        let has_pom = key_files.iter().any(|f| f.file_type == "maven_manifest");
        let has_gradle = key_files.iter().any(|f| f.file_type == "gradle_manifest");

        let rs_count = files_by_extension.get("rs").copied().unwrap_or(0);
        let js_count = files_by_extension.get("js").copied().unwrap_or(0);
        let ts_count = files_by_extension.get("ts").copied().unwrap_or(0)
            + files_by_extension.get("tsx").copied().unwrap_or(0);
        let py_count = files_by_extension.get("py").copied().unwrap_or(0);
        let go_count = files_by_extension.get("go").copied().unwrap_or(0);
        let java_count = files_by_extension.get("java").copied().unwrap_or(0);

        // Count how many ecosystems are present
        let mut ecosystems = 0;
        if has_cargo || rs_count > 0 {
            ecosystems += 1;
        }
        if has_package_json || js_count > 0 || ts_count > 0 {
            ecosystems += 1;
        }
        if has_pyproject || py_count > 0 {
            ecosystems += 1;
        }
        if has_go_mod || go_count > 0 {
            ecosystems += 1;
        }
        if has_pom || has_gradle || java_count > 0 {
            ecosystems += 1;
        }

        if ecosystems > 1 {
            return ProjectType::Mixed;
        }

        // Determine single project type
        if has_cargo || rs_count > 0 {
            ProjectType::Rust
        } else if ts_count > js_count {
            ProjectType::TypeScript
        } else if has_package_json || js_count > 0 {
            ProjectType::JavaScript
        } else if has_pyproject || py_count > 0 {
            ProjectType::Python
        } else if has_go_mod || go_count > 0 {
            ProjectType::Go
        } else if has_pom || has_gradle || java_count > 0 {
            ProjectType::Java
        } else {
            ProjectType::Unknown
        }
    }

    /// Build the directory structure tree
    fn build_directory_structure(
        &self,
        dir_file_counts: &HashMap<PathBuf, usize>,
    ) -> AuditResult<Vec<DirectoryNode>> {
        let mut root_children = Vec::new();

        // Get immediate children of root
        let entries = fs::read_dir(&self.root)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                // Skip hidden directories except .github
                if name.starts_with('.') && name != ".github" {
                    continue;
                }

                let purpose = Self::classify_directory(&name);
                let file_count = dir_file_counts.get(&path).copied().unwrap_or(0);

                let node = DirectoryNode {
                    name,
                    path: path.strip_prefix(&self.root).unwrap_or(&path).to_path_buf(),
                    purpose,
                    children: self.build_children(&path, dir_file_counts, 1)?,
                    file_count,
                };

                root_children.push(node);
            }
        }

        // Sort directories alphabetically
        root_children.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(root_children)
    }

    /// Recursively build child directory nodes
    fn build_children(
        &self,
        parent: &Path,
        dir_file_counts: &HashMap<PathBuf, usize>,
        depth: usize,
    ) -> AuditResult<Vec<DirectoryNode>> {
        // Limit recursion depth to avoid deeply nested structures
        if depth > 5 {
            return Ok(Vec::new());
        }

        let mut children = Vec::new();
        let entries = match fs::read_dir(parent) {
            Ok(e) => e,
            Err(_) => return Ok(Vec::new()),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                // Skip hidden directories and common ignored directories
                if name.starts_with('.')
                    || name == "node_modules"
                    || name == "target"
                    || name == "__pycache__"
                    || name == "venv"
                    || name == ".venv"
                {
                    continue;
                }

                let purpose = Self::classify_directory(&name);
                let file_count = dir_file_counts.get(&path).copied().unwrap_or(0);

                let node = DirectoryNode {
                    name,
                    path: path.strip_prefix(&self.root).unwrap_or(&path).to_path_buf(),
                    purpose,
                    children: self.build_children(&path, dir_file_counts, depth + 1)?,
                    file_count,
                };

                children.push(node);
            }
        }

        children.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(children)
    }

    /// Classify a directory's purpose based on its name
    fn classify_directory(name: &str) -> DirectoryPurpose {
        let name_lower = name.to_lowercase();
        match name_lower.as_str() {
            "src" | "lib" | "source" | "sources" | "app" | "pkg" | "cmd" => {
                DirectoryPurpose::Source
            }
            "test" | "tests" | "spec" | "specs" | "__tests__" | "testing" => DirectoryPurpose::Test,
            "doc" | "docs" | "documentation" => DirectoryPurpose::Documentation,
            "config" | "configs" | "configuration" | ".github" | ".vscode" => {
                DirectoryPurpose::Configuration
            }
            "build" | "dist" | "out" | "output" | "target" | "bin" => DirectoryPurpose::Build,
            "vendor" | "vendors" | "node_modules" | "deps" | "dependencies" | "third_party" => {
                DirectoryPurpose::Dependencies
            }
            "assets" | "static" | "public" | "resources" | "images" | "media" => {
                DirectoryPurpose::Assets
            }
            _ => DirectoryPurpose::Unknown,
        }
    }

    /// Get the root path
    pub fn root(&self) -> &PathBuf {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_project_type_default() {
        assert_eq!(ProjectType::default(), ProjectType::Unknown);
    }

    #[test]
    fn test_directory_purpose_default() {
        assert_eq!(DirectoryPurpose::default(), DirectoryPurpose::Unknown);
    }

    #[test]
    fn test_inventory_scanner_new() {
        let scanner = InventoryScanner::new(PathBuf::from("/test"));
        assert_eq!(scanner.root(), &PathBuf::from("/test"));
    }

    #[test]
    fn test_classify_directory_source() {
        assert_eq!(
            InventoryScanner::classify_directory("src"),
            DirectoryPurpose::Source
        );
        assert_eq!(
            InventoryScanner::classify_directory("lib"),
            DirectoryPurpose::Source
        );
    }

    #[test]
    fn test_classify_directory_test() {
        assert_eq!(
            InventoryScanner::classify_directory("tests"),
            DirectoryPurpose::Test
        );
        assert_eq!(
            InventoryScanner::classify_directory("spec"),
            DirectoryPurpose::Test
        );
    }

    #[test]
    fn test_classify_directory_docs() {
        assert_eq!(
            InventoryScanner::classify_directory("docs"),
            DirectoryPurpose::Documentation
        );
    }

    #[test]
    fn test_classify_directory_config() {
        assert_eq!(
            InventoryScanner::classify_directory(".github"),
            DirectoryPurpose::Configuration
        );
    }

    #[test]
    fn test_classify_directory_unknown() {
        assert_eq!(
            InventoryScanner::classify_directory("random_name"),
            DirectoryPurpose::Unknown
        );
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let scanner = InventoryScanner::new(temp_dir.path().to_path_buf());
        let inventory = scanner.scan().unwrap();

        assert_eq!(inventory.total_files, 0);
        assert_eq!(inventory.total_loc, 0);
        assert_eq!(inventory.project_type, ProjectType::Unknown);
    }

    #[test]
    fn test_scan_rust_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create Cargo.toml
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        // Create src directory with a Rust file
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(
            src_dir.join("main.rs"),
            "fn main() {\n    println!(\"Hello\");\n}\n",
        )
        .unwrap();

        let scanner = InventoryScanner::new(temp_dir.path().to_path_buf());
        let inventory = scanner.scan().unwrap();

        assert_eq!(inventory.project_type, ProjectType::Rust);
        assert_eq!(inventory.total_files, 2); // Cargo.toml + main.rs
        assert_eq!(inventory.total_loc, 3); // 3 lines in main.rs
        assert_eq!(inventory.files_by_extension.get("rs"), Some(&1));
        assert_eq!(inventory.files_by_extension.get("toml"), Some(&1));

        // Check key files
        assert!(inventory
            .key_files
            .iter()
            .any(|f| f.path == PathBuf::from("Cargo.toml")));
    }

    #[test]
    fn test_scan_respects_gitignore() {
        let temp_dir = TempDir::new().unwrap();

        // Initialize a git repo so .gitignore is respected
        fs::create_dir(temp_dir.path().join(".git")).unwrap();

        // Create .gitignore
        fs::write(temp_dir.path().join(".gitignore"), "*.log\nignored_dir/\n").unwrap();

        // Create files that should be counted
        fs::write(temp_dir.path().join("included.rs"), "fn test() {}\n").unwrap();

        // Create files that should be ignored
        fs::write(temp_dir.path().join("debug.log"), "log content").unwrap();

        // Create ignored directory
        let ignored_dir = temp_dir.path().join("ignored_dir");
        fs::create_dir(&ignored_dir).unwrap();
        fs::write(ignored_dir.join("file.rs"), "fn ignored() {}").unwrap();

        let scanner = InventoryScanner::new(temp_dir.path().to_path_buf());
        let inventory = scanner.scan().unwrap();

        // Should count .gitignore and included.rs, but not debug.log or ignored_dir/file.rs
        assert_eq!(inventory.total_files, 2);
        assert!(!inventory.files_by_extension.contains_key("log"));
    }

    #[test]
    fn test_scan_mixed_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create Cargo.toml (Rust)
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();

        // Create package.json (JavaScript/Node)
        fs::write(
            temp_dir.path().join("package.json"),
            "{\"name\": \"test\"}\n",
        )
        .unwrap();

        // Create source files
        fs::write(temp_dir.path().join("lib.rs"), "pub fn test() {}").unwrap();
        fs::write(temp_dir.path().join("index.js"), "console.log('test');").unwrap();

        let scanner = InventoryScanner::new(temp_dir.path().to_path_buf());
        let inventory = scanner.scan().unwrap();

        assert_eq!(inventory.project_type, ProjectType::Mixed);
    }

    #[test]
    fn test_scan_javascript_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create package.json
        fs::write(
            temp_dir.path().join("package.json"),
            "{\"name\": \"test\"}\n",
        )
        .unwrap();

        // Create JS files
        fs::write(temp_dir.path().join("index.js"), "console.log('test');").unwrap();
        fs::write(temp_dir.path().join("utils.js"), "export const foo = 1;").unwrap();

        let scanner = InventoryScanner::new(temp_dir.path().to_path_buf());
        let inventory = scanner.scan().unwrap();

        assert_eq!(inventory.project_type, ProjectType::JavaScript);
        assert_eq!(inventory.files_by_extension.get("js"), Some(&2));
    }

    #[test]
    fn test_scan_typescript_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create package.json
        fs::write(
            temp_dir.path().join("package.json"),
            "{\"name\": \"test\"}\n",
        )
        .unwrap();

        // Create TS files (more than JS)
        fs::write(temp_dir.path().join("index.ts"), "console.log('test');").unwrap();
        fs::write(temp_dir.path().join("utils.ts"), "export const foo = 1;").unwrap();
        fs::write(temp_dir.path().join("app.tsx"), "const App = () => {};").unwrap();

        let scanner = InventoryScanner::new(temp_dir.path().to_path_buf());
        let inventory = scanner.scan().unwrap();

        assert_eq!(inventory.project_type, ProjectType::TypeScript);
    }

    #[test]
    fn test_scan_python_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create requirements.txt
        fs::write(temp_dir.path().join("requirements.txt"), "flask==2.0\n").unwrap();

        // Create Python files
        fs::write(temp_dir.path().join("app.py"), "print('hello')").unwrap();

        let scanner = InventoryScanner::new(temp_dir.path().to_path_buf());
        let inventory = scanner.scan().unwrap();

        assert_eq!(inventory.project_type, ProjectType::Python);
    }

    #[test]
    fn test_scan_go_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create go.mod
        fs::write(
            temp_dir.path().join("go.mod"),
            "module example.com/test\n\ngo 1.20\n",
        )
        .unwrap();

        // Create Go files
        fs::write(
            temp_dir.path().join("main.go"),
            "package main\n\nfunc main() {}",
        )
        .unwrap();

        let scanner = InventoryScanner::new(temp_dir.path().to_path_buf());
        let inventory = scanner.scan().unwrap();

        assert_eq!(inventory.project_type, ProjectType::Go);
    }

    #[test]
    fn test_count_lines() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        fs::write(&file_path, "line1\nline2\nline3\n").unwrap();

        let scanner = InventoryScanner::new(temp_dir.path().to_path_buf());
        assert_eq!(scanner.count_lines(&file_path), 3);
    }

    #[test]
    fn test_directory_structure() {
        let temp_dir = TempDir::new().unwrap();

        // Create directory structure
        let src_dir = temp_dir.path().join("src");
        let tests_dir = temp_dir.path().join("tests");
        let docs_dir = temp_dir.path().join("docs");

        fs::create_dir(&src_dir).unwrap();
        fs::create_dir(&tests_dir).unwrap();
        fs::create_dir(&docs_dir).unwrap();

        // Add files
        fs::write(src_dir.join("lib.rs"), "pub fn test() {}").unwrap();
        fs::write(tests_dir.join("test.rs"), "#[test] fn it_works() {}").unwrap();
        fs::write(docs_dir.join("readme.md"), "# Docs").unwrap();

        let scanner = InventoryScanner::new(temp_dir.path().to_path_buf());
        let inventory = scanner.scan().unwrap();

        assert!(!inventory.structure.is_empty());

        // Check that directories are properly classified
        let src_node = inventory.structure.iter().find(|n| n.name == "src");
        assert!(src_node.is_some());
        assert_eq!(src_node.unwrap().purpose, DirectoryPurpose::Source);

        let tests_node = inventory.structure.iter().find(|n| n.name == "tests");
        assert!(tests_node.is_some());
        assert_eq!(tests_node.unwrap().purpose, DirectoryPurpose::Test);

        let docs_node = inventory.structure.iter().find(|n| n.name == "docs");
        assert!(docs_node.is_some());
        assert_eq!(docs_node.unwrap().purpose, DirectoryPurpose::Documentation);
    }

    #[test]
    fn test_key_files_detection() {
        let temp_dir = TempDir::new().unwrap();

        // Create various key files
        fs::write(temp_dir.path().join("README.md"), "# Project").unwrap();
        fs::write(temp_dir.path().join("LICENSE"), "MIT").unwrap();
        fs::write(temp_dir.path().join("Dockerfile"), "FROM rust").unwrap();

        let scanner = InventoryScanner::new(temp_dir.path().to_path_buf());
        let inventory = scanner.scan().unwrap();

        assert!(inventory
            .key_files
            .iter()
            .any(|f| f.path == PathBuf::from("README.md")));
        assert!(inventory
            .key_files
            .iter()
            .any(|f| f.path == PathBuf::from("LICENSE")));
        assert!(inventory
            .key_files
            .iter()
            .any(|f| f.path == PathBuf::from("Dockerfile")));
    }

    #[test]
    fn test_github_workflows_detection() {
        let temp_dir = TempDir::new().unwrap();

        // Create .github/workflows directory
        let workflows_dir = temp_dir.path().join(".github").join("workflows");
        fs::create_dir_all(&workflows_dir).unwrap();
        fs::write(workflows_dir.join("ci.yml"), "name: CI\non: push\n").unwrap();
        fs::write(
            workflows_dir.join("release.yaml"),
            "name: Release\non: push\n",
        )
        .unwrap();

        let scanner = InventoryScanner::new(temp_dir.path().to_path_buf());
        let inventory = scanner.scan().unwrap();

        let workflow_files: Vec<_> = inventory
            .key_files
            .iter()
            .filter(|f| f.file_type == "ci_workflow")
            .collect();
        assert_eq!(workflow_files.len(), 2);
    }
}
