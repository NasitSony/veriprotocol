#!/usr/bin/env bash
set -euo pipefail

SCHEDULERS=("fifo" "delay-leader" "timeout-first")
THRESHOLDS=(0 2 4 6 8 10 11 12 13 14 15 16 17 20)

for scheduler in "${SCHEDULERS[@]}"; do
  for t in "${THRESHOLDS[@]}"; do
    echo "=== scheduler=${scheduler}, T=${t} ==="
    cargo run --quiet -- "$scheduler" 1 42 timeout "$t" \
      | grep -E "Messages Sent:|Messages Delivered:|Stale Messages Ignored:|Timeout triggered:|View changes:|Decisions:"
    echo
  done
done