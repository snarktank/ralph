#!/bin/bash

# Check if argument is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <name>"
    exit 1
fi

# Get the name from first argument
NAME="$1"

# Print the greeting in exact format
echo "Hello, $NAME! Welcome."
