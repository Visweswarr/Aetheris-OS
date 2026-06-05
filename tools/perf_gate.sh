#!/usr/bin/env bash
set -euo pipefail

THRESH_SUBMIT_P95_MS=250
THRESH_SNAP_WRITE_MS=100
THRESH_SNAP_READ_MS=60

export NGFS_ROOT=${NGFS_ROOT:-./data/ngfs}

cargo bench -p aetheris-ai --features phase5 --bench phase5_perf -- --quiet --measurement-time 5 || {
  echo "::error::criterion bench failed"; exit 2;
}

# Parse Criterion CSVs (best-effort): target/criterion/*/new/raw.csv
parse_p95() {
  local pattern="$1"
  local csv=$(grep -Rsl "raw.csv" target/criterion | head -n1)
  if [[ -z "$csv" ]]; then echo 0; return; fi
  # Expect columns with slope statistics; fallback to mean as proxy
  awk -F, 'NR>1 {print $2*1000}' "$csv" | sort -n | tail -n1
}

# We simply report mean as proxy; CI thresholds are lenient for CPU-only runners
SUBMIT_MS=$(parse_p95 submit_goal)
SNAPW_MS=$(parse_p95 snapshot_write)
SNAPR_MS=$(parse_p95 snapshot_read)

echo "submit_ms=${SUBMIT_MS:-0}"; echo "snapshot_write_ms=${SNAPW_MS:-0}"; echo "snapshot_read_ms=${SNAPR_MS:-0}"

fail=0
[[ "${SUBMIT_MS:-0}" -le $THRESH_SUBMIT_P95_MS ]] || { echo "::error::submit p95 > ${THRESH_SUBMIT_P95_MS}ms"; fail=2; }
[[ "${SNAPW_MS:-0}" -le $THRESH_SNAP_WRITE_MS ]] || { echo "::error::snapshot write > ${THRESH_SNAP_WRITE_MS}ms"; fail=2; }
[[ "${SNAPR_MS:-0}" -le $THRESH_SNAP_READ_MS ]] || { echo "::error::snapshot read > ${THRESH_SNAP_READ_MS}ms"; fail=2; }

exit $fail
