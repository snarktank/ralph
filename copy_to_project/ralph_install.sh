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

# Copy protection and documentation files
# Template files are in the ralph repo's copy_to_project directory
RALPH_TEMPLATE_DIR="$RALPH_REPO/copy_to_project"
if [ -f "$RALPH_TEMPLATE_DIR/scripts_ralph_gitattributes" ]; then
  cp "$RALPH_TEMPLATE_DIR/scripts_ralph_gitattributes" scripts/ralph/.gitattributes
else
  # Fallback: create .gitattributes directly
  cat > scripts/ralph/.gitattributes << 'EOF'
# Ralph Scripts Protection
# These files are managed by Ralph installation - use merge=ours to prevent overwrites

ralph.sh merge=ours
prompt.md merge=ours
*.sh -crlf -text
*.md -crlf -text
EOF
fi

if [ -f "$RALPH_TEMPLATE_DIR/scripts_ralph_README.md" ]; then
  cp "$RALPH_TEMPLATE_DIR/scripts_ralph_README.md" scripts/ralph/README.md
fi

# Create version marker (git commit hash or "unknown")
RALPH_VERSION="unknown"
if command -v git &> /dev/null && [ -d "$RALPH_REPO/.git" ]; then
  RALPH_VERSION=$(cd "$RALPH_REPO" && git rev-parse HEAD 2>/dev/null || echo "unknown")
fi
echo "$RALPH_VERSION" > scripts/ralph/.ralph-version

# Create protection marker
cat > scripts/ralph/.ralph-protected << EOF
# This folder is managed by Ralph installation
# Do not manually edit ralph.sh or prompt.md
# 
# Installed version: $RALPH_VERSION
# Installation date: $(date)
# 
# To update: Re-run ralph_install.sh from your project root
EOF

# Set file permissions
chmod +x scripts/ralph/ralph.sh
chmod 644 scripts/ralph/prompt.md
chmod 644 scripts/ralph/.gitattributes 2>/dev/null || true
chmod 644 scripts/ralph/README.md 2>/dev/null || true
chmod 644 scripts/ralph/.ralph-version 2>/dev/null || true
chmod 644 scripts/ralph/.ralph-protected 2>/dev/null || true

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