# Lattice PCS evaluation

This experiment compares Akita, Greyhound, and RoKoKo on **dense** polynomial
openings. It is the source of `tab:eval-lattice-time` and
`tab:eval-lattice-resources`. Generated reports also record the host CPU,
whether AVX-512F was advertised/activated, and a GitHub commit URL per row.

## What is being compared

Payload is the target value of \(N \log_2 |\mathbb{F}|\). Akita and Greyhound
use \(q = 2^{32}-99\), so payload \(2^{k}\) means \(\log_2 N = k-5\). RoKoKo
uses \(q = 2^{50}-2687\) and only ships native sets `p-26`, `p-28`, and `p-30`.
Those rows are the closest supported instances, not bit-identical payloads:
they contain about \(25/16\) times as many logical bits as the 32-bit target
because the native field is ~50 bits rather than 32.

| Payload | Akita / Greyhound \(\log_2 N\) | RoKoKo |
| ---: | ---: | --- |
| \(2^{27}\) | 22 | unsupported |
| \(2^{29}\) | 24 | unsupported |
| \(2^{31}\) | 26 | `p-26` (\(\log_2 N = 26\)) |
| \(2^{33}\) | 28 | `p-28` (\(\log_2 N = 28\)) |
| \(2^{35}\) | 30 | `p-30` (\(\log_2 N = 30\)) |

The comparison is **single-threaded**. RoKoKo has no native multithreaded
prover. Greyhound's reference can parallelize extension products; this harness
sets `LATTICE_DOGS_THREADS=1` so the ratio is not a parallel-scaling artifact.
Every worker is launched as a fresh process. The
virtual-memory ceiling is **90% of host RAM** (`scripts/with-memlimit.sh` /
`ulimit -v`), so a 121~GiB machine grants about 109~GiB to each worker. An
OOM cell is recorded as `oom` / `\evaloom` and is never treated as a timing.
Greyhound uses the `l2-quantum128-adps16` Euclidean SIS policy (128-bit
quantum ADPS16 core-SVP). Proof sizes are the contextual wire encoding:
public `u1` and the fold schedule are verifier context. If Greyhound still
cannot secure the Ajtai commitments, the cell is `err` with a footnote; that
is not OOM. At \(\log_2 N=30\) on this machine the 128-bit policy grows ranks
until the worker exceeds the 109~GiB cap, so that cell is OOM.

Akita uses the generated `fp32-dense` planner schedule for the requested size.
The catalogs shipped in the pinned Akita revisions have production rows at
`nv ∈ {20, 26, 28, 30}`. This harness adds `nv=22` and `nv=24` by running that
same revision's planner (`scripts/extend-akita-fp32-dense.sh`) and installing
the resulting tables from `vendor/akita-catalogs/`. Nearby sizes are never
substituted.

## Pins

| Implementation | Source | Revision |
| --- | --- | --- |
| Akita (`main` pin) | https://github.com/LayerZero-Labs/akita | [`f9f7de87`](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) |
| Akita (PR #466) | https://github.com/LayerZero-Labs/akita/pull/466 | [`bb68275e`](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) |
| Greyhound | https://github.com/LayerZero-Labs/greyhound-reference | [`687a6f8b`](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397) |
| RoKoKo | https://github.com/lattice-arguments/rokoko | [`1baa91e9`](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) |

## Machine requirements

- **Akita:** Rust 1.95, any host that can build the crate graph.
- **Greyhound:** Linux x86_64 with AVX-512F for headline numbers (`BACKEND=auto`
  uses the upstream AVX-512 NTT). The reference also has a portable backend;
  do not mix that ISA into this table. Apple Silicon cannot produce headline
  Greyhound rows.
- **RoKoKo:** `rustup` nightly, and the `incomplete-rexl` backend (portable).
  Headline numbers should still be gathered on the same AVX-512 Linux box as
  Greyhound. Set `MIMALLOC_PURGE_DELAY=-1` (the wrapper does this).
  `fetch-vendors.sh` patches the pinned executor so it prints the inner
  commitment size, expanded CRS resident size, and `/proc/self/status` peak RSS.

Do not mix machines, ISAs, or thread counts when filling the table.

## Commands

The generated `report.md` / `report.tex` include this sequence. Headline numbers
checked in at [`results/lattice-x86_64/`](../results/lattice-x86_64/report.md) used
1 warmup + 3 measured samples, `RUSTFLAGS=-C target-cpu=native`, and
`RAYON_NUM_THREADS=1` on a Linux x86_64 AVX-512 host.

```bash
source "$HOME/.cargo/env"   # if cargo is not on PATH
rustup toolchain install nightly -c rustc,cargo   # once, for RoKoKo
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export RUSTFLAGS="-C target-cpu=native"
export RAYON_NUM_THREADS=1

./scripts/fetch-vendors.sh          # Greyhound, RoKoKo, Akita pins + nv=22/24 catalogs
./scripts/build-greyhound.sh        # AVX-512 host only; -march=native

# Planned matrix (no execution): unsupported cells, RoKoKo native params
cargo run -p pcs-bench-runner --bin pcs-bench -- lattice-eval matrix

# One scheme, one payload, one measured sample (must verify, status ok)
./scripts/lattice-eval.sh run --scheme akita --payload 31 --runs 1 --warmups 0

# Full 20-cell table (hours, 90% of host RAM cap, AVX-512 Linux x86_64)
./scripts/lattice-eval.sh run --out results/lattice-x86_64

# Rebuild Markdown + LaTeX from the JSONL in that directory
cargo run -p pcs-bench-runner --bin pcs-bench -- lattice-eval compare \
  results/lattice-x86_64 --out-dir results/lattice-x86_64
```

Each run writes `records.jsonl`, `table.md`, `table.tex`, `report.md`,
`report.tex`, and `provenance.txt`. The report includes the host CPU, AVX-512
status, commit hyperlinks, both paper tables, median ± sample stdev, and the
commands above.

Warmup processes are stored with `warmup: true` and are excluded from the
median. Released but non-normalized measurements must be stored with
`historical: true` and are never used for headline ratios.
