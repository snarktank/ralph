# PRD: Make Ralph Very Easy to Start, Inspect, and Recover

**Suggested Branch Name:** `ralph/easier-to-use`

## 1. Introduction / Overview

Ralph already has the right core loop: draft a PRD, convert it into `prd.json`, and let `./ralph.sh` finish one story per fresh agent iteration. The repo also already has a stronger baseline than before: `README.md` leads with a Codex-first workflow and `./ralph.sh --help` explains the happy path.

The remaining friction is practical, not architectural. A first-time user still has to guess whether their local setup is healthy, how to inspect progress without `jq`, what exact file or command to use when state is broken, and where to find one known-good end-to-end example. This project should make Ralph feel self-guiding: easy to start, easy to diagnose, and easy to resume.

**Primary users**
- Developers who use Codex and want to try Ralph for the first time
- Developers copying Ralph into another repository and needing a safe, obvious setup path
- Existing Ralph users who need to inspect or recover state between runs

## 2. Goals

- Reduce the time from opening the repo to a confident first `./ralph.sh` run
- Make setup and state problems diagnosable without reading the shell script
- Make progress and next steps visible without manual `jq` commands
- Make Ralph easy to invoke after a one-time setup instead of requiring users to remember the repo-local script path
- Provide one concrete, copyable example of the full Ralph workflow
- Keep the Codex-first path obvious while preserving Claude as an optional compatibility path

## 3. User Stories

### US-001: Add a Doctor Command
**Description:** As a first-time Ralph user, I want a preflight command so that I can verify my environment and state before starting a run.

**Acceptance Criteria:**
- [ ] `./ralph.sh --doctor` checks required binaries, prompt files, repo-local skills, and Ralph state files
- [ ] Doctor output labels each check as pass or fail and includes one concrete next action for each failure
- [ ] Doctor exits with status `0` only when all required checks pass
- [ ] Doctor is non-destructive and does not mutate Ralph state
- [ ] Typecheck passes

### US-002: Add a Status Command
**Description:** As an active Ralph user, I want a status command so that I can see progress and the next pending story without reading raw JSON.

**Acceptance Criteria:**
- [ ] `./ralph.sh --status` reports current `branchName`, total story count, passed story count, pending story count, and the next pending story ID and title
- [ ] Status output includes the exact paths to `prd.json`, `progress.txt`, and `archive/`
- [ ] If `prd.json` is missing or invalid, status explains the exact next command or file the user should use
- [ ] Status is non-destructive and does not start an iteration
- [ ] Typecheck passes

### US-003: Tighten Error Recovery Guidance
**Description:** As a new Ralph user, I want every common failure to tell me the next step so that I do not need to inspect source code to recover.

**Acceptance Criteria:**
- [ ] Invalid flags explicitly point to `./ralph.sh --help`
- [ ] Missing tool binaries explain which binary is missing and which command path the user should take next
- [ ] Missing or invalid `prd.json` explains the Codex-first sequence `\$prd` -> `\$ralph` -> `./ralph.sh`
- [ ] Errors caused by missing prompt or skill files explain which repo files must exist and where they should live
- [ ] Typecheck passes

### US-004: Add a Global Ralph Command
**Description:** As a developer using Ralph across projects, I want a simple one-time install path so that I can run `ralph` without remembering the repo-local `./ralph.sh` path every time.

**Acceptance Criteria:**
- [ ] The repo provides a documented install path that exposes `ralph` on `PATH` after one-time setup
- [ ] The installed `ralph` command runs the same loop as `./ralph.sh` with Codex as the default runtime
- [ ] The global command supports the same flags as `./ralph.sh`, or any intentional differences are explicitly documented
- [ ] Documentation explains when to use `ralph` versus `./ralph.sh` and how to undo the install
- [ ] Typecheck passes

### US-005: Add a Copy-Paste Quickstart
**Description:** As a developer trying Ralph for the first time, I want a short copy-paste quickstart so that I can reach a first run with minimal interpretation.

**Acceptance Criteria:**
- [ ] `README.md` contains a short quickstart section with the smallest successful Codex-first flow
- [ ] The quickstart includes `\$prd`, `\$ralph`, `./ralph.sh`, and at least one progress inspection command that does not require `jq` knowledge
- [ ] The quickstart links to or references `--help`, `--doctor`, and `--status` if they exist
- [ ] Claude guidance remains present but clearly secondary to the Codex path
- [ ] Typecheck passes

### US-006: Ship a Worked Example
**Description:** As a new Ralph user, I want one realistic example I can copy so that I can follow a known-good workflow instead of inventing my own.

**Acceptance Criteria:**
- [ ] The repo includes one small worked example that starts with a feature idea and ends with a valid `prd.json`
- [ ] The example uses current repo conventions and file paths
- [ ] The example is small enough that its stories are credible one-iteration Ralph tasks
- [ ] The example is referenced from `README.md`
- [ ] Typecheck passes

### US-007: Add Troubleshooting and Recovery Docs
**Description:** As a Ralph user, I want explicit recovery instructions so that I can resume, restart, or debug a broken run without guessing.

**Acceptance Criteria:**
- [ ] Documentation explains how to handle missing `prd.json`, invalid `prd.json`, empty `progress.txt`, branch changes, and archived runs
- [ ] Documentation distinguishes continuing the current run from starting a new run
- [ ] Each recovery path includes exact commands or file locations, not just conceptual guidance
- [ ] Recovery guidance is reachable from the main README
- [ ] Typecheck passes

### US-008: Align the Flowchart With the Real Workflow
**Description:** As a user learning Ralph visually, I want the flowchart content to match the repo so that the diagram reinforces the actual process.

**Acceptance Criteria:**
- [ ] The flowchart uses the current Codex-first sequence `\$prd` -> `\$ralph` -> `./ralph.sh`
- [ ] The flowchart names the real state files `prd.json`, `progress.txt`, and `archive/`
- [ ] The flowchart reflects the one-story-per-iteration behavior described in `CODEX.md`
- [ ] `cd flowchart && npm run build` passes
- [ ] Verify in browser using dev-browser skill

### US-009: Make the Flowchart Self-Guided
**Description:** As a first-time visitor, I want the flowchart to explain where to start and what to click next so that I can learn Ralph without a presenter.

**Acceptance Criteria:**
- [ ] The flowchart has a clear entry state for first-time viewers
- [ ] The flowchart gives visible progression cues as steps are revealed
- [ ] The main interaction remains usable on desktop and mobile widths
- [ ] `cd flowchart && npm run build` passes
- [ ] Verify in browser using dev-browser skill

## 4. Functional Requirements

**FR-001.** Ralph must provide a non-destructive `--doctor` mode that checks local prerequisites and Ralph state before a run starts.

**FR-002.** Ralph must provide a non-destructive `--status` mode that summarizes branch, story progress, next pending work, and key state file paths.

**FR-003.** Common CLI failures must explain the exact next step the user should take instead of stopping at a generic error.

**FR-004.** Ralph must provide a one-time install path that lets users invoke `ralph` globally without duplicating the loop implementation.

**FR-005.** The primary onboarding path in `README.md` must remain Codex-first and minimize interpretation.

**FR-006.** The repo must include at least one worked example that demonstrates the expected flow from idea to markdown PRD to `prd.json` to `./ralph.sh`.

**FR-007.** The repo must provide recovery guidance for broken or missing Ralph state and for archived or resumed runs.

**FR-008.** The flowchart must reflect the real workflow and file names used in this repository.

**FR-009.** The flowchart must support self-guided onboarding, not only presentation mode.

**FR-010.** New onboarding and inspection features must preserve Ralph's current shell-based architecture and one-fresh-agent-per-iteration model.

**FR-011.** Non-destructive CLI modes must parse and exit before any behavior that would require a healthy `prd.json` or start the agent loop.

## 5. Non-Goals

- Replacing `ralph.sh` with a GUI or hosted service
- Changing the core Ralph architecture of one fresh agent per iteration
- Removing Claude support entirely
- Adding telemetry, authentication, or any remote onboarding backend
- Expanding Ralph into a generic project management tool

## 6. Design Considerations

- Prioritize progressive disclosure: quickstart first, troubleshooting second, internals last
- Prefer copy-paste command blocks and concrete file paths over conceptual prose
- Keep naming consistent across CLI, README, PRD examples, and flowchart
- Prefer a thin global wrapper or install shim over a second source of truth for Ralph loop logic
- Optimize for "easy to recover" as much as "easy to start"

## 7. Technical Considerations

- Keep CLI improvements dependency-light and compatible with the existing bash entry point
- Any global `ralph` command should remain a thin entrypoint to repo-local `ralph.sh` so behavior and flags stay aligned
- Reuse the existing help and README improvements instead of re-implementing them in a different form
- Ensure non-destructive modes work even when Ralph state is missing or partially broken
- Keep documentation, flowchart text, and `ralph.sh` behavior aligned so they do not drift

## 8. Success Metrics

- A Codex user who has never used Ralph can get from opening the repo to a confident first `./ralph.sh` run in 5 minutes or less using the repo alone
- A first-time user can identify the next command to run after a setup failure without reading `ralph.sh`
- An active Ralph user can answer "what is next?" with `./ralph.sh --status` instead of manual `jq`
- A user can complete a one-time setup and then invoke `ralph` without remembering the repo-local script path
- The repo contains one worked example that mirrors the recommended flow and is easy to copy
- The flowchart is understandable without a live presenter

## 9. Open Questions

- Should the global install path use a symlink, a small installer script, or another packaging mechanism?
- Should the worked example live in `tasks/`, a dedicated `examples/` directory, or both?
- Should troubleshooting stay inside `README.md` or move into a dedicated `docs/troubleshooting.md` page linked from the README?
- Should `--status` also expose the most recent progress log entry, or should it stay intentionally minimal?
- Should the self-guided flowchart begin with a static quickstart panel before interactive reveals start?
