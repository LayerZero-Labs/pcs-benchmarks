Measurements were collected on a single AMD Ryzen 9 9950X 16-Core Processor (Linux x86_64, 32 logical CPUs, 121~GiB RAM). The CPU advertised AVX-512F, but the executed instruction stream was not independently traced. CPU policy: driver amd-pstate-epp, governor powersave, preference balance_performance.

Our first experiment compares Akita with prior lattice-based PCSs on dense
polynomial data at the target volumes above. Akita samples uniform full-field
coefficients and uniform extension-field opening points. This table is a declared
native-workload survey unless all displayed security and statement assumptions match.
For each input, Akita
uses the validated planner schedule selected for that field and size.
The pinned catalogs omit $n_v=22$ and $n_v=24$; those rows are generated
with that same planner at the measured commit.
Akita appears twice: the direct `fp32-dense` catalog, and the same pin
with recursive setup offloading (`fp32-dense-recursive`).
The comparison is exclusively single-threaded: RoKoKo has no native multithreaded
prover, and Greyhound is pinned to `LATTICE_DOGS_THREADS=1` even though the
reference can parallelize extension products. Greyhound uses the
`l2-quantum128-adps16` Euclidean SIS policy and reports contextual proof bytes.
Timing cells report the median and, when supported by the sample count, a
conservative distribution-free 95% confidence interval. Scheme names link to the exact
git commit that was measured.

The timing comparison separates commitment, opening, and verification, while the
cold total includes setup plus commitment and opening. If an implementation embeds
reusable setup inside commitment, that cost remains in its cold total and its separate
setup entry is reported as unknown; reusable state size is reported when measurable.
The resources table reports communication, memory, and preprocessing.

RoKoKo uses the field $\mathbb{F}_{2^{50}-2687}$ and fixed native parameter
sets, so we report its closest supported input at each target payload. An OOM entry
is a confirmed allocation failure under the 109 GiB virtual-address-space ceiling. RoKoKo's native field has about 50 bits, but its sampler is bounded to
31-bit coefficients. The displayed payload is nominal field capacity and must not
be interpreted as sampled information content or used to rescale throughput.

| Nominal payload | Scheme | Field | log₂ N | Commit (s) | Open (s) | Cold total (s) | Verify (ms) |
| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: |
| 2^{27} | Akita | $2^{32}-99$ | 22 | pending | pending | pending | pending |
| 2^{27} | Akita (offload) | $2^{32}-99$ | 22 | pending | pending | pending | pending |
| 2^{27} | Greyhound | $2^{32}-99$ | 22 | pending | pending | pending | pending |
| 2^{27} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | $2^{50}-2687$ | —(1) | —(1) | —(1) | —(1) | —(1) |
| 2^{29} | Akita | $2^{32}-99$ | 24 | pending | pending | pending | pending |
| 2^{29} | Akita (offload) | $2^{32}-99$ | 24 | pending | pending | pending | pending |
| 2^{29} | Greyhound | $2^{32}-99$ | 24 | pending | pending | pending | pending |
| 2^{29} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | $2^{50}-2687$ | —(1) | —(1) | —(1) | —(1) | —(1) |
| 2^{31} | Akita | $2^{32}-99$ | 26 | pending | pending | pending | pending |
| 2^{31} | Akita (offload) | $2^{32}-99$ | 26 | pending | pending | pending | pending |
| 2^{31} | Greyhound | $2^{32}-99$ | 26 | pending | pending | pending | pending |
| 2^{31} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | $2^{50}-2687$ | 26 | 0.816 [0.814, 0.823] | 0.733 [0.730, 0.738] | 1.92 [1.92, 1.93] | 4.9 [4.8, 5.0] |
| 2^{33} | Akita | $2^{32}-99$ | 28 | pending | pending | pending | pending |
| 2^{33} | Akita (offload) | $2^{32}-99$ | 28 | pending | pending | pending | pending |
| 2^{33} | Greyhound | $2^{32}-99$ | 28 | pending | pending | pending | pending |
| 2^{33} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | $2^{50}-2687$ | 28 | 3.35 [3.34, 3.37] | 1.49 [1.48, 1.49] | 5.58 [5.57, 5.61] | 5.0 [5.0, 5.1] |
| 2^{35} | Akita | $2^{32}-99$ | 30 | pending | pending | pending | pending |
| 2^{35} | Akita (offload) | $2^{32}-99$ | 30 | pending | pending | pending | pending |
| 2^{35} | Greyhound | $2^{32}-99$ | 30 | pending | pending | pending | pending |
| 2^{35} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | $2^{50}-2687$ | 30 | 17.4 [17.3, 17.5] | 4.47 [4.46, 4.48] | 23.6 [23.5, 23.7] | 7.4 [7.3, 7.6] |

**(1)** RoKoKo ships only native sets `p-26`, `p-28`, and `p-30`; no instance matches this payload.


| Nominal payload | Scheme | Commitment (B) | Evaluation (B) | Proof (B) | Total sent (B) | Excluded context (B) | Peak RSS (GiB) | Prep. (s) | State (GiB) |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2^{27} | Akita | pending | pending | pending | pending | pending | pending | pending | pending |
| 2^{27} | Akita (offload) | pending | pending | pending | pending | pending | pending | pending | pending |
| 2^{27} | Greyhound | pending | pending | pending | pending | pending | pending | pending | pending |
| 2^{27} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) |
| 2^{29} | Akita | pending | pending | pending | pending | pending | pending | pending | pending |
| 2^{29} | Akita (offload) | pending | pending | pending | pending | pending | pending | pending | pending |
| 2^{29} | Greyhound | pending | pending | pending | pending | pending | pending | pending | pending |
| 2^{29} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) |
| 2^{31} | Akita | pending | pending | pending | pending | pending | pending | pending | pending |
| 2^{31} | Akita (offload) | pending | pending | pending | pending | pending | pending | pending | pending |
| 2^{31} | Greyhound | pending | pending | pending | pending | pending | pending | pending | pending |
| 2^{31} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | 776 | 775 | 114812 | 116363 | 0 | 4.09 | 0.372 | 1.59 |
| 2^{33} | Akita | pending | pending | pending | pending | pending | pending | pending | pending |
| 2^{33} | Akita (offload) | pending | pending | pending | pending | pending | pending | pending | pending |
| 2^{33} | Greyhound | pending | pending | pending | pending | pending | pending | pending | pending |
| 2^{33} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | 774 | 776 | 114892 | 116442 | 0 | 10.9 | 0.744 | 3.19 |
| 2^{35} | Akita | pending | pending | pending | pending | pending | pending | pending | pending |
| 2^{35} | Akita (offload) | pending | pending | pending | pending | pending | pending | pending | pending |
| 2^{35} | Greyhound | pending | pending | pending | pending | pending | pending | pending | pending |
| 2^{35} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | 774 | 774 | 114991 | 116539 | 0 | 35.6 | 1.74 | 7.44 |

**(1)** RoKoKo ships only native sets `p-26`, `p-28`, and `p-30`; no instance matches this payload.


### Measured commits

- RoKoKo [`26d07c73`](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee)


### Reproduction template

Machine, ISA, compiler, executable, lockfile, command, and timestamp provenance
for this dataset are recorded with each observation. The infrastructure
toolchain pin is Rust **1.95** (`rust-toolchain.toml`).
Recorded runner command: `target/release/pcs-bench lattice-eval run --scheme rokoko --out results/rokoko-x86_64-26d07c73`. The commands below are a template,
not reconstructed provenance.
RoKoKo uses `rustup` **nightly-2026-09-03**. Workers are built before sampling;
every timed execution is then a fresh process wrapped
in `scripts/with-memlimit.sh` with a 109~GiB virtual-address-space ceiling (`ulimit -v`, numerically 90% of host RAM) and `RAYON_NUM_THREADS=1`.
Raw records identify warmup and measured processes separately; warmup rows
are stored with `warmup: true` and excluded from the aggregate. Workload seeds
and the `vary`/`fixed` seed mode are recorded per observation. Greyhound is
`LayerZero-Labs/greyhound-reference`, built with `-march=native -O3 -flto`,
and run with `LATTICE_DOGS_THREADS=1` and `LABRADOR_SIS_SECURITY=l2-quantum128-adps16`.
Proof sizes are contextual wire bytes. Recorded worker flags for this dataset:
`-C target-cpu=native`.
`./scripts/fetch-vendors.sh` clones the pinned implementations,
installs planner-generated `fp32-dense` rows for `nv=22` and `nv=24`, installs
the recursive `fp32-dense` setup-offload catalog, and
patches RoKoKo so the executor prints commitment, CRS, and peak RSS.

Non-interactive shells may not put Cargo on `PATH`; `source ~/.cargo/env`
is required in that case. `CARGO_NET_GIT_FETCH_WITH_CLI=true` avoids libgit2 auth
failures when fetching the pinned git dependencies.
Published numbers live in `results/lattice-x86_64/`.

```bash
# On an AVX-512 Linux x86_64 host
source "$HOME/.cargo/env"   # if cargo is not on PATH
cd /path/to/akita-benchmark

rustup toolchain install nightly-2026-09-03 -c rustc,cargo   # once, for RoKoKo
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export RUSTFLAGS="-C target-cpu=native"
export RAYON_NUM_THREADS=1

./scripts/fetch-vendors.sh          # Greyhound, RoKoKo, Akita pins + nv=22/24 + offload catalogs
./scripts/extend-akita-fp32-dense-offload.sh third_party/akita   # once; fills the offload catalog
./scripts/build-greyhound.sh

# Full 20-cell matrix (Akita, Akita offload, Greyhound, RoKoKo)
./scripts/lattice-eval.sh run --out results/lattice-x86_64

# Rebuild Markdown + LaTeX from the JSONL already in that directory
cargo run -p pcs-bench-runner --bin pcs-bench -- lattice-eval compare \
  results/lattice-x86_64 --out-dir results/lattice-x86_64
```

**Sanity-check the harness before trusting a full run.** `lattice-eval matrix`
prints the 20-cell plan (unsupported RoKoKo sizes, Akita/Greyhound `log2 N`,
RoKoKo `p-26`/`p-28`/`p-30`, and the Akita setup-offload row). A single supported cell should verify and emit
JSON with `status: ok`. Unit tests cover the RoKoKo log parser, OOM
classification, and table tokens. Each sample the runner launches is equivalent
to the worker commands below (still under the 90%-of-RAM cap).

```bash
export RUSTFLAGS="-C target-cpu=native"
export RAYON_NUM_THREADS=1

cargo test --workspace --locked
cargo run -p pcs-bench-runner --bin pcs-bench -- lattice-eval matrix
CARGO_TARGET_DIR=target/akita cargo build --release --locked \
  --manifest-path benchmarks/akita/Cargo.toml --bin lattice-eval

# One measured sample of a supported cell (payload 2^31, log2 N = 26)
./scripts/lattice-eval.sh run --scheme akita --payload 31 --runs 1 --warmups 0
./scripts/lattice-eval.sh run --scheme akita-offload --payload 31 --runs 1 --warmups 0
./scripts/lattice-eval.sh run --scheme greyhound --payload 31 --runs 1 --warmups 0
./scripts/lattice-eval.sh run --scheme rokoko --payload 31 --runs 1 --warmups 0

# Direct workers (what each harness sample wraps with with-memlimit.sh)
./scripts/with-memlimit.sh 117128687616 \
  env RAYON_NUM_THREADS=1 AKITA_PARALLEL=0 PCS_BENCH_SEED=1 \
  target/akita/release/lattice-eval \
    --log2-n 26 --payload-log2 31
./scripts/with-memlimit.sh 117128687616 \
  env RAYON_NUM_THREADS=1 AKITA_PARALLEL=0 PCS_BENCH_SEED=1 \
  target/akita/release/lattice-eval \
    --log2-n 26 --payload-log2 31 --offload
./scripts/with-memlimit.sh 117128687616 target/greyhound/lattice-eval --log2-n 26
```

