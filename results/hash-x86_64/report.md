Measurements were collected on a single AMD Ryzen 9 9950X 16-Core Processor (Linux x86_64, 32 logical CPUs, 121~GiB RAM). The CPU advertised AVX-512F, but the executed instruction stream was not independently traced. CPU policy: driver amd-pstate-epp, governor powersave, preference balance_performance.

Our second experiment compares Akita with other high-performance hash-based PCSs
on the same nominal dense payload ladder ($2^{27}$ through $2^{35}$ bits).
This is a measured-configuration survey, not an equivalent-security PCS ranking.
Nominal payload is field-capacity accounting, not a claim about sampled input entropy.
The table below records the accepted native profile and security accounting for every scheme.
KoalaBear univariate FRI/STIR pack into a
$2^{23}\times 2^{n-23}$ matrix when $\log_2 N>23$ (two-adicity 24 at rate $1/2$).
Timing cells report the median and, when supported by the sample count, a
conservative distribution-free 95% confidence interval at **1 and 8 threads**. Scheme names link to the exact git commit
that was measured. Unmeasured roster cells are pending.

The timing comparison separates commitment, opening, and verification, while the
cold total includes setup plus commitment and opening. Point-dependent claim and
transcript work supplied to proving is included in opening.
The resources table reports communication, memory (1-thread and 8-thread peak RSS),
and preprocessing. An OOM entry is a confirmed allocation failure under the 109 GiB virtual-address-space ceiling.

### Security and accepted profiles

| Scheme | Accepted profile | Security accounting |
| --- | --- | --- |
| Akita | Planner-selected direct/offloaded schedules at each native prime | 128-bit Module-SIS and 128-bit classical-ROM transcript target |
| Plonky2 FRI | Standard recursion: rate 1/8, 28 queries, 16 work bits | Approximately 100-bit conjectural FRI estimate |
| Plonky3 FRI | Upstream new_benchmark: rate 1/2, 100 queries, 16 query-PoW bits | 113.744-bit conjectural random-words estimate |
| Plonky3 STIR | Upstream PCS benchmark: rate 1/2, fold 4 throughout, at most 20 work bits per phase | 100-bit aggregate capacity/MCA target |
| Plonky3 WHIR | Closest feasible upstream PCS benchmark profile; capacity through $\log_2N=26$, unique decoding after | 128-bit round-by-round target under the pinned model |
| Binius64 BaseFold | Product default: rate 1/2, 232 queries, SHA-256 | 96-bit unique-decoding query target |
| Flock Ligerito | Default Fast: rate 1/2, Johnson, two OOD checks, SHA-256 | 128-bit round-by-round target |
| WorldFnd WHIR | CLI defaults: rate 1/2, fold 4, Johnson, BLAKE3 | 128-bit round-by-round target |
| SP1 BaseFold | Product default: rate 1/4, 124 queries, 16 work bits, stacking height 21 | 100-bit unique-decoding query target |

| Nominal payload | Scheme | Security target | Statement | Field | log₂ N | Threads | Commit (s) | Open (s) | Cold total (s) | Verify (ms) |
| ---: | --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 22 | 1 | 0.098 [0.097, 0.101] | 0.991 [0.986, 0.994] | 1.10 [1.09, 1.10] | 7.6 [7.6, 7.6] |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 22 | 8 | 0.021 [0.020, 0.021] | 0.262 [0.257, 0.268] | 0.284 [0.279, 0.291] | 6.0 [5.8, 6.4] |
| 2^{27} | [Akita (offload)(1)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | —(1) | 1 | —(1) | —(1) | —(1) | —(1) |
| 2^{27} | [Akita (offload)(1)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | —(1) | 8 | —(1) | —(1) | —(1) | —(1) |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | 21 | 1 | 0.071 [0.070, 0.073] | 0.603 [0.602, 0.609] | 0.691 [0.688, 0.698] | 6.0 [5.6, 6.1] |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | 21 | 8 | 0.019 [0.018, 0.020] | 0.172 [0.165, 0.179] | 0.194 [0.188, 0.203] | 4.5 [4.4, 5.2] |
| 2^{27} | [Akita (offload)(2)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | —(2) | 1 | —(2) | —(2) | —(2) | —(2) |
| 2^{27} | [Akita (offload)(2)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | —(2) | 8 | —(2) | —(2) | —(2) | —(2) |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | 20 | 1 | 0.127 [0.124, 0.128] | 0.511 [0.509, 0.512] | 0.655 [0.651, 0.656] | 5.7 [5.6, 5.7] |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | 20 | 8 | 0.027 [0.025, 0.027] | 0.150 [0.147, 0.155] | 0.180 [0.177, 0.186] | 4.6 [4.5, 4.9] |
| 2^{27} | [Akita (offload)(3)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | —(3) | 1 | —(3) | —(3) | —(3) | —(3) |
| 2^{27} | [Akita (offload)(3)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | —(3) | 8 | —(3) | —(3) | —(3) | —(3) |
| 2^{27} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | approx. 100-bit conjectural | univariate | $2^{64}-2^{32}+1$ | 21 | 1 | 14.3 [14.3, 14.3] | 5.26 [5.23, 5.29] | 19.6 [19.5, 19.6] | 1.9 [1.8, 1.9] |
| 2^{27} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | approx. 100-bit conjectural | univariate | $2^{64}-2^{32}+1$ | 21 | 8 | 2.25 [2.24, 2.26] | 1.77 [1.75, 1.77] | 4.02 [4.00, 4.03] | 1.9 [1.9, 1.9] |
| 2^{27} | [Plonky3 FRI](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 113.744-bit random-words conjectural | univariate | $2^{31}-2^{24}+1$ | 22 | 1 | 1.08 [1.08, 1.08] | 2.54 [2.53, 2.56] | 3.65 [3.64, 3.68] | 8.8 [8.7, 9.0] |
| 2^{27} | [Plonky3 FRI](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 113.744-bit random-words conjectural | univariate | $2^{31}-2^{24}+1$ | 22 | 8 | 0.215 [0.213, 0.220] | 0.532 [0.526, 0.543] | 0.773 [0.769, 0.783] | 8.8 [8.7, 8.9] |
| 2^{27} | [Plonky3 STIR](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 100-bit capacity | univariate | $2^{31}-2^{24}+1$ | 22 | 1 | 1.08 [1.07, 1.09] | 3.00 [2.96, 3.11] | 4.11 [4.07, 4.22] | 3.6 [3.5, 3.9] |
| 2^{27} | [Plonky3 STIR](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 100-bit capacity | univariate | $2^{31}-2^{24}+1$ | 22 | 8 | 0.217 [0.213, 0.220] | 0.693 [0.689, 0.708] | 0.935 [0.930, 0.950] | 3.7 [3.7, 3.7] |
| 2^{27} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 128-bit RBR | multilinear | $2^{31}-2^{24}+1$ | 22 | 1 | 0.089 [0.085, 0.090] | 0.494 [0.472, 0.518] | 0.584 [0.561, 0.608] | 1.7 [1.7, 1.7] |
| 2^{27} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 128-bit RBR | multilinear | $2^{31}-2^{24}+1$ | 22 | 8 | 0.024 [0.022, 0.025] | 0.113 [0.100, 0.120] | 0.138 [0.126, 0.144] | 1.8 [1.7, 1.8] |
| 2^{27} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 96-bit UDR query | multilinear | $F_{2^{128}}$ | 20 | 1 | 0.038 [0.037, 0.038] | 0.013 [0.013, 0.014] | 0.055 [0.054, 0.057] | 0.5 [0.5, 0.5] |
| 2^{27} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 96-bit UDR query | multilinear | $F_{2^{128}}$ | 20 | 8 | 0.007 [0.007, 0.007] | 0.007 [0.007, 0.008] | 0.019 [0.018, 0.019] | 0.5 [0.5, 0.5] |
| 2^{27} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 128-bit RBR | packed F128 MLE | $F_2$ | 27 | 1 | 0.034 [0.033, 0.035] | 0.101 [0.098, 0.103] | 0.135 [0.131, 0.138] | 1.3 [1.3, 1.5] |
| 2^{27} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 128-bit RBR | packed F128 MLE | $F_2$ | 27 | 8 | 0.006 [0.006, 0.007] | 0.023 [0.023, 0.025] | 0.030 [0.029, 0.032] | 1.3 [1.3, 1.4] |
| 2^{27} | [WHIR (WorldFnd)](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d) | 128-bit RBR | multilinear | $2^{64}-2^{32}+1$ | 21 | 1 | 0.382 [0.376, 0.386] | 2.17 [2.16, 2.19] | 2.56 [2.54, 2.57] | 0.9 [0.9, 1.3] |
| 2^{27} | [WHIR (WorldFnd)](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d) | 128-bit RBR | multilinear | $2^{64}-2^{32}+1$ | 21 | 8 | 0.066 [0.065, 0.068] | 0.339 [0.336, 0.342] | 0.406 [0.402, 0.409] | 1.0 [1.0, 1.0] |
| 2^{27} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 100-bit UDR query | multilinear | $2^{31}-2^{24}+1$ | 22 | 1 | 2.13 [2.08, 2.22] | 3.45 [3.40, 3.61] | 5.57 [5.49, 5.82] | 25.8 [25.5, 26.1] |
| 2^{27} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 100-bit UDR query | multilinear | $2^{31}-2^{24}+1$ | 22 | 8 | 2.15 [2.08, 2.23] | 3.33 [3.29, 3.47] | 5.51 [5.37, 5.69] | 25.7 [25.5, 26.1] |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 24 | 1 | 0.343 [0.341, 0.346] | 1.51 [1.51, 1.52] | 1.86 [1.86, 1.87] | 9.3 [9.2, 9.3] |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 24 | 8 | 0.055 [0.054, 0.056] | 0.352 [0.348, 0.357] | 0.409 [0.405, 0.415] | 6.7 [6.6, 6.9] |
| 2^{29} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 24 | 1 | 0.338 [0.337, 0.343] | 2.02 [2.02, 2.02] | 2.39 [2.38, 2.39] | 8.1 [8.1, 8.4] |
| 2^{29} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 24 | 8 | 0.052 [0.051, 0.053] | 0.419 [0.412, 0.428] | 0.480 [0.470, 0.490] | 6.4 [6.4, 6.5] |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | 23 | 1 | 0.226 [0.224, 0.230] | 1.04 [1.03, 1.04] | 1.28 [1.27, 1.28] | 8.0 [7.8, 8.3] |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | 23 | 8 | 0.041 [0.040, 0.041] | 0.268 [0.265, 0.272] | 0.312 [0.309, 0.316] | 5.3 [5.3, 5.8] |
| 2^{29} | [Akita (offload)(4)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | —(4) | 1 | —(4) | —(4) | —(4) | —(4) |
| 2^{29} | [Akita (offload)(4)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | —(4) | 8 | —(4) | —(4) | —(4) | —(4) |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | 22 | 1 | 0.433 [0.432, 0.436] | 0.867 [0.864, 0.872] | 1.32 [1.32, 1.33] | 6.7 [6.5, 6.8] |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | 22 | 8 | 0.069 [0.068, 0.072] | 0.204 [0.198, 0.210] | 0.278 [0.271, 0.286] | 4.6 [4.6, 4.8] |
| 2^{29} | [Akita (offload)(5)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | —(5) | 1 | —(5) | —(5) | —(5) | —(5) |
| 2^{29} | [Akita (offload)(5)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | —(5) | 8 | —(5) | —(5) | —(5) | —(5) |
| 2^{29} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | approx. 100-bit conjectural | univariate | $2^{64}-2^{32}+1$ | 23 | 1 | 57.2 [57.2, 57.3] | 21.0 [21.0, 21.1] | 78.3 [78.2, 78.3] | 2.3 [2.2, 2.3] |
| 2^{29} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | approx. 100-bit conjectural | univariate | $2^{64}-2^{32}+1$ | 23 | 8 | 9.02 [9.01, 9.05] | 7.21 [7.21, 7.23] | 16.2 [16.2, 16.3] | 2.3 [2.3, 2.3] |
| 2^{29} | [Plonky3 FRI(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 113.744-bit random-words conjectural | univariate batch | $2^{31}-2^{24}+1$ | 24 | 1 | 2.36 [2.35, 2.36] | 5.09 [5.08, 5.10] | 7.51 [7.49, 7.53] | 9.6 [9.5, 9.8] |
| 2^{29} | [Plonky3 FRI(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 113.744-bit random-words conjectural | univariate batch | $2^{31}-2^{24}+1$ | 24 | 8 | 0.487 [0.483, 0.491] | 1.07 [1.06, 1.08] | 1.61 [1.60, 1.62] | 9.7 [9.5, 9.9] |
| 2^{29} | [Plonky3 STIR(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 100-bit capacity | univariate batch | $2^{31}-2^{24}+1$ | 24 | 1 | 2.36 [2.35, 2.37] | 6.01 [5.92, 6.13] | 8.44 [8.35, 8.54] | 3.8 [3.7, 4.2] |
| 2^{29} | [Plonky3 STIR(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 100-bit capacity | univariate batch | $2^{31}-2^{24}+1$ | 24 | 8 | 0.486 [0.480, 0.489] | 1.44 [1.42, 1.46] | 1.98 [1.97, 1.99] | 3.9 [3.8, 3.9] |
| 2^{29} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 128-bit RBR | multilinear | $2^{31}-2^{24}+1$ | 24 | 1 | 0.368 [0.366, 0.375] | 8.93 [7.10, 15.1] | 9.32 [7.47, 15.5] | 1.9 [1.9, 1.9] |
| 2^{29} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 128-bit RBR | multilinear | $2^{31}-2^{24}+1$ | 24 | 8 | 0.110 [0.109, 0.111] | 1.51 [1.20, 2.12] | 1.63 [1.32, 2.23] | 2.0 [2.0, 2.1] |
| 2^{29} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 96-bit UDR query | multilinear | $F_{2^{128}}$ | 22 | 1 | 0.162 [0.158, 0.162] | 0.051 [0.050, 0.053] | 0.229 [0.224, 0.232] | 0.7 [0.7, 0.7] |
| 2^{29} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 96-bit UDR query | multilinear | $F_{2^{128}}$ | 22 | 8 | 0.039 [0.038, 0.040] | 0.026 [0.024, 0.029] | 0.080 [0.078, 0.083] | 0.7 [0.7, 0.7] |
| 2^{29} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 128-bit RBR | packed F128 MLE | $F_2$ | 29 | 1 | 0.116 [0.115, 0.116] | 0.416 [0.410, 0.422] | 0.533 [0.527, 0.538] | 1.4 [1.4, 1.4] |
| 2^{29} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 128-bit RBR | packed F128 MLE | $F_2$ | 29 | 8 | 0.032 [0.031, 0.033] | 0.092 [0.091, 0.095] | 0.125 [0.123, 0.127] | 1.4 [1.4, 1.5] |
| 2^{29} | [WHIR (WorldFnd)](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d) | 128-bit RBR | multilinear | $2^{64}-2^{32}+1$ | 23 | 1 | 1.73 [1.72, 1.74] | 9.21 [9.18, 9.27] | 10.9 [10.9, 11.0] | 1.0 [1.0, 1.0] |
| 2^{29} | [WHIR (WorldFnd)](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d) | 128-bit RBR | multilinear | $2^{64}-2^{32}+1$ | 23 | 8 | 0.298 [0.295, 0.304] | 1.43 [1.43, 1.44] | 1.73 [1.72, 1.74] | 1.1 [1.1, 1.1] |
| 2^{29} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 100-bit UDR query | multilinear | $2^{31}-2^{24}+1$ | 24 | 1 | 2.90 [2.87, 3.04] | 3.89 [3.82, 3.99] | 6.78 [6.72, 7.12] | 25.8 [25.6, 26.1] |
| 2^{29} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 100-bit UDR query | multilinear | $2^{31}-2^{24}+1$ | 24 | 8 | 2.97 [2.90, 3.04] | 3.69 [3.63, 3.77] | 6.62 [6.55, 6.79] | 26.1 [25.6, 26.4] |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 26 | 1 | 1.28 [1.27, 1.28] | 2.63 [2.60, 2.64] | 3.93 [3.89, 3.94] | 12.8 [12.2, 13.1] |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 26 | 8 | 0.182 [0.181, 0.186] | 0.538 [0.533, 0.545] | 0.724 [0.717, 0.734] | 7.5 [7.2, 7.8] |
| 2^{31} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 26 | 1 | 1.27 [1.27, 1.28] | 3.28 [3.26, 3.28] | 4.62 [4.62, 4.64] | 12.1 [11.6, 12.2] |
| 2^{31} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 26 | 8 | 0.182 [0.179, 0.185] | 0.673 [0.669, 0.676] | 0.873 [0.866, 0.875] | 8.8 [8.7, 9.3] |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | 25 | 1 | 0.818 [0.813, 0.823] | 1.90 [1.87, 1.90] | 2.74 [2.71, 2.74] | 10.6 [9.9, 11.2] |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | 25 | 8 | 0.124 [0.124, 0.132] | 0.422 [0.418, 0.426] | 0.550 [0.547, 0.561] | 5.9 [5.8, 6.2] |
| 2^{31} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | 25 | 1 | 0.809 [0.806, 0.811] | 2.36 [2.35, 2.36] | 3.24 [3.23, 3.24] | 8.1 [8.0, 8.4] |
| 2^{31} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | 25 | 8 | 0.123 [0.121, 0.129] | 0.510 [0.504, 0.523] | 0.655 [0.648, 0.666] | 5.6 [5.6, 6.1] |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | 24 | 1 | 2.34 [2.33, 2.35] | 1.75 [1.75, 1.76] | 4.15 [4.13, 4.15] | 10.7 [10.6, 10.8] |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | 24 | 8 | 0.345 [0.343, 0.347] | 0.383 [0.376, 0.386] | 0.737 [0.733, 0.739] | 5.9 [5.8, 6.2] |
| 2^{31} | [Akita (offload)(7)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | —(7) | 1 | —(7) | —(7) | —(7) | —(7) |
| 2^{31} | [Akita (offload)(7)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | —(7) | 8 | —(7) | —(7) | —(7) | —(7) |
| 2^{31} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | approx. 100-bit conjectural | univariate | $2^{64}-2^{32}+1$ | 25 | 1 | 238.7 [238.4, 238.9] | 84.7 [84.6, 84.8] | 323.4 [323.0, 323.7] | 2.5 [2.5, 2.5] |
| 2^{31} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | approx. 100-bit conjectural | univariate | $2^{64}-2^{32}+1$ | 25 | 8 | 38.4 [38.4, 38.6] | 29.6 [29.5, 29.6] | 68.0 [67.9, 68.2] | 2.5 [2.5, 2.5] |
| 2^{31} | [Plonky3 FRI(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 113.744-bit random-words conjectural | univariate batch | $2^{31}-2^{24}+1$ | 26 | 1 | 2.70 [2.68, 2.70] | 5.11 [5.09, 5.11] | 7.86 [7.83, 7.87] | 9.7 [9.6, 9.8] |
| 2^{31} | [Plonky3 FRI(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 113.744-bit random-words conjectural | univariate batch | $2^{31}-2^{24}+1$ | 26 | 8 | 0.677 [0.673, 0.679] | 1.06 [1.05, 1.06] | 1.79 [1.78, 1.80] | 9.7 [9.6, 9.8] |
| 2^{31} | [Plonky3 STIR(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 100-bit capacity | univariate batch | $2^{31}-2^{24}+1$ | 26 | 1 | 2.70 [2.69, 2.71] | 5.95 [5.93, 6.04] | 8.71 [8.68, 8.79] | 3.9 [3.7, 4.2] |
| 2^{31} | [Plonky3 STIR(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 100-bit capacity | univariate batch | $2^{31}-2^{24}+1$ | 26 | 8 | 0.673 [0.670, 0.677] | 1.43 [1.41, 1.44] | 2.16 [2.14, 2.17] | 3.9 [3.9, 3.9] |
| 2^{31} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 128-bit RBR | multilinear | $2^{31}-2^{24}+1$ | 26 | 1 | 1.50 [1.49, 1.51] | 37.8 [31.6, 49.3] | 39.3 [33.2, 50.8] | 2.1 [2.1, 2.1] |
| 2^{31} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 128-bit RBR | multilinear | $2^{31}-2^{24}+1$ | 26 | 8 | 0.476 [0.473, 0.482] | 6.25 [5.10, 7.53] | 6.75 [5.61, 8.03] | 2.2 [2.2, 2.2] |
| 2^{31} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 96-bit UDR query | multilinear | $F_{2^{128}}$ | 24 | 1 | 0.683 [0.678, 0.688] | 0.207 [0.205, 0.208] | 0.958 [0.950, 0.961] | 0.8 [0.8, 0.8] |
| 2^{31} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 96-bit UDR query | multilinear | $F_{2^{128}}$ | 24 | 8 | 0.205 [0.201, 0.212] | 0.104 [0.099, 0.110] | 0.377 [0.372, 0.389] | 0.8 [0.8, 0.8] |
| 2^{31} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 128-bit RBR | packed F128 MLE | $F_2$ | 31 | 1 | 0.574 [0.570, 0.576] | 1.42 [1.42, 1.43] | 1.99 [1.99, 2.01] | 1.1 [1.1, 1.1] |
| 2^{31} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 128-bit RBR | packed F128 MLE | $F_2$ | 31 | 8 | 0.124 [0.123, 0.128] | 0.300 [0.299, 0.306] | 0.427 [0.422, 0.434] | 1.1 [1.1, 1.1] |
| 2^{31} | [WHIR (WorldFnd)](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d) | 128-bit RBR | multilinear | $2^{64}-2^{32}+1$ | 25 | 1 | 7.58 [7.52, 7.63] | 39.8 [39.7, 40.0] | 47.4 [47.2, 47.5] | 1.2 [1.1, 1.2] |
| 2^{31} | [WHIR (WorldFnd)](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d) | 128-bit RBR | multilinear | $2^{64}-2^{32}+1$ | 25 | 8 | 1.31 [1.30, 1.32] | 6.26 [6.24, 6.28] | 7.56 [7.55, 7.59] | 1.2 [1.2, 1.2] |
| 2^{31} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 100-bit UDR query | multilinear | $2^{31}-2^{24}+1$ | 26 | 1 | 5.41 [5.20, 5.47] | 5.53 [5.41, 5.61] | 10.9 [10.6, 11.1] | 26.2 [25.9, 26.6] |
| 2^{31} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 100-bit UDR query | multilinear | $2^{31}-2^{24}+1$ | 26 | 8 | 5.31 [5.19, 5.45] | 4.77 [4.68, 4.87] | 10.1 [9.87, 10.3] | 26.3 [25.9, 26.7] |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 28 | 1 | 5.99 [5.95, 6.01] | 6.66 [6.64, 6.68] | 12.7 [12.6, 12.7] | 22.1 [21.5, 22.2] |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 28 | 8 | 0.797 [0.795, 0.804] | 1.19 [1.19, 1.20] | 2.00 [1.99, 2.01] | 9.5 [9.4, 9.6] |
| 2^{33} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 28 | 1 | 5.99 [5.94, 6.01] | 7.86 [7.85, 7.86] | 14.1 [14.0, 14.1] | 14.5 [14.4, 14.7] |
| 2^{33} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 28 | 8 | 0.794 [0.792, 0.797] | 1.38 [1.36, 1.38] | 2.22 [2.21, 2.22] | 8.7 [8.4, 9.2] |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | 27 | 1 | 6.70 [6.63, 6.71] | 4.42 [4.39, 4.42] | 11.2 [11.1, 11.2] | 17.6 [16.5, 18.0] |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | 27 | 8 | 0.939 [0.935, 0.942] | 0.850 [0.844, 0.858] | 1.80 [1.79, 1.80] | 7.0 [6.8, 7.1] |
| 2^{33} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | 27 | 1 | 6.63 [6.63, 6.69] | 4.97 [4.96, 4.98] | 11.9 [11.9, 11.9] | 12.7 [12.0, 12.9] |
| 2^{33} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | 27 | 8 | 0.935 [0.931, 0.939] | 0.979 [0.967, 0.990] | 1.97 [1.96, 1.98] | 7.5 [7.2, 7.8] |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | 26 | 1 | 4.83 [4.82, 4.83] | 4.42 [4.41, 4.43] | 9.35 [9.34, 9.36] | 15.5 [15.3, 15.9] |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | 26 | 8 | 0.739 [0.732, 0.769] | 0.772 [0.767, 0.778] | 1.53 [1.52, 1.56] | 6.9 [6.9, 7.0] |
| 2^{33} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | 26 | 1 | 4.82 [4.82, 4.85] | 5.07 [5.06, 5.08] | 10.3 [10.2, 10.3] | 10.4 [10.0, 10.7] |
| 2^{33} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | 26 | 8 | 0.735 [0.731, 0.738] | 0.918 [0.913, 0.923] | 1.74 [1.73, 1.74] | 6.0 [5.9, 6.2] |
| 2^{33} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | approx. 100-bit conjectural | univariate | $2^{64}-2^{32}+1$ | 27 | 1 | OOM | OOM | OOM | OOM |
| 2^{33} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | approx. 100-bit conjectural | univariate | $2^{64}-2^{32}+1$ | 27 | 8 | OOM | OOM | OOM | OOM |
| 2^{33} | [Plonky3 FRI(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 113.744-bit random-words conjectural | univariate batch | $2^{31}-2^{24}+1$ | 28 | 1 | 5.34 [5.31, 5.36] | 5.21 [5.18, 5.24] | 10.6 [10.6, 10.6] | 9.8 [9.7, 9.9] |
| 2^{33} | [Plonky3 FRI(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 113.744-bit random-words conjectural | univariate batch | $2^{31}-2^{24}+1$ | 28 | 8 | 1.57 [1.55, 1.58] | 1.05 [1.05, 1.06] | 2.68 [2.66, 2.69] | 9.8 [9.7, 10.0] |
| 2^{33} | [Plonky3 STIR(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 100-bit capacity | univariate batch | $2^{31}-2^{24}+1$ | 28 | 1 | 5.37 [5.35, 5.38] | 6.09 [6.03, 6.18] | 11.5 [11.4, 11.6] | 3.8 [3.8, 3.9] |
| 2^{33} | [Plonky3 STIR(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 100-bit capacity | univariate batch | $2^{31}-2^{24}+1$ | 28 | 8 | 1.56 [1.56, 1.57] | 1.42 [1.41, 1.44] | 3.04 [3.03, 3.05] | 4.0 [3.9, 4.0] |
| 2^{33} | [WHIR (Plonky3)(8)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 128-bit RBR | multilinear | $2^{31}-2^{24}+1$ | 28 | 1 | 4.96 [4.95, 4.97] | 26.4 [26.3, 26.5] | 31.4 [31.3, 31.5] | 10.0 [10.0, 10.2] |
| 2^{33} | [WHIR (Plonky3)(8)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 128-bit RBR | multilinear | $2^{31}-2^{24}+1$ | 28 | 8 | 1.63 [1.63, 1.65] | 7.77 [7.75, 7.80] | 9.46 [9.45, 9.50] | 10.2 [10.1, 10.2] |
| 2^{33} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 96-bit UDR query | multilinear | $F_{2^{128}}$ | 26 | 1 | 2.88 [2.87, 2.89] | 0.788 [0.784, 0.795] | 3.94 [3.93, 3.95] | 1.0 [1.0, 1.0] |
| 2^{33} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 96-bit UDR query | multilinear | $F_{2^{128}}$ | 26 | 8 | 1.08 [1.06, 1.08] | 0.393 [0.390, 0.399] | 1.74 [1.73, 1.75] | 1.0 [1.0, 1.0] |
| 2^{33} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 128-bit RBR | packed F128 MLE | $F_2$ | 33 | 1 | 1.86 [1.85, 1.87] | 5.70 [5.67, 5.72] | 7.55 [7.53, 7.58] | 1.7 [1.7, 2.0] |
| 2^{33} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 128-bit RBR | packed F128 MLE | $F_2$ | 33 | 8 | 0.583 [0.577, 0.588] | 1.21 [1.20, 1.22] | 1.79 [1.78, 1.80] | 1.8 [1.7, 1.8] |
| 2^{33} | [WHIR (WorldFnd)](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d) | 128-bit RBR | multilinear | $2^{64}-2^{32}+1$ | 27 | 1 | 33.7 [33.6, 34.0] | 172.7 [172.4, 172.8] | 206.4 [206.1, 206.7] | 1.2 [1.2, 1.3] |
| 2^{33} | [WHIR (WorldFnd)](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d) | 128-bit RBR | multilinear | $2^{64}-2^{32}+1$ | 27 | 8 | 5.74 [5.72, 5.75] | 26.8 [26.7, 26.9] | 32.6 [32.5, 32.7] | 1.3 [1.2, 1.3] |
| 2^{33} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 100-bit UDR query | multilinear | $2^{31}-2^{24}+1$ | 28 | 1 | 18.8 [18.5, 19.0] | 11.9 [11.8, 12.0] | 30.7 [30.3, 31.0] | 27.3 [26.9, 27.5] |
| 2^{33} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 100-bit UDR query | multilinear | $2^{31}-2^{24}+1$ | 28 | 8 | 18.8 [18.3, 18.9] | 9.22 [9.19, 9.38] | 28.0 [27.6, 28.3] | 27.2 [27.0, 27.5] |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 30 | 1 | 24.2 [24.1, 24.2] | 16.1 [16.1, 16.1] | 40.4 [40.3, 40.4] | 33.3 [33.2, 33.4] |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 30 | 8 | 3.45 [3.44, 3.45] | 2.72 [2.72, 2.73] | 6.18 [6.17, 6.19] | 11.3 [11.2, 11.5] |
| 2^{35} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 30 | 1 | 24.1 [24.0, 24.2] | 18.6 [18.6, 18.6] | 43.2 [43.1, 43.3] | 16.6 [15.7, 16.7] |
| 2^{35} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{32}-99$ | 30 | 8 | 3.44 [3.43, 3.45] | 3.00 [2.99, 3.01] | 6.53 [6.53, 6.55] | 10.1 [10.0, 10.3] |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | 29 | 1 | 26.4 [26.1, 26.4] | 10.5 [10.5, 10.5] | 37.0 [36.8, 37.0] | 20.9 [20.1, 21.8] |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | 29 | 8 | 3.78 [3.76, 3.79] | 1.79 [1.78, 1.79] | 5.58 [5.55, 5.60] | 7.6 [7.5, 7.7] |
| 2^{35} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | 29 | 1 | 26.4 [26.1, 26.4] | 11.4 [11.3, 11.4] | 38.2 [37.9, 38.2] | 14.4 [14.3, 14.9] |
| 2^{35} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{64}-59$ | 29 | 8 | 3.77 [3.76, 3.79] | 2.00 [1.99, 2.01] | 5.88 [5.86, 5.90] | 8.9 [8.7, 9.5] |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | 28 | 1 | 18.8 [18.8, 18.9] | 11.8 [11.7, 11.8] | 30.8 [30.7, 30.8] | 24.2 [23.9, 24.3] |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | 28 | 8 | 2.90 [2.89, 2.92] | 1.82 [1.81, 1.83] | 4.76 [4.75, 4.77] | 8.6 [8.4, 9.1] |
| 2^{35} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | 28 | 1 | 18.5 [18.5, 18.5] | 12.1 [12.1, 12.1] | 31.5 [31.5, 31.5] | 12.2 [12.2, 12.4] |
| 2^{35} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128-bit Module-SIS/ROM | multilinear | $2^{128}-2^{32}+22537$ | 28 | 8 | 2.89 [2.89, 2.90] | 2.03 [2.02, 2.05] | 5.11 [5.10, 5.12] | 7.1 [7.0, 7.1] |
| 2^{35} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | approx. 100-bit conjectural | univariate | $2^{64}-2^{32}+1$ | 29 | 1 | OOM | OOM | OOM | OOM |
| 2^{35} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | approx. 100-bit conjectural | univariate | $2^{64}-2^{32}+1$ | 29 | 8 | OOM | OOM | OOM | OOM |
| 2^{35} | [Plonky3 FRI(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 113.744-bit random-words conjectural | univariate batch | $2^{31}-2^{24}+1$ | 30 | 1 | 18.3 [18.2, 18.4] | 6.56 [6.50, 6.60] | 24.9 [24.8, 25.0] | 10.2 [10.0, 10.4] |
| 2^{35} | [Plonky3 FRI(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 113.744-bit random-words conjectural | univariate batch | $2^{31}-2^{24}+1$ | 30 | 8 | 5.90 [5.89, 5.91] | 1.27 [1.27, 1.28] | 7.23 [7.21, 7.23] | 10.2 [10.1, 10.4] |
| 2^{35} | [Plonky3 STIR(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 100-bit capacity | univariate batch | $2^{31}-2^{24}+1$ | 30 | 1 | 18.3 [18.3, 18.4] | 7.42 [7.32, 7.48] | 25.8 [25.6, 25.9] | 4.3 [4.2, 4.4] |
| 2^{35} | [Plonky3 STIR(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 100-bit capacity | univariate batch | $2^{31}-2^{24}+1$ | 30 | 8 | 5.89 [5.88, 5.91] | 1.65 [1.64, 1.66] | 7.60 [7.57, 7.62] | 4.4 [4.4, 4.4] |
| 2^{35} | [WHIR (Plonky3)(8)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 128-bit RBR | multilinear | $2^{31}-2^{24}+1$ | 30 | 1 | 16.6 [16.4, 16.6] | 28.8 [28.6, 29.0] | 45.4 [45.2, 45.7] | 11.4 [11.3, 11.5] |
| 2^{35} | [WHIR (Plonky3)(8)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 128-bit RBR | multilinear | $2^{31}-2^{24}+1$ | 30 | 8 | 5.53 [5.51, 5.55] | 8.38 [8.35, 8.44] | 14.0 [14.0, 14.0] | 11.6 [11.5, 11.8] |
| 2^{35} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 96-bit UDR query | multilinear | $F_{2^{128}}$ | 28 | 1 | 12.1 [12.1, 12.1] | 2.94 [2.94, 2.97] | 16.1 [16.1, 16.1] | 1.1 [1.1, 1.1] |
| 2^{35} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 96-bit UDR query | multilinear | $F_{2^{128}}$ | 28 | 8 | 5.02 [5.01, 5.03] | 1.45 [1.44, 1.46] | 7.50 [7.49, 7.51] | 1.1 [1.1, 1.1] |
| 2^{35} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 128-bit RBR | packed F128 MLE | $F_2$ | 35 | 1 | 9.73 [9.69, 9.79] | 23.6 [23.5, 23.9] | 33.3 [33.3, 33.6] | 1.7 [1.7, 1.7] |
| 2^{35} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 128-bit RBR | packed F128 MLE | $F_2$ | 35 | 8 | 2.38 [2.37, 2.39] | 5.21 [5.19, 5.22] | 7.59 [7.57, 7.60] | 1.6 [1.6, 1.8] |
| 2^{35} | [WHIR (WorldFnd)](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d) | 128-bit RBR | multilinear | $2^{64}-2^{32}+1$ | 29 | 1 | 145.4 [143.1, 147.1] | 764.7 [758.4, 766.4] | 910.0 [902.6, 912.8] | 1.4 [1.4, 1.4] |
| 2^{35} | [WHIR (WorldFnd)](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d) | 128-bit RBR | multilinear | $2^{64}-2^{32}+1$ | 29 | 8 | 24.7 [24.4, 25.0] | 115.9 [115.6, 116.4] | 140.8 [140.2, 141.0] | 1.4 [1.4, 1.4] |
| 2^{35} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 100-bit UDR query | multilinear | $2^{31}-2^{24}+1$ | 30 | 1 | 67.5 [66.0, 67.9] | 36.7 [36.7, 36.8] | 104.3 [102.7, 104.7] | 31.8 [31.5, 32.1] |
| 2^{35} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 100-bit UDR query | multilinear | $2^{31}-2^{24}+1$ | 30 | 8 | 67.5 [65.9, 67.8] | 26.5 [26.4, 26.6] | 94.0 [92.4, 94.4] | 31.8 [31.6, 32.1] |

**(1)** pinned Akita fp32-dense-recursive catalog row for nv=22 has no setup-prefix edges
**(2)** pinned Akita fp64-dense-recursive catalog row for nv=21 has no setup-prefix edges
**(3)** pinned Akita fp128-dense-recursive catalog row for nv=20 has no setup-prefix edges
**(4)** pinned Akita fp64-dense-recursive catalog row for nv=23 has no setup-prefix edges
**(5)** pinned Akita fp128-dense-recursive catalog row for nv=22 has no setup-prefix edges
**(6)** KoalaBear two-adicity is 24, so a rate-$1/2$ univariate cannot be a single degree-$2^{n}$ polynomial when $\log_2 N>23$. The worker packs the $2^{n}$ coefficients into a trace matrix of height $2^{23}$ and width $2^{n-23}$. That is batched univariate FRI/STIR, not one tall polynomial.
**(7)** pinned Akita fp128-dense-recursive catalog row for nv=24 has no setup-prefix edges
**(8)** WHIR uses unique decoding at this size so its 128-bit round-by-round target remains feasible on KoalaBear. Capacity bound and Johnson bound need more than 30 bits of grinding, which the field cannot support. The larger proof is the unique-decoding query schedule.


| Nominal payload | Scheme | Commitment (B) | Evaluation (B) | Proof (B) | Total sent (B) | Excluded context (B) | Peak RSS 1-thread (GiB) | Peak RSS 8-thread (GiB) | Prep. (s) | State (GiB) |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 61284 | 61428 | 214 | 0.127 | 0.136 | 0.0081 | 0.0020 |
| 2^{27} | [Akita (offload)(1)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) | —(1) |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 65816 | 65960 | 214 | 0.122 | 0.132 | 0.0162 | 0.0044 |
| 2^{27} | [Akita (offload)(2)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | —(2) | —(2) | —(2) | —(2) | —(2) | —(2) | —(2) | —(2) | —(2) |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 65746 | 65890 | 214 | 0.125 | 0.140 | 0.0167 | 0.0049 |
| 2^{27} | [Akita (offload)(3)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | —(3) | —(3) | —(3) | —(3) | —(3) | —(3) | —(3) | —(3) | —(3) |
| 2^{27} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | 520 | 16 | 87968 | 88504 | 0 | 2.79 | 2.79 | 0.0000 | 0.0000 |
| 2^{27} | [Plonky3 FRI](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 32 | 20 | 462174 | 462226 | 0 | 1.75 | 1.75 | 0.0274 | unknown |
| 2^{27} | [Plonky3 STIR](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 34 | 20 | 177788 | 177842 | 0 | 1.41 | 1.41 | 0.0275 | unknown |
| 2^{27} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 32 | 0 | 91354 | 91386 | 0 | 0.179 | 0.178 | 0.0013 | unknown |
| 2^{27} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 32 | 16 | 308640 | 308688 | 0 | 0.0917 | 0.0913 | 0.0041 | unknown |
| 2^{27} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 4136 | 16 | 408856 | 413008 | 0 | 0.0912 | 0.0912 | 0.0001 | 0.0000 |
| 2^{27} | [WHIR (WorldFnd)](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d) | 56 | 24 | 196640 | 196720 | 0 | 0.190 | 0.197 | 0.0002 | 0.0000 |
| 2^{27} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 16 | 1179448 | 1179496 | 0 | 1.33 | 1.33 | 0.0000 | unknown |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 61787 | 61931 | 214 | 0.253 | 0.261 | 0.0081 | 0.0020 |
| 2^{29} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 66157 | 66301 | 214 | 0.246 | 0.255 | 0.0297 | 0.0020 |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 66204 | 66348 | 214 | 0.219 | 0.230 | 0.0156 | 0.0044 |
| 2^{29} | [Akita (offload)(4)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | —(4) | —(4) | —(4) | —(4) | —(4) | —(4) | —(4) | —(4) | —(4) |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 66572 | 66716 | 214 | 0.217 | 0.228 | 0.0195 | 0.0059 |
| 2^{29} | [Akita (offload)(5)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | —(5) | —(5) | —(5) | —(5) | —(5) | —(5) | —(5) | —(5) | —(5) |
| 2^{29} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | 520 | 16 | 106472 | 107008 | 0 | 11.2 | 11.2 | 0.0000 | 0.0000 |
| 2^{29} | [Plonky3 FRI(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 32 | 40 | 512330 | 512402 | 0 | 3.57 | 3.57 | 0.0621 | unknown |
| 2^{29} | [Plonky3 STIR(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 34 | 40 | 190380 | 190454 | 0 | 2.88 | 2.88 | 0.0609 | unknown |
| 2^{29} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 32 | 0 | 103390 | 103422 | 0 | 0.703 | 0.703 | 0.0054 | unknown |
| 2^{29} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 32 | 16 | 390368 | 390416 | 0 | 0.357 | 0.357 | 0.0161 | unknown |
| 2^{29} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 4136 | 16 | 340832 | 344984 | 0 | 0.373 | 0.372 | 0.0002 | 0.0000 |
| 2^{29} | [WHIR (WorldFnd)](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d) | 56 | 24 | 221568 | 221648 | 0 | 0.748 | 0.780 | 0.0002 | 0.0000 |
| 2^{29} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 16 | 1182520 | 1182568 | 0 | 1.56 | 1.56 | 0.0000 | unknown |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 63055 | 63199 | 214 | 0.694 | 0.703 | 0.0196 | 0.0049 |
| 2^{31} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 66318 | 66462 | 214 | 0.717 | 0.724 | 0.0734 | 0.0039 |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 66662 | 66806 | 214 | 0.484 | 0.500 | 0.0209 | 0.0059 |
| 2^{31} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 73603 | 73747 | 214 | 0.518 | 0.535 | 0.0706 | 0.0078 |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 68326 | 68470 | 214 | 0.812 | 0.834 | 0.0506 | 0.0156 |
| 2^{31} | [Akita (offload)(7)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | —(7) | —(7) | —(7) | —(7) | —(7) | —(7) | —(7) | —(7) | —(7) |
| 2^{31} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | 520 | 16 | 117608 | 118144 | 0 | 44.6 | 44.6 | 0.0000 | 0.0000 |
| 2^{31} | [Plonky3 FRI(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 32 | 160 | 520682 | 520874 | 0 | 3.94 | 3.94 | 0.0620 | unknown |
| 2^{31} | [Plonky3 STIR(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 34 | 160 | 192388 | 192582 | 0 | 3.25 | 3.25 | 0.0619 | unknown |
| 2^{31} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 32 | 0 | 113580 | 113612 | 0 | 2.80 | 2.80 | 0.0277 | unknown |
| 2^{31} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 32 | 16 | 472832 | 472880 | 0 | 1.42 | 1.42 | 0.0670 | unknown |
| 2^{31} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 4136 | 16 | 494632 | 498784 | 0 | 1.37 | 1.37 | 0.0002 | 0.0000 |
| 2^{31} | [WHIR (WorldFnd)](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d) | 56 | 24 | 256856 | 256936 | 0 | 2.98 | 3.12 | 0.0002 | 0.0000 |
| 2^{31} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 16 | 1194808 | 1194856 | 0 | 3.50 | 3.50 | 0.0000 | unknown |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 64475 | 64619 | 214 | 1.37 | 1.38 | 0.0383 | 0.0098 |
| 2^{33} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 66872 | 67016 | 214 | 1.39 | 1.40 | 0.228 | 0.0156 |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 67628 | 67772 | 214 | 1.49 | 1.53 | 0.0538 | 0.0156 |
| 2^{33} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 70589 | 70733 | 214 | 1.50 | 1.54 | 0.264 | 0.0156 |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 68897 | 69041 | 214 | 1.58 | 1.61 | 0.101 | 0.0313 |
| 2^{33} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 71210 | 71354 | 214 | 1.60 | 1.63 | 0.371 | 0.0313 |
| 2^{33} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | OOM | OOM | OOM | OOM | OOM | OOM | OOM | OOM | OOM |
| 2^{33} | [Plonky3 FRI(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 32 | 640 | 528650 | 529322 | 0 | 5.44 | 5.44 | 0.0622 | unknown |
| 2^{33} | [Plonky3 STIR(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 34 | 640 | 200548 | 201222 | 0 | 4.75 | 4.75 | 0.0617 | unknown |
| 2^{33} | [WHIR (Plonky3)(8)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 32 | 0 | 590648 | 590680 | 0 | 9.60 | 9.60 | 0.0633 | unknown |
| 2^{33} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 32 | 16 | 569408 | 569456 | 0 | 5.67 | 5.67 | 0.272 | unknown |
| 2^{33} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 4136 | 16 | 524840 | 528992 | 0 | 5.41 | 5.42 | 0.0002 | 0.0000 |
| 2^{33} | [WHIR (WorldFnd)](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d) | 56 | 24 | 281464 | 281544 | 0 | 11.9 | 11.9 | 0.0002 | 0.0000 |
| 2^{33} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 16 | 1243960 | 1244008 | 0 | 12.5 | 12.5 | 0.0000 | unknown |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 64601 | 64745 | 214 | 4.69 | 4.71 | 0.0753 | 0.0195 |
| 2^{35} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 67307 | 67451 | 214 | 4.74 | 4.77 | 0.463 | 0.0313 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 68506 | 68650 | 214 | 4.83 | 4.90 | 0.107 | 0.0313 |
| 2^{35} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 71881 | 72025 | 214 | 4.84 | 4.93 | 0.464 | 0.0313 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 69116 | 69260 | 214 | 5.00 | 5.05 | 0.201 | 0.0625 |
| 2^{35} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 128 | 16 | 72024 | 72168 | 214 | 5.06 | 5.15 | 0.836 | 0.0625 |
| 2^{35} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | OOM | OOM | OOM | OOM | OOM | OOM | OOM | OOM | OOM |
| 2^{35} | [Plonky3 FRI(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 32 | 2562 | 570206 | 572800 | 0 | 12.2 | 12.2 | 0.0625 | unknown |
| 2^{35} | [Plonky3 STIR(6)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 34 | 2562 | 233468 | 236064 | 0 | 12.2 | 12.2 | 0.0623 | unknown |
| 2^{35} | [WHIR (Plonky3)(8)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 32 | 0 | 690886 | 690918 | 0 | 17.1 | 17.1 | 0.0628 | unknown |
| 2^{35} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 32 | 16 | 666720 | 666768 | 0 | 22.7 | 22.7 | 1.03 | unknown |
| 2^{35} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 4136 | 16 | 570464 | 574616 | 0 | 21.6 | 21.7 | 0.0002 | 0.0000 |
| 2^{35} | [WHIR (WorldFnd)](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d) | 56 | 24 | 317408 | 317488 | 0 | 47.4 | 47.4 | 0.0002 | 0.0000 |
| 2^{35} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 16 | 1440568 | 1440616 | 0 | 48.5 | 48.5 | 0.0000 | unknown |

**(1)** pinned Akita fp32-dense-recursive catalog row for nv=22 has no setup-prefix edges
**(2)** pinned Akita fp64-dense-recursive catalog row for nv=21 has no setup-prefix edges
**(3)** pinned Akita fp128-dense-recursive catalog row for nv=20 has no setup-prefix edges
**(4)** pinned Akita fp64-dense-recursive catalog row for nv=23 has no setup-prefix edges
**(5)** pinned Akita fp128-dense-recursive catalog row for nv=22 has no setup-prefix edges
**(6)** KoalaBear two-adicity is 24, so a rate-$1/2$ univariate cannot be a single degree-$2^{n}$ polynomial when $\log_2 N>23$. The worker packs the $2^{n}$ coefficients into a trace matrix of height $2^{23}$ and width $2^{n-23}$. That is batched univariate FRI/STIR, not one tall polynomial.
**(7)** pinned Akita fp128-dense-recursive catalog row for nv=24 has no setup-prefix edges
**(8)** WHIR uses unique decoding at this size so its 128-bit round-by-round target remains feasible on KoalaBear. Capacity bound and Johnson bound need more than 30 bits of grinding, which the field cannot support. The larger proof is the unique-decoding query schedule.


### Measured commits

- Akita [`d1b224d8`](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b)
- Akita (offload) [`d1b224d8`](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b)
- Akita [`d1b224d8`](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b)
- Akita (offload) [`d1b224d8`](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b)
- Akita [`d1b224d8`](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b)
- Akita (offload) [`d1b224d8`](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b)
- Plonky2 FRI [`e1c2d354`](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa)
- Plonky3 FRI [`3da160d0`](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36)
- Plonky3 STIR [`3da160d0`](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36)
- WHIR (Plonky3) [`9d496524`](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730)
- Binius64 BaseFold [`6e75a2d1`](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176)
- Flock Ligerito [`43f0eee0`](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430)
- WHIR (WorldFnd) [`8804e80e`](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d)
- BaseFold (SP1) [`0f2a1e13`](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86)


### Reproduction template

Machine, ISA, compiler, executable, lockfile, command, and timestamp provenance
for this dataset are recorded with each observation. The infrastructure
toolchain pin is Rust **1.95** (`rust-toolchain.toml`).
Recorded runner command: `target/release/pcs-bench hash-eval run --scheme plonky3-fri,plonky3-stir,binius64,worldfnd,basefold --payload 27,29,31,33,35 --threads 1,8 --runs 10 --warmups 1 --seed-mode vary --out results/hash-x86_64`. The commands below are a template,
not reconstructed provenance.
Workers are built from checked-in lockfiles before sampling. Every timed
execution is a fresh process wrapped in `scripts/with-memlimit.sh` with
a 109~GiB virtual-address-space ceiling (`ulimit -v`, numerically
90% of host RAM). Raw records identify
warmup and measured processes separately; warmup rows are stored with
`warmup: true` and excluded from the aggregate. Workload seeds and the
`vary`/`fixed` seed mode are recorded per observation. Recorded worker flags
for this dataset: `-C target-cpu=native`. Isolated Cargo trees under `benchmarks/`
fetch the pinned git revisions (Plonky3, SP1, plonky2, Binius64, Flock,
WorldFnd/WHIR) so they do not unify with the lattice workspace. Cargo fetches
those revisions on first build.

Non-interactive shells may not put Cargo on `PATH`; `source ~/.cargo/env`
is required in that case. `CARGO_NET_GIT_FETCH_WITH_CLI=true` avoids libgit2 auth
failures when fetching the pinned git dependencies.
Published numbers live in `results/hash-x86_64/`.

```bash
# On an AVX-512 Linux x86_64 host
source "$HOME/.cargo/env"   # if cargo is not on PATH
cd /path/to/akita-benchmark

export CARGO_NET_GIT_FETCH_WITH_CLI=true
export RUSTFLAGS="-C target-cpu=native"

./scripts/fetch-vendors.sh --akita   # Akita pin + nv=22/24 + fp64/fp128 + offload catalogs

# Full 140-cell matrix (14 schemes × 5 payloads × {1,8} threads)
./scripts/hash-eval.sh run --out results/hash-x86_64

# Rebuild Markdown + LaTeX from the JSONL already in that directory
cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval compare \
  results/hash-x86_64 --out-dir results/hash-x86_64
```

**Sanity-check the harness before trusting a full run.** `hash-eval matrix`
prints the 140-cell plan. A single supported cell should verify and emit JSON
with `status: ok`. Each sample the runner launches is equivalent to the worker
commands below (still under the 90%-of-RAM cap).

```bash
export RUSTFLAGS="-C target-cpu=native"

cargo test -p pcs-bench-core -p pcs-bench-runner --locked
cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval matrix

# One measured sample of a supported cell (payload 2^31, log2 N = 26, 1 thread)
./scripts/hash-eval.sh run --scheme akita --payload 31 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme akita-fp64 --payload 31 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme akita-fp128 --payload 31 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme akita-offload --payload 31 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme akita-fp64-offload --payload 31 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme whir --payload 31 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme basefold --payload 31 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme plonky2-fri --payload 27 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme plonky3-fri --payload 27 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme flock --payload 27 --threads 1 --runs 1 --warmups 0
```
