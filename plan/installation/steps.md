### Installation via packaged CLI — Execution Steps

This is a checklist-style execution plan derived from `plan.md`, formatted as steps with acceptance criteria (similar spirit to `prd.json`, but in markdown with checkboxes).

### Step 0 — Confirm scope and constraints
- [x] **Acceptance criteria**
  - [x] Goal is to distribute Ralph without manual copying of `scripts/ralph/*`.
  - [x] Amp behavior remains unchanged; Cursor is an optional second worker.
  - [x] Repo-local state (`prd.json`, `progress.txt`) remains repo-owned (packaging only distributes runner/templates).

### Step 1 — Define the intended user experience
- [x] **Acceptance criteria**
  - [x] Installation path is defined:
    - [x] npx pinned usage: `npx @jarekbird/ralph@<version> ...`
  - [x] Repo initialization is defined:
    - [x] `ralph init` creates `scripts/ralph/` and scaffolds templates.
    - [x] `ralph init` does NOT overwrite existing files unless `--force` is provided.
  - [x] Running is defined:
    - [x] `ralph run` defaults to Amp and is equivalent to invoking the repo-local runner.
    - [x] `ralph run --worker cursor` runs the Cursor worker.

### Step 2 — Define scaffolding outputs (templates written into target repo)
- [x] **Acceptance criteria**
  - [x] `ralph init` writes the following into the target repo:
    - [x] `scripts/ralph/ralph.sh` (runner)
    - [x] `scripts/ralph/prompt.md` (Amp prompt)
    - [x] `scripts/ralph/cursor/prompt.cursor.md` (Cursor prompt)
    - [x] `scripts/ralph/cursor/convert-to-prd-json.sh` (Cursor PRD conversion helper)
    - [x] `scripts/ralph/cursor/prompt.convert-to-prd-json.md` (PRD conversion prompt template)
    - [x] `scripts/ralph/prd.json.example` (PRD format example)
  - [x] Optional scaffolding is clearly defined and guarded by flags (or documented as optional):
    - [x] `.cursor/rules/ralph-prd.mdc` (PRD generation rule for Cursor IDE; optional, use `--cursor-rules` flag)
    - [x] `.cursor/cli.json` (permissions template; optional, use `--cursor-cli` flag)

### Step 3 — Implement packaging layout (npm package)
- [x] **Acceptance criteria**
  - [x] Package has a `bin` entry that exposes the `ralph` command.
  - [x] Package contains a `templates/` directory with canonical `scripts/ralph/*` contents to copy from.
  - [x] The `ralph` executable is cross-platform (Node) and only writes into the repo (no global state required beyond installation).

### Step 4 — Implement `ralph init`
- [x] **Acceptance criteria**
  - [x] Detect repo root (use current working directory as root).
  - [x] Create `scripts/ralph/` if missing.
  - [x] Copy template files into place.
  - [x] Print a clear summary: files created vs skipped (and why).
  - [x] Respect `--force` to overwrite existing files.
  - [x] Does not auto-edit `.gitignore` unless explicitly requested (flagged behavior).

### Step 5 — Implement `ralph run`
- [x] **Acceptance criteria**
  - [x] Calls the repo-local runner (`scripts/ralph/ralph.sh`) so behavior matches current Ralph.
  - [x] Supports `--worker amp|cursor` (Amp default).
  - [x] Supports `--iterations N` (maps to runner argument).
  - [x] Cursor worker selection is implemented by passing env vars / args to the runner (or by selecting an alternate runner script), but the behavior is deterministic and documented.

### Step 6 — Automated tests (required)
- [x] **Acceptance criteria**
  - [x] Tests do not require a real Amp install or a real Cursor install (use stub `amp`/`cursor` binaries injected via `PATH`).
  - [x] Tests run in a temporary “fake repo” directory (no reliance on the current repo’s state).
  - [x] **`ralph init` tests**
    - [x] Creates `scripts/ralph/` and writes the expected files.
    - [x] Does not overwrite existing files by default.
    - [x] Overwrites existing files only when `--force` is provided.
  - [x] **`ralph run` tests**
    - [x] Invokes the repo-local runner.
    - [x] Passes `--iterations N` through to the runner.
    - [x] Defaults to Amp worker unless `--worker cursor` is specified.

### Step 7 — Versioning & releases
- [x] **Acceptance criteria**
  - [x] Use semver releases.
  - [x] Publishing plan supports pinned installs via `npx @jarekbird/ralph@<version> init`.

### Step 8 — Compatibility notes are reflected in the implementation
- [x] **Acceptance criteria**
  - [x] Packaging does not assume Amp skills are shipped by this package (Amp skills remain Amp-provided).
  - [x] Cursor rules/templates remain repo-local so each client repo carries intended behavior.

### Step 9 — Deliverables checklist
- [x] **Acceptance criteria**
  - [x] `package.json` includes `bin` mapping for `ralph`.
  - [x] `templates/` exists and contains canonical `scripts/ralph/*` files.
  - [x] `bin/ralph.js` (or equivalent) implements `init` and `run`.
  - [x] Cursor-support plan references `scripts/ralph/` as the canonical template location (already aligned).

