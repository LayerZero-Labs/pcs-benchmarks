# Hash PCS evaluation

This experiment compares Akita with the hash-based PCS roster on **dense**
payloads \(2^{27}\) through \(2^{35}\) bits. It is the source of
`tab:eval-hash-time` and `tab:eval-hash-resources`. Generated reports also
record the host CPU, whether AVX-512F was advertised/activated, and a GitHub
commit URL per row.

## What is being compared

Payload is the target value of \(N \log_2 |\mathbb{F}|\) used in the lattice
table. Schemes do **not** share one coefficient width:

| Scheme | Field | \(\log_2 N\) vs payload |
| --- | --- | --- |
| Akita | \(q=2^{32}-99\) | payload \(- 5\) |
| Plonky2 FRI | Goldilocks | payload \(- 6\) |
| Plonky3 FRI / STIR / WHIR | KoalaBear | payload \(- 5\) |
| Binius64 BaseFold | \(\mathbb F_{2^{128}}\) | payload \(- 7\) |
| Flock Ligerito | bits packed in \(\mathbb F_{2^{128}}\) | payload (\(m\)) |
| WHIR (ProveKit) | Goldilocks coeffs, deg-3 challenges | payload \(- 6\) |
| BaseFold (SP1) | KoalaBear | payload \(- 5\) |

Each scheme keeps its **native** security target, hash, and rate. Cells are
not \(\lambda\)-comparable.

- **Akita:** generated `fp32-dense` planner schedule, including `nv=22` and
  `nv=24` from `scripts/fetch-vendors.sh`.
- **Plonky2 FRI:** `elliottech/plonky2`, univariate Goldilocks, rate \(1/8\),
  28 queries, 16-bit PoW, Poseidon2, native 100-bit target. The LDE has
  length \(2^{n+3}\). Payload \(2^{33}\) (\(\log_2 N=27\)) and \(2^{35}\)
  (\(\log_2 N=29\)) OOM under the 90% RAM cap.
- **Plonky3 FRI:** univariate KoalaBear, rate \(1/2\), 80 queries, 20-bit
  grind, Poseidon2, native 100-bit conjectural/capacity. KoalaBear
  two-adicity 24: a rate-\(1/2\) univariate of \(\log_2 N>23\) is packed
  into height \(2^{23}\) and width \(2^{n-23}\) (footnote).
- **Plonky3 STIR:** same pin and packing as FRI, rate \(1/2\), \(\le 20\)
  work bits, Poseidon2.
- **WHIR (Plonky3):** `p3-whir`, `security_level=128`, rate \(1/2\), folding
  factor 4 after a first-round fold large enough that the FFT stays inside
  KoalaBear two-adicity 24. Later-round log-inverse rates are lowered just
  enough that `two_adic_generator` also stays inside that two-adicity.
  The worker searches grinding budgets 20–30 independently (KoalaBear cannot
  grind 31+ bits) and prefers capacity bound at rate \(1/2\). If that cannot
  meet 128 bits within the grind limit, it tries the Johnson bound, then
  unique decoding, and then rate \(1/4\).
- **Binius64 BaseFold:** \(\mathbb F_{2^{128}}\), rate \(1/2\), SHA-256,
  unique decoding at 100 bits (not the 96-bit product default). The
  commitment is the SHA-256 Merkle root (32 bytes) written at commit time;
  proof bytes are the rest of the Fiat–Shamir transcript.
- **Flock Ligerito:** native Fast profile (`mXX_fast.toml`), SHA-256,
  Johnson+OOD. `m` is the bit-variable count; packed length is \(2^{m-7}\).
- **WHIR (ProveKit):** `worldfnd/whir` Goldilocks3 (`Basefield<Field64_3>`),
  Johnson bound, rate \(1/4\), fold 8, SHA-256, 133-bit internal target.
  The commit-phase narg is a SHA-256 Merkle root (32 bytes) plus one
  Goldilocks3 OOD evaluation (24 bytes). Proof bytes are the remaining
  narg string plus Merkle-path hints.
- **BaseFold (SP1):** SLOP stacked BaseFold, FRI `log_blowup=1`, 112 queries,
  16 bits of grinding (conjectured soundness \(1\cdot 112+16=128\)),
  stacking height 20.

Timing rows are collected at **1 and 8 threads**. A dash denotes an
unsupported parallel mode. Communication columns are independent of thread
count; peak RSS is reported for both.

Every worker is launched as a fresh process. The virtual-memory ceiling is
**90% of host RAM**. An OOM cell is recorded as `oom` / `\evaloom`.
Unmeasured roster cells are `pending` / `\evalpending`.

Do not mix machines, ISAs, or silently remap sizes.

## Pins

| Implementation | Source | Revision |
| --- | --- | --- |
| Akita (`main` pin) | https://github.com/LayerZero-Labs/akita | [`d1b224d8`](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) |
| Plonky2 FRI | https://github.com/elliottech/plonky2 | [`e1c2d354`](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) |
| Plonky3 FRI / STIR | https://github.com/Plonky3/Plonky3 | [`3da160d0`](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) |
| WHIR (`p3-whir`) | https://github.com/Plonky3/Plonky3 | [`9d496524`](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) |
| Binius64 BaseFold | https://github.com/binius-zk/binius64 | [`6e75a2d1`](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) |
| Flock Ligerito | https://github.com/succinctlabs/flock | [`43f0eee0`](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) |
| WHIR (ProveKit) | https://github.com/worldfnd/ProveKit | [`6481f961`](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9) |
| whir crate | https://github.com/worldfnd/whir | [`8804e80e`](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d) |
| BaseFold (SLOP) | https://github.com/succinctlabs/sp1 | [`0f2a1e13`](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) |

Hash-eval adapters are isolated Cargo trees under `benchmarks/` so they do
not unify with the lattice workspace. Cargo fetches the pinned git revisions
on first build. The Plonky2 worker sets `RUSTC_BOOTSTRAP=1` because
`elliottech/plonky2` uses `#![feature(specialization)]` on stable 1.95.

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

# Full 90-cell table (hours, 90% of host RAM cap)
./scripts/hash-eval.sh run --out results/hash-x86_64

cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval compare \
  results/hash-x86_64 --out-dir results/hash-x86_64
```

Each run writes `records.jsonl`, `table.md`, `table.tex`, `table-resources.md`,
`table-resources.tex`, `report.md`, `report.tex`, and `provenance.txt`.
WHIR unique-decoding rows (`log₂ N` 28 and 30 on this matrix) and packed
Plonky3 univariate rows (`log₂ N>23`) are marked with table footnotes.
Unmeasured roster rows stay `pending` until a worker records them.
