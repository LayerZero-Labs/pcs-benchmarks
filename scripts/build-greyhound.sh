#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENDOR="$ROOT/third_party/greyhound-reference"
OUT_DIR="$ROOT/target/greyhound"
OUT="$OUT_DIR/lattice-eval"
REVISION="687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397"
STAMP="$OUT_DIR/revision"

if [[ ! -f "$VENDOR/greyhound.c" ]]; then
  echo "Greyhound vendor missing. Run $ROOT/scripts/fetch-vendors.sh" >&2
  exit 1
fi

if [[ "$(uname -m)" != "x86_64" ]]; then
  echo "Headline Greyhound requires x86_64 AVX-512; this machine is $(uname -m)." >&2
  exit 1
fi

if [[ -r /proc/cpuinfo ]] && ! grep -q avx512f /proc/cpuinfo; then
  echo "Headline Greyhound requires AVX-512F. This CPU does not advertise avx512f." >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
if [[ -x "$OUT" && -f "$STAMP" && "$(cat "$STAMP")" == "$REVISION" ]]; then
  echo "Greyhound worker already built at $REVISION: $OUT"
  exit 0
fi

CC="${CC:-cc}"
# Match greyhound-reference's Makefile: -march=native -O3 -flto, AVX-512 NTT.
CFLAGS=(-std=c2x -O3 -flto=auto -fwrapv -pthread -march=native -mtune=native -Wall)
SOURCES=(
  pack.c pack_wire.c greyhound.c greyhound_wire.c dachshund.c chihuahua.c
  labrador.c proof_wire.c witness_wire.c rice.c data.c jlproj.c polx.c poly.c
  polz.c sparsemat.c aesctr.c fips202.c randombytes.c cpucycles.c parallel.c
  ntt.S invntt.S
)

cd "$VENDOR"
"$CC" "${CFLAGS[@]}" \
  -I"$VENDOR" \
  "$ROOT/benchmarks/greyhound/src/lattice_eval.c" \
  "${SOURCES[@]}" \
  -lm \
  -o "$OUT"

printf '%s\n' "$REVISION" > "$STAMP"
echo "Greyhound worker: $OUT"
