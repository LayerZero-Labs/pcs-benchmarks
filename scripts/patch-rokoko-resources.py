#!/usr/bin/env python3
"""Print RoKoKo commitment, CRS, and peak RSS on the pinned executor.

The upstream executor already prints phase timings and proof size. The lattice
resources table also needs the inner recursive commitment, expanded CRS
resident size, and process peak RSS. This patch is applied after
`fetch-vendors.sh` checks out the pinned revision.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

MODIFICATION_NOTICE = "// Modified for PCS Benchmarks to report resource measurements.\n"

COMMIT_NEEDLE = """\
    let commit_duration = start.elapsed().as_nanos();
    println!("TOTAL Commit time: {:?} ns", commit_duration);

    let boundary_note = if cut.is_some() { " (to boundary)" } else { "" };
"""

COMMIT_PATCH = """\
    let commit_duration = start.elapsed().as_nanos();
    println!("TOTAL Commit time: {:?} ns", commit_duration);
    {
        let mut bits = 0usize;
        for el in &rc_commitment {
            bits += el.compact_size_in_bits();
        }
        println!("TOTAL Commitment size: {} bytes", (bits + 7) / 8);
    }

    let boundary_note = if cut.is_some() { " (to boundary)" } else { "" };
"""

CRS_NEEDLE = """\
    println!("TOTAL CRS gen time: {:?} ns", crs_duration);

    let mut sumcheck_context = init_sumcheck(&crs, &config);
"""

CRS_PATCH = """\
    println!("TOTAL CRS gen time: {:?} ns", crs_duration);
    {
        let mut n = 0usize;
        for ck in &crs.cks {
            for row in ck {
                n += row.preprocessed_row.len();
            }
        }
        println!(
            "TOTAL CRS size: {} bytes",
            n * std::mem::size_of::<RingElement>()
        );
    }

    let mut sumcheck_context = init_sumcheck(&crs, &config);
"""

RSS_NEEDLE = """\
    println!(
        "TOTAL Verifier time{}: {:?} ns",
        boundary_note, verifier_duration
    );

    (
        proof_size_bits,
        prover_boundary,
        verifier_boundary,
        crs,
        verifier_crs,
    )
}
"""

RSS_PATCH = """\
    println!(
        "TOTAL Verifier time{}: {:?} ns",
        boundary_note, verifier_duration
    );
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            let Some(rest) = line.strip_prefix("VmHWM:") else {
                continue;
            };
            if let Some(kb) = rest
                .split_whitespace()
                .next()
                .and_then(|token| token.parse::<u64>().ok())
            {
                println!("Peak RSS: {} bytes", kb.saturating_mul(1024));
            }
            break;
        }
    }

    (
        proof_size_bits,
        prover_boundary,
        verifier_boundary,
        crs,
        verifier_crs,
    )
}
"""


def patch(text: str) -> str:
    if not text.startswith(MODIFICATION_NOTICE):
        text = MODIFICATION_NOTICE + text
    if "TOTAL Commitment size:" in text and "TOTAL CRS size:" in text and "Peak RSS:" in text:
        return text
    for needle, replacement, label in (
        (COMMIT_NEEDLE, COMMIT_PATCH, "commitment"),
        (CRS_NEEDLE, CRS_PATCH, "CRS"),
        (RSS_NEEDLE, RSS_PATCH, "peak RSS"),
    ):
        if needle not in text:
            raise SystemExit(f"error: RoKoKo executor.rs does not contain the {label} insertion point")
        text = text.replace(needle, replacement, 1)
    return text


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "rokoko_root",
        type=Path,
        help="vendored RoKoKo checkout (third_party/rokoko)",
    )
    args = parser.parse_args()
    executor = args.rokoko_root / "src/protocol/parties/executor.rs"
    if not executor.is_file():
        print(f"error: missing {executor}", file=sys.stderr)
        return 2
    original = executor.read_text()
    updated = patch(original)
    if updated == original:
        print(f"already patched {executor}")
        return 0
    executor.write_text(updated)
    print(f"patched {executor}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
