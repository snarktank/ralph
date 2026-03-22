#!/bin/bash

# Check if file argument is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <file>" >&2
    exit 1
fi

file="$1"

# Count words using wc -w
word_count=$(wc -w < "$file")

# Output in required format: FILENAME: N words
echo "$file: $word_count words"
