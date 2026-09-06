#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LABRADOR="$ROOT/third_party/labrador"
OUT_DIR="$ROOT/target/greyhound"
OUT="$OUT_DIR/lattice-eval"

if [[ ! -f "$LABRADOR/greyhound.c" ]]; then
  echo "Greyhound vendor missing. Run $ROOT/scripts/fetch-vendors.sh" >&2
  exit 1
fi

if [[ "$(uname -m)" != "x86_64" ]]; then
  echo "Greyhound requires x86_64 AVX-512; this machine is $(uname -m)." >&2
  exit 1
fi

if [[ -r /proc/cpuinfo ]] && ! grep -q avx512f /proc/cpuinfo; then
  echo "Greyhound requires AVX-512F. This CPU does not advertise avx512f." >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
CC="${CC:-cc}"
# Labrador's Makefile uses -march=native -O3 -flto. Keep that ISA pin so the
# comparison is against the released Greyhound binary shape.
CFLAGS=(-std=c2x -O3 -flto=auto -fwrapv -march=native -mtune=native -Wall)
SOURCES=(
  pack.c greyhound.c dachshund.c chihuahua.c labrador.c
  data.c jlproj.c polx.c poly.c polz.c sparsemat.c ntt.S invntt.S
  aesctr.c fips202.c randombytes.c cpucycles.c
)

cd "$LABRADOR"
"$CC" "${CFLAGS[@]}" \
  -I"$LABRADOR" \
  "$ROOT/benchmarks/greyhound/src/lattice_eval.c" \
  "${SOURCES[@]}" \
  -lm \
  -o "$OUT"

echo "Greyhound worker: $OUT"
