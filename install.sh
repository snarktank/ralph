#!/bin/bash
# Installer for Ralph - The autonomous AI agent
set -e

# Configuration
INSTALL_DIR="scripts/ralph"
GITHUB_REPO="https://raw.githubusercontent.com/snarktank/ralph/main"
FILES_TO_INSTALL=("ralph.sh" "prompt.md")

# Welcome message
echo "Installing Ralph..."
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

for skill in "prd" "ralph"; do
    SKILL_URL="$GITHUB_REPO/skills/$skill"
    DEST="$AMP_SKILLS_DIR/$skill"

    echo -n "  - Downloading $skill skill..."
    if curl -fsSL "$SKILL_URL/SKILL.md" -o "$DEST/SKILL.md"; then
        echo " ✔"
    else
        echo " ✖ FAILED (skipping)"
    fi
done
echo "✔ Skills installed."
echo ""

# Success message
echo "Ralph installed successfully!"
echo "You can now run Ralph using:"
echo "  ./$INSTALL_DIR/ralph.sh"
