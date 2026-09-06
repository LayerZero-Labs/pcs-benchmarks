//! Render the lattice timing and resource comparison as Markdown or LaTeX.

use crate::lattice::{lattice_matrix, SchemeId, PAYLOAD_LOG2};
use crate::observation::{looks_like_greyhound_sis, looks_like_oom, LatticeRecord, RunStatus};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// One aggregated timing-table row.
#[derive(Clone, Debug, PartialEq)]
pub struct TimingTableRow {
    /// Target payload exponent.
    pub payload_log2: u32,
    /// Scheme.
    pub scheme: SchemeId,
    /// Field label.
    pub field: String,
    /// Native `log2 N`, when the scheme has a matching instance.
    pub log2_n: Option<u32>,
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
    /// Footnote for an unsupported dash, when this row is not a runnable input.
    pub gap_note: Option<GapNote>,
}

/// One aggregated resources-table row.
#[derive(Clone, Debug, PartialEq)]
pub struct ResourceTableRow {
    /// Target payload exponent.
    pub payload_log2: u32,
    /// Scheme.
    pub scheme: SchemeId,
    /// Cell outcome after aggregating samples.
    pub status: RunStatus,
    /// Median commitment size in bytes.
    pub commitment_bytes: Option<u64>,
    /// Median opening-proof size in bytes.
    pub proof_bytes: Option<u64>,
    /// Median peak RSS in bytes.
    pub peak_rss_bytes: Option<u64>,
    /// Median preprocessing / CRS-generation seconds.
    pub prep_s: Option<f64>,
    /// Median reusable preprocessing state in bytes.
    pub state_bytes: Option<u64>,
    /// Whether any sample was recorded for this cell.
    pub measured: bool,
    /// Footnote for an unsupported dash, when this row is not a runnable input.
    pub gap_note: Option<GapNote>,
}

/// Why a table cell is annotated or dashed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GapNote {
    /// Akita has no generated fp32-dense row for the requested `nv`.
    AkitaCatalog,
    /// RoKoKo has no native parameter set for this payload.
    RokokoNative,
    /// Greyhound's SIS parameter search cannot secure the inner commitment.
    GreyhoundSis,
    /// WHIR used unique decoding because capacity/Johnson bounds exceed KoalaBear grind.
    WhirUniqueDecoding,
    /// Some other recorded unsupported reason.
    Custom(String),
}

/// Aggregate measured (non-warmup, non-historical) records into timing rows.
#[must_use]
pub fn aggregate_timing_rows(records: &[LatticeRecord]) -> Vec<TimingTableRow> {
    lattice_matrix()
        .into_iter()
        .map(|case| {
            let samples = measured_samples(records, case.payload_log2, case.scheme);
            timing_row_from_samples(
                case.payload_log2,
                case.scheme,
                case.field.name,
                case.log2_n,
                &samples,
            )
        })
        .collect()
}

/// Aggregate measured records into the communication / memory table.
#[must_use]
pub fn aggregate_resource_rows(records: &[LatticeRecord]) -> Vec<ResourceTableRow> {
    lattice_matrix()
        .into_iter()
        .map(|case| {
            let samples = measured_samples(records, case.payload_log2, case.scheme);
            resource_row_from_samples(case.payload_log2, case.scheme, case.log2_n, &samples)
        })
        .collect()
}

fn measured_samples(
    records: &[LatticeRecord],
    payload_log2: u32,
    scheme: SchemeId,
) -> Vec<&LatticeRecord> {
    records
        .iter()
        .filter(|record| {
            record.payload_log2 == payload_log2
                && record.scheme == scheme
                && !record.warmup
                && !record.historical
        })
        .collect()
}

fn timing_row_from_samples(
    payload_log2: u32,
    scheme: SchemeId,
    field: &str,
    planned_log2_n: Option<u32>,
    samples: &[&LatticeRecord],
) -> TimingTableRow {
    if samples.is_empty() {
        let status = if planned_log2_n.is_none() {
            RunStatus::Unsupported
        } else {
            RunStatus::Error
        };
        return TimingTableRow {
            payload_log2,
            scheme,
            field: field.to_owned(),
            log2_n: planned_log2_n,
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
            measured: false,
            gap_note: gap_note(scheme, status, samples),
        };
    }

    let ok: Vec<&LatticeRecord> = samples
        .iter()
        .copied()
        .filter(|record| record.status == RunStatus::Ok)
        .collect();
    if !ok.is_empty() {
        let commit = phase_seconds(&ok, "commit");
        let open = phase_seconds(&ok, "open");
        let total: Vec<f64> = ok
            .iter()
            .filter_map(|record| {
                let commit = *record.timings_ns.get("commit")? as f64 / 1e9;
                let open = *record.timings_ns.get("open")? as f64 / 1e9;
                Some(commit + open)
            })
            .collect();
        return TimingTableRow {
            payload_log2,
            scheme,
            field: field.to_owned(),
            log2_n: ok[0].log2_n.or(planned_log2_n),
            status: RunStatus::Ok,
            commit_s: median_f64(&commit),
            commit_s_std: sample_std(&commit),
            open_s: median_f64(&open),
            open_s_std: sample_std(&open),
            total_s: median_f64(&total),
            total_s_std: sample_std(&total),
            verify_s: median_f64(&phase_seconds(&ok, "verify")),
            verify_s_std: sample_std(&phase_seconds(&ok, "verify")),
            n_ok: ok.len(),
            measured: true,
            gap_note: None,
        };
    }

    let status = aggregate_gap_status(samples);
    TimingTableRow {
        payload_log2,
        scheme,
        field: field.to_owned(),
        log2_n: planned_log2_n,
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
        gap_note: gap_note(scheme, status, samples),
    }
}

fn resource_row_from_samples(
    payload_log2: u32,
    scheme: SchemeId,
    planned_log2_n: Option<u32>,
    samples: &[&LatticeRecord],
) -> ResourceTableRow {
    if samples.is_empty() {
        let status = if planned_log2_n.is_none() {
            RunStatus::Unsupported
        } else {
            RunStatus::Error
        };
        return ResourceTableRow {
            payload_log2,
            scheme,
            status,
            commitment_bytes: None,
            proof_bytes: None,
            peak_rss_bytes: None,
            prep_s: None,
            state_bytes: None,
            measured: false,
            gap_note: gap_note(scheme, status, samples),
        };
    }

    let ok: Vec<&LatticeRecord> = samples
        .iter()
        .copied()
        .filter(|record| record.status == RunStatus::Ok)
        .collect();
    if !ok.is_empty() {
        let prep = phase_seconds(&ok, "setup");
        let prep_s = if prep.is_empty() && matches!(scheme, SchemeId::Greyhound) {
            Some(0.0)
        } else {
            median_f64(&prep)
        };
        let state_bytes = median_u64(ok.iter().filter_map(|record| record.state_bytes));
        let state_bytes = if state_bytes.is_none() && matches!(scheme, SchemeId::Greyhound) {
            Some(0)
        } else {
            state_bytes
        };
        return ResourceTableRow {
            payload_log2,
            scheme,
            status: RunStatus::Ok,
            commitment_bytes: median_u64(ok.iter().filter_map(|record| record.commitment_bytes)),
            proof_bytes: median_u64(ok.iter().filter_map(|record| record.proof_bytes)),
            peak_rss_bytes: median_u64(ok.iter().filter_map(|record| record.peak_rss_bytes)),
            prep_s,
            state_bytes,
            measured: true,
            gap_note: None,
        };
    }

    let status = aggregate_gap_status(samples);
    ResourceTableRow {
        payload_log2,
        scheme,
        status,
        commitment_bytes: None,
        proof_bytes: None,
        peak_rss_bytes: None,
        prep_s: None,
        state_bytes: None,
        measured: true,
        gap_note: gap_note(scheme, status, samples),
    }
}

fn aggregate_gap_status(samples: &[&LatticeRecord]) -> RunStatus {
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

fn gap_note(scheme: SchemeId, status: RunStatus, samples: &[&LatticeRecord]) -> Option<GapNote> {
    if scheme == SchemeId::Greyhound
        && status == RunStatus::Error
        && samples
            .iter()
            .any(|record| looks_like_greyhound_sis(record.status_detail.as_deref()))
    {
        return Some(GapNote::GreyhoundSis);
    }
    if status != RunStatus::Unsupported {
        return None;
    }
    match scheme {
        SchemeId::Akita | SchemeId::AkitaPr466 => Some(GapNote::AkitaCatalog),
        SchemeId::Rokoko => Some(GapNote::RokokoNative),
        SchemeId::Greyhound => Some(
            samples
                .iter()
                .find_map(|record| record.status_detail.clone())
                .filter(|detail| !detail.is_empty())
                .map_or_else(
                    || GapNote::Custom("Unsupported Greyhound input.".into()),
                    GapNote::Custom,
                ),
        ),
    }
}

impl GapNote {
    fn markdown(&self) -> String {
        match self {
            Self::AkitaCatalog => {
                "Pinned Akita fp32-dense catalog has no production row for $n_v=22$ (payload $2^{27}$) or $n_v=24$ (payload $2^{29}$).".into()
            }
            Self::RokokoNative => {
                "RoKoKo ships only native sets `p-26`, `p-28`, and `p-30`; no instance matches this payload.".into()
            }
            Self::GreyhoundSis => {
                "Greyhound cannot make the inner Ajtai commitments SIS-secure at $\\log_2 N=30$ (`kappa` $\\le$ 32). Labrador rejects the instance (`polcom_reduce`: inner commitments not secure). This is not an out-of-memory failure.".into()
            }
            Self::WhirUniqueDecoding => {
                "WHIR uses unique decoding at this size so the 128-bit transcript-error target still holds on KoalaBear. Capacity bound and Johnson bound need more than 30 bits of grinding, which the field cannot support. The larger proof is the unique-decoding query schedule.".into()
            }
            Self::Custom(detail) => detail.clone(),
        }
    }

    fn latex(&self) -> String {
        match self {
            Self::AkitaCatalog => {
                "Pinned Akita fp32-dense catalog has no production row for $n_v=22$ (payload $2^{27}$) or $n_v=24$ (payload $2^{29}$).".into()
            }
            Self::RokokoNative => {
                "RoKoKo ships only native sets \\texttt{p-26}, \\texttt{p-28}, and \\texttt{p-30}; no instance matches this payload.".into()
            }
            Self::GreyhoundSis => {
                "Greyhound cannot make the inner Ajtai commitments SIS-secure at $\\log_2 N=30$ ($\\kappa \\le 32$). Labrador rejects the instance (\\texttt{polcom\\_reduce}: inner commitments not secure). This is not an out-of-memory failure.".into()
            }
            Self::WhirUniqueDecoding => {
                "WHIR uses unique decoding at this size so the 128-bit transcript-error target still holds on KoalaBear. Capacity bound and Johnson bound need more than 30 bits of grinding, which the field cannot support. The larger proof is the unique-decoding query schedule.".into()
            }
            Self::Custom(detail) => escape_tex_footnote(detail),
        }
    }
}

fn escape_tex_footnote(text: &str) -> String {
    text.replace('\\', r"\textbackslash{}")
        .replace('_', r"\_")
        .replace('%', r"\%")
        .replace('#', r"\#")
        .replace('&', r"\&")
}

fn unique_gap_notes_timing(rows: &[TimingTableRow]) -> Vec<GapNote> {
    unique_gap_notes(rows.iter().filter_map(|row| row.gap_note.as_ref()))
}

fn unique_gap_notes_resources(rows: &[ResourceTableRow]) -> Vec<GapNote> {
    unique_gap_notes(rows.iter().filter_map(|row| row.gap_note.as_ref()))
}

pub(crate) fn unique_gap_notes<'a>(notes: impl Iterator<Item = &'a GapNote>) -> Vec<GapNote> {
    let mut out = Vec::new();
    for note in notes {
        if !out.contains(note) {
            out.push(note.clone());
        }
    }
    out
}

pub(crate) fn footnote_index(notes: &[GapNote], note: Option<&GapNote>) -> Option<u32> {
    let note = note?;
    notes
        .iter()
        .position(|candidate| candidate == note)
        .map(|index| u32::try_from(index + 1).unwrap_or(1))
}

pub(crate) fn apply_mark(token: &str, index: Option<u32>, latex: bool) -> String {
    let Some(index) = index else {
        return token.to_owned();
    };
    if latex {
        format!("{token}$^{{({index})}}$")
    } else {
        format!("{token}({index})")
    }
}

pub(crate) fn markdown_footnotes(notes: &[GapNote]) -> String {
    if notes.is_empty() {
        return String::new();
    }
    let mut out = String::from("\n");
    for (offset, note) in notes.iter().enumerate() {
        let _ = writeln!(out, "**({})** {}", offset + 1, note.markdown());
    }
    out
}

pub(crate) fn latex_footnotes(notes: &[GapNote]) -> String {
    if notes.is_empty() {
        return String::new();
    }
    let mut out = String::from("\\smallskip\n{\\footnotesize\n");
    for (offset, note) in notes.iter().enumerate() {
        let _ = writeln!(
            out,
            "\\noindent$^{{({})}}$ {}\\\\",
            offset + 1,
            note.latex()
        );
    }
    out.push_str("}\n");
    out
}

pub(crate) fn phase_seconds_from<'a, R>(
    samples: &[&'a R],
    phase: &str,
    timings: impl Fn(&'a R) -> &'a BTreeMap<String, u64>,
) -> Vec<f64> {
    samples
        .iter()
        .filter_map(|record| timings(record).get(phase).copied())
        .map(|ns| ns as f64 / 1_000_000_000.0)
        .collect()
}

fn phase_seconds(samples: &[&LatticeRecord], phase: &str) -> Vec<f64> {
    phase_seconds_from(samples, phase, |record| &record.timings_ns)
}

pub(crate) fn median_f64(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        Some(sorted[mid])
    } else {
        Some(f64::midpoint(sorted[mid - 1], sorted[mid]))
    }
}

pub(crate) fn median_u64<I>(values: I) -> Option<u64>
where
    I: Iterator<Item = u64>,
{
    let mut values: Vec<u64> = values.collect();
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    let mid = values.len() / 2;
    if values.len() % 2 == 1 {
        Some(values[mid])
    } else {
        Some(values[mid - 1] / 2 + values[mid] / 2)
    }
}

/// Sample standard deviation (Bessel-corrected). `None` when `n < 2`.
#[must_use]
pub(crate) fn sample_std(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let var = values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / (n - 1.0);
    Some(var.sqrt())
}

/// Format a wall-clock second value with paper-style significant figures.
#[must_use]
pub fn format_seconds(seconds: f64) -> String {
    if seconds >= 10.0 {
        format!("{seconds:.1}")
    } else if seconds >= 1.0 {
        format!("{seconds:.2}")
    } else {
        format!("{seconds:.3}")
    }
}

/// Format a standard-deviation in seconds with enough digits to be visible.
#[must_use]
pub(crate) fn format_spread_seconds(seconds: f64) -> String {
    if seconds >= 1.0 {
        format!("{seconds:.2}")
    } else if seconds >= 0.01 {
        format!("{seconds:.3}")
    } else {
        format!("{seconds:.4}")
    }
}

/// Format verification time in milliseconds.
#[must_use]
pub fn format_millis(seconds: f64) -> String {
    let millis = seconds * 1_000.0;
    if millis >= 100.0 {
        format!("{millis:.0}")
    } else {
        format!("{millis:.1}")
    }
}

pub(crate) fn format_spread_millis(seconds: f64) -> String {
    let millis = seconds * 1_000.0;
    if millis >= 10.0 {
        format!("{millis:.1}")
    } else {
        format!("{millis:.2}")
    }
}

pub(crate) fn format_pm(median: &str, spread: Option<String>, latex: bool) -> String {
    match spread {
        Some(spread) if latex => format!("${median} \\pm {spread}$"),
        Some(spread) => format!("{median} ± {spread}"),
        None => median.to_owned(),
    }
}

pub(crate) fn format_kib(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / 1024.0)
}

pub(crate) fn format_gib(bytes: u64) -> String {
    let gib = bytes as f64 / 1_073_741_824.0;
    if gib >= 10.0 {
        format!("{gib:.1}")
    } else if gib >= 1.0 {
        format!("{gib:.2}")
    } else if gib >= 0.1 {
        format!("{gib:.3}")
    } else {
        format!("{gib:.4}")
    }
}

pub(crate) fn format_prep(seconds: f64) -> String {
    if seconds == 0.0 {
        "0".into()
    } else if seconds >= 1.0 {
        format_seconds(seconds)
    } else if seconds >= 0.1 {
        format!("{seconds:.3}")
    } else {
        format!("{seconds:.4}")
    }
}

fn scheme_cell(scheme: SchemeId, latex: bool) -> String {
    if latex {
        format!(
            r"\href{{{}}}{{{}}}",
            scheme.commit_url(),
            scheme.latex_name()
        )
    } else {
        format!("[{}]({})", scheme.display_name(), scheme.commit_url())
    }
}

fn cell(row: &TimingTableRow, value: Option<String>, latex: bool, mark: Option<u32>) -> String {
    if !row.measured && row.status == RunStatus::Error {
        return gap_token_pending(latex);
    }
    match row.status {
        RunStatus::Ok => value.unwrap_or_else(|| gap_token(RunStatus::Error, latex)),
        other => apply_mark(&gap_token(other, latex), mark, latex),
    }
}

fn resource_cell(
    row: &ResourceTableRow,
    value: Option<String>,
    latex: bool,
    mark: Option<u32>,
) -> String {
    if !row.measured && row.status == RunStatus::Error {
        return gap_token_pending(latex);
    }
    match row.status {
        RunStatus::Ok => value.unwrap_or_else(|| gap_token_pending(latex)),
        other => apply_mark(&gap_token(other, latex), mark, latex),
    }
}

pub(crate) fn gap_token(status: RunStatus, latex: bool) -> String {
    match (status, latex) {
        (RunStatus::Unsupported, false) => "—".into(),
        (RunStatus::Oom, true) => r"\evaloom".into(),
        (RunStatus::Oom, false) => "OOM".into(),
        (_, true) => r"\evalunsupported".into(),
        (_, false) => "err".into(),
    }
}

pub(crate) fn gap_token_pending(latex: bool) -> String {
    if latex {
        r"\evalpending".into()
    } else {
        "pending".into()
    }
}

fn log2_n_cell(row: &TimingTableRow, latex: bool, mark: Option<u32>) -> String {
    match (row.status, row.log2_n) {
        (RunStatus::Unsupported, _) => {
            apply_mark(&gap_token(RunStatus::Unsupported, latex), mark, latex)
        }
        (_, Some(log2_n)) => log2_n.to_string(),
        (_, None) => apply_mark(&gap_token(row.status, latex), mark, latex),
    }
}

pub(crate) fn timing_seconds_cell(
    median: Option<f64>,
    std: Option<f64>,
    latex: bool,
) -> Option<String> {
    Some(format_pm(
        &format_seconds(median?),
        std.map(format_spread_seconds),
        latex,
    ))
}

pub(crate) fn timing_millis_cell(
    median: Option<f64>,
    std: Option<f64>,
    latex: bool,
) -> Option<String> {
    Some(format_pm(
        &format_millis(median?),
        std.map(format_spread_millis),
        latex,
    ))
}

/// Markdown version of the timing comparison table.
#[must_use]
pub fn render_markdown_timing_table(rows: &[TimingTableRow]) -> String {
    let notes = unique_gap_notes_timing(rows);
    let mut out = String::from(
        "| Payload | Scheme | Field | log₂ N | Commit (s) | Open (s) | Total (s) | Verify (ms) |\n",
    );
    out.push_str("| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: |\n");
    for row in rows {
        let mark = footnote_index(&notes, row.gap_note.as_ref());
        let _ = writeln!(
            out,
            "| 2^{{{}}} | {} | ${}$ | {} | {} | {} | {} | {} |",
            row.payload_log2,
            scheme_cell(row.scheme, false),
            row.field,
            log2_n_cell(row, false, mark),
            cell(
                row,
                timing_seconds_cell(row.commit_s, row.commit_s_std, false),
                false,
                mark
            ),
            cell(
                row,
                timing_seconds_cell(row.open_s, row.open_s_std, false),
                false,
                mark
            ),
            cell(
                row,
                timing_seconds_cell(row.total_s, row.total_s_std, false),
                false,
                mark
            ),
            cell(
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

/// Markdown version of the resources comparison table.
#[must_use]
pub fn render_markdown_resource_table(rows: &[ResourceTableRow]) -> String {
    let notes = unique_gap_notes_resources(rows);
    let mut out = String::from(
        "| Payload | Scheme | Commitment (B) | Proof (KiB) | Total (KiB) | Peak RSS (GiB) | Prep. (s) | State (GiB) |\n",
    );
    out.push_str("| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for row in rows {
        let mark = footnote_index(&notes, row.gap_note.as_ref());
        let _ = writeln!(
            out,
            "| 2^{{{}}} | {} | {} | {} | {} | {} | {} | {} |",
            row.payload_log2,
            scheme_cell(row.scheme, false),
            resource_cell(
                row,
                row.commitment_bytes.map(|v| v.to_string()),
                false,
                mark
            ),
            resource_cell(row, row.proof_bytes.map(format_kib), false, mark),
            resource_cell(row, total_kib(row).map(|v| format!("{v:.1}")), false, mark),
            resource_cell(row, row.peak_rss_bytes.map(format_gib), false, mark),
            resource_cell(row, row.prep_s.map(format_prep), false, mark),
            resource_cell(row, row.state_bytes.map(format_gib), false, mark),
        );
    }
    out.push_str(&markdown_footnotes(&notes));
    out
}

fn total_kib(row: &ResourceTableRow) -> Option<f64> {
    let commitment = row.commitment_bytes?;
    let proof = row.proof_bytes?;
    Some((commitment + proof) as f64 / 1024.0)
}

/// LaTeX `tabular` matching `tab:eval-lattice-time`.
#[must_use]
pub fn render_latex_timing_table(rows: &[TimingTableRow]) -> String {
    let notes = unique_gap_notes_timing(rows);
    let mut out = String::from(
        "\\begin{table}[H]\n\
         \\centering\n\
         \\caption[Timing comparison with lattice-based PCSs]{Commitment, opening, and\n\
         verification time for Akita and prior lattice-based PCSs on dense\n\
         polynomial openings. Payload is the target value of\n\
         $N\\log_2|\\mathbb F|$.  A dash denotes an unsupported input; numbered\n\
         footnotes give the reason.\n\
         Timing cells are the median of fresh processes after warmup, shown as\n\
         median $\\pm$ sample standard deviation when $n\\ge 2$.\n\
         Scheme names link to the exact git commit that was measured.}\n\
         \\label{tab:eval-lattice-time}\n\
         \\scriptsize\n\
         \\setlength{\\tabcolsep}{4pt}\n\
         \\begin{tabular}{@{}llccrrrr@{}}\n\
         \\toprule\n\
         Payload & Scheme & Field & $\\log_2 N$\n\
         & Commit (s) & Open (s) & Total (s) & Verify (ms) \\\\\n\
         \\midrule\n",
    );
    append_payload_groups(&mut out, rows, |row| {
        let mark = footnote_index(&notes, row.gap_note.as_ref());
        format!(
            "$2^{{{}}}$ & {} & ${}$ & {} & {} & {} & {} & {} \\\\",
            row.payload_log2,
            scheme_cell(row.scheme, true),
            row.field,
            log2_n_cell(row, true, mark),
            cell(
                row,
                timing_seconds_cell(row.commit_s, row.commit_s_std, true),
                true,
                mark
            ),
            cell(
                row,
                timing_seconds_cell(row.open_s, row.open_s_std, true),
                true,
                mark
            ),
            cell(
                row,
                timing_seconds_cell(row.total_s, row.total_s_std, true),
                true,
                mark
            ),
            cell(
                row,
                timing_millis_cell(row.verify_s, row.verify_s_std, true),
                true,
                mark
            ),
        )
    });
    out.push_str("\\bottomrule\n\\end{tabular}\n");
    out.push_str(&latex_footnotes(&notes));
    out.push_str("\\end{table}\n");
    out
}

/// LaTeX `tabular` matching `tab:eval-lattice-resources`.
#[must_use]
pub fn render_latex_resource_table(rows: &[ResourceTableRow]) -> String {
    let notes = unique_gap_notes_resources(rows);
    let mut out = String::from(
        "\\begin{table}[H]\n\
         \\centering\n\
         \\caption[Communication and memory comparison with lattice-based PCSs]{Proof\n\
         communication, prover memory, and reusable preprocessing for the workloads in\n\
         \\Cref{tab:eval-lattice-time}.  Total communication is the commitment plus the\n\
         opening proof.  RoKoKo reports an encoded bit count rather than a materialized\n\
         byte string. Scheme names link to the measured git commit.\n\
         Numbered footnotes mark unsupported inputs.}\n\
         \\label{tab:eval-lattice-resources}\n\
         \\scriptsize\n\
         \\setlength{\\tabcolsep}{4pt}\n\
         \\begin{tabular}{@{}llrrrrrr@{}}\n\
         \\toprule\n\
         Payload & Scheme & Commitment (B) & Proof (KiB) & Total (KiB)\n\
         & Peak RSS (GiB) & Prep. (s) & State (GiB) \\\\\n\
         \\midrule\n",
    );

    let mut by_payload: BTreeMap<u32, Vec<&ResourceTableRow>> = BTreeMap::new();
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
                    "$2^{{{}}}$ & {} & {} & {} & {} & {} & {} & {} \\\\",
                    row.payload_log2,
                    scheme_cell(row.scheme, true),
                    resource_cell(row, row.commitment_bytes.map(|v| v.to_string()), true, mark),
                    resource_cell(row, row.proof_bytes.map(format_kib), true, mark),
                    resource_cell(row, total_kib(row).map(|v| format!("{v:.1}")), true, mark),
                    resource_cell(row, row.peak_rss_bytes.map(format_gib), true, mark),
                    resource_cell(row, row.prep_s.map(format_prep), true, mark),
                    resource_cell(row, row.state_bytes.map(format_gib), true, mark),
                );
            }
        }
    }

    out.push_str("\\bottomrule\n\\end{tabular}\n");
    out.push_str(&latex_footnotes(&notes));
    out.push_str("\\end{table}\n");
    out
}

fn append_payload_groups<F>(out: &mut String, rows: &[TimingTableRow], mut line: F)
where
    F: FnMut(&TimingTableRow) -> String,
{
    let mut by_payload: BTreeMap<u32, Vec<&TimingTableRow>> = BTreeMap::new();
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
        aggregate_resource_rows, aggregate_timing_rows, format_millis, format_seconds,
        render_latex_resource_table, render_latex_timing_table, render_markdown_resource_table,
        render_markdown_timing_table, sample_std, GapNote,
    };
    use crate::lattice::SchemeId;
    use crate::observation::{
        looks_like_oom, LatticeRecord, Provenance, RunStatus, RESULT_SCHEMA_VERSION,
    };
    use std::collections::BTreeMap;

    fn record(
        payload_log2: u32,
        scheme: SchemeId,
        status: RunStatus,
        log2_n: Option<u32>,
        commit_s: Option<f64>,
        open_s: Option<f64>,
        verify_s: Option<f64>,
    ) -> LatticeRecord {
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
        LatticeRecord {
            schema_version: RESULT_SCHEMA_VERSION,
            status,
            status_detail: None,
            scheme,
            implementation_revision: scheme.revision().into(),
            payload_log2,
            log2_n,
            field: match scheme {
                SchemeId::Rokoko => "2^{50}-2687".into(),
                _ => "2^{32}-99".into(),
            },
            native_param: None,
            threads: 1,
            sample: 0,
            warmup: false,
            historical: false,
            timings_ns,
            proof_bytes: Some(61_337),
            commitment_bytes: Some(3072),
            state_bytes: Some(18_563_072),
            peak_rss_bytes: Some(119_000_000),
            provenance: Provenance::test_fixture(),
        }
    }

    #[test]
    fn formats_match_the_paper_examples() {
        assert_eq!(format_seconds(0.159), "0.159");
        assert_eq!(format_seconds(2.07), "2.07");
        assert_eq!(format_seconds(10.7), "10.7");
        assert_eq!(format_seconds(43.8), "43.8");
        assert_eq!(format_millis(0.0419), "41.9");
        assert_eq!(format_millis(0.177), "177");
        assert_eq!(format_millis(0.0116), "11.6");
        let std = sample_std(&[1.0, 2.0, 3.0]).expect("n=3");
        assert!((std - 1.0).abs() < 1e-9);
    }

    #[test]
    fn latex_table_uses_unsupported_and_oom_tokens() {
        let records = vec![
            record(
                27,
                SchemeId::Akita,
                RunStatus::Ok,
                Some(22),
                Some(0.159),
                Some(2.07),
                Some(0.0419),
            ),
            record(
                27,
                SchemeId::Greyhound,
                RunStatus::Ok,
                Some(22),
                Some(0.256),
                Some(0.301),
                Some(0.177),
            ),
            record(
                27,
                SchemeId::Rokoko,
                RunStatus::Unsupported,
                None,
                None,
                None,
                None,
            ),
            record(
                33,
                SchemeId::Greyhound,
                RunStatus::Oom,
                Some(28),
                None,
                None,
                None,
            ),
        ];
        let rows = aggregate_timing_rows(&records);
        let latex = render_latex_timing_table(&rows);
        assert!(latex.contains(r"\href{https://github.com/LayerZero-Labs/akita/commit/f9f7de87bcf230436193dbf6ba5a3bdc077b8f53}{Akita}"));
        assert!(latex.contains("0.159"));
        assert!(latex.contains("2.07"));
        assert!(latex.contains(r"\evalunsupported"));
        assert!(latex.contains(r"\evaloom"));
        assert!(latex.contains(r"\label{tab:eval-lattice-time}"));
        let markdown = render_markdown_timing_table(&rows);
        assert!(markdown.contains("OOM"));
        assert!(markdown.contains("—"));
        assert!(markdown.contains("https://github.com/LayerZero-Labs/akita/commit/"));

        let resources = render_latex_resource_table(&aggregate_resource_rows(&records));
        assert!(resources.contains(r"\label{tab:eval-lattice-resources}"));
        assert!(resources.contains("3072"));
    }

    #[test]
    fn memory_allocation_errors_render_as_oom() {
        let mut failed = record(
            35,
            SchemeId::Rokoko,
            RunStatus::Error,
            Some(30),
            None,
            None,
            None,
        );
        failed.status_detail = Some("memory allocation of 9126805504 bytes failed".into());
        let rows = aggregate_timing_rows(&[failed]);
        let rokoko_35 = rows
            .iter()
            .find(|row| row.payload_log2 == 35 && row.scheme == SchemeId::Rokoko)
            .expect("row");
        assert_eq!(rokoko_35.status, RunStatus::Oom);
    }

    #[test]
    fn unmeasured_supported_cells_render_as_pending() {
        let rows = aggregate_timing_rows(&[]);
        let pr466 = rows
            .iter()
            .find(|row| row.payload_log2 == 31 && row.scheme == SchemeId::AkitaPr466)
            .expect("row");
        assert!(!pr466.measured);
        let markdown = render_markdown_timing_table(std::slice::from_ref(pr466));
        assert!(markdown.contains("pending"));
        let latex = render_latex_timing_table(std::slice::from_ref(pr466));
        assert!(latex.contains(r"\evalpending"));
    }

    #[test]
    fn unsupported_dashes_carry_numbered_footnotes() {
        let mut akita = record(
            27,
            SchemeId::Akita,
            RunStatus::Unsupported,
            Some(22),
            None,
            None,
            None,
        );
        akita.status_detail = Some("pinned Akita fp32 dense catalog has no row for nv=22".into());
        let rokoko = record(
            27,
            SchemeId::Rokoko,
            RunStatus::Unsupported,
            None,
            None,
            None,
            None,
        );
        let rows = aggregate_timing_rows(&[akita, rokoko]);
        let markdown = render_markdown_timing_table(&rows);
        assert!(markdown.contains("—(1)"));
        assert!(markdown.contains("—(2)"));
        assert!(markdown.contains("**(1)** Pinned Akita fp32-dense catalog"));
        assert!(markdown.contains("**(2)** RoKoKo ships only native sets"));
        let latex = render_latex_timing_table(&rows);
        assert!(latex.contains(r"\evalunsupported$^{(1)}$"));
        assert!(latex.contains(r"\evalunsupported$^{(2)}$"));
        assert!(latex.contains(r"$^{(1)}$ Pinned Akita"));
        assert!(latex.contains(r"$^{(2)}$ RoKoKo"));
    }

    #[test]
    fn greyhound_sis_errors_carry_a_footnote() {
        let mut failed = record(
            35,
            SchemeId::Greyhound,
            RunStatus::Error,
            Some(30),
            None,
            None,
            None,
        );
        failed.status_detail = Some(
            "Greyhound failed with status Some(1): ERROR in polcom_reduce(): Inner commitments not secure"
                .into(),
        );
        let rows = aggregate_timing_rows(&[failed.clone()]);
        let greyhound = rows
            .iter()
            .find(|row| row.payload_log2 == 35 && row.scheme == SchemeId::Greyhound)
            .expect("row");
        assert_eq!(greyhound.status, RunStatus::Error);
        assert_eq!(greyhound.gap_note, Some(GapNote::GreyhoundSis));
        assert!(!looks_like_oom(failed.status_detail.as_deref()));
        let markdown = render_markdown_timing_table(std::slice::from_ref(greyhound));
        assert!(markdown.contains("err(1)"));
        assert!(markdown.contains("inner Ajtai commitments SIS-secure"));
        assert!(markdown.contains("not an out-of-memory"));
        let latex = render_latex_timing_table(std::slice::from_ref(greyhound));
        assert!(latex.contains(r"\evalunsupported$^{(1)}$"));
        assert!(latex.contains("inner Ajtai commitments SIS-secure"));
        let resource_rows = aggregate_resource_rows(std::slice::from_ref(&failed));
        let greyhound_res = resource_rows
            .iter()
            .find(|row| row.payload_log2 == 35 && row.scheme == SchemeId::Greyhound)
            .expect("resource row");
        let resources = render_markdown_resource_table(std::slice::from_ref(greyhound_res));
        assert!(resources.contains("err(1)"));
        assert!(resources.contains("inner Ajtai commitments SIS-secure"));
    }
}
