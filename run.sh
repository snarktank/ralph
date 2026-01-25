#!/bin/bash

# Ralph Wiggum - Autonomous Claude Code Runner
# Usage: ralph-run <project-name>

RALPH_HOME="$HOME/Documents/Developer/GitHub Sandbox/ralph"
RALPH_PROJECTS="$HOME/Documents/Developer/GitHub Sandbox/ralph-projects"

if [ -z "$1" ]; then
    echo "Ralph Wiggum - Autonomous Project Runner"
    echo ""
    echo "Usage: ralph-run <project-name>"
    echo ""
    echo "Available projects:"
    ls -1 "$RALPH_PROJECTS" 2>/dev/null || echo "  (none yet)"
    echo ""
    echo "Create new project:"
    echo "  ralph-new <project-name>"
    exit 1
fi

PROJECT_DIR="$RALPH_PROJECTS/$1"

if [ ! -d "$PROJECT_DIR" ]; then
    echo "Error: Project '$1' not found at $PROJECT_DIR"
    echo ""
    echo "Available projects:"
    ls -1 "$RALPH_PROJECTS" 2>/dev/null || echo "  (none)"
    exit 1
fi

if [ ! -f "$PROJECT_DIR/CLAUDE.md" ]; then
    echo "Error: No CLAUDE.md found in $PROJECT_DIR"
    exit 1
fi

if [ ! -f "$PROJECT_DIR/prd.json" ]; then
    echo "Error: No prd.json found in $PROJECT_DIR"
    exit 1
fi

echo "Starting Ralph on project: $1"
echo "Project directory: $PROJECT_DIR"
echo ""

cd "$PROJECT_DIR"
claude --dangerously-skip-permissions
