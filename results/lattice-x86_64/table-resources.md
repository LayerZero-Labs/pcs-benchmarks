| Payload | Scheme | Commitment (B) | Proof (B) | Total (B) | Peak RSS (GiB) | Prep. (s) | State (GiB) |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 2^{27} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 343 | 61299 | 61642 | 0.109 | 0.0081 | 0.0020 |
| 2^{27} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | err(1) | err(1) | err(1) | err(1) | err(1) | err(1) |
| 2^{27} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397) | 2048 | 59284 | 61332 | 0.329 | 0 | 0.0000 |
| 2^{27} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | —(2) | —(2) | —(2) | —(2) | —(2) | —(2) |
| 2^{29} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 343 | 61763 | 62106 | 0.239 | 0.0081 | 0.0020 |
| 2^{29} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 343 | 66154 | 66497 | 0.232 | 0.0291 | 0.0020 |
| 2^{29} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397) | 2304 | 59089 | 61393 | 1.08 | 0 | 0.0000 |
| 2^{29} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | —(2) | —(2) | —(2) | —(2) | —(2) | —(2) |
| 2^{31} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 343 | 63081 | 63424 | 0.680 | 0.0191 | 0.0049 |
| 2^{31} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 343 | 66297 | 66640 | 0.702 | 0.0758 | 0.0039 |
| 2^{31} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397) | 2304 | 64600 | 66904 | 4.66 | 0 | 0.0000 |
| 2^{31} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | 774 | 114830 | 115604 | 4.07 | 0.335 | 1.59 |
| 2^{33} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 343 | 64487 | 64830 | 1.36 | 0.0379 | 0.0098 |
| 2^{33} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 343 | 66894 | 67237 | 1.38 | 0.230 | 0.0156 |
| 2^{33} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397) | 2304 | 64525 | 66829 | 20.7 | 0 | 0.0000 |
| 2^{33} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | 773 | 114910 | 115683 | 10.9 | 0.698 | 3.19 |
| 2^{35} | [Akita](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 343 | 64605 | 64948 | 4.68 | 0.0745 | 0.0195 |
| 2^{35} | [Akita (offload)](https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b) | 343 | 67288 | 67631 | 4.73 | 0.463 | 0.0313 |
| 2^{35} | [Greyhound](https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397) | 2560 | 66006 | 68566 | 104.8 | 0 | 0.0000 |
| 2^{35} | [RoKoKo](https://github.com/lattice-arguments/rokoko/commit/1baa91e901fc37b5fa59e65c26a630cb93849b3e) | 777 | 115056 | 115833 | 35.6 | 1.65 | 7.44 |

**(1)** The recursive `fp32-dense` planner produced a schedule for this $n_v$ with no setup-prefix edge, so the offload variant would not offload setup.
**(2)** RoKoKo ships only native sets `p-26`, `p-28`, and `p-30`; no instance matches this payload.
