#!/bin/bash
# install-ralph.sh - Install Ralph in your current project
# Usage: ./install-ralph.sh [path-to-ralph-repo]

set -e

# Get the path to the Ralph repository
RALPH_REPO="${1:-$(dirname "$(readlink -f "$0")")}"

# If script is in ralph repo, use parent directory
if [ -f "$RALPH_REPO/ralph.sh" ]; then
  RALPH_REPO="$RALPH_REPO"
elif [ -f "$(dirname "$RALPH_REPO")/ralph.sh" ]; then
  RALPH_REPO="$(dirname "$RALPH_REPO")"
else
  echo "Error: Could not find ralph.sh"
  echo "Usage: ./install-ralph.sh [path-to-ralph-repo]"
  exit 1
fi

PROJECT_ROOT="$(pwd)"

echo "Installing Ralph..."
echo "  Ralph repo: $RALPH_REPO"
echo "  Project: $PROJECT_ROOT"

# Create scripts/ralph directory
mkdir -p scripts/ralph

# Copy ralph files
cp "$RALPH_REPO/ralph.sh" scripts/ralph/
cp "$RALPH_REPO/prompt.md" scripts/ralph/

# Make ralph.sh executable
chmod +x scripts/ralph/ralph.sh

echo ""
echo "✓ Ralph installed successfully!"
echo ""
echo "Next steps:"
echo "  1. Create a PRD: agent -p \"Create a PRD for [feature]\""
echo "  2. Convert to prd.json: agent -p \"Convert tasks/prd-[feature].md to prd.json\""
echo "  3. Run Ralph: ./scripts/ralph/ralph.sh"
echo ""
echo "Optional: Copy skills for global use:"
echo "  cp -r $RALPH_REPO/skills ~/.cursor/skills/"