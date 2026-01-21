#!/usr/bin/env bash
set -e

FILE=${1:-prd.json}

# Ensure PRD file exists
if [ ! -f "$FILE" ]; then
  echo "✗ PRD file not found: $FILE"
  exit 1
fi

# Ensure jq is available
if ! command -v jq >/dev/null 2>&1; then
  echo "✗ jq is required but not installed"
  exit 1
fi

# Fields that must exist on each user story
REQUIRED_FIELDS=("id" "title" "priority")
ERROR=0

echo "Validating PRD: $FILE"
echo "-------------------------"

# Ensure userStories exists and is an array
if ! jq -e '.userStories | type == "array"' "$FILE" >/dev/null; then
  echo "✗ 'userStories' must be an array"
  exit 1
fi

# Collect all user story IDs
IDS=$(jq -r '.userStories[].id' "$FILE")

# Check for duplicate IDs
DUP_IDS=$(echo "$IDS" | sort | uniq -d)
if [ -n "$DUP_IDS" ]; then
  echo "✗ Duplicate userStory IDs:"
  echo "$DUP_IDS"
  ERROR=1
fi

# Validate each user story
COUNT=$(jq '.userStories | length' "$FILE")

for ((i=0; i<COUNT; i++)); do
  for FIELD in "${REQUIRED_FIELDS[@]}"; do
    EXISTS=$(jq ".userStories[$i] | has(\"$FIELD\")" "$FILE")
    if [ "$EXISTS" != "true" ]; then
      echo "✗ Missing field '$FIELD' in userStory index $i"
      ERROR=1
    fi
  done
done

if [ "$ERROR" -eq 0 ]; then
  echo "✓ PRD is valid"
else
  echo "✗ PRD validation failed"
  exit 1
fi
