Measurements were collected on a single AMD Ryzen 9 9950X 16-Core Processor (Linux x86_64, 32 logical CPUs, 121~GiB RAM). AVX-512F is advertised by the CPU and was activated for this run (`-C target-cpu=native`). Compiler flags: `-C target-cpu=native`.

Our second experiment compares Akita with other high-performance hash-based PCSs
on the same dense standalone payload ladder ($2^{27}$ through $2^{35}$ bits).
Each scheme uses its **native** security target, hash, field, and rate rather than a
common 128-bit retune, so cells are **not** $\lambda$-comparable.
Akita, Plonky3 WHIR, and SP1 BaseFold stay at the 128-bit transcript-error target:
WHIR is Plonky3 `p3-whir` at `security_level=128`. Capacity bound at rate $1/2$ is used
when that instance fits a 30-bit KoalaBear grind ($\log_2 N \le 26$);
unique decoding at rate $1/2$ is used at $\log_2 N=28$ and $30$, where list-decoding
bounds on KoalaBear cannot close 128 bits within that grind limit. BaseFold (SP1) is
SLOP stacked BaseFold with FRI parameters `log_blowup=1`, 112 queries, and
16 bits of grinding (conjectured soundness $1\cdot 112+16=128$).
Plonky2 FRI, Plonky3 FRI/STIR, Binius64 BaseFold, and Flock Ligerito Fast use native
**100-bit** targets. ProveKit WHIR uses Johnson-bound **133-bit** Goldilocks degree-3
challenges with base-field coefficients. KoalaBear univariate FRI/STIR pack into a
$2^{23}\times 2^{n-23}$ matrix when $\log_2 N>23$ (two-adicity 24 at rate $1/2$).
Timing cells report median ± sample standard deviation across fresh processes
after warmup, at **1 and 8 threads**. Scheme names link to the exact git commit
that was measured. Unmeasured roster cells are `pending`.

The timing comparison separates commitment, opening, and verification, while the
resources table reports communication, memory (1-thread and 8-thread peak RSS),
and preprocessing. An OOM entry exceeds the 109 GiB worker memory limit (90% of host RAM).

| Payload | Scheme | Field | log₂ N | Threads | Commit (s) | Open (s) | Total (s) | Verify (ms) |
| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | $2^{32}-99$ | 22 | 1 | 0.099 ± 0.0026 | 0.990 ± 0.016 | 1.09 ± 0.018 | 7.5 ± 0.05 |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | $2^{32}-99$ | 22 | 8 | 0.021 ± 0.0005 | 0.259 ± 0.0079 | 0.280 ± 0.0078 | 5.8 ± 0.04 |
| 2^{27} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | $2^{64}-2^{32}+1$ | 21 | 1 | 14.3 ± 0.060 | 5.20 ± 0.013 | 19.5 ± 0.072 | 1.8 ± 0.01 |
| 2^{27} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | $2^{64}-2^{32}+1$ | 21 | 8 | 2.26 ± 0.0091 | 1.75 ± 0.012 | 4.02 ± 0.019 | 1.8 ± 0.02 |
| 2^{27} | [Plonky3 FRI](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 22 | 1 | 1.10 ± 0.0057 | 2.59 ± 0.010 | 3.69 ± 0.012 | 7.3 ± 0.06 |
| 2^{27} | [Plonky3 FRI](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 22 | 8 | 0.221 ± 0.0015 | 0.537 ± 0.0042 | 0.758 ± 0.0057 | 7.3 ± 0.12 |
| 2^{27} | [Plonky3 STIR](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 22 | 1 | 1.10 ± 0.0037 | 2.60 ± 0.0038 | 3.70 ± 0.0074 | 3.3 ± 0.06 |
| 2^{27} | [Plonky3 STIR](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 22 | 8 | 0.222 ± 0.0056 | 0.621 ± 0.0042 | 0.842 ± 0.0020 | 3.3 ± 0.03 |
| 2^{27} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 22 | 1 | 0.088 ± 0.0006 | 0.524 ± 0.0007 | 0.612 ± 0.0003 | 1.5 ± 0.02 |
| 2^{27} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 22 | 8 | 0.024 ± 0.0020 | 0.110 ± 0.0052 | 0.135 ± 0.0038 | 1.5 ± 0.01 |
| 2^{27} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | $F_{2^{128}}$ | 20 | 1 | 0.065 ± 0.0007 | 0.019 ± 0.0003 | 0.084 ± 0.0006 | 0.5 ± 0.00 |
| 2^{27} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | $F_{2^{128}}$ | 20 | 8 | 0.041 ± 0.0004 | 0.006 ± 0.0002 | 0.047 ± 0.0003 | 0.5 ± 0.12 |
| 2^{27} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | $F_2$ | 27 | 1 | 0.033 ± 0.0012 | 0.091 ± 0.0025 | 0.124 ± 0.0036 | 1.3 ± 0.13 |
| 2^{27} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | $F_2$ | 27 | 8 | 0.006 ± 0.0002 | 0.022 ± 0.0011 | 0.029 ± 0.0013 | 1.3 ± 0.02 |
| 2^{27} | [WHIR (ProveKit)](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9) | $2^{64}-2^{32}+1$ | 21 | 1 | 0.526 ± 0.0014 | 0.951 ± 0.0032 | 1.48 ± 0.0018 | 2.3 ± 0.09 |
| 2^{27} | [WHIR (ProveKit)](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9) | $2^{64}-2^{32}+1$ | 21 | 8 | 0.095 ± 0.0007 | 0.190 ± 0.0062 | 0.286 ± 0.0064 | 2.0 ± 0.07 |
| 2^{27} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 22 | 1 | 0.571 ± 0.0085 | 0.844 ± 0.020 | 1.41 ± 0.012 | 19.7 ± 0.24 |
| 2^{27} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 22 | 8 | 0.563 ± 0.0044 | 0.812 ± 0.012 | 1.38 ± 0.0076 | 19.8 ± 0.32 |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | $2^{32}-99$ | 24 | 1 | 0.345 ± 0.0007 | 1.52 ± 0.018 | 1.86 ± 0.018 | 8.8 ± 0.30 |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | $2^{32}-99$ | 24 | 8 | 0.055 ± 0.0006 | 0.349 ± 0.0001 | 0.404 ± 0.0005 | 6.8 ± 0.32 |
| 2^{29} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | $2^{64}-2^{32}+1$ | 23 | 1 | 57.2 ± 0.021 | 21.0 ± 0.0091 | 78.3 ± 0.016 | 2.2 ± 0.01 |
| 2^{29} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | $2^{64}-2^{32}+1$ | 23 | 8 | 9.17 ± 0.038 | 7.25 ± 0.0078 | 16.4 ± 0.036 | 2.2 ± 0.00 |
| 2^{29} | [Plonky3 FRI(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 24 | 1 | 2.36 ± 0.012 | 5.21 ± 0.023 | 7.57 ± 0.036 | 8.2 ± 0.16 |
| 2^{29} | [Plonky3 FRI(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 24 | 8 | 0.491 ± 0.0006 | 1.07 ± 0.011 | 1.56 ± 0.011 | 8.2 ± 0.12 |
| 2^{29} | [Plonky3 STIR(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 24 | 1 | 2.36 ± 0.0076 | 5.26 ± 0.0052 | 7.63 ± 0.0079 | 3.3 ± 0.04 |
| 2^{29} | [Plonky3 STIR(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 24 | 8 | 0.490 ± 0.0015 | 1.28 ± 0.0075 | 1.77 ± 0.0090 | 3.4 ± 0.01 |
| 2^{29} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 24 | 1 | 0.377 ± 0.0038 | 5.59 ± 0.038 | 5.97 ± 0.042 | 1.6 ± 0.01 |
| 2^{29} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 24 | 8 | 0.112 ± 0.0020 | 1.54 ± 0.687 | 1.65 ± 0.687 | 1.7 ± 0.01 |
| 2^{29} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | $F_{2^{128}}$ | 22 | 1 | 0.276 ± 0.0037 | 0.073 ± 0.0013 | 0.351 ± 0.0039 | 0.7 ± 0.01 |
| 2^{29} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | $F_{2^{128}}$ | 22 | 8 | 0.184 ± 0.0008 | 0.025 ± 0.0011 | 0.207 ± 0.0007 | 0.7 ± 0.01 |
| 2^{29} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | $F_2$ | 29 | 1 | 0.117 ± 0.0007 | 0.377 ± 0.0035 | 0.494 ± 0.0041 | 1.4 ± 0.00 |
| 2^{29} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | $F_2$ | 29 | 8 | 0.031 ± 0.0016 | 0.080 ± 0.0043 | 0.111 ± 0.0059 | 1.4 ± 0.01 |
| 2^{29} | [WHIR (ProveKit)](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9) | $2^{64}-2^{32}+1$ | 23 | 1 | 2.40 ± 0.0018 | 4.22 ± 0.013 | 6.63 ± 0.014 | 2.6 ± 0.28 |
| 2^{29} | [WHIR (ProveKit)](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9) | $2^{64}-2^{32}+1$ | 23 | 8 | 0.411 ± 0.0032 | 0.780 ± 0.0062 | 1.19 ± 0.0068 | 2.2 ± 0.29 |
| 2^{29} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 24 | 1 | 0.837 ± 0.020 | 0.918 ± 0.022 | 1.75 ± 0.0046 | 19.7 ± 0.34 |
| 2^{29} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 24 | 8 | 0.864 ± 0.0070 | 0.839 ± 0.011 | 1.70 ± 0.018 | 20.6 ± 0.67 |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | $2^{32}-99$ | 26 | 1 | 1.30 ± 0.0080 | 2.63 ± 0.0015 | 3.93 ± 0.0091 | 12.8 ± 0.01 |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | $2^{32}-99$ | 26 | 8 | 0.188 ± 0.0018 | 0.532 ± 0.0042 | 0.720 ± 0.0059 | 7.5 ± 0.39 |
| 2^{31} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | $2^{64}-2^{32}+1$ | 25 | 1 | 235.7 ± 0.309 | 84.5 ± 0.134 | 320.3 ± 0.177 | 2.4 ± 0.00 |
| 2^{31} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | $2^{64}-2^{32}+1$ | 25 | 8 | 38.7 ± 0.075 | 29.5 ± 0.014 | 68.2 ± 0.070 | 2.4 ± 0.01 |
| 2^{31} | [Plonky3 FRI(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 26 | 1 | 2.69 ± 0.0062 | 5.15 ± 0.011 | 7.84 ± 0.015 | 7.8 ± 0.11 |
| 2^{31} | [Plonky3 FRI(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 26 | 8 | 0.681 ± 0.0018 | 1.06 ± 0.0035 | 1.73 ± 0.0028 | 8.2 ± 0.05 |
| 2^{31} | [Plonky3 STIR(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 26 | 1 | 2.70 ± 0.0057 | 5.18 ± 0.0087 | 7.88 ± 0.0052 | 3.3 ± 0.02 |
| 2^{31} | [Plonky3 STIR(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 26 | 8 | 0.670 ± 0.0095 | 1.27 ± 0.0068 | 1.94 ± 0.0029 | 3.4 ± 0.03 |
| 2^{31} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 26 | 1 | 1.52 ± 0.0042 | 44.7 ± 0.330 | 46.2 ± 0.328 | 1.8 ± 0.03 |
| 2^{31} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 26 | 8 | 0.474 ± 0.0054 | 6.26 ± 2.10 | 6.74 ± 2.09 | 1.9 ± 0.01 |
| 2^{31} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | $F_{2^{128}}$ | 24 | 1 | 1.17 ± 0.0021 | 0.302 ± 0.0041 | 1.47 ± 0.0032 | 0.8 ± 0.01 |
| 2^{31} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | $F_{2^{128}}$ | 24 | 8 | 0.810 ± 0.0017 | 0.096 ± 0.0036 | 0.906 ± 0.0051 | 0.8 ± 0.00 |
| 2^{31} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | $F_2$ | 31 | 1 | 0.571 ± 0.0017 | 1.23 ± 0.010 | 1.80 ± 0.012 | 1.1 ± 0.00 |
| 2^{31} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | $F_2$ | 31 | 8 | 0.127 ± 0.0002 | 0.246 ± 0.0010 | 0.374 ± 0.0010 | 1.1 ± 0.01 |
| 2^{31} | [WHIR (ProveKit)](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9) | $2^{64}-2^{32}+1$ | 25 | 1 | 11.6 ± 0.0056 | 20.6 ± 0.045 | 32.2 ± 0.050 | 2.8 ± 0.04 |
| 2^{31} | [WHIR (ProveKit)](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9) | $2^{64}-2^{32}+1$ | 25 | 8 | 1.90 ± 0.0071 | 3.87 ± 0.0085 | 5.77 ± 0.015 | 2.6 ± 0.48 |
| 2^{31} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 26 | 1 | 2.53 ± 0.044 | 1.02 ± 0.020 | 3.55 ± 0.025 | 20.5 ± 0.12 |
| 2^{31} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 26 | 8 | 2.51 ± 0.034 | 0.982 ± 0.018 | 3.48 ± 0.022 | 20.6 ± 0.23 |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | $2^{32}-99$ | 28 | 1 | 6.10 ± 0.0059 | 6.61 ± 0.0055 | 12.7 ± 0.0033 | 22.2 ± 0.45 |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | $2^{32}-99$ | 28 | 8 | 0.804 ± 0.0015 | 1.19 ± 0.0079 | 1.99 ± 0.0079 | 9.3 ± 0.16 |
| 2^{33} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | $2^{64}-2^{32}+1$ | 27 | 1 | OOM | OOM | OOM | OOM |
| 2^{33} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | $2^{64}-2^{32}+1$ | 27 | 8 | OOM | OOM | OOM | OOM |
| 2^{33} | [Plonky3 FRI(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 28 | 1 | 5.38 ± 0.0041 | 5.27 ± 0.014 | 10.6 ± 0.010 | 7.9 ± 0.02 |
| 2^{33} | [Plonky3 FRI(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 28 | 8 | 1.57 ± 0.0084 | 1.06 ± 0.011 | 2.65 ± 0.013 | 8.2 ± 0.08 |
| 2^{33} | [Plonky3 STIR(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 28 | 1 | 5.37 ± 0.014 | 5.28 ± 0.014 | 10.7 ± 0.028 | 3.4 ± 0.03 |
| 2^{33} | [Plonky3 STIR(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 28 | 8 | 1.59 ± 0.0040 | 1.30 ± 0.013 | 2.88 ± 0.012 | 3.5 ± 0.03 |
| 2^{33} | [WHIR (Plonky3)(2)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 28 | 1 | 5.09 ± 0.011 | 28.3 ± 0.018 | 33.4 ± 0.022 | 8.7 ± 0.10 |
| 2^{33} | [WHIR (Plonky3)(2)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 28 | 8 | 1.67 ± 0.012 | 8.15 ± 0.044 | 9.82 ± 0.036 | 8.8 ± 0.06 |
| 2^{33} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | $F_{2^{128}}$ | 26 | 1 | 4.96 ± 0.010 | 1.22 ± 0.0023 | 6.18 ± 0.0093 | 1.0 ± 0.00 |
| 2^{33} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | $F_{2^{128}}$ | 26 | 8 | 3.52 ± 0.0066 | 0.356 ± 0.0037 | 3.88 ± 0.0030 | 1.0 ± 0.00 |
| 2^{33} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | $F_2$ | 33 | 1 | 1.82 ± 0.0040 | 4.92 ± 0.0059 | 6.74 ± 0.0094 | 1.7 ± 0.00 |
| 2^{33} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | $F_2$ | 33 | 8 | 0.588 ± 0.0050 | 0.997 ± 0.0084 | 1.58 ± 0.0074 | 1.8 ± 0.00 |
| 2^{33} | [WHIR (ProveKit)](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9) | $2^{64}-2^{32}+1$ | 27 | 1 | 53.2 ± 0.0082 | 96.0 ± 0.307 | 149.2 ± 0.313 | 2.9 ± 0.03 |
| 2^{33} | [WHIR (ProveKit)](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9) | $2^{64}-2^{32}+1$ | 27 | 8 | 8.85 ± 0.0015 | 17.1 ± 0.031 | 25.9 ± 0.031 | 2.6 ± 0.40 |
| 2^{33} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 28 | 1 | 9.22 ± 0.168 | 1.49 ± 0.046 | 10.7 ± 0.124 | 22.6 ± 0.21 |
| 2^{33} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 28 | 8 | 9.16 ± 0.114 | 1.37 ± 0.017 | 10.5 ± 0.102 | 22.7 ± 0.27 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | $2^{32}-99$ | 30 | 1 | 24.5 ± 0.188 | 16.1 ± 0.021 | 40.6 ± 0.171 | 33.2 ± 0.06 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | $2^{32}-99$ | 30 | 8 | 3.47 ± 0.015 | 2.70 ± 0.014 | 6.17 ± 0.025 | 11.3 ± 0.19 |
| 2^{35} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | $2^{64}-2^{32}+1$ | 29 | 1 | OOM | OOM | OOM | OOM |
| 2^{35} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | $2^{64}-2^{32}+1$ | 29 | 8 | OOM | OOM | OOM | OOM |
| 2^{35} | [Plonky3 FRI(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 30 | 1 | 18.6 ± 0.020 | 6.68 ± 0.028 | 25.3 ± 0.013 | 8.5 ± 0.13 |
| 2^{35} | [Plonky3 FRI(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 30 | 8 | 6.00 ± 0.0014 | 1.29 ± 0.0046 | 7.29 ± 0.0054 | 8.6 ± 0.09 |
| 2^{35} | [Plonky3 STIR(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 30 | 1 | 18.5 ± 0.0079 | 6.62 ± 0.012 | 25.1 ± 0.019 | 3.8 ± 0.01 |
| 2^{35} | [Plonky3 STIR(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | $2^{31}-2^{24}+1$ | 30 | 8 | 6.01 ± 0.0044 | 1.52 ± 0.0046 | 7.53 ± 0.0086 | 3.9 ± 0.02 |
| 2^{35} | [WHIR (Plonky3)(2)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 30 | 1 | 17.1 ± 0.015 | 30.8 ± 0.122 | 47.9 ± 0.134 | 9.9 ± 0.03 |
| 2^{35} | [WHIR (Plonky3)(2)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | $2^{31}-2^{24}+1$ | 30 | 8 | 5.67 ± 0.0063 | 8.89 ± 0.021 | 14.6 ± 0.016 | 10.2 ± 0.05 |
| 2^{35} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | $F_{2^{128}}$ | 28 | 1 | 21.1 ± 0.010 | 5.04 ± 0.019 | 26.1 ± 0.011 | 1.1 ± 0.02 |
| 2^{35} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | $F_{2^{128}}$ | 28 | 8 | 15.3 ± 0.033 | 1.43 ± 0.0052 | 16.7 ± 0.033 | 1.1 ± 0.02 |
| 2^{35} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | $F_2$ | 35 | 1 | 9.58 ± 0.016 | 20.3 ± 0.019 | 29.9 ± 0.018 | 1.7 ± 0.01 |
| 2^{35} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | $F_2$ | 35 | 8 | 2.40 ± 0.019 | 4.20 ± 0.0055 | 6.60 ± 0.015 | 1.6 ± 0.02 |
| 2^{35} | [WHIR (ProveKit)](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9) | $2^{64}-2^{32}+1$ | 29 | 1 | 238.6 ± 0.171 | 477.2 ± 0.582 | 716.0 ± 0.486 | 3.1 ± 0.05 |
| 2^{35} | [WHIR (ProveKit)](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9) | $2^{64}-2^{32}+1$ | 29 | 8 | 39.7 ± 0.182 | 78.6 ± 0.051 | 118.4 ± 0.152 | 3.1 ± 0.31 |
| 2^{35} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 30 | 1 | 34.8 ± 0.371 | 3.56 ± 0.0087 | 38.4 ± 0.366 | 30.3 ± 0.02 |
| 2^{35} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | $2^{31}-2^{24}+1$ | 30 | 8 | 35.4 ± 0.357 | 3.02 ± 0.013 | 38.4 ± 0.347 | 30.6 ± 0.10 |

**(1)** KoalaBear two-adicity is 24, so a rate-$1/2$ univariate cannot be a single degree-$2^{n}$ polynomial when $\log_2 N>23$. The worker packs the $2^{n}$ coefficients into a trace matrix of height $2^{23}$ and width $2^{n-23}$. That is batched univariate FRI/STIR, not one tall polynomial.
**(2)** WHIR uses unique decoding at this size so the 128-bit transcript-error target still holds on KoalaBear. Capacity bound and Johnson bound need more than 30 bits of grinding, which the field cannot support. The larger proof is the unique-decoding query schedule.


| Payload | Scheme | Commitment (B) | Proof (B) | Total (B) | Peak RSS 1-thread (GiB) | Peak RSS 8-thread (GiB) | Prep. (s) | State (GiB) |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 343 | 61299 | 61642 | 0.187 | 0.193 | 0.0077 | 0.0020 |
| 2^{27} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | 520 | 87968 | 88488 | 2.79 | 2.79 | 0.0000 | 0.0000 |
| 2^{27} | [Plonky3 FRI](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 33 | 384190 | 384223 | 1.75 | 1.75 | 0.0269 | 0.0000 |
| 2^{27} | [Plonky3 STIR](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 34 | 154860 | 154894 | 1.36 | 1.36 | 0.0269 | 0.0000 |
| 2^{27} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 33 | 91274 | 91307 | 0.179 | 0.179 | 0.0013 | 0.0000 |
| 2^{27} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 32 | 319008 | 319040 | 0.0759 | 0.0757 | 0.0000 | 0.0000 |
| 2^{27} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 4137 | 408856 | 412993 | 0.240 | 0.240 | 0.0000 | 0.0000 |
| 2^{27} | [WHIR (ProveKit)](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9) | 56 | 446504 | 446560 | 0.288 | 0.287 | 0.0000 | 0.0000 |
| 2^{27} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 905160 | 905192 | 0.362 | 0.362 | 0.0000 | 0.0000 |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 343 | 61763 | 62106 | 0.633 | 0.632 | 0.0079 | 0.0020 |
| 2^{29} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | 520 | 106472 | 106992 | 11.2 | 11.2 | 0.0000 | 0.0000 |
| 2^{29} | [Plonky3 FRI(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 33 | 423479 | 423512 | 3.57 | 3.57 | 0.0582 | 0.0000 |
| 2^{29} | [Plonky3 STIR(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 34 | 161788 | 161822 | 2.78 | 2.78 | 0.0606 | 0.0000 |
| 2^{29} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 33 | 102655 | 102688 | 0.703 | 0.702 | 0.0057 | 0.0000 |
| 2^{29} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 32 | 403904 | 403936 | 0.294 | 0.294 | 0.0000 | 0.0000 |
| 2^{29} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 4137 | 340832 | 344969 | 0.957 | 0.956 | 0.0000 | 0.0000 |
| 2^{29} | [WHIR (ProveKit)](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9) | 56 | 459464 | 459520 | 1.14 | 1.14 | 0.0000 | 0.0000 |
| 2^{29} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 910728 | 910760 | 0.502 | 0.502 | 0.0000 | 0.0000 |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 343 | 63081 | 63424 | 1.93 | 1.94 | 0.0200 | 0.0049 |
| 2^{31} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | 520 | 117608 | 118128 | 44.6 | 44.6 | 0.0000 | 0.0000 |
| 2^{31} | [Plonky3 FRI(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 33 | 418487 | 418520 | 3.94 | 3.94 | 0.0599 | 0.0000 |
| 2^{31} | [Plonky3 STIR(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 34 | 164164 | 164198 | 3.16 | 3.16 | 0.0609 | 0.0000 |
| 2^{31} | [WHIR (Plonky3)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 33 | 113068 | 113101 | 2.80 | 2.80 | 0.0275 | 0.0000 |
| 2^{31} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 32 | 489248 | 489280 | 1.17 | 1.17 | 0.0000 | 0.0000 |
| 2^{31} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 4137 | 494632 | 498769 | 3.79 | 3.78 | 0.0000 | 0.0000 |
| 2^{31} | [WHIR (ProveKit)](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9) | 56 | 564160 | 564216 | 4.56 | 4.57 | 0.0000 | 0.0000 |
| 2^{31} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 933000 | 933032 | 1.75 | 1.75 | 0.0000 | 0.0000 |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 343 | 64487 | 64830 | 6.36 | 6.37 | 0.0373 | 0.0098 |
| 2^{33} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | OOM | OOM | OOM | OOM | OOM | OOM | OOM |
| 2^{33} | [Plonky3 FRI(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 33 | 433943 | 433976 | 5.44 | 5.44 | 0.0621 | 0.0000 |
| 2^{33} | [Plonky3 STIR(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 34 | 173251 | 173285 | 4.66 | 4.66 | 0.0611 | 0.0000 |
| 2^{33} | [WHIR (Plonky3)(2)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 33 | 589081 | 589114 | 9.60 | 9.60 | 0.0599 | 0.0000 |
| 2^{33} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 32 | 589568 | 589600 | 4.67 | 4.67 | 0.0000 | 0.0000 |
| 2^{33} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 4137 | 524840 | 528977 | 15.1 | 15.1 | 0.0000 | 0.0000 |
| 2^{33} | [WHIR (ProveKit)](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9) | 56 | 573456 | 573512 | 18.2 | 18.3 | 0.0000 | 0.0000 |
| 2^{33} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 1022088 | 1022120 | 7.00 | 7.00 | 0.0000 | 0.0000 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 343 | 64605 | 64948 | 24.7 | 24.7 | 0.0747 | 0.0195 |
| 2^{35} | [Plonky2 FRI](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa) | OOM | OOM | OOM | OOM | OOM | OOM | OOM |
| 2^{35} | [Plonky3 FRI(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 33 | 471815 | 471848 | 12.2 | 12.2 | 0.0618 | 0.0000 |
| 2^{35} | [Plonky3 STIR(1)](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36) | 34 | 207227 | 207261 | 12.2 | 12.2 | 0.0592 | 0.0000 |
| 2^{35} | [WHIR (Plonky3)(2)](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730) | 33 | 692790 | 692823 | 17.1 | 17.1 | 0.0581 | 0.0000 |
| 2^{35} | [Binius64 BaseFold](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176) | 32 | 690336 | 690368 | 18.7 | 18.7 | 0.0000 | 0.0000 |
| 2^{35} | [Flock Ligerito](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430) | 4137 | 570464 | 574601 | 60.5 | 60.5 | 0.0000 | 0.0000 |
| 2^{35} | [WHIR (ProveKit)](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9) | 56 | 584624 | 584680 | 72.9 | 73.1 | 0.0000 | 0.0000 |
| 2^{35} | [BaseFold (SP1)](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86) | 32 | 1378440 | 1378472 | 28.0 | 28.0 | 0.0000 | 0.0000 |

**(1)** KoalaBear two-adicity is 24, so a rate-$1/2$ univariate cannot be a single degree-$2^{n}$ polynomial when $\log_2 N>23$. The worker packs the $2^{n}$ coefficients into a trace matrix of height $2^{23}$ and width $2^{n-23}$. That is batched univariate FRI/STIR, not one tall polynomial.
**(2)** WHIR uses unique decoding at this size so the 128-bit transcript-error target still holds on KoalaBear. Capacity bound and Johnson bound need more than 30 bits of grinding, which the field cannot support. The larger proof is the unique-decoding query schedule.


### Measured commits

- Akita [`d1b224d8`](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b)
- Plonky2 FRI [`e1c2d354`](https://github.com/elliottech/plonky2/commit/e1c2d35450948b88fca6a7e69e2643c3ecad3caa)
- Plonky3 FRI [`3da160d0`](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36)
- Plonky3 STIR [`3da160d0`](https://github.com/Plonky3/Plonky3/commit/3da160d09d1c6a878adaa5b339939fcdccda5d36)
- WHIR (Plonky3) [`9d496524`](https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730)
- Binius64 BaseFold [`6e75a2d1`](https://github.com/binius-zk/binius64/commit/6e75a2d1d2e716578ae3ccb62806413fb1615176)
- Flock Ligerito [`43f0eee0`](https://github.com/succinctlabs/flock/commit/43f0eee06d887d87ad25d72614cbc2b17fe91430)
- WHIR (ProveKit) [`6481f961`](https://github.com/worldfnd/ProveKit/commit/6481f961fc78615811b9cbaa9aa2380f1f6703c9); whir [`8804e80e`](https://github.com/worldfnd/whir/commit/8804e80e8e890d01bb585f2bd5e5b564ac0fd80d)
- BaseFold (SP1) [`0f2a1e13`](https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86)


### Commands used for these numbers

These tables were produced on a Linux **x86_64** AVX-512 host (AMD Ryzen 9 9950X)
from this repository. The toolchain pin is Rust **1.95** (`rust-toolchain.toml`).
Every timed worker is a fresh process wrapped in `scripts/with-memlimit.sh` at
109~GiB (`ulimit -v`, 90% of host RAM). The runner defaults are
**1 warmup + 3 measured** samples per cell; warmup rows are stored with
`warmup: true` and excluded from the median. Workers inherit
`RUSTFLAGS=-C target-cpu=native`. Isolated Cargo trees under `benchmarks/`
fetch the pinned git revisions (Plonky3, SP1, plonky2, Binius64, Flock,
ProveKit/whir) so they do not unify with the lattice workspace. Cargo fetches
those revisions on first build.

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

# Full 90-cell matrix (9 schemes × 5 payloads × {1,8} threads)
./scripts/hash-eval.sh run --out results/hash-x86_64

# Rebuild Markdown + LaTeX from the JSONL already in that directory
cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval compare \
  results/hash-x86_64 --out-dir results/hash-x86_64
```

**Sanity-check the harness before trusting a full run.** `hash-eval matrix`
prints the 90-cell plan. A single supported cell should verify and emit JSON
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
./scripts/hash-eval.sh run --scheme plonky2-fri --payload 27 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme plonky3-fri --payload 27 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme flock --payload 27 --threads 1 --runs 1 --warmups 0
```

