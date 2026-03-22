#!/bin/bash

# Check if file argument is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <csv_file>"
    exit 1
fi

CSV_FILE="$1"

# Check if file exists
if [ ! -f "$CSV_FILE" ]; then
    echo "Error: File '$CSV_FILE' not found"
    exit 1
fi

# Read the header to find the 'value' column index
HEADER=$(head -n 1 "$CSV_FILE")
IFS=',' read -ra HEADERS <<< "$HEADER"

VALUE_COL=-1
for i in "${!HEADERS[@]}"; do
    # Remove quotes and trim whitespace
    COL_NAME=$(echo "${HEADERS[$i]}" | sed 's/^"//g' | sed 's/"$//g' | xargs)
    if [ "$COL_NAME" = "value" ]; then
        VALUE_COL=$((i + 1)) # awk uses 1-based indexing
        break
    fi
done

if [ $VALUE_COL -eq -1 ]; then
    echo "Error: 'value' column not found in CSV"
    exit 1
fi

# Count data rows (excluding header)
ROWS=$(tail -n +2 "$CSV_FILE" | wc -l)

# Sum values from the value column
SUM=$(tail -n +2 "$CSV_FILE" | awk -F, -v col="$VALUE_COL" '{
    gsub(/"/, "", $col)  # Remove quotes
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", $col)  # Trim whitespace
    if ($col ~ /^-?[0-9]+$/) {
        sum += $col
    }
} END { print sum + 0 }')

# Calculate integer average (floor)
if [ "$ROWS" -gt 0 ]; then
    AVG=$((SUM / ROWS))
else
    AVG=0
fi

# Output results
echo "rows: $ROWS"
echo "sum: $SUM"
echo "avg: $AVG"
