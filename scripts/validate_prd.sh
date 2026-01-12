#!/usr/bin/env bash
set -e

FILE=${1:-prd.json}

if [ ! -f "$FILE" ]; then
  echo "✗ PRD file not found: $FILE"
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "✗ jq is required but not installed"
  exit 1
fi

REQUIRED_FIELDS=("id" "title" "priority")
ERROR=0

echo "Validating PRD: $FILE"
echo "-------------------------"

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
    VALUE=$(jq -r ".userStories[$i].$FIELD // empty" "$FILE")
    if [ -z "$VALUE" ]; then
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


# // this is the main file that i have added validate_prd.sh file in the main root folder i have made a file name 
# prd.json.example also in the main root folder
# // now i will run this script to validate the prd.json file
