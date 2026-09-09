//! Render the hash timing and resource comparison as Markdown or LaTeX.

use crate::hash::{hash_case, hash_matrix, plonky3_is_packed, HashSchemeId, HASH_SCHEME_COUNT};
use crate::lattice::PAYLOAD_LOG2;
use crate::observation::{looks_like_oom, HashRecord, RunStatus};
use crate::table::{
    apply_mark, footnote_index, format_gib, format_prep, gap_token, gap_token_pending,
    latex_footnotes, markdown_footnotes, median_ci95, median_f64, median_u64, phase_seconds_from,
    timing_millis_cell, timing_seconds_cell, unique_gap_notes, unknown_token, GapNote,
};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// One aggregated hash timing-table row (payload × scheme × threads).
#[derive(Clone, Debug, PartialEq)]
pub struct HashTimingTableRow {
    /// Target payload exponent.
    pub payload_log2: u32,
    /// Scheme.
    pub scheme: HashSchemeId,
    /// Recorded implementation revision used by this aggregate.
    pub implementation_revision: Option<String>,
    /// Field label.
    pub field: String,
    /// Native `log2 N`.
    pub log2_n: Option<u32>,
    /// Worker thread count.
    pub threads: u32,
    /// Cell outcome after aggregating samples.
    pub status: RunStatus,
    /// Median commit seconds, when `status` is ok.
    pub commit_s: Option<f64>,
    /// Distribution-free two-sided confidence interval for median commit seconds.
    pub commit_s_ci95: Option<(f64, f64)>,
    /// Median opening/prove seconds, when `status` is ok.
    pub open_s: Option<f64>,
    /// Distribution-free two-sided confidence interval for median opening seconds.
    pub open_s_ci95: Option<(f64, f64)>,
    /// Median cold setup+commit+open seconds, when `status` is ok.
    pub total_s: Option<f64>,
    /// Confidence interval for median cold setup+commit+open seconds.
    pub total_s_ci95: Option<(f64, f64)>,
    /// Median verify seconds, when `status` is ok.
    pub verify_s: Option<f64>,
    /// Distribution-free two-sided confidence interval for median verify seconds.
    pub verify_s_ci95: Option<(f64, f64)>,
    /// Number of non-warmup ok samples used for the median.
    pub n_ok: usize,
    /// Whether any sample was recorded for this cell.
    pub measured: bool,
    /// Footnote for unique decoding or an unsupported dash.
    pub gap_note: Option<GapNote>,
}

/// One aggregated hash resources-table row (payload × scheme, both thread counts).
#[derive(Clone, Debug, PartialEq)]
pub struct HashResourceTableRow {
    /// Target payload exponent.
    pub payload_log2: u32,
    /// Scheme.
    pub scheme: HashSchemeId,
    /// Recorded implementation revision used by this aggregate.
    pub implementation_revision: Option<String>,
    /// Cell outcome after aggregating 1-thread samples (communication / prep).
    pub status: RunStatus,
    /// Median commitment size in bytes.
    pub commitment_bytes: Option<u64>,
    /// Median opening-proof size in bytes.
    pub proof_bytes: Option<u64>,
    /// Median separately transmitted evaluation size in bytes.
    pub evaluation_bytes: Option<u64>,
    /// Median excluded public/verifier context size in bytes.
    pub public_context_bytes: Option<u64>,
    /// Median peak RSS at 1 thread.
    pub peak_rss_bytes_1: Option<u64>,
    /// Median peak RSS at 8 threads.
    pub peak_rss_bytes_8: Option<u64>,
    /// Median preprocessing seconds (1-thread samples).
    pub prep_s: Option<f64>,
    /// Median reusable preprocessing state in bytes.
    pub state_bytes: Option<u64>,
    /// Whether any 1-thread sample was recorded.
    pub measured: bool,
    /// Whether any 8-thread sample was recorded.
    pub measured_8: bool,
    /// 8-thread cell outcome, when measured.
    pub status_8: RunStatus,
    /// Footnote for unique decoding or an unsupported dash.
    pub gap_note: Option<GapNote>,
}

/// Aggregate measured hash records into timing rows.
#[must_use]
pub fn aggregate_hash_timing_rows(records: &[HashRecord]) -> Vec<HashTimingTableRow> {
    hash_matrix()
        .into_iter()
        .map(|case| {
            let samples = measured_samples(records, case.payload_log2, case.scheme, case.threads);
            hash_timing_row(
                case.payload_log2,
                case.scheme,
                case.field.name,
                Some(case.log2_n),
                case.threads,
                &samples,
            )
        })
        .collect()
}

/// Aggregate measured hash records into the communication / memory table.
#[must_use]
pub fn aggregate_hash_resource_rows(records: &[HashRecord]) -> Vec<HashResourceTableRow> {
    let mut rows = Vec::with_capacity(PAYLOAD_LOG2.len() * HASH_SCHEME_COUNT);
    for payload in PAYLOAD_LOG2 {
        for scheme in HashSchemeId::all() {
            let samples_1 = measured_samples(records, payload, scheme, 1);
            let samples_8 = measured_samples(records, payload, scheme, 8);
            rows.push(hash_resource_row(payload, scheme, &samples_1, &samples_8));
        }
    }
    rows
}

fn measured_samples(
    records: &[HashRecord],
    payload_log2: u32,
    scheme: HashSchemeId,
    threads: u32,
) -> Vec<&HashRecord> {
    records
        .iter()
        .filter(|record| {
            record.payload_log2 == payload_log2
                && record.scheme == scheme
                && record.threads == threads
                && !record.warmup
        })
        .collect()
}

fn incomparable_samples(samples: &[&HashRecord]) -> bool {
    let Some(first) = samples.first() else {
        return false;
    };
    let mut first_provenance = first.provenance.clone();
    first_provenance.workload_seed = None;
    samples.iter().skip(1).any(|record| {
        let mut provenance = record.provenance.clone();
        provenance.workload_seed = None;
        record.implementation_revision != first.implementation_revision
            || record.log2_n != first.log2_n
            || record.field != first.field
            || record.native_param != first.native_param
            || record.threads != first.threads
            || provenance != first_provenance
    })
}

fn different_builds_across_threads(samples_1: &[&HashRecord], samples_8: &[&HashRecord]) -> bool {
    let (Some(one), Some(eight)) = (samples_1.first(), samples_8.first()) else {
        return false;
    };
    let mut one_provenance = one.provenance.clone();
    let mut eight_provenance = eight.provenance.clone();
    one_provenance.threads = 0;
    eight_provenance.threads = 0;
    one_provenance.workload_seed = None;
    eight_provenance.workload_seed = None;
    one.implementation_revision != eight.implementation_revision
        || one.log2_n != eight.log2_n
        || one.field != eight.field
        || one.native_param != eight.native_param
        || one_provenance != eight_provenance
}

fn invalid_success_timings(samples: &[&HashRecord]) -> bool {
    samples.iter().any(|record| {
        record.status == RunStatus::Ok
            && ["commit", "open", "verify"]
                .iter()
                .any(|phase| !record.timings_ns.contains_key(*phase))
    })
}

fn invalid_success_resources(samples: &[&HashRecord]) -> bool {
    samples.iter().any(|record| {
        record.status == RunStatus::Ok
            && (record.proof_bytes.is_none()
                || record.commitment_bytes.is_none()
                || record.evaluation_bytes.is_none()
                || record.public_context_bytes.is_none())
    })
}

#[allow(clippy::too_many_lines)]
fn hash_timing_row(
    payload_log2: u32,
    scheme: HashSchemeId,
    field: &str,
    planned_log2_n: Option<u32>,
    threads: u32,
    samples: &[&HashRecord],
) -> HashTimingTableRow {
    if samples.is_empty() {
        return HashTimingTableRow {
            payload_log2,
            scheme,
            implementation_revision: None,
            field: field.to_owned(),
            log2_n: planned_log2_n,
            threads,
            status: RunStatus::Error,
            commit_s: None,
            commit_s_ci95: None,
            open_s: None,
            open_s_ci95: None,
            total_s: None,
            total_s_ci95: None,
            verify_s: None,
            verify_s_ci95: None,
            n_ok: 0,
            measured: false,
            gap_note: planned_gap_note(scheme, planned_log2_n.unwrap_or(0)),
        };
    }

    if incomparable_samples(samples) {
        return HashTimingTableRow {
            payload_log2,
            scheme,
            implementation_revision: None,
            field: field.to_owned(),
            log2_n: planned_log2_n,
            threads,
            status: RunStatus::Error,
            commit_s: None,
            commit_s_ci95: None,
            open_s: None,
            open_s_ci95: None,
            total_s: None,
            total_s_ci95: None,
            verify_s: None,
            verify_s_ci95: None,
            n_ok: 0,
            measured: true,
            gap_note: Some(GapNote::Custom(
                "Samples with different revisions, parameters, or environments were rejected."
                    .into(),
            )),
        };
    }

    if invalid_success_timings(samples) {
        return HashTimingTableRow {
            payload_log2,
            scheme,
            implementation_revision: Some(samples[0].implementation_revision.clone()),
            field: field.to_owned(),
            log2_n: planned_log2_n,
            threads,
            status: RunStatus::Error,
            commit_s: None,
            commit_s_ci95: None,
            open_s: None,
            open_s_ci95: None,
            total_s: None,
            total_s_ci95: None,
            verify_s: None,
            verify_s_ci95: None,
            n_ok: 0,
            measured: true,
            gap_note: Some(GapNote::Custom(
                "A successful sample was missing a required timing phase.".into(),
            )),
        };
    }

    let ok: Vec<&HashRecord> = samples
        .iter()
        .copied()
        .filter(|record| record.status == RunStatus::Ok)
        .collect();
    if ok.len() == samples.len() {
        let commit = phase_seconds_from(&ok, "commit", |record| &record.timings_ns);
        let open = phase_seconds_from(&ok, "open", |record| &record.timings_ns);
        let total: Vec<f64> = ok
            .iter()
            .filter_map(|record| {
                let setup = record.timings_ns.get("setup").copied().unwrap_or(0);
                let commit = *record.timings_ns.get("commit")? as f64 / 1e9;
                let open = *record.timings_ns.get("open")? as f64 / 1e9;
                Some(setup as f64 / 1e9 + commit + open)
            })
            .collect();
        return HashTimingTableRow {
            payload_log2,
            scheme,
            implementation_revision: Some(ok[0].implementation_revision.clone()),
            field: field.to_owned(),
            log2_n: ok[0].log2_n.or(planned_log2_n),
            threads,
            status: RunStatus::Ok,
            commit_s: median_f64(&commit),
            commit_s_ci95: median_ci95(&commit),
            open_s: median_f64(&open),
            open_s_ci95: median_ci95(&open),
            total_s: median_f64(&total),
            total_s_ci95: median_ci95(&total),
            verify_s: median_f64(&phase_seconds_from(&ok, "verify", |record| {
                &record.timings_ns
            })),
            verify_s_ci95: median_ci95(&phase_seconds_from(&ok, "verify", |record| {
                &record.timings_ns
            })),
            n_ok: ok.len(),
            measured: true,
            gap_note: whir_soundness_note(&ok)
                .or_else(|| planned_gap_note(scheme, ok[0].log2_n.or(planned_log2_n).unwrap_or(0))),
        };
    }

    let status = aggregate_gap_status(samples);
    HashTimingTableRow {
        payload_log2,
        scheme,
        implementation_revision: Some(samples[0].implementation_revision.clone()),
        field: field.to_owned(),
        log2_n: planned_log2_n,
        threads,
        status,
        commit_s: None,
        commit_s_ci95: None,
        open_s: None,
        open_s_ci95: None,
        total_s: None,
        total_s_ci95: None,
        verify_s: None,
        verify_s_ci95: None,
        n_ok: ok.len(),
        measured: true,
        gap_note: mixed_outcome_note(samples)
            .or_else(|| hash_gap_note(status, samples))
            .or_else(|| planned_gap_note(scheme, planned_log2_n.unwrap_or(0))),
    }
}

#[allow(clippy::too_many_lines)]
fn hash_resource_row(
    payload_log2: u32,
    scheme: HashSchemeId,
    samples_1: &[&HashRecord],
    samples_8: &[&HashRecord],
) -> HashResourceTableRow {
    if invalid_success_resources(samples_1)
        || invalid_success_resources(samples_8)
        || incomparable_samples(samples_1)
        || incomparable_samples(samples_8)
        || different_builds_across_threads(samples_1, samples_8)
    {
        return HashResourceTableRow {
            payload_log2,
            scheme,
            implementation_revision: None,
            status: RunStatus::Error,
            commitment_bytes: None,
            proof_bytes: None,
            evaluation_bytes: None,
            public_context_bytes: None,
            peak_rss_bytes_1: None,
            peak_rss_bytes_8: None,
            prep_s: None,
            state_bytes: None,
            measured: !samples_1.is_empty() || !samples_8.is_empty(),
            measured_8: !samples_8.is_empty(),
            status_8: RunStatus::Error,
            gap_note: Some(GapNote::Custom(
                "Incomplete or incomparable resource samples were rejected.".into(),
            )),
        };
    }

    let comm = if samples_1.is_empty() {
        samples_8
    } else {
        samples_1
    };
    let ok_comm: Vec<&HashRecord> = comm
        .iter()
        .copied()
        .filter(|record| record.status == RunStatus::Ok)
        .collect();
    let ok_1: Vec<&HashRecord> = samples_1
        .iter()
        .copied()
        .filter(|record| record.status == RunStatus::Ok)
        .collect();
    let ok_8: Vec<&HashRecord> = samples_8
        .iter()
        .copied()
        .filter(|record| record.status == RunStatus::Ok)
        .collect();

    if ok_comm.is_empty() || ok_comm.len() != comm.len() {
        let status = if samples_1.is_empty() && samples_8.is_empty() {
            RunStatus::Error
        } else {
            aggregate_gap_status(
                &samples_1
                    .iter()
                    .chain(samples_8.iter())
                    .copied()
                    .collect::<Vec<_>>(),
            )
        };
        return HashResourceTableRow {
            payload_log2,
            scheme,
            implementation_revision: comm
                .first()
                .map(|record| record.implementation_revision.clone()),
            status,
            commitment_bytes: None,
            proof_bytes: None,
            evaluation_bytes: None,
            public_context_bytes: None,
            peak_rss_bytes_1: None,
            peak_rss_bytes_8: None,
            prep_s: None,
            state_bytes: None,
            measured: !samples_1.is_empty() || !samples_8.is_empty(),
            measured_8: !samples_8.is_empty(),
            status_8: if samples_8.is_empty() {
                RunStatus::Error
            } else if samples_8
                .iter()
                .all(|record| record.status == RunStatus::Ok)
            {
                RunStatus::Ok
            } else {
                aggregate_gap_status(samples_8)
            },
            gap_note: mixed_outcome_note(comm)
                .or_else(|| hash_gap_note(status, samples_1))
                .or_else(|| whir_soundness_note(samples_1))
                .or_else(|| planned_gap_note(scheme, planned_log2_n(scheme, payload_log2))),
        };
    }

    let prep = phase_seconds_from(&ok_comm, "setup", |record| &record.timings_ns);
    HashResourceTableRow {
        payload_log2,
        scheme,
        implementation_revision: Some(ok_comm[0].implementation_revision.clone()),
        status: RunStatus::Ok,
        commitment_bytes: median_u64(ok_comm.iter().filter_map(|record| record.commitment_bytes)),
        proof_bytes: median_u64(ok_comm.iter().filter_map(|record| record.proof_bytes)),
        evaluation_bytes: median_u64(ok_comm.iter().filter_map(|record| record.evaluation_bytes)),
        public_context_bytes: median_u64(
            ok_comm
                .iter()
                .filter_map(|record| record.public_context_bytes),
        ),
        peak_rss_bytes_1: median_u64(ok_1.iter().filter_map(|record| record.peak_rss_bytes)),
        peak_rss_bytes_8: median_u64(ok_8.iter().filter_map(|record| record.peak_rss_bytes)),
        prep_s: median_f64(&prep),
        state_bytes: median_u64(ok_comm.iter().filter_map(|record| record.state_bytes)),
        measured: !samples_1.is_empty() || !samples_8.is_empty(),
        measured_8: !samples_8.is_empty(),
        status_8: if ok_8.is_empty() || ok_8.len() != samples_8.len() {
            if samples_8.is_empty() {
                RunStatus::Error
            } else {
                aggregate_gap_status(samples_8)
            }
        } else {
            RunStatus::Ok
        },
        gap_note: whir_soundness_note(&ok_comm)
            .or_else(|| planned_gap_note(scheme, planned_log2_n(scheme, payload_log2))),
    }
}

fn aggregate_gap_status(samples: &[&HashRecord]) -> RunStatus {
    if samples.iter().any(|record| {
        record.status == RunStatus::Oom || looks_like_oom(record.status_detail.as_deref())
    }) {
        RunStatus::Oom
    } else if samples
        .iter()
        .any(|record| record.status == RunStatus::Unsupported)
    {
        RunStatus::Unsupported
    } else {
        RunStatus::Error
    }
}

fn mixed_outcome_note(samples: &[&HashRecord]) -> Option<GapNote> {
    let ok = samples
        .iter()
        .filter(|record| record.status == RunStatus::Ok)
        .count();
    if ok == 0 || ok == samples.len() {
        return None;
    }
    let oom = samples
        .iter()
        .filter(|record| {
            record.status == RunStatus::Oom || looks_like_oom(record.status_detail.as_deref())
        })
        .count();
    let unsupported = samples
        .iter()
        .filter(|record| record.status == RunStatus::Unsupported)
        .count();
    let errors = samples.len().saturating_sub(ok + oom + unsupported);
    Some(GapNote::Custom(format!(
        "Partial result rejected: {ok}/{} samples succeeded, {oom} OOM, {unsupported} unsupported, {errors} errors.",
        samples.len()
    )))
}

fn hash_gap_note(status: RunStatus, samples: &[&HashRecord]) -> Option<GapNote> {
    if status != RunStatus::Unsupported && status != RunStatus::Error {
        return None;
    }
    samples
        .iter()
        .find_map(|record| record.status_detail.clone())
        .filter(|detail| !detail.is_empty())
        .map(GapNote::Custom)
}

fn planned_log2_n(scheme: HashSchemeId, payload_log2: u32) -> u32 {
    hash_case(payload_log2, scheme, 1).map_or(0, |case| case.log2_n)
}

fn planned_gap_note(scheme: HashSchemeId, log2_n: u32) -> Option<GapNote> {
    matches!(scheme, HashSchemeId::Plonky3Fri | HashSchemeId::Plonky3Stir)
        .then(|| plonky3_is_packed(log2_n))
        .filter(|packed| *packed)
        .map(|_| GapNote::PackedUnivariate)
}

fn statement_label(row: &HashTimingTableRow) -> &'static str {
    if matches!(
        row.scheme,
        HashSchemeId::Plonky3Fri | HashSchemeId::Plonky3Stir
    ) && row.log2_n.is_some_and(plonky3_is_packed)
    {
        "univariate batch"
    } else {
        row.scheme.statement()
    }
}

fn whir_soundness_note(samples: &[&HashRecord]) -> Option<GapNote> {
    samples
        .iter()
        .any(|record| {
            record
                .status_detail
                .as_deref()
                .is_some_and(|detail| detail.contains("UniqueDecoding"))
        })
        .then_some(GapNote::WhirUniqueDecoding)
}

fn scheme_cell(
    scheme: HashSchemeId,
    implementation_revision: Option<&str>,
    latex: bool,
    mark: Option<u32>,
) -> String {
    let label = apply_mark(
        if latex {
            scheme.latex_name()
        } else {
            scheme.display_name()
        },
        mark,
        latex,
    );
    let Some(revision) = implementation_revision else {
        return label;
    };
    let url = format!("{}/commit/{revision}", scheme.source_repo());
    if latex {
        format!(r"\href{{{url}}}{{{label}}}")
    } else {
        format!("[{label}]({url})")
    }
}

fn timing_cell(
    row: &HashTimingTableRow,
    value: Option<String>,
    latex: bool,
    mark: Option<u32>,
) -> String {
    if !row.measured && row.status == RunStatus::Error {
        return gap_token_pending(latex);
    }
    match row.status {
        RunStatus::Ok => value.unwrap_or_else(|| gap_token(RunStatus::Error, latex)),
        other => apply_mark(&gap_token(other, latex), mark, latex),
    }
}

fn resource_value_cell(
    measured: bool,
    status: RunStatus,
    value: Option<String>,
    latex: bool,
    mark: Option<u32>,
) -> String {
    if !measured && status == RunStatus::Error {
        return gap_token_pending(latex);
    }
    match status {
        RunStatus::Ok => value.unwrap_or_else(|| unknown_token(latex)),
        other => apply_mark(&gap_token(other, latex), mark, latex),
    }
}

fn log2_n_cell(row: &HashTimingTableRow, latex: bool, mark: Option<u32>) -> String {
    match (row.status, row.log2_n) {
        (RunStatus::Unsupported, _) => {
            apply_mark(&gap_token(RunStatus::Unsupported, latex), mark, latex)
        }
        (_, Some(log2_n)) => log2_n.to_string(),
        (_, None) => apply_mark(&gap_token(row.status, latex), mark, latex),
    }
}

fn total_bytes(row: &HashResourceTableRow) -> Option<u64> {
    Some(row.commitment_bytes? + row.evaluation_bytes? + row.proof_bytes?)
}

fn unique_gap_notes_timing(rows: &[HashTimingTableRow]) -> Vec<GapNote> {
    unique_gap_notes(rows.iter().filter_map(|row| row.gap_note.as_ref()))
}

fn unique_gap_notes_resources(rows: &[HashResourceTableRow]) -> Vec<GapNote> {
    unique_gap_notes(rows.iter().filter_map(|row| row.gap_note.as_ref()))
}

/// Markdown version of the hash timing comparison table.
#[must_use]
pub fn render_markdown_hash_timing_table(rows: &[HashTimingTableRow]) -> String {
    let notes = unique_gap_notes_timing(rows);
    let mut out = String::from(
        "| Nominal payload | Scheme | Security target | Statement | Field | log₂ N | Threads | Commit (s) | Open (s) | Cold total (s) | Verify (ms) |\n",
    );
    out.push_str("| ---: | --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for row in rows {
        let mark = footnote_index(&notes, row.gap_note.as_ref());
        let _ = writeln!(
            out,
            "| 2^{{{}}} | {} | {} bits | {} | ${}$ | {} | {} | {} | {} | {} | {} |",
            row.payload_log2,
            scheme_cell(
                row.scheme,
                row.implementation_revision.as_deref(),
                false,
                mark
            ),
            row.scheme.security_bits(),
            statement_label(row),
            row.field,
            log2_n_cell(row, false, mark),
            row.threads,
            timing_cell(
                row,
                timing_seconds_cell(row.commit_s, row.commit_s_ci95, false),
                false,
                mark
            ),
            timing_cell(
                row,
                timing_seconds_cell(row.open_s, row.open_s_ci95, false),
                false,
                mark
            ),
            timing_cell(
                row,
                timing_seconds_cell(row.total_s, row.total_s_ci95, false),
                false,
                mark
            ),
            timing_cell(
                row,
                timing_millis_cell(row.verify_s, row.verify_s_ci95, false),
                false,
                mark
            ),
        );
    }
    out.push_str(&markdown_footnotes(&notes));
    out
}

/// Markdown version of the hash resources comparison table.
#[must_use]
pub fn render_markdown_hash_resource_table(rows: &[HashResourceTableRow]) -> String {
    let notes = unique_gap_notes_resources(rows);
    let mut out = String::from(
        "| Nominal payload | Scheme | Commitment (B) | Evaluation (B) | Proof (B) | Total sent (B) | Excluded context (B) | Peak RSS 1-thread (GiB) | Peak RSS 8-thread (GiB) | Prep. (s) | State (GiB) |\n",
    );
    out.push_str("| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for row in rows {
        let mark = footnote_index(&notes, row.gap_note.as_ref());
        let _ = writeln!(
            out,
            "| 2^{{{}}} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            row.payload_log2,
            scheme_cell(
                row.scheme,
                row.implementation_revision.as_deref(),
                false,
                mark
            ),
            resource_value_cell(
                row.measured,
                row.status,
                row.commitment_bytes.map(|v| v.to_string()),
                false,
                mark
            ),
            resource_value_cell(
                row.measured,
                row.status,
                row.evaluation_bytes.map(|v| v.to_string()),
                false,
                mark
            ),
            resource_value_cell(
                row.measured,
                row.status,
                row.proof_bytes.map(|v| v.to_string()),
                false,
                mark
            ),
            resource_value_cell(
                row.measured,
                row.status,
                total_bytes(row).map(|v| v.to_string()),
                false,
                mark
            ),
            resource_value_cell(
                row.measured,
                row.status,
                row.public_context_bytes.map(|v| v.to_string()),
                false,
                mark
            ),
            resource_value_cell(
                row.measured,
                row.status,
                row.peak_rss_bytes_1.map(format_gib),
                false,
                mark
            ),
            resource_value_cell(
                row.measured_8,
                row.status_8,
                row.peak_rss_bytes_8.map(format_gib),
                false,
                mark
            ),
            resource_value_cell(
                row.measured,
                row.status,
                row.prep_s.map(format_prep),
                false,
                mark
            ),
            resource_value_cell(
                row.measured,
                row.status,
                row.state_bytes.map(format_gib),
                false,
                mark
            ),
        );
    }
    out.push_str(&markdown_footnotes(&notes));
    out
}

/// LaTeX `tabular` matching `tab:eval-hash-time`.
#[must_use]
pub fn render_latex_hash_timing_table(rows: &[HashTimingTableRow]) -> String {
    let notes = unique_gap_notes_timing(rows);
    let mut out = String::from(
        "\\begin{table}[H]\n\
         \\centering\n\
         \\caption[Timing comparison with hash-based PCSs]{Commitment, opening, and\n\
         verification time for the hash-based PCS roster on declared nominal payloads.\n\
         Security and statement columns expose configurations that are not directly comparable.\n\
         A dash denotes an unsupported parallel mode.\n\
         Timing cells are the median of fresh processes after warmup, followed by\n\
         a conservative distribution-free 95\\% confidence interval when the sample count supports one.\n\
         Scheme names link to the exact git commit that was measured.}\n\
         \\label{tab:eval-hash-time}\n\
         \\scriptsize\n\
         \\setlength{\\tabcolsep}{4pt}\n\
         \\begin{tabular}{@{}llclcccrrrr@{}}\n\
         \\toprule\n\
         Nominal payload & Scheme & Security & Statement & Field & $\\log_2 N$ & Threads\n\
         & Commit (s) & Open (s) & Cold total (s) & Verify (ms) \\\\\n\
         \\midrule\n",
    );
    append_payload_groups(
        rows,
        |row| {
            let mark = footnote_index(&notes, row.gap_note.as_ref());
            format!(
                "$2^{{{}}}$ & {} & {} bits & {} & ${}$ & {} & {} & {} & {} & {} & {} \\\\",
                row.payload_log2,
                scheme_cell(
                    row.scheme,
                    row.implementation_revision.as_deref(),
                    true,
                    mark
                ),
                row.scheme.security_bits(),
                statement_label(row),
                row.field,
                log2_n_cell(row, true, mark),
                row.threads,
                timing_cell(
                    row,
                    timing_seconds_cell(row.commit_s, row.commit_s_ci95, true),
                    true,
                    mark
                ),
                timing_cell(
                    row,
                    timing_seconds_cell(row.open_s, row.open_s_ci95, true),
                    true,
                    mark
                ),
                timing_cell(
                    row,
                    timing_seconds_cell(row.total_s, row.total_s_ci95, true),
                    true,
                    mark
                ),
                timing_cell(
                    row,
                    timing_millis_cell(row.verify_s, row.verify_s_ci95, true),
                    true,
                    mark
                ),
            )
        },
        &mut out,
    );
    out.push_str("\\bottomrule\n\\end{tabular}\n");
    out.push_str(&latex_footnotes(&notes));
    out.push_str("\\end{table}\n");
    out
}

/// LaTeX `tabular` matching `tab:eval-hash-resources`.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn render_latex_hash_resource_table(rows: &[HashResourceTableRow]) -> String {
    let notes = unique_gap_notes_resources(rows);
    let mut out = String::from(
        "\\begin{table}[H]\n\
         \\centering\n\
         \\caption[Communication and memory comparison with hash-based PCSs]{Proof\n\
         communication, prover memory, and reusable preprocessing for the workloads in\n\
         \\Cref{tab:eval-hash-time}. Total sent includes commitment, separately transmitted\n\
         evaluation, and proof; excluded verifier context is shown separately.}\n\
         \\label{tab:eval-hash-resources}\n\
         \\scriptsize\n\
         \\setlength{\\tabcolsep}{4pt}\n\
         \\begin{tabular}{@{}llrrrrrrrrr@{}}\n\
         \\toprule\n\
         Nominal payload & Scheme & Commitment (B) & Evaluation (B) & Proof (B) & Total sent (B) & Excl. context (B)\n\
         & \\multicolumn{2}{c}{Peak RSS (GiB)} & Prep. (s) & State (GiB) \\\\\n\
         \\cmidrule(lr){8-9}\n\
         & & & & & & & 1 thread & 8 threads & & \\\\\n\
         \\midrule\n",
    );

    let mut by_payload: BTreeMap<u32, Vec<&HashResourceTableRow>> = BTreeMap::new();
    for row in rows {
        by_payload.entry(row.payload_log2).or_default().push(row);
    }
    for (payload_index, payload) in PAYLOAD_LOG2.iter().enumerate() {
        if payload_index > 0 {
            out.push_str("\\addlinespace\n");
        }
        if let Some(group) = by_payload.get(payload) {
            for row in group {
                let mark = footnote_index(&notes, row.gap_note.as_ref());
                let _ = writeln!(
                    out,
                    "$2^{{{}}}$ & {} & {} & {} & {} & {} & {} & {} & {} & {} & {} \\\\",
                    row.payload_log2,
                    scheme_cell(
                        row.scheme,
                        row.implementation_revision.as_deref(),
                        true,
                        mark
                    ),
                    resource_value_cell(
                        row.measured,
                        row.status,
                        row.commitment_bytes.map(|v| v.to_string()),
                        true,
                        mark
                    ),
                    resource_value_cell(
                        row.measured,
                        row.status,
                        row.evaluation_bytes.map(|v| v.to_string()),
                        true,
                        mark
                    ),
                    resource_value_cell(
                        row.measured,
                        row.status,
                        row.proof_bytes.map(|v| v.to_string()),
                        true,
                        mark
                    ),
                    resource_value_cell(
                        row.measured,
                        row.status,
                        total_bytes(row).map(|v| v.to_string()),
                        true,
                        mark
                    ),
                    resource_value_cell(
                        row.measured,
                        row.status,
                        row.public_context_bytes.map(|v| v.to_string()),
                        true,
                        mark
                    ),
                    resource_value_cell(
                        row.measured,
                        row.status,
                        row.peak_rss_bytes_1.map(format_gib),
                        true,
                        mark
                    ),
                    resource_value_cell(
                        row.measured_8,
                        row.status_8,
                        row.peak_rss_bytes_8.map(format_gib),
                        true,
                        mark
                    ),
                    resource_value_cell(
                        row.measured,
                        row.status,
                        row.prep_s.map(format_prep),
                        true,
                        mark
                    ),
                    resource_value_cell(
                        row.measured,
                        row.status,
                        row.state_bytes.map(format_gib),
                        true,
                        mark
                    ),
                );
            }
        }
    }

    out.push_str("\\bottomrule\n\\end{tabular}\n");
    out.push_str(&latex_footnotes(&notes));
    out.push_str("\\end{table}\n");
    out
}

fn append_payload_groups<F>(rows: &[HashTimingTableRow], mut line: F, out: &mut String)
where
    F: FnMut(&HashTimingTableRow) -> String,
{
    let mut by_payload: BTreeMap<u32, Vec<&HashTimingTableRow>> = BTreeMap::new();
    for row in rows {
        by_payload.entry(row.payload_log2).or_default().push(row);
    }
    for (payload_index, payload) in PAYLOAD_LOG2.iter().enumerate() {
        if payload_index > 0 {
            out.push_str("\\addlinespace\n");
        }
        if let Some(group) = by_payload.get(payload) {
            for row in group {
                out.push_str(&line(row));
                out.push('\n');
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        aggregate_hash_resource_rows, aggregate_hash_timing_rows, render_latex_hash_resource_table,
        render_latex_hash_timing_table, render_markdown_hash_resource_table,
        render_markdown_hash_timing_table,
    };
    use crate::hash::HashSchemeId;
    use crate::observation::{HashRecord, Provenance, RunStatus};
    use crate::table::GapNote;
    use std::collections::BTreeMap;

    fn record(
        payload_log2: u32,
        scheme: HashSchemeId,
        threads: u32,
        status: RunStatus,
        commit_s: Option<f64>,
        open_s: Option<f64>,
        verify_s: Option<f64>,
    ) -> HashRecord {
        let mut timings_ns = BTreeMap::new();
        if let Some(seconds) = commit_s {
            timings_ns.insert("commit".into(), (seconds * 1e9) as u64);
        }
        if let Some(seconds) = open_s {
            timings_ns.insert("open".into(), (seconds * 1e9) as u64);
        }
        if let Some(seconds) = verify_s {
            timings_ns.insert("verify".into(), (seconds * 1e9) as u64);
        }
        HashRecord {
            status,
            status_detail: None,
            scheme,
            implementation_revision: scheme.revision().into(),
            payload_log2,
            log2_n: Some(payload_log2 - 5),
            field: match scheme {
                HashSchemeId::Akita => "2^{32}-99".into(),
                _ => "2^{31}-2^{24}+1".into(),
            },
            native_param: None,
            threads,
            sample: 0,
            warmup: false,
            timings_ns,
            proof_bytes: Some(61_337),
            commitment_bytes: Some(32),
            evaluation_bytes: Some(16),
            public_context_bytes: Some(0),
            state_bytes: Some(1_048_576),
            peak_rss_bytes: Some(119_000_000),
            provenance: Provenance::test_fixture(),
        }
    }

    #[test]
    fn hash_latex_table_has_threads_and_commit_links() {
        let records = vec![
            record(
                27,
                HashSchemeId::Akita,
                1,
                RunStatus::Ok,
                Some(0.159),
                Some(2.07),
                Some(0.0419),
            ),
            record(
                27,
                HashSchemeId::Akita,
                8,
                RunStatus::Ok,
                Some(0.080),
                Some(0.40),
                Some(0.042),
            ),
        ];
        let rows = aggregate_hash_timing_rows(&records);
        let latex = render_latex_hash_timing_table(&rows);
        assert!(latex.contains(r"\label{tab:eval-hash-time}"));
        assert!(latex.contains("Threads"));
        assert!(latex.contains("0.159"));
        assert!(latex.contains(r"\evalpending"));
        assert!(latex.contains(r"\href{https://github.com/LayerZero-Labs/akita/commit/"));
        let markdown = render_markdown_hash_timing_table(&rows);
        assert!(markdown.contains("| 1 |"));
        assert!(markdown.contains("| 8 |"));
        let resources = aggregate_hash_resource_rows(&records);
        let akita = resources
            .iter()
            .find(|row| row.payload_log2 == 27 && row.scheme == HashSchemeId::Akita)
            .expect("row");
        assert!(akita.peak_rss_bytes_1.is_some());
        assert!(akita.peak_rss_bytes_8.is_some());
        let resource_md = render_markdown_hash_resource_table(&resources);
        assert!(resource_md.contains("Proof (B)"));
        assert!(resource_md.contains("Evaluation (B)"));
        assert!(resource_md.contains("Excluded context (B)"));
        assert!(!resource_md.contains("KiB"));
        assert!(resource_md.contains("| 61337 |"));
        assert!(resource_md.contains("| 61385 |"));
        let resource_tex = render_latex_hash_resource_table(&resources);
        assert!(resource_tex.contains("Proof (B)"));
        assert!(resource_tex.contains("61337"));
    }

    #[test]
    fn unique_decoding_whir_rows_carry_a_footnote() {
        let mut one = record(
            33,
            HashSchemeId::Whir,
            1,
            RunStatus::Ok,
            Some(5.0),
            Some(30.0),
            Some(0.009),
        );
        one.status_detail = Some("soundness=UniqueDecoding,rate=1/2,pow_bits=20".into());
        let mut eight = one.clone();
        eight.threads = 8;
        let rows = aggregate_hash_timing_rows(&[one.clone(), eight]);
        let whir = rows
            .iter()
            .find(|row| {
                row.payload_log2 == 33 && row.scheme == HashSchemeId::Whir && row.threads == 1
            })
            .expect("row");
        assert_eq!(whir.gap_note, Some(GapNote::WhirUniqueDecoding));
        let latex = render_latex_hash_timing_table(std::slice::from_ref(whir));
        assert!(latex.contains(r"$^{(1)}$"));
        assert!(latex.contains("unique decoding"));
        let markdown = render_markdown_hash_timing_table(std::slice::from_ref(whir));
        assert!(markdown.contains("[WHIR (Plonky3)(1)]("));
        let resources = aggregate_hash_resource_rows(std::slice::from_ref(&one));
        let resource = resources
            .iter()
            .find(|row| row.payload_log2 == 33 && row.scheme == HashSchemeId::Whir)
            .expect("resource row");
        assert_eq!(resource.gap_note, Some(GapNote::WhirUniqueDecoding));
    }

    #[test]
    fn hash_matrix_unmeasured_cells_are_pending() {
        let rows = aggregate_hash_timing_rows(&[]);
        assert_eq!(rows.len(), crate::hash::HASH_CELL_COUNT);
        let whir = rows
            .iter()
            .find(|row| {
                row.payload_log2 == 31 && row.scheme == HashSchemeId::Whir && row.threads == 1
            })
            .expect("row");
        assert!(!whir.measured);
        let markdown = render_markdown_hash_timing_table(std::slice::from_ref(whir));
        assert!(markdown.contains("pending"));
    }

    #[test]
    fn packed_plonky3_univariate_rows_carry_a_footnote() {
        let rows = aggregate_hash_timing_rows(&[]);
        let packed = rows
            .iter()
            .find(|row| {
                row.payload_log2 == 29 && row.scheme == HashSchemeId::Plonky3Fri && row.threads == 1
            })
            .expect("packed fri");
        assert_eq!(packed.gap_note, Some(GapNote::PackedUnivariate));
        let unpacked = rows
            .iter()
            .find(|row| {
                row.payload_log2 == 27 && row.scheme == HashSchemeId::Plonky3Fri && row.threads == 1
            })
            .expect("unpacked fri");
        assert_eq!(unpacked.gap_note, None);
    }

    #[test]
    fn partial_success_does_not_hide_oom_samples() {
        let ok = record(
            27,
            HashSchemeId::Akita,
            1,
            RunStatus::Ok,
            Some(1.0),
            Some(2.0),
            Some(0.1),
        );
        let mut oom_one = ok.clone();
        oom_one.sample = 1;
        oom_one.status = RunStatus::Oom;
        oom_one.timings_ns.clear();
        let mut oom_two = oom_one.clone();
        oom_two.sample = 2;

        let records = [ok, oom_one, oom_two];
        let rows = aggregate_hash_timing_rows(&records);
        let row = rows
            .iter()
            .find(|row| {
                row.payload_log2 == 27 && row.scheme == HashSchemeId::Akita && row.threads == 1
            })
            .expect("row");
        assert_eq!(row.status, RunStatus::Oom);
        assert_eq!(row.n_ok, 1);
        assert_eq!(row.total_s, None);
        assert!(matches!(
            row.gap_note,
            Some(GapNote::Custom(ref note)) if note.contains("1/3 samples succeeded")
        ));
    }

    #[test]
    fn mixed_environments_are_rejected() {
        let one = record(
            27,
            HashSchemeId::Akita,
            1,
            RunStatus::Ok,
            Some(1.0),
            Some(2.0),
            Some(0.1),
        );
        let mut two = one.clone();
        two.sample = 1;
        two.provenance.cpu_model = "different machine".into();

        let rows = aggregate_hash_timing_rows(&[one, two]);
        let row = rows
            .iter()
            .find(|row| {
                row.payload_log2 == 27 && row.scheme == HashSchemeId::Akita && row.threads == 1
            })
            .expect("row");
        assert_eq!(row.status, RunStatus::Error);
        assert_eq!(row.n_ok, 0);
        assert!(matches!(
            row.gap_note,
            Some(GapNote::Custom(ref note)) if note.contains("different revisions")
        ));
    }

    #[test]
    fn resource_row_does_not_replace_failed_one_thread_samples() {
        let mut failed = record(27, HashSchemeId::Akita, 1, RunStatus::Oom, None, None, None);
        failed.timings_ns.clear();
        failed.commitment_bytes = None;
        let eight = record(
            27,
            HashSchemeId::Akita,
            8,
            RunStatus::Ok,
            Some(1.0),
            Some(2.0),
            Some(0.1),
        );

        let rows = aggregate_hash_resource_rows(&[failed, eight]);
        let row = rows
            .iter()
            .find(|row| row.payload_log2 == 27 && row.scheme == HashSchemeId::Akita)
            .expect("row");
        assert_eq!(row.status, RunStatus::Oom);
        assert_eq!(row.status_8, RunStatus::Ok);
        assert_eq!(row.commitment_bytes, None);
    }
}
