#!/bin/bash

# Directory tree generator
# Usage: ./tree_gen.sh <directory_path>

if [ $# -ne 1 ]; then
    echo "Usage: $0 <directory_path>" >&2
    exit 1
fi

DIR_PATH="$1"

if [ ! -d "$DIR_PATH" ]; then
    echo "Error: '$DIR_PATH' is not a directory" >&2
    exit 1
fi

# Function to print tree with max depth 2
tree_gen() {
    local current_dir="$1"
    local indent="$2"
    local depth="$3"
    
    if [ "$depth" -gt 2 ]; then
        return
    fi
    
    # Get sorted list of entries (excluding hidden files and symlinks)
    find "$current_dir" -maxdepth 1 -mindepth 1 ! -name ".*" -type f -o ! -name ".*" -type d | sort | while IFS= read -r entry; do
        basename_entry=$(basename "$entry")
        
        # Skip symlinks
        if [ -L "$entry" ]; then
            continue
        fi
        
        if [ -d "$entry" ]; then
            echo "${indent}${basename_entry}/"
            if [ "$depth" -lt 2 ]; then
                tree_gen "$entry" "${indent}  " $((depth + 1))
            fi
        elif [ -f "$entry" ]; then
            echo "${indent}${basename_entry}"
        fi
    done
}

# Start tree generation from root directory
tree_gen "$DIR_PATH" "" 1
