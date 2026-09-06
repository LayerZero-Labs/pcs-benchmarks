Measurements were collected on a single AMD Ryzen 9 9950X 16-Core Processor (Linux x86_64, 32 logical CPUs, 121~GiB RAM). AVX-512F is advertised by the CPU and was activated for this run (Greyhound `-march=native`; Akita and RoKoKo `-C target-cpu=native`). Compiler flags: `-C target-cpu=native`.

Our first experiment compares Akita with prior lattice-based PCSs on dense
polynomial data at the target volumes above. For each input, Akita
uses the validated planner schedule selected for that field and size.
The pinned catalogs omit $n_v=22$ and $n_v=24$; those rows are generated
with that same planner at the measured commit.
The comparison is exclusively single-threaded: RoKoKo has no native multithreaded
prover, and Greyhound is pinned to `LATTICE_DOGS_THREADS=1` even though the
reference can parallelize extension products. Greyhound uses the
`l2-quantum128-adps16` Euclidean SIS policy and reports contextual proof bytes.
Timing cells report median ± sample standard
deviation across fresh processes after warmup. Scheme names link to the exact
git commit that was measured. Akita is reported both at the pinned `main` commit
and at the tip of [PR #466](https://github.com/LayerZero-Labs/akita/pull/466).

The timing comparison separates commitment, opening, and verification, while the
resources table reports communication, memory, and preprocessing. Released but
non-normalized measurements are labeled historical when retained and are never used
to form headline ratios.

RoKoKo uses the field $\mathbb{F}_{2^{50}-2687}$ and fixed native parameter
sets, so we report its closest supported input at each target payload. An OOM entry
exceeds the 109 GiB worker memory limit (90% of host RAM). The populated RoKoKo rows contain approximately
$25/16$ times the target number of logical bits, since their native field has about
50 bits rather than 32. They report the cost of those native instances.

| Payload | Scheme | Field | log₂ N | Commit (s) | Open (s) | Total (s) | Verify (ms) |
| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | $2^{32}-99$ | 22 | 0.099 ± 0.0002 | 0.959 ± 0.0009 | 1.06 ± 0.0007 | 7.2 ± 0.01 |
| 2^{27} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | $2^{32}-99$ | 22 | 0.099 ± 0.0008 | 0.995 ± 0.0008 | 1.09 ± 0.0013 | 7.5 ± 0.00 |
| 2^{27} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397) | $2^{32}-99$ | 22 | 0.110 ± 0.0011 | 0.168 ± 0.0027 | 0.278 ± 0.0039 | 74.3 ± 0.87 |
| 2^{27} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | $2^{50}-2687$ | —(1) | —(1) | —(1) | —(1) | —(1) |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | $2^{32}-99$ | 24 | 0.345 ± 0.0004 | 1.43 ± 0.015 | 1.77 ± 0.015 | 11.6 ± 0.29 |
| 2^{29} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | $2^{32}-99$ | 24 | 0.346 ± 0.0007 | 1.52 ± 0.0021 | 1.87 ± 0.0026 | 9.3 ± 0.04 |
| 2^{29} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397) | $2^{32}-99$ | 24 | 0.435 ± 0.0016 | 0.337 ± 0.013 | 0.772 ± 0.015 | 149 ± 0.42 |
| 2^{29} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | $2^{50}-2687$ | —(1) | —(1) | —(1) | —(1) | —(1) |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | $2^{32}-99$ | 26 | 1.29 ± 0.0036 | 2.57 ± 0.0052 | 3.85 ± 0.0087 | 13.9 ± 0.10 |
| 2^{31} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | $2^{32}-99$ | 26 | 1.28 ± 0.0063 | 2.62 ± 0.018 | 3.90 ± 0.012 | 12.7 ± 0.53 |
| 2^{31} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397) | $2^{32}-99$ | 26 | 2.27 ± 0.0077 | 0.881 ± 0.0019 | 3.15 ± 0.0059 | 325 ± 0.21 |
| 2^{31} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | $2^{50}-2687$ | 26 | 0.876 ± 0.015 | 0.743 ± 0.0055 | 1.62 ± 0.015 | 4.8 ± 0.08 |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | $2^{32}-99$ | 28 | 6.06 ± 0.023 | 6.78 ± 0.0063 | 12.8 ± 0.027 | 19.7 ± 0.51 |
| 2^{33} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | $2^{32}-99$ | 28 | 6.05 ± 0.021 | 6.65 ± 0.027 | 12.7 ± 0.040 | 21.6 ± 0.47 |
| 2^{33} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397) | $2^{32}-99$ | 28 | 11.3 ± 0.031 | 3.43 ± 0.087 | 14.8 ± 0.065 | 643 ± 1.33 |
| 2^{33} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | $2^{50}-2687$ | 28 | 3.50 ± 0.012 | 1.59 ± 0.011 | 5.10 ± 0.0052 | 4.9 ± 0.13 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | $2^{32}-99$ | 30 | 24.4 ± 0.075 | 16.9 ± 0.0060 | 41.2 ± 0.075 | 33.0 ± 0.16 |
| 2^{35} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | $2^{32}-99$ | 30 | 24.6 ± 0.065 | 16.0 ± 0.0045 | 40.6 ± 0.063 | 33.2 ± 0.11 |
| 2^{35} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397) | $2^{32}-99$ | 30 | OOM | OOM | OOM | OOM |
| 2^{35} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | $2^{50}-2687$ | 30 | 18.0 ± 0.252 | 4.94 ± 0.028 | 23.0 ± 0.250 | 7.6 ± 0.33 |

**(1)** RoKoKo ships only native sets `p-26`, `p-28`, and `p-30`; no instance matches this payload.


| Payload | Scheme | Commitment (B) | Proof (B) | Total (B) | Peak RSS (GiB) | Prep. (s) | State (GiB) |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | 343 | 65283 | 65626 | 0.190 | 0.0067 | 0.0017 |
| 2^{27} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | 343 | 61299 | 61642 | 0.187 | 0.0080 | 0.0020 |
| 2^{27} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397) | 2048 | 59284 | 61332 | 0.329 | 0 | 0.0000 |
| 2^{27} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | 343 | 66052 | 66395 | 0.632 | 0.0082 | 0.0020 |
| 2^{29} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | 343 | 61763 | 62106 | 0.633 | 0.0081 | 0.0020 |
| 2^{29} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397) | 2304 | 59089 | 61393 | 1.08 | 0 | 0.0000 |
| 2^{29} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | 343 | 66884 | 67227 | 1.93 | 0.0186 | 0.0049 |
| 2^{31} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | 343 | 63081 | 63424 | 1.93 | 0.0190 | 0.0049 |
| 2^{31} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397) | 2304 | 64600 | 66904 | 4.66 | 0 | 0.0000 |
| 2^{31} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | 774 | 114830 | 115604 | 4.07 | 0.335 | 1.59 |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | 343 | 68143 | 68486 | 6.38 | 0.0406 | 0.0107 |
| 2^{33} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | 343 | 64487 | 64830 | 6.36 | 0.0376 | 0.0098 |
| 2^{33} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397) | 2304 | 64525 | 66829 | 20.7 | 0 | 0.0000 |
| 2^{33} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | 773 | 114910 | 115683 | 10.9 | 0.698 | 3.19 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | 343 | 69264 | 69607 | 24.7 | 0.0812 | 0.0215 |
| 2^{35} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | 343 | 64605 | 64948 | 24.7 | 0.0743 | 0.0195 |
| 2^{35} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397) | OOM | OOM | OOM | OOM | OOM | OOM |
| 2^{35} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | 777 | 115056 | 115833 | 35.6 | 1.65 | 7.44 |

**(1)** RoKoKo ships only native sets `p-26`, `p-28`, and `p-30`; no instance matches this payload.


### Measured commits

- Akita [`f9f7de87`](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53)
- Akita (#466) [`bb68275e`](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740)
- Greyhound [`687a6f8b`](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397)
- RoKoKo [`1baa91e9`](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e)
- Akita PR: <https://github.com/LayerZero-Labs/akita/pull/466>


### Commands used for these numbers

These tables were produced on a Linux **x86_64** AVX-512 host (AMD Ryzen 9 9950X)
from this repository. The toolchain pin is Rust **1.95** (`rust-toolchain.toml`).
RoKoKo uses `rustup` **nightly**. Every timed worker is a fresh process wrapped
in `scripts/with-memlimit.sh` at 109~GiB (`ulimit -v`, 90% of host RAM) with `RAYON_NUM_THREADS=1`.
The runner defaults are **1 warmup + 3 measured** samples per cell; warmup rows
are stored with `warmup: true` and excluded from the median. Greyhound is
`LayerZero-Labs/greyhound-reference`, built with `-march=native -O3 -flto`,
and run with `LATTICE_DOGS_THREADS=1` and `LABRADOR_SIS_SECURITY=l2-quantum128-adps16`.
Proof sizes are contextual wire bytes. Akita and RoKoKo inherit `RUSTFLAGS=-C target-cpu=native`.
Akita PR #466 is a separate Cargo tree (`benchmarks/akita-pr466`,
`CARGO_TARGET_DIR=target/akita-pr466`) so it does not unify with the pinned
`main` revision. `./scripts/fetch-vendors.sh` clones both Akita pins,
installs planner-generated `fp32-dense` rows for `nv=22` and `nv=24`, and
patches RoKoKo so the executor prints commitment, CRS, and peak RSS.

Non-interactive shells may not put Cargo on `PATH`; `source ~/.cargo/env`
is required in that case. `CARGO_NET_GIT_FETCH_WITH_CLI=true` avoids libgit2 auth
failures when fetching the pinned git dependencies.
Checked-in numbers live in `results/lattice-x86_64/`.

```bash
# On an AVX-512 Linux x86_64 host
source "$HOME/.cargo/env"   # if cargo is not on PATH
cd /path/to/akita-benchmark

rustup toolchain install nightly -c rustc,cargo   # once, for RoKoKo
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export RUSTFLAGS="-C target-cpu=native"
export RAYON_NUM_THREADS=1

./scripts/fetch-vendors.sh          # Greyhound, RoKoKo, Akita pins + nv=22/24 catalogs
./scripts/build-greyhound.sh

# Full 20-cell matrix (Akita main, Akita #466, Greyhound, RoKoKo)
./scripts/lattice-eval.sh run --out results/lattice-x86_64

# Rebuild Markdown + LaTeX from the JSONL already in that directory
cargo run -p pcs-bench-runner --bin pcs-bench -- lattice-eval compare \
  results/lattice-x86_64 --out-dir results/lattice-x86_64
```

**Sanity-check the harness before trusting a full run.** `lattice-eval matrix`
prints the 20-cell plan (unsupported RoKoKo sizes, Akita/Greyhound `log2 N`,
RoKoKo `p-26`/`p-28`/`p-30`). A single supported cell should verify and emit
JSON with `status: ok`. Unit tests cover the RoKoKo log parser, OOM
classification, and table tokens. Each sample the runner launches is equivalent
to the worker commands below (still under the 90%-of-RAM cap).

```bash
export RUSTFLAGS="-C target-cpu=native"
export RAYON_NUM_THREADS=1

cargo test --workspace --locked
cargo run -p pcs-bench-runner --bin pcs-bench -- lattice-eval matrix

# One measured sample of a supported cell (payload 2^31, log2 N = 26)
./scripts/lattice-eval.sh run --scheme akita --payload 31 --runs 1 --warmups 0
./scripts/lattice-eval.sh run --scheme greyhound --payload 31 --runs 1 --warmups 0
./scripts/lattice-eval.sh run --scheme rokoko --payload 31 --runs 1 --warmups 0
./scripts/lattice-eval.sh run --scheme akita-pr466 --payload 31 --runs 1 --warmups 0

# Direct workers (what each harness sample wraps with with-memlimit.sh)
./scripts/with-memlimit.sh 117128687616 \
  env RAYON_NUM_THREADS=1 AKITA_PARALLEL=0 \
  cargo run --release -p pcs-bench-akita --bin lattice-eval -- \
    --log2-n 26 --payload-log2 31
./scripts/with-memlimit.sh 117128687616 target/greyhound/lattice-eval --log2-n 26
```

