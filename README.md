# PCS Benchmarks

Reproducible benchmarks for lattice- and hash-based polynomial commitment
schemes. The harness records explicit workload, build, machine, timing,
communication, memory, and correctness metadata for every process.

The repository supports one strict result contract. Canonical measurements are
written to:

- `results/lattice-x86_64/`
- `results/hash-x86_64/`

Results from another schema, machine cohort, seed schedule, build, or workload
configuration are rejected rather than silently combined.

## Benchmark families

The lattice survey covers Akita, Greyhound, and RoKoKo. The hash survey covers
Akita, Plonky2 FRI, Plonky3 FRI/STIR/WHIR, Binius64 BaseFold, Flock Ligerito,
WHIR through the ProveKit adapter, and SP1 BaseFold.

These are native-configuration surveys, not unconditional scheme rankings.
Reports expose security targets, statement types, input distributions, packing,
thread counts, setup models, and excluded verifier context.

See:

- [Benchmark methodology](docs/methodology.md)
- [Lattice evaluation](docs/lattice-eval.md)
- [Hash evaluation](docs/hash-eval.md)

## Reproducibility contract

- Workers are built once from checked-in lockfiles, then invoked directly.
- A successful sample requires a successful exit and one valid worker response.
- Mixed machines, builds, parameters, seed modes, or source identities cannot
  enter one aggregate.
- Inputs default to deterministic per-sample seeds; `--seed-mode fixed`
  separately measures machine/runtime noise.
- Timers include useful claim preparation and complete transcript work.
- Communication reports commitment, separately transmitted evaluation, proof,
  total sent, and excluded public context.
- Peak RSS is whole-process `VmHWM`. The `ulimit -v` ceiling is an address-space
  limit, not an RSS measurement.
- Correctness-only negative checks run with
  `PCS_BENCH_NEGATIVE_CHECK=1` and cannot be persisted as performance runs.

## Requirements

- Rust 1.95 for infrastructure and Rust workers
- `nightly-2026-09-03` for RoKoKo
- Linux x86_64 with AVX-512F for headline Greyhound and matched headline runs
- `RUSTFLAGS="-C target-cpu=native"` on the controlled benchmark host

Fetch pinned sources:

```bash
./scripts/fetch-vendors.sh
```

## Run

Inspect the planned matrices:

```bash
cargo run -p pcs-bench-runner --bin pcs-bench -- lattice-eval matrix
cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval matrix
```

Collect the canonical result sets:

```bash
export RUSTFLAGS="-C target-cpu=native"
./scripts/lattice-eval.sh run --out results/lattice-x86_64
./scripts/hash-eval.sh run --out results/hash-x86_64
```

Each command defaults to one warmup and ten measured processes per cell.

Regenerate reports from current records:

```bash
cargo run -p pcs-bench-runner --bin pcs-bench -- lattice-eval compare \
  results/lattice-x86_64 --out-dir results/lattice-x86_64
cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval compare \
  results/hash-x86_64 --out-dir results/hash-x86_64
```

## Development checks

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

CI separately checks every isolated Rust adapter, the pinned RoKoKo build, and
the Greyhound C adapter source. Full measurements run only on the controlled
x86 benchmark host.
