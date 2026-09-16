# Hash-PCS profile refresh completed

## WorldFnd memory correction completed

WorldFnd was remeasured using adapter commit `a5a6962` on 2026-09-16,
from 18:26:48 to 22:39:44 UTC (4 hours 12 minutes 56 seconds), with exit code 0.
All 110 samples succeeded: 100 measured samples and 10 warmups across five
payload sizes at 1 and 8 threads. Separate correctness smoke tests passed at
both thread counts, including rejection of an altered evaluation claim.

The input vector now moves into the prover buffer without retaining a duplicate.
At payload 2^35, median process peak RSS was 43.350 GiB at 1 thread and
43.376 GiB at 8 threads; the previous run recorded 47.351 and 47.375 GiB,
respectively. The separate runs show approximately 4 GiB less peak RSS at both
thread counts. They were not interleaved performance trials. The removed copy
was outside the timers, so no copying time is subtracted from the measured
phases. Security parameters and proof-size accounting remain unchanged.

All 110 prior WorldFnd records were replaced; the other 1,430 records were
retained verbatim. Combined reports passed the harness environment, seed,
profile, and build-identity checks. Original revision strings, including their
historical dirty suffixes, are preserved.

```bash
./scripts/hash-eval.sh run --scheme worldfnd \
  --payload 27,29,31,33,35 --threads 1,8 \
  --runs 10 --warmups 1 --seed-mode vary --out results/hash-x86_64
```

Raw remote artifacts and the previous local dataset are archived under
`results/hash-run-archive/20260916T182648Z-worldfnd/` (git-ignored).

## SP1 timing and memory correction completed

SP1 was remeasured using adapter commit `946c0a7` on 2026-09-16,
from 16:23:43 to 17:08:16 UTC (44 minutes 33 seconds), with exit code 0.
All 110 samples succeeded: 100 measured samples and 10 warmups across five
payload sizes at 1 and 8 threads. Separate correctness smoke tests passed at
both thread counts, including rejection of an altered evaluation claim.

The adapter no longer retains a witness clone or charges a redundant full-witness
evaluation to opening. It checks an independent, factored correctness oracle
before ownership transfer; opening includes interpolation of the proof's batch
evaluations. The security profile is unchanged. All 110 prior SP1 records were
replaced, and the other 1,430 records were retained verbatim. Combined reports
passed the harness environment, seed, profile, and build-identity checks.

The completed run used:

```bash
./scripts/hash-eval.sh run --scheme basefold \
  --payload 27,29,31,33,35 --threads 1,8 \
  --runs 10 --warmups 1 --seed-mode vary --out results/hash-x86_64
```

Raw remote artifacts and the previous local dataset are archived under
`results/hash-run-archive/20260916T162343Z-sp1/` (git-ignored).

## Earlier security-profile refresh

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
