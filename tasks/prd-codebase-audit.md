# PRD: Codebase Audit Feature

## Introduction

Add a comprehensive codebase audit, inventory, and evaluation feature to Ralph that enables developers and AI agents to deeply understand any codebase before starting work. The feature scans and analyzes codebases across multiple languages (Rust, TypeScript, Python, Go), identifies architecture gaps and improvement opportunities, conducts interactive Q&A to refine understanding, and generates actionable PRDs with ralph-ready prd.json files.

This creates a complete audit-to-execution pipeline: `ralph audit` → findings → PRD → prd.json → `ralph run`.

## Goals

- Provide pre-development context by analyzing codebase structure, patterns, and conventions
- Assess project health including test coverage, documentation gaps, and technical debt
- Detect architecture gaps and identify complementary feature opportunities
- Support multiple languages from day one (Rust, TypeScript/JS, Python, Go)
- Enable interactive refinement through smart Q&A (ask questions when confidence is low)
- Generate improvement PRDs automatically from audit findings
- Convert audit-generated PRDs to prd.json for immediate Ralph execution
- Serve both CLI users (`ralph audit`) and AI agents (MCP tools) equally

## User Stories

### US-001: Create audit module structure
**Description:** As a developer, I need the audit module scaffolding so other audit features can be built on top of it.

**Acceptance Criteria:**
- [ ] Create `src/audit/mod.rs` with module organization and re-exports
- [ ] Define core types: `AuditReport`, `AuditFinding`, `FeatureOpportunity`, `Severity`
- [ ] Define `AuditError` enum with thiserror
- [ ] Add `mod audit;` to `src/main.rs`
- [ ] Typecheck passes

### US-002: Implement file inventory scanner
**Description:** As a user, I want the audit to scan the directory structure so I can see what files and folders exist in the codebase.

**Acceptance Criteria:**
- [ ] Create `src/audit/inventory.rs` with `InventoryScanner` struct
- [ ] Scan directory tree respecting .gitignore patterns
- [ ] Count files by extension
- [ ] Identify key files (README, Cargo.toml, package.json, etc.)
- [ ] Calculate total lines of code
- [ ] Return `FileInventory` struct with results
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-003: Implement language detection
**Description:** As a user, I want the audit to detect which programming languages are used so I understand the tech stack.

**Acceptance Criteria:**
- [ ] Create `src/audit/languages/mod.rs` with `LanguageDetector`
- [ ] Detect Rust (Cargo.toml, .rs files)
- [ ] Detect TypeScript/JavaScript (package.json, .ts/.js files)
- [ ] Detect Python (pyproject.toml, requirements.txt, .py files)
- [ ] Detect Go (go.mod, .go files)
- [ ] Return primary and secondary languages with percentages
- [ ] Unit tests for each language detection
- [ ] Typecheck passes

### US-004: Implement dependency parser
**Description:** As a user, I want the audit to analyze dependencies so I can see what libraries the project uses and if any are outdated.

**Acceptance Criteria:**
- [ ] Create `src/audit/dependencies.rs` with `DependencyParser`
- [ ] Parse Cargo.toml for Rust dependencies
- [ ] Parse package.json for npm dependencies
- [ ] Parse pyproject.toml/requirements.txt for Python dependencies
- [ ] Parse go.mod for Go dependencies
- [ ] Return `DependencyAnalysis` with direct and dev dependencies
- [ ] Unit tests for each ecosystem parser
- [ ] Typecheck passes

### US-005: Implement code pattern detection
**Description:** As a user, I want the audit to detect code patterns and conventions so agents can follow existing practices.

**Acceptance Criteria:**
- [ ] Create `src/audit/patterns.rs` with `PatternAnalyzer`
- [ ] Detect naming conventions (snake_case, camelCase, PascalCase)
- [ ] Detect module organization patterns
- [ ] Detect error handling patterns
- [ ] Detect async patterns if applicable
- [ ] Return `PatternAnalysis` struct
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-006: Implement architecture analysis
**Description:** As a user, I want the audit to identify the architecture pattern so I understand how the codebase is organized.

**Acceptance Criteria:**
- [ ] Create `src/audit/architecture.rs` with `ArchitectureAnalyzer`
- [ ] Detect architecture patterns (layered, modular, hexagonal, clean, etc.)
- [ ] Identify module/layer boundaries
- [ ] Detect coupling between modules
- [ ] Return `ArchitectureAnalysis` struct
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-007: Implement API inventory
**Description:** As a user, I want the audit to discover API endpoints and interfaces so I know what the codebase exposes.

**Acceptance Criteria:**
- [ ] Create `src/audit/api.rs` with `ApiInventory`
- [ ] Detect HTTP endpoints (axum, actix, express patterns)
- [ ] Detect CLI commands (clap patterns)
- [ ] Detect MCP tools if present
- [ ] Return `ApiAnalysis` with endpoints, commands, tools
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-008: Implement test coverage analysis
**Description:** As a user, I want the audit to analyze test coverage so I know which areas lack tests.

**Acceptance Criteria:**
- [ ] Create `src/audit/testing.rs` with `TestAnalyzer`
- [ ] Count test files and test functions
- [ ] Identify untested modules (modules with no corresponding tests)
- [ ] Detect test patterns (unit, integration, e2e)
- [ ] Return `TestAnalysis` struct
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-009: Implement documentation gap detection
**Description:** As a user, I want the audit to find missing documentation so I know what needs to be documented.

**Acceptance Criteria:**
- [ ] Create `src/audit/documentation.rs` with `DocAnalyzer`
- [ ] Check for README presence and completeness
- [ ] Detect missing doc comments on public items (Rust)
- [ ] Identify undocumented API endpoints
- [ ] Return `DocumentationAnalysis` with gaps
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-010: Implement architecture gap detector
**Description:** As a user, I want the audit to identify architecture gaps so I know what improvements are needed.

**Acceptance Criteria:**
- [ ] Create `src/audit/detectors/mod.rs` and `architecture_gaps.rs`
- [ ] Detect missing abstraction layers
- [ ] Detect inconsistent module boundaries
- [ ] Detect layer violations (e.g., UI calling DB directly)
- [ ] Return list of `AuditFinding` with recommendations
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-011: Implement opportunity detector
**Description:** As a user, I want the audit to suggest complementary features so I know what could be added.

**Acceptance Criteria:**
- [ ] Create `src/audit/detectors/opportunities.rs`
- [ ] Define opportunity patterns (e.g., "has quality gates but no audit command")
- [ ] Match patterns against audit results
- [ ] Return list of `FeatureOpportunity` with suggested stories
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-012: Implement technical debt detector
**Description:** As a user, I want the audit to find technical debt so I can prioritize cleanup.

**Acceptance Criteria:**
- [ ] Create `src/audit/detectors/tech_debt.rs`
- [ ] Detect TODO/FIXME/HACK comments
- [ ] Detect outdated dependencies
- [ ] Detect dead code indicators
- [ ] Return list of `AuditFinding` for tech debt items
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-013: Implement JSON report output
**Description:** As a user, I want audit results in JSON format so tools can consume them programmatically.

**Acceptance Criteria:**
- [ ] Create `src/audit/output/mod.rs` and `structured.rs`
- [ ] Serialize `AuditReport` to JSON
- [ ] Include all sections: metadata, inventory, dependencies, findings, opportunities
- [ ] Write to `audit.json` file
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-014: Implement markdown report output
**Description:** As a user, I want audit results in markdown format so I can read them easily.

**Acceptance Criteria:**
- [ ] Create `src/audit/output/markdown.rs`
- [ ] Generate executive summary with key metrics
- [ ] Format findings with severity badges
- [ ] Format opportunities with complexity indicators
- [ ] Include table of contents
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-015: Implement agent context output
**Description:** As a developer, I want audit patterns written to progress.txt so agents can follow existing conventions.

**Acceptance Criteria:**
- [ ] Create `src/audit/output/agent_context.rs`
- [ ] Generate "Codebase Patterns" section for progress.txt
- [ ] Include naming conventions, architecture pattern, key conventions
- [ ] Append to existing progress.txt (don't overwrite)
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-016: Add audit CLI subcommand
**Description:** As a user, I want to run `ralph audit` from the command line so I can audit any codebase.

**Acceptance Criteria:**
- [ ] Add `Audit` variant to `Commands` enum in `src/main.rs`
- [ ] Support `-d, --dir` for target directory
- [ ] Support `-f, --format` for output format (json, markdown, context, all)
- [ ] Support `-o, --output` for output file
- [ ] Support `--profile` for quality profile
- [ ] Support `--smart` for smart Q&A mode
- [ ] Support `--no-interactive` to skip Q&A
- [ ] Support `--generate-prd` to auto-generate PRD
- [ ] Print help text with `-h, --help`
- [ ] Typecheck passes

### US-017: Implement interactive Q&A flow
**Description:** As a user, I want the audit to ask me clarifying questions so it better understands my goals.

**Acceptance Criteria:**
- [ ] Create `src/audit/interactive.rs` with Q&A logic
- [ ] Ask 3-5 questions with A/B/C/D options
- [ ] Questions include: project purpose, priorities, target users
- [ ] Parse user responses (e.g., "1A, 2C, 3B")
- [ ] Refine findings based on answers
- [ ] Skip questions in `--no-interactive` mode
- [ ] In `--smart` mode, only ask when confidence < threshold
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-018: Implement audit-to-PRD generation
**Description:** As a user, I want to generate a PRD from audit findings so I can start improving the codebase.

**Acceptance Criteria:**
- [ ] Create `src/audit/prd_generator.rs`
- [ ] Convert findings to user stories
- [ ] Convert opportunities to user stories
- [ ] Generate PRD markdown following existing /prd skill format
- [ ] Save to `tasks/prd-<project>-improvements.md`
- [ ] Prompt user before generating (unless `--generate-prd` flag)
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-019: Implement PRD-to-prd.json conversion
**Description:** As a user, I want the audit to convert the generated PRD to prd.json so I can run ralph immediately.

**Acceptance Criteria:**
- [ ] Create `src/audit/prd_converter.rs`
- [ ] Parse generated PRD markdown
- [ ] Extract user stories and acceptance criteria
- [ ] Generate valid prd.json with project, branchName, userStories
- [ ] Set all stories to `passes: false`
- [ ] Save to `prd.json` in working directory
- [ ] Prompt user before converting
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-020: Create /audit skill for Claude Code
**Description:** As a user, I want to type `/audit` in Claude Code so I can run an interactive audit.

**Acceptance Criteria:**
- [ ] Create `skills/audit/SKILL.md` with full skill instructions
- [ ] Define interactive Q&A flow matching CLI behavior
- [ ] Include instructions for presenting findings
- [ ] Include instructions for generating PRD
- [ ] Include instructions for converting to prd.json
- [ ] Skill follows existing /prd and /ralph skill patterns

### US-021: Implement start_audit MCP tool
**Description:** As an AI agent, I want to start an audit via MCP so I can analyze codebases programmatically.

**Acceptance Criteria:**
- [ ] Create `src/mcp/tools/audit.rs` with `start_audit` tool
- [ ] Accept `path` parameter (optional, defaults to PRD directory)
- [ ] Accept `sections` parameter to limit analysis scope
- [ ] Accept `format` parameter for output format
- [ ] Return audit ID for status checking
- [ ] Register tool in `src/mcp/server.rs`
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-022: Implement get_audit_status MCP tool
**Description:** As an AI agent, I want to check audit progress so I know when it's complete.

**Acceptance Criteria:**
- [ ] Add `get_audit_status` tool to `src/mcp/tools/audit.rs`
- [ ] Accept audit ID parameter
- [ ] Return status: pending, running, completed, failed
- [ ] Return progress percentage if running
- [ ] Register tool in `src/mcp/server.rs`
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-023: Implement get_audit_results MCP tool
**Description:** As an AI agent, I want to retrieve audit results so I can present findings to the user.

**Acceptance Criteria:**
- [ ] Add `get_audit_results` tool to `src/mcp/tools/audit.rs`
- [ ] Accept audit ID parameter
- [ ] Return full `AuditReport` as JSON
- [ ] Return error if audit not complete
- [ ] Register tool in `src/mcp/server.rs`
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-024: Implement generate_prd_from_audit MCP tool
**Description:** As an AI agent, I want to generate a PRD from audit results so users can start improving their codebase.

**Acceptance Criteria:**
- [ ] Add `generate_prd_from_audit` tool to `src/mcp/tools/audit.rs`
- [ ] Accept audit ID parameter
- [ ] Accept optional user answers from Q&A
- [ ] Generate PRD markdown and prd.json
- [ ] Return paths to generated files
- [ ] Register tool in `src/mcp/server.rs`
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-025: Add audit settings to quality profiles
**Description:** As a user, I want to configure audit behavior via quality profiles so I can customize thresholds.

**Acceptance Criteria:**
- [ ] Add `[profiles.*.audit]` section to `quality/ralph-quality.toml`
- [ ] Add `enabled` boolean
- [ ] Add `max_critical_findings`, `max_high_findings` thresholds
- [ ] Add section toggles (structure, patterns, api, deps, tests, docs, arch)
- [ ] Load audit config in quality profile loader
- [ ] Unit tests pass
- [ ] Typecheck passes

### US-026: Integration test on Ralph's own codebase
**Description:** As a developer, I want to run the audit on Ralph itself so I can verify it works end-to-end.

**Acceptance Criteria:**
- [ ] Create integration test that runs full audit on ralph codebase
- [ ] Verify all analysis sections produce results
- [ ] Verify JSON output is valid and parseable
- [ ] Verify markdown output is well-formed
- [ ] Verify findings are reasonable for Ralph codebase
- [ ] Test passes

## Functional Requirements

- FR-1: The system must scan any directory and produce a file inventory with counts by file type
- FR-2: The system must detect the primary programming language(s) in a codebase
- FR-3: The system must parse dependency manifests for Rust (Cargo.toml), TypeScript/JS (package.json), Python (pyproject.toml, requirements.txt), and Go (go.mod)
- FR-4: The system must identify code patterns including naming conventions, module organization, and error handling
- FR-5: The system must detect architecture patterns (layered, modular, hexagonal, clean, etc.)
- FR-6: The system must inventory API endpoints (HTTP, CLI, MCP tools)
- FR-7: The system must analyze test coverage and identify untested areas
- FR-8: The system must detect documentation gaps (missing README, undocumented APIs)
- FR-9: The system must identify architecture gaps and generate findings with recommendations
- FR-10: The system must detect complementary feature opportunities based on what exists
- FR-11: The system must detect technical debt (TODO comments, outdated deps, dead code)
- FR-12: The system must output reports in JSON, markdown, and agent-context formats
- FR-13: The system must save structured audit data to `audit.json`
- FR-14: The system must append patterns to `progress.txt` for agent consumption
- FR-15: The system must provide CLI command `ralph audit` with configurable options
- FR-16: The system must ask interactive clarifying questions (3-5 questions with A/B/C/D options)
- FR-17: The system must support `--smart` mode that only asks questions when confidence is low
- FR-18: The system must generate PRD markdown from audit findings when requested
- FR-19: The system must convert generated PRD to prd.json format for Ralph execution
- FR-20: The system must expose audit functionality via MCP tools for AI agent access
- FR-21: The system must integrate with quality profiles for configurable thresholds

## Non-Goals

- No real-time file watching or continuous auditing
- No automated fixes or code modifications (audit is read-only)
- No support for binary files or compiled artifacts analysis
- No integration with external code quality services (SonarQube, CodeClimate, etc.)
- No support for monorepo-specific analysis (analyzing multiple projects at once)
- No historical comparison (comparing current audit to previous audits)
- No security vulnerability scanning (use dedicated tools like cargo-audit)

## Design Considerations

- CLI output should use Ralph's existing TUI components for consistent styling
- Progress indicators should show during long-running scans
- Interactive Q&A should follow the same A/B/C/D pattern as the /prd skill
- Generated PRDs should be indistinguishable from manually-written PRDs
- Audit should complete in under 30 seconds for typical codebases (<10k files)

## Technical Considerations

- Respect .gitignore patterns when scanning files
- Use parallel scanning for large codebases (rayon or tokio tasks)
- Cache analysis results to avoid re-scanning unchanged directories
- Language analyzers should be pluggable for future expansion
- All audit types should implement a common `Analyzer` trait
- MCP tools should follow existing patterns in `src/mcp/tools/`

## Success Metrics

- Audit completes in under 30 seconds for codebases with <10k files
- At least 90% of detected languages are correctly identified
- At least 80% of findings are actionable (not noise)
- Generated PRDs are valid and can be converted to prd.json successfully
- Users can go from `ralph audit` to `ralph run` in under 2 minutes

## Open Questions

- Should audit results be cached to speed up subsequent audits?
- Should we add a `--watch` mode for continuous auditing during development?
- Should findings include auto-fix suggestions where possible?
- Should we support custom detector rules via configuration?
