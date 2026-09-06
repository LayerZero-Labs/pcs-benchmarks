Vendored PCS implementations live here after `./scripts/fetch-vendors.sh`.

They are cloned at pinned commits and are not part of the Cargo workspace:

- `akita` — LayerZero Akita at the `main` pin, with planner-generated
  `fp32-dense` rows for `nv=22` and `nv=24` installed from
  `vendor/akita-catalogs/`
- `akita-pr466` — the same overlay on the PR #466 pin
- `labrador` — Greyhound Pack (`lattice-dogs/labrador`)
- `rokoko` — RoKoKo PCS chain (`lattice-arguments/rokoko`), with
  `scripts/patch-rokoko-resources.py` printing commitment, CRS, and peak RSS

Hash-eval WHIR and BaseFold are **not** cloned here. Isolated crates under
`benchmarks/whir` and `benchmarks/basefold` fetch the pinned Plonky3 and SP1
git revisions through Cargo.

Do not commit the clones. See `docs/lattice-eval.md` and `docs/hash-eval.md`.
