#!/bin/bash

# Check if name argument is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <name>"
    exit 1
fi

name="$1"
echo "Hello, $name! Welcome."
