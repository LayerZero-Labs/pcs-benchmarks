#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mkdir -p "$ROOT/third_party"

AKITA_ONLY=0
if [[ "${1:-}" == "--akita" ]]; then
  AKITA_ONLY=1
fi

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

if [[ "$AKITA_ONLY" -eq 0 ]]; then
  clone_pin \
    https://github.com/LayerZero-Labs/greyhound-reference.git \
    "$ROOT/third_party/greyhound-reference" \
    687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397
  init_greyhound_submodules "$ROOT/third_party/greyhound-reference"

  clone_pin \
    https://github.com/lattice-arguments/rokoko.git \
    "$ROOT/third_party/rokoko" \
    1baa91e901fc37b5fa59e65c26a630cb93849b3e
  python3 "$ROOT/scripts/patch-rokoko-resources.py" "$ROOT/third_party/rokoko"
fi

clone_pin \
  https://github.com/LayerZero-Labs/akita.git \
  "$ROOT/third_party/akita" \
  d1b224d809c7edc357b0dbab0f607e19b475910b
install_fp32_dense_catalog \
  "$ROOT/third_party/akita" \
  "$ROOT/vendor/akita-catalogs/fp32_dense-main.rs"

echo "Vendors pinned under $ROOT/third_party"
