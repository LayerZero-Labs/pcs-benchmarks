# Required hash-PCS reruns

The checked-in timings in this directory predate the benchmark-profile repair.
They remain an honest record of the configurations that were measured, but the
current harness deliberately rejects records for the five changed profiles.
Do not regenerate or publish the comparison from this directory until these
rows have been rerun on the publication host.

| Scheme | Recorded profile | Profile now accepted by the harness | Why it changed |
| --- | --- | --- | --- |
| Plonky3 FRI | rate 1/2, 80 queries, 20 query-PoW bits (98.2-bit random-words estimate) | pinned upstream `new_benchmark`: rate 1/2, 100 queries, 16 query-PoW bits (113.744-bit estimate) | The old tuple was a legacy retune below the intended estimate. |
| Plonky3 STIR | rate 1/2, fold 4 first and fold 16 thereafter | pinned upstream PCS benchmark: rate 1/2 and fold 4 throughout | The later fold-16 schedule was an undisclosed custom optimization. |
| Binius64 BaseFold | custom 100-bit UDR query target | product-default 96-bit UDR query target | The custom query count made the baseline more expensive than its native profile. |
| WorldFnd WHIR | custom 133-bit RBR, rate 1/4, fold 8, SHA-256 | pinned CLI defaults: 128-bit RBR, rate 1/2, fold 4, BLAKE3 | The custom tuple departed from every performance-relevant CLI default. |
| SP1 BaseFold | custom 128-bit tuple, rate 1/2, 112 queries, height 20 | product-default 100-bit tuple, rate 1/4, 124 queries, height 21 | The published row did not measure the product configuration. |

Plonky2 FRI, Plonky3 WHIR, Flock, and Akita retain their existing profiles.
The changed `native_param` identities are the enforcement mechanism: importing
the old JSONL against the repaired matrix must fail rather than relabel its
timings. Rerun the full matrix with:

```bash
./scripts/hash-eval.sh run --out results/hash-x86_64
```

Then regenerate the reports only after every changed profile has fresh records:

```bash
cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval compare \
  results/hash-x86_64 --out-dir results/hash-x86_64
```
