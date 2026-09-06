#!/usr/bin/env bash
# Add nv=22 and nv=24 to a cloned Akita fp32-dense catalog using that
# revision's planner. The upstream pins ship only nv ∈ {20, 26, 28, 30}.
set -euo pipefail

DEST="$(cd "${1:?usage: extend-akita-fp32-dense.sh <akita-checkout>}" && pwd)"

if [[ ! -d "$DEST/.git" ]]; then
  echo "error: $DEST is not a git checkout" >&2
  exit 2
fi

python3 - "$DEST" <<'PY'
from pathlib import Path
import sys

root = Path(sys.argv[1])
path = root / "crates/akita-planner/src/generated_families.rs"
text = path.read_text()
old = """const FP32_DENSE_KEYS: &[PolynomialGroupLayout] = &[
    PolynomialGroupLayout::singleton(20),
    PolynomialGroupLayout::singleton(26),
    PolynomialGroupLayout::singleton(28),
    PolynomialGroupLayout::singleton(30),
];"""
new = """const FP32_DENSE_KEYS: &[PolynomialGroupLayout] = &[
    PolynomialGroupLayout::singleton(20),
    PolynomialGroupLayout::singleton(22),
    PolynomialGroupLayout::singleton(24),
    PolynomialGroupLayout::singleton(26),
    PolynomialGroupLayout::singleton(28),
    PolynomialGroupLayout::singleton(30),
];"""
if "PolynomialGroupLayout::singleton(22)" in text and "FP32_DENSE_KEYS" in text:
    print(f"{path}: nv=22 already listed")
elif old not in text:
    sys.exit(f"{path}: expected FP32_DENSE_KEYS block not found")
else:
    path.write_text(text.replace(old, new, 1))
    print(f"{path}: added nv=22 and nv=24")
PY

cd "$DEST"
"$DEST/scripts/generate-schedule-tables.sh" --row-progress fp32_dense
