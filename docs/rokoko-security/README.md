# RoKoKo security accounting for the published lattice benchmark

## Conclusion

The published RoKoKo rows do **not** currently have a substantiated overall
security-bit value. In particular, neither 128 bits nor the 123-bit minimum
reported by a separate lattice diagnostic is an overall PCS security estimate.
There are explicit statistical terms below 100 bits, and the concrete folding
sampler does not satisfy the formal paper's challenge-set hypothesis without
an additional approximate-sampling argument.

This analysis establishes component calculations and identifies what prevents
turning them into an end-to-end bound. It is not a complete cryptographic audit,
a demonstrated forgery, or a claim that an attack achieves the calculated costs.

## Scope and provenance

- Benchmark records: `results/lattice-x86_64/records.jsonl`. All 55 RoKoKo records
  name revision `26d07c73c54872b9e8d2b3200117a6a0a21b10ee`; each of p-22, p-24,
  p-26, p-28, and p-30 has 11 records, including warmup.
- Parameter extraction uses that vendored revision. The local executor includes
  the benchmark's existing timing/resource and seed changes. No vendor code was
  changed for this analysis.
- [RoKoKo paper](https://eprint.iacr.org/2026/575.pdf), retrieved 2026-09-15,
  63 PDF pages. SHA-256:
  `325817d34687378949cf2157e14baa178dbe177a0fe0f16bf11da24cfbfb3acf`.
  This is the retrieved paper version, not a claim that the paper and pinned
  implementation have identical parameter selection or sampling.
- `profiles.csv` was extracted directly from P_MICRO, P_TINY, P_SMALL, P_MEDIUM,
  and P_LARGE. No large witness or new timing matrix was required.

## 1. What the paper claims

Section 9.3 separates a 128-bit lattice-estimator target from an approximately
2^-100 statistical soundness target. Section 9.1, however, discusses an
approximately 2^-94 non-invertibility heuristic for short challenges. Those
numbers describe different parts of the analysis; 128 is not an overall
statistical soundness claim.

For q = 2^50 - 2687, ring degree phi = 128, and residue degree e = 2:

    log2(q^2) = approximately 100
    epsilon_nonunit ~= phi / (e * q^e) = 64 / q^2
    -log2(epsilon_nonunit) = approximately 94

The latter is the paper's heuristic scale, not an independently established
bound for the pinned sampler. A bad event in a knowledge-extraction argument is
also not automatically an attack with the inverse of that event's probability.

## 2. The actual short-challenge sampler differs from the description

Pinned [`short_challenge.rs`](https://github.com/lattice-arguments/rokoko/blob/26d07c73c54872b9e8d2b3200117a6a0a21b10ee/src/common/short_challenge.rs)
uses exactly 22 nonzero +/-1 coefficients among 128 positions and rejects when
the operator norm exceeds 10. The paper's Section 9.1 describes independent
ternary coefficients with zero probability 1/3. These are different distributions.

Before norm rejection, the fixed-weight support size is:

    C(128, 22) * 2^22 = approximately 2^103.30785

The pinned `repetition_rate()` function, run locally on aarch64 with its
10,000 deterministic trials, returned **1.8801** attempts per accepted sample.
This suggests an accepted support of approximately **2^102.39704**, assuming
ideal independent XOF output. It is an empirical support estimate, not a
rare-event estimate or proof. Earlier comments suggesting roughly 1.24 attempts
and 103 accepted bits do not match this run.

A genuine subtractive challenge set (every distinct pair has invertible
difference) in this product of quadratic fields can have at most q^2 elements:
projection onto any one factor must be injective. The empirical support estimate
exceeds that maximum. We therefore cannot apply the formal folding lemma's
r/|C| term directly to this whole accepted support. The distribution after norm
rejection needs its own bound on non-invertible differences in the extractor's
required sampling/conditioning model. The paper's independent-ternary heuristic
does not provide that bound for fixed-weight, norm-rejected sampling.

## 3. A computable statistical component: fine-projection batching

Pinned [`project_fine.rs`](https://github.com/lattice-arguments/rokoko/blob/26d07c73c54872b9e8d2b3200117a6a0a21b10ee/src/protocol/project_fine.rs)
`sample_layers` samples tensor challenges in the **base field**. The verifier
uses two independently sampled batches (`NOF_BATCHES = 2`).

For a fixed nonzero multilinear residual, a single batch is a polynomial
identity check in k scalar coordinates. A Schwartz-Zippel argument with maximum
point probability p_max gives failure probability at most k*p_max. Two
independent repetitions give at most (k*p_max)^2. This is conditional on fixing
the residual before these challenges and ideal XOF randomness; binding,
extraction, and Fiat-Shamir composition are separate obligations.

The code samples each scalar using `u64 % q`, so we use its actual maximum mass:

    p_max = ceil(2^64 / q) / 2^64
    d = (witness_height / projection_ratio) * 128 / projection_height
    k = log2(d) + log2(projection_height) + log2(witness_width)
    component_bits = -2 * log2(k * p_max)

The final SimpleConfig checks columns separately, so its effective width in
this formula is 1. This matches the special case in
`verifier_sample_projection_challenges`; we do not infer a vector-wide bound
by treating separate column checks as a single event.

| Native profile | Config stages | Largest k in a fine-projection batch | Worst two-batch component bound (bits) |
| --- | ---: | ---: | ---: |
| p-22 | 6 | 15 | 92.186 |
| p-24 | 6 | 16 | 92.000 |
| p-26 | 7 | 16 | 92.000 |
| p-28 | 7 | 17 | 91.825 |
| p-30 | 7 | 15 | 92.186 |

These are **component bounds**, not measured attack costs or overall security.
A loose upper bound on failure probability does not prove that the failure
probability attains that bound. In particular, this table does not prove an
approximately 92-bit attack. It shows what this generic argument certifies for
that component under the stated conditions.

The paper's Lemma 8 has an analogous tensor-batching term and an additional
projection-error term. The calculation above follows the actual code's scalar
challenge dimensions, including its projection-row coordinates, rather than
assuming an identical paper-to-code dimension convention.

## 4. Remaining terms and assumptions

- **Sumcheck and linearisation:** a 100-bit quadratic field is a denominator,
  not the final bound. Lemma 13 includes polynomial degrees and variable counts;
  the linearisation argument also includes batching errors. The code uses
  independent ring batching coefficients and a ring-to-field combination, so
  its exact accounting must be derived from that implementation, not copied
  unchanged from the paper's tensor batching formula.
- **Projection failures:** paper Lemmas 7 and 8 include dimension-dependent
  multiples of the projection failure probability. The 128-bit projection
  primitive target is not automatically the complete projection bound.
- **Folding:** Lemma 9 assumes a subtractive challenge set and has a width
  factor. The actual initial widths are 64, 128, 128, 256, and 512 respectively.
  An approximate-set replacement must account for the actual distribution and
  the extractor's rewinding conditions.
- **Lattice hardness:** the earlier p-22 smoke runs returned 123 through 168
  rounded classical MATZOV estimates. They used honest-run-dependent norms.
  A security claim must cover the verifier's full accepted norm bounds and all
  commitment layers, and justify use of generic SIS costs for the structured
  vanishing-SIS assumption. Other profiles have not been diagnosed here.
- **Noninteractive composition:** the implementation uses BLAKE3 Fiat-Shamir.
  An end-to-end ROM/QROM claim needs a concrete reduction, query budget, and
  applicable loss factors. We have not established that reduction here.

We do not add all component bit counts, average lattice estimates, or silently
apply a universal subtraction for the number of rounds. Ordinary aggregate
soundness and an RBR parameter are distinct. Choosing either requires proving
that the relevant theorem applies; an RBR label does not remove the folding
sampler hypothesis or the noninteractive reduction obligation.

## Publication recommendation

The publication report uses **< 100 bits** as a broad reporting category,
following the chosen reporting convention. This does not assert a precise
validated end-to-end bound. The calculator continues to leave
`overall_security_bits` null. In particular, do not replace that category with
92, 94, 100, or the lattice diagnostic's 123 as an established overall level.

To close the analysis requires, first, a quantified approximate-challenge
extraction bound for this exact fixed-weight/rejection sampler; then concrete
composition including the other statistical terms, verifier-accepted lattice
bounds, and Fiat-Shamir losses. More honest-run smoke tests cannot establish
probabilities on the order of 2^-94.

## Reproduction

Run the checked-in component calculator:

    python3 docs/rokoko-security/calculate.py

It consumes `profiles.csv` and reproduces `components.json`. Every profile's
`overall_security_bits` is deliberately null.

To regenerate the CSV, place `profile_dump.rs` in a temporary Cargo project's
`src/main.rs`, with a dependency on the absolute path of `third_party/rokoko`:

    [package]
    name = "rokoko-security-analysis"
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    rokoko = { path = "/absolute/path/to/pcs-benchmarks/third_party/rokoko" }

Run with `cargo +nightly-2026-09-03 run --release --offline --manifest-path
/path/to/temporary/Cargo.toml`. Stdout contains the CSV; stderr contains the
sampler's mean attempts. CSV columns are profile, stage, witness height,
witness width, projection ratio, projection height, projection kind, and rank.
The sampler measurement may vary with floating-point behavior across platforms.
