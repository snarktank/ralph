#!/bin/bash
# Installer for Ralph - The autonomous AI agent
# Installs from GitHub Releases for stable, versioned installation
set -e

# Configuration
INSTALL_DIR="scripts/ralph"
GITHUB_REPO="snarktank/ralph"
VERSION="${1:-latest}"

# Resolve version to release URL
if [ "$VERSION" = "latest" ]; then
    RELEASE_URL="https://github.com/$GITHUB_REPO/releases/latest/download"
else
    RELEASE_URL="https://github.com/$GITHUB_REPO/releases/download/v$VERSION"
fi

FILES_TO_INSTALL=("ralph.sh" "prompt.md")
SKILLS=("prd" "ralph")

# Welcome message
echo "Installing Ralph from release: $VERSION"
echo "This will create a '$INSTALL_DIR' directory in your current project."
echo ""

# Create installation directory
mkdir -p "$INSTALL_DIR"
echo "✔ Created directory: $INSTALL_DIR"

# Download and install files
for file in "${FILES_TO_INSTALL[@]}"; do
    URL="$RELEASE_URL/$file"
    DEST="$INSTALL_DIR/$file"

    echo -n "  - Downloading $file..."
    if curl -fsSL "$URL" -o "$DEST"; then
        echo " ✔"
    else
        echo " ✖ FAILED"
        echo "Error: Could not download $URL"
        echo "Please check the version exists and your internet connection."
        exit 1
    fi
done

# Make ralph.sh executable
chmod +x "$INSTALL_DIR/ralph.sh"
echo "✔ Made ralph.sh executable"
echo ""

# Configure Amp auto-handoff
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
        # Add the autoHandoff setting
        jq '."amp.experimental.autoHandoff" = {"context": 90}' "$AMP_SETTINGS_FILE" > "$AMP_SETTINGS_FILE.tmp" && mv "$AMP_SETTINGS_FILE.tmp" "$AMP_SETTINGS_FILE"
        echo "✔ Enabled Amp auto-handoff for large stories."
    fi
fi
echo ""

# Install skills globally
AMP_SKILLS_DIR="$HOME/.config/amp/skills"
mkdir -p "$AMP_SKILLS_DIR"
echo "Installing skills globally to $AMP_SKILLS_DIR..."

for skill in "${SKILLS[@]}"; do
    mkdir -p "$AMP_SKILLS_DIR/$skill"

    echo -n "  - Downloading $skill skill..."
    TEMP_FILE=$(mktemp)
    # Skills are uploaded with prefixed names (e.g., prd-SKILL.md, ralph-SKILL.md)
    if curl -fsSL "$RELEASE_URL/$skill-SKILL.md" -o "$TEMP_FILE" 2>/dev/null && mv "$TEMP_FILE" "$AMP_SKILLS_DIR/$skill/SKILL.md"; then
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
echo "  Version: $VERSION"
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
echo ""
echo "For more details, see: https://github.com/snarktank/ralph"
