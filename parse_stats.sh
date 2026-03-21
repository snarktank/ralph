#!/bin/bash

# Check if jq is available
if ! command -v jq &> /dev/null; then
    echo "Error: jq is required but not installed" >&2
    exit 1
fi

# Check if prd.json exists
if [ ! -f "prd.json" ]; then
    echo "Error: prd.json not found" >&2
    exit 1
fi

# Parse prd.json and count stories
TOTAL=$(jq '.userStories | length' prd.json)
PASSED=$(jq '[.userStories[] | select(.passes == true)] | length' prd.json)
FAILED=$(jq '[.userStories[] | select(.passes == false)] | length' prd.json)

# Output the required format
echo "Stories: $TOTAL total, $PASSED passed, $FAILED failed"
