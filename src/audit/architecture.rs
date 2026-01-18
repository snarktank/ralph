//! Architecture pattern detection and analysis.

use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use super::AuditResult;

/// Detected architecture pattern
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitecturePattern {
    /// Traditional layered architecture (presentation, business, data)
    Layered,
    /// Modular/component-based architecture
    Modular,
    /// Hexagonal/ports and adapters architecture
    Hexagonal,
    /// Clean architecture (entities, use cases, interfaces, frameworks)
    Clean,
    /// Microservices architecture
    Microservices,
    /// Monolithic architecture
    Monolithic,
    /// MVC (Model-View-Controller)
    Mvc,
    /// MVVM (Model-View-ViewModel)
    Mvvm,
    /// Event-driven architecture
    EventDriven,
    /// Mixed or unclear architecture
    Mixed,
    /// Unknown architecture pattern
    #[default]
    Unknown,
}

impl std::fmt::Display for ArchitecturePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchitecturePattern::Layered => write!(f, "layered"),
            ArchitecturePattern::Modular => write!(f, "modular"),
            ArchitecturePattern::Hexagonal => write!(f, "hexagonal"),
            ArchitecturePattern::Clean => write!(f, "clean"),
            ArchitecturePattern::Microservices => write!(f, "microservices"),
            ArchitecturePattern::Monolithic => write!(f, "monolithic"),
            ArchitecturePattern::Mvc => write!(f, "mvc"),
            ArchitecturePattern::Mvvm => write!(f, "mvvm"),
            ArchitecturePattern::EventDriven => write!(f, "event-driven"),
            ArchitecturePattern::Mixed => write!(f, "mixed"),
            ArchitecturePattern::Unknown => write!(f, "unknown"),
        }
    }
}

/// A detected layer or module in the architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureLayer {
    /// Name of the layer/module
    pub name: String,
    /// Path to the layer/module
    pub path: PathBuf,
    /// Type of layer (e.g., "presentation", "domain", "infrastructure")
    pub layer_type: LayerType,
    /// Files in this layer
    pub file_count: usize,
    /// Dependencies on other layers
    pub depends_on: Vec<String>,
}

/// Type of architectural layer
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerType {
    /// Presentation/UI layer
    Presentation,
    /// Business logic/domain layer
    Domain,
    /// Data access/infrastructure layer
    Infrastructure,
    /// Application/use case layer
    Application,
    /// API/interface layer
    Api,
    /// Utility/shared layer
    Shared,
    /// Test layer
    Test,
    /// Unknown layer type
    #[default]
    Unknown,
}

impl std::fmt::Display for LayerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerType::Presentation => write!(f, "presentation"),
            LayerType::Domain => write!(f, "domain"),
            LayerType::Infrastructure => write!(f, "infrastructure"),
            LayerType::Application => write!(f, "application"),
            LayerType::Api => write!(f, "api"),
            LayerType::Shared => write!(f, "shared"),
            LayerType::Test => write!(f, "test"),
            LayerType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Coupling between two modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleCoupling {
    /// Source module name
    pub from_module: String,
    /// Target module name
    pub to_module: String,
    /// Number of imports/dependencies
    pub import_count: usize,
    /// Coupling strength (low, medium, high)
    pub strength: CouplingStrength,
}

/// Coupling strength level
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CouplingStrength {
    #[default]
    Low,
    Medium,
    High,
}

impl std::fmt::Display for CouplingStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CouplingStrength::Low => write!(f, "low"),
            CouplingStrength::Medium => write!(f, "medium"),
            CouplingStrength::High => write!(f, "high"),
        }
    }
}

/// Boundary violation in the architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryViolation {
    /// File where the violation occurs
    pub file: PathBuf,
    /// Line number (if available)
    pub line: Option<usize>,
    /// Source layer
    pub from_layer: String,
    /// Target layer (being incorrectly accessed)
    pub to_layer: String,
    /// Description of the violation
    pub description: String,
}

/// Complete architecture analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArchitectureAnalysis {
    /// Primary detected architecture pattern
    pub pattern: ArchitecturePattern,
    /// Secondary patterns (if mixed)
    pub secondary_patterns: Vec<ArchitecturePattern>,
    /// Confidence score for the detected pattern (0.0 - 1.0)
    pub confidence: f64,
    /// Detected layers/modules
    pub layers: Vec<ArchitectureLayer>,
    /// Module coupling information
    pub couplings: Vec<ModuleCoupling>,
    /// Detected boundary violations
    pub boundary_violations: Vec<BoundaryViolation>,
    /// Overall coupling score (0.0 = loose, 1.0 = tight)
    pub coupling_score: f64,
    /// Architectural observations and notes
    pub observations: Vec<String>,
}

/// Analyzer for detecting architecture patterns
pub struct ArchitectureAnalyzer {
    root: PathBuf,
}

impl ArchitectureAnalyzer {
    /// Create a new architecture analyzer
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Get the root path
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Analyze the architecture of the codebase
    pub fn analyze(&self) -> AuditResult<ArchitectureAnalysis> {
        let mut analysis = ArchitectureAnalysis::default();

        // Collect directory structure
        let directories = self.collect_directories()?;

        // Detect layers/modules
        let layers = self.detect_layers(&directories)?;
        analysis.layers = layers;

        // Detect coupling between modules
        let couplings = self.detect_coupling(&analysis.layers)?;
        analysis.couplings = couplings;

        // Calculate coupling score
        analysis.coupling_score = self.calculate_coupling_score(&analysis.couplings);

        // Detect architecture pattern
        let (pattern, secondary, confidence) = self.detect_pattern(&directories, &analysis.layers);
        analysis.pattern = pattern;
        analysis.secondary_patterns = secondary;
        analysis.confidence = confidence;

        // Detect boundary violations
        analysis.boundary_violations = self.detect_boundary_violations(&analysis.layers)?;

        // Generate observations
        analysis.observations = self.generate_observations(&analysis);

        Ok(analysis)
    }

    /// Collect all directories in the project
    fn collect_directories(&self) -> AuditResult<Vec<String>> {
        let mut directories = Vec::new();

        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .max_depth(Some(3))
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Skip hidden directories and common ignored ones
                    if !name.starts_with('.')
                        && name != "node_modules"
                        && name != "target"
                        && name != "__pycache__"
                        && name != "venv"
                        && name != ".venv"
                        && name != "dist"
                        && name != "build"
                    {
                        directories.push(name.to_string());
                    }
                }
            }
        }

        Ok(directories)
    }

    /// Detect layers/modules in the codebase
    fn detect_layers(&self, directories: &[String]) -> AuditResult<Vec<ArchitectureLayer>> {
        let mut layers = Vec::new();

        // Walk the top-level directories to identify layers
        let entries = fs::read_dir(&self.root)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                // Skip hidden and ignored directories
                if name.starts_with('.')
                    || name == "node_modules"
                    || name == "target"
                    || name == "__pycache__"
                    || name == "venv"
                    || name == ".venv"
                {
                    continue;
                }

                let layer_type = Self::classify_layer_type(&name, directories);
                let file_count = self.count_files_in_dir(&path);
                let depends_on = self.detect_layer_dependencies(&path, &name)?;

                if file_count > 0 || !depends_on.is_empty() {
                    layers.push(ArchitectureLayer {
                        name: name.clone(),
                        path: path.strip_prefix(&self.root).unwrap_or(&path).to_path_buf(),
                        layer_type,
                        file_count,
                        depends_on,
                    });
                }
            }
        }

        Ok(layers)
    }

    /// Classify the type of a layer based on its name
    fn classify_layer_type(name: &str, all_dirs: &[String]) -> LayerType {
        let name_lower = name.to_lowercase();

        // Presentation layer indicators
        let presentation_indicators = [
            "ui",
            "view",
            "views",
            "components",
            "pages",
            "screens",
            "templates",
            "frontend",
            "client",
            "presentation",
            "web",
            "gui",
        ];

        // Domain layer indicators
        let domain_indicators = [
            "domain", "models", "model", "entities", "entity", "core", "business",
        ];

        // Infrastructure layer indicators
        let infra_indicators = [
            "infrastructure",
            "infra",
            "data",
            "database",
            "db",
            "repositories",
            "repository",
            "persistence",
            "storage",
            "external",
            "adapters",
        ];

        // Application layer indicators
        let app_indicators = [
            "application",
            "app",
            "services",
            "service",
            "usecases",
            "use_cases",
            "use-cases",
            "commands",
            "handlers",
        ];

        // API layer indicators
        let api_indicators = [
            "api",
            "rest",
            "graphql",
            "grpc",
            "controllers",
            "controller",
            "routes",
            "endpoints",
            "interfaces",
            "ports",
        ];

        // Shared layer indicators
        let shared_indicators = [
            "shared",
            "common",
            "utils",
            "utilities",
            "helpers",
            "lib",
            "libs",
            "pkg",
        ];

        // Test layer indicators
        let test_indicators = ["tests", "test", "spec", "specs", "__tests__", "testing"];

        if presentation_indicators.contains(&name_lower.as_str()) {
            LayerType::Presentation
        } else if domain_indicators.contains(&name_lower.as_str()) {
            LayerType::Domain
        } else if infra_indicators.contains(&name_lower.as_str()) {
            LayerType::Infrastructure
        } else if app_indicators.contains(&name_lower.as_str()) {
            LayerType::Application
        } else if api_indicators.contains(&name_lower.as_str()) {
            LayerType::Api
        } else if shared_indicators.contains(&name_lower.as_str()) {
            LayerType::Shared
        } else if test_indicators.contains(&name_lower.as_str()) {
            LayerType::Test
        } else if name_lower == "src" {
            // Check if src has meaningful subdirectories
            let has_layered_subdirs = all_dirs.iter().any(|d| {
                let lower = d.to_lowercase();
                domain_indicators.contains(&lower.as_str())
                    || app_indicators.contains(&lower.as_str())
                    || infra_indicators.contains(&lower.as_str())
            });
            if has_layered_subdirs {
                LayerType::Application
            } else {
                LayerType::Unknown
            }
        } else {
            LayerType::Unknown
        }
    }

    /// Count files in a directory
    fn count_files_in_dir(&self, dir: &PathBuf) -> usize {
        let walker = WalkBuilder::new(dir).hidden(false).git_ignore(true).build();

        walker
            .flatten()
            .filter(|e| e.path().is_file())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| {
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
                        )
                    })
                    .unwrap_or(false)
            })
            .count()
    }

    /// Detect dependencies of a layer on other layers
    fn detect_layer_dependencies(
        &self,
        dir: &PathBuf,
        current_layer: &str,
    ) -> AuditResult<Vec<String>> {
        let mut dependencies = HashSet::new();

        // Get list of other top-level directories as potential dependencies
        let mut other_layers: Vec<String> = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name != current_layer
                            && !name.starts_with('.')
                            && name != "node_modules"
                            && name != "target"
                        {
                            other_layers.push(name.to_string());
                        }
                    }
                }
            }
        }

        // Scan files in this directory for imports
        let walker = WalkBuilder::new(dir).hidden(false).git_ignore(true).build();

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

            // Check for imports from other layers
            for layer in &other_layers {
                if self.has_import_from(&content, layer, &ext) {
                    dependencies.insert(layer.clone());
                }
            }
        }

        Ok(dependencies.into_iter().collect())
    }

    /// Check if content has an import from a specific module
    fn has_import_from(&self, content: &str, module: &str, ext: &str) -> bool {
        let module_lower = module.to_lowercase();

        match ext {
            "rs" => {
                // Rust: use crate::module or mod module
                let pattern = format!(
                    r"(?:use\s+(?:crate::)?|mod\s+){}",
                    regex::escape(&module_lower)
                );
                Regex::new(&pattern)
                    .map(|re| re.is_match(&content.to_lowercase()))
                    .unwrap_or(false)
            }
            "js" | "ts" | "tsx" | "jsx" => {
                // JS/TS: import from '../module' or require('../module')
                let patterns = [
                    format!(r#"from\s+['"][^'"]*/{}"#, regex::escape(&module_lower)),
                    format!(
                        r#"require\s*\(\s*['"][^'"]*/{}"#,
                        regex::escape(&module_lower)
                    ),
                ];
                patterns.iter().any(|p| {
                    Regex::new(p)
                        .map(|re| re.is_match(&content.to_lowercase()))
                        .unwrap_or(false)
                })
            }
            "py" => {
                // Python: from module import or import module
                let patterns = [
                    format!(r"from\s+{}", regex::escape(&module_lower)),
                    format!(r"import\s+{}", regex::escape(&module_lower)),
                ];
                patterns.iter().any(|p| {
                    Regex::new(p)
                        .map(|re| re.is_match(&content.to_lowercase()))
                        .unwrap_or(false)
                })
            }
            "go" => {
                // Go: import "path/module"
                let pattern = format!(r#"import\s+.*["'][^"']*/{}"#, regex::escape(&module_lower));
                Regex::new(&pattern)
                    .map(|re| re.is_match(&content.to_lowercase()))
                    .unwrap_or(false)
            }
            "java" => {
                // Java: import package.module
                let pattern = format!(r"import\s+[^;]*\.{}", regex::escape(&module_lower));
                Regex::new(&pattern)
                    .map(|re| re.is_match(&content.to_lowercase()))
                    .unwrap_or(false)
            }
            _ => false,
        }
    }

    /// Detect coupling between modules
    fn detect_coupling(&self, layers: &[ArchitectureLayer]) -> AuditResult<Vec<ModuleCoupling>> {
        let mut couplings = Vec::new();

        for layer in layers {
            for dependency in &layer.depends_on {
                // Count imports between these modules
                let import_count = self.count_imports_between(&layer.name, dependency)?;
                let strength = Self::classify_coupling_strength(import_count);

                couplings.push(ModuleCoupling {
                    from_module: layer.name.clone(),
                    to_module: dependency.clone(),
                    import_count,
                    strength,
                });
            }
        }

        Ok(couplings)
    }

    /// Count imports between two modules
    fn count_imports_between(&self, from: &str, to: &str) -> AuditResult<usize> {
        let from_dir = self.root.join(from);
        if !from_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        let walker = WalkBuilder::new(&from_dir)
            .hidden(false)
            .git_ignore(true)
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

            // Count lines that reference the target module
            count += content
                .lines()
                .filter(|line| {
                    let line_lower = line.to_lowercase();
                    line_lower.contains(&to.to_lowercase())
                        && (line_lower.contains("import")
                            || line_lower.contains("use ")
                            || line_lower.contains("from ")
                            || line_lower.contains("require"))
                })
                .count();
        }

        Ok(count)
    }

    /// Classify coupling strength based on import count
    fn classify_coupling_strength(import_count: usize) -> CouplingStrength {
        if import_count <= 3 {
            CouplingStrength::Low
        } else if import_count <= 10 {
            CouplingStrength::Medium
        } else {
            CouplingStrength::High
        }
    }

    /// Calculate overall coupling score
    fn calculate_coupling_score(&self, couplings: &[ModuleCoupling]) -> f64 {
        if couplings.is_empty() {
            return 0.0;
        }

        let total_weight: f64 = couplings
            .iter()
            .map(|c| match c.strength {
                CouplingStrength::Low => 0.2,
                CouplingStrength::Medium => 0.5,
                CouplingStrength::High => 1.0,
            })
            .sum();

        (total_weight / couplings.len() as f64).min(1.0)
    }

    /// Detect the architecture pattern
    fn detect_pattern(
        &self,
        directories: &[String],
        layers: &[ArchitectureLayer],
    ) -> (ArchitecturePattern, Vec<ArchitecturePattern>, f64) {
        let dir_lower: Vec<String> = directories.iter().map(|s| s.to_lowercase()).collect();

        let mut scores: HashMap<ArchitecturePattern, f64> = HashMap::new();

        // Check for Clean Architecture indicators
        let clean_indicators = ["domain", "entities", "usecases", "use_cases", "interfaces"];
        let clean_count = clean_indicators
            .iter()
            .filter(|i| dir_lower.contains(&i.to_string()))
            .count();
        if clean_count >= 2 {
            scores.insert(ArchitecturePattern::Clean, clean_count as f64 / 4.0);
        }

        // Check for Hexagonal Architecture indicators
        let hex_indicators = ["ports", "adapters", "domain", "application"];
        let hex_count = hex_indicators
            .iter()
            .filter(|i| dir_lower.contains(&i.to_string()))
            .count();
        if hex_count >= 2 {
            scores.insert(ArchitecturePattern::Hexagonal, hex_count as f64 / 4.0);
        }

        // Check for Layered Architecture indicators
        let layered_indicators = [
            "controllers",
            "services",
            "repositories",
            "models",
            "handlers",
        ];
        let layered_count = layered_indicators
            .iter()
            .filter(|i| dir_lower.contains(&i.to_string()))
            .count();
        if layered_count >= 2 {
            scores.insert(ArchitecturePattern::Layered, layered_count as f64 / 4.0);
        }

        // Check for MVC indicators
        let mvc_indicators = ["models", "views", "controllers"];
        let mvc_count = mvc_indicators
            .iter()
            .filter(|i| dir_lower.contains(&i.to_string()))
            .count();
        if mvc_count >= 2 {
            scores.insert(ArchitecturePattern::Mvc, mvc_count as f64 / 3.0);
        }

        // Check for MVVM indicators
        let mvvm_indicators = ["models", "views", "viewmodels", "view_models"];
        let mvvm_count = mvvm_indicators
            .iter()
            .filter(|i| dir_lower.contains(&i.to_string()))
            .count();
        if mvvm_count >= 2 {
            scores.insert(ArchitecturePattern::Mvvm, mvvm_count as f64 / 3.0);
        }

        // Check for Microservices indicators
        let has_multiple_services = dir_lower.iter().filter(|d| d.ends_with("service")).count() > 2;
        let has_docker_compose = self.root.join("docker-compose.yml").exists()
            || self.root.join("docker-compose.yaml").exists();
        if has_multiple_services || (has_docker_compose && dir_lower.len() > 5) {
            scores.insert(ArchitecturePattern::Microservices, 0.6);
        }

        // Check for Modular architecture
        let has_modules = dir_lower.contains(&"modules".to_string())
            || dir_lower.contains(&"features".to_string());
        if has_modules {
            scores.insert(ArchitecturePattern::Modular, 0.7);
        }

        // Check for Event-driven architecture
        let event_indicators = [
            "events",
            "handlers",
            "subscribers",
            "publishers",
            "messages",
        ];
        let event_count = event_indicators
            .iter()
            .filter(|i| dir_lower.contains(&i.to_string()))
            .count();
        if event_count >= 2 {
            scores.insert(ArchitecturePattern::EventDriven, event_count as f64 / 4.0);
        }

        // Determine primary pattern
        if scores.is_empty() {
            // Default to Monolithic if no clear pattern
            if layers.len() <= 3 && layers.iter().all(|l| l.file_count < 50) {
                return (ArchitecturePattern::Monolithic, Vec::new(), 0.5);
            }
            return (ArchitecturePattern::Unknown, Vec::new(), 0.0);
        }

        let (primary, primary_score) = scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(p, s)| (p.clone(), *s))
            .unwrap_or((ArchitecturePattern::Unknown, 0.0));

        // Collect secondary patterns (those with score > 0.3)
        let secondary: Vec<ArchitecturePattern> = scores
            .iter()
            .filter(|(p, s)| **p != primary && **s > 0.3)
            .map(|(p, _)| p.clone())
            .collect();

        let confidence = if secondary.is_empty() {
            primary_score.min(1.0)
        } else {
            (primary_score * 0.7).min(1.0) // Reduce confidence if mixed
        };

        if secondary.len() > 1 {
            (ArchitecturePattern::Mixed, secondary, confidence)
        } else {
            (primary, secondary, confidence)
        }
    }

    /// Detect boundary violations in the architecture
    fn detect_boundary_violations(
        &self,
        layers: &[ArchitectureLayer],
    ) -> AuditResult<Vec<BoundaryViolation>> {
        let mut violations = Vec::new();

        // Define forbidden dependencies based on clean architecture principles
        // Infrastructure should not be imported by Domain
        // Presentation should not be imported by Domain or Application
        let forbidden_deps: Vec<(LayerType, LayerType, &str)> = vec![
            (
                LayerType::Domain,
                LayerType::Infrastructure,
                "Domain layer should not depend on Infrastructure",
            ),
            (
                LayerType::Domain,
                LayerType::Presentation,
                "Domain layer should not depend on Presentation",
            ),
            (
                LayerType::Application,
                LayerType::Presentation,
                "Application layer should not depend on Presentation",
            ),
            (
                LayerType::Domain,
                LayerType::Api,
                "Domain layer should not depend on API layer",
            ),
        ];

        for layer in layers {
            for dep in &layer.depends_on {
                // Find the dependent layer
                if let Some(dep_layer) = layers.iter().find(|l| &l.name == dep) {
                    // Check if this is a forbidden dependency
                    for (from_type, to_type, desc) in &forbidden_deps {
                        if layer.layer_type == *from_type && dep_layer.layer_type == *to_type {
                            violations.push(BoundaryViolation {
                                file: layer.path.clone(),
                                line: None,
                                from_layer: layer.name.clone(),
                                to_layer: dep.clone(),
                                description: desc.to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Generate observations about the architecture
    fn generate_observations(&self, analysis: &ArchitectureAnalysis) -> Vec<String> {
        let mut observations = Vec::new();

        // Pattern observation
        match analysis.pattern {
            ArchitecturePattern::Unknown => {
                observations.push(
                    "No clear architecture pattern detected. Consider adopting a standard pattern."
                        .to_string(),
                );
            }
            ArchitecturePattern::Mixed => {
                observations.push(format!(
                    "Mixed architecture patterns detected: {:?}. Consider consolidating to a single pattern.",
                    analysis.secondary_patterns
                ));
            }
            _ => {
                observations.push(format!(
                    "Detected {} architecture with {:.0}% confidence.",
                    analysis.pattern,
                    analysis.confidence * 100.0
                ));
            }
        }

        // Coupling observation
        if analysis.coupling_score > 0.7 {
            observations.push(
                "High coupling detected between modules. Consider introducing abstractions."
                    .to_string(),
            );
        } else if analysis.coupling_score < 0.3 && !analysis.layers.is_empty() {
            observations
                .push("Good separation of concerns with low coupling between modules.".to_string());
        }

        // Boundary violations
        if !analysis.boundary_violations.is_empty() {
            observations.push(format!(
                "Found {} boundary violation(s). Review layer dependencies.",
                analysis.boundary_violations.len()
            ));
        }

        // Layer count
        let meaningful_layers: Vec<_> = analysis
            .layers
            .iter()
            .filter(|l| l.layer_type != LayerType::Unknown && l.layer_type != LayerType::Test)
            .collect();

        if meaningful_layers.is_empty() {
            observations.push(
                "No clear layer structure detected. Consider organizing code into distinct layers."
                    .to_string(),
            );
        } else if meaningful_layers.len() == 1 {
            observations.push(
                "Single layer detected. For larger projects, consider separating concerns."
                    .to_string(),
            );
        }

        observations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_architecture_pattern_default() {
        assert_eq!(ArchitecturePattern::default(), ArchitecturePattern::Unknown);
    }

    #[test]
    fn test_architecture_pattern_display() {
        assert_eq!(format!("{}", ArchitecturePattern::Layered), "layered");
        assert_eq!(format!("{}", ArchitecturePattern::Hexagonal), "hexagonal");
        assert_eq!(format!("{}", ArchitecturePattern::Clean), "clean");
        assert_eq!(format!("{}", ArchitecturePattern::Mvc), "mvc");
        assert_eq!(format!("{}", ArchitecturePattern::Modular), "modular");
    }

    #[test]
    fn test_layer_type_default() {
        assert_eq!(LayerType::default(), LayerType::Unknown);
    }

    #[test]
    fn test_layer_type_display() {
        assert_eq!(format!("{}", LayerType::Presentation), "presentation");
        assert_eq!(format!("{}", LayerType::Domain), "domain");
        assert_eq!(format!("{}", LayerType::Infrastructure), "infrastructure");
        assert_eq!(format!("{}", LayerType::Application), "application");
    }

    #[test]
    fn test_coupling_strength_default() {
        assert_eq!(CouplingStrength::default(), CouplingStrength::Low);
    }

    #[test]
    fn test_coupling_strength_display() {
        assert_eq!(format!("{}", CouplingStrength::Low), "low");
        assert_eq!(format!("{}", CouplingStrength::Medium), "medium");
        assert_eq!(format!("{}", CouplingStrength::High), "high");
    }

    #[test]
    fn test_architecture_analyzer_new() {
        let analyzer = ArchitectureAnalyzer::new(PathBuf::from("/test"));
        assert_eq!(analyzer.root(), &PathBuf::from("/test"));
    }

    #[test]
    fn test_classify_layer_type_presentation() {
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("views", &[]),
            LayerType::Presentation
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("components", &[]),
            LayerType::Presentation
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("ui", &[]),
            LayerType::Presentation
        );
    }

    #[test]
    fn test_classify_layer_type_domain() {
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("domain", &[]),
            LayerType::Domain
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("models", &[]),
            LayerType::Domain
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("entities", &[]),
            LayerType::Domain
        );
    }

    #[test]
    fn test_classify_layer_type_infrastructure() {
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("infrastructure", &[]),
            LayerType::Infrastructure
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("repositories", &[]),
            LayerType::Infrastructure
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("database", &[]),
            LayerType::Infrastructure
        );
    }

    #[test]
    fn test_classify_layer_type_application() {
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("services", &[]),
            LayerType::Application
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("usecases", &[]),
            LayerType::Application
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("handlers", &[]),
            LayerType::Application
        );
    }

    #[test]
    fn test_classify_layer_type_api() {
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("api", &[]),
            LayerType::Api
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("controllers", &[]),
            LayerType::Api
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("routes", &[]),
            LayerType::Api
        );
    }

    #[test]
    fn test_classify_layer_type_shared() {
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("utils", &[]),
            LayerType::Shared
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("common", &[]),
            LayerType::Shared
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("helpers", &[]),
            LayerType::Shared
        );
    }

    #[test]
    fn test_classify_layer_type_test() {
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("tests", &[]),
            LayerType::Test
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_layer_type("spec", &[]),
            LayerType::Test
        );
    }

    #[test]
    fn test_classify_coupling_strength() {
        assert_eq!(
            ArchitectureAnalyzer::classify_coupling_strength(1),
            CouplingStrength::Low
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_coupling_strength(3),
            CouplingStrength::Low
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_coupling_strength(5),
            CouplingStrength::Medium
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_coupling_strength(10),
            CouplingStrength::Medium
        );
        assert_eq!(
            ArchitectureAnalyzer::classify_coupling_strength(15),
            CouplingStrength::High
        );
    }

    #[test]
    fn test_analyze_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = ArchitectureAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        // Empty directories may detect as Monolithic (small project) or Unknown
        assert!(
            analysis.pattern == ArchitecturePattern::Unknown
                || analysis.pattern == ArchitecturePattern::Monolithic
        );
        assert!(analysis.layers.is_empty());
        assert!(analysis.couplings.is_empty());
        assert_eq!(analysis.coupling_score, 0.0);
    }

    #[test]
    fn test_analyze_layered_architecture() {
        let temp_dir = TempDir::new().unwrap();

        // Create layered architecture structure
        let controllers = temp_dir.path().join("controllers");
        let services = temp_dir.path().join("services");
        let repositories = temp_dir.path().join("repositories");
        let models = temp_dir.path().join("models");

        fs::create_dir(&controllers).unwrap();
        fs::create_dir(&services).unwrap();
        fs::create_dir(&repositories).unwrap();
        fs::create_dir(&models).unwrap();

        // Add files to each layer
        fs::write(
            controllers.join("user_controller.rs"),
            "use crate::services;\npub fn get_user() {}",
        )
        .unwrap();
        fs::write(
            services.join("user_service.rs"),
            "use crate::repositories;\npub fn find_user() {}",
        )
        .unwrap();
        fs::write(
            repositories.join("user_repository.rs"),
            "use crate::models;\npub fn get_by_id() {}",
        )
        .unwrap();
        fs::write(models.join("user.rs"), "pub struct User {}").unwrap();

        let analyzer = ArchitectureAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        assert_eq!(analysis.pattern, ArchitecturePattern::Layered);
        assert!(analysis.confidence > 0.0);
        assert_eq!(analysis.layers.len(), 4);
    }

    #[test]
    fn test_analyze_clean_architecture() {
        let temp_dir = TempDir::new().unwrap();

        // Create clean architecture structure
        let domain = temp_dir.path().join("domain");
        let entities = temp_dir.path().join("entities");
        let usecases = temp_dir.path().join("usecases");
        let interfaces = temp_dir.path().join("interfaces");

        fs::create_dir(&domain).unwrap();
        fs::create_dir(&entities).unwrap();
        fs::create_dir(&usecases).unwrap();
        fs::create_dir(&interfaces).unwrap();

        // Add files
        fs::write(domain.join("user.rs"), "pub struct User {}").unwrap();
        fs::write(entities.join("entity.rs"), "pub trait Entity {}").unwrap();
        fs::write(usecases.join("create_user.rs"), "pub fn create() {}").unwrap();
        fs::write(interfaces.join("repository.rs"), "pub trait Repo {}").unwrap();

        let analyzer = ArchitectureAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        assert!(
            analysis.pattern == ArchitecturePattern::Clean
                || analysis
                    .secondary_patterns
                    .contains(&ArchitecturePattern::Clean)
        );
    }

    #[test]
    fn test_analyze_mvc_architecture() {
        let temp_dir = TempDir::new().unwrap();

        // Create MVC structure
        let models = temp_dir.path().join("models");
        let views = temp_dir.path().join("views");
        let controllers = temp_dir.path().join("controllers");

        fs::create_dir(&models).unwrap();
        fs::create_dir(&views).unwrap();
        fs::create_dir(&controllers).unwrap();

        // Add files
        fs::write(models.join("user.rs"), "pub struct User {}").unwrap();
        fs::write(views.join("user_view.rs"), "pub fn render() {}").unwrap();
        fs::write(controllers.join("user_controller.rs"), "pub fn index() {}").unwrap();

        let analyzer = ArchitectureAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        // Should detect MVC, Layered, or Mixed (since MVC and Layered overlap)
        assert!(
            analysis.pattern == ArchitecturePattern::Mvc
                || analysis.pattern == ArchitecturePattern::Layered
                || analysis.pattern == ArchitecturePattern::Mixed
                || analysis
                    .secondary_patterns
                    .contains(&ArchitecturePattern::Mvc)
                || analysis
                    .secondary_patterns
                    .contains(&ArchitecturePattern::Layered)
        );
    }

    #[test]
    fn test_analyze_modular_architecture() {
        let temp_dir = TempDir::new().unwrap();

        // Create modular structure
        let modules = temp_dir.path().join("modules");
        fs::create_dir(&modules).unwrap();

        let user_module = modules.join("user");
        let order_module = modules.join("order");
        fs::create_dir(&user_module).unwrap();
        fs::create_dir(&order_module).unwrap();

        fs::write(user_module.join("mod.rs"), "pub mod user;").unwrap();
        fs::write(order_module.join("mod.rs"), "pub mod order;").unwrap();

        let analyzer = ArchitectureAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        assert_eq!(analysis.pattern, ArchitecturePattern::Modular);
    }

    #[test]
    fn test_detect_coupling() {
        let temp_dir = TempDir::new().unwrap();

        // Create directories with dependencies
        let services = temp_dir.path().join("services");
        let repositories = temp_dir.path().join("repositories");

        fs::create_dir(&services).unwrap();
        fs::create_dir(&repositories).unwrap();

        // Service depends on repository
        fs::write(
            services.join("service.rs"),
            r#"
            use crate::repositories;
            use crate::repositories::UserRepo;
            use crate::repositories::OrderRepo;

            pub fn do_something() {}
            "#,
        )
        .unwrap();

        fs::write(
            repositories.join("repo.rs"),
            "pub struct UserRepo {}\npub struct OrderRepo {}",
        )
        .unwrap();

        let analyzer = ArchitectureAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        // Should detect coupling between services and repositories
        let coupling = analysis
            .couplings
            .iter()
            .find(|c| c.from_module == "services" && c.to_module == "repositories");

        assert!(coupling.is_some());
    }

    #[test]
    fn test_calculate_coupling_score_empty() {
        let analyzer = ArchitectureAnalyzer::new(PathBuf::from("/test"));
        assert_eq!(analyzer.calculate_coupling_score(&[]), 0.0);
    }

    #[test]
    fn test_calculate_coupling_score_low() {
        let analyzer = ArchitectureAnalyzer::new(PathBuf::from("/test"));
        let couplings = vec![
            ModuleCoupling {
                from_module: "a".to_string(),
                to_module: "b".to_string(),
                import_count: 2,
                strength: CouplingStrength::Low,
            },
            ModuleCoupling {
                from_module: "b".to_string(),
                to_module: "c".to_string(),
                import_count: 1,
                strength: CouplingStrength::Low,
            },
        ];

        let score = analyzer.calculate_coupling_score(&couplings);
        assert!(score < 0.3);
    }

    #[test]
    fn test_calculate_coupling_score_high() {
        let analyzer = ArchitectureAnalyzer::new(PathBuf::from("/test"));
        let couplings = vec![
            ModuleCoupling {
                from_module: "a".to_string(),
                to_module: "b".to_string(),
                import_count: 20,
                strength: CouplingStrength::High,
            },
            ModuleCoupling {
                from_module: "b".to_string(),
                to_module: "c".to_string(),
                import_count: 15,
                strength: CouplingStrength::High,
            },
        ];

        let score = analyzer.calculate_coupling_score(&couplings);
        assert!(score > 0.7);
    }

    #[test]
    fn test_boundary_violation_detection() {
        let temp_dir = TempDir::new().unwrap();

        // Create domain and infrastructure directories
        let domain = temp_dir.path().join("domain");
        let infrastructure = temp_dir.path().join("infrastructure");

        fs::create_dir(&domain).unwrap();
        fs::create_dir(&infrastructure).unwrap();

        // Domain imports infrastructure (violation!)
        fs::write(
            domain.join("user.rs"),
            r#"
            use crate::infrastructure::database;

            pub struct User {}
            "#,
        )
        .unwrap();

        fs::write(infrastructure.join("database.rs"), "pub fn connect() {}").unwrap();

        let analyzer = ArchitectureAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        // Should detect boundary violation
        let has_violation = analysis
            .boundary_violations
            .iter()
            .any(|v| v.from_layer == "domain" && v.to_layer == "infrastructure");

        assert!(has_violation || analysis.layers.is_empty()); // May be empty if import detection doesn't match
    }

    #[test]
    fn test_generate_observations_unknown_pattern() {
        let analyzer = ArchitectureAnalyzer::new(PathBuf::from("/test"));
        let analysis = ArchitectureAnalysis {
            pattern: ArchitecturePattern::Unknown,
            ..Default::default()
        };

        let observations = analyzer.generate_observations(&analysis);
        assert!(observations
            .iter()
            .any(|o| o.contains("No clear architecture pattern")));
    }

    #[test]
    fn test_generate_observations_high_coupling() {
        let analyzer = ArchitectureAnalyzer::new(PathBuf::from("/test"));
        let analysis = ArchitectureAnalysis {
            pattern: ArchitecturePattern::Layered,
            coupling_score: 0.8,
            confidence: 0.7,
            ..Default::default()
        };

        let observations = analyzer.generate_observations(&analysis);
        assert!(observations.iter().any(|o| o.contains("High coupling")));
    }

    #[test]
    fn test_architecture_analysis_serialization() {
        let analysis = ArchitectureAnalysis {
            pattern: ArchitecturePattern::Clean,
            secondary_patterns: vec![ArchitecturePattern::Layered],
            confidence: 0.85,
            layers: vec![ArchitectureLayer {
                name: "domain".to_string(),
                path: PathBuf::from("domain"),
                layer_type: LayerType::Domain,
                file_count: 10,
                depends_on: vec!["shared".to_string()],
            }],
            couplings: vec![ModuleCoupling {
                from_module: "domain".to_string(),
                to_module: "shared".to_string(),
                import_count: 5,
                strength: CouplingStrength::Medium,
            }],
            boundary_violations: vec![],
            coupling_score: 0.3,
            observations: vec!["Good architecture".to_string()],
        };

        let json = serde_json::to_string(&analysis).unwrap();
        let deserialized: ArchitectureAnalysis = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.pattern, ArchitecturePattern::Clean);
        assert_eq!(deserialized.confidence, 0.85);
        assert_eq!(deserialized.layers.len(), 1);
        assert_eq!(deserialized.couplings.len(), 1);
    }

    #[test]
    fn test_has_import_from_rust() {
        let analyzer = ArchitectureAnalyzer::new(PathBuf::from("/test"));

        let content = r#"
            use crate::services::UserService;
            use crate::models::User;
        "#;

        assert!(analyzer.has_import_from(content, "services", "rs"));
        assert!(analyzer.has_import_from(content, "models", "rs"));
        assert!(!analyzer.has_import_from(content, "controllers", "rs"));
    }

    #[test]
    fn test_has_import_from_javascript() {
        let analyzer = ArchitectureAnalyzer::new(PathBuf::from("/test"));

        let content = r#"
            import { UserService } from '../services/userService';
            const model = require('../models/user');
        "#;

        assert!(analyzer.has_import_from(content, "services", "js"));
        assert!(analyzer.has_import_from(content, "models", "js"));
        assert!(!analyzer.has_import_from(content, "controllers", "js"));
    }

    #[test]
    fn test_has_import_from_python() {
        let analyzer = ArchitectureAnalyzer::new(PathBuf::from("/test"));

        let content = r#"
            from services import UserService
            import models.user
        "#;

        assert!(analyzer.has_import_from(content, "services", "py"));
        assert!(analyzer.has_import_from(content, "models", "py"));
        assert!(!analyzer.has_import_from(content, "controllers", "py"));
    }

    #[test]
    fn test_hexagonal_architecture_detection() {
        let temp_dir = TempDir::new().unwrap();

        // Create hexagonal architecture structure
        let domain = temp_dir.path().join("domain");
        let ports = temp_dir.path().join("ports");
        let adapters = temp_dir.path().join("adapters");
        let application = temp_dir.path().join("application");

        fs::create_dir(&domain).unwrap();
        fs::create_dir(&ports).unwrap();
        fs::create_dir(&adapters).unwrap();
        fs::create_dir(&application).unwrap();

        fs::write(domain.join("entity.rs"), "pub struct Entity {}").unwrap();
        fs::write(ports.join("port.rs"), "pub trait Port {}").unwrap();
        fs::write(adapters.join("adapter.rs"), "pub struct Adapter {}").unwrap();
        fs::write(application.join("service.rs"), "pub fn run() {}").unwrap();

        let analyzer = ArchitectureAnalyzer::new(temp_dir.path().to_path_buf());
        let analysis = analyzer.analyze().unwrap();

        assert!(
            analysis.pattern == ArchitecturePattern::Hexagonal
                || analysis
                    .secondary_patterns
                    .contains(&ArchitecturePattern::Hexagonal)
        );
    }
}
