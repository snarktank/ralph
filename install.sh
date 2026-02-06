#!/bin/bash
# Installer for Ralph - The autonomous AI agent
# NOTE: This installs from the main branch (latest development version).
# For stable releases, use install-from-release.sh instead.
# Usage: ./install.sh [--tool amp|claude]
set -e

# Configuration
INSTALL_DIR="scripts/ralph"
GITHUB_REPO="https://raw.githubusercontent.com/snarktank/ralph/main"
TOOL="amp"  # Default tool

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --tool)
      TOOL="$2"
      shift 2
      ;;
    --tool=*)
      TOOL="${1#*=}"
      shift
      ;;
    *)
      echo "Error: Unknown argument '$1'"
      echo "Usage: $0 [--tool amp|claude]"
      exit 1
      ;;
  esac
done

# Validate tool choice
if [[ "$TOOL" != "amp" && "$TOOL" != "claude" ]]; then
  echo "Error: Invalid tool '$TOOL'. Must be 'amp' or 'claude'."
  exit 1
fi

# Determine files to install based on tool
if [[ "$TOOL" == "amp" ]]; then
    FILES_TO_INSTALL=("ralph.sh" "prompt.md")
else
    FILES_TO_INSTALL=("ralph.sh" "CLAUDE.md")
fi

# Welcome message
echo "Installing Ralph from main branch (tool: $TOOL)"
echo "This will create a '$INSTALL_DIR' directory in your current project."
echo ""

# Create installation directory
mkdir -p "$INSTALL_DIR"
echo "✔ Created directory: $INSTALL_DIR"

# Download and install files
for file in "${FILES_TO_INSTALL[@]}"; do
    URL="$GITHUB_REPO/$file"
    DEST="$INSTALL_DIR/$file"

    echo -n "  - Downloading $file..."
    if curl -fsSL "$URL" -o "$DEST"; then
        echo " ✔"
    else
        echo " ✖ FAILED"
        echo "Error: Could not download $URL"
        echo "Please check the URL and your internet connection."
        exit 1
    fi
done

# Make ralph.sh executable
chmod +x "$INSTALL_DIR/ralph.sh"
echo "✔ Made ralph.sh executable"
echo ""

# Configure Amp auto-handoff (only for Amp)
if [[ "$TOOL" == "amp" ]]; then
    AMP_CONFIG_DIR="$HOME/.config/amp"
    AMP_SETTINGS_FILE="$AMP_CONFIG_DIR/settings.json"

    echo "Configuring Amp auto-handoff..."

    # Check for jq
    if ! command -v jq &> /dev/null; then
        echo "  - jq is not installed. Please install it to auto-configure Amp."
        echo "    (e.g., 'brew install jq' on macOS)"
        echo "  - Skipping auto-configuration."
    else
        # Ensure config directory and file exist
        mkdir -p "$AMP_CONFIG_DIR"
        [ -f "$AMP_SETTINGS_FILE" ] || echo "{}" > "$AMP_SETTINGS_FILE"

        # Check if autoHandoff is already configured
        if jq -e '."amp.experimental.autoHandoff"' "$AMP_SETTINGS_FILE" > /dev/null; then
            echo "✔ Amp auto-handoff is already configured."
        else
            # Add the autoHandoff setting (atomic write with cleanup)
            TEMP_FILE=$(mktemp "$AMP_SETTINGS_FILE".XXXXXX)
            jq '."amp.experimental.autoHandoff" = {"context": 90}' "$AMP_SETTINGS_FILE" > "$TEMP_FILE" && mv "$TEMP_FILE" "$AMP_SETTINGS_FILE"
            echo "✔ Enabled Amp auto-handoff for large stories."
        fi
    fi
    echo ""

    # Install skills for Amp
    SKILLS_DIR="$HOME/.config/amp/skills"
    mkdir -p "$SKILLS_DIR"
    echo "Installing skills globally to $SKILLS_DIR..."
else
    # Install skills for Claude Code
    SKILLS_DIR="$HOME/.claude/skills"
    mkdir -p "$SKILLS_DIR"
    echo "Installing skills globally to $SKILLS_DIR..."
fi

for skill in "prd" "ralph"; do
    mkdir -p "$SKILLS_DIR/$skill"

    echo -n "  - Downloading $skill skill..."
    TEMP_FILE=$(mktemp)
    if curl -fsSL "$GITHUB_REPO/skills/$skill/SKILL.md" -o "$TEMP_FILE" 2>/dev/null && mv "$TEMP_FILE" "$SKILLS_DIR/$skill/SKILL.md"; then
        echo " ✔"
    else
        echo " ✖ FAILED (skipping)"
        rm -f "$TEMP_FILE"
    fi
done
echo ""

# Success message
echo ""
echo "============================================"
echo "  Ralph installed successfully!"
echo "  Branch: main (development)"
echo "  Tool: $TOOL"
echo "============================================"
echo ""
echo "Next steps:"
echo ""
echo "1. Create a PRD"
echo "   Load the prd skill and create a PRD for [your feature description]"
echo "   Answer the clarifying questions. The skill saves output to tasks/prd-[feature-name].md."
echo ""
echo "2. Convert PRD to Ralph format"
echo "   Load the ralph skill and convert tasks/prd-[feature-name].md to prd.json"
echo ""
echo "3. Run Ralph"
echo "   ./scripts/ralph/ralph.sh [max_iterations]"
if [[ "$TOOL" == "claude" ]]; then
    echo "   (use --tool claude if not using default)"
fi
echo ""
echo "For more details, see: https://github.com/snarktank/ralph"
