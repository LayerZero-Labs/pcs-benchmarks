Vendored PCS implementations live here after `./scripts/fetch-vendors.sh`.

They are cloned at pinned commits and are not part of the Cargo workspace:

- `akita` — unmodified LayerZero Akita at `c0cb822f`, including its upstream
  direct and dense setup-offload schedule artifacts for fp32/fp64/fp128.
  The worker embeds these artifacts; no local catalog overlays are installed.
- `greyhound-reference` — Greyhound Pack (`LayerZero-Labs/greyhound-reference`)
- `rokoko` — RoKoKo PCS chain (`lattice-arguments/rokoko`), with
  `scripts/patch-rokoko-resources.py` printing commitment, CRS, and peak RSS

Hash-eval adapters are **not** cloned here. Isolated crates under
`benchmarks/` (`whir`, `basefold`, `plonky2-fri`, `plonky3-uni`, `binius64`,
`flock-ligerito`, `whir-provekit`) fetch the pinned git revisions through Cargo.

Do not commit the clones. See `docs/lattice-eval.md` and `docs/hash-eval.md`.
