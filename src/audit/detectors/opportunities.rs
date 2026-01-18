//! Opportunity detection for identifying complementary features.
//!
//! This module analyzes the codebase to suggest complementary features based on:
//! - Existing patterns (e.g., has quality gates but no audit command)
//! - Missing common features (e.g., has API but no health check)
//! - Enhancement opportunities (e.g., has tests but no coverage reporting)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::audit::api::ApiAnalysis;
use crate::audit::documentation::DocumentationAnalysis;
use crate::audit::testing::TestAnalysis;
use crate::audit::{AuditResult, Complexity, FeatureOpportunity, SuggestedStory};

use super::ArchitectureGapsAnalysis;

/// Type of opportunity pattern detected
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityType {
    /// Has CI/CD but missing audit command
    MissingAuditCommand,
    /// Has API but missing health check endpoint
    MissingHealthCheck,
    /// Has API but missing OpenAPI/Swagger documentation
    MissingApiDocs,
    /// Has tests but no coverage reporting
    MissingCoverageReporting,
    /// Has tests but no benchmarks
    MissingBenchmarks,
    /// Has CLI but no shell completions
    MissingShellCompletions,
    /// Has CLI but no man pages
    MissingManPages,
    /// Has configuration but no validation
    MissingConfigValidation,
    /// Has logging but no structured logging
    MissingStructuredLogging,
    /// Has errors but no error catalog
    MissingErrorCatalog,
    /// Has multiple services but no service discovery
    MissingServiceDiscovery,
    /// Has database but no migrations
    MissingMigrations,
    /// Has Docker but no docker-compose
    MissingDockerCompose,
    /// Has tests but missing integration tests
    MissingIntegrationTests,
    /// Has tests but missing e2e tests
    MissingE2eTests,
    /// Has API but missing rate limiting
    MissingRateLimiting,
    /// Has API but missing authentication
    MissingAuthentication,
    /// Has code but missing linting configuration
    MissingLinting,
    /// Has code but missing pre-commit hooks
    MissingPreCommitHooks,
}

impl std::fmt::Display for OpportunityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpportunityType::MissingAuditCommand => write!(f, "missing_audit_command"),
            OpportunityType::MissingHealthCheck => write!(f, "missing_health_check"),
            OpportunityType::MissingApiDocs => write!(f, "missing_api_docs"),
            OpportunityType::MissingCoverageReporting => write!(f, "missing_coverage_reporting"),
            OpportunityType::MissingBenchmarks => write!(f, "missing_benchmarks"),
            OpportunityType::MissingShellCompletions => write!(f, "missing_shell_completions"),
            OpportunityType::MissingManPages => write!(f, "missing_man_pages"),
            OpportunityType::MissingConfigValidation => write!(f, "missing_config_validation"),
            OpportunityType::MissingStructuredLogging => write!(f, "missing_structured_logging"),
            OpportunityType::MissingErrorCatalog => write!(f, "missing_error_catalog"),
            OpportunityType::MissingServiceDiscovery => write!(f, "missing_service_discovery"),
            OpportunityType::MissingMigrations => write!(f, "missing_migrations"),
            OpportunityType::MissingDockerCompose => write!(f, "missing_docker_compose"),
            OpportunityType::MissingIntegrationTests => write!(f, "missing_integration_tests"),
            OpportunityType::MissingE2eTests => write!(f, "missing_e2e_tests"),
            OpportunityType::MissingRateLimiting => write!(f, "missing_rate_limiting"),
            OpportunityType::MissingAuthentication => write!(f, "missing_authentication"),
            OpportunityType::MissingLinting => write!(f, "missing_linting"),
            OpportunityType::MissingPreCommitHooks => write!(f, "missing_pre_commit_hooks"),
        }
    }
}

/// An opportunity pattern that can be matched against audit results
#[derive(Debug, Clone)]
pub struct OpportunityPattern {
    /// Type of opportunity
    pub opportunity_type: OpportunityType,
    /// Title of the opportunity
    pub title: String,
    /// Rationale for why this feature would be valuable
    pub rationale: String,
    /// Complexity of implementing this feature
    pub complexity: Complexity,
    /// Suggested user stories
    pub stories: Vec<SuggestedStory>,
}

/// Context for opportunity detection containing all analysis results
#[derive(Debug, Clone, Default)]
pub struct OpportunityContext {
    /// API analysis results
    pub api: Option<ApiAnalysis>,
    /// Test analysis results
    pub test: Option<TestAnalysis>,
    /// Documentation analysis results
    pub documentation: Option<DocumentationAnalysis>,
    /// Architecture gaps analysis results
    pub architecture_gaps: Option<ArchitectureGapsAnalysis>,
    /// Whether CI/CD configuration exists
    pub has_ci_cd: bool,
    /// Whether Dockerfile exists
    pub has_dockerfile: bool,
    /// Whether docker-compose exists
    pub has_docker_compose: bool,
    /// Whether linting configuration exists
    pub has_linting_config: bool,
    /// Whether pre-commit hooks exist
    pub has_pre_commit_hooks: bool,
    /// Whether OpenAPI/Swagger docs exist
    pub has_openapi_docs: bool,
    /// Whether health check endpoint exists
    pub has_health_check: bool,
    /// Whether database migrations exist
    pub has_migrations: bool,
    /// Whether coverage reporting is configured
    pub has_coverage_config: bool,
    /// Whether benchmarks exist
    pub has_benchmarks: bool,
    /// Whether shell completions exist
    pub has_shell_completions: bool,
    /// Whether structured logging is used
    pub has_structured_logging: bool,
    /// Whether rate limiting is implemented
    pub has_rate_limiting: bool,
    /// Whether authentication is implemented
    pub has_authentication: bool,
}

/// Complete opportunity detection analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpportunityAnalysis {
    /// List of detected opportunities
    pub opportunities: Vec<FeatureOpportunity>,
    /// Total number of opportunities
    pub total_opportunities: usize,
    /// High-value opportunities (low complexity, high impact)
    pub high_value_count: usize,
    /// Observations about opportunities
    pub observations: Vec<String>,
}

/// Detector for feature opportunities
pub struct OpportunityDetector {
    root: PathBuf,
}

impl OpportunityDetector {
    /// Create a new opportunity detector
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Get the root path
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Analyze the codebase for feature opportunities
    pub fn analyze(&self, context: &OpportunityContext) -> AuditResult<OpportunityAnalysis> {
        let mut analysis = OpportunityAnalysis::default();
        let mut id_counter = 1;

        // Get all patterns
        let patterns = self.get_all_patterns();

        // Match each pattern against the context
        for pattern in patterns {
            if self.matches_pattern(&pattern, context) {
                analysis.opportunities.push(FeatureOpportunity {
                    id: format!("FEAT-{:03}", id_counter),
                    title: pattern.title,
                    rationale: pattern.rationale,
                    complexity: pattern.complexity,
                    suggested_stories: pattern.stories,
                });
                id_counter += 1;
            }
        }

        // Calculate statistics
        analysis.total_opportunities = analysis.opportunities.len();
        analysis.high_value_count = analysis
            .opportunities
            .iter()
            .filter(|o| o.complexity == Complexity::Low)
            .count();

        // Generate observations
        analysis.observations = self.generate_observations(&analysis, context);

        Ok(analysis)
    }

    /// Build context from file system and analysis results
    pub fn build_context(
        &self,
        api: Option<&ApiAnalysis>,
        test: Option<&TestAnalysis>,
        documentation: Option<&DocumentationAnalysis>,
        architecture_gaps: Option<&ArchitectureGapsAnalysis>,
    ) -> AuditResult<OpportunityContext> {
        let mut context = OpportunityContext {
            api: api.cloned(),
            test: test.cloned(),
            documentation: documentation.cloned(),
            architecture_gaps: architecture_gaps.cloned(),
            ..Default::default()
        };

        // Check for CI/CD configuration
        context.has_ci_cd = self.root.join(".github/workflows").exists()
            || self.root.join(".gitlab-ci.yml").exists()
            || self.root.join(".circleci").exists()
            || self.root.join("Jenkinsfile").exists()
            || self.root.join(".travis.yml").exists()
            || self.root.join("azure-pipelines.yml").exists();

        // Check for Docker configuration
        context.has_dockerfile =
            self.root.join("Dockerfile").exists() || self.root.join("dockerfile").exists();
        context.has_docker_compose = self.root.join("docker-compose.yml").exists()
            || self.root.join("docker-compose.yaml").exists()
            || self.root.join("compose.yml").exists()
            || self.root.join("compose.yaml").exists();

        // Check for linting configuration
        context.has_linting_config = self.root.join(".eslintrc").exists()
            || self.root.join(".eslintrc.js").exists()
            || self.root.join(".eslintrc.json").exists()
            || self.root.join("eslint.config.js").exists()
            || self.root.join(".prettierrc").exists()
            || self.root.join("rustfmt.toml").exists()
            || self.root.join(".rustfmt.toml").exists()
            || self.root.join("clippy.toml").exists()
            || self.root.join(".clippy.toml").exists()
            || self.root.join("pyproject.toml").exists()
            || self.root.join(".flake8").exists()
            || self.root.join(".golangci.yml").exists();

        // Check for pre-commit hooks
        context.has_pre_commit_hooks = self.root.join(".pre-commit-config.yaml").exists()
            || self.root.join(".husky").exists()
            || self.root.join(".git/hooks/pre-commit").exists();

        // Check for OpenAPI/Swagger documentation
        context.has_openapi_docs = self.root.join("openapi.yaml").exists()
            || self.root.join("openapi.yml").exists()
            || self.root.join("openapi.json").exists()
            || self.root.join("swagger.yaml").exists()
            || self.root.join("swagger.yml").exists()
            || self.root.join("swagger.json").exists()
            || self.root.join("docs/api").exists();

        // Check for health check endpoint
        if let Some(api) = api {
            context.has_health_check = api.endpoints.iter().any(|e| {
                let path_lower = e.path.to_lowercase();
                path_lower.contains("/health")
                    || path_lower.contains("/healthz")
                    || path_lower.contains("/ready")
                    || path_lower.contains("/readyz")
                    || path_lower.contains("/live")
                    || path_lower.contains("/livez")
                    || path_lower.contains("/ping")
                    || path_lower.contains("/status")
            });

            // Check for rate limiting patterns in API
            context.has_rate_limiting = api.endpoints.iter().any(|_e| false); // Would need content analysis

            // Check for authentication patterns
            context.has_authentication = api.endpoints.iter().any(|e| {
                e.path.contains("/auth") || e.path.contains("/login") || e.path.contains("/oauth")
            });
        }

        // Check for migrations
        context.has_migrations = self.root.join("migrations").exists()
            || self.root.join("db/migrations").exists()
            || self.root.join("database/migrations").exists()
            || self.root.join("src/migrations").exists();

        // Check for coverage configuration
        context.has_coverage_config = self.root.join("codecov.yml").exists()
            || self.root.join(".codecov.yml").exists()
            || self.root.join("lcov.info").exists()
            || self.root.join("coverage").exists()
            || self.root.join(".nyc_output").exists()
            || self.root.join("tarpaulin.toml").exists();

        // Check for benchmarks
        if let Some(test) = test {
            context.has_benchmarks = test
                .test_patterns
                .iter()
                .any(|p| p.pattern == crate::audit::testing::TestPattern::Benchmark);
        }
        // Also check for benchmark directories
        if !context.has_benchmarks {
            context.has_benchmarks = self.root.join("benches").exists()
                || self.root.join("benchmarks").exists()
                || self.root.join("bench").exists();
        }

        // Check for shell completions
        context.has_shell_completions = self.root.join("completions").exists()
            || self.root.join("shell-completions").exists()
            || self.root.join("contrib/completions").exists();

        // Check for structured logging (would need content analysis, simplified check)
        context.has_structured_logging = false; // Default to false, would need content analysis

        Ok(context)
    }

    /// Get all opportunity patterns
    fn get_all_patterns(&self) -> Vec<OpportunityPattern> {
        vec![
            // Missing audit command pattern
            OpportunityPattern {
                opportunity_type: OpportunityType::MissingAuditCommand,
                title: "Add Codebase Audit Command".to_string(),
                rationale: "Project has CI/CD but no audit command to analyze code quality, architecture, and potential improvements.".to_string(),
                complexity: Complexity::Medium,
                stories: vec![
                    SuggestedStory {
                        title: "Implement audit CLI command".to_string(),
                        description: "Add a CLI command that runs comprehensive codebase analysis.".to_string(),
                        acceptance_criteria: vec![
                            "Command accepts path to analyze".to_string(),
                            "Outputs findings in JSON and human-readable formats".to_string(),
                            "Integrates with CI/CD pipeline".to_string(),
                        ],
                        priority: 1,
                    },
                ],
            },
            // Missing health check pattern
            OpportunityPattern {
                opportunity_type: OpportunityType::MissingHealthCheck,
                title: "Add Health Check Endpoint".to_string(),
                rationale: "API exists but lacks health check endpoint for monitoring and orchestration.".to_string(),
                complexity: Complexity::Low,
                stories: vec![
                    SuggestedStory {
                        title: "Implement health check endpoint".to_string(),
                        description: "Add /health endpoint that returns service health status.".to_string(),
                        acceptance_criteria: vec![
                            "GET /health returns 200 when healthy".to_string(),
                            "Includes checks for database connectivity".to_string(),
                            "Includes checks for external dependencies".to_string(),
                            "Returns structured JSON response".to_string(),
                        ],
                        priority: 1,
                    },
                ],
            },
            // Missing API documentation pattern
            OpportunityPattern {
                opportunity_type: OpportunityType::MissingApiDocs,
                title: "Add OpenAPI Documentation".to_string(),
                rationale: "API endpoints exist but lack OpenAPI/Swagger documentation for consumers.".to_string(),
                complexity: Complexity::Medium,
                stories: vec![
                    SuggestedStory {
                        title: "Generate OpenAPI specification".to_string(),
                        description: "Create OpenAPI 3.0 specification for all endpoints.".to_string(),
                        acceptance_criteria: vec![
                            "All endpoints documented in OpenAPI format".to_string(),
                            "Request/response schemas defined".to_string(),
                            "Authentication requirements documented".to_string(),
                        ],
                        priority: 1,
                    },
                    SuggestedStory {
                        title: "Add Swagger UI".to_string(),
                        description: "Serve interactive API documentation via Swagger UI.".to_string(),
                        acceptance_criteria: vec![
                            "Swagger UI accessible at /docs or /swagger".to_string(),
                            "Try-it-out functionality works".to_string(),
                        ],
                        priority: 2,
                    },
                ],
            },
            // Missing coverage reporting pattern
            OpportunityPattern {
                opportunity_type: OpportunityType::MissingCoverageReporting,
                title: "Add Test Coverage Reporting".to_string(),
                rationale: "Tests exist but no coverage reporting is configured to track test coverage over time.".to_string(),
                complexity: Complexity::Low,
                stories: vec![
                    SuggestedStory {
                        title: "Configure coverage reporting".to_string(),
                        description: "Set up test coverage collection and reporting.".to_string(),
                        acceptance_criteria: vec![
                            "Coverage reports generated on test runs".to_string(),
                            "Coverage integrated with CI/CD".to_string(),
                            "Coverage badge added to README".to_string(),
                        ],
                        priority: 1,
                    },
                ],
            },
            // Missing benchmarks pattern
            OpportunityPattern {
                opportunity_type: OpportunityType::MissingBenchmarks,
                title: "Add Performance Benchmarks".to_string(),
                rationale: "Tests exist but no benchmarks to track performance over time.".to_string(),
                complexity: Complexity::Medium,
                stories: vec![
                    SuggestedStory {
                        title: "Implement benchmark suite".to_string(),
                        description: "Add benchmarks for critical code paths.".to_string(),
                        acceptance_criteria: vec![
                            "Benchmarks for key algorithms/operations".to_string(),
                            "Benchmark results tracked over time".to_string(),
                            "Performance regression detection in CI".to_string(),
                        ],
                        priority: 2,
                    },
                ],
            },
            // Missing shell completions pattern
            OpportunityPattern {
                opportunity_type: OpportunityType::MissingShellCompletions,
                title: "Add Shell Completions".to_string(),
                rationale: "CLI commands exist but no shell completions for better UX.".to_string(),
                complexity: Complexity::Low,
                stories: vec![
                    SuggestedStory {
                        title: "Generate shell completions".to_string(),
                        description: "Add shell completion scripts for bash, zsh, and fish.".to_string(),
                        acceptance_criteria: vec![
                            "Bash completions generated".to_string(),
                            "Zsh completions generated".to_string(),
                            "Fish completions generated".to_string(),
                            "Installation instructions documented".to_string(),
                        ],
                        priority: 3,
                    },
                ],
            },
            // Missing docker-compose pattern
            OpportunityPattern {
                opportunity_type: OpportunityType::MissingDockerCompose,
                title: "Add Docker Compose Configuration".to_string(),
                rationale: "Dockerfile exists but no docker-compose for easy local development.".to_string(),
                complexity: Complexity::Low,
                stories: vec![
                    SuggestedStory {
                        title: "Create docker-compose.yml".to_string(),
                        description: "Add docker-compose configuration for local development.".to_string(),
                        acceptance_criteria: vec![
                            "docker-compose up starts all services".to_string(),
                            "Includes all required dependencies (database, cache, etc.)".to_string(),
                            "Development and production configurations".to_string(),
                        ],
                        priority: 1,
                    },
                ],
            },
            // Missing integration tests pattern
            OpportunityPattern {
                opportunity_type: OpportunityType::MissingIntegrationTests,
                title: "Add Integration Tests".to_string(),
                rationale: "Unit tests exist but no integration tests to verify component interactions.".to_string(),
                complexity: Complexity::Medium,
                stories: vec![
                    SuggestedStory {
                        title: "Implement integration test suite".to_string(),
                        description: "Add integration tests for critical workflows.".to_string(),
                        acceptance_criteria: vec![
                            "Integration tests for main user flows".to_string(),
                            "Tests run in CI pipeline".to_string(),
                            "Test database/service isolation".to_string(),
                        ],
                        priority: 1,
                    },
                ],
            },
            // Missing e2e tests pattern
            OpportunityPattern {
                opportunity_type: OpportunityType::MissingE2eTests,
                title: "Add End-to-End Tests".to_string(),
                rationale: "Unit/integration tests exist but no e2e tests for full user journey validation.".to_string(),
                complexity: Complexity::High,
                stories: vec![
                    SuggestedStory {
                        title: "Implement e2e test suite".to_string(),
                        description: "Add end-to-end tests for critical user journeys.".to_string(),
                        acceptance_criteria: vec![
                            "E2e tests for main user flows".to_string(),
                            "Tests run in CI pipeline".to_string(),
                            "Visual regression testing considered".to_string(),
                        ],
                        priority: 2,
                    },
                ],
            },
            // Missing linting pattern
            OpportunityPattern {
                opportunity_type: OpportunityType::MissingLinting,
                title: "Add Linting Configuration".to_string(),
                rationale: "Project lacks linting configuration to enforce code style and catch errors.".to_string(),
                complexity: Complexity::Low,
                stories: vec![
                    SuggestedStory {
                        title: "Configure linting tools".to_string(),
                        description: "Set up linting for consistent code style.".to_string(),
                        acceptance_criteria: vec![
                            "Linter configured for the project language".to_string(),
                            "Linting runs in CI pipeline".to_string(),
                            "Documentation for running linting locally".to_string(),
                        ],
                        priority: 1,
                    },
                ],
            },
            // Missing pre-commit hooks pattern
            OpportunityPattern {
                opportunity_type: OpportunityType::MissingPreCommitHooks,
                title: "Add Pre-commit Hooks".to_string(),
                rationale: "Project lacks pre-commit hooks to catch issues before they're committed.".to_string(),
                complexity: Complexity::Low,
                stories: vec![
                    SuggestedStory {
                        title: "Configure pre-commit hooks".to_string(),
                        description: "Set up pre-commit hooks for linting, formatting, and testing.".to_string(),
                        acceptance_criteria: vec![
                            "Pre-commit framework configured".to_string(),
                            "Hooks for linting and formatting".to_string(),
                            "Documentation for hook installation".to_string(),
                        ],
                        priority: 2,
                    },
                ],
            },
        ]
    }

    /// Check if a pattern matches the current context
    fn matches_pattern(&self, pattern: &OpportunityPattern, context: &OpportunityContext) -> bool {
        match pattern.opportunity_type {
            OpportunityType::MissingAuditCommand => {
                // Has CI/CD but no audit command (simplified check)
                context.has_ci_cd
            }
            OpportunityType::MissingHealthCheck => {
                // Has API endpoints but no health check
                context
                    .api
                    .as_ref()
                    .is_some_and(|a| !a.endpoints.is_empty())
                    && !context.has_health_check
            }
            OpportunityType::MissingApiDocs => {
                // Has API endpoints but no OpenAPI docs
                context
                    .api
                    .as_ref()
                    .is_some_and(|a| !a.endpoints.is_empty())
                    && !context.has_openapi_docs
            }
            OpportunityType::MissingCoverageReporting => {
                // Has tests but no coverage configuration
                context.test.as_ref().is_some_and(|t| t.test_file_count > 0)
                    && !context.has_coverage_config
            }
            OpportunityType::MissingBenchmarks => {
                // Has tests but no benchmarks
                context.test.as_ref().is_some_and(|t| t.test_file_count > 0)
                    && !context.has_benchmarks
            }
            OpportunityType::MissingShellCompletions => {
                // Has CLI commands but no shell completions
                context.api.as_ref().is_some_and(|a| !a.commands.is_empty())
                    && !context.has_shell_completions
            }
            OpportunityType::MissingManPages => {
                // Has CLI commands but no man pages (always suggest for CLIs)
                context.api.as_ref().is_some_and(|a| !a.commands.is_empty())
            }
            OpportunityType::MissingConfigValidation => {
                // Has configuration files (simplified, always false for now)
                false
            }
            OpportunityType::MissingStructuredLogging => {
                // Has code but no structured logging
                !context.has_structured_logging
                    && context
                        .api
                        .as_ref()
                        .is_some_and(|a| !a.endpoints.is_empty())
            }
            OpportunityType::MissingErrorCatalog => {
                // Complex check, skip for now
                false
            }
            OpportunityType::MissingServiceDiscovery => {
                // Has multiple services, skip for now
                false
            }
            OpportunityType::MissingMigrations => {
                // Has database-related code but no migrations
                !context.has_migrations
                    && context.api.as_ref().is_some_and(|a| {
                        a.endpoints.iter().any(|e| {
                            e.path.contains("/db")
                                || e.path.contains("/database")
                                || e.path.contains("/users")
                                || e.path.contains("/data")
                        })
                    })
            }
            OpportunityType::MissingDockerCompose => {
                // Has Dockerfile but no docker-compose
                context.has_dockerfile && !context.has_docker_compose
            }
            OpportunityType::MissingIntegrationTests => {
                // Has unit tests but no integration tests
                context.test.as_ref().is_some_and(|t| {
                    t.test_file_count > 0
                        && !t
                            .test_patterns
                            .iter()
                            .any(|p| p.pattern == crate::audit::testing::TestPattern::Integration)
                })
            }
            OpportunityType::MissingE2eTests => {
                // Has tests but no e2e tests
                context.test.as_ref().is_some_and(|t| {
                    t.test_file_count > 0
                        && !t
                            .test_patterns
                            .iter()
                            .any(|p| p.pattern == crate::audit::testing::TestPattern::E2e)
                })
            }
            OpportunityType::MissingRateLimiting => {
                // Has API but no rate limiting
                context
                    .api
                    .as_ref()
                    .is_some_and(|a| !a.endpoints.is_empty())
                    && !context.has_rate_limiting
            }
            OpportunityType::MissingAuthentication => {
                // Has API but no authentication
                context
                    .api
                    .as_ref()
                    .is_some_and(|a| !a.endpoints.is_empty())
                    && !context.has_authentication
            }
            OpportunityType::MissingLinting => {
                // Has code but no linting configuration
                !context.has_linting_config
            }
            OpportunityType::MissingPreCommitHooks => {
                // Has linting but no pre-commit hooks
                context.has_linting_config && !context.has_pre_commit_hooks
            }
        }
    }

    /// Generate observations about detected opportunities
    fn generate_observations(
        &self,
        analysis: &OpportunityAnalysis,
        context: &OpportunityContext,
    ) -> Vec<String> {
        let mut observations = Vec::new();

        if analysis.total_opportunities == 0 {
            observations.push(
                "No significant feature opportunities detected. The project appears well-equipped."
                    .to_string(),
            );
            return observations;
        }

        // Summary observation
        observations.push(format!(
            "Found {} feature opportunity(ies), {} of which are low-complexity quick wins.",
            analysis.total_opportunities, analysis.high_value_count
        ));

        // Category-specific observations
        let has_api_opportunities = analysis.opportunities.iter().any(|o| {
            o.title.contains("Health Check")
                || o.title.contains("OpenAPI")
                || o.title.contains("Rate Limiting")
        });
        if has_api_opportunities {
            observations.push(
                "API-related opportunities detected. Consider improving API documentation and reliability.".to_string(),
            );
        }

        let has_testing_opportunities = analysis.opportunities.iter().any(|o| {
            o.title.contains("Test")
                || o.title.contains("Coverage")
                || o.title.contains("Benchmark")
        });
        if has_testing_opportunities {
            observations.push(
                "Testing-related opportunities detected. Consider expanding test coverage and adding benchmarks.".to_string(),
            );
        }

        let has_devex_opportunities = analysis.opportunities.iter().any(|o| {
            o.title.contains("Lint")
                || o.title.contains("Pre-commit")
                || o.title.contains("Shell Completions")
                || o.title.contains("Docker Compose")
        });
        if has_devex_opportunities {
            observations.push(
                "Developer experience opportunities detected. Consider improving tooling and automation.".to_string(),
            );
        }

        // Context-specific observations
        if context.has_ci_cd && !context.has_coverage_config {
            if let Some(test) = &context.test {
                if test.test_file_count > 0 {
                    observations.push(
                        "CI/CD is configured but test coverage reporting is not. Consider adding coverage tracking.".to_string(),
                    );
                }
            }
        }

        observations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::api::{ApiAnalysis, CliCommand, HttpEndpoint, HttpMethod};
    use crate::audit::testing::{TestAnalysis, TestPattern, TestPatternInfo};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_opportunity_type_display() {
        assert_eq!(
            format!("{}", OpportunityType::MissingAuditCommand),
            "missing_audit_command"
        );
        assert_eq!(
            format!("{}", OpportunityType::MissingHealthCheck),
            "missing_health_check"
        );
        assert_eq!(
            format!("{}", OpportunityType::MissingApiDocs),
            "missing_api_docs"
        );
        assert_eq!(
            format!("{}", OpportunityType::MissingCoverageReporting),
            "missing_coverage_reporting"
        );
        assert_eq!(
            format!("{}", OpportunityType::MissingBenchmarks),
            "missing_benchmarks"
        );
    }

    #[test]
    fn test_opportunity_detector_new() {
        let detector = OpportunityDetector::new(PathBuf::from("/test"));
        assert_eq!(detector.root(), &PathBuf::from("/test"));
    }

    #[test]
    fn test_analyze_empty_context() {
        let temp_dir = TempDir::new().unwrap();
        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());
        let context = OpportunityContext::default();
        let analysis = detector.analyze(&context).unwrap();

        // Should suggest linting since no config exists
        assert!(analysis
            .opportunities
            .iter()
            .any(|o| o.title.contains("Linting")));
    }

    #[test]
    fn test_detect_missing_health_check() {
        let temp_dir = TempDir::new().unwrap();
        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());

        // Create context with API but no health check
        let api = ApiAnalysis {
            endpoints: vec![HttpEndpoint {
                method: HttpMethod::Get,
                path: "/users".to_string(),
                handler: Some("get_users".to_string()),
                file: PathBuf::from("src/main.rs"),
                line: Some(10),
                framework: crate::audit::api::ApiFramework::Axum,
            }],
            ..Default::default()
        };

        let context = OpportunityContext {
            api: Some(api),
            has_health_check: false,
            ..Default::default()
        };

        let analysis = detector.analyze(&context).unwrap();

        assert!(analysis
            .opportunities
            .iter()
            .any(|o| o.title.contains("Health Check")));
    }

    #[test]
    fn test_no_health_check_opportunity_when_exists() {
        let temp_dir = TempDir::new().unwrap();
        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());

        // Create context with API and health check
        let api = ApiAnalysis {
            endpoints: vec![
                HttpEndpoint {
                    method: HttpMethod::Get,
                    path: "/users".to_string(),
                    handler: Some("get_users".to_string()),
                    file: PathBuf::from("src/main.rs"),
                    line: Some(10),
                    framework: crate::audit::api::ApiFramework::Axum,
                },
                HttpEndpoint {
                    method: HttpMethod::Get,
                    path: "/health".to_string(),
                    handler: Some("health_check".to_string()),
                    file: PathBuf::from("src/main.rs"),
                    line: Some(20),
                    framework: crate::audit::api::ApiFramework::Axum,
                },
            ],
            ..Default::default()
        };

        let context = OpportunityContext {
            api: Some(api),
            has_health_check: true,
            ..Default::default()
        };

        let analysis = detector.analyze(&context).unwrap();

        assert!(!analysis
            .opportunities
            .iter()
            .any(|o| o.title.contains("Health Check")));
    }

    #[test]
    fn test_detect_missing_api_docs() {
        let temp_dir = TempDir::new().unwrap();
        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());

        let api = ApiAnalysis {
            endpoints: vec![HttpEndpoint {
                method: HttpMethod::Get,
                path: "/users".to_string(),
                handler: Some("get_users".to_string()),
                file: PathBuf::from("src/main.rs"),
                line: Some(10),
                framework: crate::audit::api::ApiFramework::Axum,
            }],
            ..Default::default()
        };

        let context = OpportunityContext {
            api: Some(api),
            has_openapi_docs: false,
            ..Default::default()
        };

        let analysis = detector.analyze(&context).unwrap();

        assert!(analysis
            .opportunities
            .iter()
            .any(|o| o.title.contains("OpenAPI")));
    }

    #[test]
    fn test_detect_missing_coverage_reporting() {
        let temp_dir = TempDir::new().unwrap();
        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());

        let test = TestAnalysis {
            test_file_count: 5,
            test_function_count: 20,
            ..Default::default()
        };

        let context = OpportunityContext {
            test: Some(test),
            has_coverage_config: false,
            ..Default::default()
        };

        let analysis = detector.analyze(&context).unwrap();

        assert!(analysis
            .opportunities
            .iter()
            .any(|o| o.title.contains("Coverage")));
    }

    #[test]
    fn test_detect_missing_benchmarks() {
        let temp_dir = TempDir::new().unwrap();
        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());

        let test = TestAnalysis {
            test_file_count: 5,
            test_function_count: 20,
            test_patterns: vec![TestPatternInfo {
                pattern: TestPattern::Unit,
                count: 20,
                examples: vec![],
            }],
            ..Default::default()
        };

        let context = OpportunityContext {
            test: Some(test),
            has_benchmarks: false,
            ..Default::default()
        };

        let analysis = detector.analyze(&context).unwrap();

        assert!(analysis
            .opportunities
            .iter()
            .any(|o| o.title.contains("Benchmark")));
    }

    #[test]
    fn test_detect_missing_shell_completions() {
        let temp_dir = TempDir::new().unwrap();
        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());

        let api = ApiAnalysis {
            commands: vec![CliCommand {
                name: "myapp".to_string(),
                description: Some("My app".to_string()),
                subcommands: vec!["init".to_string()],
                file: PathBuf::from("src/main.rs"),
                line: Some(5),
                framework: "clap".to_string(),
            }],
            ..Default::default()
        };

        let context = OpportunityContext {
            api: Some(api),
            has_shell_completions: false,
            ..Default::default()
        };

        let analysis = detector.analyze(&context).unwrap();

        assert!(analysis
            .opportunities
            .iter()
            .any(|o| o.title.contains("Shell Completions")));
    }

    #[test]
    fn test_detect_missing_docker_compose() {
        let temp_dir = TempDir::new().unwrap();
        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());

        let context = OpportunityContext {
            has_dockerfile: true,
            has_docker_compose: false,
            ..Default::default()
        };

        let analysis = detector.analyze(&context).unwrap();

        assert!(analysis
            .opportunities
            .iter()
            .any(|o| o.title.contains("Docker Compose")));
    }

    #[test]
    fn test_detect_missing_integration_tests() {
        let temp_dir = TempDir::new().unwrap();
        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());

        let test = TestAnalysis {
            test_file_count: 5,
            test_function_count: 20,
            test_patterns: vec![TestPatternInfo {
                pattern: TestPattern::Unit,
                count: 20,
                examples: vec![],
            }],
            ..Default::default()
        };

        let context = OpportunityContext {
            test: Some(test),
            ..Default::default()
        };

        let analysis = detector.analyze(&context).unwrap();

        assert!(analysis
            .opportunities
            .iter()
            .any(|o| o.title.contains("Integration Tests")));
    }

    #[test]
    fn test_detect_missing_e2e_tests() {
        let temp_dir = TempDir::new().unwrap();
        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());

        let test = TestAnalysis {
            test_file_count: 5,
            test_function_count: 20,
            test_patterns: vec![
                TestPatternInfo {
                    pattern: TestPattern::Unit,
                    count: 15,
                    examples: vec![],
                },
                TestPatternInfo {
                    pattern: TestPattern::Integration,
                    count: 5,
                    examples: vec![],
                },
            ],
            ..Default::default()
        };

        let context = OpportunityContext {
            test: Some(test),
            ..Default::default()
        };

        let analysis = detector.analyze(&context).unwrap();

        assert!(analysis
            .opportunities
            .iter()
            .any(|o| o.title.contains("End-to-End Tests")));
    }

    #[test]
    fn test_detect_missing_pre_commit_hooks() {
        let temp_dir = TempDir::new().unwrap();
        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());

        let context = OpportunityContext {
            has_linting_config: true,
            has_pre_commit_hooks: false,
            ..Default::default()
        };

        let analysis = detector.analyze(&context).unwrap();

        assert!(analysis
            .opportunities
            .iter()
            .any(|o| o.title.contains("Pre-commit Hooks")));
    }

    #[test]
    fn test_build_context_detects_ci_cd() {
        let temp_dir = TempDir::new().unwrap();

        // Create GitHub Actions directory
        let workflows = temp_dir.path().join(".github/workflows");
        fs::create_dir_all(&workflows).unwrap();
        fs::write(workflows.join("ci.yml"), "name: CI").unwrap();

        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());
        let context = detector.build_context(None, None, None, None).unwrap();

        assert!(context.has_ci_cd);
    }

    #[test]
    fn test_build_context_detects_dockerfile() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("Dockerfile"), "FROM rust:1.70").unwrap();

        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());
        let context = detector.build_context(None, None, None, None).unwrap();

        assert!(context.has_dockerfile);
    }

    #[test]
    fn test_build_context_detects_docker_compose() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("docker-compose.yml"), "version: '3'").unwrap();

        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());
        let context = detector.build_context(None, None, None, None).unwrap();

        assert!(context.has_docker_compose);
    }

    #[test]
    fn test_build_context_detects_linting_config() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("rustfmt.toml"), "edition = \"2021\"").unwrap();

        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());
        let context = detector.build_context(None, None, None, None).unwrap();

        assert!(context.has_linting_config);
    }

    #[test]
    fn test_build_context_detects_pre_commit() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join(".pre-commit-config.yaml"), "repos: []").unwrap();

        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());
        let context = detector.build_context(None, None, None, None).unwrap();

        assert!(context.has_pre_commit_hooks);
    }

    #[test]
    fn test_build_context_detects_openapi() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("openapi.yaml"), "openapi: 3.0.0").unwrap();

        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());
        let context = detector.build_context(None, None, None, None).unwrap();

        assert!(context.has_openapi_docs);
    }

    #[test]
    fn test_build_context_detects_health_check_endpoint() {
        let temp_dir = TempDir::new().unwrap();

        let api = ApiAnalysis {
            endpoints: vec![HttpEndpoint {
                method: HttpMethod::Get,
                path: "/health".to_string(),
                handler: Some("health".to_string()),
                file: PathBuf::from("src/main.rs"),
                line: Some(10),
                framework: crate::audit::api::ApiFramework::Axum,
            }],
            ..Default::default()
        };

        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());
        let context = detector
            .build_context(Some(&api), None, None, None)
            .unwrap();

        assert!(context.has_health_check);
    }

    #[test]
    fn test_build_context_detects_migrations() {
        let temp_dir = TempDir::new().unwrap();

        let migrations = temp_dir.path().join("migrations");
        fs::create_dir(&migrations).unwrap();

        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());
        let context = detector.build_context(None, None, None, None).unwrap();

        assert!(context.has_migrations);
    }

    #[test]
    fn test_build_context_detects_coverage_config() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("codecov.yml"), "coverage: {}").unwrap();

        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());
        let context = detector.build_context(None, None, None, None).unwrap();

        assert!(context.has_coverage_config);
    }

    #[test]
    fn test_build_context_detects_benchmarks_from_directory() {
        let temp_dir = TempDir::new().unwrap();

        let benches = temp_dir.path().join("benches");
        fs::create_dir(&benches).unwrap();

        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());
        let context = detector.build_context(None, None, None, None).unwrap();

        assert!(context.has_benchmarks);
    }

    #[test]
    fn test_build_context_detects_shell_completions() {
        let temp_dir = TempDir::new().unwrap();

        let completions = temp_dir.path().join("completions");
        fs::create_dir(&completions).unwrap();

        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());
        let context = detector.build_context(None, None, None, None).unwrap();

        assert!(context.has_shell_completions);
    }

    #[test]
    fn test_high_value_count() {
        let temp_dir = TempDir::new().unwrap();
        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());

        let context = OpportunityContext {
            has_dockerfile: true,
            has_docker_compose: false,
            has_linting_config: false,
            ..Default::default()
        };

        let analysis = detector.analyze(&context).unwrap();

        // Docker Compose and Linting are both low complexity
        assert!(analysis.high_value_count >= 2);
    }

    #[test]
    fn test_opportunity_analysis_serialization() {
        let analysis = OpportunityAnalysis {
            opportunities: vec![FeatureOpportunity {
                id: "FEAT-001".to_string(),
                title: "Add Health Check".to_string(),
                rationale: "API lacks health check".to_string(),
                complexity: Complexity::Low,
                suggested_stories: vec![SuggestedStory {
                    title: "Implement health check".to_string(),
                    description: "Add /health endpoint".to_string(),
                    acceptance_criteria: vec!["Returns 200".to_string()],
                    priority: 1,
                }],
            }],
            total_opportunities: 1,
            high_value_count: 1,
            observations: vec!["Found 1 opportunity".to_string()],
        };

        let json = serde_json::to_string(&analysis).unwrap();
        let deserialized: OpportunityAnalysis = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_opportunities, 1);
        assert_eq!(deserialized.high_value_count, 1);
        assert_eq!(deserialized.opportunities.len(), 1);
        assert_eq!(deserialized.opportunities[0].id, "FEAT-001");
    }

    #[test]
    fn test_observations_no_opportunities() {
        let temp_dir = TempDir::new().unwrap();

        // Create a well-configured project
        fs::write(temp_dir.path().join("rustfmt.toml"), "").unwrap();
        fs::write(temp_dir.path().join(".pre-commit-config.yaml"), "").unwrap();
        fs::write(temp_dir.path().join("openapi.yaml"), "").unwrap();
        fs::write(temp_dir.path().join("codecov.yml"), "").unwrap();
        fs::create_dir(temp_dir.path().join("benches")).unwrap();
        fs::create_dir(temp_dir.path().join("completions")).unwrap();

        let api = ApiAnalysis {
            endpoints: vec![HttpEndpoint {
                method: HttpMethod::Get,
                path: "/health".to_string(),
                handler: Some("health".to_string()),
                file: PathBuf::from("src/main.rs"),
                line: Some(10),
                framework: crate::audit::api::ApiFramework::Axum,
            }],
            commands: vec![CliCommand {
                name: "myapp".to_string(),
                description: None,
                subcommands: vec![],
                file: PathBuf::from("src/main.rs"),
                line: Some(1),
                framework: "clap".to_string(),
            }],
            ..Default::default()
        };

        let test = TestAnalysis {
            test_file_count: 5,
            test_patterns: vec![
                TestPatternInfo {
                    pattern: TestPattern::Unit,
                    count: 10,
                    examples: vec![],
                },
                TestPatternInfo {
                    pattern: TestPattern::Integration,
                    count: 5,
                    examples: vec![],
                },
                TestPatternInfo {
                    pattern: TestPattern::E2e,
                    count: 2,
                    examples: vec![],
                },
                TestPatternInfo {
                    pattern: TestPattern::Benchmark,
                    count: 3,
                    examples: vec![],
                },
            ],
            ..Default::default()
        };

        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());
        let context = detector
            .build_context(Some(&api), Some(&test), None, None)
            .unwrap();
        let analysis = detector.analyze(&context).unwrap();

        // Most patterns should not trigger due to well-configured project
        // The analysis should have few or no opportunities related to the configured items
        assert!(analysis
            .observations
            .iter()
            .any(|o| o.contains("opportunity") || o.contains("well-equipped")));
    }

    #[test]
    fn test_feature_opportunity_ids_are_unique() {
        let temp_dir = TempDir::new().unwrap();
        let detector = OpportunityDetector::new(temp_dir.path().to_path_buf());

        // Create context that triggers multiple opportunities
        let api = ApiAnalysis {
            endpoints: vec![HttpEndpoint {
                method: HttpMethod::Get,
                path: "/users".to_string(),
                handler: Some("get_users".to_string()),
                file: PathBuf::from("src/main.rs"),
                line: Some(10),
                framework: crate::audit::api::ApiFramework::Axum,
            }],
            commands: vec![CliCommand {
                name: "myapp".to_string(),
                description: None,
                subcommands: vec![],
                file: PathBuf::from("src/main.rs"),
                line: Some(1),
                framework: "clap".to_string(),
            }],
            ..Default::default()
        };

        let test = TestAnalysis {
            test_file_count: 5,
            test_function_count: 20,
            test_patterns: vec![TestPatternInfo {
                pattern: TestPattern::Unit,
                count: 20,
                examples: vec![],
            }],
            ..Default::default()
        };

        let context = OpportunityContext {
            api: Some(api),
            test: Some(test),
            has_dockerfile: true,
            has_docker_compose: false,
            ..Default::default()
        };

        let analysis = detector.analyze(&context).unwrap();

        // Check all IDs are unique
        let ids: Vec<_> = analysis.opportunities.iter().map(|o| &o.id).collect();
        let unique_ids: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique_ids.len());
    }
}
