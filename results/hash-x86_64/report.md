Measurements were collected on a single AMD Ryzen 9 9950X 16-Core Processor (Linux x86_64, 32 logical CPUs, 121~GiB RAM). AVX-512F is advertised by the CPU and was activated for this run (`-C target-cpu=native`). Compiler flags: `-C target-cpu=native`.

Our second experiment compares Akita with WHIR and BaseFold, representative
high-performance hash-based PCSs, on the same dense standalone workloads.
Akita uses the validated planner schedule for each field and input size
(`fp32-dense`, including planner-generated $n_v=22$ and $n_v=24$ rows).
The comparison uses a common **128-bit** transcript-error target: WHIR is
Plonky3 `p3-whir` at `security_level=128`. Capacity bound at rate $1/2$ is used
when that instance fits a 30-bit KoalaBear grind ($\log_2 N \le 26$);
unique decoding at rate $1/2$ is used at $\log_2 N=28$ and $30$, where list-decoding
bounds on KoalaBear cannot close 128 bits within that grind limit. BaseFold is
SP1 SLOP stacked BaseFold with FRI parameters `log_blowup=1`, 112 queries, and
16 bits of grinding (conjectured soundness $1\cdot 112+16=128$).
WHIR and BaseFold run over KoalaBear ($q=2^{31}-2^{24}+1$) at the same
$\log_2 N$ as Akita so the dense tables have matching coefficient counts.
Timing cells report median ± sample standard deviation across fresh processes
after warmup, at **1 and 8 threads**. Scheme names link to the exact git commit
that was measured.

The timing comparison separates commitment, opening, and verification, while the
resources table reports communication, memory (1-thread and 8-thread peak RSS),
and preprocessing. An OOM entry exceeds the 109 GiB worker memory limit (90% of host RAM).

| Payload | Scheme | Field | log₂ N | Threads | Commit (s) | Open (s) | Total (s) | Verify (ms) |
| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | $2^{32}-99$ | 22 | 1 | 0.098 ± 0.0004 | 0.956 ± 0.0018 | 1.05 ± 0.0022 | 7.2 ± 0.01 |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | $2^{32}-99$ | 22 | 8 | 0.021 ± 0.0003 | 0.251 ± 0.0090 | 0.272 ± 0.0088 | 5.0 ± 0.25 |
| 2^{27} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 22 | 1 | 0.087 ± 0.0011 | 0.526 ± 0.0007 | 0.612 ± 0.0018 | 1.5 ± 0.01 |
| 2^{27} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 22 | 8 | 0.024 ± 0.0004 | 0.113 ± 0.014 | 0.137 ± 0.013 | 1.5 ± 0.02 |
| 2^{27} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 22 | 1 | 0.581 ± 0.0093 | 0.824 ± 0.010 | 1.40 ± 0.0014 | 20.0 ± 0.21 |
| 2^{27} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 22 | 8 | 0.561 ± 0.011 | 0.817 ± 0.025 | 1.38 ± 0.015 | 19.7 ± 0.25 |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | $2^{32}-99$ | 24 | 1 | 0.343 ± 0.0032 | 1.43 ± 0.0012 | 1.77 ± 0.0027 | 11.5 ± 0.06 |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | $2^{32}-99$ | 24 | 8 | 0.055 ± 0.0010 | 0.340 ± 0.0048 | 0.395 ± 0.0058 | 7.0 ± 0.38 |
| 2^{29} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 24 | 1 | 0.376 ± 0.0012 | 5.53 ± 0.027 | 5.91 ± 0.027 | 1.6 ± 0.08 |
| 2^{29} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 24 | 8 | 0.109 ± 0.0011 | 1.76 ± 0.126 | 1.87 ± 0.127 | 1.7 ± 0.00 |
| 2^{29} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 24 | 1 | 0.864 ± 0.012 | 0.887 ± 0.0054 | 1.75 ± 0.015 | 20.1 ± 0.07 |
| 2^{29} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 24 | 8 | 0.862 ± 0.013 | 0.837 ± 0.011 | 1.69 ± 0.0037 | 20.2 ± 0.40 |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | $2^{32}-99$ | 26 | 1 | 1.30 ± 0.0069 | 2.58 ± 0.0022 | 3.88 ± 0.0088 | 13.9 ± 0.05 |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | $2^{32}-99$ | 26 | 8 | 0.187 ± 0.0019 | 0.523 ± 0.0039 | 0.710 ± 0.0021 | 6.7 ± 0.04 |
| 2^{31} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 26 | 1 | 1.52 ± 0.0032 | 44.9 ± 0.236 | 46.4 ± 0.239 | 1.8 ± 0.02 |
| 2^{31} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 26 | 8 | 0.473 ± 0.0065 | 6.39 ± 1.38 | 6.86 ± 1.38 | 1.9 ± 0.01 |
| 2^{31} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 26 | 1 | 2.52 ± 0.0061 | 1.01 ± 0.0021 | 3.53 ± 0.0081 | 20.7 ± 0.11 |
| 2^{31} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 26 | 8 | 2.51 ± 0.113 | 0.970 ± 0.017 | 3.48 ± 0.102 | 20.3 ± 0.64 |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | $2^{32}-99$ | 28 | 1 | 6.10 ± 0.029 | 6.84 ± 0.012 | 13.0 ± 0.035 | 19.8 ± 0.41 |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | $2^{32}-99$ | 28 | 8 | 0.803 ± 0.0016 | 1.18 ± 0.011 | 1.99 ± 0.010 | 7.0 ± 0.06 |
| 2^{33} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 28 | 1 | 5.39 ± 1.93 | 35.3 ± 3.63 | 40.4 ± 5.05 | 9.0 ± 0.78 |
| 2^{33} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 28 | 8 | 1.68 ± 0.019 | 8.60 ± 0.100 | 10.3 ± 0.106 | 8.9 ± 0.06 |
| 2^{33} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 28 | 1 | 9.41 ± 0.0093 | 1.58 ± 0.013 | 11.0 ± 0.011 | 22.4 ± 0.32 |
| 2^{33} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 28 | 8 | 9.03 ± 0.167 | 1.39 ± 0.016 | 10.4 ± 0.155 | 22.8 ± 0.40 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | $2^{32}-99$ | 30 | 1 | 25.6 ± 0.250 | 17.4 ± 0.357 | 43.0 ± 0.603 | 33.1 ± 0.88 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | $2^{32}-99$ | 30 | 8 | 3.94 ± 1.00 | 3.68 ± 0.548 | 7.34 ± 1.39 | 11.9 ± 1.44 |
| 2^{35} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 30 | 1 | 16.7 ± 0.102 | 30.4 ± 0.026 | 47.1 ± 0.096 | 10.0 ± 0.08 |
| 2^{35} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 30 | 8 | 5.59 ± 0.018 | 8.74 ± 0.034 | 14.3 ± 0.037 | 10.1 ± 0.06 |
| 2^{35} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 30 | 1 | 35.2 ± 0.549 | 3.49 ± 0.022 | 38.6 ± 0.541 | 30.3 ± 0.30 |
| 2^{35} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 30 | 8 | 35.2 ± 0.588 | 2.96 ± 0.166 | 38.1 ± 0.718 | 31.0 ± 0.77 |


| Payload | Scheme | Commitment (B) | Proof (KiB) | Total (KiB) | Peak RSS 1-thread (GiB) | Peak RSS 8-thread (GiB) | Prep. (s) | State (GiB) |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | 343 | 63.8 | 64.1 | 0.190 | 0.197 | 0.0072 | 0.0017 |
| 2^{27} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 33 | 89.1 | 89.2 | 0.179 | 0.178 | 0.0013 | 0.0000 |
| 2^{27} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 883.9 | 884.0 | 0.362 | 0.361 | 0.0000 | 0.0000 |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | 343 | 64.5 | 64.8 | 0.632 | 0.632 | 0.0079 | 0.0020 |
| 2^{29} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 33 | 100.2 | 100.3 | 0.703 | 0.702 | 0.0057 | 0.0000 |
| 2^{29} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 889.4 | 889.4 | 0.502 | 0.502 | 0.0000 | 0.0000 |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | 343 | 65.3 | 65.7 | 1.93 | 1.94 | 0.0197 | 0.0049 |
| 2^{31} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 33 | 110.4 | 110.5 | 2.80 | 2.80 | 0.0269 | 0.0000 |
| 2^{31} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 911.1 | 911.2 | 1.75 | 1.75 | 0.0000 | 0.0000 |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | 343 | 66.5 | 66.9 | 6.38 | 6.39 | 0.0420 | 0.0107 |
| 2^{33} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 33 | 575.3 | 575.3 | 9.60 | 9.60 | 0.0730 | 0.0000 |
| 2^{33} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 998.1 | 998.2 | 7.00 | 7.00 | 0.0000 | 0.0000 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | 343 | 67.6 | 68.0 | 24.7 | 24.7 | 0.0834 | 0.0215 |
| 2^{35} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 33 | 676.6 | 676.6 | 17.1 | 17.1 | 0.0569 | 0.0000 |
| 2^{35} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 1346.1 | 1346.2 | 28.0 | 28.0 | 0.0000 | 0.0000 |


### Measured commits

- Akita [`f9f7de87`](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53)
- WHIR [`9d496524`](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730)
- BaseFold [`0f2a1e13`](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86)


### Commands used for these numbers

These tables were produced on a Linux **x86_64** AVX-512 host (AMD Ryzen 9 9950X)
from this repository. The toolchain pin is Rust **1.95** (`rust-toolchain.toml`).
Every timed worker is a fresh process wrapped in `scripts/with-memlimit.sh` at
109~GiB (`ulimit -v`, 90% of host RAM). The runner defaults are
**1 warmup + 3 measured** samples per cell; warmup rows are stored with
`warmup: true` and excluded from the median. Workers inherit
`RUSTFLAGS=-C target-cpu=native`. WHIR and BaseFold are isolated Cargo trees
(`benchmarks/whir`, `benchmarks/basefold`) so they do not unify with the lattice
workspace. Cargo fetches the pinned Plonky3 and SP1 git revisions on first build.

Non-interactive shells may not put Cargo on `PATH`; `source ~/.cargo/env`
is required in that case. `CARGO_NET_GIT_FETCH_WITH_CLI=true` avoids libgit2 auth
failures when fetching the pinned git dependencies.
Checked-in numbers live in `results/hash-x86_64/`.

```bash
# On an AVX-512 Linux x86_64 host
source "$HOME/.cargo/env"   # if cargo is not on PATH
cd /path/to/akita-benchmark

export CARGO_NET_GIT_FETCH_WITH_CLI=true
export RUSTFLAGS="-C target-cpu=native"

./scripts/fetch-vendors.sh --akita   # Akita pin + nv=22/24 catalogs

# Full 30-cell matrix (3 schemes × 5 payloads × {1,8} threads)
./scripts/hash-eval.sh run --out results/hash-x86_64

# Rebuild Markdown + LaTeX from the JSONL already in that directory
cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval compare \
  results/hash-x86_64 --out-dir results/hash-x86_64
```

**Sanity-check the harness before trusting a full run.** `hash-eval matrix`
prints the 30-cell plan. A single supported cell should verify and emit JSON
with `status: ok`. Each sample the runner launches is equivalent to the worker
commands below (still under the 90%-of-RAM cap).

```bash
export RUSTFLAGS="-C target-cpu=native"

cargo test -p pcs-bench-core -p pcs-bench-runner --locked
cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval matrix

# One measured sample of a supported cell (payload 2^31, log2 N = 26, 1 thread)
./scripts/hash-eval.sh run --scheme akita --payload 31 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme whir --payload 31 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme basefold --payload 31 --threads 1 --runs 1 --warmups 0
```

