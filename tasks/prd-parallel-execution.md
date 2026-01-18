# PRD: Parallel Story Execution

## Introduction

Add parallel execution capabilities to Ralph, enabling multiple independent user stories to execute concurrently. This feature introduces a dependency graph system to determine which stories can run in parallel, conflict detection to identify when parallel changes clash, and a reconciliation pass to clean up inconsistencies. The goal is to significantly reduce total PRD execution time while maintaining code quality and correctness.

## Goals

- Reduce total execution time by running independent stories concurrently
- Better utilize system resources by spawning multiple AI agent processes
- Automatically infer story dependencies from file patterns and semantic similarity
- Detect and handle conflicts between parallel code changes gracefully
- Provide multiple layers of conflict resolution (auto-resolve → fallback → reconciliation)
- Allow both automatic optimization and manual configuration of parallelism settings

## User Stories

### US-001: Extend PRD schema with dependency fields
**Description:** As a PRD author, I want to optionally specify story dependencies and target files so that Ralph can determine safe parallelization.

**Acceptance Criteria:**
- [ ] `PrdUserStory` struct includes optional `dependsOn: Vec<String>` field for explicit dependencies
- [ ] `PrdUserStory` struct includes optional `targetFiles: Vec<String>` field for file glob patterns
- [ ] Existing PRDs without these fields continue to work (backward compatible)
- [ ] Schema validation rejects invalid story IDs in `dependsOn`
- [ ] Typecheck passes (`cargo check`)

### US-002: Implement dependency graph data structure
**Description:** As a developer, I need a DAG data structure to model story dependencies so that I can determine execution order and parallelization opportunities.

**Acceptance Criteria:**
- [ ] `DependencyGraph` struct using `petgraph` crate holds story nodes and dependency edges
- [ ] `from_stories()` builds graph from PRD user stories
- [ ] `validate()` detects cycles and returns descriptive error
- [ ] `topological_order()` returns valid execution order respecting dependencies
- [ ] `get_ready_stories()` returns stories whose dependencies are satisfied
- [ ] Unit tests cover cycle detection, topological sort, and ready story selection
- [ ] Typecheck passes (`cargo check`)

### US-003: Implement file-based dependency inference
**Description:** As a user, I want Ralph to automatically infer dependencies from `targetFiles` overlap so that I don't have to manually specify every dependency.

**Acceptance Criteria:**
- [ ] When two stories have overlapping `targetFiles` patterns, higher-priority story becomes a dependency
- [ ] Glob pattern matching works for patterns like `src/**/*.rs`, `tests/*.py`
- [ ] Inferred dependencies are combined with explicit `dependsOn` (explicit takes precedence)
- [ ] Inference can be disabled via configuration
- [ ] Unit tests verify correct inference from various overlap scenarios
- [ ] Typecheck passes (`cargo check`)

### US-004: Implement embedding-based dependency inference
**Description:** As a user, I want Ralph to use semantic similarity to detect related stories so that even without `targetFiles`, it can identify potential dependencies.

**Acceptance Criteria:**
- [ ] Story titles and descriptions are embedded using local embedding model (fastembed or candle)
- [ ] Stories with high semantic similarity (configurable threshold) are flagged as potentially dependent
- [ ] Embedding inference is combined with file-based inference using configurable weights
- [ ] Embedding computation is cached to avoid redundant processing
- [ ] Can fall back to file-based only if embedding model unavailable
- [ ] Unit tests verify similarity detection between related stories
- [ ] Typecheck passes (`cargo check`)

### US-005: Implement parallel execution scheduler
**Description:** As a user, I want Ralph to execute independent stories concurrently so that total PRD completion time is reduced.

**Acceptance Criteria:**
- [ ] `ParallelRunner` executes stories respecting dependency graph order
- [ ] Configurable max concurrency (default 3, 0 = unlimited)
- [ ] Semaphore limits concurrent story executions
- [ ] Tracks in-flight, completed, and failed stories
- [ ] Waits for dependencies before starting a story
- [ ] Continues with other stories if one fails (doesn't block entire PRD)
- [ ] Typecheck passes (`cargo check`)

### US-006: Implement file locking for parallel execution
**Description:** As a developer, I need file-level locking during parallel execution so that two stories don't corrupt the same file simultaneously.

**Acceptance Criteria:**
- [ ] `ParallelExecutionState` tracks which files are locked by which story
- [ ] Story acquires locks on `targetFiles` before execution starts
- [ ] Lock acquisition fails gracefully if files already locked (triggers sequential fallback)
- [ ] Locks are released when story completes (success or failure)
- [ ] Git operations (add, commit) are serialized through a mutex
- [ ] Unit tests verify lock contention handling
- [ ] Typecheck passes (`cargo check`)

### US-007: Implement AST-based conflict detection
**Description:** As a user, I want Ralph to detect conflicts at the function/class level so that more stories can run in parallel (not just file-level).

**Acceptance Criteria:**
- [ ] `tree-sitter` parses source files to extract functions, classes, and imports
- [ ] Supports at minimum: Rust, Python, TypeScript/JavaScript
- [ ] Detects conflict when two stories modify the same function/class
- [ ] Detects conflict when story A modifies function that story B's code calls
- [ ] Returns list of conflicting entities with file locations
- [ ] Falls back to file-level detection for unsupported languages
- [ ] Unit tests verify entity extraction and conflict detection
- [ ] Typecheck passes (`cargo check`)

### US-008: Implement conflict resolution strategies
**Description:** As a user, I want multiple conflict handling strategies so that Ralph can choose the best approach based on conflict severity.

**Acceptance Criteria:**
- [ ] `ConflictStrategy::EntityBased` (default) uses AST analysis for fine-grained detection
- [ ] `ConflictStrategy::FileBased` uses conservative file-level overlap detection
- [ ] `ConflictStrategy::None` trusts explicit dependencies only
- [ ] Simple conflicts (non-overlapping changes in same file) are auto-merged
- [ ] Complex conflicts trigger sequential fallback for affected stories
- [ ] Strategy is configurable via CLI and prd.json
- [ ] Typecheck passes (`cargo check`)

### US-009: Implement reconciliation pass
**Description:** As a user, I want a cleanup pass after parallel execution so that any inconsistencies are detected and fixed.

**Acceptance Criteria:**
- [ ] `ReconciliationEngine` runs after each batch of parallel stories completes
- [ ] Detects git merge conflicts and reports affected files
- [ ] Runs language-specific type checking (cargo check, tsc, mypy, etc.)
- [ ] Detects and removes duplicate imports
- [ ] Re-runs quality gates on affected files
- [ ] Auto-resolves trivial issues (duplicate imports, formatting)
- [ ] Flags complex issues for manual review or re-run
- [ ] Typecheck passes (`cargo check`)

### US-010: Implement automatic concurrency optimization
**Description:** As a user, I want Ralph to automatically determine optimal concurrency so that I get good performance without manual tuning.

**Acceptance Criteria:**
- [ ] Analyzes dependency graph to determine maximum useful parallelism
- [ ] Considers system resources (CPU cores, available memory)
- [ ] Reduces concurrency if conflict rate exceeds threshold
- [ ] Increases concurrency if stories are completing without conflicts
- [ ] Logs concurrency decisions for transparency
- [ ] Can be overridden by explicit `--max-concurrency` flag
- [ ] Typecheck passes (`cargo check`)

### US-011: Add parallel execution CLI flags
**Description:** As a developer, I want CLI flags to control parallel execution so that I can tune behavior for different scenarios.

**Acceptance Criteria:**
- [ ] `--parallel` flag enables parallel execution (default: off for backward compatibility)
- [ ] `--max-concurrency N` sets max concurrent stories (default: 3, 0 = unlimited)
- [ ] `--conflict-strategy [entity|file|none]` sets detection strategy (default: entity)
- [ ] `--inference-mode [files|embeddings|hybrid]` sets dependency inference (default: hybrid)
- [ ] `--no-fallback` disables sequential fallback on conflict
- [ ] `--no-reconcile` disables reconciliation pass
- [ ] Help text explains each flag
- [ ] Typecheck passes (`cargo check`)

### US-012: Add parallel configuration to prd.json
**Description:** As a PRD author, I want to configure parallelism in the PRD file so that settings are project-specific and version-controlled.

**Acceptance Criteria:**
- [ ] `PrdFile` struct includes optional `parallel` configuration section
- [ ] Supports `enabled`, `maxConcurrency`, `conflictStrategy`, `inferenceMode` fields
- [ ] PRD-level settings override CLI defaults
- [ ] CLI flags override PRD-level settings (highest precedence)
- [ ] Missing parallel section uses sensible defaults
- [ ] Typecheck passes (`cargo check`)

### US-013: Integrate parallel runner into main runner
**Description:** As a developer, I need to wire up the parallel runner so that users can actually use parallel execution.

**Acceptance Criteria:**
- [ ] `Runner::run()` routes to `ParallelRunner` when parallel mode enabled
- [ ] Existing sequential behavior preserved when parallel mode disabled
- [ ] Progress display shows multiple in-flight stories
- [ ] Final summary shows parallel execution statistics (time saved, conflicts resolved)
- [ ] Error handling gracefully degrades to sequential if parallel infrastructure fails
- [ ] Typecheck passes (`cargo check`)

### US-014: Write integration tests for parallel execution
**Description:** As a developer, I need integration tests to verify parallel execution works end-to-end.

**Acceptance Criteria:**
- [ ] Test: 3 independent stories execute in parallel, all pass
- [ ] Test: Stories with explicit dependencies execute in correct order
- [ ] Test: Conflicting stories fall back to sequential execution
- [ ] Test: Reconciliation detects and reports type errors
- [ ] Test: CLI flags correctly control parallel behavior
- [ ] Test: prd.json parallel config is respected
- [ ] All tests pass (`cargo test`)

### US-015: Add parallel execution documentation
**Description:** As a user, I want documentation explaining parallel execution so that I can use it effectively.

**Acceptance Criteria:**
- [ ] README section explains parallel execution feature
- [ ] Documents `dependsOn` and `targetFiles` PRD fields with examples
- [ ] Documents all CLI flags with usage examples
- [ ] Documents prd.json parallel configuration
- [ ] Includes troubleshooting guide for common conflict scenarios
- [ ] Documents how to interpret parallel execution logs

## Functional Requirements

- FR-1: Add `dependsOn` and `targetFiles` optional fields to `PrdUserStory` struct
- FR-2: Implement `DependencyGraph` using petgraph with cycle detection and topological sort
- FR-3: Infer dependencies from `targetFiles` glob pattern overlap
- FR-4: Infer dependencies from semantic similarity using local embeddings
- FR-5: Combine file-based and embedding-based inference with configurable weights
- FR-6: Implement `ParallelRunner` with semaphore-based concurrency control
- FR-7: Track file locks to prevent concurrent modification of same files
- FR-8: Serialize git operations (add, commit) through a mutex
- FR-9: Parse source files with tree-sitter to extract code entities
- FR-10: Detect conflicts at entity level (function, class, import)
- FR-11: Auto-resolve simple conflicts (non-overlapping changes in same file)
- FR-12: Fall back to sequential execution for complex conflicts
- FR-13: Run reconciliation pass after parallel batch completion
- FR-14: Auto-fix trivial issues (duplicate imports, formatting)
- FR-15: Re-run quality gates after reconciliation
- FR-16: Automatically optimize concurrency based on conflict rate
- FR-17: Expose parallel settings via CLI flags
- FR-18: Allow parallel configuration in prd.json
- FR-19: CLI flags take precedence over prd.json settings
- FR-20: Preserve backward compatibility (parallel off by default)

## Non-Goals

- No distributed execution across multiple machines (single machine only)
- No support for multiple PRDs executing in parallel (one PRD at a time)
- No automatic retry of failed stories (existing behavior preserved)
- No real-time conflict resolution during story execution (detect after completion)
- No GUI for visualizing dependency graph (CLI output only)
- No support for circular dependencies (must be a valid DAG)

## Design Considerations

- Progress display must clearly show which stories are executing in parallel
- Conflict messages should identify specific files/entities involved
- Logs should indicate when sequential fallback is triggered and why
- Reconciliation issues should provide actionable fix suggestions

## Technical Considerations

- **petgraph**: DAG data structure and algorithms (topological sort, cycle detection)
- **tree-sitter**: Multi-language AST parsing (rust, python, typescript grammars)
- **fastembed** or **candle**: Local embedding model for semantic similarity
- **glob**: Pattern matching for `targetFiles`
- **tokio**: Async runtime with semaphore for concurrency control
- Existing `Arc<RwLock<>>` patterns in codebase should be followed for shared state
- Git mutex should use `tokio::sync::Mutex` for async compatibility

## Success Metrics

- PRDs with 5+ independent stories complete in < 50% of sequential time
- Conflict detection catches > 95% of actual conflicts (minimal false negatives)
- False positive rate for conflict detection < 20% (don't over-sequentialize)
- Reconciliation auto-resolves > 80% of trivial issues
- Zero data loss or corruption from parallel execution

## Open Questions

- Should embedding model be bundled or downloaded on first use?
- What's the right semantic similarity threshold for dependency inference?
- Should we support custom tree-sitter grammars for additional languages?
- How should parallel execution interact with `--max-iterations` limit?
