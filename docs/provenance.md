# Historical result-only dirty revisions

Raw `records.jsonl` files preserve the revision and dirty fingerprint captured
at measurement time. Generated hash reports display the clean commit for only
the three exact fingerprints below.

| Recorded harness revision | Affected measurements |
| --- | --- |
| `37a528a43333cd54d9b8a9a6fa0382676c636d98+dirty.1d0750363ab2` | Plonky3 FRI/STIR and Binius64 (330 records) |
| `946c0a78c50ba9800a1c452ce0c27b73d0cce24a+dirty.e6c3243c86f9` | Corrected SP1 BaseFold (110 records) |
| `a5a6962520e7e174a0bd5cb93795c33ead2934cf+dirty.2fe6b7a25d43` | Corrected WorldFnd WHIR (110 records) |

The source-state audit reproduced these fingerprints from the corresponding
commits with tracked hash result files removed during run staging. The dirty
state affected result files, not benchmark source or parameters. Remeasurement
is therefore unnecessary for this provenance correction.

This finding is limited to those exact fingerprints. It does not certify
arbitrary dirty checkouts or independently establish every byte of ignored
vendor directories. Worker hashes, implementation pins, compiler flags, and
other per-observation provenance remain available in the raw records.

The report renderer uses an explicit allowlist, leaves unknown dirty revisions
visible, and never rewrites raw observations. Akita was remeasured separately
at upstream commit `c0cb822f28b7b9efe85b1924b029d36e13cdf516`; its new records
already carry a clean harness revision.
