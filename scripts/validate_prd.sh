#!/usr/bin/env bash
set -e

FILE=${1:-prd.json}

if [ ! -f "$FILE" ]; then
  echo "✗ PRD file not found: $FILE"
  exit 1
fi

REQUIRED_FIELDS=("id" "description" "status" "priority")
VALID_STATUS=("todo" "in_progress" "done")

ERROR=0

echo "Validating PRD: $FILE"
echo "-------------------------"

IDS=$(jq -r '.tasks[].id' "$FILE")

# Duplicate IDs
DUP_IDS=$(echo "$IDS" | sort | uniq -d)
if [ -n "$DUP_IDS" ]; then
  echo "✗ Duplicate task IDs:"
  echo "$DUP_IDS"
  ERROR=1
fi

# Validate each task
COUNT=$(jq '.tasks | length' "$FILE")

for ((i=0; i<COUNT; i++)); do
  for FIELD in "${REQUIRED_FIELDS[@]}"; do
    VALUE=$(jq -r ".tasks[$i].$FIELD // empty" "$FILE")
    if [ -z "$VALUE" ]; then
      echo "✗ Missing field '$FIELD' in task index $i"
      ERROR=1
    fi
  done

  STATUS=$(jq -r ".tasks[$i].status" "$FILE")
  if [[ ! " ${VALID_STATUS[*]} " =~ " $STATUS " ]]; then
    echo "✗ Invalid status '$STATUS' in task index $i"
    ERROR=1
  fi
done

if [ "$ERROR" -eq 0 ]; then
  echo "✓ PRD is valid"
else
  echo "✗ PRD validation failed"
  exit 1
fi
