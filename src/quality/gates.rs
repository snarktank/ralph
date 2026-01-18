//! Quality gate checking functionality for Ralph.
//!
//! This module provides the infrastructure for running quality gates
//! against a codebase, including coverage, linting, formatting, and security checks.

// Allow dead_code for now - these types will be used in future stories
#![allow(dead_code)]

use crate::quality::Profile;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

/// Progress state for a quality gate.
///
/// Used in progress callbacks to report gate status changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateProgressState {
    /// Gate is currently running
    Running,
    /// Gate completed successfully
    Passed,
    /// Gate failed
    Failed,
}

/// Progress update for a quality gate.
///
/// Contains information about a gate's current state and duration.
#[derive(Debug, Clone)]
pub struct GateProgressUpdate {
    /// Name of the quality gate
    pub gate_name: String,
    /// Current progress state
    pub state: GateProgressState,
    /// Duration of the gate execution (only set for Passed/Failed states)
    pub duration: Option<Duration>,
}

impl GateProgressUpdate {
    /// Create a new Running progress update.
    pub fn running(gate_name: impl Into<String>) -> Self {
        Self {
            gate_name: gate_name.into(),
            state: GateProgressState::Running,
            duration: None,
        }
    }

    /// Create a new Passed progress update with duration.
    pub fn passed(gate_name: impl Into<String>, duration: Duration) -> Self {
        Self {
            gate_name: gate_name.into(),
            state: GateProgressState::Passed,
            duration: Some(duration),
        }
    }

    /// Create a new Failed progress update with duration.
    pub fn failed(gate_name: impl Into<String>, duration: Duration) -> Self {
        Self {
            gate_name: gate_name.into(),
            state: GateProgressState::Failed,
            duration: Some(duration),
        }
    }

    /// Check if this is a Running state.
    pub fn is_running(&self) -> bool {
        self.state == GateProgressState::Running
    }

    /// Check if this is a Passed state.
    pub fn is_passed(&self) -> bool {
        self.state == GateProgressState::Passed
    }

    /// Check if this is a Failed state.
    pub fn is_failed(&self) -> bool {
        self.state == GateProgressState::Failed
    }

    /// Check if the gate has completed (Passed or Failed).
    pub fn is_completed(&self) -> bool {
        matches!(
            self.state,
            GateProgressState::Passed | GateProgressState::Failed
        )
    }

    /// Format the duration for display, if available.
    pub fn format_duration(&self) -> Option<String> {
        self.duration.map(|d| {
            if d.as_secs() >= 60 {
                format!(
                    "{}m{:.1}s",
                    d.as_secs() / 60,
                    (d.as_millis() % 60000) as f64 / 1000.0
                )
            } else {
                format!("{:.1}s", d.as_secs_f64())
            }
        })
    }
}

/// The result of running a single quality gate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    /// Name of the quality gate that was run
    pub gate_name: String,
    /// Whether the gate passed
    pub passed: bool,
    /// Human-readable message describing the result
    pub message: String,
    /// Additional details about the gate result (e.g., specific errors, metrics)
    pub details: Option<String>,
}

impl GateResult {
    /// Create a new passing gate result.
    pub fn pass(gate_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            gate_name: gate_name.into(),
            passed: true,
            message: message.into(),
            details: None,
        }
    }

    /// Create a new failing gate result.
    pub fn fail(
        gate_name: impl Into<String>,
        message: impl Into<String>,
        details: Option<String>,
    ) -> Self {
        Self {
            gate_name: gate_name.into(),
            passed: false,
            message: message.into(),
            details,
        }
    }

    /// Create a new skipped gate result.
    pub fn skipped(gate_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            gate_name: gate_name.into(),
            passed: true, // Skipped gates count as passed
            message: format!("Skipped: {}", reason.into()),
            details: None,
        }
    }
}

/// A checker that runs quality gates based on a profile configuration.
pub struct QualityGateChecker {
    /// The quality profile to check against
    profile: Profile,
    /// The root directory of the project to check
    project_root: PathBuf,
}

impl QualityGateChecker {
    /// Create a new quality gate checker.
    ///
    /// # Arguments
    ///
    /// * `profile` - The quality profile containing gate configurations
    /// * `project_root` - The root directory of the project to check
    pub fn new(profile: Profile, project_root: impl Into<PathBuf>) -> Self {
        Self {
            profile,
            project_root: project_root.into(),
        }
    }

    /// Get the profile being used for quality checks.
    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    /// Get the project root directory.
    pub fn project_root(&self) -> &PathBuf {
        &self.project_root
    }

    /// Check code coverage against the profile threshold.
    ///
    /// This method runs either `cargo llvm-cov` or `cargo tarpaulin` to measure
    /// code coverage and compares it against the threshold configured in the profile.
    ///
    /// # Returns
    ///
    /// A `GateResult` indicating whether the coverage threshold was met.
    /// If coverage tools are not installed, returns a failure with installation instructions.
    pub fn check_coverage(&self) -> GateResult {
        let threshold = self.profile.testing.coverage_threshold;

        // If threshold is 0, skip coverage check
        if threshold == 0 {
            return GateResult::skipped("coverage", "Coverage threshold is 0 - no check required");
        }

        // Try cargo-llvm-cov first (more common in CI environments)
        let llvm_cov_result = self.run_llvm_cov();
        if let Some(result) = llvm_cov_result {
            return result;
        }

        // Fall back to cargo-tarpaulin
        let tarpaulin_result = self.run_tarpaulin();
        if let Some(result) = tarpaulin_result {
            return result;
        }

        // Neither tool is available
        GateResult::fail(
            "coverage",
            "No coverage tool available",
            Some(
                "Install cargo-llvm-cov: cargo install cargo-llvm-cov\n\
                 Or install cargo-tarpaulin: cargo install cargo-tarpaulin"
                    .to_string(),
            ),
        )
    }

    /// Run cargo-llvm-cov and parse the coverage percentage.
    fn run_llvm_cov(&self) -> Option<GateResult> {
        // Check if cargo-llvm-cov is installed
        let check_installed = Command::new("cargo")
            .args(["llvm-cov", "--version"])
            .current_dir(&self.project_root)
            .output();

        if check_installed.is_err() || !check_installed.unwrap().status.success() {
            return None; // Tool not installed
        }

        // Run cargo llvm-cov with JSON output for parsing
        let output = Command::new("cargo")
            .args(["llvm-cov", "--json", "--quiet"])
            .current_dir(&self.project_root)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Some(GateResult::fail(
                        "coverage",
                        "cargo llvm-cov failed",
                        Some(format!("stderr: {}", stderr)),
                    ));
                }

                // Parse the JSON output for coverage percentage
                if let Some(coverage) = Self::parse_llvm_cov_json(&stdout) {
                    Some(self.evaluate_coverage(coverage, "cargo-llvm-cov"))
                } else {
                    // If JSON parsing fails, try running with summary output
                    self.run_llvm_cov_summary()
                }
            }
            Err(e) => Some(GateResult::fail(
                "coverage",
                "Failed to run cargo llvm-cov",
                Some(e.to_string()),
            )),
        }
    }

    /// Run cargo-llvm-cov with summary output and parse the percentage.
    fn run_llvm_cov_summary(&self) -> Option<GateResult> {
        let output = Command::new("cargo")
            .args(["llvm-cov", "--quiet"])
            .current_dir(&self.project_root)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !output.status.success() {
                    return Some(GateResult::fail(
                        "coverage",
                        "cargo llvm-cov failed",
                        Some(format!("stderr: {}", stderr)),
                    ));
                }

                // Parse the summary output for coverage percentage
                // llvm-cov outputs lines like "TOTAL ... 75.00%"
                if let Some(coverage) = Self::parse_coverage_percentage(&stdout) {
                    Some(self.evaluate_coverage(coverage, "cargo-llvm-cov"))
                } else {
                    Some(GateResult::fail(
                        "coverage",
                        "Failed to parse llvm-cov output",
                        Some(format!("Output: {}", stdout)),
                    ))
                }
            }
            Err(e) => Some(GateResult::fail(
                "coverage",
                "Failed to run cargo llvm-cov",
                Some(e.to_string()),
            )),
        }
    }

    /// Parse llvm-cov JSON output for total coverage percentage.
    fn parse_llvm_cov_json(json_str: &str) -> Option<f64> {
        // llvm-cov JSON has a "data" array with coverage info
        // We need to extract the total line coverage percentage
        // Format: { "data": [{ "totals": { "lines": { "percent": 75.5 } } }] }
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
            json.get("data")
                .and_then(|d| d.get(0))
                .and_then(|d| d.get("totals"))
                .and_then(|t| t.get("lines"))
                .and_then(|l| l.get("percent"))
                .and_then(|p| p.as_f64())
        } else {
            None
        }
    }

    /// Run cargo-tarpaulin and parse the coverage percentage.
    fn run_tarpaulin(&self) -> Option<GateResult> {
        // Check if cargo-tarpaulin is installed
        let check_installed = Command::new("cargo")
            .args(["tarpaulin", "--version"])
            .current_dir(&self.project_root)
            .output();

        if check_installed.is_err() || !check_installed.unwrap().status.success() {
            return None; // Tool not installed
        }

        // Run cargo tarpaulin
        let output = Command::new("cargo")
            .args(["tarpaulin", "--skip-clean", "--out", "Stdout"])
            .current_dir(&self.project_root)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                // tarpaulin returns exit code 0 even on low coverage
                // Parse the output for coverage percentage
                // Format: "XX.XX% coverage"
                if let Some(coverage) = Self::parse_coverage_percentage(&stdout) {
                    Some(self.evaluate_coverage(coverage, "cargo-tarpaulin"))
                } else if let Some(coverage) = Self::parse_coverage_percentage(&stderr) {
                    // Sometimes tarpaulin outputs to stderr
                    Some(self.evaluate_coverage(coverage, "cargo-tarpaulin"))
                } else {
                    Some(GateResult::fail(
                        "coverage",
                        "Failed to parse tarpaulin output",
                        Some(format!("stdout: {}\nstderr: {}", stdout, stderr)),
                    ))
                }
            }
            Err(e) => Some(GateResult::fail(
                "coverage",
                "Failed to run cargo tarpaulin",
                Some(e.to_string()),
            )),
        }
    }

    /// Parse coverage percentage from text output.
    /// Looks for patterns like "75.00%" or "75.00% coverage" or "TOTAL ... 75.00%"
    fn parse_coverage_percentage(output: &str) -> Option<f64> {
        // Look for percentage patterns
        let re_patterns = [
            // Match "XX.XX% coverage" (tarpaulin format)
            r"(\d+(?:\.\d+)?)\s*%\s*coverage",
            // Match "TOTAL ... XX.XX%" (llvm-cov format)
            r"TOTAL\s+.*?(\d+(?:\.\d+)?)\s*%",
            // Match standalone percentage at end of line
            r"(\d+(?:\.\d+)?)\s*%\s*$",
        ];

        for pattern in &re_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(captures) = re.captures(output) {
                    if let Some(match_) = captures.get(1) {
                        if let Ok(coverage) = match_.as_str().parse::<f64>() {
                            return Some(coverage);
                        }
                    }
                }
            }
        }

        None
    }

    /// Evaluate coverage against the threshold and return a GateResult.
    fn evaluate_coverage(&self, coverage: f64, tool_name: &str) -> GateResult {
        let threshold = self.profile.testing.coverage_threshold as f64;
        let coverage_str = format!("{:.2}%", coverage);

        if coverage >= threshold {
            GateResult::pass(
                "coverage",
                format!(
                    "Coverage {coverage_str} meets threshold of {threshold:.0}% (via {tool_name})"
                ),
            )
        } else {
            GateResult::fail(
                "coverage",
                format!("Coverage {coverage_str} is below threshold of {threshold:.0}%"),
                Some(format!(
                    "Measured with {tool_name}. Increase test coverage to meet the threshold."
                )),
            )
        }
    }

    /// Check code linting using cargo clippy.
    ///
    /// Runs `cargo clippy -- -D warnings` which treats all warnings as errors.
    ///
    /// # Returns
    ///
    /// A `GateResult` indicating whether clippy passed without warnings.
    pub fn check_lint(&self) -> GateResult {
        if !self.profile.ci.lint_check {
            return GateResult::skipped("lint", "Lint checking not enabled in profile");
        }

        let output = Command::new("cargo")
            .args(["clippy", "--", "-D", "warnings"])
            .current_dir(&self.project_root)
            .output();

        match output {
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);

                if output.status.success() {
                    GateResult::pass("lint", "No clippy warnings found")
                } else {
                    // Extract the error details from stderr
                    let details = Self::extract_clippy_errors(&stderr);
                    GateResult::fail("lint", "Clippy found warnings or errors", Some(details))
                }
            }
            Err(e) => GateResult::fail(
                "lint",
                "Failed to run cargo clippy",
                Some(format!("Error: {}. Is clippy installed?", e)),
            ),
        }
    }

    /// Extract relevant error messages from clippy stderr output.
    fn extract_clippy_errors(stderr: &str) -> String {
        // Clippy outputs errors and warnings to stderr
        // Filter to show the most relevant lines (errors, warnings, and their context)
        let relevant_lines: Vec<&str> = stderr
            .lines()
            .filter(|line| {
                line.contains("error")
                    || line.contains("warning")
                    || line.starts_with("  -->")
                    || line.starts_with("   |")
            })
            .take(50) // Limit to first 50 lines to avoid huge output
            .collect();

        if relevant_lines.is_empty() {
            stderr.to_string()
        } else {
            relevant_lines.join("\n")
        }
    }

    /// Check code formatting using cargo fmt.
    ///
    /// Runs `cargo fmt --check` which returns non-zero if formatting changes are needed.
    ///
    /// # Returns
    ///
    /// A `GateResult` indicating whether code is properly formatted.
    pub fn check_format(&self) -> GateResult {
        if !self.profile.ci.format_check {
            return GateResult::skipped("format", "Format checking not enabled in profile");
        }

        let output = Command::new("cargo")
            .args(["fmt", "--check"])
            .current_dir(&self.project_root)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if output.status.success() {
                    GateResult::pass("format", "All files are properly formatted")
                } else {
                    // cargo fmt --check outputs files that need formatting to stdout
                    let details = Self::extract_format_errors(&stdout, &stderr);
                    GateResult::fail("format", "Some files need formatting", Some(details))
                }
            }
            Err(e) => GateResult::fail(
                "format",
                "Failed to run cargo fmt",
                Some(format!("Error: {}. Is rustfmt installed?", e)),
            ),
        }
    }

    /// Extract relevant information from cargo fmt output.
    fn extract_format_errors(stdout: &str, stderr: &str) -> String {
        // cargo fmt --check outputs "Diff in <file>" for each unformatted file
        let mut result = String::new();

        // Count files needing formatting
        let unformatted_files: Vec<&str> = stdout
            .lines()
            .filter(|line| line.starts_with("Diff in"))
            .collect();

        if !unformatted_files.is_empty() {
            result.push_str(&format!(
                "{} file(s) need formatting:\n",
                unformatted_files.len()
            ));
            for file in unformatted_files.iter().take(20) {
                result.push_str(file);
                result.push('\n');
            }
            if unformatted_files.len() > 20 {
                result.push_str(&format!(
                    "... and {} more files\n",
                    unformatted_files.len() - 20
                ));
            }
            result.push_str("\nRun `cargo fmt` to fix formatting issues.");
        } else if !stderr.is_empty() {
            result = stderr.to_string();
        } else if !stdout.is_empty() {
            result = stdout.to_string();
        } else {
            result = "Formatting check failed (no additional details)".to_string();
        }

        result
    }

    /// Check for security vulnerabilities using cargo-audit.
    ///
    /// Runs `cargo audit` to check for known security vulnerabilities in dependencies.
    ///
    /// # Returns
    ///
    /// A `GateResult` indicating whether any vulnerabilities were found.
    /// If cargo-audit is not installed, returns a failure with installation instructions.
    pub fn check_security_audit(&self) -> GateResult {
        if !self.profile.security.cargo_audit {
            return GateResult::skipped("security_audit", "Security audit not enabled in profile");
        }

        // Check if cargo-audit is installed
        let check_installed = Command::new("cargo")
            .args(["audit", "--version"])
            .current_dir(&self.project_root)
            .output();

        match check_installed {
            Ok(output) if output.status.success() => {
                // cargo-audit is installed, run the audit
                self.run_cargo_audit()
            }
            _ => {
                // cargo-audit is not installed
                GateResult::fail(
                    "security_audit",
                    "cargo-audit is not installed",
                    Some(
                        "Install cargo-audit: cargo install cargo-audit\n\
                         cargo-audit checks for known security vulnerabilities in dependencies."
                            .to_string(),
                    ),
                )
            }
        }
    }

    /// Run cargo audit and parse the results.
    fn run_cargo_audit(&self) -> GateResult {
        // Run cargo audit with JSON output for easier parsing
        let output = Command::new("cargo")
            .args(["audit", "--json"])
            .current_dir(&self.project_root)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                // Try to parse JSON output first
                if let Some(result) = self.parse_audit_json(&stdout) {
                    return result;
                }

                // If JSON parsing fails, fall back to exit code and text parsing
                if output.status.success() {
                    GateResult::pass("security_audit", "No known vulnerabilities found")
                } else {
                    // Non-zero exit means vulnerabilities found or error
                    let details = Self::extract_audit_vulnerabilities(&stdout, &stderr);
                    GateResult::fail(
                        "security_audit",
                        "Security vulnerabilities found",
                        Some(details),
                    )
                }
            }
            Err(e) => GateResult::fail(
                "security_audit",
                "Failed to run cargo audit",
                Some(format!("Error: {}", e)),
            ),
        }
    }

    /// Parse cargo audit JSON output.
    fn parse_audit_json(&self, json_str: &str) -> Option<GateResult> {
        // cargo audit --json outputs a JSON object with vulnerabilities
        // Format: { "vulnerabilities": { "count": N, "list": [...] }, ... }
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
            let vuln_count = json
                .get("vulnerabilities")
                .and_then(|v| v.get("count"))
                .and_then(|c| c.as_u64())
                .unwrap_or(0);

            if vuln_count == 0 {
                return Some(GateResult::pass(
                    "security_audit",
                    "No known vulnerabilities found",
                ));
            }

            // Extract vulnerability details
            let details = self.format_vulnerabilities_from_json(&json, vuln_count);
            return Some(GateResult::fail(
                "security_audit",
                format!(
                    "Found {} known vulnerabilit{}",
                    vuln_count,
                    if vuln_count == 1 { "y" } else { "ies" }
                ),
                Some(details),
            ));
        }

        None
    }

    /// Format vulnerability details from JSON output.
    fn format_vulnerabilities_from_json(&self, json: &serde_json::Value, count: u64) -> String {
        let mut details = format!(
            "{} vulnerabilit{} found:\n\n",
            count,
            if count == 1 { "y" } else { "ies" }
        );

        if let Some(list) = json
            .get("vulnerabilities")
            .and_then(|v| v.get("list"))
            .and_then(|l| l.as_array())
        {
            for (i, vuln) in list.iter().take(10).enumerate() {
                let advisory = vuln.get("advisory");
                let id = advisory
                    .and_then(|a| a.get("id"))
                    .and_then(|i| i.as_str())
                    .unwrap_or("Unknown");
                let title = advisory
                    .and_then(|a| a.get("title"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("Unknown");
                let severity = advisory
                    .and_then(|a| a.get("severity"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");
                let package_name = vuln
                    .get("package")
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown");
                let package_version = vuln
                    .get("package")
                    .and_then(|p| p.get("version"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                details.push_str(&format!(
                    "{}. {} ({})\n   Package: {} v{}\n   Severity: {}\n\n",
                    i + 1,
                    id,
                    title,
                    package_name,
                    package_version,
                    severity
                ));
            }

            if list.len() > 10 {
                details.push_str(&format!("... and {} more\n", list.len() - 10));
            }
        }

        details.push_str("\nRun `cargo audit` for full details.");
        details
    }

    /// Extract vulnerability information from text output.
    fn extract_audit_vulnerabilities(stdout: &str, stderr: &str) -> String {
        let mut result = String::new();

        // Look for vulnerability indicators in the output
        let combined = format!("{}\n{}", stdout, stderr);

        // Count warnings/errors
        let warning_count = combined.matches("warning:").count();
        let error_count = combined.matches("error:").count();

        if warning_count > 0 || error_count > 0 {
            result.push_str(&format!(
                "Found {} warning(s) and {} error(s)\n\n",
                warning_count, error_count
            ));
        }

        // Extract lines containing vulnerability IDs (RUSTSEC-YYYY-NNNN)
        let vuln_lines: Vec<&str> = combined
            .lines()
            .filter(|line| {
                line.contains("RUSTSEC-")
                    || line.contains("Crate:")
                    || line.contains("Version:")
                    || line.contains("Title:")
                    || line.contains("warning:")
                    || line.contains("error:")
            })
            .take(50)
            .collect();

        if !vuln_lines.is_empty() {
            result.push_str(&vuln_lines.join("\n"));
        } else if !combined.trim().is_empty() {
            // If no structured output found, include the raw output (limited)
            let truncated: String = combined.chars().take(2000).collect();
            result.push_str(&truncated);
            if combined.len() > 2000 {
                result.push_str("\n... (output truncated)");
            }
        } else {
            result.push_str("Security audit failed (no additional details available)");
        }

        result.push_str("\n\nRun `cargo audit` for full details.");
        result
    }

    /// Run all quality gates configured in the profile.
    ///
    /// Returns a vector of `GateResult` for each gate that was run.
    /// Gates that are not enabled in the profile will be skipped.
    ///
    /// # Returns
    ///
    /// A `Vec<GateResult>` containing the results of all gates.
    pub fn run_all(&self) -> Vec<GateResult> {
        vec![
            self.check_coverage(),
            self.check_lint(),
            self.check_format(),
            self.check_security_audit(),
        ]
    }

    /// Run all quality gates with progress callbacks.
    ///
    /// This method runs all configured quality gates and calls the progress
    /// callback before and after each gate execution:
    ///
    /// - Emits `Running` state before each gate starts
    /// - Emits `Passed` or `Failed` state after each gate completes, with duration
    ///
    /// # Arguments
    ///
    /// * `callback` - A mutable callback that receives progress updates.
    ///   The callback signature is `FnMut(&str, GateProgressState)` for simple
    ///   status tracking, but it also receives duration information via
    ///   `GateProgressUpdate`.
    ///
    /// # Returns
    ///
    /// A `Vec<GateResult>` containing the results of all gates.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let checker = QualityGateChecker::new(profile, project_root);
    /// let results = checker.run_all_gates_with_progress(|update| {
    ///     match update.state {
    ///         GateProgressState::Running => println!("Starting: {}", update.gate_name),
    ///         GateProgressState::Passed => println!("Passed: {} ({:?})", update.gate_name, update.duration),
    ///         GateProgressState::Failed => println!("Failed: {} ({:?})", update.gate_name, update.duration),
    ///     }
    /// });
    /// ```
    pub fn run_all_gates_with_progress<F>(&self, mut callback: F) -> Vec<GateResult>
    where
        F: FnMut(GateProgressUpdate),
    {
        let mut results = Vec::new();

        // Run coverage check
        callback(GateProgressUpdate::running("coverage"));
        let start = Instant::now();
        let result = self.check_coverage();
        let duration = start.elapsed();
        if result.passed {
            callback(GateProgressUpdate::passed("coverage", duration));
        } else {
            callback(GateProgressUpdate::failed("coverage", duration));
        }
        results.push(result);

        // Run lint check
        callback(GateProgressUpdate::running("lint"));
        let start = Instant::now();
        let result = self.check_lint();
        let duration = start.elapsed();
        if result.passed {
            callback(GateProgressUpdate::passed("lint", duration));
        } else {
            callback(GateProgressUpdate::failed("lint", duration));
        }
        results.push(result);

        // Run format check
        callback(GateProgressUpdate::running("format"));
        let start = Instant::now();
        let result = self.check_format();
        let duration = start.elapsed();
        if result.passed {
            callback(GateProgressUpdate::passed("format", duration));
        } else {
            callback(GateProgressUpdate::failed("format", duration));
        }
        results.push(result);

        // Run security audit
        callback(GateProgressUpdate::running("security_audit"));
        let start = Instant::now();
        let result = self.check_security_audit();
        let duration = start.elapsed();
        if result.passed {
            callback(GateProgressUpdate::passed("security_audit", duration));
        } else {
            callback(GateProgressUpdate::failed("security_audit", duration));
        }
        results.push(result);

        results
    }

    /// Check if all gates passed.
    pub fn all_passed(results: &[GateResult]) -> bool {
        results.iter().all(|r| r.passed)
    }

    /// Get a summary of gate results.
    pub fn summary(results: &[GateResult]) -> String {
        let passed = results.iter().filter(|r| r.passed).count();
        let total = results.len();
        let failed: Vec<&str> = results
            .iter()
            .filter(|r| !r.passed)
            .map(|r| r.gate_name.as_str())
            .collect();

        if failed.is_empty() {
            format!("All {total} gates passed")
        } else {
            format!(
                "{passed}/{total} gates passed. Failed: {}",
                failed.join(", ")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::{CiConfig, Profile, SecurityConfig, TestingConfig};

    fn create_test_profile(coverage: u8, lint: bool, format: bool, audit: bool) -> Profile {
        Profile {
            description: "Test profile".to_string(),
            testing: TestingConfig {
                coverage_threshold: coverage,
                unit_tests: true,
                integration_tests: false,
            },
            ci: CiConfig {
                required: true,
                lint_check: lint,
                format_check: format,
            },
            security: SecurityConfig {
                cargo_audit: audit,
                cargo_deny: false,
                sast: false,
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_gate_result_pass() {
        let result = GateResult::pass("test_gate", "Test passed");
        assert!(result.passed);
        assert_eq!(result.gate_name, "test_gate");
        assert_eq!(result.message, "Test passed");
        assert!(result.details.is_none());
    }

    #[test]
    fn test_gate_result_fail() {
        let result = GateResult::fail(
            "test_gate",
            "Test failed",
            Some("Error details".to_string()),
        );
        assert!(!result.passed);
        assert_eq!(result.gate_name, "test_gate");
        assert_eq!(result.message, "Test failed");
        assert_eq!(result.details, Some("Error details".to_string()));
    }

    #[test]
    fn test_gate_result_skipped() {
        let result = GateResult::skipped("test_gate", "Not enabled");
        assert!(result.passed); // Skipped counts as passed
        assert_eq!(result.gate_name, "test_gate");
        assert!(result.message.contains("Skipped"));
    }

    #[test]
    fn test_checker_run_all_minimal() {
        let profile = create_test_profile(0, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let results = checker.run_all();

        assert_eq!(results.len(), 4);
        assert!(QualityGateChecker::all_passed(&results));
    }

    #[test]
    fn test_checker_run_all_comprehensive() {
        let profile = create_test_profile(90, true, true, true);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let results = checker.run_all();

        assert_eq!(results.len(), 4);
        // Coverage gate may fail if tools not installed, lint/format/security are still skipped
    }

    #[test]
    fn test_all_passed_true() {
        let results = vec![
            GateResult::pass("gate1", "Passed"),
            GateResult::pass("gate2", "Passed"),
        ];
        assert!(QualityGateChecker::all_passed(&results));
    }

    #[test]
    fn test_all_passed_false() {
        let results = vec![
            GateResult::pass("gate1", "Passed"),
            GateResult::fail("gate2", "Failed", None),
        ];
        assert!(!QualityGateChecker::all_passed(&results));
    }

    #[test]
    fn test_summary_all_passed() {
        let results = vec![
            GateResult::pass("gate1", "Passed"),
            GateResult::pass("gate2", "Passed"),
        ];
        let summary = QualityGateChecker::summary(&results);
        assert_eq!(summary, "All 2 gates passed");
    }

    #[test]
    fn test_summary_some_failed() {
        let results = vec![
            GateResult::pass("gate1", "Passed"),
            GateResult::fail("gate2", "Failed", None),
            GateResult::fail("gate3", "Failed", None),
        ];
        let summary = QualityGateChecker::summary(&results);
        assert!(summary.contains("1/3 gates passed"));
        assert!(summary.contains("gate2"));
        assert!(summary.contains("gate3"));
    }

    // Coverage gate tests

    #[test]
    fn test_check_coverage_zero_threshold_skipped() {
        let profile = create_test_profile(0, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let result = checker.check_coverage();

        assert!(result.passed);
        assert_eq!(result.gate_name, "coverage");
        assert!(result.message.contains("Skipped"));
        assert!(result.message.contains("threshold is 0"));
    }

    #[test]
    fn test_check_coverage_with_threshold() {
        // This test checks that the coverage gate attempts to run when threshold > 0
        let profile = create_test_profile(70, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let result = checker.check_coverage();

        // Should either pass (if tools installed) or fail with "no coverage tool available"
        assert_eq!(result.gate_name, "coverage");
        // The result depends on whether coverage tools are installed
        // If not installed, it should fail with a helpful message
        if !result.passed {
            assert!(
                result.message.contains("No coverage tool")
                    || result.message.contains("failed")
                    || result.message.contains("below threshold"),
                "Unexpected failure message: {}",
                result.message
            );
        }
    }

    #[test]
    fn test_parse_coverage_percentage_tarpaulin_format() {
        // Test tarpaulin-style output
        assert_eq!(
            QualityGateChecker::parse_coverage_percentage("75.00% coverage"),
            Some(75.0)
        );
        assert_eq!(
            QualityGateChecker::parse_coverage_percentage("100% coverage"),
            Some(100.0)
        );
        assert_eq!(
            QualityGateChecker::parse_coverage_percentage("0.5% coverage"),
            Some(0.5)
        );
    }

    #[test]
    fn test_parse_coverage_percentage_llvm_cov_format() {
        // Test llvm-cov-style TOTAL line
        assert_eq!(
            QualityGateChecker::parse_coverage_percentage("TOTAL 100 50 50.00%"),
            Some(50.0)
        );
        assert_eq!(
            QualityGateChecker::parse_coverage_percentage(
                "Filename   Functions  Lines\nTOTAL      10         75.50%"
            ),
            Some(75.5)
        );
    }

    #[test]
    fn test_parse_coverage_percentage_invalid() {
        assert_eq!(
            QualityGateChecker::parse_coverage_percentage("no match here"),
            None
        );
        assert_eq!(QualityGateChecker::parse_coverage_percentage(""), None);
    }

    #[test]
    fn test_parse_llvm_cov_json() {
        let json = r#"{
            "data": [{
                "totals": {
                    "lines": {
                        "percent": 82.5
                    }
                }
            }]
        }"#;
        assert_eq!(QualityGateChecker::parse_llvm_cov_json(json), Some(82.5));
    }

    #[test]
    fn test_parse_llvm_cov_json_invalid() {
        assert_eq!(QualityGateChecker::parse_llvm_cov_json("not json"), None);
        assert_eq!(QualityGateChecker::parse_llvm_cov_json("{}"), None);
        assert_eq!(
            QualityGateChecker::parse_llvm_cov_json(r#"{"data": []}"#),
            None
        );
    }

    #[test]
    fn test_evaluate_coverage_pass() {
        let profile = create_test_profile(70, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let result = checker.evaluate_coverage(80.0, "test-tool");

        assert!(result.passed);
        assert!(result.message.contains("80.00%"));
        assert!(result.message.contains("meets threshold"));
        assert!(result.message.contains("70%"));
    }

    #[test]
    fn test_evaluate_coverage_fail() {
        let profile = create_test_profile(70, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let result = checker.evaluate_coverage(50.0, "test-tool");

        assert!(!result.passed);
        assert!(result.message.contains("50.00%"));
        assert!(result.message.contains("below threshold"));
        assert!(result.details.is_some());
        assert!(result.details.unwrap().contains("test-tool"));
    }

    #[test]
    fn test_evaluate_coverage_exact_threshold() {
        let profile = create_test_profile(70, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let result = checker.evaluate_coverage(70.0, "test-tool");

        assert!(result.passed, "Coverage at exactly threshold should pass");
    }

    // Lint gate tests

    #[test]
    fn test_check_lint_disabled() {
        let profile = create_test_profile(0, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let result = checker.check_lint();

        assert!(result.passed);
        assert_eq!(result.gate_name, "lint");
        assert!(result.message.contains("Skipped"));
        assert!(result.message.contains("not enabled"));
    }

    #[test]
    fn test_check_lint_enabled() {
        // This test runs against a real project directory if available
        let profile = create_test_profile(0, true, false, false);
        // Use the actual Ralph project directory for testing
        let project_root = std::env::current_dir().unwrap_or_else(|_| "/tmp/test".into());
        let checker = QualityGateChecker::new(profile, &project_root);
        let result = checker.check_lint();

        assert_eq!(result.gate_name, "lint");
        // Result depends on whether clippy finds issues
        // If it passes or fails, the message should reflect that
        if result.passed {
            assert!(result.message.contains("No clippy warnings"));
        } else {
            assert!(
                result.message.contains("warnings")
                    || result.message.contains("errors")
                    || result.message.contains("Failed"),
                "Unexpected failure message: {}",
                result.message
            );
        }
    }

    #[test]
    fn test_extract_clippy_errors_with_errors() {
        let stderr = r#"error: unused variable: `x`
  --> src/main.rs:10:5
   |
10 |     let x = 5;
   |         ^ help: if this is intentional, prefix it with an underscore: `_x`
   |
   = note: `#[deny(unused_variables)]` on by default

warning: function `foo` is never used
  --> src/main.rs:5:4
   |
5  | fn foo() {}
   |    ^^^
"#;
        let result = QualityGateChecker::extract_clippy_errors(stderr);

        assert!(result.contains("error"));
        assert!(result.contains("warning"));
        assert!(result.contains("unused variable"));
    }

    #[test]
    fn test_extract_clippy_errors_empty() {
        let stderr = "";
        let result = QualityGateChecker::extract_clippy_errors(stderr);
        assert!(result.is_empty());
    }

    // Format gate tests

    #[test]
    fn test_check_format_disabled() {
        let profile = create_test_profile(0, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let result = checker.check_format();

        assert!(result.passed);
        assert_eq!(result.gate_name, "format");
        assert!(result.message.contains("Skipped"));
        assert!(result.message.contains("not enabled"));
    }

    #[test]
    fn test_check_format_enabled() {
        // This test runs against a real project directory if available
        let profile = create_test_profile(0, false, true, false);
        // Use the actual Ralph project directory for testing
        let project_root = std::env::current_dir().unwrap_or_else(|_| "/tmp/test".into());
        let checker = QualityGateChecker::new(profile, &project_root);
        let result = checker.check_format();

        assert_eq!(result.gate_name, "format");
        // Result depends on whether files need formatting
        if result.passed {
            assert!(result.message.contains("properly formatted"));
        } else {
            assert!(
                result.message.contains("need formatting") || result.message.contains("Failed"),
                "Unexpected failure message: {}",
                result.message
            );
        }
    }

    #[test]
    fn test_extract_format_errors_with_diffs() {
        let stdout = "Diff in /src/main.rs at line 1:\nDiff in /src/lib.rs at line 5:\n";
        let stderr = "";
        let result = QualityGateChecker::extract_format_errors(stdout, stderr);

        assert!(result.contains("2 file(s) need formatting"));
        assert!(result.contains("cargo fmt"));
    }

    #[test]
    fn test_extract_format_errors_empty() {
        let stdout = "";
        let stderr = "";
        let result = QualityGateChecker::extract_format_errors(stdout, stderr);
        assert!(result.contains("Formatting check failed"));
    }

    #[test]
    fn test_extract_format_errors_with_stderr() {
        let stdout = "";
        let stderr = "error: couldn't parse file";
        let result = QualityGateChecker::extract_format_errors(stdout, stderr);
        assert!(result.contains("couldn't parse"));
    }

    // Security audit gate tests

    #[test]
    fn test_check_security_audit_disabled() {
        let profile = create_test_profile(0, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");
        let result = checker.check_security_audit();

        assert!(result.passed);
        assert_eq!(result.gate_name, "security_audit");
        assert!(result.message.contains("Skipped"));
        assert!(result.message.contains("not enabled"));
    }

    #[test]
    fn test_check_security_audit_enabled() {
        // This test runs against a real project directory if available
        let profile = create_test_profile(0, false, false, true);
        // Use the actual Ralph project directory for testing
        let project_root = std::env::current_dir().unwrap_or_else(|_| "/tmp/test".into());
        let checker = QualityGateChecker::new(profile, &project_root);
        let result = checker.check_security_audit();

        assert_eq!(result.gate_name, "security_audit");
        // Result depends on whether cargo-audit is installed and if vulnerabilities exist
        if result.passed {
            assert!(result.message.contains("No known vulnerabilities"));
        } else {
            // Could fail due to: not installed, vulnerabilities found, or command error
            assert!(
                result.message.contains("not installed")
                    || result.message.contains("vulnerabilit")
                    || result.message.contains("Failed"),
                "Unexpected failure message: {}",
                result.message
            );
        }
    }

    #[test]
    fn test_parse_audit_json_no_vulnerabilities() {
        let profile = create_test_profile(0, false, false, true);
        let checker = QualityGateChecker::new(profile, "/tmp/test");

        let json = r#"{
            "database": {},
            "lockfile": {},
            "vulnerabilities": {
                "count": 0,
                "list": []
            }
        }"#;

        let result = checker.parse_audit_json(json);
        assert!(result.is_some());
        let gate_result = result.unwrap();
        assert!(gate_result.passed);
        assert!(gate_result.message.contains("No known vulnerabilities"));
    }

    #[test]
    fn test_parse_audit_json_with_vulnerabilities() {
        let profile = create_test_profile(0, false, false, true);
        let checker = QualityGateChecker::new(profile, "/tmp/test");

        let json = r#"{
            "database": {},
            "lockfile": {},
            "vulnerabilities": {
                "count": 2,
                "list": [
                    {
                        "advisory": {
                            "id": "RUSTSEC-2021-0001",
                            "title": "Test vulnerability 1",
                            "severity": "high"
                        },
                        "package": {
                            "name": "test-crate",
                            "version": "1.0.0"
                        }
                    },
                    {
                        "advisory": {
                            "id": "RUSTSEC-2021-0002",
                            "title": "Test vulnerability 2",
                            "severity": "medium"
                        },
                        "package": {
                            "name": "another-crate",
                            "version": "2.0.0"
                        }
                    }
                ]
            }
        }"#;

        let result = checker.parse_audit_json(json);
        assert!(result.is_some());
        let gate_result = result.unwrap();
        assert!(!gate_result.passed);
        assert!(gate_result.message.contains("2 known vulnerabilities"));
        let details = gate_result.details.unwrap();
        assert!(details.contains("RUSTSEC-2021-0001"));
        assert!(details.contains("RUSTSEC-2021-0002"));
        assert!(details.contains("test-crate"));
        assert!(details.contains("high"));
    }

    #[test]
    fn test_parse_audit_json_single_vulnerability() {
        let profile = create_test_profile(0, false, false, true);
        let checker = QualityGateChecker::new(profile, "/tmp/test");

        let json = r#"{
            "vulnerabilities": {
                "count": 1,
                "list": [
                    {
                        "advisory": {
                            "id": "RUSTSEC-2022-0001",
                            "title": "Single vuln",
                            "severity": "critical"
                        },
                        "package": {
                            "name": "vulnerable-crate",
                            "version": "0.1.0"
                        }
                    }
                ]
            }
        }"#;

        let result = checker.parse_audit_json(json);
        assert!(result.is_some());
        let gate_result = result.unwrap();
        assert!(!gate_result.passed);
        // Check singular form
        assert!(gate_result.message.contains("1 known vulnerability"));
    }

    #[test]
    fn test_parse_audit_json_invalid() {
        let profile = create_test_profile(0, false, false, true);
        let checker = QualityGateChecker::new(profile, "/tmp/test");

        assert!(checker.parse_audit_json("not json").is_none());
        assert!(checker.parse_audit_json("{}").is_some()); // Valid JSON but no vulnerabilities = 0 count = pass
    }

    #[test]
    fn test_extract_audit_vulnerabilities_with_rustsec() {
        let stdout = r#"
Crate:   test-crate
Version: 1.0.0
Title:   Test vulnerability
RUSTSEC-2021-0001
        "#;
        let stderr = "";

        let result = QualityGateChecker::extract_audit_vulnerabilities(stdout, stderr);
        assert!(result.contains("RUSTSEC-2021-0001"));
        assert!(result.contains("Crate:"));
        assert!(result.contains("cargo audit"));
    }

    #[test]
    fn test_extract_audit_vulnerabilities_with_warnings() {
        let stdout = "";
        let stderr = r#"
warning: 1 vulnerability found!
warning: some other warning
error: critical issue
        "#;

        let result = QualityGateChecker::extract_audit_vulnerabilities(stdout, stderr);
        assert!(result.contains("warning(s)"));
        assert!(result.contains("error(s)"));
    }

    #[test]
    fn test_extract_audit_vulnerabilities_empty() {
        let stdout = "";
        let stderr = "";

        let result = QualityGateChecker::extract_audit_vulnerabilities(stdout, stderr);
        assert!(result.contains("no additional details"));
    }

    #[test]
    fn test_format_vulnerabilities_from_json() {
        let profile = create_test_profile(0, false, false, true);
        let checker = QualityGateChecker::new(profile, "/tmp/test");

        let json: serde_json::Value = serde_json::from_str(
            r#"{
            "vulnerabilities": {
                "count": 1,
                "list": [
                    {
                        "advisory": {
                            "id": "RUSTSEC-2023-0001",
                            "title": "Memory safety issue",
                            "severity": "high"
                        },
                        "package": {
                            "name": "unsafe-crate",
                            "version": "3.0.0"
                        }
                    }
                ]
            }
        }"#,
        )
        .unwrap();

        let details = checker.format_vulnerabilities_from_json(&json, 1);
        assert!(details.contains("1 vulnerability found"));
        assert!(details.contains("RUSTSEC-2023-0001"));
        assert!(details.contains("Memory safety issue"));
        assert!(details.contains("unsafe-crate"));
        assert!(details.contains("v3.0.0"));
        assert!(details.contains("high"));
    }

    // ========================================================================
    // GateProgressState Tests
    // ========================================================================

    #[test]
    fn test_gate_progress_state_equality() {
        assert_eq!(GateProgressState::Running, GateProgressState::Running);
        assert_eq!(GateProgressState::Passed, GateProgressState::Passed);
        assert_eq!(GateProgressState::Failed, GateProgressState::Failed);
        assert_ne!(GateProgressState::Running, GateProgressState::Passed);
        assert_ne!(GateProgressState::Passed, GateProgressState::Failed);
    }

    // ========================================================================
    // GateProgressUpdate Tests
    // ========================================================================

    #[test]
    fn test_gate_progress_update_running() {
        let update = GateProgressUpdate::running("lint");
        assert_eq!(update.gate_name, "lint");
        assert_eq!(update.state, GateProgressState::Running);
        assert!(update.duration.is_none());
        assert!(update.is_running());
        assert!(!update.is_passed());
        assert!(!update.is_failed());
        assert!(!update.is_completed());
    }

    #[test]
    fn test_gate_progress_update_passed() {
        let duration = Duration::from_secs_f64(1.5);
        let update = GateProgressUpdate::passed("format", duration);
        assert_eq!(update.gate_name, "format");
        assert_eq!(update.state, GateProgressState::Passed);
        assert_eq!(update.duration, Some(duration));
        assert!(!update.is_running());
        assert!(update.is_passed());
        assert!(!update.is_failed());
        assert!(update.is_completed());
    }

    #[test]
    fn test_gate_progress_update_failed() {
        let duration = Duration::from_secs_f64(2.3);
        let update = GateProgressUpdate::failed("coverage", duration);
        assert_eq!(update.gate_name, "coverage");
        assert_eq!(update.state, GateProgressState::Failed);
        assert_eq!(update.duration, Some(duration));
        assert!(!update.is_running());
        assert!(!update.is_passed());
        assert!(update.is_failed());
        assert!(update.is_completed());
    }

    #[test]
    fn test_gate_progress_update_format_duration_none() {
        let update = GateProgressUpdate::running("test");
        assert!(update.format_duration().is_none());
    }

    #[test]
    fn test_gate_progress_update_format_duration_seconds() {
        let update = GateProgressUpdate::passed("test", Duration::from_secs_f64(1.234));
        let formatted = update.format_duration().unwrap();
        assert!(formatted.contains("1.2"));
        assert!(formatted.ends_with('s'));
    }

    #[test]
    fn test_gate_progress_update_format_duration_minutes() {
        let update = GateProgressUpdate::passed("test", Duration::from_secs(125));
        let formatted = update.format_duration().unwrap();
        assert!(formatted.contains("2m"));
    }

    // ========================================================================
    // run_all_gates_with_progress Tests
    // ========================================================================

    #[test]
    fn test_run_all_gates_with_progress_emits_running_first() {
        let profile = create_test_profile(0, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");

        let mut updates: Vec<GateProgressUpdate> = Vec::new();
        checker.run_all_gates_with_progress(|update| {
            updates.push(update);
        });

        // Should have 8 updates (Running + Passed/Failed for each of 4 gates)
        assert_eq!(updates.len(), 8);

        // First update should be Running for coverage
        assert!(updates[0].is_running());
        assert_eq!(updates[0].gate_name, "coverage");

        // Second update should be completed for coverage
        assert!(updates[1].is_completed());
        assert_eq!(updates[1].gate_name, "coverage");
    }

    #[test]
    fn test_run_all_gates_with_progress_correct_gate_order() {
        let profile = create_test_profile(0, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");

        let mut gate_names: Vec<String> = Vec::new();
        checker.run_all_gates_with_progress(|update| {
            if update.is_running() {
                gate_names.push(update.gate_name.clone());
            }
        });

        // Should run gates in order: coverage, lint, format, security_audit
        assert_eq!(
            gate_names,
            vec!["coverage", "lint", "format", "security_audit"]
        );
    }

    #[test]
    fn test_run_all_gates_with_progress_includes_duration() {
        let profile = create_test_profile(0, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");

        let mut completed_updates: Vec<GateProgressUpdate> = Vec::new();
        checker.run_all_gates_with_progress(|update| {
            if update.is_completed() {
                completed_updates.push(update);
            }
        });

        // All completed updates should have duration
        for update in &completed_updates {
            assert!(
                update.duration.is_some(),
                "Gate {} should have duration",
                update.gate_name
            );
            assert!(
                update.duration.unwrap().as_nanos() > 0,
                "Duration should be positive"
            );
        }
    }

    #[test]
    fn test_run_all_gates_with_progress_returns_results() {
        let profile = create_test_profile(0, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");

        let mut callback_count = 0;
        let results = checker.run_all_gates_with_progress(|_| {
            callback_count += 1;
        });

        // Should return 4 gate results
        assert_eq!(results.len(), 4);
        assert_eq!(results[0].gate_name, "coverage");
        assert_eq!(results[1].gate_name, "lint");
        assert_eq!(results[2].gate_name, "format");
        assert_eq!(results[3].gate_name, "security_audit");

        // Callback should be called 8 times (2 per gate)
        assert_eq!(callback_count, 8);
    }

    #[test]
    fn test_run_all_gates_with_progress_running_before_complete() {
        let profile = create_test_profile(0, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");

        let mut update_sequence: Vec<(String, GateProgressState)> = Vec::new();
        checker.run_all_gates_with_progress(|update| {
            update_sequence.push((update.gate_name.clone(), update.state));
        });

        // For each gate, Running should come before Passed/Failed
        let gate_order = ["coverage", "lint", "format", "security_audit"];
        for gate in gate_order {
            let running_pos = update_sequence
                .iter()
                .position(|(name, state)| name == gate && *state == GateProgressState::Running);
            let complete_pos = update_sequence.iter().position(|(name, state)| {
                name == gate
                    && matches!(
                        *state,
                        GateProgressState::Passed | GateProgressState::Failed
                    )
            });

            assert!(
                running_pos.is_some(),
                "Gate {} should have Running update",
                gate
            );
            assert!(
                complete_pos.is_some(),
                "Gate {} should have completed update",
                gate
            );
            assert!(
                running_pos.unwrap() < complete_pos.unwrap(),
                "Gate {} Running should come before completed",
                gate
            );
        }
    }

    #[test]
    fn test_run_all_gates_with_progress_matches_run_all_results() {
        let profile = create_test_profile(0, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");

        // Get results from run_all
        let run_all_results = checker.run_all();

        // Get results from run_all_gates_with_progress
        let progress_results = checker.run_all_gates_with_progress(|_| {});

        // Results should match (same gate names and pass/fail status)
        assert_eq!(run_all_results.len(), progress_results.len());
        for (ra, pr) in run_all_results.iter().zip(progress_results.iter()) {
            assert_eq!(ra.gate_name, pr.gate_name);
            assert_eq!(ra.passed, pr.passed);
        }
    }

    #[test]
    fn test_run_all_gates_with_progress_state_matches_result() {
        let profile = create_test_profile(0, false, false, false);
        let checker = QualityGateChecker::new(profile, "/tmp/test");

        let mut completed_states: std::collections::HashMap<String, GateProgressState> =
            std::collections::HashMap::new();

        let results = checker.run_all_gates_with_progress(|update| {
            if update.is_completed() {
                completed_states.insert(update.gate_name.clone(), update.state);
            }
        });

        // Verify that progress state matches result
        for result in results {
            let state = completed_states.get(&result.gate_name).unwrap();
            if result.passed {
                assert_eq!(*state, GateProgressState::Passed);
            } else {
                assert_eq!(*state, GateProgressState::Failed);
            }
        }
    }
}
