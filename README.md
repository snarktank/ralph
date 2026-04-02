# Ralph

![Ralph](ralph.webp)

Ralph is a small autonomous loop that repeatedly launches a fresh coding agent until every story in `prd.json` passes. This repo is Codex-first and exposes Codex-native repo-local skills for planning and PRD conversion.

Based on [Geoffrey Huntley's Ralph pattern](https://ghuntley.com/ralph/).

[Read my in-depth article on how I use Ralph](https://x.com/ryancarson/status/2008548371712135632)

## Install A Global `ralph` Command

From the Ralph repo root, install the thin wrapper once, then use `ralph` from any repository that contains a repo-local `ralph.sh`:

```bash
mkdir -p "$HOME/.local/bin"
ln -sf "$(pwd)/bin/ralph" "$HOME/.local/bin/ralph"
export PATH="$HOME/.local/bin:$PATH"
```

If `~/.local/bin` is not already on your shell `PATH`, add that export line to `~/.zshrc`, `~/.bashrc`, or the startup file for your shell, then open a new shell.

The installed `ralph` command is only a wrapper. It walks up from your current working directory until it finds `ralph.sh`, then forwards every argument unchanged to that repo-local script. That keeps the global command aligned with `./ralph.sh` instead of creating a second source of truth.

Use `ralph` after the one-time PATH setup when you are already inside a Ralph-enabled repository and want the shortest command. Use `./ralph.sh` when you have not installed the wrapper yet, when you want to be explicit in docs or automation, or when you are debugging the wrapper itself.

Undo the install with:

```bash
rm -f "$HOME/.local/bin/ralph"
```

If you added a dedicated PATH export just for Ralph, remove that line from your shell startup file too.

## Codex Workflow In This Repo

Open this repo in Codex. The repo-local skills live under `.agents/skills`, so Codex can discover them natively.

### 1. Confirm the skills are visible

In Codex, optionally run:

```text
/skills
```

You should see `prd` and `ralph`.

### 2. Draft a PRD with `$prd`

In Codex:

```text
Use $prd to create a PRD for [your feature idea]
```

That skill writes a markdown PRD to `tasks/prd-[feature-name].md`.

### 3. Convert the PRD with `$ralph`

In Codex:

```text
Use $ralph to convert tasks/prd-[feature-name].md into ./prd.json
```

The generated `./prd.json` should contain small, dependency-ordered stories with `passes: false`.

### 4. Run Ralph

```bash
./ralph.sh
./ralph.sh 20
ralph
ralph --status
```

### 5. Check progress

```bash
./ralph.sh --status
cat ./prd.json | jq '.userStories[] | {id, title, passes}'
cat ./progress.txt
git log --oneline -10
```

## What `./ralph.sh` Does

Each iteration starts a fresh Codex run using `CODEX.md`, injects the absolute paths for Ralph state, and asks the agent to finish exactly one story.

That iteration should:

1. Read `branchName` from `./prd.json`
2. Pick the highest-priority story where `passes` is `false`
3. Implement the story
4. Run project checks
5. Commit if checks pass
6. Mark the story as passed
7. Append learnings to `./progress.txt`

When every story has `passes: true`, the agent emits `<promise>COMPLETE</promise>` and the loop exits.

## Global Wrapper Behavior

After the one-time install above, `ralph` supports the same flags and positional arguments as `./ralph.sh` because it forwards everything to the repo-local script unchanged:

```bash
ralph --doctor
ralph --status
ralph --tool claude 5
```

If you run `ralph` outside a repository that contains `ralph.sh`, the wrapper stops immediately and tells you to either change into a Ralph-enabled repo or invoke that repo's `./ralph.sh` directly.

## File Layout

| File | Purpose |
|------|---------|
| `ralph.sh` | Loop runner. Defaults to Codex, optional `--tool claude` |
| `bin/ralph` | Thin global wrapper that discovers the nearest repo-local `ralph.sh` and forwards all args |
| `CODEX.md` | Codex instructions for one Ralph iteration |
| `.agents/skills/prd/` | Codex-native PRD skill, invoked as `$prd` |
| `.agents/skills/ralph/` | Codex-native PRD-to-`prd.json` skill, invoked as `$ralph` |
| `tasks/` | Default output folder for markdown PRDs created by `$prd` |
| `prd.json` | Story list Ralph works through. In this repo it lives at `./prd.json` |
| `progress.txt` | Append-only memory across iterations |
| `archive/` | Archived `prd.json` and `progress.txt` from previous runs |
| `prd.json.example` | Example PRD format |
| `skills/` | Compatibility copies of the skills for manual reuse outside Codex repo-local discovery |
| `flowchart/` | Interactive explanation of the Ralph loop |

## Writing Good Stories

Ralph works only if stories are sized for a single clean-context iteration.

- Good story shape:
  - one focused change
  - explicit acceptance criteria
  - concrete checks like `Typecheck passes` or `Tests pass`
- Bad story shape:
  - multi-area refactors
  - vague UX goals
  - features that require several dependent steps at once

For UI work, include browser verification in the acceptance criteria.

## Memory Model

Ralph does not carry conversational memory between iterations. The handoff happens through:

- git history
- `./prd.json`
- `./progress.txt`
- `AGENTS.md`

For Codex, reusable patterns should be written to `AGENTS.md`. That is the instruction file future Codex iterations will automatically read.

## Customizing `CODEX.md`

If you copy Ralph into another repository, keep `ralph.sh`, `CODEX.md`, and the `.agents/skills/` directories together at the repo root. Then use `$ralph` to create `./prd.json` there and customize `CODEX.md` for that stack:

- add project-specific quality check commands
- include codebase conventions
- add common gotchas for your stack

Keep the prompt focused on one-story iterations. The shell loop is intentionally simple; the prompt carries most of the behavior.

## Optional Claude Code Support

Claude Code still works, but it is not the default path in this repo.

```bash
cp /path/to/ralph/CLAUDE.md ./CLAUDE.md
./ralph.sh --tool claude
```

## Flowchart

[![Ralph Flowchart](ralph-flowchart.png)](https://snarktank.github.io/ralph/)

**[View Interactive Flowchart](https://snarktank.github.io/ralph/)** to click through the loop visually.

The source is in `flowchart/`:

```bash
cd flowchart
npm install
npm run dev
```

## Archiving

Ralph automatically archives previous runs when you start a new feature with a different `branchName`. Archives are saved to `archive/YYYY-MM-DD-feature-name/`.

## References

- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Codex CLI documentation](https://developers.openai.com/codex/cli)
- [Codex Skills documentation](https://developers.openai.com/codex/skills)
- [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code)
