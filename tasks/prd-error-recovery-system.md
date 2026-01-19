# PRD: Error Handling & Recovery System

## Introduction

Implement a robust error handling, checkpoint/resume, and graceful degradation system for Ralph to handle API rate limits, usage quotas (Claude Code plan limits), timeouts, and stalled executions. This system ensures execution can be paused, checkpointed, and resumed without losing progress, while providing clear visibility into system state.

## Goals

- Classify errors into actionable categories (transient, usage limit, fatal, timeout)
- Provide configurable recovery strategies (retry, pause, stop)
- Save execution state at checkpoints for resume capability
- Detect stalled agents via heartbeat monitoring and hard timeouts
- Give users clear visibility into execution state and recovery actions
- Support resuming from checkpoints across sessions

## User Stories

### US-001: Error Classification System
**Description:** As Ralph, I need to classify errors from agent execution so I can determine the appropriate recovery strategy.

**Acceptance Criteria:**
- [ ] ErrorCategory enum with variants: Transient, UsageLimit, Fatal, Timeout
- [ ] Pattern-based detector for Claude Code error messages (rate limit, usage limit, auth errors)
- [ ] RecoveryHint enum providing actionable guidance (RetryNow, RetryAfter, WaitForUser, StopExecution, ResumeFromCheckpoint)
- [ ] Exit code analysis for timeout detection
- [ ] Unit tests for pattern matching on known error messages
- [ ] Typecheck/lint passes

### US-002: Checkpoint Save/Load
**Description:** As a user, I want execution state saved to `.ralph/checkpoint.json` so I can resume after interruptions.

**Acceptance Criteria:**
- [ ] Checkpoint struct with version, timestamp, story state, pause reason, uncommitted files
- [ ] CheckpointManager with save(), load(), clear(), verify() methods
- [ ] Checkpoint saved before each iteration starts
- [ ] Checkpoint saved when pausing for any reason
- [ ] Checkpoint cleared on successful story completion
- [ ] JSON schema validation on load
- [ ] Typecheck/lint passes

### US-003: Checkpoint Resume on Startup
**Description:** As a user, I want Ralph to detect existing checkpoints and prompt me to resume so I don't lose progress.

**Acceptance Criteria:**
- [ ] On startup, check for `.ralph/checkpoint.json`
- [ ] Display checkpoint summary (story ID, iteration, pause reason, age)
- [ ] Prompt user: Resume / Discard / View details
- [ ] --resume flag to skip prompt and auto-resume
- [ ] --no-resume flag to skip prompt and start fresh
- [ ] Typecheck/lint passes

### US-004: Agent Execution Timeout
**Description:** As Ralph, I need to timeout agent execution that exceeds time limits so stuck agents don't block forever.

**Acceptance Criteria:**
- [ ] TimeoutConfig struct with configurable durations
- [ ] Wrap agent execution with tokio::time::timeout()
- [ ] Default agent timeout: 10 minutes
- [ ] Default iteration timeout: 15 minutes
- [ ] --timeout CLI flag to override agent timeout
- [ ] On timeout, save checkpoint before stopping
- [ ] Typecheck/lint passes

### US-005: Heartbeat Stall Detection
**Description:** As Ralph, I need to detect stalled agents via heartbeat so I can intervene before hard timeout.

**Acceptance Criteria:**
- [ ] HeartbeatMonitor background task
- [ ] Configurable heartbeat interval (default: 30 seconds)
- [ ] Configurable missed heartbeats before stall (default: 3)
- [ ] Agent executor updates heartbeat on activity
- [ ] On stall detection, warn user before hard timeout
- [ ] Typecheck/lint passes

### US-006: Retry with Backoff
**Description:** As Ralph, I need to retry transient errors with exponential backoff so temporary issues don't fail the execution.

**Acceptance Criteria:**
- [ ] Exponential backoff calculator (base: 1s, max: 60s, jitter: 10%)
- [ ] Automatic retry for Transient errors up to max retries (default: 3)
- [ ] Display countdown timer during retry wait
- [ ] Cancel retry with Ctrl+C (saves checkpoint)
- [ ] Typecheck/lint passes

### US-007: Rate Limit Handling
**Description:** As Ralph, I need to handle rate limits by waiting the specified duration so execution can continue automatically.

**Acceptance Criteria:**
- [ ] Detect rate limit errors (429, "rate limit", "too many requests")
- [ ] Parse retry-after header or default to 60 seconds
- [ ] Display countdown with rate limit notification
- [ ] Auto-continue after wait period
- [ ] Allow user to skip wait (proceed at risk)
- [ ] Typecheck/lint passes

### US-008: Usage Limit Pause
**Description:** As a user, I want Ralph to pause when I hit my Claude Code plan limit so I can resume later without losing work.

**Acceptance Criteria:**
- [ ] Detect usage limit errors ("plan limit", "usage limit", "quota exceeded")
- [ ] Save checkpoint immediately
- [ ] Display clear notification with action required
- [ ] Exit with special exit code (75 = temp failure)
- [ ] On next run, prompt to resume from checkpoint
- [ ] Typecheck/lint passes

### US-009: Pause/Resume Control
**Description:** As a user, I want to manually pause execution so I can take a break and resume later.

**Acceptance Criteria:**
- [ ] PauseController for pause state management
- [ ] 'p' key to request pause (between iterations)
- [ ] Pause saves checkpoint and exits gracefully
- [ ] Resume from checkpoint on next run
- [ ] Display "Pausing after current iteration..." message
- [ ] Typecheck/lint passes

### US-010: User Notifications
**Description:** As a user, I want clear notifications about errors and recovery actions so I understand what's happening.

**Acceptance Criteria:**
- [ ] Notification enum for different scenarios
- [ ] Themed notification panels (error, warning, info, success)
- [ ] Rate limit: show countdown timer
- [ ] Usage limit: show action required
- [ ] Timeout: show checkpoint confirmation
- [ ] Retry: show attempt number and delay
- [ ] Typecheck/lint passes

### US-011: Execution State Extensions
**Description:** As Ralph, I need new execution states to represent pause and retry conditions.

**Acceptance Criteria:**
- [ ] Add Paused variant to ExecutionState enum
- [ ] Add WaitingForRetry variant with countdown info
- [ ] Update get_status MCP tool to return new states
- [ ] Update UI display to handle new states
- [ ] Typecheck/lint passes

### US-012: CLI Status Command
**Description:** As a user, I want a `ralph status` command so I can check execution state without starting a run.

**Acceptance Criteria:**
- [ ] `ralph status` command shows current/last execution state
- [ ] Display checkpoint info if present
- [ ] Show pause reason and suggested action
- [ ] Exit code reflects state (0=idle, 1=failed, 75=paused)
- [ ] Typecheck/lint passes

## Functional Requirements

- FR-1: Classify errors into categories: Transient, UsageLimit, Fatal, Timeout
- FR-2: Provide RecoveryHint for each error category with specific action guidance
- FR-3: Detect Claude Code errors via pattern matching on stderr/stdout
- FR-4: Save checkpoint to `.ralph/checkpoint.json` at iteration boundaries
- FR-5: Save checkpoint immediately on any pause or error condition
- FR-6: Load and verify checkpoint integrity on resume
- FR-7: Prompt for resume when checkpoint exists on startup
- FR-8: Support --resume and --no-resume CLI flags
- FR-9: Wrap agent execution with configurable timeout (default 10 min)
- FR-10: Monitor agent heartbeat for stall detection (30s interval, 3 missed = stall)
- FR-11: Retry transient errors with exponential backoff (max 3 retries)
- FR-12: Wait for rate limit reset with countdown display
- FR-13: Pause and checkpoint on usage limit detection
- FR-14: Allow manual pause via 'p' key during execution
- FR-15: Display themed notification panels for all error states
- FR-16: Add Paused and WaitingForRetry states to ExecutionState
- FR-17: Provide `ralph status` command for state inspection

## Non-Goals

- No automatic billing/payment integration for usage limits
- No distributed checkpoint storage (local file only)
- No checkpoint encryption or compression
- No rollback to previous checkpoints (single checkpoint only)
- No agent-level retry (retry at iteration level only)
- No integration with external monitoring systems

## Technical Considerations

- Use serde for checkpoint JSON serialization
- Use tokio::time::timeout for hard timeouts
- Use tokio::sync::watch for heartbeat signaling
- Checkpoint versioning for forward compatibility
- Pattern matching uses case-insensitive regex
- Exit codes: 0=success, 1=error, 75=temp failure (paused)
- Reuse existing RalphDisplay for notifications

## Module Structure

```
src/
├── error/
│   ├── mod.rs              # Module exports
│   ├── classification.rs   # ErrorCategory, ClassifiedError, RecoveryHint
│   └── detector.rs         # Pattern matching for error classification
├── checkpoint/
│   ├── mod.rs              # Checkpoint, StoryCheckpoint, PauseReason
│   └── manager.rs          # CheckpointManager save/load/verify
├── timeout/
│   ├── mod.rs              # TimeoutConfig
│   └── heartbeat.rs        # HeartbeatMonitor background task
├── pause/
│   └── mod.rs              # PauseController, PauseState
└── notification/
    ├── mod.rs              # Notification enum variants
    └── renderer.rs         # Themed notification panels
```

## Success Metrics

- Zero data loss on usage limit interruption (checkpoint recovery works)
- Resume from checkpoint completes story successfully
- Transient errors retry automatically without user intervention
- Users understand system state via clear notifications
- Timeout detection catches stalled agents within 2 heartbeat intervals

## Open Questions

- Should checkpoint include git stash of uncommitted changes?
- Should there be a max checkpoint age before auto-discard?
- Should rate limit wait be skippable by the user?
