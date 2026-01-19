//! Interactive guidance prompts for steering stuck stories.
//!
//! This module provides the CLI interface for prompting users to provide
//! steering corrections when stories are detected as stuck or failing.

use std::io::{self, Write};

use crate::iteration::context::SteeringGuidance;
use crate::iteration::futility::FutilityVerdict;
use crate::mcp::tools::executor::ExecutionResult;

/// Choice the user can make when prompted for guidance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuidanceChoice {
    /// Provide guidance and continue
    ProvideGuidance,
    /// Skip this story and move to the next
    SkipStory,
    /// Abort the entire execution
    Abort,
}

/// Display the failure summary and futility verdict to the user.
pub fn display_guidance_prompt(result: &ExecutionResult) {
    println!("\n{}", "=".repeat(80));
    println!("⚠️  STORY NEEDS GUIDANCE");
    println!("{}\n", "=".repeat(80));

    // Display the error if available
    if let Some(ref error) = result.error {
        println!("Reason: {}\n", error);
    }

    // Display suggestions from futility verdict
    if let Some(FutilityVerdict::PauseForGuidance {
        reason,
        suggestions,
    }) = &result.futility_verdict
    {
        println!("Analysis:");
        println!("  {}\n", reason);

        if !suggestions.is_empty() {
            println!("Suggestions:");
            for suggestion in suggestions {
                println!("  • {}", suggestion);
            }
            println!();
        }
    }

    // Display recent errors from iteration context
    if let Some(ref context) = result.iteration_context {
        if !context.error_history.is_empty() {
            println!("Recent Errors:");
            for error in context.error_history.iter().rev().take(3).rev() {
                println!(
                    "  Iteration {}: [{}] {}",
                    error.iteration,
                    error.category.as_str(),
                    truncate_string(&error.message, 100)
                );
            }
            println!();
        }
    }
}

/// Prompt the user to choose an action.
pub fn prompt_guidance_choice() -> io::Result<GuidanceChoice> {
    println!("What would you like to do?");
    println!("  1. Provide guidance and retry (recommended)");
    println!("  2. Skip this story and continue with others");
    println!("  3. Abort execution");
    print!("\nYour choice (1-3): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" => Ok(GuidanceChoice::ProvideGuidance),
        "2" => Ok(GuidanceChoice::SkipStory),
        "3" => Ok(GuidanceChoice::Abort),
        _ => {
            println!("Invalid choice. Please enter 1, 2, or 3.");
            prompt_guidance_choice()
        }
    }
}

/// Prompt the user to provide guidance text.
pub fn prompt_guidance_text(
    story_id: &str,
    current_iteration: u32,
) -> io::Result<SteeringGuidance> {
    println!("\n{}", "=".repeat(80));
    println!("Provide guidance for story {}", story_id);
    println!("{}", "=".repeat(80));
    println!("Describe what you want the agent to do differently or additional");
    println!("context to help it succeed. Be specific about the issue and solution.");
    println!("(Press Ctrl+D or Ctrl+Z when done, or type 'END' on a line by itself)\n");

    let mut guidance_lines = Vec::new();
    let stdin = io::stdin();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break, // EOF (Ctrl+D)
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed == "END" {
                    break;
                }
                guidance_lines.push(line);
            }
            Err(e) => return Err(e),
        }
    }

    let guidance_text = guidance_lines.join("");

    if guidance_text.trim().is_empty() {
        println!("\nNo guidance provided. Please try again.");
        return prompt_guidance_text(story_id, current_iteration);
    }

    println!("\nGuidance received ({} characters)", guidance_text.len());

    Ok(SteeringGuidance::new(guidance_text, current_iteration))
}

/// Truncate a string to a maximum length, adding ellipsis if needed.
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_string_short() {
        assert_eq!(truncate_string("short", 100), "short");
    }

    #[test]
    fn test_truncate_string_exact() {
        assert_eq!(truncate_string("exact", 5), "exact");
    }

    #[test]
    fn test_truncate_string_long() {
        let result = truncate_string("this is a very long string", 10);
        assert_eq!(result, "this is...");
        assert!(result.len() <= 10);
    }

    #[test]
    fn test_steering_guidance_new() {
        let guidance = SteeringGuidance::new("test guidance", 5);
        assert_eq!(guidance.guidance_text, "test guidance");
        assert_eq!(guidance.provided_at_iteration, 5);
        assert!(guidance.focus_files.is_empty());
    }
}
