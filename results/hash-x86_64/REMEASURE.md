# Hash-PCS profile refresh completed

The five changed profiles were remeasured on the x86 benchmark machine on
2026-09-16 using harness commit `37a528a43333cd54d9b8a9a6fa0382676c636d98`.
The detached run completed successfully from 00:56:26 to 06:40:11 UTC,
including worker builds. All 550 new records succeeded: 500 measured samples
and 50 warmups across 50 cells.

| Scheme | Previous profile | Refreshed profile | Why it changed |
| --- | --- | --- | --- |
| Plonky3 FRI | rate 1/2, 80 queries, 20 query-PoW bits (98.2-bit random-words estimate) | pinned upstream `new_benchmark`: rate 1/2, 100 queries, 16 query-PoW bits (113.744-bit estimate) | The old tuple was a legacy retune below the intended estimate. |
| Plonky3 STIR | rate 1/2, fold 4 first and fold 16 thereafter | pinned upstream PCS benchmark: rate 1/2 and fold 4 throughout | The later fold-16 schedule was an undisclosed custom optimization. |
| Binius64 BaseFold | custom 100-bit UDR query target | product-default 96-bit UDR query target | The custom query count made the baseline more expensive than its native profile. |
| WorldFnd WHIR | custom 133-bit RBR, rate 1/4, fold 8, SHA-256 | pinned CLI defaults: 128-bit RBR, rate 1/2, fold 4, BLAKE3 | The custom tuple departed from every performance-relevant CLI default. |
| SP1 BaseFold | custom 128-bit tuple, rate 1/2, 112 queries, height 20 | product-default 100-bit tuple, rate 1/4, 124 queries, height 21 | The published row did not measure the product configuration. |


Akita, Plonky2 FRI, Plonky3 WHIR, and Flock retain their original measurements.
The combined dataset contains 1,540 records: 550 refreshed and 990 retained.
All old records for the five changed profiles were replaced. The cohorts pass
the harness checks for matching environment, profile identities, seed schedules,
and consistent builds within each scheme. They were collected on different dates;
this is not a contemporaneous full-matrix run.

## Selective rerun command

Archive existing output first; the runner refuses to overwrite `records.jsonl`.

```bash
./scripts/hash-eval.sh run \
  --scheme plonky3-fri,plonky3-stir,binius64,worldfnd,basefold \
  --payload 27,29,31,33,35 --threads 1,8 \
  --runs 10 --warmups 1 --seed-mode vary \
  --out results/hash-x86_64
```

After replacing these schemes' records and retaining the other schemes unchanged,
regenerate and validate the combined reports:

```bash
cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval compare \
  results/hash-x86_64 --out-dir results/hash-x86_64
```

Per-observation provenance is preserved in `records.jsonl`; `provenance.txt`
indexes the cohorts. The complete remote artifacts, log, launch script, exit
status, and previous results are archived locally under
`results/hash-run-archive/20260916T005626Z/` (git-ignored).
