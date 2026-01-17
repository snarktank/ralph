# PRD: Ralph Enterprise Expansion

## Introduction

Expand Ralph from a basic autonomous AI agent loop into an enterprise-ready development framework. This expansion adds MCP server capabilities (for Claude Desktop integration), project management integration (GitHub Projects, Linear), a configurable quality framework with documentation automation, and blog post generation for major features.

The goal is to make Ralph a comprehensive tool for individuals, teams, and AI/automation engineers who want to orchestrate autonomous software development with full visibility, quality gates, and project tracking.

## Goals

- Enable Ralph to run as an MCP server for Claude Desktop and programmatic API access
- Sync PRD stories bidirectionally with GitHub Projects and Linear
- Provide configurable quality profiles (minimal/standard/comprehensive) for different project needs
- Auto-generate documentation, architecture diagrams, and blog post drafts
- Support local development, self-hosted deployment, with cloud service potential
- Maintain backward compatibility with existing Ralph CLI workflow

## User Stories

### Foundation

#### US-001: Clone and initialize expanded project structure
**Description:** As a developer, I need the expanded project structure set up so I can begin implementing new features.

**Acceptance Criteria:**
- [ ] Repository cloned with all existing functionality intact
- [ ] New directories created: `src/mcp/`, `src/integrations/`, `src/quality/`, `docs/architecture/`, `docs/blog/`, `quality/`, `tests/unit/`, `tests/integration/`
- [ ] Cargo.toml updated with workspace configuration
- [ ] Typecheck passes
- [ ] Existing `ralph` CLI still works

#### US-002: Set up basic CI/CD pipeline
**Description:** As a developer, I want automated checks on every PR so code quality is enforced.

**Acceptance Criteria:**
- [ ] `.github/workflows/ci.yml` runs on push and PR
- [ ] Pipeline includes: format check, clippy lint, unit tests, build
- [ ] Pipeline runs on ubuntu-latest, macos-latest, windows-latest
- [ ] Badge added to README showing CI status
- [ ] Typecheck passes

#### US-003: Create documentation structure with ADR template
**Description:** As a developer, I want a structured place for architecture decisions and documentation.

**Acceptance Criteria:**
- [ ] `docs/architecture/adr/` directory with template.md
- [ ] `docs/architecture/adr/0001-record-architecture-decisions.md` created
- [ ] `docs/architecture/diagrams/` directory created
- [ ] System overview Mermaid diagram created
- [ ] Typecheck passes

---

### Quality Framework

#### US-004: Create quality profiles configuration
**Description:** As a developer, I want to configure how thorough quality checks should be for my project.

**Acceptance Criteria:**
- [ ] `quality/ralph-quality.toml` created with three profiles: minimal, standard, comprehensive
- [ ] Each profile defines: coverage threshold, security checks, test types, blog generation
- [ ] Configuration is parseable and validates correctly
- [ ] Typecheck passes

#### US-005: Implement quality profile parser
**Description:** As a developer, I need the Rust code to load and parse quality profiles.

**Acceptance Criteria:**
- [ ] `src/quality/mod.rs` exports quality module
- [ ] `src/quality/profiles.rs` implements `QualityConfig` struct with serde deserialization
- [ ] Can load config from file and environment variables
- [ ] Unit tests for profile loading
- [ ] Typecheck passes

#### US-006: Implement quality gate checker
**Description:** As a developer, I want to run quality gates based on the active profile.

**Acceptance Criteria:**
- [ ] `src/quality/gates.rs` implements `QualityGateChecker`
- [ ] Gates include: coverage, lint, format, security_audit
- [ ] Each gate returns pass/fail with message
- [ ] Gates respect profile settings (skip disabled gates)
- [ ] Unit tests for gate logic
- [ ] Typecheck passes

#### US-007: Add quality CLI commands
**Description:** As a user, I want CLI commands to run quality checks and view configuration.

**Acceptance Criteria:**
- [ ] `ralph quality check --profile <name>` runs enabled gates
- [ ] `ralph quality check --gates coverage,lint` runs specific gates
- [ ] `ralph quality config` shows current profile settings
- [ ] Exit code reflects pass/fail status
- [ ] Typecheck passes

---

### MCP Server

#### US-008: Add MCP server dependencies
**Description:** As a developer, I need the MCP SDK and related dependencies added to the project.

**Acceptance Criteria:**
- [ ] `rmcp` crate added with server, transport-io, macros features
- [ ] `schemars` crate added for JSON schema generation
- [ ] `tokio` configured with full features
- [ ] Dependencies compile without errors
- [ ] Typecheck passes

#### US-009: Create MCP server module structure
**Description:** As a developer, I need the MCP server module scaffolding in place.

**Acceptance Criteria:**
- [ ] `src/mcp/mod.rs` exports server module
- [ ] `src/mcp/server.rs` defines `RalphMcpServer` struct
- [ ] `src/mcp/tools/mod.rs` created for tool implementations
- [ ] `src/mcp/resources/mod.rs` created for resource implementations
- [ ] Typecheck passes

#### US-010: Implement RalphMcpServer with ServerHandler trait
**Description:** As a developer, I need the core MCP server implementing the rmcp ServerHandler trait.

**Acceptance Criteria:**
- [ ] `RalphMcpServer` implements `ServerHandler` trait
- [ ] `get_info()` returns server name, version, capabilities
- [ ] Server supports tools and resources capabilities
- [ ] Shared state managed with `Arc<RwLock<_>>`
- [ ] Unit tests for server initialization
- [ ] Typecheck passes

#### US-011: Implement list_stories MCP tool
**Description:** As a user, I want to list stories from the loaded PRD via MCP.

**Acceptance Criteria:**
- [ ] `list_stories` tool registered with MCP server
- [ ] Accepts optional `status_filter` parameter (pending/completed/all)
- [ ] Returns JSON array of story summaries
- [ ] Tool has proper JSON schema for parameters
- [ ] Typecheck passes

#### US-012: Implement get_status MCP tool
**Description:** As a user, I want to check Ralph's current execution status via MCP.

**Acceptance Criteria:**
- [ ] `get_status` tool registered with MCP server
- [ ] Returns current state: Idle, Running (with story_id, iteration), Completed, Failed
- [ ] Includes timestamp and progress information
- [ ] Typecheck passes

#### US-013: Implement load_prd MCP tool
**Description:** As a user, I want to load a PRD file into Ralph via MCP.

**Acceptance Criteria:**
- [ ] `load_prd` tool accepts `path` parameter
- [ ] Validates PRD JSON structure
- [ ] Returns success with story count or error message
- [ ] Updates server state with loaded PRD
- [ ] Typecheck passes

#### US-014: Implement run_story MCP tool
**Description:** As a user, I want to execute a specific story via MCP.

**Acceptance Criteria:**
- [ ] `run_story` tool accepts `story_id` and optional `max_iterations`
- [ ] Sends progress notifications during execution
- [ ] Returns execution result with commit info on success
- [ ] Returns error details on failure
- [ ] Prevents concurrent executions (only one story at a time)
- [ ] Typecheck passes

#### US-015: Implement stop_execution MCP tool
**Description:** As a user, I want to cancel a running execution via MCP.

**Acceptance Criteria:**
- [ ] `stop_execution` tool cancels current execution
- [ ] Uses cancellation token pattern for graceful stop
- [ ] Returns confirmation message
- [ ] Handles case when nothing is running
- [ ] Typecheck passes

#### US-016: Implement MCP resources
**Description:** As a user, I want to read Ralph state via MCP resources.

**Acceptance Criteria:**
- [ ] `ralph://prd/current` returns loaded PRD JSON
- [ ] `ralph://status` returns current execution status
- [ ] `ralph://logs/{story_id}` returns logs for specific story
- [ ] Resources have proper MIME types
- [ ] Typecheck passes

#### US-017: Add mcp-server CLI subcommand
**Description:** As a user, I want to start Ralph in MCP server mode.

**Acceptance Criteria:**
- [ ] `ralph mcp-server` starts server on stdio transport
- [ ] `ralph mcp-server --prd <path>` preloads a PRD
- [ ] Logs go to stderr (stdout reserved for MCP protocol)
- [ ] Clean shutdown on SIGTERM/SIGINT
- [ ] Typecheck passes

#### US-018: Create Claude Desktop configuration example
**Description:** As a user, I want documentation for configuring Ralph in Claude Desktop.

**Acceptance Criteria:**
- [ ] `docs/guides/claude-desktop-setup.md` created
- [ ] Example `claude_desktop_config.json` snippet included
- [ ] Environment variable configuration documented
- [ ] Troubleshooting section included
- [ ] Typecheck passes

---

### Project Management Integration

#### US-019: Create ProjectTracker trait
**Description:** As a developer, I need an abstraction layer for project management providers.

**Acceptance Criteria:**
- [ ] `src/integrations/mod.rs` exports integrations module
- [ ] `src/integrations/traits.rs` defines `ProjectTracker` trait
- [ ] Trait includes: create_item, update_item, create_failure_issue, add_comment, update_status
- [ ] `ItemStatus` enum defined (Backlog, Todo, InProgress, Done, Cancelled)
- [ ] Typecheck passes

#### US-020: Create provider registry
**Description:** As a developer, I need to manage multiple project management providers.

**Acceptance Criteria:**
- [ ] `src/integrations/registry.rs` implements `ProviderRegistry`
- [ ] Can register providers by type
- [ ] Can set and get active provider
- [ ] Loads providers from configuration
- [ ] Typecheck passes

#### US-021: Implement GitHub Projects provider
**Description:** As a user, I want Ralph to sync stories to GitHub Projects.

**Acceptance Criteria:**
- [ ] `src/integrations/github.rs` implements `ProjectTracker` for GitHub
- [ ] Creates draft items in GitHub Projects V2
- [ ] Updates item status via GraphQL
- [ ] Creates issues for failed stories
- [ ] Adds comments with progress updates
- [ ] Typecheck passes

#### US-022: Implement Linear provider
**Description:** As a user, I want Ralph to sync stories to Linear.

**Acceptance Criteria:**
- [ ] `src/integrations/linear.rs` implements `ProjectTracker` for Linear
- [ ] Creates issues in Linear with proper state mapping
- [ ] Updates issue status on story completion
- [ ] Creates failure issues with error details
- [ ] Adds comments with progress updates
- [ ] Typecheck passes

#### US-023: Create integration configuration
**Description:** As a user, I want to configure which project management provider to use.

**Acceptance Criteria:**
- [ ] `ralph-integrations.toml` configuration file format defined
- [ ] Supports GitHub and Linear configuration sections
- [ ] Active provider selection
- [ ] Environment variable support for secrets
- [ ] Typecheck passes

#### US-024: Implement sync engine
**Description:** As a developer, I need an engine to orchestrate syncing between Ralph and external systems.

**Acceptance Criteria:**
- [ ] `src/integrations/sync_engine.rs` implements `SyncEngine`
- [ ] `initial_sync()` pushes all stories to external system
- [ ] `sync_story_update()` syncs individual story changes
- [ ] `handle_story_completed()` updates external status to Done
- [ ] `handle_story_failed()` creates failure issue
- [ ] Typecheck passes

#### US-025: Implement webhook server
**Description:** As a developer, I need a webhook server for bidirectional sync.

**Acceptance Criteria:**
- [ ] `src/integrations/webhooks/server.rs` implements Axum webhook server
- [ ] `/webhooks/github` endpoint handles GitHub webhooks
- [ ] `/webhooks/linear` endpoint handles Linear webhooks
- [ ] Signature verification for security
- [ ] Events parsed and forwarded to sync engine
- [ ] Typecheck passes

#### US-026: Add sync CLI commands
**Description:** As a user, I want CLI commands to manage project management sync.

**Acceptance Criteria:**
- [ ] `ralph sync init` performs initial sync to external system
- [ ] `ralph sync status` shows sync state
- [ ] `ralph sync webhook-server` starts webhook server
- [ ] Typecheck passes

---

### Blog Generation

#### US-027: Create blog post templates
**Description:** As a developer, I need templates for auto-generated blog posts.

**Acceptance Criteria:**
- [ ] `docs/blog/templates/feature-release.md` template created
- [ ] `docs/blog/templates/technical-deep-dive.md` template created
- [ ] Templates use placeholder syntax for dynamic content
- [ ] Typecheck passes

#### US-028: Implement blog generator module
**Description:** As a developer, I need code to generate blog posts from context.

**Acceptance Criteria:**
- [ ] `src/quality/blog_generator.rs` implements `BlogGenerator`
- [ ] `BlogContext` struct captures: title, problem, solution, challenges, wins, lessons
- [ ] `generate()` renders template with context
- [ ] `save()` writes to `docs/blog/posts/`
- [ ] Typecheck passes

#### US-029: Integrate blog generation with agent loop
**Description:** As a user, I want blog posts auto-generated when using comprehensive profile.

**Acceptance Criteria:**
- [ ] Agent loop captures implementation context during execution
- [ ] On completion with comprehensive profile, blog draft generated
- [ ] Blog saved to `docs/blog/posts/{date}-{feature}.md`
- [ ] Typecheck passes

---

### Security & Release

#### US-030: Add cargo-deny configuration
**Description:** As a developer, I need dependency security and license checking.

**Acceptance Criteria:**
- [ ] `deny.toml` configuration created
- [ ] Advisories check enabled (deny vulnerabilities)
- [ ] License allowlist defined (MIT, Apache-2.0, BSD, etc.)
- [ ] `cargo deny check` passes
- [ ] Typecheck passes

#### US-031: Add security scanning workflow
**Description:** As a developer, I want automated security scanning in CI.

**Acceptance Criteria:**
- [ ] `.github/workflows/security.yml` created
- [ ] Runs cargo-audit for vulnerability scanning
- [ ] Runs cargo-deny for license compliance
- [ ] Runs on schedule (daily) and on PR
- [ ] Typecheck passes

#### US-032: Add release automation workflow
**Description:** As a developer, I want automated releases with cross-platform binaries.

**Acceptance Criteria:**
- [ ] `.github/workflows/release.yml` created
- [ ] Triggers on version tags (v*)
- [ ] Builds for: linux-x86_64, linux-aarch64, macos-x86_64, macos-aarch64, windows-x86_64
- [ ] Creates GitHub release with binaries
- [ ] Typecheck passes

---

### Integration Testing

#### US-033: Create integration test suite
**Description:** As a developer, I need integration tests for the full workflow.

**Acceptance Criteria:**
- [ ] `tests/integration/` directory with test files
- [ ] CLI tests using `assert_cmd` crate
- [ ] MCP server tests with mock transport
- [ ] Test fixtures in `tests/fixtures/`
- [ ] `cargo test --test integration` runs all integration tests
- [ ] Typecheck passes

#### US-034: Create end-to-end test workflow
**Description:** As a developer, I need an E2E test that validates the complete flow.

**Acceptance Criteria:**
- [ ] Test loads PRD, runs story, verifies completion
- [ ] Test verifies quality gates run based on profile
- [ ] Test verifies sync to mock external system
- [ ] Typecheck passes

## Functional Requirements

- FR-01: MCP server must implement stdio transport for Claude Desktop compatibility
- FR-02: MCP tools must include JSON schemas for all parameters
- FR-03: Progress notifications must be sent during long-running operations
- FR-04: Quality profiles must be selectable via CLI flag or config file
- FR-05: Coverage thresholds must be enforced based on active profile
- FR-06: Project management sync must be bidirectional when webhooks are configured
- FR-07: Blog posts must be generated as local markdown files only
- FR-08: All CLI commands must have `--help` documentation
- FR-09: Configuration must support environment variable substitution for secrets
- FR-10: Logs must go to stderr when running in MCP server mode

## Non-Goals

- No hosted/cloud service in this phase (local and self-hosted only)
- No web UI dashboard (CLI and MCP only)
- No real-time collaboration features
- No automatic blog publishing (local drafts only)
- No payment or billing integration
- No user authentication system (uses external provider tokens)
- No mobile app or native desktop app

## Technical Considerations

- **Rust Edition:** 2021
- **Async Runtime:** Tokio with full features
- **MCP SDK:** rmcp (official Rust SDK)
- **HTTP Client:** reqwest with rustls-tls
- **Web Framework:** Axum for webhook server
- **GitHub API:** octocrab crate
- **Linear API:** Custom GraphQL with reqwest
- **Configuration:** TOML with config crate
- **Testing:** Built-in Rust tests + proptest for property testing
- **Coverage:** cargo-llvm-cov (cross-platform)

## Success Metrics

- MCP server responds to all defined tools within 100ms (excluding execution)
- Quality check with minimal profile completes in under 5 seconds
- Quality check with comprehensive profile completes in under 2 minutes
- Story sync to GitHub/Linear completes within 3 seconds
- Blog post generation completes within 1 second
- All CI checks pass on ubuntu, macos, and windows
- Code coverage meets profile thresholds (70% standard, 90% comprehensive)

## Open Questions

1. Should MCP server support SSE transport in addition to stdio?
2. Should we add Jira as a third project management provider?
3. Should quality profiles be per-directory or global?
4. Should failed webhook deliveries be retried automatically?
