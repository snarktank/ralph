//! Architecture gap detection for identifying structural improvements.
//!
//! This module analyzes the codebase to find:
//! - Missing abstraction layers
//! - Inconsistent module boundaries
//! - Layer violations (e.g., UI calling DB directly)

use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::audit::architecture::{ArchitectureAnalysis, LayerType};
use crate::audit::{AuditFinding, AuditResult, Severity};

/// Type of architecture gap detected
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureGapType {
    /// Missing abstraction layer (e.g., no service layer between controller and repository)
    MissingAbstractionLayer,
    /// Inconsistent module boundary (e.g., mixing concerns within a module)
    InconsistentModuleBoundary,
    /// Layer violation (e.g., presentation layer accessing data layer directly)
    LayerViolation,
    /// Direct database access from UI/presentation layer
    DirectDatabaseAccess,
    /// Missing interface/trait for dependency injection
    MissingInterface,
    /// Circular dependency between modules
    CircularDependency,
    /// God module (module with too many responsibilities)
    GodModule,
    /// Orphan module (module with no clear layer assignment)
    OrphanModule,
}

impl std::fmt::Display for ArchitectureGapType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchitectureGapType::MissingAbstractionLayer => write!(f, "missing_abstraction_layer"),
            ArchitectureGapType::InconsistentModuleBoundary => {
                write!(f, "inconsistent_module_boundary")
            }
            ArchitectureGapType::LayerViolation => write!(f, "layer_violation"),
            ArchitectureGapType::DirectDatabaseAccess => write!(f, "direct_database_access"),
            ArchitectureGapType::MissingInterface => write!(f, "missing_interface"),
            ArchitectureGapType::CircularDependency => write!(f, "circular_dependency"),
            ArchitectureGapType::GodModule => write!(f, "god_module"),
            ArchitectureGapType::OrphanModule => write!(f, "orphan_module"),
        }
    }
}

/// An architecture gap found in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureGap {
    /// Type of gap
    pub gap_type: ArchitectureGapType,
    /// Files involved in the gap
    pub files: Vec<PathBuf>,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// Source layer/module
    pub source: Option<String>,
    /// Target layer/module (for violations)
    pub target: Option<String>,
    /// Description of the gap
    pub description: String,
    /// Severity of the gap
    pub severity: Severity,
    /// Recommendation for addressing the gap
    pub recommendation: String,
}

/// Complete architecture gap analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArchitectureGapsAnalysis {
    /// List of detected architecture gaps
    pub gaps: Vec<ArchitectureGap>,
    /// Count of gaps by type
    pub gap_counts: HashMap<String, usize>,
    /// Total number of gaps
    pub total_gaps: usize,
    /// High severity gaps count
    pub high_severity_count: usize,
    /// Medium severity gaps count
    pub medium_severity_count: usize,
    /// Low severity gaps count
    pub low_severity_count: usize,
    /// Observations about the architecture
    pub observations: Vec<String>,
}

/// Detector for architecture gaps
pub struct ArchitectureGapsDetector {
    root: PathBuf,
}

impl ArchitectureGapsDetector {
    /// Create a new architecture gaps detector
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Get the root path
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Analyze the codebase for architecture gaps
    pub fn analyze(
        &self,
        arch_analysis: Option<&ArchitectureAnalysis>,
    ) -> AuditResult<ArchitectureGapsAnalysis> {
        let mut analysis = ArchitectureGapsAnalysis::default();

        // Detect missing abstraction layers
        self.detect_missing_abstraction_layers(&mut analysis, arch_analysis)?;

        // Detect inconsistent module boundaries
        self.detect_inconsistent_boundaries(&mut analysis)?;

        // Detect layer violations
        self.detect_layer_violations(&mut analysis, arch_analysis)?;

        // Detect direct database access from UI
        self.detect_direct_database_access(&mut analysis)?;

        // Detect circular dependencies
        self.detect_circular_dependencies(&mut analysis, arch_analysis)?;

        // Detect god modules
        self.detect_god_modules(&mut analysis)?;

        // Detect orphan modules
        self.detect_orphan_modules(&mut analysis, arch_analysis)?;

        // Calculate statistics
        analysis.total_gaps = analysis.gaps.len();
        analysis.high_severity_count = analysis
            .gaps
            .iter()
            .filter(|g| g.severity == Severity::High || g.severity == Severity::Critical)
            .count();
        analysis.medium_severity_count = analysis
            .gaps
            .iter()
            .filter(|g| g.severity == Severity::Medium)
            .count();
        analysis.low_severity_count = analysis
            .gaps
            .iter()
            .filter(|g| g.severity == Severity::Low)
            .count();

        // Count by type
        for gap in &analysis.gaps {
            *analysis
                .gap_counts
                .entry(gap.gap_type.to_string())
                .or_insert(0) += 1;
        }

        // Generate observations
        analysis.observations = self.generate_observations(&analysis);

        Ok(analysis)
    }

    /// Convert architecture gaps to AuditFindings
    pub fn to_findings(&self, analysis: &ArchitectureGapsAnalysis) -> Vec<AuditFinding> {
        let mut findings = Vec::new();
        let mut id_counter = 1;

        for gap in &analysis.gaps {
            findings.push(AuditFinding {
                id: format!("ARCH-GAP-{:03}", id_counter),
                severity: gap.severity,
                category: "architecture".to_string(),
                title: self.gap_type_to_title(&gap.gap_type),
                description: gap.description.clone(),
                affected_files: gap.files.clone(),
                recommendation: gap.recommendation.clone(),
            });
            id_counter += 1;
        }

        findings
    }

    /// Detect missing abstraction layers
    fn detect_missing_abstraction_layers(
        &self,
        analysis: &mut ArchitectureGapsAnalysis,
        arch_analysis: Option<&ArchitectureAnalysis>,
    ) -> AuditResult<()> {
        // Check if we have architecture analysis
        let layers = match arch_analysis {
            Some(arch) => &arch.layers,
            None => return Ok(()),
        };

        // Check for common missing layer patterns
        let has_presentation = layers
            .iter()
            .any(|l| l.layer_type == LayerType::Presentation);
        let has_domain = layers.iter().any(|l| l.layer_type == LayerType::Domain);
        let has_infrastructure = layers
            .iter()
            .any(|l| l.layer_type == LayerType::Infrastructure);
        let has_application = layers
            .iter()
            .any(|l| l.layer_type == LayerType::Application);
        let has_api = layers.iter().any(|l| l.layer_type == LayerType::Api);

        // Pattern: Controller directly accessing Repository (missing Service layer)
        if (has_api || has_presentation) && has_infrastructure && !has_application {
            analysis.gaps.push(ArchitectureGap {
                gap_type: ArchitectureGapType::MissingAbstractionLayer,
                files: vec![],
                line: None,
                source: Some("api/presentation".to_string()),
                target: Some("infrastructure".to_string()),
                description:
                    "No application/service layer between API/presentation and infrastructure layers."
                        .to_string(),
                severity: Severity::Medium,
                recommendation: "Consider adding a service/application layer to handle business logic and act as a mediator between presentation and data access.".to_string(),
            });
        }

        // Pattern: Missing Domain layer
        if (has_presentation || has_api) && has_infrastructure && !has_domain {
            analysis.gaps.push(ArchitectureGap {
                gap_type: ArchitectureGapType::MissingAbstractionLayer,
                files: vec![],
                line: None,
                source: None,
                target: None,
                description: "No domain/model layer detected. Business entities may be scattered."
                    .to_string(),
                severity: Severity::Low,
                recommendation:
                    "Consider creating a dedicated domain layer for business entities and rules."
                        .to_string(),
            });
        }

        // Check for direct imports from presentation to infrastructure
        if let Some(arch) = arch_analysis {
            for layer in &arch.layers {
                if layer.layer_type == LayerType::Presentation || layer.layer_type == LayerType::Api
                {
                    for dep in &layer.depends_on {
                        // Check if this dependency is an infrastructure layer
                        if let Some(dep_layer) = arch.layers.iter().find(|l| &l.name == dep) {
                            if dep_layer.layer_type == LayerType::Infrastructure {
                                analysis.gaps.push(ArchitectureGap {
                                    gap_type: ArchitectureGapType::MissingAbstractionLayer,
                                    files: vec![layer.path.clone()],
                                    line: None,
                                    source: Some(layer.name.clone()),
                                    target: Some(dep.clone()),
                                    description: format!(
                                        "{} layer directly depends on {} layer without abstraction.",
                                        layer.name, dep
                                    ),
                                    severity: Severity::Medium,
                                    recommendation: "Introduce an abstraction layer (service/use case) between presentation and infrastructure.".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Detect inconsistent module boundaries
    fn detect_inconsistent_boundaries(
        &self,
        analysis: &mut ArchitectureGapsAnalysis,
    ) -> AuditResult<()> {
        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .max_depth(Some(4))
            .build();

        // Track what each module contains
        let mut module_contents: HashMap<String, HashSet<String>> = HashMap::new();

        // Patterns for different concerns
        let controller_re = Regex::new(r"(?i)(controller|handler|endpoint|route)").unwrap();
        let service_re = Regex::new(r"(?i)(service|usecase|use_case)").unwrap();
        let repository_re = Regex::new(r"(?i)(repository|repo|dao|store)").unwrap();
        let model_re = Regex::new(r"(?i)(model|entity|domain)").unwrap();
        let ui_re = Regex::new(r"(?i)(component|view|page|screen|template)").unwrap();

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let relative_path = path.strip_prefix(&self.root).unwrap_or(path);

            // Get the top-level module
            let module = relative_path
                .components()
                .next()
                .and_then(|c| c.as_os_str().to_str())
                .unwrap_or("")
                .to_string();

            if module.is_empty() || module.starts_with('.') {
                continue;
            }

            let path_str = relative_path.to_string_lossy().to_lowercase();

            // Categorize files by their apparent concern
            let concerns = module_contents.entry(module).or_default();

            if controller_re.is_match(&path_str) {
                concerns.insert("controller".to_string());
            }
            if service_re.is_match(&path_str) {
                concerns.insert("service".to_string());
            }
            if repository_re.is_match(&path_str) {
                concerns.insert("repository".to_string());
            }
            if model_re.is_match(&path_str) {
                concerns.insert("model".to_string());
            }
            if ui_re.is_match(&path_str) {
                concerns.insert("ui".to_string());
            }
        }

        // Detect modules with mixed concerns
        for (module, concerns) in &module_contents {
            // Skip common utility directories
            if module == "src"
                || module == "lib"
                || module == "tests"
                || module == "test"
                || module == "common"
                || module == "shared"
                || module == "utils"
            {
                continue;
            }

            // Flag modules that mix UI/controller concerns with repository concerns
            if (concerns.contains("ui") || concerns.contains("controller"))
                && concerns.contains("repository")
            {
                analysis.gaps.push(ArchitectureGap {
                    gap_type: ArchitectureGapType::InconsistentModuleBoundary,
                    files: vec![PathBuf::from(module)],
                    line: None,
                    source: Some(module.clone()),
                    target: None,
                    description: format!(
                        "Module '{}' mixes presentation concerns (UI/controller) with data access (repository).",
                        module
                    ),
                    severity: Severity::Medium,
                    recommendation: "Consider separating presentation and data access into distinct modules.".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Detect layer violations
    fn detect_layer_violations(
        &self,
        analysis: &mut ArchitectureGapsAnalysis,
        arch_analysis: Option<&ArchitectureAnalysis>,
    ) -> AuditResult<()> {
        let Some(arch) = arch_analysis else {
            return Ok(());
        };

        // Use existing boundary violations from architecture analysis
        for violation in &arch.boundary_violations {
            analysis.gaps.push(ArchitectureGap {
                gap_type: ArchitectureGapType::LayerViolation,
                files: vec![violation.file.clone()],
                line: violation.line,
                source: Some(violation.from_layer.clone()),
                target: Some(violation.to_layer.clone()),
                description: violation.description.clone(),
                severity: Severity::High,
                recommendation: format!(
                    "Refactor {} to not depend directly on {}. Use dependency injection or add an abstraction layer.",
                    violation.from_layer, violation.to_layer
                ),
            });
        }

        Ok(())
    }

    /// Detect direct database access from UI/presentation layer
    fn detect_direct_database_access(
        &self,
        analysis: &mut ArchitectureGapsAnalysis,
    ) -> AuditResult<()> {
        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .build();

        // Patterns that indicate UI/presentation files
        let ui_patterns = Regex::new(
            r"(?i)(component|view|page|screen|template|ui|frontend|client|\.vue|\.jsx|\.tsx)",
        )
        .unwrap();

        // Patterns that indicate direct database access
        let db_patterns = [
            Regex::new(r"(?i)SELECT\s+.+\s+FROM\s+").unwrap(),
            Regex::new(r"(?i)INSERT\s+INTO\s+").unwrap(),
            Regex::new(r"(?i)UPDATE\s+.+\s+SET\s+").unwrap(),
            Regex::new(r"(?i)DELETE\s+FROM\s+").unwrap(),
            Regex::new(r#"(?i)\.query\s*\(\s*['"`]"#).unwrap(),
            Regex::new(r#"(?i)\.execute\s*\(\s*['"`]"#).unwrap(),
            Regex::new(r"(?i)mongoose\.(find|create|update|delete)").unwrap(),
            Regex::new(r"(?i)prisma\.\w+\.(find|create|update|delete)").unwrap(),
            Regex::new(r"(?i)sequelize\.\w+\.(find|create|update|destroy)").unwrap(),
            Regex::new(r"(?i)sqlx::(query|execute)").unwrap(),
            Regex::new(r"(?i)diesel::(insert|update|delete|select)").unwrap(),
        ];

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let path_str = path.to_string_lossy();

            // Check if this looks like a UI file
            if !ui_patterns.is_match(&path_str) {
                continue;
            }

            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Only check source files
            if !matches!(
                ext.as_str(),
                "rs" | "js" | "ts" | "tsx" | "jsx" | "vue" | "py" | "go" | "java"
            ) {
                continue;
            }

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Check for direct database patterns
            for pattern in &db_patterns {
                if pattern.is_match(&content) {
                    let relative_path = path.strip_prefix(&self.root).unwrap_or(path);
                    analysis.gaps.push(ArchitectureGap {
                        gap_type: ArchitectureGapType::DirectDatabaseAccess,
                        files: vec![relative_path.to_path_buf()],
                        line: None,
                        source: Some("ui/presentation".to_string()),
                        target: Some("database".to_string()),
                        description: format!(
                            "UI/presentation file '{}' contains direct database access patterns.",
                            relative_path.display()
                        ),
                        severity: Severity::High,
                        recommendation: "Move database operations to a repository or service layer. UI should only interact with services.".to_string(),
                    });
                    break; // Only report once per file
                }
            }
        }

        Ok(())
    }

    /// Detect circular dependencies between modules
    fn detect_circular_dependencies(
        &self,
        analysis: &mut ArchitectureGapsAnalysis,
        arch_analysis: Option<&ArchitectureAnalysis>,
    ) -> AuditResult<()> {
        let Some(arch) = arch_analysis else {
            return Ok(());
        };

        // Build dependency graph
        let mut deps: HashMap<&str, HashSet<&str>> = HashMap::new();
        for layer in &arch.layers {
            let entry = deps.entry(&layer.name).or_default();
            for dep in &layer.depends_on {
                entry.insert(dep.as_str());
            }
        }

        // Simple cycle detection: check for A -> B -> A patterns
        for (module, dependencies) in &deps {
            for dep in dependencies {
                if let Some(dep_deps) = deps.get(dep) {
                    if dep_deps.contains(module) {
                        // Found a cycle
                        analysis.gaps.push(ArchitectureGap {
                            gap_type: ArchitectureGapType::CircularDependency,
                            files: vec![PathBuf::from(*module), PathBuf::from(*dep)],
                            line: None,
                            source: Some(module.to_string()),
                            target: Some(dep.to_string()),
                            description: format!(
                                "Circular dependency detected: {} <-> {}",
                                module, dep
                            ),
                            severity: Severity::High,
                            recommendation: format!(
                                "Break the circular dependency between '{}' and '{}' by introducing an interface or restructuring the modules.",
                                module, dep
                            ),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Detect god modules (modules with too many responsibilities)
    fn detect_god_modules(&self, analysis: &mut ArchitectureGapsAnalysis) -> AuditResult<()> {
        let entries = match fs::read_dir(&self.root) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Skip hidden and common directories
            if name.starts_with('.')
                || name == "node_modules"
                || name == "target"
                || name == "vendor"
                || name == "__pycache__"
                || name == "tests"
                || name == "test"
            {
                continue;
            }

            // Count files in this module
            let walker = WalkBuilder::new(&path)
                .hidden(false)
                .git_ignore(true)
                .build();

            let file_count: usize = walker
                .flatten()
                .filter(|e| {
                    let p = e.path();
                    p.is_file()
                        && p.extension()
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
                .count();

            // Flag modules with more than 50 files as potentially being "god modules"
            if file_count > 50 {
                let relative_path = path.strip_prefix(&self.root).unwrap_or(&path);
                analysis.gaps.push(ArchitectureGap {
                    gap_type: ArchitectureGapType::GodModule,
                    files: vec![relative_path.to_path_buf()],
                    line: None,
                    source: Some(name.clone()),
                    target: None,
                    description: format!(
                        "Module '{}' contains {} files, which may indicate too many responsibilities.",
                        name, file_count
                    ),
                    severity: Severity::Low,
                    recommendation: "Consider breaking this module into smaller, more focused modules with single responsibilities.".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Detect orphan modules (modules with no clear layer assignment)
    fn detect_orphan_modules(
        &self,
        analysis: &mut ArchitectureGapsAnalysis,
        arch_analysis: Option<&ArchitectureAnalysis>,
    ) -> AuditResult<()> {
        let Some(arch) = arch_analysis else {
            return Ok(());
        };

        // Find layers with unknown type that have files
        for layer in &arch.layers {
            if layer.layer_type == LayerType::Unknown && layer.file_count > 0 {
                // Check if it's not a common utility directory
                let name_lower = layer.name.to_lowercase();
                if name_lower == "src"
                    || name_lower == "lib"
                    || name_lower == "pkg"
                    || name_lower == "internal"
                    || name_lower == "cmd"
                {
                    continue;
                }

                analysis.gaps.push(ArchitectureGap {
                    gap_type: ArchitectureGapType::OrphanModule,
                    files: vec![layer.path.clone()],
                    line: None,
                    source: Some(layer.name.clone()),
                    target: None,
                    description: format!(
                        "Module '{}' ({} files) has no clear architectural role.",
                        layer.name, layer.file_count
                    ),
                    severity: Severity::Low,
                    recommendation: "Consider renaming this module to clarify its purpose or reorganizing its contents into appropriate layers.".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Generate observations about the architecture gaps
    fn generate_observations(&self, analysis: &ArchitectureGapsAnalysis) -> Vec<String> {
        let mut observations = Vec::new();

        if analysis.total_gaps == 0 {
            observations.push("No significant architecture gaps detected.".to_string());
            return observations;
        }

        // Summary observation
        observations.push(format!(
            "Found {} architecture gap(s): {} high severity, {} medium, {} low.",
            analysis.total_gaps,
            analysis.high_severity_count,
            analysis.medium_severity_count,
            analysis.low_severity_count
        ));

        // Gap-specific observations
        if let Some(count) = analysis.gap_counts.get("layer_violation") {
            if *count > 0 {
                observations.push(format!(
                    "{} layer violation(s) detected. Review dependency directions.",
                    count
                ));
            }
        }

        if let Some(count) = analysis.gap_counts.get("missing_abstraction_layer") {
            if *count > 0 {
                observations.push(format!(
                    "{} missing abstraction layer(s). Consider adding service/application layers.",
                    count
                ));
            }
        }

        if let Some(count) = analysis.gap_counts.get("direct_database_access") {
            if *count > 0 {
                observations.push(format!(
                    "{} instance(s) of direct database access from UI layer.",
                    count
                ));
            }
        }

        if let Some(count) = analysis.gap_counts.get("circular_dependency") {
            if *count > 0 {
                observations.push(format!(
                    "{} circular dependency(ies) detected. This can lead to tight coupling.",
                    count
                ));
            }
        }

        if let Some(count) = analysis.gap_counts.get("god_module") {
            if *count > 0 {
                observations.push(format!(
                    "{} module(s) may have too many responsibilities.",
                    count
                ));
            }
        }

        observations
    }

    /// Convert gap type to human-readable title
    fn gap_type_to_title(&self, gap_type: &ArchitectureGapType) -> String {
        match gap_type {
            ArchitectureGapType::MissingAbstractionLayer => "Missing Abstraction Layer".to_string(),
            ArchitectureGapType::InconsistentModuleBoundary => {
                "Inconsistent Module Boundary".to_string()
            }
            ArchitectureGapType::LayerViolation => "Layer Violation".to_string(),
            ArchitectureGapType::DirectDatabaseAccess => {
                "Direct Database Access from UI".to_string()
            }
            ArchitectureGapType::MissingInterface => "Missing Interface".to_string(),
            ArchitectureGapType::CircularDependency => "Circular Dependency".to_string(),
            ArchitectureGapType::GodModule => "God Module (Too Many Responsibilities)".to_string(),
            ArchitectureGapType::OrphanModule => "Orphan Module (No Clear Role)".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_gap_type_display() {
        assert_eq!(
            format!("{}", ArchitectureGapType::MissingAbstractionLayer),
            "missing_abstraction_layer"
        );
        assert_eq!(
            format!("{}", ArchitectureGapType::InconsistentModuleBoundary),
            "inconsistent_module_boundary"
        );
        assert_eq!(
            format!("{}", ArchitectureGapType::LayerViolation),
            "layer_violation"
        );
        assert_eq!(
            format!("{}", ArchitectureGapType::DirectDatabaseAccess),
            "direct_database_access"
        );
        assert_eq!(
            format!("{}", ArchitectureGapType::CircularDependency),
            "circular_dependency"
        );
        assert_eq!(format!("{}", ArchitectureGapType::GodModule), "god_module");
        assert_eq!(
            format!("{}", ArchitectureGapType::OrphanModule),
            "orphan_module"
        );
    }

    #[test]
    fn test_detector_new() {
        let detector = ArchitectureGapsDetector::new(PathBuf::from("/test"));
        assert_eq!(detector.root(), &PathBuf::from("/test"));
    }

    #[test]
    fn test_analyze_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ArchitectureGapsDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        assert_eq!(analysis.total_gaps, 0);
        assert!(analysis
            .observations
            .iter()
            .any(|o| o.contains("No significant architecture gaps")));
    }

    #[test]
    fn test_detect_direct_database_access() {
        let temp_dir = TempDir::new().unwrap();

        // Create a UI component that has direct SQL
        let ui_dir = temp_dir.path().join("components");
        fs::create_dir(&ui_dir).unwrap();

        fs::write(
            ui_dir.join("UserList.tsx"),
            r#"
            import React from 'react';

            function UserList() {
                const query = "SELECT * FROM users WHERE active = true";
                // ... more code
            }
            "#,
        )
        .unwrap();

        let detector = ArchitectureGapsDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        assert!(analysis
            .gaps
            .iter()
            .any(|g| g.gap_type == ArchitectureGapType::DirectDatabaseAccess));
    }

    #[test]
    fn test_detect_god_module() {
        let temp_dir = TempDir::new().unwrap();

        // Create a module with many files
        let big_module = temp_dir.path().join("bigmodule");
        fs::create_dir(&big_module).unwrap();

        // Create 55 files
        for i in 0..55 {
            fs::write(
                big_module.join(format!("file{}.rs", i)),
                format!("pub fn func{}() {{}}", i),
            )
            .unwrap();
        }

        let detector = ArchitectureGapsDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        assert!(analysis
            .gaps
            .iter()
            .any(|g| g.gap_type == ArchitectureGapType::GodModule));
    }

    #[test]
    fn test_detect_inconsistent_boundaries() {
        let temp_dir = TempDir::new().unwrap();

        // Create a module that mixes controller and repository
        let mixed_module = temp_dir.path().join("users");
        fs::create_dir(&mixed_module).unwrap();

        fs::write(mixed_module.join("controller.rs"), "pub fn handler() {}").unwrap();
        fs::write(mixed_module.join("repository.rs"), "pub fn find() {}").unwrap();

        let detector = ArchitectureGapsDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        assert!(analysis
            .gaps
            .iter()
            .any(|g| g.gap_type == ArchitectureGapType::InconsistentModuleBoundary));
    }

    #[test]
    fn test_detect_missing_abstraction_layer() {
        let temp_dir = TempDir::new().unwrap();

        // Create a project with presentation and infrastructure but no service layer
        let controllers = temp_dir.path().join("controllers");
        let repositories = temp_dir.path().join("repositories");

        fs::create_dir(&controllers).unwrap();
        fs::create_dir(&repositories).unwrap();

        fs::write(
            controllers.join("user_controller.rs"),
            r#"
            use crate::repositories;
            pub fn get_users() {}
            "#,
        )
        .unwrap();
        fs::write(
            repositories.join("user_repository.rs"),
            "pub fn find_all() {}",
        )
        .unwrap();

        // Create mock architecture analysis
        use crate::audit::architecture::{ArchitectureAnalysis, ArchitectureLayer};

        let arch = ArchitectureAnalysis {
            layers: vec![
                ArchitectureLayer {
                    name: "controllers".to_string(),
                    path: PathBuf::from("controllers"),
                    layer_type: LayerType::Api,
                    file_count: 1,
                    depends_on: vec!["repositories".to_string()],
                },
                ArchitectureLayer {
                    name: "repositories".to_string(),
                    path: PathBuf::from("repositories"),
                    layer_type: LayerType::Infrastructure,
                    file_count: 1,
                    depends_on: vec![],
                },
            ],
            ..Default::default()
        };

        let detector = ArchitectureGapsDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(Some(&arch)).unwrap();

        // Should detect missing service layer
        assert!(analysis.gaps.iter().any(|g| {
            g.gap_type == ArchitectureGapType::MissingAbstractionLayer
                && g.description.contains("application/service layer")
        }));
    }

    #[test]
    fn test_detect_circular_dependency() {
        use crate::audit::architecture::{ArchitectureAnalysis, ArchitectureLayer};

        // Create mock architecture with circular dependency
        let arch = ArchitectureAnalysis {
            layers: vec![
                ArchitectureLayer {
                    name: "moduleA".to_string(),
                    path: PathBuf::from("moduleA"),
                    layer_type: LayerType::Application,
                    file_count: 5,
                    depends_on: vec!["moduleB".to_string()],
                },
                ArchitectureLayer {
                    name: "moduleB".to_string(),
                    path: PathBuf::from("moduleB"),
                    layer_type: LayerType::Application,
                    file_count: 5,
                    depends_on: vec!["moduleA".to_string()],
                },
            ],
            ..Default::default()
        };

        let temp_dir = TempDir::new().unwrap();
        let detector = ArchitectureGapsDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(Some(&arch)).unwrap();

        assert!(analysis
            .gaps
            .iter()
            .any(|g| g.gap_type == ArchitectureGapType::CircularDependency));
    }

    #[test]
    fn test_detect_layer_violation() {
        use crate::audit::architecture::{ArchitectureAnalysis, BoundaryViolation};

        // Create mock architecture with boundary violation
        let arch = ArchitectureAnalysis {
            boundary_violations: vec![BoundaryViolation {
                file: PathBuf::from("domain/user.rs"),
                line: Some(10),
                from_layer: "domain".to_string(),
                to_layer: "infrastructure".to_string(),
                description: "Domain layer should not depend on Infrastructure".to_string(),
            }],
            ..Default::default()
        };

        let temp_dir = TempDir::new().unwrap();
        let detector = ArchitectureGapsDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(Some(&arch)).unwrap();

        assert!(analysis
            .gaps
            .iter()
            .any(|g| g.gap_type == ArchitectureGapType::LayerViolation));
    }

    #[test]
    fn test_detect_orphan_module() {
        use crate::audit::architecture::{ArchitectureAnalysis, ArchitectureLayer};

        // Create mock architecture with orphan module
        let arch = ArchitectureAnalysis {
            layers: vec![ArchitectureLayer {
                name: "misc".to_string(),
                path: PathBuf::from("misc"),
                layer_type: LayerType::Unknown,
                file_count: 10,
                depends_on: vec![],
            }],
            ..Default::default()
        };

        let temp_dir = TempDir::new().unwrap();
        let detector = ArchitectureGapsDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(Some(&arch)).unwrap();

        assert!(analysis
            .gaps
            .iter()
            .any(|g| g.gap_type == ArchitectureGapType::OrphanModule));
    }

    #[test]
    fn test_to_findings() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ArchitectureGapsDetector::new(temp_dir.path().to_path_buf());

        let analysis = ArchitectureGapsAnalysis {
            gaps: vec![
                ArchitectureGap {
                    gap_type: ArchitectureGapType::LayerViolation,
                    files: vec![PathBuf::from("src/domain/user.rs")],
                    line: Some(10),
                    source: Some("domain".to_string()),
                    target: Some("infrastructure".to_string()),
                    description: "Domain depends on infrastructure".to_string(),
                    severity: Severity::High,
                    recommendation: "Fix the violation".to_string(),
                },
                ArchitectureGap {
                    gap_type: ArchitectureGapType::GodModule,
                    files: vec![PathBuf::from("src/bigmodule")],
                    line: None,
                    source: Some("bigmodule".to_string()),
                    target: None,
                    description: "Module has too many files".to_string(),
                    severity: Severity::Low,
                    recommendation: "Split the module".to_string(),
                },
            ],
            total_gaps: 2,
            high_severity_count: 1,
            medium_severity_count: 0,
            low_severity_count: 1,
            ..Default::default()
        };

        let findings = detector.to_findings(&analysis);

        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].id, "ARCH-GAP-001");
        assert_eq!(findings[0].category, "architecture");
        assert_eq!(findings[0].severity, Severity::High);
        assert_eq!(findings[1].id, "ARCH-GAP-002");
        assert_eq!(findings[1].severity, Severity::Low);
    }

    #[test]
    fn test_analysis_serialization() {
        let analysis = ArchitectureGapsAnalysis {
            gaps: vec![ArchitectureGap {
                gap_type: ArchitectureGapType::LayerViolation,
                files: vec![PathBuf::from("src/test.rs")],
                line: Some(42),
                source: Some("domain".to_string()),
                target: Some("infra".to_string()),
                description: "Test violation".to_string(),
                severity: Severity::Medium,
                recommendation: "Fix it".to_string(),
            }],
            gap_counts: [("layer_violation".to_string(), 1)].into_iter().collect(),
            total_gaps: 1,
            high_severity_count: 0,
            medium_severity_count: 1,
            low_severity_count: 0,
            observations: vec!["Test observation".to_string()],
        };

        let json = serde_json::to_string(&analysis).unwrap();
        let deserialized: ArchitectureGapsAnalysis = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_gaps, 1);
        assert_eq!(deserialized.gaps.len(), 1);
        assert_eq!(
            deserialized.gaps[0].gap_type,
            ArchitectureGapType::LayerViolation
        );
    }

    #[test]
    fn test_generate_observations_no_gaps() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ArchitectureGapsDetector::new(temp_dir.path().to_path_buf());

        let analysis = ArchitectureGapsAnalysis::default();
        let observations = detector.generate_observations(&analysis);

        assert!(observations
            .iter()
            .any(|o| o.contains("No significant architecture gaps")));
    }

    #[test]
    fn test_generate_observations_with_gaps() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ArchitectureGapsDetector::new(temp_dir.path().to_path_buf());

        let mut gap_counts = HashMap::new();
        gap_counts.insert("layer_violation".to_string(), 2);
        gap_counts.insert("circular_dependency".to_string(), 1);

        let analysis = ArchitectureGapsAnalysis {
            total_gaps: 3,
            high_severity_count: 3,
            medium_severity_count: 0,
            low_severity_count: 0,
            gap_counts,
            gaps: vec![],
            observations: vec![],
        };

        let observations = detector.generate_observations(&analysis);

        assert!(observations
            .iter()
            .any(|o| o.contains("3 architecture gap(s)")));
        assert!(observations
            .iter()
            .any(|o| o.contains("2 layer violation(s)")));
        assert!(observations
            .iter()
            .any(|o| o.contains("1 circular dependency")));
    }

    #[test]
    fn test_gap_type_to_title() {
        let temp_dir = TempDir::new().unwrap();
        let detector = ArchitectureGapsDetector::new(temp_dir.path().to_path_buf());

        assert_eq!(
            detector.gap_type_to_title(&ArchitectureGapType::MissingAbstractionLayer),
            "Missing Abstraction Layer"
        );
        assert_eq!(
            detector.gap_type_to_title(&ArchitectureGapType::LayerViolation),
            "Layer Violation"
        );
        assert_eq!(
            detector.gap_type_to_title(&ArchitectureGapType::CircularDependency),
            "Circular Dependency"
        );
        assert_eq!(
            detector.gap_type_to_title(&ArchitectureGapType::GodModule),
            "God Module (Too Many Responsibilities)"
        );
    }

    #[test]
    fn test_no_false_positive_for_service_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create services directory (not UI)
        let services = temp_dir.path().join("services");
        fs::create_dir(&services).unwrap();

        // This should NOT trigger direct database access warning
        fs::write(
            services.join("user_service.rs"),
            r#"
            pub fn get_users() {
                let query = "SELECT * FROM users";
            }
            "#,
        )
        .unwrap();

        let detector = ArchitectureGapsDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        // Services directory is not UI, so no direct database access gap
        assert!(!analysis
            .gaps
            .iter()
            .any(|g| g.gap_type == ArchitectureGapType::DirectDatabaseAccess));
    }

    #[test]
    fn test_skip_common_directories_for_god_module() {
        let temp_dir = TempDir::new().unwrap();

        // Create tests directory with many files - should not be flagged
        let tests = temp_dir.path().join("tests");
        fs::create_dir(&tests).unwrap();

        for i in 0..60 {
            fs::write(
                tests.join(format!("test{}.rs", i)),
                format!("pub fn test{}() {{}}", i),
            )
            .unwrap();
        }

        let detector = ArchitectureGapsDetector::new(temp_dir.path().to_path_buf());
        let analysis = detector.analyze(None).unwrap();

        // tests directory should be skipped
        assert!(!analysis.gaps.iter().any(|g| {
            g.gap_type == ArchitectureGapType::GodModule
                && g.files
                    .iter()
                    .any(|f| f.to_string_lossy().contains("tests"))
        }));
    }
}
