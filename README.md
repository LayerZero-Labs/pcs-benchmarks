# PCS Benchmark

Reproducible benchmarks for polynomial commitment schemes, starting with
lattice PCSs
([Akita](https://github.com/LayerZero-Labs/akita),
[Greyhound](https://github.com/lattice-dogs/labrador),
[RoKoKo](https://github.com/lattice-arguments/rokoko))
and hash-based PCSs
([WHIR](https://github.com/Plonky3/Plonky3) via Plonky3 `p3-whir`,
[BaseFold](https://github.com/succinctlabs/sp1) via SP1 SLOP).

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
and `provenance.txt`. Hash-eval also writes `table-resources.md` and
`table-resources.tex`.

Greyhound needs Linux x86_64 + AVX-512. RoKoKo needs `rustup` nightly. Full
methodology, pins, and the LaTeX table command are in
[docs/lattice-eval.md](docs/lattice-eval.md).

RoKoKo has no native instance at payload \(2^{27}\) or \(2^{29}\). Greyhound at
payload \(2^{35}\) (`log₂ N = 30`) fails Labrador's inner-commitment SIS check;
that row is `err`, not OOM.

## Hash comparison (WHIR and BaseFold)

The second experiment compares Akita with WHIR and BaseFold on the same dense
payloads, at **1 and 8 threads**, with a common **128-bit** transcript-error
target. WHIR is Plonky3 `p3-whir`. BaseFold is SP1 SLOP stacked BaseFold.
Both hash schemes use KoalaBear \(q=2^{31}-2^{24}+1\) at the same \(\log_2 N\)
as Akita (matched coefficient counts, not bit-identical payload).

WHIR prefers capacity bound at rate \(1/2\). That meets 128 bits within a
30-bit KoalaBear grind for \(\log_2 N\le 26\). At \(\log_2 N=28\) and \(30\),
list-decoding bounds cannot close 128 bits, so those rows use unique decoding
at rate \(1/2\) (footnote 1). BaseFold uses FRI `log_blowup=1`, 112 queries,
and 16 bits of grinding (conjectured soundness \(1\cdot 112+16=128\)), with
stacking height 20.

Headline numbers were collected on the same Linux x86_64 AVX-512 machine as
the lattice table. Timing cells are median ± sample standard deviation
(1 warmup + 3 measured processes). Scheme names link to the exact git commit
that was measured.

Checked-in JSONL, Markdown, and LaTeX live in
[`results/hash-x86_64/`](results/hash-x86_64/report.md).

| Payload | Scheme | Field | log₂ N | Threads | Commit (s) | Open (s) | Total (s) | Verify (ms) |
| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | \(2^{32}-99\) | 22 | 1 | 0.098 ± 0.0004 | 0.956 ± 0.0018 | 1.05 ± 0.0022 | 7.2 ± 0.01 |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | \(2^{32}-99\) | 22 | 8 | 0.021 ± 0.0003 | 0.251 ± 0.0090 | 0.272 ± 0.0088 | 5.0 ± 0.25 |
| 2^{27} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | \(2^{31}-2^{24}+1\) | 22 | 1 | 0.087 ± 0.0011 | 0.526 ± 0.0007 | 0.612 ± 0.0018 | 1.5 ± 0.01 |
| 2^{27} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | \(2^{31}-2^{24}+1\) | 22 | 8 | 0.024 ± 0.0004 | 0.113 ± 0.014 | 0.137 ± 0.013 | 1.5 ± 0.02 |
| 2^{27} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | \(2^{31}-2^{24}+1\) | 22 | 1 | 0.581 ± 0.0093 | 0.824 ± 0.010 | 1.40 ± 0.0014 | 20.0 ± 0.21 |
| 2^{27} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | \(2^{31}-2^{24}+1\) | 22 | 8 | 0.561 ± 0.011 | 0.817 ± 0.025 | 1.38 ± 0.015 | 19.7 ± 0.25 |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | \(2^{32}-99\) | 24 | 1 | 0.343 ± 0.0032 | 1.43 ± 0.0012 | 1.77 ± 0.0027 | 11.5 ± 0.06 |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | \(2^{32}-99\) | 24 | 8 | 0.055 ± 0.0010 | 0.340 ± 0.0048 | 0.395 ± 0.0058 | 7.0 ± 0.38 |
| 2^{29} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | \(2^{31}-2^{24}+1\) | 24 | 1 | 0.376 ± 0.0012 | 5.53 ± 0.027 | 5.91 ± 0.027 | 1.6 ± 0.08 |
| 2^{29} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | \(2^{31}-2^{24}+1\) | 24 | 8 | 0.109 ± 0.0011 | 1.76 ± 0.126 | 1.87 ± 0.127 | 1.7 ± 0.00 |
| 2^{29} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | \(2^{31}-2^{24}+1\) | 24 | 1 | 0.864 ± 0.012 | 0.887 ± 0.0054 | 1.75 ± 0.015 | 20.1 ± 0.07 |
| 2^{29} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | \(2^{31}-2^{24}+1\) | 24 | 8 | 0.862 ± 0.013 | 0.837 ± 0.011 | 1.69 ± 0.0037 | 20.2 ± 0.40 |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | \(2^{32}-99\) | 26 | 1 | 1.30 ± 0.0069 | 2.58 ± 0.0022 | 3.88 ± 0.0088 | 13.9 ± 0.05 |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | \(2^{32}-99\) | 26 | 8 | 0.187 ± 0.0019 | 0.523 ± 0.0039 | 0.710 ± 0.0021 | 6.7 ± 0.04 |
| 2^{31} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | \(2^{31}-2^{24}+1\) | 26 | 1 | 1.52 ± 0.0032 | 44.9 ± 0.236 | 46.4 ± 0.239 | 1.8 ± 0.02 |
| 2^{31} | [WHIR](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | \(2^{31}-2^{24}+1\) | 26 | 8 | 0.473 ± 0.0065 | 6.39 ± 1.38 | 6.86 ± 1.38 | 1.9 ± 0.01 |
| 2^{31} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | \(2^{31}-2^{24}+1\) | 26 | 1 | 2.52 ± 0.0061 | 1.01 ± 0.0021 | 3.53 ± 0.0081 | 20.7 ± 0.11 |
| 2^{31} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | \(2^{31}-2^{24}+1\) | 26 | 8 | 2.51 ± 0.113 | 0.970 ± 0.017 | 3.48 ± 0.102 | 20.3 ± 0.64 |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | \(2^{32}-99\) | 28 | 1 | 6.10 ± 0.029 | 6.84 ± 0.012 | 13.0 ± 0.035 | 19.8 ± 0.41 |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | \(2^{32}-99\) | 28 | 8 | 0.803 ± 0.0016 | 1.18 ± 0.011 | 1.99 ± 0.010 | 7.0 ± 0.06 |
| 2^{33} | [WHIR(1)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | \(2^{31}-2^{24}+1\) | 28 | 1 | 5.39 ± 1.93 | 35.3 ± 3.63 | 40.4 ± 5.05 | 9.0 ± 0.78 |
| 2^{33} | [WHIR(1)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | \(2^{31}-2^{24}+1\) | 28 | 8 | 1.68 ± 0.019 | 8.60 ± 0.100 | 10.3 ± 0.106 | 8.9 ± 0.06 |
| 2^{33} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | \(2^{31}-2^{24}+1\) | 28 | 1 | 9.41 ± 0.0093 | 1.58 ± 0.013 | 11.0 ± 0.011 | 22.4 ± 0.32 |
| 2^{33} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | \(2^{31}-2^{24}+1\) | 28 | 8 | 9.03 ± 0.167 | 1.39 ± 0.016 | 10.4 ± 0.155 | 22.8 ± 0.40 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | \(2^{32}-99\) | 30 | 1 | 25.6 ± 0.250 | 17.4 ± 0.357 | 43.0 ± 0.603 | 33.1 ± 0.88 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | \(2^{32}-99\) | 30 | 8 | 3.94 ± 1.00 | 3.68 ± 0.548 | 7.34 ± 1.39 | 11.9 ± 1.44 |
| 2^{35} | [WHIR(1)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | \(2^{31}-2^{24}+1\) | 30 | 1 | 16.7 ± 0.102 | 30.4 ± 0.026 | 47.1 ± 0.096 | 10.0 ± 0.08 |
| 2^{35} | [WHIR(1)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | \(2^{31}-2^{24}+1\) | 30 | 8 | 5.59 ± 0.018 | 8.74 ± 0.034 | 14.3 ± 0.037 | 10.1 ± 0.06 |
| 2^{35} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | \(2^{31}-2^{24}+1\) | 30 | 1 | 35.2 ± 0.549 | 3.49 ± 0.022 | 38.6 ± 0.541 | 30.3 ± 0.30 |
| 2^{35} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | \(2^{31}-2^{24}+1\) | 30 | 8 | 35.2 ± 0.588 | 2.96 ± 0.166 | 38.1 ± 0.718 | 31.0 ± 0.77 |

**(1)** WHIR uses unique decoding at this size so the 128-bit transcript-error
target still holds on KoalaBear. Capacity bound and Johnson bound need more
than 30 bits of grinding, which the field cannot support. The larger proof is
the unique-decoding query schedule.

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
| 2^{33} | [WHIR(1)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 33 | 575.3 | 575.3 | 9.60 | 9.60 | 0.0730 | 0.0000 |
| 2^{33} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 998.1 | 998.2 | 7.00 | 7.00 | 0.0000 | 0.0000 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53) | 343 | 67.6 | 68.0 | 24.7 | 24.7 | 0.0834 | 0.0215 |
| 2^{35} | [WHIR(1)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 33 | 676.6 | 676.6 | 17.1 | 17.1 | 0.0569 | 0.0000 |
| 2^{35} | [BaseFold](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 1346.1 | 1346.2 | 28.0 | 28.0 | 0.0000 | 0.0000 |

**(1)** WHIR uses unique decoding at this size so the 128-bit transcript-error
target still holds on KoalaBear. Capacity bound and Johnson bound need more
than 30 bits of grinding, which the field cannot support. The larger proof is
the unique-decoding query schedule.

```bash
cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval matrix
./scripts/fetch-vendors.sh --akita
./scripts/hash-eval.sh run --scheme whir --payload 31 --threads 1 --runs 1 --warmups 0
cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval compare \
  results/hash-x86_64 --out-dir results/hash-x86_64
```

Methodology and pins are in [docs/hash-eval.md](docs/hash-eval.md).

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
under `results/<timestamp>/` (gitignored except `results/lattice-x86_64/` and
`results/hash-x86_64/`).

## Adding another PCS

1. Keep the implementation out of Akita's crate graph.
2. Pin an immutable revision.
3. Emit the `WorkerOutput` JSON schema (or a documented native log).
4. Add a `SchemeId` and matrix policy in `pcs-bench-core`.
5. Document field, security, threading, and any payload mismatch.

See [methodology](docs/methodology.md) and [contributing](CONTRIBUTING.md).
