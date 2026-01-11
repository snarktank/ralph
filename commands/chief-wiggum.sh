#!/bin/bash
# Chief Wiggum - Autonomous PRD executor for Claude Code
# Two-tier architecture: Chief Wiggum (outer loop) + /ralph-loop:ralph-loop (inner loop per story)
# Usage: ./commands/chief-wiggum.sh [max_stories] or via /chief-wiggum command

set -e

# Script configuration - resolve to plugin root (parent of commands/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Files in current working directory (user's project)
PRD_FILE="$(pwd)/prd.json"
PROGRESS_FILE="$(pwd)/progress.txt"
ARCHIVE_DIR="$(pwd)/archive"
LAST_BRANCH_FILE="$(pwd)/.chief-wiggum-last-branch"

# Files in plugin directory
CONFIG_FILE="$PLUGIN_DIR/chief-wiggum.config.json"
TEMPLATE_FILE="$PLUGIN_DIR/story-prompt.template.md"

# Load configuration
if [ ! -f "$CONFIG_FILE" ]; then
  echo "Error: Configuration file not found: $CONFIG_FILE"
  exit 1
fi

MAX_ITERATIONS_PER_STORY=$(jq -r '.maxIterationsPerStory // 25' "$CONFIG_FILE")
COMPLETION_PROMISE=$(jq -r '.completionPromise // "STORY_COMPLETE"' "$CONFIG_FILE")
BLOCKED_PROMISE=$(jq -r '.blockedPromise // "BLOCKED"' "$CONFIG_FILE")

# Command line args override config
MAX_STORIES=${1:-100}

# Check for required files
if [ ! -f "$PRD_FILE" ]; then
  echo "Error: PRD file not found: $PRD_FILE"
  echo "Create a prd.json file with your user stories first."
  exit 1
fi

if [ ! -f "$TEMPLATE_FILE" ]; then
  echo "Error: Template file not found: $TEMPLATE_FILE"
  exit 1
fi

# Archive previous run if branch changed
if [ -f "$PRD_FILE" ] && [ -f "$LAST_BRANCH_FILE" ]; then
  CURRENT_BRANCH=$(jq -r '.branchName // empty' "$PRD_FILE" 2>/dev/null || echo "")
  LAST_BRANCH=$(cat "$LAST_BRANCH_FILE" 2>/dev/null || echo "")

  if [ -n "$CURRENT_BRANCH" ] && [ -n "$LAST_BRANCH" ] && [ "$CURRENT_BRANCH" != "$LAST_BRANCH" ]; then
    # Archive the previous run
    DATE=$(date +%Y-%m-%d)
    # Strip "chief-wiggum/" prefix from branch name for folder
    FOLDER_NAME=$(echo "$LAST_BRANCH" | sed 's|^chief-wiggum/||')
    ARCHIVE_FOLDER="$ARCHIVE_DIR/$DATE-$FOLDER_NAME"

    echo "Archiving previous run: $LAST_BRANCH"
    mkdir -p "$ARCHIVE_FOLDER"
    [ -f "$PRD_FILE" ] && cp "$PRD_FILE" "$ARCHIVE_FOLDER/"
    [ -f "$PROGRESS_FILE" ] && cp "$PROGRESS_FILE" "$ARCHIVE_FOLDER/"
    echo "   Archived to: $ARCHIVE_FOLDER"

    # Reset progress file for new run
    echo "# Chief Wiggum Progress Log" > "$PROGRESS_FILE"
    echo "Started: $(date)" >> "$PROGRESS_FILE"
    echo "---" >> "$PROGRESS_FILE"
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
  echo "# Chief Wiggum Progress Log" > "$PROGRESS_FILE"
  echo "Started: $(date)" >> "$PROGRESS_FILE"
  echo "---" >> "$PROGRESS_FILE"
fi

# Function to build quality checks string from config
build_quality_checks() {
  jq -r '.qualityChecks[]? | "   - \(.name): \(.command)"' "$CONFIG_FILE" 2>/dev/null || echo "   - typecheck: npm run typecheck"
}

# Function to render the prompt template with story data
render_prompt() {
  local story_id="$1"
  local story_title="$2"
  local story_description="$3"
  local acceptance_criteria="$4"
  local project_name="$5"
  local branch_name="$6"
  local project_description="$7"

  local quality_checks
  quality_checks=$(build_quality_checks)

  # Read template and substitute placeholders
  cat "$TEMPLATE_FILE" | \
    sed "s|{{STORY_ID}}|$story_id|g" | \
    sed "s|{{STORY_TITLE}}|$story_title|g" | \
    sed "s|{{STORY_DESCRIPTION}}|$story_description|g" | \
    sed "s|{{ACCEPTANCE_CRITERIA}}|$acceptance_criteria|g" | \
    sed "s|{{PROJECT_NAME}}|$project_name|g" | \
    sed "s|{{BRANCH_NAME}}|$branch_name|g" | \
    sed "s|{{PROJECT_DESCRIPTION}}|$project_description|g" | \
    sed "s|{{QUALITY_CHECKS}}|$quality_checks|g" | \
    sed "s|{{COMPLETION_PROMISE}}|$COMPLETION_PROMISE|g" | \
    sed "s|{{BLOCKED_PROMISE}}|$BLOCKED_PROMISE|g"
}

# Function to get the next incomplete story
get_next_story() {
  jq -r '.userStories | map(select(.passes == false)) | sort_by(.priority) | .[0] // empty' "$PRD_FILE"
}

# Function to mark a story as complete
mark_story_complete() {
  local story_id="$1"
  local tmp_file=$(mktemp)
  jq --arg id "$story_id" '(.userStories[] | select(.id == $id)).passes = true' "$PRD_FILE" > "$tmp_file"
  mv "$tmp_file" "$PRD_FILE"
}

# Function to check if all stories are complete
all_stories_complete() {
  local incomplete=$(jq -r '.userStories | map(select(.passes == false)) | length' "$PRD_FILE")
  [ "$incomplete" -eq 0 ]
}

# Main execution
echo "=================================================="
echo "  Chief Wiggum - Claude Code Story Orchestrator"
echo "=================================================="
echo ""

PROJECT_NAME=$(jq -r '.project // "Unknown"' "$PRD_FILE")
BRANCH_NAME=$(jq -r '.branchName // "main"' "$PRD_FILE")
PROJECT_DESCRIPTION=$(jq -r '.description // ""' "$PRD_FILE")
TOTAL_STORIES=$(jq -r '.userStories | length' "$PRD_FILE")
COMPLETED_STORIES=$(jq -r '.userStories | map(select(.passes == true)) | length' "$PRD_FILE")

echo "Project: $PROJECT_NAME"
echo "Branch: $BRANCH_NAME"
echo "Stories: $COMPLETED_STORIES/$TOTAL_STORIES complete"
echo "Max iterations per story: $MAX_ITERATIONS_PER_STORY"
echo ""

STORY_COUNT=0

while [ $STORY_COUNT -lt $MAX_STORIES ]; do
  # Check if all stories are complete
  if all_stories_complete; then
    echo ""
    echo "=================================================="
    echo "  ALL STORIES COMPLETE!"
    echo "=================================================="
    echo ""
    echo "Chief Wiggum has successfully completed all $TOTAL_STORIES stories."
    exit 0
  fi

  # Get next story
  STORY_JSON=$(get_next_story)
  if [ -z "$STORY_JSON" ]; then
    echo "No more stories to process."
    break
  fi

  STORY_ID=$(echo "$STORY_JSON" | jq -r '.id')
  STORY_TITLE=$(echo "$STORY_JSON" | jq -r '.title')
  STORY_DESCRIPTION=$(echo "$STORY_JSON" | jq -r '.description')
  STORY_PRIORITY=$(echo "$STORY_JSON" | jq -r '.priority')

  # Build acceptance criteria as a formatted list
  ACCEPTANCE_CRITERIA=$(echo "$STORY_JSON" | jq -r '.acceptanceCriteria | map("- [ ] " + .) | join("\n")')

  STORY_COUNT=$((STORY_COUNT + 1))

  echo ""
  echo "=================================================="
  echo "  Story $STORY_COUNT: $STORY_ID - $STORY_TITLE"
  echo "  Priority: $STORY_PRIORITY"
  echo "=================================================="
  echo ""

  # Render the prompt with story data
  PROMPT=$(render_prompt "$STORY_ID" "$STORY_TITLE" "$STORY_DESCRIPTION" "$ACCEPTANCE_CRITERIA" "$PROJECT_NAME" "$BRANCH_NAME" "$PROJECT_DESCRIPTION")

  # Escape the prompt for command line (handle quotes and special chars)
  ESCAPED_PROMPT=$(echo "$PROMPT" | sed 's/"/\\"/g' | tr '\n' ' ')

  # Execute Claude with /ralph-loop:ralph-loop
  echo "Spawning Claude Code with /ralph-loop:ralph-loop..."
  echo "Max iterations: $MAX_ITERATIONS_PER_STORY"
  echo ""

  # Run Claude CLI with /ralph-loop:ralph-loop skill
  OUTPUT=$(claude --dangerously-skip-permissions --print "/ralph-loop:ralph-loop \"$ESCAPED_PROMPT\" --max-iterations $MAX_ITERATIONS_PER_STORY --completion-promise $COMPLETION_PROMISE" 2>&1 | tee /dev/stderr) || true

  # Check for completion signal
  if echo "$OUTPUT" | grep -q "<promise>$COMPLETION_PROMISE</promise>"; then
    echo ""
    echo "Story $STORY_ID completed successfully!"
    mark_story_complete "$STORY_ID"

    # Update progress
    COMPLETED_STORIES=$((COMPLETED_STORIES + 1))
    echo ""
    echo "Progress: $COMPLETED_STORIES/$TOTAL_STORIES stories complete"

    # Log completion to progress file
    echo "" >> "$PROGRESS_FILE"
    echo "## $(date) - $STORY_ID COMPLETED" >> "$PROGRESS_FILE"
    echo "Story: $STORY_TITLE" >> "$PROGRESS_FILE"
    echo "---" >> "$PROGRESS_FILE"

  elif echo "$OUTPUT" | grep -q "<promise>$BLOCKED_PROMISE</promise>"; then
    echo ""
    echo "Story $STORY_ID is BLOCKED!"
    echo "Check progress.txt for blocker details."

    # Log blocker to progress file
    echo "" >> "$PROGRESS_FILE"
    echo "## $(date) - $STORY_ID BLOCKED" >> "$PROGRESS_FILE"
    echo "Story: $STORY_TITLE" >> "$PROGRESS_FILE"
    echo "Check Claude output for blocker details." >> "$PROGRESS_FILE"
    echo "---" >> "$PROGRESS_FILE"

    echo ""
    echo "Stopping due to blocker. Fix the issue and restart."
    exit 1
  else
    echo ""
    echo "Story $STORY_ID did not complete within $MAX_ITERATIONS_PER_STORY iterations."
    echo "Check output for details. The story may need to be split into smaller tasks."

    # Log timeout to progress file
    echo "" >> "$PROGRESS_FILE"
    echo "## $(date) - $STORY_ID TIMEOUT" >> "$PROGRESS_FILE"
    echo "Story: $STORY_TITLE" >> "$PROGRESS_FILE"
    echo "Did not complete within max iterations." >> "$PROGRESS_FILE"
    echo "---" >> "$PROGRESS_FILE"
  fi

  echo ""
  echo "Waiting before next story..."
  sleep 2
done

echo ""
echo "=================================================="
echo "  Chief Wiggum Session Complete"
echo "=================================================="
echo ""

if all_stories_complete; then
  echo "All stories completed successfully!"
  exit 0
else
  REMAINING=$(jq -r '.userStories | map(select(.passes == false)) | length' "$PRD_FILE")
  echo "Processed $STORY_COUNT stories."
  echo "Remaining stories: $REMAINING"
  echo "Check progress.txt for status."
  exit 1
fi
