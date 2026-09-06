//! Render the hash timing and resource comparison as Markdown or LaTeX.

use crate::hash::{hash_matrix, HashSchemeId};
use crate::lattice::PAYLOAD_LOG2;
use crate::observation::{looks_like_oom, HashRecord, RunStatus};
use crate::table::{
    apply_mark, footnote_index, format_gib, format_kib, format_prep, gap_token, gap_token_pending,
    latex_footnotes, markdown_footnotes, median_f64, median_u64, phase_seconds_from, sample_std,
    timing_millis_cell, timing_seconds_cell, unique_gap_notes, GapNote,
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
    /// Sample standard deviation of commit seconds.
    pub commit_s_std: Option<f64>,
    /// Median opening/prove seconds, when `status` is ok.
    pub open_s: Option<f64>,
    /// Sample standard deviation of opening seconds.
    pub open_s_std: Option<f64>,
    /// Median commit+open seconds, when `status` is ok.
    pub total_s: Option<f64>,
    /// Sample standard deviation of per-sample commit+open seconds.
    pub total_s_std: Option<f64>,
    /// Median verify seconds, when `status` is ok.
    pub verify_s: Option<f64>,
    /// Sample standard deviation of verify seconds.
    pub verify_s_std: Option<f64>,
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
    /// Cell outcome after aggregating 1-thread samples (communication / prep).
    pub status: RunStatus,
    /// Median commitment size in bytes.
    pub commitment_bytes: Option<u64>,
    /// Median opening-proof size in bytes.
    pub proof_bytes: Option<u64>,
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
    let mut rows = Vec::with_capacity(15);
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
                && !record.historical
        })
        .collect()
}

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
            field: field.to_owned(),
            log2_n: planned_log2_n,
            threads,
            status: RunStatus::Error,
            commit_s: None,
            commit_s_std: None,
            open_s: None,
            open_s_std: None,
            total_s: None,
            total_s_std: None,
            verify_s: None,
            verify_s_std: None,
            n_ok: 0,
            measured: false,
            gap_note: None,
        };
    }

    let ok: Vec<&HashRecord> = samples
        .iter()
        .copied()
        .filter(|record| record.status == RunStatus::Ok)
        .collect();
    if !ok.is_empty() {
        let commit = phase_seconds_from(&ok, "commit", |record| &record.timings_ns);
        let open = phase_seconds_from(&ok, "open", |record| &record.timings_ns);
        let total: Vec<f64> = ok
            .iter()
            .filter_map(|record| {
                let commit = *record.timings_ns.get("commit")? as f64 / 1e9;
                let open = *record.timings_ns.get("open")? as f64 / 1e9;
                Some(commit + open)
            })
            .collect();
        return HashTimingTableRow {
            payload_log2,
            scheme,
            field: field.to_owned(),
            log2_n: ok[0].log2_n.or(planned_log2_n),
            threads,
            status: RunStatus::Ok,
            commit_s: median_f64(&commit),
            commit_s_std: sample_std(&commit),
            open_s: median_f64(&open),
            open_s_std: sample_std(&open),
            total_s: median_f64(&total),
            total_s_std: sample_std(&total),
            verify_s: median_f64(&phase_seconds_from(&ok, "verify", |record| {
                &record.timings_ns
            })),
            verify_s_std: sample_std(&phase_seconds_from(&ok, "verify", |record| {
                &record.timings_ns
            })),
            n_ok: ok.len(),
            measured: true,
            gap_note: whir_soundness_note(&ok),
        };
    }

    let status = aggregate_gap_status(samples);
    HashTimingTableRow {
        payload_log2,
        scheme,
        field: field.to_owned(),
        log2_n: planned_log2_n,
        threads,
        status,
        commit_s: None,
        commit_s_std: None,
        open_s: None,
        open_s_std: None,
        total_s: None,
        total_s_std: None,
        verify_s: None,
        verify_s_std: None,
        n_ok: 0,
        measured: true,
        gap_note: hash_gap_note(status, samples),
    }
}

fn hash_resource_row(
    payload_log2: u32,
    scheme: HashSchemeId,
    samples_1: &[&HashRecord],
    samples_8: &[&HashRecord],
) -> HashResourceTableRow {
    let comm = if samples_1
        .iter()
        .any(|record| record.status == RunStatus::Ok)
    {
        samples_1
    } else {
        samples_8
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

    if ok_comm.is_empty() {
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
            status,
            commitment_bytes: None,
            proof_bytes: None,
            peak_rss_bytes_1: None,
            peak_rss_bytes_8: None,
            prep_s: None,
            state_bytes: None,
            measured: !samples_1.is_empty() || !samples_8.is_empty(),
            measured_8: !samples_8.is_empty(),
            status_8: if samples_8.is_empty() {
                RunStatus::Error
            } else {
                aggregate_gap_status(samples_8)
            },
            gap_note: hash_gap_note(status, samples_1).or_else(|| whir_soundness_note(samples_1)),
        };
    }

    let prep = phase_seconds_from(&ok_comm, "setup", |record| &record.timings_ns);
    HashResourceTableRow {
        payload_log2,
        scheme,
        status: RunStatus::Ok,
        commitment_bytes: median_u64(ok_comm.iter().filter_map(|record| record.commitment_bytes)),
        proof_bytes: median_u64(ok_comm.iter().filter_map(|record| record.proof_bytes)),
        peak_rss_bytes_1: median_u64(ok_1.iter().filter_map(|record| record.peak_rss_bytes)),
        peak_rss_bytes_8: median_u64(ok_8.iter().filter_map(|record| record.peak_rss_bytes)),
        prep_s: median_f64(&prep),
        state_bytes: median_u64(ok_comm.iter().filter_map(|record| record.state_bytes)),
        measured: !samples_1.is_empty() || !samples_8.is_empty(),
        measured_8: !samples_8.is_empty(),
        status_8: if ok_8.is_empty() {
            if samples_8.is_empty() {
                RunStatus::Error
            } else {
                aggregate_gap_status(samples_8)
            }
        } else {
            RunStatus::Ok
        },
        gap_note: whir_soundness_note(&ok_comm),
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

fn scheme_cell(scheme: HashSchemeId, latex: bool, mark: Option<u32>) -> String {
    if latex {
        apply_mark(
            &format!(
                r"\href{{{}}}{{{}}}",
                scheme.commit_url(),
                scheme.latex_name()
            ),
            mark,
            true,
        )
    } else {
        format!(
            "[{}]({})",
            apply_mark(scheme.display_name(), mark, false),
            scheme.commit_url()
        )
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
        RunStatus::Ok => value.unwrap_or_else(|| gap_token_pending(latex)),
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

fn total_kib(row: &HashResourceTableRow) -> Option<f64> {
    let commitment = row.commitment_bytes?;
    let proof = row.proof_bytes?;
    Some((commitment + proof) as f64 / 1024.0)
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
        "| Payload | Scheme | Field | log₂ N | Threads | Commit (s) | Open (s) | Total (s) | Verify (ms) |\n",
    );
    out.push_str("| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for row in rows {
        let mark = footnote_index(&notes, row.gap_note.as_ref());
        let _ = writeln!(
            out,
            "| 2^{{{}}} | {} | ${}$ | {} | {} | {} | {} | {} | {} |",
            row.payload_log2,
            scheme_cell(row.scheme, false, mark),
            row.field,
            log2_n_cell(row, false, mark),
            row.threads,
            timing_cell(
                row,
                timing_seconds_cell(row.commit_s, row.commit_s_std, false),
                false,
                mark
            ),
            timing_cell(
                row,
                timing_seconds_cell(row.open_s, row.open_s_std, false),
                false,
                mark
            ),
            timing_cell(
                row,
                timing_seconds_cell(row.total_s, row.total_s_std, false),
                false,
                mark
            ),
            timing_cell(
                row,
                timing_millis_cell(row.verify_s, row.verify_s_std, false),
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
        "| Payload | Scheme | Commitment (B) | Proof (KiB) | Total (KiB) | Peak RSS 1-thread (GiB) | Peak RSS 8-thread (GiB) | Prep. (s) | State (GiB) |\n",
    );
    out.push_str("| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for row in rows {
        let mark = footnote_index(&notes, row.gap_note.as_ref());
        let _ = writeln!(
            out,
            "| 2^{{{}}} | {} | {} | {} | {} | {} | {} | {} | {} |",
            row.payload_log2,
            scheme_cell(row.scheme, false, mark),
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
                row.proof_bytes.map(format_kib),
                false,
                mark
            ),
            resource_value_cell(
                row.measured,
                row.status,
                total_kib(row).map(|v| format!("{v:.1}")),
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
         verification time for Akita, WHIR, and BaseFold on matched dense multilinear\n\
         openings.  A dash denotes an unsupported parallel mode.\n\
         Timing cells are the median of fresh processes after warmup, shown as\n\
         median $\\pm$ sample standard deviation when $n\\ge 2$.\n\
         Scheme names link to the exact git commit that was measured.}\n\
         \\label{tab:eval-hash-time}\n\
         \\scriptsize\n\
         \\setlength{\\tabcolsep}{4pt}\n\
         \\begin{tabular}{@{}llcccrrrr@{}}\n\
         \\toprule\n\
         Payload & Scheme & Field & $\\log_2 N$ & Threads\n\
         & Commit (s) & Open (s) & Total (s) & Verify (ms) \\\\\n\
         \\midrule\n",
    );
    append_payload_groups(
        rows,
        |row| {
            let mark = footnote_index(&notes, row.gap_note.as_ref());
            format!(
                "$2^{{{}}}$ & {} & ${}$ & {} & {} & {} & {} & {} & {} \\\\",
                row.payload_log2,
                scheme_cell(row.scheme, true, mark),
                row.field,
                log2_n_cell(row, true, mark),
                row.threads,
                timing_cell(
                    row,
                    timing_seconds_cell(row.commit_s, row.commit_s_std, true),
                    true,
                    mark
                ),
                timing_cell(
                    row,
                    timing_seconds_cell(row.open_s, row.open_s_std, true),
                    true,
                    mark
                ),
                timing_cell(
                    row,
                    timing_seconds_cell(row.total_s, row.total_s_std, true),
                    true,
                    mark
                ),
                timing_cell(
                    row,
                    timing_millis_cell(row.verify_s, row.verify_s_std, true),
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
pub fn render_latex_hash_resource_table(rows: &[HashResourceTableRow]) -> String {
    let notes = unique_gap_notes_resources(rows);
    let mut out = String::from(
        "\\begin{table}[H]\n\
         \\centering\n\
         \\caption[Communication and memory comparison with hash-based PCSs]{Proof\n\
         communication, prover memory, and reusable preprocessing for the workloads in\n\
         \\Cref{tab:eval-hash-time}.}\n\
         \\label{tab:eval-hash-resources}\n\
         \\scriptsize\n\
         \\setlength{\\tabcolsep}{4pt}\n\
         \\begin{tabular}{@{}llrrrrrrr@{}}\n\
         \\toprule\n\
         Payload & Scheme & Commitment (B) & Proof (KiB) & Total (KiB)\n\
         & \\multicolumn{2}{c}{Peak RSS (GiB)} & Prep. (s) & State (GiB) \\\\\n\
         \\cmidrule(lr){6-7}\n\
         & & & & & 1 thread & 8 threads & & \\\\\n\
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
                    "$2^{{{}}}$ & {} & {} & {} & {} & {} & {} & {} & {} \\\\",
                    row.payload_log2,
                    scheme_cell(row.scheme, true, mark),
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
                        row.proof_bytes.map(format_kib),
                        true,
                        mark
                    ),
                    resource_value_cell(
                        row.measured,
                        row.status,
                        total_kib(row).map(|v| format!("{v:.1}")),
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
        aggregate_hash_resource_rows, aggregate_hash_timing_rows, render_latex_hash_timing_table,
        render_markdown_hash_timing_table,
    };
    use crate::hash::HashSchemeId;
    use crate::observation::{HashRecord, Provenance, RunStatus, RESULT_SCHEMA_VERSION};
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
            schema_version: RESULT_SCHEMA_VERSION,
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
            historical: false,
            timings_ns,
            proof_bytes: Some(61_337),
            commitment_bytes: Some(32),
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
        assert!(markdown.contains("[WHIR(1)]("));
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
        assert_eq!(rows.len(), 30);
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
}
