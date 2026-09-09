#!/usr/bin/env python3
"""Fix RoKoKo timing boundaries and print resource measurements.

The upstream executor already prints phase timings and proof size. The lattice
resources table also needs the inner recursive commitment, expanded CRS
resident size, and process peak RSS. Its prover timer also includes an
independent full-witness claim check; this patch ends that timer before the
check while preserving the check. The patch is applied after `fetch-vendors.sh`
checks out the pinned revision.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

MODIFICATION_NOTICE = "// Modified for PCS Benchmarks to report resource measurements.\n"

IMPORT_NEEDLE = "common::{matrix::VerticallyAlignedMatrix, ring_arithmetic::RingElement},"
IMPORT_PATCH = (
    "common::{matrix::VerticallyAlignedMatrix, ring_arithmetic::{seed_rng, RingElement}},"
)

POINT_SEED_NEEDLE = """\
    let evaluation_points = sample_initial_evaluation_points(
"""

POINT_SEED_PATCH = """\
    // PCS benchmark workload seeds are recorded per observation.
    let workload_seed =
        std::env::var("PCS_BENCH_SEED").unwrap_or_else(|_| "legacy".to_owned());
    seed_rng(&format!("pcs-bench-point-{workload_seed}"));
    let evaluation_points = sample_initial_evaluation_points(
"""

WITNESS_SEED_NEEDLE = """\
    let witness = witness_sampler();
"""

WITNESS_SEED_PATCH = """\
    seed_rng(&format!("pcs-bench-witness-{workload_seed}"));
    let witness = witness_sampler();
"""

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

SETUP_TIMING_NEEDLE = """\
    let crs_duration = crs_start.elapsed().as_nanos();
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
    let mut sumcheck_context_verifier = init_verifier(&verifier_crs, &config);
"""

SETUP_TIMING_PATCH = """\
    // PCS benchmark setup includes reusable prover and verifier contexts.
    let mut sumcheck_context = init_sumcheck(&crs, &config);
    let mut sumcheck_context_verifier = init_verifier(&verifier_crs, &config);
    let crs_duration = crs_start.elapsed().as_nanos();
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
    if std::env::var("PCS_BENCH_NEGATIVE_CHECK").as_deref() == Ok("1") {
        let mut altered_bytes = bytes.clone();
        if let Some(last) = altered_bytes.last_mut() {
            *last ^= 1;
        }
        if let Ok(altered_proof) = wire::from_bytes(&altered_bytes) {
            let mut negative_context = init_verifier(&verifier_crs, &config);
            let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                verifier_round(
                    &verifier_crs,
                    &config,
                    &rc_commitment,
                    &altered_proof,
                    &evaluation_points.inner,
                    &evaluation_points.outer,
                    &claims,
                    &mut negative_context,
                    None,
                    None,
                );
            }))
            .is_err();
            assert!(rejected, "RoKoKo verifier accepted an altered proof");
        }
    }
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

NEGATIVE_NEEDLE = """\
    println!(
        "TOTAL Verifier time{}: {:?} ns",
        boundary_note, verifier_duration
    );
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
"""

NEGATIVE_PATCH = """\
    println!(
        "TOTAL Verifier time{}: {:?} ns",
        boundary_note, verifier_duration
    );
    if std::env::var("PCS_BENCH_NEGATIVE_CHECK").as_deref() == Ok("1") {
        let mut altered_bytes = bytes.clone();
        if let Some(last) = altered_bytes.last_mut() {
            *last ^= 1;
        }
        if let Ok(altered_proof) = wire::from_bytes(&altered_bytes) {
            let mut negative_context = init_verifier(&verifier_crs, &config);
            let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                verifier_round(
                    &verifier_crs,
                    &config,
                    &rc_commitment,
                    &altered_proof,
                    &evaluation_points.inner,
                    &evaluation_points.outer,
                    &claims,
                    &mut negative_context,
                    None,
                    None,
                );
            }))
            .is_err();
            assert!(rejected, "RoKoKo verifier accepted an altered proof");
        }
    }
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
"""

PROVER_NEEDLE = """\
    drop(prover_span);
    let claims = claims.expect("Prover round must return claims when with_claims is true.");
    {
        let _s = tracing::info_span!("verify_claims").entered();
        check_prover_claims_match_witness(&witness, &evaluation_points, &claims);
    }

    let prover_duration = start.elapsed().as_nanos();
"""

PROVER_PATCH = """\
    drop(prover_span);
    // PCS benchmark prover interval ends before independent claim validation.
    let prover_duration = start.elapsed().as_nanos();
    let claims = claims.expect("Prover round must return claims when with_claims is true.");
    {
        let _s = tracing::info_span!("verify_claims").entered();
        check_prover_claims_match_witness(&witness, &evaluation_points, &claims);
    }
    let evaluation_bits: usize = claims.iter().map(RingElement::compact_size_in_bits).sum();
    println!("TOTAL Evaluation size: {} bytes", (evaluation_bits + 7) / 8);
"""

EVALUATION_NEEDLE = """\
        check_prover_claims_match_witness(&witness, &evaluation_points, &claims);
    }
    println!("TOTAL Prover time{}: {:?} ns", boundary_note, prover_duration);
"""

EVALUATION_PATCH = """\
        check_prover_claims_match_witness(&witness, &evaluation_points, &claims);
    }
    let evaluation_bits: usize = claims.iter().map(RingElement::compact_size_in_bits).sum();
    println!("TOTAL Evaluation size: {} bytes", (evaluation_bits + 7) / 8);
    println!("TOTAL Prover time{}: {:?} ns", boundary_note, prover_duration);
"""


def patch(text: str) -> str:
    if not text.startswith(MODIFICATION_NOTICE):
        text = MODIFICATION_NOTICE + text
    if (
        "TOTAL Commitment size:" in text
        and "TOTAL CRS size:" in text
        and "Peak RSS:" in text
        and "PCS benchmark prover interval ends before independent claim validation" in text
        and "pcs-bench-witness-" in text
        and "RoKoKo verifier accepted an altered proof" in text
        and "PCS benchmark setup includes reusable prover and verifier contexts" in text
        and "TOTAL Evaluation size:" in text
    ):
        return text
    for needle, replacement, label, marker in (
        (IMPORT_NEEDLE, IMPORT_PATCH, "seed import", "ring_arithmetic::{seed_rng, RingElement}"),
        (POINT_SEED_NEEDLE, POINT_SEED_PATCH, "point seed", "pcs-bench-point-"),
        (WITNESS_SEED_NEEDLE, WITNESS_SEED_PATCH, "witness seed", "pcs-bench-witness-"),
        (COMMIT_NEEDLE, COMMIT_PATCH, "commitment", "TOTAL Commitment size:"),
        (CRS_NEEDLE, CRS_PATCH, "CRS", "TOTAL CRS size:"),
        (
            SETUP_TIMING_NEEDLE,
            SETUP_TIMING_PATCH,
            "setup timing",
            "PCS benchmark setup includes reusable prover and verifier contexts",
        ),
        (RSS_NEEDLE, RSS_PATCH, "peak RSS", "Peak RSS:"),
        (
            NEGATIVE_NEEDLE,
            NEGATIVE_PATCH,
            "negative proof check",
            "RoKoKo verifier accepted an altered proof",
        ),
        (
            PROVER_NEEDLE,
            PROVER_PATCH,
            "prover timing",
            "PCS benchmark prover interval ends before independent claim validation",
        ),
        (
            EVALUATION_NEEDLE,
            EVALUATION_PATCH,
            "evaluation size",
            "TOTAL Evaluation size:",
        ),
    ):
        if marker in text:
            continue
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
