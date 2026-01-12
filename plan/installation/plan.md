### Master plan: Installation via a packaged CLI (Option 3)

### Goal
- Make Ralph installable and runnable without copying files by hand.
- Preserve current Amp behavior unchanged; add Cursor as a second worker option.

### User experience (target)
1) Install the CLI once
- npx: `npx @jarekbird/ralph@<version> ...`

2) Initialize a target repo
- From project root: `ralph init`
- This writes a small, repo-local scaffolding (defaults chosen to match today’s README pattern):
  - `scripts/ralph/ralph.sh` (runner)
  - `scripts/ralph/prompt.md` (Amp prompt; unchanged)
  - `scripts/ralph/prompt.cursor.md` (Cursor prompt; new)
  - `scripts/ralph/prd.json.example`
  - (optional) `.cursor/rules/ralph-prd.mdc` (PRD generation rule; Cursor)
  - (optional) `.cursor/cli.json` template (if you want consistent permissions)
- It should NOT overwrite existing files unless `--force` is passed.

3) Run Ralph
- Amp (default): `ralph run` (equivalent to `./scripts/ralph/ralph.sh`)
- Cursor: `ralph run --worker cursor`

### Packaging approach (recommended)
- Implement as an npm package with a `bin` entry:
  - `@jarekbird/ralph`
  - `bin/ralph` → Node script (cross-platform) that:
    - Finds the package’s embedded template files
    - Copies them into the target repo
    - Prints what it wrote / what it skipped
- Keep `scripts/ralph/ralph.sh` as bash (matches current Ralph), but distribute it via the package.

### CLI commands (minimal)
- `ralph init`
  - Detect repo root (cwd)
  - Create `scripts/ralph/` if missing
  - Copy templates into place
  - Optionally add/update `.gitignore` entries if requested by user (do not do automatically unless designed)
- `ralph run [--worker amp|cursor] [--iterations N]`
  - Calls the repo-local runner (`scripts/ralph/ralph.sh`) so behavior stays consistent with today
  - For Cursor worker, either:
    - pass env vars consumed by the runner (recommended), or
    - call a separate runner script (e.g. `ralph-cursor.sh`)

### Versioning & releases
- Use semver and publish to npm.
- Allow `npx @jarekbird/ralph@<version> init` for pinned installs in CI.

### Compatibility notes / constraints
- Ralph still needs repo-local state files (`prd.json`, `progress.txt`) created/managed by the run itself; packaging only removes manual copying of the runner/templates.
- Amp “skills” remain Amp-provided; we keep existing Amp workflow intact.
- Cursor rules/templates are repo-local so each client repo carries the intended behavior.

### Deliverables
- New package:
  - `package.json` with `bin` mapping
  - `templates/` directory inside the package containing the canonical `scripts/ralph/*` files
  - `src/cli.ts` (or similar) implementing `init` + `run`
- Update the Cursor-support plan to reference `scripts/ralph/` as the canonical template location (packaging is separate work).

