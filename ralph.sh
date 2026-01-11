#!/bin/bash
# Ralph Wiggum - Long-running AI agent loop
# Usage: ./ralph.sh [--tool amp|claude|opencode] [max_iterations]
#
# Exit codes:
#   0 - All stories completed successfully
#   1 - Error occurred (invalid arguments, missing dependencies, max iterations reached)
#   2 - Gracefully stopped by user (--stop flag)

set -e

# Help function
show_help() {
  cat << 'EOF'
Ralph - Autonomous AI agent loop for completing PRD user stories

USAGE:
  ralph.sh [OPTIONS] [max_iterations] [-- tool_args...]

OPTIONS:
  --tool <name>          AI tool to use: amp, claude, or opencode (default: amp)
  --custom-prompt <file> Use a custom prompt file instead of the embedded default
  --stop                 Signal Ralph to stop before the next iteration
  --help, -h             Show this help message
  --                     Everything after -- is passed to the tool as additional arguments

ARGUMENTS:
  max_iterations   Maximum iterations to run (default: 10)
  tool_args        Additional arguments to pass to the tool (after --)

FLOW:
  ┌─────────────────┐    ralph skill     ┌─────────────────────┐    ralph.sh     ┌─────────────┐
  │ plans/foo.md    │ ────────────────►  │  prd.json           │ ──────────────► │ Agent Loop  │
  │ (source PRD)    │     converts       │  source: plans/foo  │    reads both   │             │
  └─────────────────┘                    └─────────────────────┘                 └─────────────┘
                                                   │                                    │
                                                   └────────────────────────────────────┘
                                                        agent reads source for context

EXAMPLES:
  ralph.sh                        # Run with amp, 10 iterations
  ralph.sh 5                      # Run with amp, 5 iterations
  ralph.sh --tool claude 20       # Run with Claude Code, 20 iterations
  ralph.sh --tool opencode        # Run with OpenCode, 10 iterations
  ralph.sh --stop                 # Stop Ralph before the next iteration
  ralph.sh --tool claude -- --model opus  # Pass --model opus to claude
  ralph.sh 15 -- --verbose        # Run 15 iterations with --verbose passed to tool
  ralph.sh --custom-prompt my-prompt.md   # Use a custom prompt file

REQUIREMENTS:
  - prd.json must exist in the current directory
  - Use the 'ralph' skill to convert a PRD markdown file to prd.json

CUSTOMIZING THE PROMPT:
  By default, Ralph uses an embedded prompt. To customize:
  1. Copy prompt-template.md from the Ralph repo to your project
  2. Modify it for your needs
  3. Run with: ralph.sh --custom-prompt your-prompt.md

EXIT CODES:
  0 - All stories completed successfully
  1 - Error occurred (invalid arguments, missing dependencies, max iterations reached)
  2 - Gracefully stopped by user (--stop flag)

EOF
  exit 0
}

# Parse arguments
TOOL="amp"  # Default to amp for backwards compatibility
MAX_ITERATIONS=10
TOOL_ARGS=()  # Additional args to pass to the tool
CUSTOM_PROMPT=""  # Optional custom prompt file

while [[ $# -gt 0 ]]; do
  case $1 in
    --help|-h)
      show_help
      ;;
    --stop)
      touch "./.ralph-stop"
      echo "Stop signal sent. Ralph will stop before the next iteration."
      exit 0
      ;;
    --tool)
      TOOL="$2"
      shift 2
      ;;
    --tool=*)
      TOOL="${1#*=}"
      shift
      ;;
    --custom-prompt)
      CUSTOM_PROMPT="$2"
      shift 2
      ;;
    --custom-prompt=*)
      CUSTOM_PROMPT="${1#*=}"
      shift
      ;;
    --)
      # Everything after -- is passed to the tool
      shift
      TOOL_ARGS=("$@")
      break
      ;;
    *)
      # Assume it's max_iterations if it's a number; otherwise, it's an error
      if [[ "$1" =~ ^[0-9]+$ ]]; then
        MAX_ITERATIONS="$1"
        shift
      else
        echo "Error: Unrecognized argument '$1'. See --help for usage." >&2
        exit 1
      fi
      ;;
  esac
done

# Validate tool choice
if [[ "$TOOL" != "amp" && "$TOOL" != "claude" && "$TOOL" != "opencode" ]]; then
  echo "Error: Invalid tool '$TOOL'. Must be 'amp', 'claude', or 'opencode'."
  exit 1
fi

# Check if the selected tool exists (using type to detect aliases, functions, and executables)
TOOL_CMD="$TOOL"  # Default to the tool name
if ! type "$TOOL" &> /dev/null; then
  # Special case for claude: check for local installation
  if [[ "$TOOL" == "claude" ]] && [[ -f "$HOME/.claude/local/claude" ]]; then
    echo "Using local Claude installation: ~/.claude/local/claude"
    TOOL_CMD="$HOME/.claude/local/claude"
  else
    echo "Error: Tool '$TOOL' is not available."
    echo "Please install or configure '$TOOL' before running Ralph."
    exit 1
  fi
fi

# Validate custom prompt file if provided
if [[ -n "$CUSTOM_PROMPT" ]] && [[ ! -f "$CUSTOM_PROMPT" ]]; then
  echo "Error: Custom prompt file not found: $CUSTOM_PROMPT"
  exit 1
fi
# All paths relative to current working directory (project root)
PRD_FILE="./prd.json"
PROGRESS_FILE="./progress.txt"
ARCHIVE_DIR="./archive"
LAST_BRANCH_FILE="./.last-branch"
STOP_FILE="./.ralph-stop"

# Validate prd.json exists in current directory
if [ ! -f "$PRD_FILE" ]; then
  echo "Error: prd.json not found in current directory."
  echo "Ralph must be run from a project root containing prd.json"
  echo ""
  echo "Usage: Run from a directory with prd.json, e.g.:"
  echo "  cd /path/to/your/project"
  echo "  /path/to/ralph.sh [--tool amp|claude|opencode] [max_iterations]"
  exit 1
fi

# Check if jq is available
if ! command -v jq &> /dev/null; then
  echo "Error: jq is required but not installed."
  echo "Please install jq: https://jqlang.github.io/jq/download/"
  exit 1
fi

# Check if sponge is available
if ! command -v sponge &> /dev/null; then
  echo "Error: sponge (from moreutils) is required but not installed."
  echo "Install with: brew install moreutils (macOS) or apt-get install moreutils (Ubuntu)"
  exit 1
fi

# Helper function to initialize progress file header
init_progress_header() {
  local prd_file="$1"
  local tool="$2"
  local tool_args="$3"

  echo "# Ralph Progress Log"
  echo "Started: $(date)"
  echo "Tool: $tool"

  # Add tool args if present
  if [[ -n "$tool_args" ]]; then
    echo "Tool args: $tool_args"
  fi

  # Extract and add source from prd.json if present
  if [ -f "$prd_file" ]; then
    local source=$(jq -r '.source // empty' "$prd_file" 2>/dev/null || echo "")
    if [[ -n "$source" ]]; then
      echo "Source PRD: $source"
    fi
  fi

  echo "---"
}

# Archive previous run if branch changed
if [ -f "$PRD_FILE" ] && [ -f "$LAST_BRANCH_FILE" ]; then
  CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
  LAST_BRANCH=$(cat "$LAST_BRANCH_FILE" 2>/dev/null || echo "")
  
  if [ -n "$CURRENT_BRANCH" ] && [ -n "$LAST_BRANCH" ] && [ "$CURRENT_BRANCH" != "$LAST_BRANCH" ]; then
    # Archive the previous run
    DATE=$(date +%Y-%m-%d)
    # Strip "ralph/" prefix from branch name for folder
    FOLDER_NAME=$(echo "$LAST_BRANCH" | sed 's|^ralph/||')
    ARCHIVE_FOLDER="$ARCHIVE_DIR/$DATE-$FOLDER_NAME"
    
    echo "Archiving previous run: $LAST_BRANCH"
    mkdir -p "$ARCHIVE_FOLDER"
    [ -f "$PRD_FILE" ] && cp "$PRD_FILE" "$ARCHIVE_FOLDER/"
    [ -f "$PROGRESS_FILE" ] && cp "$PROGRESS_FILE" "$ARCHIVE_FOLDER/"
    echo "   Archived to: $ARCHIVE_FOLDER"

    # Reset progress file for new run
    init_progress_header "$PRD_FILE" "$TOOL" "${TOOL_ARGS[*]}" > "$PROGRESS_FILE"
  fi
fi

# Track current branch
if [ -f "$PRD_FILE" ]; then
  CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
  if [ -n "$CURRENT_BRANCH" ]; then
    echo "$CURRENT_BRANCH" > "$LAST_BRANCH_FILE"
  fi
fi

# Initialize progress file if it doesn't exist
if [ ! -f "$PROGRESS_FILE" ]; then
  init_progress_header "$PRD_FILE" "$TOOL" "${TOOL_ARGS[*]}" > "$PROGRESS_FILE"
fi

# Generate the prompt - conditionally include AMP thread URL section
generate_prompt() {
  local tool="$1"
  
  # Base prompt content
  cat << 'PROMPT_START'
# Ralph Agent Instructions

You are an autonomous coding agent working on a software project.

## Your Task

1. Get the next story to work on using: `jq '[.userStories[] | select(.passes == false)] | min_by(.priority)' prd.json`
2. If `prd.json` has a `source` field, you may read that file for full context on the feature requirements **only if**:
   - The `source` value is a relative path within this project (no leading `/`, no `..` segments), and
   - It does not point outside the repository or to system files. If it is absolute, contains `..`, or looks suspicious, ignore it and do not attempt to read it.
3. Read the progress log at `progress.txt` (check Codebase Patterns section first)
4. Check you're on the correct branch from PRD `branchName`. If not, check it out or create from main.
5. Work on the user story from step 1
6. Run quality checks (e.g., typecheck, lint, test - use whatever your project requires)
7. Update AGENTS.md files if you discover reusable patterns (see below)
8. If checks pass, commit ALL changes with message: `feat: [Story ID] - [Story Title]`
9. Update the PRD to set `passes: true` for the completed story using: `jq '(.userStories[] | select(.id == "STORY-ID") | .passes) = true' prd.json | sponge prd.json`
10. Append your progress to `progress.txt`

## Progress Report Format

APPEND to progress.txt (never replace, always append):
```
## [Date/Time] - [Story ID]
PROMPT_START

  # Conditionally include AMP thread URL line
  if [[ "$tool" == "amp" ]]; then
    echo 'Thread: https://ampcode.com/threads/$AMP_CURRENT_THREAD_ID'
  fi

  cat << 'PROMPT_END'
- What was implemented
- Files changed
- **Learnings for future iterations:**
  - Patterns discovered (e.g., "this codebase uses X for Y")
  - Gotchas encountered (e.g., "don't forget to update Z when changing W")
  - Useful context (e.g., "the evaluation panel is in component X")
---
```

Include the thread URL so future iterations can use the `read_thread` tool to reference previous work if needed.

The learnings section is critical - it helps future iterations avoid repeating mistakes and understand the codebase better.

## Consolidate Patterns

If you discover a **reusable pattern** that future iterations should know, add it to the `## Codebase Patterns` section at the TOP of progress.txt (create it if it doesn't exist). This section should consolidate the most important learnings:

```
## Codebase Patterns
- Example: Use `sql<number>` template for aggregations
- Example: Always use `IF NOT EXISTS` for migrations
- Example: Export types from actions.ts for UI components
```

Only add patterns that are **general and reusable**, not story-specific details.

## Update AGENTS.md Files

Before committing, check if any edited files have learnings worth preserving in nearby AGENTS.md files:

1. **Identify directories with edited files** - Look at which directories you modified
2. **Check for existing AGENTS.md** - Look for AGENTS.md in those directories or parent directories
3. **Add valuable learnings** - If you discovered something future developers/agents should know:
   - API patterns or conventions specific to that module
   - Gotchas or non-obvious requirements
   - Dependencies between files
   - Testing approaches for that area
   - Configuration or environment requirements

**Examples of good AGENTS.md additions:**
- "When modifying X, also update Y to keep them in sync"
- "This module uses pattern Z for all API calls"
- "Tests require the dev server running on PORT 3000"
- "Field names must match the template exactly"

**Do NOT add:**
- Story-specific implementation details
- Temporary debugging notes
- Information already in progress.txt

Only update AGENTS.md if you have **genuinely reusable knowledge** that would help future work in that directory.

## Quality Requirements

- ALL commits must pass your project's quality checks (typecheck, lint, test)
- Do NOT commit broken code
- Keep changes focused and minimal
- Follow existing code patterns

## Browser Testing (Required for Frontend Stories)

For any story that changes UI, you MUST verify it works in the browser:

1. Load the `dev-browser` skill
2. Navigate to the relevant page
3. Verify the UI changes work as expected
4. Take a screenshot if helpful for the progress log

A frontend story is NOT complete until browser verification passes.

## Stop Condition

After completing a user story, check if ALL stories have `passes: true`.

If ALL stories are complete and passing, reply with:
<promise>COMPLETE</promise>

If there are still stories with `passes: false`, end your response normally (another iteration will pick up the next story).

## Important

- Work on ONE story per iteration
- Commit frequently
- Keep CI green
- Read the Codebase Patterns section in progress.txt before starting
PROMPT_END
}

echo "Starting Ralph - Tool: $TOOL - Max iterations: $MAX_ITERATIONS"

for i in $(seq 1 $MAX_ITERATIONS); do
  # Check for stop signal
  if [ -f "$STOP_FILE" ]; then
    echo ""
    echo "Stop signal detected. Stopping gracefully..."
    rm -f "$STOP_FILE"
    exit 2
  fi

  echo ""
  echo "==============================================================="
  echo "  Ralph Iteration $i of $MAX_ITERATIONS ($TOOL)"
  echo "==============================================================="

  # Generate the prompt - use custom prompt file if provided, otherwise generate
  if [[ -n "$CUSTOM_PROMPT" ]]; then
    PROMPT=$(cat "$CUSTOM_PROMPT")
  else
    PROMPT=$(generate_prompt "$TOOL")
  fi

  # Run the selected tool with the ralph prompt
  if [[ "$TOOL" == "amp" ]]; then
    OUTPUT=$(echo "$PROMPT" | "$TOOL_CMD" --dangerously-allow-all "${TOOL_ARGS[@]}" 2>&1 | tee /dev/stderr) || true
  elif [[ "$TOOL" == "claude" ]]; then
    # Claude Code: use --dangerously-skip-permissions for autonomous operation, --print for output
    OUTPUT=$(echo "$PROMPT" | "$TOOL_CMD" --dangerously-skip-permissions --print "${TOOL_ARGS[@]}" 2>&1 | tee /dev/stderr) || true
  else
    # OpenCode: use run command for non-interactive mode
    if ((${#TOOL_ARGS[@]})); then
      OUTPUT=$("$TOOL_CMD" run "$PROMPT" "${TOOL_ARGS[@]}" 2>&1 | tee /dev/stderr) || true
    else
      OUTPUT=$("$TOOL_CMD" run "$PROMPT" 2>&1 | tee /dev/stderr) || true
    fi
  fi
  
  # Check for completion signal
  if echo "$OUTPUT" | grep -q "<promise>COMPLETE</promise>"; then
    echo ""
    echo "Ralph completed all tasks!"
    echo "Completed at iteration $i of $MAX_ITERATIONS"
    exit 0
  fi
  
  echo "Iteration $i complete. Continuing..."
  sleep 2
done

echo ""
echo "Ralph reached max iterations ($MAX_ITERATIONS) without completing all tasks."
echo "Check $PROGRESS_FILE for status."
exit 1
