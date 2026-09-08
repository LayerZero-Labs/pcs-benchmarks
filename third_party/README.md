Vendored PCS implementations live here after `./scripts/fetch-vendors.sh`.

They are cloned at pinned commits and are not part of the Cargo workspace:

- `akita` — LayerZero Akita at the `main` pin, with planner-generated
  `fp32-dense` rows for `nv=22` and `nv=24`, recursive `fp32-dense`
  setup-offload, `fp64-dense` rows for `nv=21/23/25/27`, and `fp128-dense`
  rows for `nv=20/22` installed from `vendor/akita-catalogs/`
- `greyhound-reference` — Greyhound Pack (`LayerZero-Labs/greyhound-reference`)
- `rokoko` — RoKoKo PCS chain (`lattice-arguments/rokoko`), with
  `scripts/patch-rokoko-resources.py` printing commitment, CRS, and peak RSS

Hash-eval adapters are **not** cloned here. Isolated crates under
`benchmarks/` (`whir`, `basefold`, `plonky2-fri`, `plonky3-uni`, `binius64`,
`flock-ligerito`, `whir-provekit`) fetch the pinned git revisions through Cargo.

Do not commit the clones. See `docs/lattice-eval.md` and `docs/hash-eval.md`.
