#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
RESULT_DIR="${PCS_BENCH_RESULT_DIR:-"$ROOT/results/$STAMP"}"
FILTER="${1:-}"

mkdir -p "$RESULT_DIR"

cpu_model() {
  if command -v sysctl >/dev/null 2>&1; then
    sysctl -n machdep.cpu.brand_string 2>/dev/null ||
      sysctl -n hw.model 2>/dev/null ||
      printf 'unknown\n'
  elif [ -r /proc/cpuinfo ]; then
    awk -F: '/model name/ { sub(/^ /, "", $2); print $2; exit }' /proc/cpuinfo
  else
    printf 'unknown\n'
  fi
}

{
  printf 'captured_at_utc=%s\n' "$STAMP"
  printf 'harness_revision=%s\n' "$(git -C "$ROOT" rev-parse HEAD 2>/dev/null || printf uncommitted)"
  printf 'akita_revision=%s\n' 'd1b224d809c7edc357b0dbab0f607e19b475910b'
  printf 'target=%s\n' "$(uname -sm)"
  printf 'cpu_model=%s\n' "$(cpu_model)"
  printf 'logical_cpus=%s\n' "$(getconf _NPROCESSORS_ONLN 2>/dev/null || sysctl -n hw.logicalcpu)"
  printf 'rustflags=%s\n' "${RUSTFLAGS:-"-C target-cpu=native"}"
  printf 'filter=%s\n' "$FILTER"
  rustc -Vv
  cargo -V
} >"$RESULT_DIR/provenance.txt"

export CARGO_NET_GIT_FETCH_WITH_CLI="${CARGO_NET_GIT_FETCH_WITH_CLI:-true}"
export RUSTFLAGS="${RUSTFLAGS:--C target-cpu=native}"

command=(cargo bench -p pcs-bench-akita --bench e2e --)
if [ -n "$FILTER" ]; then
  command+=("$FILTER")
fi

printf '%q ' "${command[@]}" >"$RESULT_DIR/command.txt"
printf '\n' >>"$RESULT_DIR/command.txt"

cd "$ROOT"
"${command[@]}" 2>&1 | tee "$RESULT_DIR/output.log"

printf 'Criterion reports: %s\n' "$ROOT/target/criterion"
printf 'Run metadata: %s\n' "$RESULT_DIR"

