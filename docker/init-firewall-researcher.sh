#!/usr/bin/env bash
set -euo pipefail
#
# Researcher firewall: no-op.
# Docker default networking allows all outbound traffic.
# Researcher agents need full internet access for web searches, docs, etc.
#

echo "[firewall] Researcher firewall: no restrictions (full internet access)."
