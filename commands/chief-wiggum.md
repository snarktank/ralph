---
name: chief-wiggum
description: "Execute all user stories from prd.json autonomously using ralph-loop"
argument-hint: "[max_stories]"
allowed-tools: ["Bash($CLAUDE_PLUGIN_ROOT/chief-wiggum.sh:*)"]
---

# Chief Wiggum - Autonomous PRD Executor

Execute the Chief Wiggum orchestrator to process user stories:

```!
"$CLAUDE_PLUGIN_ROOT/chief-wiggum.sh" $ARGUMENTS
```

This will read `prd.json` from your current directory and execute each incomplete story using `/ralph-loop`.

## What happens:

1. Reads `prd.json` from current directory
2. Finds highest priority story where `passes: false`
3. Spawns Claude with `/ralph-loop` for that story
4. On `STORY_COMPLETE`: marks story as passed, continues to next
5. On `BLOCKED`: stops and logs the blocker
6. Repeats until all stories complete or blocked

## Prerequisites

- `ralph-loop` plugin must be installed
- `prd.json` must exist in current directory
- Project should have quality checks (typecheck, lint, test)

## Usage

```bash
/chief-wiggum           # Process all stories
/chief-wiggum 5         # Process max 5 stories
```
