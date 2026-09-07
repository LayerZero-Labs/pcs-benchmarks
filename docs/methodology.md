# Benchmark methodology

## Goal

Measure implementation performance without overstating protocol-level
comparability. Raw timings are useful only when the workload, security claim,
build, and machine provenance are explicit.

## Measurement rules

1. **Correctness first.** Every measured proof must verify. Adapters must also
   have a negative test that rejects a modified claim or proof before results
   are published.
2. **Immutable dependencies.** PCS implementations and non-registry
   dependencies use commit hashes, never moving branches or tags.
3. **Deterministic workloads.** Inputs use documented seeds. Input generation,
   allocation, and expected-opening computation occur outside timed regions.
4. **Separated phases.** Setup, commitment, proving/opening, and verification
   are measured independently. Whether setup includes preprocessing must be
   stated.
5. **Fresh mutable state.** A timed operation must not consume state reused by
   later iterations. Criterion's batched iteration is used when cloning or
   reconstruction must stay outside the measured region.
6. **Raw samples.** Keep Criterion sample data. Do not report only a rounded
   mean or a screenshot.
7. **Stable environment.** Disable frequency-changing background workloads,
   connect laptops to power, and use a fixed performance governor where the
   platform supports it. Record thermal or throttling anomalies.
8. **No heterogeneous deltas.** Regression percentages require the same
   physical machine, target ISA, thread count, compiler, flags, and interleaved
   runs of candidate and baseline.

## Cross-scheme comparability

A row is directly comparable only when all of these dimensions agree:

- claimed classical and post-quantum security;
- trusted, transparent, or structured setup model;
- polynomial type and representation;
- number of coefficients/evaluations, committed polynomials, and opening points;
- field and extension-field semantics;
- hiding / zero-knowledge mode;
- batching and recursion behavior;
- CPU ISA, thread count, compiler, flags, and memory limits.

When exact alignment is impossible, publish the mismatch beside the result and
label the comparison directional rather than equivalent.

## Lattice PCS comparison

The headline experiment is documented in [lattice-eval.md](lattice-eval.md).
Additional rules that apply only to that table:

1. **Single-threaded.** Set `RAYON_NUM_THREADS=1` and Greyhound
   `LATTICE_DOGS_THREADS=1`. RoKoKo has no native multithreaded prover; Akita
   and Greyhound are pinned to one thread so the ratio is not a
   parallel-scaling artifact.
2. **Process isolation.** One fresh process per sample. Discard warmup
   processes. Median of the measured processes is the table entry, reported
   with the sample standard deviation when \(n \ge 2\).
3. **90% of host RAM.** `scripts/with-memlimit.sh` applies `ulimit -v` at
   nine-tenths of detected `MemTotal` / `hw.memsize`. Exceeding it
   is `oom`, not a slow run.
4. **Do not bit-match RoKoKo.** Report the native `p-26`/`p-28`/`p-30`
   instance next to the 32-bit payload it is closest to, and say that the
   native field is ~50 bits.
5. **Historical rows.** If a measurement was taken with a different ISA,
   allocator, or thread count, keep it only with `historical: true`. Those
   rows must not form headline ratios.
6. **Catalog honesty.** If Akita has no generated schedule for a requested
   `nv`, record unsupported. Do not silently run a nearby size. For the
   lattice table, `nv=22` and `nv=24` are generated with the pinned revision's
   planner rather than omitted.
7. **Greyhound reference.** Use `LayerZero-Labs/greyhound-reference` at the
   pinned commit, not `lattice-dogs/labrador`. Run with
   `LABRADOR_SIS_SECURITY=l2-quantum128-adps16`. Report contextual proof
   bytes (public `u1` is verifier context). If the instance still cannot
   make Ajtai commitments SIS-secure, the cell is `err` with a footnote, not
   `oom`.

## Hash PCS comparison

The second experiment is documented in [hash-eval.md](hash-eval.md). Additional
rules that apply only to that table:

1. **Native security targets.** Do not retune every scheme to 128 bits.
   Akita, Plonky3 WHIR, and SP1 BaseFold stay at 128-bit transcript error.
   WHIR uses Plonky3 `p3-whir` with `security_level=128`. Capacity bound at
   rate 1/2 is used when the derived grind fits 30 bits (KoalaBear); unique
   decoding at rate 1/2 is used when list-decoding bounds cannot close 128
   bits (`log2 N` 28 and 30 in this matrix). Generated tables footnote those
   WHIR rows. BaseFold uses SP1 SLOP FRI parameters whose conjectured
   soundness is `log_blowup * queries + pow = 128`. Plonky2 FRI, Plonky3
   FRI/STIR, Binius64 BaseFold, and Flock Ligerito Fast use native 100-bit
   targets. ProveKit WHIR uses a 133-bit Johnson-bound Goldilocks instance.
   Akita uses the same validated `fp32-dense` planner schedule as the lattice
   table. Cells are not \(\lambda\)-comparable.
2. **Matched payloads, native \(\log_2 N\).** Convert payload bits by
   coefficient width (32-bit \(-5\), Goldilocks \(-6\), \(\mathbb F_{2^{128}}\)
   \(-7\), Flock bits \(=\) payload). KoalaBear univariate FRI/STIR pack
   \(\log_2 N>23\) into height \(2^{23}\) because two-adicity is 24 at rate
   \(1/2\); footnote those rows.
3. **1 and 8 threads.** Each cell is a fresh process with `RAYON_NUM_THREADS`
   set to the row's thread count. A dash is an unsupported parallel mode.
   Smoke-check that Flock and Binius64 honor the env var.
4. **Process isolation and 90% RAM** are the same as the lattice table.
   Record OOM; do not drop a scheme because one payload OOMs.
5. **Immutable git pins.** Isolated Cargo trees keep those graphs out of the
   lattice workspace. Pins are commit SHAs, never a moving branch. The
   Plonky2 adapter enables `RUSTC_BOOTSTRAP=1` so `specialization` compiles
   on the workspace's Rust 1.95.

## Statistics


Criterion's bootstrap confidence intervals and outlier classification are the
default for microbenchmarks. Use at least 10 samples after warmup. Expensive
end-to-end cases should additionally run in fresh processes, discard at least
one warmup process, retain every raw observation, and report median and
95% confidence intervals. A regression threshold is not a substitute for
examining noise and absolute effect size.

## Result provenance

Published results must include:

- harness and implementation commit hashes;
- UTC timestamp;
- complete benchmark command and case identifier;
- CPU model, architecture, OS, logical CPU count, and memory;
- `rustc -Vv`, Cargo profile, `RUSTFLAGS`, and enabled features;
- sample count, warmup duration, and measurement duration.

The schema in `pcs-bench-core` is versioned. Breaking result changes increment
`RESULT_SCHEMA_VERSION`.

