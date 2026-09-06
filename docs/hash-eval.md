# Hash PCS evaluation

This experiment compares Akita, WHIR, and BaseFold on **dense** multilinear
openings. It is the source of `tab:eval-hash-time` and
`tab:eval-hash-resources`. Generated reports also record the host CPU,
whether AVX-512F was advertised/activated, and a GitHub commit URL per row.

## What is being compared

Payload is the target value of \(N \log_2 |\mathbb{F}|\) used in the lattice
table. All three schemes run at the **same coefficient count** as Akita
(\(\log_2 N =\) payload exponent \(- 5\)). Akita uses \(q = 2^{32}-99\).
WHIR and BaseFold use KoalaBear \(q = 2^{31}-2^{24}+1\) (~31-bit), so the
logical bit payload is slightly smaller than the 32-bit target; the dense
tables still have matching length.

The comparison uses a common **128-bit** transcript-error target:

- **WHIR:** Plonky3 `p3-whir`, `security_level=128`, rate \(1/2\), folding
  factor 4 after a first-round fold large enough that the FFT stays inside
  KoalaBear two-adicity 24. Later-round log-inverse rates are lowered just
  enough that `two_adic_generator` also stays inside that two-adicity.
  The worker searches grinding budgets 20–30 independently (KoalaBear cannot
  grind 31+ bits) and prefers capacity bound at rate \(1/2\). If that cannot
  meet 128 bits within the grind limit, it tries the Johnson bound, then
  unique decoding, and then rate \(1/4\).
- **BaseFold:** SP1 SLOP stacked BaseFold (`slop-basefold`), FRI
  `log_blowup=1`, 112 queries, 16 bits of grinding (conjectured soundness
  \(1\cdot 112+16=128\)), stacking height 20.

Akita uses the generated `fp32-dense` planner schedule for the requested
size, including the `nv=22` and `nv=24` rows installed by
`scripts/fetch-vendors.sh`.

Timing rows are collected at **1 and 8 threads**. A dash denotes an
unsupported parallel mode. Communication columns are independent of thread
count; peak RSS is reported for both.

Every worker is launched as a fresh process. The virtual-memory ceiling is
**90% of host RAM**. An OOM cell is recorded as `oom` / `\evaloom`.

Do not mix machines, ISAs, or silently remap sizes.

## Pins

| Implementation | Source | Revision |
| --- | --- | --- |
| Akita (`main` pin) | https://github.com/LayerZero-Labs/akita | [`f9f7de87`](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) |
| WHIR (`p3-whir`) | https://github.com/Plonky3/Plonky3 | [`9d496524`](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) |
| BaseFold (SLOP) | https://github.com/succinctlabs/sp1 | [`0f2a1e13`](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) |

WHIR and BaseFold are isolated Cargo trees (`benchmarks/whir`,
`benchmarks/basefold`) so they do not unify with the lattice workspace.
Cargo fetches the pinned git revisions on first build.

## Machine requirements

Headline numbers should be gathered on the same Linux x86_64 AVX-512 host as
the lattice table, with `RUSTFLAGS=-C target-cpu=native`.

## Commands

```bash
source "$HOME/.cargo/env"   # if cargo is not on PATH
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export RUSTFLAGS="-C target-cpu=native"

./scripts/fetch-vendors.sh --akita

cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval matrix

# One scheme, one payload, one thread count, one measured sample
./scripts/hash-eval.sh run --scheme akita --payload 31 --threads 1 --runs 1 --warmups 0

# Full 30-cell table (hours, 90% of host RAM cap)
./scripts/hash-eval.sh run --out results/hash-x86_64

cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval compare \
  results/hash-x86_64 --out-dir results/hash-x86_64
```

Each run writes `records.jsonl`, `table.md`, `table.tex`, `table-resources.md`,
`table-resources.tex`, `report.md`, `report.tex`, and `provenance.txt`.
WHIR unique-decoding rows (`log₂ N` 28 and 30 on this matrix) are marked
with a table footnote rather than left as unmarked numbers.
