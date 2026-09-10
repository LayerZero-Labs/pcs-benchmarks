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
    /// Recorded implementation revision used by this aggregate.
    pub implementation_revision: Option<String>,
    /// Field label.
    pub field: String,
    /// Native `log2 N`, when the scheme has a matching instance.
    pub log2_n: Option<u32>,
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
    /// Recorded implementation revision used by this aggregate.
    pub implementation_revision: Option<String>,
    /// Cell outcome after aggregating samples.
    pub status: RunStatus,
    /// Median commitment size in bytes.
    pub commitment_bytes: Option<u64>,
    /// Median opening-proof size in bytes.
    pub proof_bytes: Option<u64>,
    /// Median separately transmitted evaluation size in bytes.
    pub evaluation_bytes: Option<u64>,
    /// Median excluded public/verifier context size in bytes.
    pub public_context_bytes: Option<u64>,
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
    /// Akita setup-offload has no generated recursive catalog row for the requested `nv`.
    AkitaOffloadCatalog,
    /// Recursive planner produced a row that does not offload setup (no prefix edge).
    AkitaOffloadNoPrefix,
    /// RoKoKo has no native parameter set for this payload.
    RokokoNative,
    /// Greyhound's SIS parameter search cannot secure the inner commitment.
    GreyhoundSis,
    /// WHIR used unique decoding because capacity/Johnson bounds exceed KoalaBear grind.
    WhirUniqueDecoding,
    /// Plonky3 univariate FRI/STIR packed a degree-\(2^n\) claim into a shorter matrix.
    PackedUnivariate,
    /// Some other recorded unsupported reason.
    Custom(String),
}

/// Aggregate measured non-warmup records into timing rows.
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
            record.payload_log2 == payload_log2 && record.scheme == scheme && !record.warmup
        })
        .collect()
}

fn incomparable_samples(samples: &[&LatticeRecord]) -> bool {
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

fn invalid_success_timings(samples: &[&LatticeRecord]) -> bool {
    samples.iter().any(|record| {
        record.status == RunStatus::Ok
            && ["commit", "open", "verify"]
                .iter()
                .any(|phase| !record.timings_ns.contains_key(*phase))
    })
}

fn invalid_success_resources(samples: &[&LatticeRecord]) -> bool {
    samples.iter().any(|record| {
        record.status == RunStatus::Ok
            && (record.proof_bytes.is_none()
                || record.commitment_bytes.is_none()
                || record.evaluation_bytes.is_none()
                || record.public_context_bytes.is_none())
    })
}

#[allow(clippy::too_many_lines)]
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
            implementation_revision: None,
            field: field.to_owned(),
            log2_n: planned_log2_n,
            status,
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
            gap_note: gap_note(scheme, status, samples),
        };
    }

    if incomparable_samples(samples) {
        return TimingTableRow {
            payload_log2,
            scheme,
            implementation_revision: None,
            field: field.to_owned(),
            log2_n: planned_log2_n,
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
                "Samples with different revisions, parameters, thread counts, or environments were rejected."
                    .into(),
            )),
        };
    }

    if invalid_success_timings(samples) {
        return TimingTableRow {
            payload_log2,
            scheme,
            implementation_revision: Some(samples[0].implementation_revision.clone()),
            field: field.to_owned(),
            log2_n: planned_log2_n,
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

    let ok: Vec<&LatticeRecord> = samples
        .iter()
        .copied()
        .filter(|record| record.status == RunStatus::Ok)
        .collect();
    if ok.len() == samples.len() {
        let commit = phase_seconds(&ok, "commit");
        let open = phase_seconds(&ok, "open");
        let total: Vec<f64> = ok
            .iter()
            .filter_map(|record| {
                let setup = record.timings_ns.get("setup").copied().unwrap_or(0);
                let commit = *record.timings_ns.get("commit")? as f64 / 1e9;
                let open = *record.timings_ns.get("open")? as f64 / 1e9;
                Some(setup as f64 / 1e9 + commit + open)
            })
            .collect();
        return TimingTableRow {
            payload_log2,
            scheme,
            implementation_revision: Some(ok[0].implementation_revision.clone()),
            field: field.to_owned(),
            log2_n: ok[0].log2_n.or(planned_log2_n),
            status: RunStatus::Ok,
            commit_s: median_f64(&commit),
            commit_s_ci95: median_ci95(&commit),
            open_s: median_f64(&open),
            open_s_ci95: median_ci95(&open),
            total_s: median_f64(&total),
            total_s_ci95: median_ci95(&total),
            verify_s: median_f64(&phase_seconds(&ok, "verify")),
            verify_s_ci95: median_ci95(&phase_seconds(&ok, "verify")),
            n_ok: ok.len(),
            measured: true,
            gap_note: None,
        };
    }

    let status = aggregate_gap_status(scheme, samples);
    TimingTableRow {
        payload_log2,
        scheme,
        implementation_revision: Some(samples[0].implementation_revision.clone()),
        field: field.to_owned(),
        log2_n: planned_log2_n,
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
        gap_note: mixed_outcome_note(samples).or_else(|| gap_note(scheme, status, samples)),
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
            implementation_revision: None,
            status,
            commitment_bytes: None,
            proof_bytes: None,
            evaluation_bytes: None,
            public_context_bytes: None,
            peak_rss_bytes: None,
            prep_s: None,
            state_bytes: None,
            measured: false,
            gap_note: gap_note(scheme, status, samples),
        };
    }

    if invalid_success_resources(samples) || incomparable_samples(samples) {
        return ResourceTableRow {
            payload_log2,
            scheme,
            implementation_revision: None,
            status: RunStatus::Error,
            commitment_bytes: None,
            proof_bytes: None,
            evaluation_bytes: None,
            public_context_bytes: None,
            peak_rss_bytes: None,
            prep_s: None,
            state_bytes: None,
            measured: true,
            gap_note: Some(GapNote::Custom(
                "Incomplete or incomparable resource samples were rejected.".into(),
            )),
        };
    }

    let ok: Vec<&LatticeRecord> = samples
        .iter()
        .copied()
        .filter(|record| record.status == RunStatus::Ok)
        .collect();
    if ok.len() == samples.len() {
        let prep = phase_seconds(&ok, "setup");
        let prep_s = median_f64(&prep);
        let state_bytes = median_u64(ok.iter().filter_map(|record| record.state_bytes));
        return ResourceTableRow {
            payload_log2,
            scheme,
            implementation_revision: Some(ok[0].implementation_revision.clone()),
            status: RunStatus::Ok,
            commitment_bytes: median_u64(ok.iter().filter_map(|record| record.commitment_bytes)),
            proof_bytes: median_u64(ok.iter().filter_map(|record| record.proof_bytes)),
            evaluation_bytes: median_u64(ok.iter().filter_map(|record| record.evaluation_bytes)),
            public_context_bytes: median_u64(
                ok.iter().filter_map(|record| record.public_context_bytes),
            ),
            peak_rss_bytes: median_u64(ok.iter().filter_map(|record| record.peak_rss_bytes)),
            prep_s,
            state_bytes,
            measured: true,
            gap_note: None,
        };
    }

    let status = aggregate_gap_status(scheme, samples);
    ResourceTableRow {
        payload_log2,
        scheme,
        implementation_revision: Some(samples[0].implementation_revision.clone()),
        status,
        commitment_bytes: None,
        proof_bytes: None,
        evaluation_bytes: None,
        public_context_bytes: None,
        peak_rss_bytes: None,
        prep_s: None,
        state_bytes: None,
        measured: true,
        gap_note: mixed_outcome_note(samples).or_else(|| gap_note(scheme, status, samples)),
    }
}

fn aggregate_gap_status(scheme: SchemeId, samples: &[&LatticeRecord]) -> RunStatus {
    if samples.iter().any(|record| {
        record.status == RunStatus::Oom || looks_like_oom(record.status_detail.as_deref())
    }) {
        RunStatus::Oom
    } else if (scheme == SchemeId::AkitaOffload
        && samples.iter().any(|record| {
            record
                .status_detail
                .as_deref()
                .is_some_and(|detail| detail.contains("no setup-prefix"))
        }))
        || samples
            .iter()
            .any(|record| record.status == RunStatus::Unsupported)
    {
        RunStatus::Unsupported
    } else {
        RunStatus::Error
    }
}

fn mixed_outcome_note(samples: &[&LatticeRecord]) -> Option<GapNote> {
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

fn gap_note(scheme: SchemeId, status: RunStatus, samples: &[&LatticeRecord]) -> Option<GapNote> {
    if scheme == SchemeId::Greyhound
        && status == RunStatus::Error
        && samples
            .iter()
            .any(|record| looks_like_greyhound_sis(record.status_detail.as_deref()))
    {
        return Some(GapNote::GreyhoundSis);
    }
    if scheme == SchemeId::AkitaOffload
        && samples.iter().any(|record| {
            record
                .status_detail
                .as_deref()
                .is_some_and(|detail| detail.contains("no setup-prefix"))
        })
    {
        return Some(GapNote::AkitaOffloadNoPrefix);
    }
    if status != RunStatus::Unsupported {
        return None;
    }
    match scheme {
        SchemeId::Akita => Some(GapNote::AkitaCatalog),
        SchemeId::AkitaOffload => Some(GapNote::AkitaOffloadCatalog),
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
            Self::AkitaOffloadCatalog => {
                "Akita setup-offload uses the recursive `fp32-dense` catalog. That catalog has no generated row for this $n_v$.".into()
            }
            Self::AkitaOffloadNoPrefix => {
                "The recursive `fp32-dense` planner produced a schedule for this $n_v$ with no setup-prefix edge, so the offload variant would not offload setup.".into()
            }
            Self::RokokoNative => {
                "RoKoKo ships only native sets `p-26`, `p-28`, and `p-30`; no instance matches this payload.".into()
            }
            Self::GreyhoundSis => {
                "Greyhound cannot make the Ajtai commitments SIS-secure at this size under the `l2-quantum128-adps16` policy (ADPS16 quantum core-SVP). This is not an out-of-memory failure.".into()
            }
            Self::WhirUniqueDecoding => {
                "WHIR uses unique decoding at this size so the 128-bit transcript-error target still holds on KoalaBear. Capacity bound and Johnson bound need more than 30 bits of grinding, which the field cannot support. The larger proof is the unique-decoding query schedule.".into()
            }
            Self::PackedUnivariate => {
                "KoalaBear two-adicity is 24, so a rate-$1/2$ univariate cannot be a single degree-$2^{n}$ polynomial when $\\log_2 N>23$. The worker packs the $2^{n}$ coefficients into a trace matrix of height $2^{23}$ and width $2^{n-23}$. That is batched univariate FRI/STIR, not one tall polynomial.".into()
            }
            Self::Custom(detail) => detail.clone(),
        }
    }

    fn latex(&self) -> String {
        match self {
            Self::AkitaCatalog => {
                "Pinned Akita fp32-dense catalog has no production row for $n_v=22$ (payload $2^{27}$) or $n_v=24$ (payload $2^{29}$).".into()
            }
            Self::AkitaOffloadCatalog => {
                "Akita setup-offload uses the recursive \\texttt{fp32-dense} catalog. That catalog has no generated row for this $n_v$.".into()
            }
            Self::AkitaOffloadNoPrefix => {
                "The recursive \\texttt{fp32-dense} planner produced a schedule for this $n_v$ with no setup-prefix edge, so the offload variant would not offload setup.".into()
            }
            Self::RokokoNative => {
                "RoKoKo ships only native sets \\texttt{p-26}, \\texttt{p-28}, and \\texttt{p-30}; no instance matches this payload.".into()
            }
            Self::GreyhoundSis => {
                "Greyhound cannot make the Ajtai commitments SIS-secure at this size under the \\texttt{l2-quantum128-adps16} policy (ADPS16 quantum core-SVP). This is not an out-of-memory failure.".into()
            }
            Self::WhirUniqueDecoding => {
                "WHIR uses unique decoding at this size so the 128-bit transcript-error target still holds on KoalaBear. Capacity bound and Johnson bound need more than 30 bits of grinding, which the field cannot support. The larger proof is the unique-decoding query schedule.".into()
            }
            Self::PackedUnivariate => {
                "KoalaBear two-adicity is 24, so a rate-$1/2$ univariate cannot be a single degree-$2^{n}$ polynomial when $\\log_2 N>23$. The worker packs the $2^{n}$ coefficients into a trace matrix of height $2^{23}$ and width $2^{n-23}$. That is batched univariate FRI/STIR, not one tall polynomial.".into()
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

/// Conservative distribution-free two-sided confidence interval for a median.
///
/// Returns `None` when there are too few observations to attain at least 95%
/// coverage using order statistics.
#[must_use]
pub(crate) fn median_ci95(values: &[f64]) -> Option<(f64, f64)> {
    let n = values.len();
    if n == 0 {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);

    let mut coefficient = 1.0;
    let probability = 0.5f64.powi(i32::try_from(n).ok()?);
    let mut lower_tail = 0.0;
    let mut endpoint = None;
    for k in 0..=(n / 2) {
        if k > 0 {
            coefficient *= (n + 1 - k) as f64 / k as f64;
        }
        let next_tail = lower_tail + coefficient * probability;
        if next_tail > 0.025 {
            break;
        }
        lower_tail = next_tail;
        endpoint = Some(k);
    }
    let endpoint = endpoint?;
    Some((sorted[endpoint], sorted[n - 1 - endpoint]))
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

pub(crate) fn format_ci95(median: &str, interval: Option<(String, String)>, latex: bool) -> String {
    match interval {
        Some((low, high)) if latex => format!("${median}\\;[{low}, {high}]$"),
        Some((low, high)) => format!("{median} [{low}, {high}]"),
        None => median.to_owned(),
    }
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

fn scheme_cell(scheme: SchemeId, implementation_revision: Option<&str>, latex: bool) -> String {
    let Some(revision) = implementation_revision else {
        return if latex {
            scheme.latex_name().to_owned()
        } else {
            scheme.display_name().to_owned()
        };
    };
    let url = format!("{}/commit/{revision}", scheme.source_repo());
    if latex {
        format!(r"\href{{{url}}}{{{}}}", scheme.latex_name())
    } else {
        format!("[{}]({url})", scheme.display_name())
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
        RunStatus::Ok => value.unwrap_or_else(|| unknown_token(latex)),
        other => apply_mark(&gap_token(other, latex), mark, latex),
    }
}

pub(crate) fn unknown_token(latex: bool) -> String {
    if latex {
        r"\textit{unknown}".into()
    } else {
        "unknown".into()
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
    ci95: Option<(f64, f64)>,
    latex: bool,
) -> Option<String> {
    Some(format_ci95(
        &format_seconds(median?),
        ci95.map(|(low, high)| (format_seconds(low), format_seconds(high))),
        latex,
    ))
}

pub(crate) fn timing_millis_cell(
    median: Option<f64>,
    ci95: Option<(f64, f64)>,
    latex: bool,
) -> Option<String> {
    Some(format_ci95(
        &format_millis(median?),
        ci95.map(|(low, high)| (format_millis(low), format_millis(high))),
        latex,
    ))
}

/// Markdown version of the timing comparison table.
#[must_use]
pub fn render_markdown_timing_table(rows: &[TimingTableRow]) -> String {
    let notes = unique_gap_notes_timing(rows);
    let mut out = String::from(
        "| Nominal payload | Scheme | Field | log₂ N | Commit (s) | Open (s) | Cold total (s) | Verify (ms) |\n",
    );
    out.push_str("| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: |\n");
    for row in rows {
        let mark = footnote_index(&notes, row.gap_note.as_ref());
        let _ = writeln!(
            out,
            "| 2^{{{}}} | {} | ${}$ | {} | {} | {} | {} | {} |",
            row.payload_log2,
            scheme_cell(row.scheme, row.implementation_revision.as_deref(), false),
            row.field,
            log2_n_cell(row, false, mark),
            cell(
                row,
                timing_seconds_cell(row.commit_s, row.commit_s_ci95, false),
                false,
                mark
            ),
            cell(
                row,
                timing_seconds_cell(row.open_s, row.open_s_ci95, false),
                false,
                mark
            ),
            cell(
                row,
                timing_seconds_cell(row.total_s, row.total_s_ci95, false),
                false,
                mark
            ),
            cell(
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

/// Markdown version of the resources comparison table.
#[must_use]
pub fn render_markdown_resource_table(rows: &[ResourceTableRow]) -> String {
    let notes = unique_gap_notes_resources(rows);
    let mut out = String::from(
        "| Nominal payload | Scheme | Commitment (B) | Evaluation (B) | Proof (B) | Total sent (B) | Excluded context (B) | Peak RSS (GiB) | Prep. (s) | State (GiB) |\n",
    );
    out.push_str("| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for row in rows {
        let mark = footnote_index(&notes, row.gap_note.as_ref());
        let _ = writeln!(
            out,
            "| 2^{{{}}} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            row.payload_log2,
            scheme_cell(row.scheme, row.implementation_revision.as_deref(), false),
            resource_cell(
                row,
                row.commitment_bytes.map(|v| v.to_string()),
                false,
                mark
            ),
            resource_cell(
                row,
                row.evaluation_bytes.map(|v| v.to_string()),
                false,
                mark
            ),
            resource_cell(row, row.proof_bytes.map(|v| v.to_string()), false, mark),
            resource_cell(row, total_bytes(row).map(|v| v.to_string()), false, mark),
            resource_cell(
                row,
                row.public_context_bytes.map(|v| v.to_string()),
                false,
                mark
            ),
            resource_cell(row, row.peak_rss_bytes.map(format_gib), false, mark),
            resource_cell(row, row.prep_s.map(format_prep), false, mark),
            resource_cell(row, row.state_bytes.map(format_gib), false, mark),
        );
    }
    out.push_str(&markdown_footnotes(&notes));
    out
}

fn total_bytes(row: &ResourceTableRow) -> Option<u64> {
    Some(row.commitment_bytes? + row.evaluation_bytes? + row.proof_bytes?)
}

/// LaTeX `tabular` matching `tab:eval-lattice-time`.
#[must_use]
pub fn render_latex_timing_table(rows: &[TimingTableRow]) -> String {
    let notes = unique_gap_notes_timing(rows);
    let mut out = String::from(
        "\\begin{table}[H]\n\
         \\centering\n\
         \\caption[Timing comparison with lattice-based PCSs]{Commitment, opening, and\n\
         verification time for Akita (direct and setup-offload) and prior lattice-based PCSs on dense\n\
         polynomial openings. Nominal payload is the target value of\n\
         $N\\log_2|\\mathbb F|$.  A dash denotes an unsupported input; numbered\n\
         footnotes give the reason.\n\
         Timing cells are the median of fresh processes after warmup, followed by\n\
         a conservative distribution-free 95\\% confidence interval when the sample count supports one.\n\
         Scheme names link to the exact git commit that was measured.}\n\
         \\label{tab:eval-lattice-time}\n\
         \\scriptsize\n\
         \\setlength{\\tabcolsep}{4pt}\n\
         \\begin{tabular}{@{}llccrrrr@{}}\n\
         \\toprule\n\
         Nominal payload & Scheme & Field & $\\log_2 N$\n\
         & Commit (s) & Open (s) & Cold total (s) & Verify (ms) \\\\\n\
         \\midrule\n",
    );
    append_payload_groups(&mut out, rows, |row| {
        let mark = footnote_index(&notes, row.gap_note.as_ref());
        format!(
            "$2^{{{}}}$ & {} & ${}$ & {} & {} & {} & {} & {} \\\\",
            row.payload_log2,
            scheme_cell(row.scheme, row.implementation_revision.as_deref(), true),
            row.field,
            log2_n_cell(row, true, mark),
            cell(
                row,
                timing_seconds_cell(row.commit_s, row.commit_s_ci95, true),
                true,
                mark
            ),
            cell(
                row,
                timing_seconds_cell(row.open_s, row.open_s_ci95, true),
                true,
                mark
            ),
            cell(
                row,
                timing_seconds_cell(row.total_s, row.total_s_ci95, true),
                true,
                mark
            ),
            cell(
                row,
                timing_millis_cell(row.verify_s, row.verify_s_ci95, true),
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
         \\Cref{tab:eval-lattice-time}. Total sent is commitment plus separately transmitted\n\
         evaluation plus opening proof; excluded verifier context is shown separately.\n\
         RoKoKo reports an encoded bit count rather than a materialized\n\
         byte string. Scheme names link to the measured git commit.\n\
         Numbered footnotes mark unsupported inputs.}\n\
         \\label{tab:eval-lattice-resources}\n\
         \\scriptsize\n\
         \\setlength{\\tabcolsep}{4pt}\n\
         \\begin{tabular}{@{}llrrrrrrrr@{}}\n\
         \\toprule\n\
         Nominal payload & Scheme & Commitment (B) & Evaluation (B) & Proof (B) & Total sent (B)\n\
         & Excl. context (B) & Peak RSS (GiB) & Prep. (s) & State (GiB) \\\\\n\
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
                    "$2^{{{}}}$ & {} & {} & {} & {} & {} & {} & {} & {} & {} \\\\",
                    row.payload_log2,
                    scheme_cell(row.scheme, row.implementation_revision.as_deref(), true),
                    resource_cell(row, row.commitment_bytes.map(|v| v.to_string()), true, mark),
                    resource_cell(row, row.evaluation_bytes.map(|v| v.to_string()), true, mark),
                    resource_cell(row, row.proof_bytes.map(|v| v.to_string()), true, mark),
                    resource_cell(row, total_bytes(row).map(|v| v.to_string()), true, mark),
                    resource_cell(
                        row,
                        row.public_context_bytes.map(|v| v.to_string()),
                        true,
                        mark
                    ),
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
        aggregate_resource_rows, aggregate_timing_rows, format_millis, format_seconds, median_ci95,
        render_latex_resource_table, render_latex_timing_table, render_markdown_resource_table,
        render_markdown_timing_table, GapNote,
    };
    use crate::lattice::SchemeId;
    use crate::observation::{looks_like_oom, LatticeRecord, Provenance, RunStatus};
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
            timings_ns,
            proof_bytes: Some(61_337),
            commitment_bytes: Some(3072),
            evaluation_bytes: Some(8),
            public_context_bytes: Some(0),
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
        assert_eq!(median_ci95(&[1.0, 2.0, 3.0]), None);
        assert_eq!(
            median_ci95(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]),
            Some((1.0, 6.0))
        );
        assert_eq!(
            median_ci95(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]),
            Some((2.0, 9.0))
        );
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
        assert!(latex.contains(r"\href{https://github.com/LayerZero-Labs/akita/commit/d1b224d809c7edc357b0dbab0f607e19b475910b}{Akita}"));
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
        assert!(resources.contains("Proof (B)"));
        assert!(resources.contains("Evaluation (B)"));
        assert!(resources.contains("Excl. context (B)"));
        assert!(resources.contains("Total sent (B)"));
        assert!(!resources.contains("KiB"));
        assert!(resources.contains("3072"));
        assert!(resources.contains("61337"));
        let resources_md = render_markdown_resource_table(&aggregate_resource_rows(&records));
        assert!(resources_md.contains("Proof (B)"));
        assert!(resources_md.contains("Evaluation (B)"));
        assert!(resources_md.contains("Excluded context (B)"));
        assert!(resources_md.contains("| 61337 |"));
        assert!(resources_md.contains("| 64417 |"));
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
        assert_eq!(rows.len(), crate::LATTICE_CELL_COUNT);
        let akita = rows
            .iter()
            .find(|row| row.payload_log2 == 31 && row.scheme == SchemeId::Akita)
            .expect("row");
        assert!(!akita.measured);
        let offload = rows
            .iter()
            .find(|row| row.payload_log2 == 31 && row.scheme == SchemeId::AkitaOffload)
            .expect("offload row");
        assert!(!offload.measured);
        let markdown = render_markdown_timing_table(std::slice::from_ref(akita));
        assert!(markdown.contains("pending"));
        let latex = render_latex_timing_table(std::slice::from_ref(akita));
        assert!(latex.contains(r"\evalpending"));
    }

    #[test]
    fn offload_catalog_gaps_carry_a_footnote() {
        let mut offload = record(
            31,
            SchemeId::AkitaOffload,
            RunStatus::Unsupported,
            Some(26),
            None,
            None,
            None,
        );
        offload.status_detail =
            Some("pinned Akita fp32-dense-recursive catalog has no row for nv=26".into());
        let rows = aggregate_timing_rows(&[offload]);
        let row = rows
            .iter()
            .find(|row| row.payload_log2 == 31 && row.scheme == SchemeId::AkitaOffload)
            .expect("row");
        assert_eq!(row.gap_note, Some(GapNote::AkitaOffloadCatalog));
        let markdown = render_markdown_timing_table(std::slice::from_ref(row));
        assert!(markdown.contains("—(1)"));
        assert!(markdown.contains("recursive `fp32-dense` catalog"));
        let latex = render_latex_timing_table(std::slice::from_ref(row));
        assert!(latex.contains(r"\evalunsupported$^{(1)}$"));
        assert!(latex.contains("recursive \\texttt{fp32-dense} catalog"));
    }

    #[test]
    fn offload_without_setup_prefix_carries_a_footnote() {
        let mut offload = record(
            27,
            SchemeId::AkitaOffload,
            RunStatus::Error,
            Some(22),
            None,
            None,
            None,
        );
        offload.status_detail = Some(
            "pinned Akita fp32-dense-recursive catalog row for nv=22 has no setup-prefix edges"
                .into(),
        );
        let rows = aggregate_timing_rows(&[offload]);
        let row = rows
            .iter()
            .find(|row| row.payload_log2 == 27 && row.scheme == SchemeId::AkitaOffload)
            .expect("row");
        assert_eq!(row.status, RunStatus::Unsupported);
        assert_eq!(row.gap_note, Some(GapNote::AkitaOffloadNoPrefix));
        let markdown = render_markdown_timing_table(std::slice::from_ref(row));
        assert!(markdown.contains("—(1)"));
        assert!(markdown.contains("no setup-prefix edge"));
        let latex = render_latex_timing_table(std::slice::from_ref(row));
        assert!(latex.contains(r"\evalunsupported$^{(1)}$"));
        assert!(latex.contains("no setup-prefix edge"));
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
        assert!(markdown.contains("Ajtai commitments SIS-secure"));
        assert!(markdown.contains("l2-quantum128-adps16"));
        assert!(markdown.contains("not an out-of-memory"));
        let latex = render_latex_timing_table(std::slice::from_ref(greyhound));
        assert!(latex.contains(r"\evalunsupported$^{(1)}$"));
        assert!(latex.contains("Ajtai commitments SIS-secure"));
        let resource_rows = aggregate_resource_rows(std::slice::from_ref(&failed));
        let greyhound_res = resource_rows
            .iter()
            .find(|row| row.payload_log2 == 35 && row.scheme == SchemeId::Greyhound)
            .expect("resource row");
        let resources = render_markdown_resource_table(std::slice::from_ref(greyhound_res));
        assert!(resources.contains("err(1)"));
        assert!(resources.contains("Ajtai commitments SIS-secure"));
    }

    #[test]
    fn partial_success_does_not_hide_oom_samples() {
        let ok = record(
            27,
            SchemeId::Akita,
            RunStatus::Ok,
            Some(22),
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
        let rows = aggregate_timing_rows(&records);
        let row = rows
            .iter()
            .find(|row| row.payload_log2 == 27 && row.scheme == SchemeId::Akita)
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
    fn mixed_thread_counts_and_environments_are_rejected() {
        let one = record(
            27,
            SchemeId::Akita,
            RunStatus::Ok,
            Some(22),
            Some(1.0),
            Some(2.0),
            Some(0.1),
        );
        let mut two = one.clone();
        two.sample = 1;
        two.threads = 8;
        two.provenance.threads = 8;

        let rows = aggregate_timing_rows(&[one, two]);
        let row = rows
            .iter()
            .find(|row| row.payload_log2 == 27 && row.scheme == SchemeId::Akita)
            .expect("row");
        assert_eq!(row.status, RunStatus::Error);
        assert_eq!(row.n_ok, 0);
        assert!(matches!(
            row.gap_note,
            Some(GapNote::Custom(ref note)) if note.contains("thread counts")
        ));
    }

    #[test]
    fn ten_samples_report_a_distribution_free_median_interval() {
        let records: Vec<_> = (1..=10)
            .map(|seconds| {
                let mut sample = record(
                    27,
                    SchemeId::Akita,
                    RunStatus::Ok,
                    Some(22),
                    Some(f64::from(seconds)),
                    Some(1.0),
                    Some(0.1),
                );
                sample.sample = seconds as u32 - 1;
                sample
            })
            .collect();
        let rows = aggregate_timing_rows(&records);
        let row = rows
            .iter()
            .find(|row| row.payload_log2 == 27 && row.scheme == SchemeId::Akita)
            .expect("row");
        assert_eq!(row.commit_s, Some(5.5));
        assert_eq!(row.commit_s_ci95, Some((2.0, 9.0)));
        let markdown = render_markdown_timing_table(std::slice::from_ref(row));
        assert!(markdown.contains("5.50 [2.00, 9.00]"));
    }
}
