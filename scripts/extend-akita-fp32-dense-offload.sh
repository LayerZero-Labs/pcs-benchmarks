#!/usr/bin/env bash
# Generate the fp32-dense-recursive (setup-offload) catalog for lattice-eval.
#
# Usage: extend-akita-fp32-dense-offload.sh <akita-checkout>
# Thin wrapper kept for the lattice-eval setup instructions; the generalised
# form is extend-akita-dense-offload.sh <akita-checkout> [fp32|fp64|fp128].
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec "$ROOT/scripts/extend-akita-dense-offload.sh" \
  "${1:?usage: extend-akita-fp32-dense-offload.sh <akita-checkout>}" fp32
