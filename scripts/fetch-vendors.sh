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

if [[ "$FETCH_GREYHOUND" -eq 1 ]]; then
  clone_pin \
    https://github.com/LayerZero-Labs/greyhound-reference.git \
    "$ROOT/third_party/greyhound-reference" \
    672e74100496f6ef698ba35e241cf7593e3d57af
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
    c0cb822f28b7b9efe85b1924b029d36e13cdf516
fi

echo "Vendors pinned under $ROOT/third_party"
