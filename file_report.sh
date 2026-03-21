#!/bin/bash

# Find all .sh files in current directory (not subdirectories)
sh_files=(*.sh)

# Create an array to store file info
files_info=()

# Get size for each .sh file
for file in "${sh_files[@]}"; do
    if [[ -f "$file" ]]; then
        size=$(stat -c%s "$file")
        files_info+=("$file $size bytes")
    fi
done

# Sort by size descending (numeric sort on the second field)
sorted_files=($(printf '%s\n' "${files_info[@]}" | sort -k2 -nr))

# Print each file with size
printf '%s\n' "${sorted_files[@]}"

# Print total count
echo "Total: ${#sh_files[@]} files"
