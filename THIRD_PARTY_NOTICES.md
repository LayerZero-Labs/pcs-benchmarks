# Third-party material

PCS Benchmarks is licensed under the Apache License, Version 2.0. Third-party
software fetched by Cargo or by `scripts/fetch-vendors.sh` is not relicensed;
it remains subject to its respective copyright and license terms. The fetch
workflow retains the license and notice files in each upstream checkout.

The committed `vendor/akita-catalogs/fp32_dense-main.rs`,
`fp32_dense_recursive-main.rs`, `fp64_dense-main.rs`, and
`fp128_dense-main.rs` files were generated with Akita at commit
`d1b224d809c7edc357b0dbab0f607e19b475910b` and include additional benchmark
schedule rows. Akita is available under Apache-2.0 or MIT; this repository
uses the Apache-2.0 option.

The `scripts/patch-rokoko-resources.py` patcher contains matching context for
and modifies RoKoKo's `src/protocol/parties/executor.rs` at commit
`1baa91e901fc37b5fa59e65c26a630cb93849b3e`. RoKoKo is licensed under
Apache-2.0. The patcher marks the modified checkout accordingly.

Upstream license texts:

- Akita: <https://github.com/LayerZero-Labs/akita/tree/d1b224d809c7edc357b0dbab0f607e19b475910b>
- RoKoKo: <https://github.com/lattice-arguments/rokoko/blob/1baa91e901fc37b5fa59e65c26a630cb93849b3e/LICENSE>

Redistributions that bundle fetched source code or compiled dependencies must
also carry all license and NOTICE files required by those dependencies.
