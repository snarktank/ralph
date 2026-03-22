#!/bin/bash

# Check if file argument is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <file>"
    exit 1
fi

file="$1"

# Check if file exists
if [ ! -f "$file" ]; then
    echo "Error: File '$file' not found"
    exit 1
fi

# Count words using wc -w
word_count=$(wc -w < "$file")

# Output in required format
echo "${file}: ${word_count} words"
