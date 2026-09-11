#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mkdir -p "$ROOT/third_party"

FETCH_AKITA=1
FETCH_GREYHOUND=1
FETCH_ROKOKO=1
case "${1:-}" in
  "")
    ;;
  --akita)
    FETCH_GREYHOUND=0
    FETCH_ROKOKO=0
    ;;
  --greyhound)
    FETCH_AKITA=0
    FETCH_ROKOKO=0
    ;;
  --rokoko)
    FETCH_AKITA=0
    FETCH_GREYHOUND=0
    ;;
  *)
    echo "usage: $0 [--akita|--greyhound|--rokoko]" >&2
    exit 2
    ;;
esac

clone_pin() {
  local url="$1"
  local dest="$2"
  local rev="$3"
  if [[ ! -d "$dest/.git" ]]; then
    git clone --filter=blob:none "$url" "$dest"
  fi
  git -C "$dest" fetch --filter=blob:none origin "$rev"
  git -C "$dest" checkout --detach "$rev"
}

# Greyhound's SIMDe backend is a git submodule. Other vendors either have no
# submodules or pin them over SSH remotes that this host cannot fetch.
init_greyhound_submodules() {
  local dest="$1"
  if [[ -f "$dest/.gitmodules" ]]; then
    git -C "$dest" submodule update --init --recursive
  fi
}

install_fp32_dense_catalog() {
  local dest="$1"
  local overlay="$2"
  local generated="$dest/crates/akita-schedules/src/generated/fp32_dense.rs"
  if [[ ! -f "$generated" ]]; then
    echo "error: missing $generated" >&2
    exit 2
  fi
  if [[ -f "$overlay" ]]; then
    cp "$overlay" "$generated"
    echo "installed $(basename "$overlay") into $generated"
  else
    "$ROOT/scripts/extend-akita-fp32-dense.sh" "$dest"
  fi
}

install_schedule_overlay() {
  local dest="$1"
  local rel="$2"
  local overlay="$3"
  local generated="$dest/$rel"
  if [[ ! -f "$generated" ]]; then
    echo "error: missing $generated" >&2
    exit 2
  fi
  if [[ -f "$overlay" ]]; then
    cp "$overlay" "$generated"
    echo "installed $(basename "$overlay") into $generated"
  else
    echo "warning: missing $overlay; leaving upstream $(basename "$generated") (hash-eval fp64/fp128 rows need the extend scripts)" >&2
  fi
}

if [[ "$FETCH_GREYHOUND" -eq 1 ]]; then
  clone_pin \
    https://github.com/LayerZero-Labs/greyhound-reference.git \
    "$ROOT/third_party/greyhound-reference" \
    687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397
  init_greyhound_submodules "$ROOT/third_party/greyhound-reference"
fi

if [[ "$FETCH_ROKOKO" -eq 1 ]]; then
  clone_pin \
    https://github.com/lattice-arguments/rokoko.git \
    "$ROOT/third_party/rokoko" \
    26d07c73c54872b9e8d2b3200117a6a0a21b10ee
  python3 "$ROOT/scripts/patch-rokoko-resources.py" "$ROOT/third_party/rokoko"
fi

if [[ "$FETCH_AKITA" -eq 1 ]]; then
  clone_pin \
    https://github.com/LayerZero-Labs/akita.git \
    "$ROOT/third_party/akita" \
    d1b224d809c7edc357b0dbab0f607e19b475910b
  install_fp32_dense_catalog \
    "$ROOT/third_party/akita" \
    "$ROOT/vendor/akita-catalogs/fp32_dense-main.rs"
  python3 "$ROOT/scripts/patch-akita-fp32-dense-offload.py" "$ROOT/third_party/akita"
  RECURSIVE_OVERLAY="$ROOT/vendor/akita-catalogs/fp32_dense_recursive-main.rs"
  if [[ -f "$RECURSIVE_OVERLAY" ]]; then
    cp "$RECURSIVE_OVERLAY" \
      "$ROOT/third_party/akita/crates/akita-schedules/src/generated/fp32_dense_recursive.rs"
    echo "installed $(basename "$RECURSIVE_OVERLAY") into Akita fp32_dense_recursive catalog"
  fi
  install_schedule_overlay \
    "$ROOT/third_party/akita" \
    crates/akita-schedules/src/generated/fp64_dense.rs \
    "$ROOT/vendor/akita-catalogs/fp64_dense-main.rs"
  install_schedule_overlay \
    "$ROOT/third_party/akita" \
    crates/akita-schedules/src/generated/fp128_dense.rs \
    "$ROOT/vendor/akita-catalogs/fp128_dense-main.rs"
fi

echo "Vendors pinned under $ROOT/third_party"
