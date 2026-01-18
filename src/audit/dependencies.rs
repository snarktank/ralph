//! Dependency analysis and parsing.

use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::AuditResult;

/// Supported dependency ecosystems
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyEcosystem {
    Cargo,
    Npm,
    Pip,
    Go,
    Maven,
    Gradle,
    #[default]
    Unknown,
}

impl std::fmt::Display for DependencyEcosystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyEcosystem::Cargo => write!(f, "cargo"),
            DependencyEcosystem::Npm => write!(f, "npm"),
            DependencyEcosystem::Pip => write!(f, "pip"),
            DependencyEcosystem::Go => write!(f, "go"),
            DependencyEcosystem::Maven => write!(f, "maven"),
            DependencyEcosystem::Gradle => write!(f, "gradle"),
            DependencyEcosystem::Unknown => write!(f, "unknown"),
        }
    }
}

/// Information about an outdated dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedInfo {
    /// Latest available version
    pub latest_version: String,
    /// Whether this is a major version bump
    pub is_major_bump: bool,
    /// Security advisory if any
    pub security_advisory: Option<String>,
}

/// A single dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Package name
    pub name: String,
    /// Current version
    pub version: String,
    /// Ecosystem this dependency belongs to
    pub ecosystem: DependencyEcosystem,
    /// Whether this is a dev/test dependency
    pub is_dev: bool,
    /// Path to manifest file
    pub manifest_path: PathBuf,
    /// Outdated info if available
    pub outdated: Option<OutdatedInfo>,
}

/// Complete dependency analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    /// All detected dependencies
    pub dependencies: Vec<Dependency>,
    /// Count by ecosystem
    pub ecosystem_counts: Vec<(DependencyEcosystem, usize)>,
    /// Number of outdated dependencies
    pub outdated_count: usize,
    /// Number of dependencies with security advisories
    pub vulnerable_count: usize,
}

impl DependencyAnalysis {
    /// Get direct (non-dev) dependencies
    pub fn direct_dependencies(&self) -> Vec<&Dependency> {
        self.dependencies.iter().filter(|d| !d.is_dev).collect()
    }

    /// Get dev dependencies
    pub fn dev_dependencies(&self) -> Vec<&Dependency> {
        self.dependencies.iter().filter(|d| d.is_dev).collect()
    }

    /// Get dependencies by ecosystem
    pub fn by_ecosystem(&self, ecosystem: &DependencyEcosystem) -> Vec<&Dependency> {
        self.dependencies
            .iter()
            .filter(|d| &d.ecosystem == ecosystem)
            .collect()
    }
}

/// Parser for extracting dependencies from manifest files
pub struct DependencyParser {
    root: PathBuf,
}

impl DependencyParser {
    /// Create a new dependency parser
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Parse all dependencies in the project
    pub fn parse(&self) -> AuditResult<DependencyAnalysis> {
        let mut dependencies = Vec::new();

        // Walk the directory tree to find all manifest files
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

            let filename = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name,
                None => continue,
            };

            let parsed = match filename {
                "Cargo.toml" => self.parse_cargo_toml(path),
                "package.json" => self.parse_package_json(path),
                "pyproject.toml" => self.parse_pyproject_toml(path),
                "requirements.txt" => self.parse_requirements_txt(path),
                "go.mod" => self.parse_go_mod(path),
                _ => continue,
            };

            if let Ok(mut deps) = parsed {
                dependencies.append(&mut deps);
            }
        }

        // Calculate ecosystem counts
        let mut ecosystem_map: HashMap<DependencyEcosystem, usize> = HashMap::new();
        for dep in &dependencies {
            *ecosystem_map.entry(dep.ecosystem.clone()).or_insert(0) += 1;
        }
        let ecosystem_counts: Vec<_> = ecosystem_map.into_iter().collect();

        // Calculate outdated and vulnerable counts
        let outdated_count = dependencies.iter().filter(|d| d.outdated.is_some()).count();
        let vulnerable_count = dependencies
            .iter()
            .filter(|d| {
                d.outdated
                    .as_ref()
                    .is_some_and(|o| o.security_advisory.is_some())
            })
            .count();

        Ok(DependencyAnalysis {
            dependencies,
            ecosystem_counts,
            outdated_count,
            vulnerable_count,
        })
    }

    /// Get the root path
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Parse Cargo.toml for Rust dependencies
    fn parse_cargo_toml(&self, path: &Path) -> AuditResult<Vec<Dependency>> {
        let content = fs::read_to_string(path)?;
        let value: toml::Value = toml::from_str(&content)
            .map_err(|e| super::AuditError::ParseError(format!("Invalid Cargo.toml: {}", e)))?;

        let mut dependencies = Vec::new();
        let manifest_path = path.to_path_buf();

        // Parse [dependencies]
        if let Some(deps) = value.get("dependencies").and_then(|v| v.as_table()) {
            for (name, spec) in deps {
                let version = Self::extract_cargo_version(spec);
                dependencies.push(Dependency {
                    name: name.clone(),
                    version,
                    ecosystem: DependencyEcosystem::Cargo,
                    is_dev: false,
                    manifest_path: manifest_path.clone(),
                    outdated: None,
                });
            }
        }

        // Parse [dev-dependencies]
        if let Some(deps) = value.get("dev-dependencies").and_then(|v| v.as_table()) {
            for (name, spec) in deps {
                let version = Self::extract_cargo_version(spec);
                dependencies.push(Dependency {
                    name: name.clone(),
                    version,
                    ecosystem: DependencyEcosystem::Cargo,
                    is_dev: true,
                    manifest_path: manifest_path.clone(),
                    outdated: None,
                });
            }
        }

        // Parse [build-dependencies]
        if let Some(deps) = value.get("build-dependencies").and_then(|v| v.as_table()) {
            for (name, spec) in deps {
                let version = Self::extract_cargo_version(spec);
                dependencies.push(Dependency {
                    name: name.clone(),
                    version,
                    ecosystem: DependencyEcosystem::Cargo,
                    is_dev: true, // Treat build deps as dev deps
                    manifest_path: manifest_path.clone(),
                    outdated: None,
                });
            }
        }

        Ok(dependencies)
    }

    /// Extract version string from Cargo.toml dependency specification
    fn extract_cargo_version(spec: &toml::Value) -> String {
        match spec {
            toml::Value::String(v) => v.clone(),
            toml::Value::Table(t) => t
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("*")
                .to_string(),
            _ => "*".to_string(),
        }
    }

    /// Parse package.json for npm dependencies
    fn parse_package_json(&self, path: &Path) -> AuditResult<Vec<Dependency>> {
        let content = fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| super::AuditError::ParseError(format!("Invalid package.json: {}", e)))?;

        let mut dependencies = Vec::new();
        let manifest_path = path.to_path_buf();

        // Parse "dependencies"
        if let Some(deps) = value.get("dependencies").and_then(|v| v.as_object()) {
            for (name, version) in deps {
                dependencies.push(Dependency {
                    name: name.clone(),
                    version: version.as_str().unwrap_or("*").to_string(),
                    ecosystem: DependencyEcosystem::Npm,
                    is_dev: false,
                    manifest_path: manifest_path.clone(),
                    outdated: None,
                });
            }
        }

        // Parse "devDependencies"
        if let Some(deps) = value.get("devDependencies").and_then(|v| v.as_object()) {
            for (name, version) in deps {
                dependencies.push(Dependency {
                    name: name.clone(),
                    version: version.as_str().unwrap_or("*").to_string(),
                    ecosystem: DependencyEcosystem::Npm,
                    is_dev: true,
                    manifest_path: manifest_path.clone(),
                    outdated: None,
                });
            }
        }

        // Parse "peerDependencies" (treat as direct deps)
        if let Some(deps) = value.get("peerDependencies").and_then(|v| v.as_object()) {
            for (name, version) in deps {
                dependencies.push(Dependency {
                    name: name.clone(),
                    version: version.as_str().unwrap_or("*").to_string(),
                    ecosystem: DependencyEcosystem::Npm,
                    is_dev: false,
                    manifest_path: manifest_path.clone(),
                    outdated: None,
                });
            }
        }

        // Parse "optionalDependencies" (treat as direct deps)
        if let Some(deps) = value
            .get("optionalDependencies")
            .and_then(|v| v.as_object())
        {
            for (name, version) in deps {
                dependencies.push(Dependency {
                    name: name.clone(),
                    version: version.as_str().unwrap_or("*").to_string(),
                    ecosystem: DependencyEcosystem::Npm,
                    is_dev: false,
                    manifest_path: manifest_path.clone(),
                    outdated: None,
                });
            }
        }

        Ok(dependencies)
    }

    /// Parse pyproject.toml for Python dependencies
    fn parse_pyproject_toml(&self, path: &Path) -> AuditResult<Vec<Dependency>> {
        let content = fs::read_to_string(path)?;
        let value: toml::Value = toml::from_str(&content)
            .map_err(|e| super::AuditError::ParseError(format!("Invalid pyproject.toml: {}", e)))?;

        let mut dependencies = Vec::new();
        let manifest_path = path.to_path_buf();

        // Parse [project.dependencies] (PEP 621)
        if let Some(deps) = value
            .get("project")
            .and_then(|p| p.get("dependencies"))
            .and_then(|d| d.as_array())
        {
            for dep in deps {
                if let Some(dep_str) = dep.as_str() {
                    let (name, version) = Self::parse_python_dependency(dep_str);
                    dependencies.push(Dependency {
                        name,
                        version,
                        ecosystem: DependencyEcosystem::Pip,
                        is_dev: false,
                        manifest_path: manifest_path.clone(),
                        outdated: None,
                    });
                }
            }
        }

        // Parse [project.optional-dependencies] for dev dependencies
        if let Some(optional) = value
            .get("project")
            .and_then(|p| p.get("optional-dependencies"))
            .and_then(|d| d.as_table())
        {
            for (group, deps) in optional {
                let is_dev = group == "dev" || group == "test" || group == "testing";
                if let Some(deps_array) = deps.as_array() {
                    for dep in deps_array {
                        if let Some(dep_str) = dep.as_str() {
                            let (name, version) = Self::parse_python_dependency(dep_str);
                            dependencies.push(Dependency {
                                name,
                                version,
                                ecosystem: DependencyEcosystem::Pip,
                                is_dev,
                                manifest_path: manifest_path.clone(),
                                outdated: None,
                            });
                        }
                    }
                }
            }
        }

        // Parse [tool.poetry.dependencies] (Poetry)
        if let Some(deps) = value
            .get("tool")
            .and_then(|t| t.get("poetry"))
            .and_then(|p| p.get("dependencies"))
            .and_then(|d| d.as_table())
        {
            for (name, spec) in deps {
                if name == "python" {
                    continue; // Skip Python version constraint
                }
                let version = Self::extract_poetry_version(spec);
                dependencies.push(Dependency {
                    name: name.clone(),
                    version,
                    ecosystem: DependencyEcosystem::Pip,
                    is_dev: false,
                    manifest_path: manifest_path.clone(),
                    outdated: None,
                });
            }
        }

        // Parse [tool.poetry.dev-dependencies] (Poetry)
        if let Some(deps) = value
            .get("tool")
            .and_then(|t| t.get("poetry"))
            .and_then(|p| p.get("dev-dependencies"))
            .and_then(|d| d.as_table())
        {
            for (name, spec) in deps {
                let version = Self::extract_poetry_version(spec);
                dependencies.push(Dependency {
                    name: name.clone(),
                    version,
                    ecosystem: DependencyEcosystem::Pip,
                    is_dev: true,
                    manifest_path: manifest_path.clone(),
                    outdated: None,
                });
            }
        }

        // Parse [tool.poetry.group.*.dependencies] (Poetry 1.2+)
        if let Some(groups) = value
            .get("tool")
            .and_then(|t| t.get("poetry"))
            .and_then(|p| p.get("group"))
            .and_then(|g| g.as_table())
        {
            for (group_name, group) in groups {
                let is_dev = group_name == "dev" || group_name == "test";
                if let Some(deps) = group.get("dependencies").and_then(|d| d.as_table()) {
                    for (name, spec) in deps {
                        let version = Self::extract_poetry_version(spec);
                        dependencies.push(Dependency {
                            name: name.clone(),
                            version,
                            ecosystem: DependencyEcosystem::Pip,
                            is_dev,
                            manifest_path: manifest_path.clone(),
                            outdated: None,
                        });
                    }
                }
            }
        }

        Ok(dependencies)
    }

    /// Extract version from Poetry dependency specification
    fn extract_poetry_version(spec: &toml::Value) -> String {
        match spec {
            toml::Value::String(v) => v.clone(),
            toml::Value::Table(t) => t
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("*")
                .to_string(),
            _ => "*".to_string(),
        }
    }

    /// Parse Python dependency string (e.g., "requests>=2.28.0")
    fn parse_python_dependency(dep_str: &str) -> (String, String) {
        // Match patterns like: package>=1.0, package==1.0, package~=1.0, package[extra]>=1.0
        let re = Regex::new(r"^([a-zA-Z0-9_-]+)(?:\[[^\]]+\])?\s*([<>=!~]+.+)?$").unwrap();

        if let Some(caps) = re.captures(dep_str.trim()) {
            let name = caps.get(1).map(|m| m.as_str()).unwrap_or(dep_str);
            let version = caps
                .get(2)
                .map(|m| m.as_str().trim())
                .unwrap_or("*")
                .to_string();
            (name.to_string(), version)
        } else {
            (dep_str.to_string(), "*".to_string())
        }
    }

    /// Parse requirements.txt for Python dependencies
    fn parse_requirements_txt(&self, path: &Path) -> AuditResult<Vec<Dependency>> {
        let content = fs::read_to_string(path)?;
        let mut dependencies = Vec::new();
        let manifest_path = path.to_path_buf();

        // Check if this is a dev requirements file
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let is_dev = filename.contains("dev") || filename.contains("test");

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Skip -r includes and other options
            if line.starts_with('-') {
                continue;
            }

            let (name, version) = Self::parse_python_dependency(line);
            dependencies.push(Dependency {
                name,
                version,
                ecosystem: DependencyEcosystem::Pip,
                is_dev,
                manifest_path: manifest_path.clone(),
                outdated: None,
            });
        }

        Ok(dependencies)
    }

    /// Parse go.mod for Go dependencies
    fn parse_go_mod(&self, path: &Path) -> AuditResult<Vec<Dependency>> {
        let content = fs::read_to_string(path)?;
        let mut dependencies = Vec::new();
        let manifest_path = path.to_path_buf();

        // Match require directives
        // Single: require github.com/pkg/errors v0.9.1
        // Block: require (\n\tgithub.com/pkg/errors v0.9.1\n)
        let single_require = Regex::new(r"^require\s+(\S+)\s+(\S+)").unwrap();
        let block_require = Regex::new(r"^\s*(\S+)\s+(\S+)(?:\s*//.*)?$").unwrap();

        let mut in_require_block = false;

        for line in content.lines() {
            let line = line.trim();

            // Check for block start
            if line.starts_with("require (") || line == "require(" {
                in_require_block = true;
                continue;
            }

            // Check for block end
            if in_require_block && line == ")" {
                in_require_block = false;
                continue;
            }

            if in_require_block {
                // Skip comments and indirect dependencies
                if line.starts_with("//") || line.contains("// indirect") {
                    if let Some(caps) = block_require.captures(line) {
                        let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                        let version = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                        if !name.is_empty() {
                            dependencies.push(Dependency {
                                name: name.to_string(),
                                version: version.to_string(),
                                ecosystem: DependencyEcosystem::Go,
                                is_dev: true, // Treat indirect as dev
                                manifest_path: manifest_path.clone(),
                                outdated: None,
                            });
                        }
                    }
                    continue;
                }

                if let Some(caps) = block_require.captures(line) {
                    let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    let version = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                    if !name.is_empty() {
                        dependencies.push(Dependency {
                            name: name.to_string(),
                            version: version.to_string(),
                            ecosystem: DependencyEcosystem::Go,
                            is_dev: false,
                            manifest_path: manifest_path.clone(),
                            outdated: None,
                        });
                    }
                }
            } else if let Some(caps) = single_require.captures(line) {
                let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let version = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                if !name.is_empty() {
                    dependencies.push(Dependency {
                        name: name.to_string(),
                        version: version.to_string(),
                        ecosystem: DependencyEcosystem::Go,
                        is_dev: false,
                        manifest_path: manifest_path.clone(),
                        outdated: None,
                    });
                }
            }
        }

        Ok(dependencies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_ecosystem_default() {
        assert_eq!(DependencyEcosystem::default(), DependencyEcosystem::Unknown);
    }

    #[test]
    fn test_ecosystem_display() {
        assert_eq!(format!("{}", DependencyEcosystem::Cargo), "cargo");
        assert_eq!(format!("{}", DependencyEcosystem::Npm), "npm");
        assert_eq!(format!("{}", DependencyEcosystem::Pip), "pip");
        assert_eq!(format!("{}", DependencyEcosystem::Go), "go");
    }

    #[test]
    fn test_dependency_parser_new() {
        let parser = DependencyParser::new(PathBuf::from("/test"));
        assert_eq!(parser.root(), &PathBuf::from("/test"));
    }

    #[test]
    fn test_dependency_analysis_direct_and_dev() {
        let analysis = DependencyAnalysis {
            dependencies: vec![
                Dependency {
                    name: "serde".to_string(),
                    version: "1.0".to_string(),
                    ecosystem: DependencyEcosystem::Cargo,
                    is_dev: false,
                    manifest_path: PathBuf::from("Cargo.toml"),
                    outdated: None,
                },
                Dependency {
                    name: "tempfile".to_string(),
                    version: "3.0".to_string(),
                    ecosystem: DependencyEcosystem::Cargo,
                    is_dev: true,
                    manifest_path: PathBuf::from("Cargo.toml"),
                    outdated: None,
                },
            ],
            ecosystem_counts: vec![(DependencyEcosystem::Cargo, 2)],
            outdated_count: 0,
            vulnerable_count: 0,
        };

        let direct = analysis.direct_dependencies();
        assert_eq!(direct.len(), 1);
        assert_eq!(direct[0].name, "serde");

        let dev = analysis.dev_dependencies();
        assert_eq!(dev.len(), 1);
        assert_eq!(dev[0].name, "tempfile");
    }

    #[test]
    fn test_dependency_analysis_by_ecosystem() {
        let analysis = DependencyAnalysis {
            dependencies: vec![
                Dependency {
                    name: "serde".to_string(),
                    version: "1.0".to_string(),
                    ecosystem: DependencyEcosystem::Cargo,
                    is_dev: false,
                    manifest_path: PathBuf::from("Cargo.toml"),
                    outdated: None,
                },
                Dependency {
                    name: "react".to_string(),
                    version: "18.0".to_string(),
                    ecosystem: DependencyEcosystem::Npm,
                    is_dev: false,
                    manifest_path: PathBuf::from("package.json"),
                    outdated: None,
                },
            ],
            ecosystem_counts: vec![
                (DependencyEcosystem::Cargo, 1),
                (DependencyEcosystem::Npm, 1),
            ],
            outdated_count: 0,
            vulnerable_count: 0,
        };

        let cargo_deps = analysis.by_ecosystem(&DependencyEcosystem::Cargo);
        assert_eq!(cargo_deps.len(), 1);
        assert_eq!(cargo_deps[0].name, "serde");

        let npm_deps = analysis.by_ecosystem(&DependencyEcosystem::Npm);
        assert_eq!(npm_deps.len(), 1);
        assert_eq!(npm_deps[0].name, "react");
    }

    #[test]
    fn test_parse_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");

        fs::write(
            &cargo_toml,
            r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
tempfile = "3.0"

[build-dependencies]
cc = "1.0"
"#,
        )
        .unwrap();

        let parser = DependencyParser::new(temp_dir.path().to_path_buf());
        let deps = parser.parse_cargo_toml(&cargo_toml).unwrap();

        assert_eq!(deps.len(), 4);

        let serde = deps.iter().find(|d| d.name == "serde").unwrap();
        assert_eq!(serde.version, "1.0");
        assert!(!serde.is_dev);
        assert_eq!(serde.ecosystem, DependencyEcosystem::Cargo);

        let tokio = deps.iter().find(|d| d.name == "tokio").unwrap();
        assert_eq!(tokio.version, "1.0");
        assert!(!tokio.is_dev);

        let tempfile = deps.iter().find(|d| d.name == "tempfile").unwrap();
        assert!(tempfile.is_dev);

        let cc = deps.iter().find(|d| d.name == "cc").unwrap();
        assert!(cc.is_dev); // build deps treated as dev
    }

    #[test]
    fn test_parse_package_json() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = temp_dir.path().join("package.json");

        fs::write(
            &package_json,
            r#"{
  "name": "test",
  "version": "1.0.0",
  "dependencies": {
    "react": "^18.2.0",
    "lodash": "4.17.21"
  },
  "devDependencies": {
    "typescript": "^5.0.0",
    "jest": "^29.0.0"
  },
  "peerDependencies": {
    "react-dom": "^18.0.0"
  }
}"#,
        )
        .unwrap();

        let parser = DependencyParser::new(temp_dir.path().to_path_buf());
        let deps = parser.parse_package_json(&package_json).unwrap();

        assert_eq!(deps.len(), 5);

        let react = deps.iter().find(|d| d.name == "react").unwrap();
        assert_eq!(react.version, "^18.2.0");
        assert!(!react.is_dev);
        assert_eq!(react.ecosystem, DependencyEcosystem::Npm);

        let typescript = deps.iter().find(|d| d.name == "typescript").unwrap();
        assert!(typescript.is_dev);

        let react_dom = deps.iter().find(|d| d.name == "react-dom").unwrap();
        assert!(!react_dom.is_dev); // peer deps are not dev
    }

    #[test]
    fn test_parse_pyproject_toml_pep621() {
        let temp_dir = TempDir::new().unwrap();
        let pyproject = temp_dir.path().join("pyproject.toml");

        fs::write(
            &pyproject,
            r#"
[project]
name = "test"
version = "0.1.0"
dependencies = [
    "requests>=2.28.0",
    "click~=8.0",
    "pydantic[email]>=1.10",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.0",
    "black>=23.0",
]
test = [
    "coverage>=7.0",
]
"#,
        )
        .unwrap();

        let parser = DependencyParser::new(temp_dir.path().to_path_buf());
        let deps = parser.parse_pyproject_toml(&pyproject).unwrap();

        assert_eq!(deps.len(), 6);

        let requests = deps.iter().find(|d| d.name == "requests").unwrap();
        assert_eq!(requests.version, ">=2.28.0");
        assert!(!requests.is_dev);
        assert_eq!(requests.ecosystem, DependencyEcosystem::Pip);

        let pydantic = deps.iter().find(|d| d.name == "pydantic").unwrap();
        assert_eq!(pydantic.version, ">=1.10");
        assert!(!pydantic.is_dev);

        let pytest = deps.iter().find(|d| d.name == "pytest").unwrap();
        assert!(pytest.is_dev);

        let coverage = deps.iter().find(|d| d.name == "coverage").unwrap();
        assert!(coverage.is_dev); // test group is dev
    }

    #[test]
    fn test_parse_pyproject_toml_poetry() {
        let temp_dir = TempDir::new().unwrap();
        let pyproject = temp_dir.path().join("pyproject.toml");

        fs::write(
            &pyproject,
            r#"
[tool.poetry]
name = "test"
version = "0.1.0"

[tool.poetry.dependencies]
python = "^3.9"
requests = "^2.28.0"
flask = { version = "^2.0", extras = ["async"] }

[tool.poetry.dev-dependencies]
pytest = "^7.0"

[tool.poetry.group.test.dependencies]
coverage = "^7.0"
"#,
        )
        .unwrap();

        let parser = DependencyParser::new(temp_dir.path().to_path_buf());
        let deps = parser.parse_pyproject_toml(&pyproject).unwrap();

        assert_eq!(deps.len(), 4); // python is skipped

        let requests = deps.iter().find(|d| d.name == "requests").unwrap();
        assert_eq!(requests.version, "^2.28.0");
        assert!(!requests.is_dev);

        let flask = deps.iter().find(|d| d.name == "flask").unwrap();
        assert_eq!(flask.version, "^2.0");
        assert!(!flask.is_dev);

        let pytest = deps.iter().find(|d| d.name == "pytest").unwrap();
        assert!(pytest.is_dev);

        let coverage = deps.iter().find(|d| d.name == "coverage").unwrap();
        assert!(coverage.is_dev);
    }

    #[test]
    fn test_parse_requirements_txt() {
        let temp_dir = TempDir::new().unwrap();
        let requirements = temp_dir.path().join("requirements.txt");

        fs::write(
            &requirements,
            r#"
# This is a comment
requests>=2.28.0
flask==2.0.0
pydantic~=1.10

-r base.txt
click
"#,
        )
        .unwrap();

        let parser = DependencyParser::new(temp_dir.path().to_path_buf());
        let deps = parser.parse_requirements_txt(&requirements).unwrap();

        assert_eq!(deps.len(), 4);

        let requests = deps.iter().find(|d| d.name == "requests").unwrap();
        assert_eq!(requests.version, ">=2.28.0");
        assert!(!requests.is_dev);
        assert_eq!(requests.ecosystem, DependencyEcosystem::Pip);

        let flask = deps.iter().find(|d| d.name == "flask").unwrap();
        assert_eq!(flask.version, "==2.0.0");

        let click = deps.iter().find(|d| d.name == "click").unwrap();
        assert_eq!(click.version, "*");
    }

    #[test]
    fn test_parse_requirements_dev_txt() {
        let temp_dir = TempDir::new().unwrap();
        let requirements = temp_dir.path().join("requirements-dev.txt");

        fs::write(
            &requirements,
            r#"
pytest>=7.0
black>=23.0
"#,
        )
        .unwrap();

        let parser = DependencyParser::new(temp_dir.path().to_path_buf());
        let deps = parser.parse_requirements_txt(&requirements).unwrap();

        assert_eq!(deps.len(), 2);

        // All deps from dev requirements file should be dev
        for dep in &deps {
            assert!(dep.is_dev);
        }
    }

    #[test]
    fn test_parse_go_mod() {
        let temp_dir = TempDir::new().unwrap();
        let go_mod = temp_dir.path().join("go.mod");

        fs::write(
            &go_mod,
            r#"
module example.com/myproject

go 1.21

require github.com/pkg/errors v0.9.1

require (
	github.com/gin-gonic/gin v1.9.0
	github.com/stretchr/testify v1.8.0
	golang.org/x/sys v0.5.0 // indirect
)
"#,
        )
        .unwrap();

        let parser = DependencyParser::new(temp_dir.path().to_path_buf());
        let deps = parser.parse_go_mod(&go_mod).unwrap();

        assert_eq!(deps.len(), 4);

        let errors = deps
            .iter()
            .find(|d| d.name == "github.com/pkg/errors")
            .unwrap();
        assert_eq!(errors.version, "v0.9.1");
        assert!(!errors.is_dev);
        assert_eq!(errors.ecosystem, DependencyEcosystem::Go);

        let gin = deps
            .iter()
            .find(|d| d.name == "github.com/gin-gonic/gin")
            .unwrap();
        assert_eq!(gin.version, "v1.9.0");
        assert!(!gin.is_dev);

        let sys = deps.iter().find(|d| d.name == "golang.org/x/sys").unwrap();
        assert!(sys.is_dev); // indirect deps treated as dev
    }

    #[test]
    fn test_parse_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let parser = DependencyParser::new(temp_dir.path().to_path_buf());
        let analysis = parser.parse().unwrap();

        assert!(analysis.dependencies.is_empty());
        assert!(analysis.ecosystem_counts.is_empty());
        assert_eq!(analysis.outdated_count, 0);
        assert_eq!(analysis.vulnerable_count, 0);
    }

    #[test]
    fn test_parse_mixed_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create Cargo.toml
        fs::write(
            temp_dir.path().join("Cargo.toml"),
            r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#,
        )
        .unwrap();

        // Create package.json in a subdirectory
        let frontend_dir = temp_dir.path().join("frontend");
        fs::create_dir(&frontend_dir).unwrap();
        fs::write(
            frontend_dir.join("package.json"),
            r#"{
  "name": "frontend",
  "dependencies": {
    "react": "^18.0.0"
  }
}"#,
        )
        .unwrap();

        let parser = DependencyParser::new(temp_dir.path().to_path_buf());
        let analysis = parser.parse().unwrap();

        assert_eq!(analysis.dependencies.len(), 2);

        let cargo_deps = analysis.by_ecosystem(&DependencyEcosystem::Cargo);
        assert_eq!(cargo_deps.len(), 1);

        let npm_deps = analysis.by_ecosystem(&DependencyEcosystem::Npm);
        assert_eq!(npm_deps.len(), 1);
    }

    #[test]
    fn test_python_dependency_parsing() {
        let test_cases = vec![
            ("requests>=2.28.0", ("requests", ">=2.28.0")),
            ("flask==2.0.0", ("flask", "==2.0.0")),
            ("pydantic~=1.10", ("pydantic", "~=1.10")),
            ("click", ("click", "*")),
            ("pydantic[email]>=1.10", ("pydantic", ">=1.10")),
            (
                "django-rest-framework>=3.14",
                ("django-rest-framework", ">=3.14"),
            ),
        ];

        for (input, expected) in test_cases {
            let (name, version) = DependencyParser::parse_python_dependency(input);
            assert_eq!(name, expected.0, "Failed for input: {}", input);
            assert_eq!(version, expected.1, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_dependency_serialization() {
        let dep = Dependency {
            name: "serde".to_string(),
            version: "1.0".to_string(),
            ecosystem: DependencyEcosystem::Cargo,
            is_dev: false,
            manifest_path: PathBuf::from("Cargo.toml"),
            outdated: None,
        };

        let json = serde_json::to_string(&dep).unwrap();
        assert!(json.contains("\"name\":\"serde\""));
        assert!(json.contains("\"ecosystem\":\"cargo\""));

        let deserialized: Dependency = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "serde");
        assert_eq!(deserialized.ecosystem, DependencyEcosystem::Cargo);
    }

    #[test]
    fn test_dependency_analysis_serialization() {
        let analysis = DependencyAnalysis {
            dependencies: vec![Dependency {
                name: "serde".to_string(),
                version: "1.0".to_string(),
                ecosystem: DependencyEcosystem::Cargo,
                is_dev: false,
                manifest_path: PathBuf::from("Cargo.toml"),
                outdated: None,
            }],
            ecosystem_counts: vec![(DependencyEcosystem::Cargo, 1)],
            outdated_count: 0,
            vulnerable_count: 0,
        };

        let json = serde_json::to_string(&analysis).unwrap();
        let deserialized: DependencyAnalysis = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.dependencies.len(), 1);
        assert_eq!(deserialized.dependencies[0].name, "serde");
    }

    #[test]
    fn test_outdated_count() {
        let analysis = DependencyAnalysis {
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
                    version: "1.0".to_string(),
                    ecosystem: DependencyEcosystem::Cargo,
                    is_dev: false,
                    manifest_path: PathBuf::from("Cargo.toml"),
                    outdated: Some(OutdatedInfo {
                        latest_version: "1.25".to_string(),
                        is_major_bump: false,
                        security_advisory: Some("CVE-2023-1234".to_string()),
                    }),
                },
                Dependency {
                    name: "clap".to_string(),
                    version: "4.0".to_string(),
                    ecosystem: DependencyEcosystem::Cargo,
                    is_dev: false,
                    manifest_path: PathBuf::from("Cargo.toml"),
                    outdated: None,
                },
            ],
            ecosystem_counts: vec![(DependencyEcosystem::Cargo, 3)],
            outdated_count: 2,
            vulnerable_count: 1,
        };

        assert_eq!(analysis.outdated_count, 2);
        assert_eq!(analysis.vulnerable_count, 1);
    }
}
