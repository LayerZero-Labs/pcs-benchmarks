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

**(1)** WHIR uses unique decoding at this size so the 128-bit transcript-error target still holds on KoalaBear. Capacity bound and Johnson bound need more than 30 bits of grinding, which the field cannot support. The larger proof is the unique-decoding query schedule.
