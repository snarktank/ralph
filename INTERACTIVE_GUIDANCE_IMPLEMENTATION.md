# Interactive Steering Corrections Implementation

## Overview

This document describes the complete implementation of interactive steering corrections for Ralph Macchio. This allows users to provide guidance when stories get stuck, all within the same CLI session.

## What Was Implemented

### 1. SteeringGuidance Struct (src/iteration/context.rs)
- Added `SteeringGuidance` struct to capture user guidance
- Fields for guidance text, modified acceptance criteria, focus/avoid files, relaxed gates
- Method to build prompt section from guidance
- Integrated into `IterationContext` as optional field

### 2. ExecutionResult Extension (src/mcp/tools/executor.rs)
- Added `needs_guidance: bool` field to `ExecutionResult`
- Set to `true` when `FutilityVerdict::PauseForGuidance` is detected
- Updated all ExecutionResult construction sites

### 3. Executor Methods (src/mcp/tools/executor.rs)
- Added `continue_with_guidance()` method to resume with user input
- Refactored `execute_story()` to use internal `execute_story_with_context()`
- Steering guidance is injected into agent prompts automatically

### 4. Interactive Prompt Module (src/interactive_guidance.rs)
- `display_guidance_prompt()` - Shows failure summary and suggestions
- `prompt_guidance_choice()` - Prompts user for action (provide guidance/skip/abort)
- `prompt_guidance_text()` - Collects multi-line guidance from user

## How To Integrate Into Runner

Add this code to `src/runner.rs` in the story execution result handling (around line 355):

```rust
match result {
    Ok(exec_result) => {
        // Check if guidance is needed
        if exec_result.needs_guidance {
            use crate::interactive_guidance::{
                display_guidance_prompt, prompt_guidance_choice, prompt_guidance_text,
                GuidanceChoice,
            };

            // Display the failure summary and suggestions
            display_guidance_prompt(&exec_result);

            // Prompt user for choice
            match prompt_guidance_choice() {
                Ok(GuidanceChoice::ProvideGuidance) => {
                    // Get guidance from user
                    let current_iter = exec_result.iterations_used;
                    match prompt_guidance_text(&story_id, current_iter) {
                        Ok(guidance) => {
                            println!("\n✓ Retrying with your guidance...\n");

                            // Continue execution with guidance
                            if let Some(context) = exec_result.iteration_context {
                                let retry_result = executor
                                    .continue_with_guidance(
                                        &story_id,
                                        context,
                                        guidance,
                                        cancel_rx,
                                        |iter, _max| {
                                            let adjusted_iter = iter + start_iteration - 1;
                                            display.update_iteration(adjusted_iter, max_iterations);
                                        },
                                    )
                                    .await;

                                // Handle retry result
                                match retry_result {
                                    Ok(retry_exec) => {
                                        if retry_exec.success {
                                            self.clear_checkpoint();
                                            display.complete_story(&story_id, retry_exec.commit_hash.as_deref());
                                        } else if retry_exec.needs_guidance {
                                            // Needs guidance again - loop back to prompt
                                            // (could add a max guidance attempts limit here)
                                            continue; // Will loop back for another guidance attempt
                                        } else {
                                            // Failed for other reasons
                                            self.save_checkpoint(
                                                &story_id,
                                                retry_exec.iterations_used,
                                                max_iterations,
                                                PauseReason::Error(
                                                    retry_exec.error.clone().unwrap_or_else(|| "Failed after guidance".to_string())
                                                ),
                                            );
                                            display.fail_story(&story_id, retry_exec.error.as_deref().unwrap_or("unknown"));
                                        }
                                    }
                                    Err(e) => {
                                        display.fail_story(&story_id, &e.to_string());
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to get guidance: {}", e);
                            display.fail_story(&story_id, "Failed to get user guidance");
                        }
                    }
                }
                Ok(GuidanceChoice::SkipStory) => {
                    println!("\n⏭  Skipping story {} and continuing...\n", story_id);
                    self.clear_checkpoint();
                    display.fail_story(&story_id, "Skipped by user");
                    // Continue to next story
                }
                Ok(GuidanceChoice::Abort) => {
                    println!("\n🛑 Aborting execution as requested.\n");
                    self.save_checkpoint(
                        &story_id,
                        exec_result.iterations_used,
                        max_iterations,
                        PauseReason::UserRequested,
                    );
                    return RunResult {
                        all_passed: false,
                        stories_passed: self.count_passing_stories().unwrap_or(0),
                        total_stories,
                        total_iterations,
                        error: Some("User aborted execution".to_string()),
                    };
                }
                Err(e) => {
                    eprintln!("Failed to get user choice: {}", e);
                    display.fail_story(&story_id, "Failed to get user input");
                }
            }
        } else if exec_result.success {
            // Normal success path
            self.clear_checkpoint();
            display.complete_story(&story_id, exec_result.commit_hash.as_deref());
        } else {
            // Normal failure path (not needs_guidance)
            let final_iteration = start_iteration + exec_result.iterations_used - 1;
            self.save_checkpoint(
                &story_id,
                final_iteration,
                max_iterations,
                PauseReason::Error(
                    exec_result.error.clone().unwrap_or_else(|| "Quality gates failed".to_string()),
                ),
            );
            display.fail_story(&story_id, exec_result.error.as_deref().unwrap_or("unknown"));
        }
    }
    Err(e) => {
        // Handle errors as before
        // ... existing error handling code ...
    }
}
```

## User Experience Flow

```
Story US-061 executing...
Iteration 1: FAIL (compilation error)
Iteration 2: FAIL (compilation error)
Iteration 3: FAIL (compilation error)

================================================================================
⚠️  STORY NEEDS GUIDANCE
================================================================================
Reason: Error 'compilation:none' has occurred 3 times. Consider providing
additional context or breaking down the task.

Analysis:
  Same error repeating indicates the agent may not be able to resolve this
  issue without guidance.

Suggestions:
  • Provide more specific implementation guidance
  • Break the story into smaller subtasks
  • Check for missing dependencies or prerequisites

Recent Errors:
  Iteration 1: [compilation] cargo check failed: missing field `created_at`
  Iteration 2: [compilation] cargo check failed: missing field `created_at`
  Iteration 3: [compilation] cargo check failed: missing field `created_at`

What would you like to do?
  1. Provide guidance and retry (recommended)
  2. Skip this story and continue with others
  3. Abort execution

Your choice (1-3): 1

================================================================================
Provide guidance for story US-061
================================================================================
Describe what you want the agent to do differently or additional
context to help it succeed. Be specific about the issue and solution.
(Press Ctrl+D or Ctrl+Z when done, or type 'END' on a line by itself)

> The struct is missing the created_at field. Add it with type DateTime<Utc>
> from the chrono crate. Make sure to import chrono::DateTime and Utc.
> END

Guidance received (142 characters)

✓ Retrying with your guidance...

Iteration 4: Running with user guidance...
Iteration 4: PASS ✓

Story US-061 completed successfully!

Continuing with remaining stories...
```

## Benefits

1. **No Manual Restarts**: Users stay in the same session
2. **Context Preservation**: Full error history available when prompting
3. **Smart Detection**: Only prompts when futility detector identifies stuck patterns
4. **Flexible Actions**: Users can provide guidance, skip, or abort
5. **Rich Context**: Shows recent errors, suggestions, and pattern analysis
6. **Immediate Application**: Guidance is injected into the next iteration's prompt

## Testing

To test the interactive flow:

```bash
# Create a story that will trigger stagnation
# (e.g., a story with unclear requirements that causes repeated failures)
ralph run

# When prompted for guidance:
# 1. Try providing helpful guidance
# 2. Verify it continues with that guidance
# 3. Check that the guidance appears in agent prompts
```

## Future Enhancements

1. **Guidance History**: Track what guidance was provided and outcomes
2. **Guidance Templates**: Suggest common fixes based on error patterns
3. **File Selection**: Interactive file picker for focus/avoid files
4. **Gate Relaxation**: Interactive selection of gates to temporarily disable
5. **Multi-attempt Limit**: Limit guidance attempts before forcing skip/abort
6. **Guidance Learning**: Learn from successful guidance patterns

## Key Files Modified

- `src/iteration/context.rs` - Added SteeringGuidance struct
- `src/mcp/tools/executor.rs` - Added continue_with_guidance method
- `src/interactive_guidance.rs` - New module for CLI prompts
- `src/lib.rs` - Registered new module
- `src/runner.rs` - Integration point (see code above)
