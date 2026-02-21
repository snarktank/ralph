#!/usr/bin/env bash
set -euo pipefail
#
# stop.sh — Graceful shutdown of Ralph parallel agents.
#
# Usage: ./parallel/stop.sh [--project DIR]
#
# Creates a stop_requested file that agents check each iteration.
# Waits up to 120s for graceful exit, then force-kills remaining containers.
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

# --- Signal stop ---
log_info "Requesting graceful stop for Ralph parallel agents..."
mkdir -p "$PROJECT_DIR/.ralph"
echo "stop" > "$PROJECT_DIR/.ralph/stop_requested"

# --- Wait for containers to stop ---
TIMEOUT=120
ELAPSED=0
CHECK_INTERVAL=5

log_info "Waiting up to ${TIMEOUT}s for agents to finish current iteration..."

while [ $ELAPSED -lt $TIMEOUT ]; do
    RUNNING=$(docker ps --filter "name=ralph-agent-" --format "{{.Names}}" 2>/dev/null || true)

    if [ -z "$RUNNING" ]; then
        log_info "All agents stopped gracefully."
        rm -f "$PROJECT_DIR/.ralph/stop_requested"
        exit 0
    fi

    RUNNING_COUNT=$(echo "$RUNNING" | wc -l | tr -d ' ')
    log_info "Still running: $RUNNING_COUNT containers ($ELAPSED/${TIMEOUT}s)"

    sleep "$CHECK_INTERVAL"
    ELAPSED=$((ELAPSED + CHECK_INTERVAL))
done

# --- Force kill remaining containers ---
REMAINING=$(docker ps --filter "name=ralph-agent-" --format "{{.Names}}" 2>/dev/null || true)

if [ -n "$REMAINING" ]; then
    log_warn "Timeout reached. Force-stopping remaining containers..."
    for name in $REMAINING; do
        log_warn "Force-stopping: $name"
        docker kill "$name" 2>/dev/null || true
        docker rm "$name" 2>/dev/null || true
    done
fi

rm -f "$PROJECT_DIR/.ralph/stop_requested"
log_info "Shutdown complete."
