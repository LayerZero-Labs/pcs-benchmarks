"""Reproduce component calculations; deliberately does not assign overall security."""
import csv
import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parent
Q = 2**50 - 2687
PHI = 128
MEAN_ATTEMPTS = 1.8801  # 10,000 deterministic sampler trials, local aarch64 build
# The pinned sampler reduces a uniform u64 modulo Q, rather than rejection sampling.
P_MAX = ((2**64 + Q - 1) // Q) / 2**64
profiles = {}
for name, index, height, width, ratio, projection_height, kind, rank in csv.reader(
    (ROOT / "profiles.csv").open()
):
    height, width, ratio, projection_height = map(int, (height, width, ratio, projection_height))
    row = dict(round=int(index), height=height, width=width, projection=kind, rank=int(rank))
    if kind in ("fine", "simple"):
        # Matches project_fine::sample_layers and its SimpleConfig special case.
        blocks = (height // ratio) * PHI // projection_height
        effective_width = 1 if kind == "simple" else width
        values = (blocks, projection_height, effective_width)
        assert all(v > 0 and v & (v - 1) == 0 for v in values)
        layers = sum(v.bit_length() - 1 for v in values)
        # Conditional on a fixed nonzero multilinear residual and independent XOF
        # outputs, each repetition fails with probability <= layers * P_MAX.
        row["batching_layers"] = layers
        row["two_batch_component_bound_bits"] = -2 * math.log2(layers * P_MAX)
    profiles.setdefault(name, []).append(row)

result = {
    "revision": "26d07c73c54872b9e8d2b3200117a6a0a21b10ee",
    "scope": "Component calculations only; no overall security estimate or attack claim",
    "field_bits": math.log2(Q),
    "quadratic_field_bits": 2 * math.log2(Q),
    "paper_noninvertibility_heuristic_bits": 2 * math.log2(Q) - math.log2(PHI // 2),
    "raw_fixed_weight_support_bits": math.log2(math.comb(PHI, 22)) + 22,
    "empirical_accepted_support_bits": math.log2(math.comb(PHI, 22)) + 22 - math.log2(MEAN_ATTEMPTS),
    "sampler_trials": 10000,
    "sampler_mean_attempts": MEAN_ATTEMPTS,
    "profiles": {},
}
for name, rows in profiles.items():
    result["profiles"][name] = {
        "overall_security_bits": None,
        "rounds": rows,
        "worst_two_batch_component_bound_bits": min(
            r["two_batch_component_bound_bits"] for r in rows if "two_batch_component_bound_bits" in r
        ),
    }
print(json.dumps(result, indent=2))
