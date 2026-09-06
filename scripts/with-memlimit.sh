#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "usage: with-memlimit.sh BYTES command..." >&2
  exit 2
fi

limit_bytes="$1"
shift
if [[ "$limit_bytes" -eq 0 ]]; then
  export RAYON_NUM_THREADS="${RAYON_NUM_THREADS:-1}"
  exec "$@"
fi
limit_kb=$((limit_bytes / 1024))
if ulimit -v "$limit_kb" 2>/dev/null; then
  :
else
  echo "warning: ulimit -v is unavailable; running without a hard virtual-memory cap" >&2
fi

export RAYON_NUM_THREADS="${RAYON_NUM_THREADS:-1}"
exec "$@"
