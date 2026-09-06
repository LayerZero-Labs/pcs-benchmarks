# Contributing

Keep changes reviewable and benchmark claims reproducible.

Before opening a pull request:

```bash
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

For lattice-table changes, include the exact command used, explain any workload or
security-assumption change, and keep `results/lattice-x86_64/` in sync with the
JSONL that produced the report. Never update a PCS revision in the same commit
as a harness behavior change: separate commits make performance changes
attributable.

New scheme adapters must pin dependencies, use the shared workload vocabulary,
test successful and failing verification, and document deviations from
`docs/methodology.md`.

