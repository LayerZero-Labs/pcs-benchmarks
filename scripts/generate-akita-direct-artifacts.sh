#!/usr/bin/env bash
# Generate direct rows missing from the pinned upstream catalogs, without
# modifying the upstream checkout. Generated artifacts are committed here.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AKITA="$ROOT/third_party/akita"
PIN=c0cb822f28b7b9efe85b1924b029d36e13cdf516
[[ "$(git -C "$AKITA" rev-parse HEAD)" == "$PIN" ]]
[[ -z "$(git -C "$AKITA" status --porcelain)" ]]
export CARGO_TARGET_DIR="$ROOT/target/akita-planner"
cargo build --release --locked --manifest-path "$AKITA/Cargo.toml" \
  -p akita-planner --features catalog-gen --bin gen_schedule_artifacts
for row in fp32:22 fp32:24 fp64:21 fp64:23 fp64:25 fp64:27 fp128:20 fp128:22; do
  field="${row%:*}"
  nv="${row#*:}"
  "$CARGO_TARGET_DIR/release/gen_schedule_artifacts" \
    "$ROOT/vendor/akita-catalogs/c0cb822f/$field-nv$nv" \
    --row-progress --final-group "${field}_dense:$nv:1"
done
