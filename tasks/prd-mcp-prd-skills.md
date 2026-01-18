# PRD: MCP PRD Generation Skills

## Introduction

Add `/prd` and `/ralph` skills as MCP tools so Claude Desktop users can generate PRDs and convert them to Ralph's prd.json format directly from the chat interface. This brings feature parity between the terminal CLI experience and the MCP/Claude Desktop experience.

## Goals

- Expose PRD generation capability (`/prd` skill) as an MCP tool
- Expose PRD-to-JSON conversion (`/ralph` skill) as an MCP tool
- Return generated content AND optionally write to user-specified paths
- Support pre-answered clarifying questions for streamlined workflows
- Auto-load converted prd.json for immediate execution

## User Stories

### US-001: Add generate_prd MCP tool schema
**Description:** As a developer, I need to define the MCP tool schema for PRD generation so Claude Desktop can discover and call it.

**Acceptance Criteria:**
- [ ] Add `generate_prd` tool registration in `src/mcp/server.rs`
- [ ] Tool accepts `description` (required), `questions` (optional object), `output_path` (optional)
- [ ] Tool returns `content` (PRD markdown) and `file_path` (if written)
- [ ] Schema includes proper descriptions for Claude to understand usage
- [ ] Typecheck passes

### US-002: Implement generate_prd tool handler
**Description:** As a Claude Desktop user, I want to generate a PRD from a feature description so I can plan features without leaving the chat.

**Acceptance Criteria:**
- [ ] Create `src/mcp/tools/generate_prd.rs` module
- [ ] Load PRD skill template from embedded resource or skills directory
- [ ] Generate PRD markdown following the skill template structure
- [ ] If `questions` parameter provided, use those answers instead of defaults
- [ ] If `output_path` provided, write file and return path
- [ ] Always return generated PRD content in response
- [ ] Typecheck passes

### US-003: Add convert_prd_to_json MCP tool schema
**Description:** As a developer, I need to define the MCP tool schema for PRD-to-JSON conversion so Claude Desktop can discover and call it.

**Acceptance Criteria:**
- [ ] Add `convert_prd_to_json` tool registration in `src/mcp/server.rs`
- [ ] Tool accepts `prd_content` (string) OR `prd_path` (file path)
- [ ] Tool accepts `output_path` (optional, defaults to workspace/prd.json)
- [ ] Tool accepts `auto_load` (boolean, default true)
- [ ] Schema includes proper descriptions for Claude to understand usage
- [ ] Typecheck passes

### US-004: Implement convert_prd_to_json tool handler
**Description:** As a Claude Desktop user, I want to convert a PRD to Ralph's JSON format so I can start autonomous execution.

**Acceptance Criteria:**
- [ ] Create `src/mcp/tools/convert_prd.rs` module
- [ ] Parse PRD markdown and extract user stories, requirements
- [ ] Generate valid prd.json with proper structure (project, branchName, userStories)
- [ ] Auto-generate `branchName` from project name if not specified
- [ ] Set all stories to `passes: false` initially
- [ ] Write to `output_path` if provided
- [ ] If `auto_load` is true, call internal load_prd logic
- [ ] Return JSON content and loaded status in response
- [ ] Typecheck passes

### US-005: Embed skill templates in binary
**Description:** As a developer, I need the skill templates embedded in the binary so the Docker image is self-contained.

**Acceptance Criteria:**
- [ ] Use `include_str!` or similar to embed `skills/prd/SKILL.md` content
- [ ] Use `include_str!` or similar to embed `skills/ralph/SKILL.md` content
- [ ] Templates accessible at runtime without external file dependencies
- [ ] Typecheck passes

### US-006: Add integration tests for new MCP tools
**Description:** As a developer, I need tests to verify the new tools work correctly.

**Acceptance Criteria:**
- [ ] Test `generate_prd` with minimal description returns valid markdown
- [ ] Test `generate_prd` with questions parameter uses provided answers
- [ ] Test `generate_prd` with output_path writes file correctly
- [ ] Test `convert_prd_to_json` parses sample PRD correctly
- [ ] Test `convert_prd_to_json` with auto_load=true loads the PRD
- [ ] Test `convert_prd_to_json` generates valid JSON structure
- [ ] All tests pass

### US-007: Update documentation for new MCP tools
**Description:** As a user, I need documentation on how to use the new MCP tools in Claude Desktop.

**Acceptance Criteria:**
- [ ] Update `docs/guides/docker-mcp-setup.md` with new tool examples
- [ ] Document `generate_prd` parameters and usage
- [ ] Document `convert_prd_to_json` parameters and usage
- [ ] Add example conversation flows
- [ ] All markdown links valid

## Functional Requirements

- FR-1: `generate_prd` tool must accept feature description and return PRD markdown
- FR-2: `generate_prd` must support optional pre-answered questions object
- FR-3: `generate_prd` must optionally write to user-specified path
- FR-4: `convert_prd_to_json` must accept PRD as string content or file path
- FR-5: `convert_prd_to_json` must generate valid Ralph prd.json structure
- FR-6: `convert_prd_to_json` must auto-load PRD when `auto_load=true` (default)
- FR-7: Both tools must work in Docker container with mounted workspace
- FR-8: Skill templates must be embedded in binary (no external file dependencies)

## Non-Goals

- No interactive multi-turn clarifying questions (use `questions` param instead)
- No GUI or web interface for PRD editing
- No PRD versioning or history tracking
- No AI-powered PRD improvement suggestions (just generation/conversion)

## Technical Considerations

- Use `include_str!` macro to embed skill markdown at compile time
- Reuse existing `LoadPrdRequest`/`LoadPrdResponse` types where possible
- PRD parsing can use simple regex/string parsing (no need for full markdown parser)
- JSON serialization via existing serde infrastructure

## Success Metrics

- Users can generate PRD and convert to JSON in 2 tool calls
- Generated prd.json passes validation and can be executed by Ralph
- No regression in existing MCP tool functionality
- Docker image size increase < 100KB from embedded templates

## Open Questions

- Should we support custom skill templates via mounted volume?
- Should `generate_prd` support different PRD formats/templates?
