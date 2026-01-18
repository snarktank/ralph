//! Quality profile definitions for Ralph.
//!
//! This module defines the data structures for quality profiles that can be
//! loaded from TOML configuration files.

// Allow dead_code for now - these types will be used in future stories (US-009+)
#![allow(dead_code)]

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// The level of a quality profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProfileLevel {
    /// Minimal quality gates for rapid prototyping
    Minimal,
    /// Standard quality gates for production-ready features
    #[default]
    Standard,
    /// Comprehensive quality gates for critical features
    Comprehensive,
}

/// Documentation requirements for a profile.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DocumentationConfig {
    /// Whether documentation is required
    #[serde(default)]
    pub required: bool,
    /// Whether README updates are required
    #[serde(default)]
    pub readme: bool,
    /// Whether inline comments are required
    #[serde(default)]
    pub inline_comments: bool,
}

/// Testing requirements for a profile.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TestingConfig {
    /// Whether unit tests are required
    #[serde(default)]
    pub unit_tests: bool,
    /// Whether integration tests are required
    #[serde(default)]
    pub integration_tests: bool,
    /// Minimum code coverage percentage (0-100)
    #[serde(default)]
    pub coverage_threshold: u8,
}

/// CI requirements for a profile.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CiConfig {
    /// Whether CI is required
    #[serde(default)]
    pub required: bool,
    /// Whether format checking is required
    #[serde(default)]
    pub format_check: bool,
    /// Whether lint checking is required
    #[serde(default)]
    pub lint_check: bool,
}

/// Security requirements for a profile.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SecurityConfig {
    /// Whether cargo-audit is required
    #[serde(default)]
    pub cargo_audit: bool,
    /// Whether cargo-deny is required
    #[serde(default)]
    pub cargo_deny: bool,
    /// Whether SAST (Static Application Security Testing) is required
    #[serde(default)]
    pub sast: bool,
}

/// Blog generation configuration for a profile.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BlogConfig {
    /// Whether to generate a blog post
    #[serde(default)]
    pub generate: bool,
    /// Template to use for blog generation
    #[serde(default)]
    pub template: Option<String>,
}

/// Audit section toggles for a profile.
#[derive(Debug, Clone, Deserialize)]
pub struct AuditSections {
    /// Whether to audit code structure
    #[serde(default = "default_true")]
    pub structure: bool,
    /// Whether to audit code patterns
    #[serde(default = "default_true")]
    pub patterns: bool,
    /// Whether to audit API design
    #[serde(default = "default_true")]
    pub api: bool,
    /// Whether to audit dependencies
    #[serde(default = "default_true")]
    pub deps: bool,
    /// Whether to audit tests
    #[serde(default = "default_true")]
    pub tests: bool,
    /// Whether to audit documentation
    #[serde(default = "default_true")]
    pub docs: bool,
    /// Whether to audit architecture
    #[serde(default = "default_true")]
    pub arch: bool,
}

impl Default for AuditSections {
    fn default() -> Self {
        Self {
            structure: true,
            patterns: true,
            api: true,
            deps: true,
            tests: true,
            docs: true,
            arch: true,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Audit configuration for a profile.
#[derive(Debug, Clone, Deserialize)]
pub struct AuditConfig {
    /// Whether auditing is enabled for this profile
    #[serde(default)]
    pub enabled: bool,
    /// Maximum number of critical findings allowed before failing
    #[serde(default = "default_max_critical")]
    pub max_critical_findings: u32,
    /// Maximum number of high-severity findings allowed before failing
    #[serde(default = "default_max_high")]
    pub max_high_findings: u32,
    /// Section toggles for the audit
    #[serde(default)]
    pub sections: AuditSections,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_critical_findings: 0,
            max_high_findings: 5,
            sections: AuditSections::default(),
        }
    }
}

fn default_max_critical() -> u32 {
    0
}

fn default_max_high() -> u32 {
    5
}

/// A quality profile containing all configuration sections.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Profile {
    /// Human-readable description of this profile
    #[serde(default)]
    pub description: String,
    /// Documentation requirements
    #[serde(default)]
    pub documentation: DocumentationConfig,
    /// Testing requirements
    #[serde(default)]
    pub testing: TestingConfig,
    /// CI requirements
    #[serde(default)]
    pub ci: CiConfig,
    /// Security requirements
    #[serde(default)]
    pub security: SecurityConfig,
    /// Blog generation configuration
    #[serde(default)]
    pub blog: BlogConfig,
    /// Audit configuration
    #[serde(default)]
    pub audit: AuditConfig,
}

/// Errors that can occur when loading quality configuration.
#[derive(Debug, Error)]
pub enum QualityConfigError {
    /// The configuration file was not found.
    #[error("configuration file not found: {0}")]
    FileNotFound(String),

    /// The configuration file could not be parsed.
    #[error("failed to parse configuration: {0}")]
    ParseError(#[from] ConfigError),

    /// The configuration file path is invalid.
    #[error("invalid configuration path: {0}")]
    InvalidPath(String),
}

/// Root configuration structure containing all quality profiles.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct QualityConfig {
    /// Map of profile names to their configurations
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
}

impl QualityConfig {
    /// Load quality configuration from a file path.
    ///
    /// This function loads the configuration from the specified TOML file and
    /// supports environment variable overrides with the `RALPH_` prefix.
    ///
    /// # Environment Variable Overrides
    ///
    /// Environment variables can override configuration values using the format:
    /// `RALPH__<SECTION>__<KEY>` (e.g., `RALPH__PROFILES__MINIMAL__TESTING__COVERAGE_THRESHOLD=50`)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The configuration file does not exist
    /// - The configuration file cannot be parsed
    /// - The path is invalid
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ralphmacchio::quality::QualityConfig;
    ///
    /// let config = QualityConfig::load("quality/ralph-quality.toml")?;
    /// # Ok::<(), ralphmacchio::quality::profiles::QualityConfigError>(())
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, QualityConfigError> {
        let path = path.as_ref();

        // Verify the path is valid
        let path_str = path
            .to_str()
            .ok_or_else(|| QualityConfigError::InvalidPath(format!("{:?}", path)))?;

        // Check if the file exists
        if !path.exists() {
            return Err(QualityConfigError::FileNotFound(path_str.to_string()));
        }

        // Build configuration with file and environment variable sources
        let config = Config::builder()
            // Load from the specified file
            .add_source(File::with_name(path_str))
            // Add environment variable overrides with RALPH_ prefix
            // Use double underscore as separator for nested keys
            .add_source(
                Environment::with_prefix("RALPH")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        // Deserialize into QualityConfig
        let quality_config: QualityConfig = config.try_deserialize()?;

        Ok(quality_config)
    }

    /// Get a profile by its level.
    pub fn get_profile(&self, level: ProfileLevel) -> Option<&Profile> {
        let name = match level {
            ProfileLevel::Minimal => "minimal",
            ProfileLevel::Standard => "standard",
            ProfileLevel::Comprehensive => "comprehensive",
        };
        self.profiles.get(name)
    }

    /// Get a profile by name.
    pub fn get_profile_by_name(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    /// List all available profile names.
    pub fn profile_names(&self) -> Vec<&str> {
        self.profiles.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_level_default() {
        assert_eq!(ProfileLevel::default(), ProfileLevel::Standard);
    }

    #[test]
    fn test_deserialize_minimal_profile() {
        let toml_str = r#"
            [profiles.minimal]
            description = "Test profile"

            [profiles.minimal.documentation]
            required = false

            [profiles.minimal.testing]
            unit_tests = true
            coverage_threshold = 0

            [profiles.minimal.ci]
            required = false

            [profiles.minimal.security]
            cargo_audit = false

            [profiles.minimal.blog]
            generate = false
        "#;

        let config: QualityConfig = toml::from_str(toml_str).unwrap();
        let profile = config.get_profile(ProfileLevel::Minimal).unwrap();

        assert!(!profile.documentation.required);
        assert!(profile.testing.unit_tests);
        assert_eq!(profile.testing.coverage_threshold, 0);
    }

    #[test]
    fn test_load_file_not_found() {
        let result = QualityConfig::load("nonexistent/path/config.toml");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, QualityConfigError::FileNotFound(_)));
    }

    #[test]
    fn test_load_from_actual_config() {
        // Test loading the actual quality config file
        let result = QualityConfig::load("quality/ralph-quality.toml");
        assert!(result.is_ok(), "Failed to load config: {:?}", result);

        let config = result.unwrap();

        // Verify we have all three profiles
        assert!(config.get_profile(ProfileLevel::Minimal).is_some());
        assert!(config.get_profile(ProfileLevel::Standard).is_some());
        assert!(config.get_profile(ProfileLevel::Comprehensive).is_some());

        // Verify minimal profile has 0 coverage threshold
        let minimal = config.get_profile(ProfileLevel::Minimal).unwrap();
        assert_eq!(minimal.testing.coverage_threshold, 0);

        // Verify standard profile has 70 coverage threshold
        let standard = config.get_profile(ProfileLevel::Standard).unwrap();
        assert_eq!(standard.testing.coverage_threshold, 70);

        // Verify comprehensive profile has 90 coverage threshold and blog enabled
        let comprehensive = config.get_profile(ProfileLevel::Comprehensive).unwrap();
        assert_eq!(comprehensive.testing.coverage_threshold, 90);
        assert!(comprehensive.blog.generate);
    }

    #[test]
    fn test_error_display() {
        let err = QualityConfigError::FileNotFound("test.toml".to_string());
        assert_eq!(err.to_string(), "configuration file not found: test.toml");

        let err = QualityConfigError::InvalidPath("invalid/path".to_string());
        assert_eq!(err.to_string(), "invalid configuration path: invalid/path");
    }

    #[test]
    fn test_audit_config_defaults() {
        let audit = AuditConfig::default();
        assert!(!audit.enabled);
        assert_eq!(audit.max_critical_findings, 0);
        assert_eq!(audit.max_high_findings, 5);

        // All sections should be enabled by default
        assert!(audit.sections.structure);
        assert!(audit.sections.patterns);
        assert!(audit.sections.api);
        assert!(audit.sections.deps);
        assert!(audit.sections.tests);
        assert!(audit.sections.docs);
        assert!(audit.sections.arch);
    }

    #[test]
    fn test_audit_sections_defaults() {
        let sections = AuditSections::default();
        assert!(sections.structure);
        assert!(sections.patterns);
        assert!(sections.api);
        assert!(sections.deps);
        assert!(sections.tests);
        assert!(sections.docs);
        assert!(sections.arch);
    }

    #[test]
    fn test_deserialize_audit_config() {
        let toml_str = r#"
            [profiles.test]
            description = "Test profile with audit"

            [profiles.test.audit]
            enabled = true
            max_critical_findings = 2
            max_high_findings = 10

            [profiles.test.audit.sections]
            structure = true
            patterns = true
            api = false
            deps = true
            tests = false
            docs = false
            arch = true
        "#;

        let config: QualityConfig = toml::from_str(toml_str).unwrap();
        let profile = config.get_profile_by_name("test").unwrap();

        assert!(profile.audit.enabled);
        assert_eq!(profile.audit.max_critical_findings, 2);
        assert_eq!(profile.audit.max_high_findings, 10);

        assert!(profile.audit.sections.structure);
        assert!(profile.audit.sections.patterns);
        assert!(!profile.audit.sections.api);
        assert!(profile.audit.sections.deps);
        assert!(!profile.audit.sections.tests);
        assert!(!profile.audit.sections.docs);
        assert!(profile.audit.sections.arch);
    }

    #[test]
    fn test_load_audit_from_actual_config() {
        let result = QualityConfig::load("quality/ralph-quality.toml");
        assert!(result.is_ok(), "Failed to load config: {:?}", result);

        let config = result.unwrap();

        // Verify minimal profile audit settings
        let minimal = config.get_profile(ProfileLevel::Minimal).unwrap();
        assert!(!minimal.audit.enabled);
        assert_eq!(minimal.audit.max_critical_findings, 0);
        assert_eq!(minimal.audit.max_high_findings, 10);
        assert!(minimal.audit.sections.structure);
        assert!(minimal.audit.sections.patterns);
        assert!(!minimal.audit.sections.api);

        // Verify standard profile audit settings
        let standard = config.get_profile(ProfileLevel::Standard).unwrap();
        assert!(standard.audit.enabled);
        assert_eq!(standard.audit.max_critical_findings, 0);
        assert_eq!(standard.audit.max_high_findings, 5);
        assert!(standard.audit.sections.structure);
        assert!(standard.audit.sections.tests);
        assert!(!standard.audit.sections.docs);

        // Verify comprehensive profile audit settings
        let comprehensive = config.get_profile(ProfileLevel::Comprehensive).unwrap();
        assert!(comprehensive.audit.enabled);
        assert_eq!(comprehensive.audit.max_critical_findings, 0);
        assert_eq!(comprehensive.audit.max_high_findings, 0);
        assert!(comprehensive.audit.sections.structure);
        assert!(comprehensive.audit.sections.docs);
        assert!(comprehensive.audit.sections.arch);
    }

    #[test]
    fn test_audit_config_without_sections_uses_defaults() {
        let toml_str = r#"
            [profiles.test]
            description = "Test profile with audit but no sections"

            [profiles.test.audit]
            enabled = true
            max_critical_findings = 1
            max_high_findings = 3
        "#;

        let config: QualityConfig = toml::from_str(toml_str).unwrap();
        let profile = config.get_profile_by_name("test").unwrap();

        assert!(profile.audit.enabled);
        assert_eq!(profile.audit.max_critical_findings, 1);
        assert_eq!(profile.audit.max_high_findings, 3);

        // Sections should use defaults (all true)
        assert!(profile.audit.sections.structure);
        assert!(profile.audit.sections.patterns);
        assert!(profile.audit.sections.api);
        assert!(profile.audit.sections.deps);
        assert!(profile.audit.sections.tests);
        assert!(profile.audit.sections.docs);
        assert!(profile.audit.sections.arch);
    }

    #[test]
    fn test_profile_without_audit_uses_defaults() {
        let toml_str = r#"
            [profiles.test]
            description = "Test profile without audit section"

            [profiles.test.documentation]
            required = true
        "#;

        let config: QualityConfig = toml::from_str(toml_str).unwrap();
        let profile = config.get_profile_by_name("test").unwrap();

        // Audit should use defaults
        assert!(!profile.audit.enabled);
        assert_eq!(profile.audit.max_critical_findings, 0);
        assert_eq!(profile.audit.max_high_findings, 5);
    }
}
