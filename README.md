# PCS Benchmark

Reproducible benchmarks for lattice polynomial commitment schemes, starting
with [Akita](https://github.com/LayerZero-Labs/akita),
[Greyhound](https://github.com/lattice-dogs/labrador), and
[RoKoKo](https://github.com/lattice-arguments/rokoko).

Each implementation is isolated: Akita is a workspace crate, Greyhound is a C
worker, and RoKoKo runs in its own nightly Cargo tree. A shared crate defines
the workload schema and renders the comparison tables.

## Lattice comparison (headline table)

The first experiment compares Akita, Greyhound, and RoKoKo on **dense** polynomial data.
Payload is \(N \log_2|\mathbb F|\). Akita and Greyhound use \(q=2^{32}-99\).
RoKoKo uses \(q=2^{50}-2687\) and reports its closest native instance
(`p-26`, `p-28`, `p-30`). Those native rows carry about \(25/16\) times the
target number of logical bits. The run is **single-threaded** because neither
Greyhound nor RoKoKo natively supports multithreading. A cell that exceeds
90% of host RAM is recorded as OOM and is not used as a timing.
If Greyhound rejects an instance because the inner Ajtai commitment is not
SIS-secure, the cell is `err` with a footnote; that is not OOM.

Headline numbers in this repository were collected on one **Linux x86_64**
AVX-512 machine (AMD Ryzen 9 9950X, 121~GiB RAM, 90% worker cap ≈ 109~GiB).
Timing cells are median ± sample standard deviation (1 warmup + 3 measured
processes). Scheme names in the generated report link to the exact git commit
that was measured. Akita is reported both at the pinned `main` commit and at
[PR #466](https://github.com/LayerZero-Labs/akita/pull/466).

Checked-in JSONL, Markdown, and LaTeX live in
[`results/lattice-x86_64/`](results/lattice-x86_64/report.md).

| Payload | Scheme | Field | log₂ N | Commit (s) | Open (s) | Total (s) | Verify (ms) |
| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | \(2^{32}-99\) | 22 | 0.099 ± 0.0002 | 0.959 ± 0.0009 | 1.06 ± 0.0007 | 7.2 ± 0.01 |
| 2^{27} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | \(2^{32}-99\) | 22 | 0.099 ± 0.0008 | 0.995 ± 0.0008 | 1.09 ± 0.0013 | 7.5 ± 0.00 |
| 2^{27} | [Greyhound](https://github.com/lattice-dogs/labrador/commit/8b6626b26afd4c0162ddd089759d21d3d51bfbdf) | \(2^{32}-99\) | 22 | 0.097 ± 0.0028 | 0.090 ± 0.0006 | 0.187 ± 0.0032 | 43.5 ± 0.38 |
| 2^{27} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | \(2^{50}-2687\) | —(1) | —(1) | —(1) | —(1) | —(1) |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | \(2^{32}-99\) | 24 | 0.345 ± 0.0004 | 1.43 ± 0.015 | 1.77 ± 0.015 | 11.6 ± 0.29 |
| 2^{29} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | \(2^{32}-99\) | 24 | 0.346 ± 0.0007 | 1.52 ± 0.0021 | 1.87 ± 0.0026 | 9.3 ± 0.04 |
| 2^{29} | [Greyhound](https://github.com/lattice-dogs/labrador/commit/8b6626b26afd4c0162ddd089759d21d3d51bfbdf) | \(2^{32}-99\) | 24 | 0.381 ± 0.0026 | 0.183 ± 0.0053 | 0.564 ± 0.0078 | 84.4 ± 0.80 |
| 2^{29} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | \(2^{50}-2687\) | —(1) | —(1) | —(1) | —(1) | —(1) |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | \(2^{32}-99\) | 26 | 1.29 ± 0.0036 | 2.57 ± 0.0052 | 3.85 ± 0.0087 | 13.9 ± 0.10 |
| 2^{31} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | \(2^{32}-99\) | 26 | 1.28 ± 0.0063 | 2.62 ± 0.018 | 3.90 ± 0.012 | 12.7 ± 0.53 |
| 2^{31} | [Greyhound](https://github.com/lattice-dogs/labrador/commit/8b6626b26afd4c0162ddd089759d21d3d51bfbdf) | \(2^{32}-99\) | 26 | 2.03 ± 0.0095 | 0.527 ± 0.0008 | 2.55 ± 0.010 | 214 ± 0.29 |
| 2^{31} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | \(2^{50}-2687\) | 26 | 0.876 ± 0.015 | 0.743 ± 0.0055 | 1.62 ± 0.015 | 4.8 ± 0.08 |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | \(2^{32}-99\) | 28 | 6.06 ± 0.023 | 6.78 ± 0.0063 | 12.8 ± 0.027 | 19.7 ± 0.51 |
| 2^{33} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | \(2^{32}-99\) | 28 | 6.05 ± 0.021 | 6.65 ± 0.027 | 12.7 ± 0.040 | 21.6 ± 0.47 |
| 2^{33} | [Greyhound](https://github.com/lattice-dogs/labrador/commit/8b6626b26afd4c0162ddd089759d21d3d51bfbdf) | \(2^{32}-99\) | 28 | 10.5 ± 0.047 | 2.02 ± 0.0100 | 12.5 ± 0.048 | 379 ± 1.93 |
| 2^{33} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | \(2^{50}-2687\) | 28 | 3.50 ± 0.012 | 1.59 ± 0.011 | 5.10 ± 0.0052 | 4.9 ± 0.13 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | \(2^{32}-99\) | 30 | 24.4 ± 0.075 | 16.9 ± 0.0060 | 41.2 ± 0.075 | 33.0 ± 0.16 |
| 2^{35} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | \(2^{32}-99\) | 30 | 24.6 ± 0.065 | 16.0 ± 0.0045 | 40.6 ± 0.063 | 33.2 ± 0.11 |
| 2^{35} | [Greyhound](https://github.com/lattice-dogs/labrador/commit/8b6626b26afd4c0162ddd089759d21d3d51bfbdf) | \(2^{32}-99\) | 30 | err(2) | err(2) | err(2) | err(2) |
| 2^{35} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | \(2^{50}-2687\) | 30 | 18.0 ± 0.252 | 4.94 ± 0.028 | 23.0 ± 0.250 | 7.6 ± 0.33 |

**(1)** RoKoKo ships only native sets `p-26`, `p-28`, and `p-30`; no instance matches this payload.
**(2)** Greyhound cannot make the inner Ajtai commitments SIS-secure at \(\log_2 N=30\) (`kappa` \(\le\) 32). Labrador rejects the instance (`polcom_reduce`: inner commitments not secure). This is not an out-of-memory failure.

| Payload | Scheme | Commitment (B) | Proof (KiB) | Total (KiB) | Peak RSS (GiB) | Prep. (s) | State (GiB) |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | 343 | 63.8 | 64.1 | 0.190 | 0.0067 | 0.0017 |
| 2^{27} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | 343 | 59.9 | 60.2 | 0.187 | 0.0080 | 0.0020 |
| 2^{27} | [Greyhound](https://github.com/lattice-dogs/labrador/commit/8b6626b26afd4c0162ddd089759d21d3d51bfbdf) | 3072 | 50.4 | 53.4 | 0.288 | 0 | 0.0000 |
| 2^{27} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | 343 | 64.5 | 64.8 | 0.632 | 0.0082 | 0.0020 |
| 2^{29} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | 343 | 60.3 | 60.7 | 0.633 | 0.0081 | 0.0020 |
| 2^{29} | [Greyhound](https://github.com/lattice-dogs/labrador/commit/8b6626b26afd4c0162ddd089759d21d3d51bfbdf) | 3584 | 53.5 | 57.0 | 1.00 | 0 | 0.0000 |
| 2^{29} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | 343 | 65.3 | 65.7 | 1.93 | 0.0186 | 0.0049 |
| 2^{31} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | 343 | 61.6 | 61.9 | 1.93 | 0.0190 | 0.0049 |
| 2^{31} | [Greyhound](https://github.com/lattice-dogs/labrador/commit/8b6626b26afd4c0162ddd089759d21d3d51bfbdf) | 3584 | 54.7 | 58.2 | 3.78 | 0 | 0.0000 |
| 2^{31} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | 774 | 112.1 | 112.9 | 4.07 | 0.335 | 1.59 |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | 343 | 66.5 | 66.9 | 6.38 | 0.0406 | 0.0107 |
| 2^{33} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | 343 | 63.0 | 63.3 | 6.36 | 0.0376 | 0.0098 |
| 2^{33} | [Greyhound](https://github.com/lattice-dogs/labrador/commit/8b6626b26afd4c0162ddd089759d21d3d51bfbdf) | 3584 | 56.4 | 59.9 | 17.2 | 0 | 0.0000 |
| 2^{33} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | 773 | 112.2 | 113.0 | 10.9 | 0.698 | 3.19 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | 343 | 67.6 | 68.0 | 24.7 | 0.0812 | 0.0215 |
| 2^{35} | [Akita (#466)](https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740) | 343 | 63.1 | 63.4 | 24.7 | 0.0743 | 0.0195 |
| 2^{35} | [Greyhound](https://github.com/lattice-dogs/labrador/commit/8b6626b26afd4c0162ddd089759d21d3d51bfbdf) | err(2) | err(2) | err(2) | err(2) | err(2) | err(2) |
| 2^{35} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | 777 | 112.4 | 113.1 | 35.6 | 1.65 | 7.44 |

**(1)** RoKoKo ships only native sets `p-26`, `p-28`, and `p-30`; no instance matches this payload.
**(2)** Greyhound cannot make the inner Ajtai commitments SIS-secure at \(\log_2 N=30\) (`kappa` \(\le\) 32). Labrador rejects the instance (`polcom_reduce`: inner commitments not secure). This is not an out-of-memory failure.

## Commands

```bash
# Show the 20-cell matrix (5 payloads × 4 implementations)
cargo run -p pcs-bench-runner --bin pcs-bench -- lattice-eval matrix

# Fetch Greyhound, RoKoKo, and Akita pins (Akita catalogs include nv=22/24)
./scripts/fetch-vendors.sh

# Run one Akita payload (log2 N = 26, payload 2^31)
./scripts/lattice-eval.sh run --scheme akita --payload 31 --runs 1 --warmups 0

# Rebuild the paper-style report from the checked-in JSONL
cargo run -p pcs-bench-runner --bin pcs-bench -- lattice-eval compare \
  results/lattice-x86_64 --out-dir results/lattice-x86_64
```

Each run writes `records.jsonl`, `table.md`, `table.tex`, `report.md`, `report.tex`,
and `provenance.txt`.

Greyhound needs Linux x86_64 + AVX-512. RoKoKo needs `rustup` nightly. Full
methodology, pins, and the LaTeX table command are in
[docs/lattice-eval.md](docs/lattice-eval.md).

RoKoKo has no native instance at payload \(2^{27}\) or \(2^{29}\). Greyhound at
payload \(2^{35}\) (`log₂ N = 30`) fails Labrador's inner-commitment SIS check;
that row is `err`, not OOM.

## Criterion microbenchmarks (Akita only)

Smaller fp128 dense phases for local iteration. These are **not** the lattice
comparison table.

```bash
cargo test -p pcs-bench-core
./scripts/fetch-vendors.sh --akita
cargo check --workspace --all-targets

./scripts/bench-akita.sh 'akita/fp128/dense/nv14'
./scripts/bench-akita.sh
```

Criterion writes reports to `target/criterion/`. The wrapper stores provenance
under `results/<timestamp>/` (gitignored except `results/lattice-x86_64/`).

## Adding another PCS

1. Keep the implementation out of Akita's crate graph.
2. Pin an immutable revision.
3. Emit the `WorkerOutput` JSON schema (or a documented native log).
4. Add a `SchemeId` and matrix policy in `pcs-bench-core`.
5. Document field, security, threading, and any payload mismatch.

See [methodology](docs/methodology.md) and [contributing](CONTRIBUTING.md).
