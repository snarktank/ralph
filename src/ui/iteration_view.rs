//! Iteration view component for Ralph's terminal UI.
//!
//! Displays what's planned before each iteration starts,
//! showing the gates that will run.

#![allow(dead_code)]

use owo_colors::OwoColorize;

use crate::ui::colors::Theme;

/// Preview of what will happen in an iteration.
///
/// Shows the list of quality gates that will be executed.
#[derive(Debug, Clone)]
pub struct IterationPreview {
    /// Names of the gates to be run
    gates: Vec<String>,
    /// Color theme for rendering
    theme: Theme,
}

impl IterationPreview {
    /// Create a new iteration preview with the given gates.
    pub fn new(gates: Vec<String>) -> Self {
        Self {
            gates,
            theme: Theme::default(),
        }
    }

    /// Create an iteration preview with a custom theme.
    pub fn with_theme(gates: Vec<String>, theme: Theme) -> Self {
        Self { gates, theme }
    }

    /// Get the list of gates to be run.
    pub fn gates(&self) -> &[String] {
        &self.gates
    }

    /// Render the pre-iteration header showing gates to run.
    ///
    /// Format: "Will run: build → lint → test"
    pub fn render(&self) -> String {
        if self.gates.is_empty() {
            return format!(
                "{} No gates configured\n",
                "○".color(self.theme.muted)
            );
        }

        let gate_chain = self
            .gates
            .iter()
            .map(|g| g.as_str())
            .collect::<Vec<_>>()
            .join(" → ");

        format!(
            "{} {}\n",
            "Will run:".color(self.theme.muted),
            gate_chain.color(self.theme.in_progress)
        )
    }

    /// Render as a compact inline format.
    ///
    /// Format: "build → lint → test"
    pub fn render_compact(&self) -> String {
        if self.gates.is_empty() {
            return "No gates".to_string();
        }

        self.gates
            .iter()
            .map(|g| g.as_str())
            .collect::<Vec<_>>()
            .join(" → ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iteration_preview_new() {
        let gates = vec!["build".to_string(), "lint".to_string(), "test".to_string()];
        let preview = IterationPreview::new(gates.clone());
        assert_eq!(preview.gates(), &gates);
    }

    #[test]
    fn test_iteration_preview_empty() {
        let preview = IterationPreview::new(vec![]);
        assert!(preview.gates().is_empty());
        let output = preview.render();
        assert!(output.contains("No gates configured"));
    }

    #[test]
    fn test_iteration_preview_render() {
        let gates = vec!["build".to_string(), "lint".to_string(), "test".to_string()];
        let preview = IterationPreview::new(gates);
        let output = preview.render();

        // Check that the output contains the gate names
        assert!(output.contains("Will run:"));
        assert!(output.contains("build"));
        assert!(output.contains("lint"));
        assert!(output.contains("test"));
        // Check for arrow separator
        assert!(output.contains("→"));
    }

    #[test]
    fn test_iteration_preview_render_compact() {
        let gates = vec!["build".to_string(), "lint".to_string()];
        let preview = IterationPreview::new(gates);
        let output = preview.render_compact();

        assert_eq!(output, "build → lint");
    }

    #[test]
    fn test_iteration_preview_render_compact_empty() {
        let preview = IterationPreview::new(vec![]);
        let output = preview.render_compact();

        assert_eq!(output, "No gates");
    }

    #[test]
    fn test_iteration_preview_with_theme() {
        let gates = vec!["format".to_string()];
        let theme = Theme::default();
        let preview = IterationPreview::with_theme(gates.clone(), theme);

        assert_eq!(preview.gates(), &gates);
    }

    #[test]
    fn test_iteration_preview_single_gate() {
        let gates = vec!["typecheck".to_string()];
        let preview = IterationPreview::new(gates);
        let output = preview.render_compact();

        // Single gate should not have arrow
        assert_eq!(output, "typecheck");
        assert!(!output.contains("→"));
    }
}
