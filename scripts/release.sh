#!/bin/bash
# Helper script to create a new release
set -e

# Get current version from git tags
CURRENT_VERSION=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
echo "Current version: $CURRENT_VERSION"

# Parse version components
VERSION_REGEX="v([0-9]+)\.([0-9]+)\.([0-9]+)"
if [[ $CURRENT_VERSION =~ $VERSION_REGEX ]]; then
    MAJOR="${BASH_REMATCH[1]}"
    MINOR="${BASH_REMATCH[2]}"
    PATCH="${BASH_REMATCH[3]}"
else
    MAJOR=0
    MINOR=0
    PATCH=0
fi

# Determine bump type
BUMP_TYPE="${1:-patch}"
case $BUMP_TYPE in
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    patch)
        PATCH=$((PATCH + 1))
        ;;
    *)
        echo "Usage: $0 [major|minor|patch]"
        echo "  major: Breaking changes (v1.0.0 -> v2.0.0)"
        echo "  minor: New features (v1.0.0 -> v1.1.0)"
        echo "  patch: Bug fixes (v1.0.0 -> v1.0.1)"
        exit 1
        ;;
esac

NEW_VERSION="v$MAJOR.$MINOR.$PATCH"
echo "New version: $NEW_VERSION"
echo ""

# Confirm
read -p "Create release $NEW_VERSION? [y/N] " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 0
fi

# Create and push tag
git tag -a "$NEW_VERSION" -m "Release $NEW_VERSION"
git push origin "$NEW_VERSION"

echo ""
echo "✔ Tag $NEW_VERSION created and pushed."
echo "✔ GitHub Actions will create the release automatically."
echo ""
echo "View release at: https://github.com/snarktank/ralph/releases/tag/$NEW_VERSION"
