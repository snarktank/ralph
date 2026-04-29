#!/usr/bin/env bash
set -Eeuo pipefail

SIZE_GB="${1:-8}"
SWAPFILE="${RALPH_SWAPFILE:-/swapfile}"

if [[ "$EUID" -ne 0 ]]; then
  echo "Usage: sudo $0 [size_gb]" >&2
  exit 1
fi

if ! [[ "$SIZE_GB" =~ ^[0-9]+$ ]] || [[ "$SIZE_GB" -lt 1 ]]; then
  echo "Error: size_gb must be a positive integer." >&2
  exit 1
fi

if swapon --show=NAME --noheadings | grep -Fxq "$SWAPFILE"; then
  echo "Swap is already active at $SWAPFILE"
  exit 0
fi

if [[ -e "$SWAPFILE" ]]; then
  echo "Error: $SWAPFILE already exists but is not active swap. Inspect it before continuing." >&2
  exit 1
fi

fallocate -l "${SIZE_GB}G" "$SWAPFILE" || dd if=/dev/zero of="$SWAPFILE" bs=1G count="$SIZE_GB" status=progress
chmod 600 "$SWAPFILE"
mkswap "$SWAPFILE"
swapon "$SWAPFILE"

if ! grep -Eq "^[^#].*[[:space:]]${SWAPFILE//\//\\/}[[:space:]]" /etc/fstab; then
  printf '%s none swap sw 0 0\n' "$SWAPFILE" >> /etc/fstab
fi

echo "Created and enabled ${SIZE_GB}GB swap at $SWAPFILE"
swapon --show
