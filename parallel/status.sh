#!/usr/bin/env bash
set -euo pipefail
#
# status.sh — Show status of Ralph parallel agents and PRD stories.
#
# Usage: ./parallel/status.sh [--project DIR]
#

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RALPH_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

source "$SCRIPT_DIR/lib/logging.sh"

PROJECT_DIR=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --project) PROJECT_DIR="$2"; shift 2 ;;
        --project=*) PROJECT_DIR="${1#*=}"; shift ;;
        *) shift ;;
    esac
done
if [ -z "$PROJECT_DIR" ]; then
    PROJECT_DIR="$(pwd)"
fi
PROJECT_DIR="$(cd "$PROJECT_DIR" && pwd)"
PRD_FILE="$PROJECT_DIR/prd.json"
BARE_REPO="$PROJECT_DIR/.ralph/repo.git"

# Helper: read file from bare repo
read_from_bare_repo() {
    local file="$1"
    git --git-dir="$BARE_REPO" show "HEAD:${file}" 2>/dev/null || echo ""
}

# Load project info — prefer bare repo (has latest agent pushes), fallback to working dir
PROJECT_NAME="unknown"
PRD_CONTENT=""
if [ -d "$BARE_REPO" ]; then
    PRD_CONTENT=$(read_from_bare_repo "prd.json")
fi
if [ -z "$PRD_CONTENT" ] && [ -f "$PRD_FILE" ]; then
    PRD_CONTENT=$(cat "$PRD_FILE")
fi
if [ -n "$PRD_CONTENT" ]; then
    PROJECT_NAME=$(echo "$PRD_CONTENT" | jq -r '.project // "unknown"' 2>/dev/null || echo "unknown")
fi

echo "========================================"
echo " Ralph Parallel Status: $PROJECT_NAME"
echo "========================================"
echo ""

# --- Stop signal check ---
if [ -s "$PROJECT_DIR/.ralph/stop_requested" ]; then
    echo "** STOP REQUESTED -- agents will exit after current iteration **"
    echo ""
fi

# --- Container Status ---
echo "--- Containers ---"
CONTAINERS=$(docker ps -a --filter "name=ralph-agent-" --format "table {{.Names}}\t{{.Status}}\t{{.RunningFor}}" 2>/dev/null || true)
if [ -n "$CONTAINERS" ]; then
    echo "$CONTAINERS"
else
    echo "No Ralph containers found."
fi
echo ""

# --- Story Board from prd.json ---
echo "--- Story Board ---"
if [ -n "$PRD_CONTENT" ]; then
    # Available stories (passes: false, no claim)
    echo "Available:"
    AVAILABLE=$(echo "$PRD_CONTENT" | jq -r '
        .userStories[]
        | select(.passes == false and (.claimed_by == null or .claimed_by == ""))
        | "  [ ] \(.id): \(.title) (priority: \(.priority))"
    ' 2>/dev/null || echo "")
    if [ -n "$AVAILABLE" ]; then
        echo "$AVAILABLE"
    else
        echo "  (none)"
    fi

    # Claimed stories (passes: false, has claim)
    echo "Claimed:"
    CLAIMED=$(echo "$PRD_CONTENT" | jq -r '
        .userStories[]
        | select(.passes == false and .claimed_by != null and .claimed_by != "")
        | "  [~] \(.id): \(.title) (by \(.claimed_by) at \(.claimed_at // "?"))"
    ' 2>/dev/null || echo "")
    if [ -n "$CLAIMED" ]; then
        echo "$CLAIMED"
    else
        echo "  (none)"
    fi

    # Complete stories (passes: true)
    echo "Done:"
    DONE=$(echo "$PRD_CONTENT" | jq -r '
        .userStories[]
        | select(.passes == true)
        | "  [x] \(.id): \(.title)"
    ' 2>/dev/null || echo "")
    if [ -n "$DONE" ]; then
        echo "$DONE"
    else
        echo "  (none)"
    fi

    # Summary
    TOTAL=$(echo "$PRD_CONTENT" | jq '.userStories | length' 2>/dev/null || echo "?")
    DONE_COUNT=$(echo "$PRD_CONTENT" | jq '[.userStories[] | select(.passes == true)] | length' 2>/dev/null || echo "?")
    echo ""
    echo "Progress: $DONE_COUNT/$TOTAL stories complete"
else
    echo "No prd.json found."
fi
echo ""

# --- Recent Logs ---
echo "--- Recent Logs (last 10 lines, 3 most recent) ---"
LOG_DIR="$PROJECT_DIR/agent_logs"
if [ -d "$LOG_DIR" ]; then
    LATEST_LOGS=$(ls -t "$LOG_DIR"/*.log 2>/dev/null | head -3)
    if [ -n "$LATEST_LOGS" ]; then
        for logfile in $LATEST_LOGS; do
            echo "  $(basename "$logfile"):"
            tail -n 10 "$logfile" | sed 's/^/    /'
            echo ""
        done
    else
        echo "  No log files yet."
    fi
else
    echo "  No log directory found."
fi

# --- Git Log ---
echo "--- Recent Commits ---"
if [ -d "$BARE_REPO" ]; then
    git --git-dir="$BARE_REPO" log --oneline -10 2>/dev/null || echo "  No commits yet."
elif [ -d "$PROJECT_DIR/.git" ]; then
    git -C "$PROJECT_DIR" log --oneline -10 2>/dev/null || echo "  No commits yet."
else
    echo "  Not a git repository."
fi
echo ""
