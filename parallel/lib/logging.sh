#!/usr/bin/env bash
#
# logging.sh — Timestamped log helpers for Ralph parallel mode.
#

log_info() {
    echo "[$(date -u +"%H:%M:%S")] INFO: $*"
}

log_warn() {
    echo "[$(date -u +"%H:%M:%S")] WARN: $*" >&2
}

log_error() {
    echo "[$(date -u +"%H:%M:%S")] ERROR: $*" >&2
}
