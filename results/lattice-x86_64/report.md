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
sets corresponding to each target's coefficient count. An OOM entry
is a confirmed allocation failure under the 109 GiB virtual-address-space ceiling. RoKoKo's native field has about 50 bits, but its sampler is bounded to
31-bit coefficients. The displayed payload is nominal field capacity and must not
be interpreted as sampled information content or used to rescale throughput.

### Security targets and accounting

| Scheme | Security bits | Accounting |
| --- | --- | --- |
| Akita (direct and offload) | 128-bit target | Planner-validated Module-SIS and classical-ROM transcript targets. |
| Greyhound | 128-bit SIS target | Euclidean SIS under ADPS16 quantum core-SVP (l2-quantum128-adps16). This is a lattice-hardness policy, not a validated end-to-end transcript bound. |
| RoKoKo | < 100 bits | Fixed native profiles; heuristic soundness accounting. |

These are reported security categories with different accounting scopes, not equivalent end-to-end security guarantees. RoKoKo is reported as a below-100-bit category, not a precise validated estimate.

| Nominal payload | Scheme | Security | Field | log₂ N | Commit (s) | Open (s) | Cold total (s) | Verify (ms) |
| ---: | --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128-bit target | $2^{32}-99$ | 22 | 0.097 [0.096, 0.098] | 0.999 [0.998, 1.00] | 1.10 [1.10, 1.11] | 7.5 [7.5, 7.6] |
| 2^{27} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128-bit target | $2^{32}-99$ | —(1) | —(1) | —(1) | —(1) | —(1) |
| 2^{27} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/672e74100496f6ef698ba35e241cf7593e3d57af) | 128-bit SIS target | $2^{32}-99$ | 22 | 0.110 [0.109, 0.114] | 0.183 [0.180, 0.191] | 0.296 [0.292, 0.301] | 75.0 [74.7, 75.2] |
| 2^{27} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | < 100 bits | $2^{50}-2687$ | 22 | 0.037 [0.036, 0.037] | 0.158 [0.155, 0.161] | 0.286 [0.283, 0.288] | 4.0 [4.0, 4.1] |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128-bit target | $2^{32}-99$ | 24 | 0.333 [0.330, 0.336] | 1.53 [1.53, 1.54] | 1.87 [1.87, 1.88] | 9.3 [8.9, 9.4] |
| 2^{29} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128-bit target | $2^{32}-99$ | 24 | 0.329 [0.328, 0.336] | 2.04 [2.03, 2.04] | 2.40 [2.39, 2.41] | 8.1 [8.1, 8.3] |
| 2^{29} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/672e74100496f6ef698ba35e241cf7593e3d57af) | 128-bit SIS target | $2^{32}-99$ | 24 | 0.444 [0.442, 0.450] | 0.396 [0.394, 0.407] | 0.840 [0.837, 0.857] | 155 [155, 156] |
| 2^{29} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | < 100 bits | $2^{50}-2687$ | 24 | 0.122 [0.122, 0.124] | 0.275 [0.266, 0.277] | 0.584 [0.575, 0.586] | 4.3 [4.2, 4.4] |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128-bit target | $2^{32}-99$ | 26 | 1.25 [1.24, 1.25] | 2.70 [2.70, 2.70] | 3.96 [3.96, 3.98] | 12.7 [12.7, 12.8] |
| 2^{31} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128-bit target | $2^{32}-99$ | 26 | 1.24 [1.23, 1.25] | 3.32 [3.32, 3.33] | 4.63 [4.63, 4.64] | 12.1 [11.7, 12.2] |
| 2^{31} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/672e74100496f6ef698ba35e241cf7593e3d57af) | 128-bit SIS target | $2^{32}-99$ | 26 | 2.10 [2.10, 2.11] | 1.06 [1.04, 1.11] | 3.16 [3.14, 3.21] | 334 [332, 337] |
| 2^{31} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | < 100 bits | $2^{50}-2687$ | 26 | 0.437 [0.436, 0.439] | 0.726 [0.717, 0.730] | 1.61 [1.61, 1.62] | 5.0 [4.8, 5.1] |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128-bit target | $2^{32}-99$ | 28 | 6.02 [5.97, 6.04] | 6.59 [6.56, 6.59] | 12.7 [12.6, 12.7] | 20.8 [20.0, 21.4] |
| 2^{33} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128-bit target | $2^{32}-99$ | 28 | 6.02 [5.99, 6.03] | 7.78 [7.77, 7.78] | 14.0 [14.0, 14.0] | 14.4 [13.7, 14.6] |
| 2^{33} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/672e74100496f6ef698ba35e241cf7593e3d57af) | 128-bit SIS target | $2^{32}-99$ | 28 | 11.5 [11.4, 11.5] | 4.50 [4.46, 4.59] | 16.0 [15.9, 16.1] | 676 [673, 677] |
| 2^{33} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | < 100 bits | $2^{50}-2687$ | 28 | 1.69 [1.68, 1.69] | 1.48 [1.47, 1.49] | 4.05 [4.04, 4.07] | 4.9 [4.8, 4.9] |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128-bit target | $2^{32}-99$ | 30 | 24.3 [24.1, 25.0] | 16.1 [16.1, 16.1] | 40.5 [40.3, 41.1] | 32.5 [32.4, 33.3] |
| 2^{35} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128-bit target | $2^{32}-99$ | 30 | 24.4 [24.2, 24.5] | 18.4 [18.4, 18.4] | 43.3 [43.0, 43.3] | 15.9 [15.7, 15.9] |
| 2^{35} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/672e74100496f6ef698ba35e241cf7593e3d57af) | 128-bit SIS target | $2^{32}-99$ | 30 | 52.3 [52.2, 52.6] | 19.2 [18.9, 19.5] | 71.6 [71.1, 72.2] | 1423 [1421, 1427] |
| 2^{35} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | < 100 bits | $2^{50}-2687$ | 30 | 8.27 [8.21, 8.33] | 4.43 [4.42, 4.45] | 14.8 [14.8, 14.9] | 7.2 [7.1, 7.6] |

**(1)** The recursive `fp32-dense` planner produced a schedule for this $n_v$ with no setup-prefix edge, so the offload variant would not offload setup.


| Nominal payload | Scheme | Commitment (B) | Evaluation (B) | Proof (B) | Total sent (B) | Excluded context (B) | Peak RSS (GiB) | Prep. (s) | State (GiB) |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128 | 16 | 61306 | 61450 | 214 | 0.128 | 0.0080 | 0.0020 |
| 2^{27} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) |
| 2^{27} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/672e74100496f6ef698ba35e241cf7593e3d57af) | 2048 | 8 | 60066 | 62122 | 0 | 0.330 | unknown | 0.0064 |
| 2^{27} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | 775 | 776 | 109289 | 110840 | 0 | 0.852 | 0.0907 | 0.428 |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128 | 16 | 61791 | 61935 | 214 | 0.254 | 0.0084 | 0.0020 |
| 2^{29} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128 | 16 | 66165 | 66309 | 214 | 0.253 | 0.0295 | 0.0020 |
| 2^{29} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/672e74100496f6ef698ba35e241cf7593e3d57af) | 2304 | 8 | 60630 | 62942 | 0 | 1.08 | unknown | 0.0133 |
| 2^{29} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | 774 | 774 | 109359 | 110907 | 0 | 1.72 | 0.186 | 0.855 |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128 | 16 | 63065 | 63209 | 214 | 0.696 | 0.0197 | 0.0049 |
| 2^{31} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128 | 16 | 66308 | 66452 | 214 | 0.699 | 0.0707 | 0.0039 |
| 2^{31} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/672e74100496f6ef698ba35e241cf7593e3d57af) | 2304 | 8 | 63261 | 65573 | 0 | 3.93 | unknown | 0.0290 |
| 2^{31} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | 776 | 775 | 114812 | 116363 | 0 | 4.47 | 0.452 | 1.83 |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128 | 16 | 64477 | 64621 | 214 | 1.38 | 0.0383 | 0.0098 |
| 2^{33} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128 | 16 | 66881 | 67025 | 214 | 1.42 | 0.219 | 0.0156 |
| 2^{33} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/672e74100496f6ef698ba35e241cf7593e3d57af) | 2304 | 8 | 64897 | 67209 | 0 | 20.7 | unknown | 0.0521 |
| 2^{33} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | 774 | 776 | 114892 | 116442 | 0 | 12.3 | 0.893 | 3.66 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128 | 16 | 64595 | 64739 | 214 | 4.69 | 0.0744 | 0.0195 |
| 2^{35} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516) | 128 | 16 | 67322 | 67466 | 214 | 4.71 | 0.443 | 0.0313 |
| 2^{35} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/672e74100496f6ef698ba35e241cf7593e3d57af) | 2560 | 8 | 67212 | 69780 | 0 | 92.6 | unknown | 0.104 |
| 2^{35} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee) | 774 | 774 | 114991 | 116539 | 0 | 36.9 | 2.10 | 8.56 |

**(1)** The recursive `fp32-dense` planner produced a schedule for this $n_v$ with no setup-prefix edge, so the offload variant would not offload setup.


### Measured commits

- Akita [`c0cb822f`](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516)
- Akita (offload) [`c0cb822f`](https://github.com/LayerZero-Labs/akita/commit/c0cb822f28b7b9efe85b1924b029d36e13cdf516)
- Greyhound [`672e7410`](https://github.com/LayerZero-Labs/greyhound-reference/commit/672e74100496f6ef698ba35e241cf7593e3d57af)
- RoKoKo [`26d07c73`](https://github.com/lattice-arguments/rokoko/commit/26d07c73c54872b9e8d2b3200117a6a0a21b10ee)


### Reproduction template

Machine, ISA, compiler, executable, lockfile, command, and timestamp provenance
for this dataset are recorded with each observation. The infrastructure
toolchain pin is Rust **1.95** (`rust-toolchain.toml`).
Recorded runner command(s): `target/release/pcs-bench lattice-eval run --out /home/taghi/pcs-benchmarks-refresh.8wETs2/results-lattice-x86_64-final; target/release/pcs-bench lattice-eval run --scheme akita,akita-offload --payload 27,29,31,33,35 --runs 10 --warmups 1 --seed-mode vary --out results/lattice-x86_64`. The commands below are a template,
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
`./scripts/fetch-vendors.sh` clones the pinned implementations and patches
RoKoKo so the executor prints commitment, CRS, and peak RSS. Akita embeds
the pinned upstream schedule artifacts and committed supplemental direct rows;
no Akita patches or schedule generation are needed.

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

./scripts/fetch-vendors.sh          # Greyhound, RoKoKo, and Akita pins
./scripts/build-greyhound.sh

# Full 20-cell matrix (Akita, Akita offload, Greyhound, RoKoKo)
./scripts/lattice-eval.sh run --out results/lattice-x86_64

# Rebuild Markdown + LaTeX from the JSONL already in that directory
cargo run -p pcs-bench-runner --bin pcs-bench -- lattice-eval compare \
  results/lattice-x86_64 --out-dir results/lattice-x86_64
```

**Sanity-check the harness before trusting a full run.** `lattice-eval matrix`
prints the 20-cell plan (Akita/Greyhound `log2 N`, RoKoKo
`p-22`/`p-24`/`p-26`/`p-28`/`p-30`, and the Akita setup-offload row). A single supported cell should verify and emit
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
