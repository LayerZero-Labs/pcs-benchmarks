//! Paper-style hash evaluation report (prose + both tables).

use crate::hash_table::{
    aggregate_hash_resource_rows, aggregate_hash_timing_rows, render_latex_hash_resource_table,
    render_latex_hash_timing_table, render_markdown_hash_resource_table,
    render_markdown_hash_timing_table,
};
use crate::observation::{HashRecord, Provenance};
use std::fmt::Write as _;

/// Markdown report matching the paper's hash-eval write-up.
#[must_use]
pub fn render_markdown_hash_eval_report(records: &[HashRecord]) -> String {
    let provenance = first_provenance(records);
    let homogeneous_machine = records_share_machine(records, &provenance);
    let timing = aggregate_hash_timing_rows(records);
    let resources = aggregate_hash_resource_rows(records);
    format!(
        "{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n",
        markdown_prose(&provenance, homogeneous_machine),
        markdown_security_table(),
        render_markdown_hash_timing_table(&timing),
        render_markdown_hash_resource_table(&resources),
        markdown_pins(records),
        harness_provenance(records, false),
        markdown_reproduction(&provenance)
    )
}

/// LaTeX report matching `tab:eval-hash-time` and `tab:eval-hash-resources`.
#[must_use]
pub fn render_latex_hash_eval_report(records: &[HashRecord]) -> String {
    let provenance = first_provenance(records);
    let homogeneous_machine = records_share_machine(records, &provenance);
    let timing = aggregate_hash_timing_rows(records);
    let resources = aggregate_hash_resource_rows(records);
    format!(
        "{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n",
        latex_prose(&provenance, homogeneous_machine),
        latex_security_table(),
        render_latex_hash_timing_table(&timing),
        render_latex_hash_resource_table(&resources),
        latex_pins(records),
        harness_provenance(records, true),
        latex_reproduction(&provenance)
    )
}

// Exact source-state fingerprints audited in docs/provenance.md.
const RESULTS_ONLY_REVISIONS: [&str; 3] = [
    "37a528a43333cd54d9b8a9a6fa0382676c636d98+dirty.1d0750363ab2",
    "946c0a78c50ba9800a1c452ce0c27b73d0cce24a+dirty.e6c3243c86f9",
    "a5a6962520e7e174a0bd5cb93795c33ead2934cf+dirty.2fe6b7a25d43",
];

fn displayed_harness_revision(revision: &str) -> &str {
    if RESULTS_ONLY_REVISIONS.contains(&revision) {
        revision
            .split_once('+')
            .map_or(revision, |(commit, _)| commit)
    } else {
        revision
    }
}

fn harness_provenance(records: &[HashRecord], latex: bool) -> String {
    let revisions: std::collections::BTreeSet<_> = records
        .iter()
        .map(|record| record.provenance.harness_revision.as_str())
        .collect();
    let mut text = if latex {
        "\\paragraph{Harness provenance}\n".to_owned()
    } else {
        "### Harness provenance\n\n".to_owned()
    };
    text.push_str("Harness revisions represented in these records:\n\n");
    for revision in revisions {
        let display = displayed_harness_revision(revision);
        if latex {
            let _ = writeln!(text, "\\texttt{{{}}}\\par", escape_tex(display));
        } else if display.len() == 40 && display.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            let _ = writeln!(
                text,
                "- [{display}](https://github.com/LayerZero-Labs/pcs-benchmarks/commit/{display})"
            );
        } else {
            let _ = writeln!(text, "- `{display}`");
        }
    }
    if records
        .iter()
        .any(|record| RESULTS_ONLY_REVISIONS.contains(&record.provenance.harness_revision.as_str()))
    {
        text.push_str("\nThree exact historical fingerprints were audited as result-file-only changes; their displayed revisions use the corresponding commits. Raw JSONL retains the original fingerprints. Unknown dirty revisions are preserved. See docs/provenance.md for the audit scope.\n");
    }
    text
}

fn first_provenance(records: &[HashRecord]) -> Provenance {
    records
        .iter()
        .filter(|record| !record.warmup)
        .max_by_key(|record| {
            let provenance = &record.provenance;
            u32::from(provenance.avx512)
                + u32::from(provenance.logical_cpus > 0)
                + u32::from(provenance.memory_bytes.is_some())
                + u32::from(provenance.memory_limit_bytes.is_some())
        })
        .map_or_else(Provenance::test_fixture, |record| record.provenance.clone())
}

fn records_share_machine(records: &[HashRecord], expected: &Provenance) -> bool {
    records
        .iter()
        .filter(|record| !record.warmup)
        .all(|record| {
            let candidate = &record.provenance;
            candidate.target == expected.target
                && candidate.cpu_model == expected.cpu_model
                && candidate.machine_id_hash == expected.machine_id_hash
                && candidate.logical_cpus == expected.logical_cpus
                && candidate.memory_bytes == expected.memory_bytes
        })
}

fn machine_sentence(provenance: &Provenance, latex: bool, homogeneous_machine: bool) -> String {
    if !homogeneous_machine {
        return "The input records contain multiple machine descriptions; this is not a comparable reporting cohort and must be split before publication.".into();
    }
    let avx = if provenance.avx512 {
        "The CPU advertised AVX-512F, but the executed instruction stream was not independently traced."
    } else {
        "This host does not advertise AVX-512F; headline numbers should be gathered on the same x86_64 AVX-512 machine as the lattice table."
    };
    let mem = provenance.memory_bytes.map_or_else(String::new, |bytes| {
        format!(", {:.0}~GiB RAM", bytes as f64 / 1_073_741_824.0)
    });
    let cpus = if provenance.logical_cpus == 0 {
        String::new()
    } else {
        format!(", {} logical CPUs", provenance.logical_cpus)
    };
    let policy = provenance
        .cpu_governor
        .as_deref()
        .map_or_else(String::new, |governor| {
            let driver = provenance
                .cpu_scaling_driver
                .as_deref()
                .unwrap_or("unknown");
            let preference = provenance
                .cpu_energy_preference
                .as_deref()
                .unwrap_or("unknown");
            format!(" CPU policy: driver {driver}, governor {governor}, preference {preference}.")
        });
    if latex {
        format!(
            "Measurements were collected on a single {} ({}{cpus}{mem}). {avx}{}",
            escape_tex(&provenance.cpu_model),
            escape_tex(&provenance.target),
            escape_tex(&policy),
        )
    } else {
        format!(
            "Measurements were collected on a single {} ({}{cpus}{mem}). {avx}{policy}",
            provenance.cpu_model, provenance.target
        )
    }
}

fn markdown_prose(provenance: &Provenance, homogeneous_machine: bool) -> String {
    format!(
        "{}\n\n\
         Our second experiment compares Akita with other high-performance hash-based PCSs\n\
         on the same nominal dense payload ladder ($2^{{27}}$ through $2^{{35}}$ bits).\n\
         This is a measured-configuration survey, not an equivalent-security PCS ranking.\n\
         Nominal payload is field-capacity accounting, not a claim about sampled input entropy.\n\
         The table below records the accepted native profile and security accounting for every scheme.\n\
         KoalaBear univariate FRI/STIR pack into a\n\
         $2^{{23}}\\times 2^{{n-23}}$ matrix when $\\log_2 N>23$ (two-adicity 24 at rate $1/2$).\n\
         Timing cells report the median and, when supported by the sample count, a\n\
         conservative distribution-free 95% confidence interval at **1 and 8 threads**. Scheme names link to the exact git commit\n\
         that was measured. Unmeasured roster cells are pending.\n\n\
         The timing comparison separates commitment, opening, and verification, while the\n\
         cold total includes setup plus commitment and opening. Point-dependent claim and\n\
         transcript work supplied to proving is included in opening.\n\
         The resources table reports communication, memory (1-thread and 8-thread peak RSS),\n\
         and preprocessing. An OOM entry {oom}.",
        machine_sentence(provenance, false, homogeneous_machine),
        oom = oom_clause(provenance, false),
    )
}

fn latex_prose(provenance: &Provenance, homogeneous_machine: bool) -> String {
    format!(
        "{}\n\n\
         Our second experiment compares Akita with other high-performance hash-based PCSs\n\
         on the same nominal dense payload ladder ($2^{{27}}$ through $2^{{35}}$ bits).\n\
         This is a measured-configuration survey, not an equivalent-security PCS ranking.\n\
         Nominal payload is field-capacity accounting, not a claim about sampled input entropy.\n\
         The table below records the accepted native profile and security accounting for every scheme.\n\
         KoalaBear univariate FRI/STIR pack into a $2^{{23}}\\times 2^{{n-23}}$ matrix when $\\log_2 N>23$. Timing cells\n\
         report the median and, when supported by the sample count, a conservative distribution-free 95\\% confidence interval,\n\
         at 1 and 8 threads. Scheme names are hyperlinks to the exact git commit that was\n\
         measured. Unmeasured roster cells are \\evalpending{{}}.\n\n\
         The timing comparison in \\Cref{{tab:eval-hash-time}} separates commitment,\n\
         opening, and verification; cold total includes setup, commitment, and opening.\n\
         Point-dependent claim and transcript work supplied to proving is included in opening.\n\
         \\Cref{{tab:eval-hash-resources}} reports\n\
         communication, memory, and preprocessing. An \\evaloom{{}} entry {oom}.",
        machine_sentence(provenance, true, homogeneous_machine),
        oom = oom_clause(provenance, true),
    )
}

fn markdown_security_table() -> &'static str {
    "### Security and accepted profiles\n\n\
| Scheme | Accepted profile | Security accounting |\n\
| --- | --- | --- |\n\
| Akita | Planner-selected direct/offloaded schedules at each native prime | 128-bit Module-SIS and 128-bit classical-ROM transcript target |\n\
| Plonky2 FRI | Standard recursion: rate 1/8, 28 queries, 16 work bits | Approximately 100-bit conjectural FRI estimate |\n\
| Plonky3 FRI | Upstream new_benchmark: rate 1/2, 100 queries, 16 query-PoW bits | 113.744-bit conjectural random-words estimate |\n\
| Plonky3 STIR | Upstream PCS benchmark: rate 1/2, fold 4 throughout, at most 20 work bits per phase | 100-bit aggregate capacity/MCA target |\n\
| Plonky3 WHIR | Closest feasible upstream PCS benchmark profile; capacity through $\\log_2N=26$, unique decoding after | 128-bit round-by-round target under the pinned model |\n\
| Binius64 BaseFold | Product default: rate 1/2, 232 queries, SHA-256 | 96-bit unique-decoding query target |\n\
| Flock Ligerito | Default Fast: rate 1/2, Johnson, two OOD checks, SHA-256 | 128-bit round-by-round target |\n\
| WorldFnd WHIR | CLI defaults: rate 1/2, fold 4, Johnson, BLAKE3 | 128-bit round-by-round target |\n\
| SP1 BaseFold | Product default: rate 1/4, 124 queries, 16 work bits, stacking height 21 | 100-bit unique-decoding query target |"
}

fn latex_security_table() -> &'static str {
    "\\begin{table}[t]\n\
\\centering\n\
\\caption{Security accounting and accepted native profiles. RBR denotes round-by-round soundness; UDR denotes unique decoding.}\n\
\\begin{tabularx}{\\linewidth}{@{}lXX@{}}\n\
\\toprule\n\
Scheme & Accepted profile & Security accounting \\\\\n\
\\midrule\n\
Akita & Planner-selected schedules at each native prime & 128-bit Module-SIS and classical-ROM target \\\\\n\
Plonky2 FRI & Rate $1/8$, 28 queries, 16 work bits & $\\sim$100-bit conjectural FRI \\\\\n\
Plonky3 FRI & Rate $1/2$, 100 queries, 16 query-PoW bits & 113.744-bit conjectural random-words \\\\\n\
Plonky3 STIR & Rate $1/2$, fold 4 throughout, at most 20 work bits per phase & 100-bit aggregate capacity/MCA \\\\\n\
Plonky3 WHIR & Capacity through $\\log_2N=26$, then unique decoding & 128-bit RBR under pinned model \\\\\n\
Binius64 & Rate $1/2$, 232 queries, SHA-256 & 96-bit UDR query target \\\\\n\
Flock & Default \\texttt{Fast}, SHA-256 & 128-bit RBR \\\\\n\
WorldFnd & Rate $1/2$, fold 4, Johnson, BLAKE3 & 128-bit RBR \\\\\n\
SP1 & Rate $1/4$, 124 queries, 16 work bits, height 21 & 100-bit UDR query target \\\\\n\
\\bottomrule\n\
\\end{tabularx}\n\
\\end{table}"
}

fn markdown_pins(records: &[HashRecord]) -> String {
    let mut out = String::from("### Measured commits\n\n");
    let pins: std::collections::BTreeSet<_> = records
        .iter()
        .map(|record| (record.scheme, record.implementation_revision.as_str()))
        .collect();
    for (scheme, revision) in pins {
        let short = revision.get(..8).unwrap_or(revision);
        let url = format!("{}/commit/{revision}", scheme.source_repo());
        let _ = writeln!(out, "- {} [`{}`]({})", scheme.display_name(), short, url);
    }
    out
}

fn latex_pins(records: &[HashRecord]) -> String {
    let mut out = String::from(
        "\\medskip\n\\noindent\\textbf{Measured commits.}\n\\begin{itemize}\\setlength{\\itemsep}{0pt}\n",
    );
    let pins: std::collections::BTreeSet<_> = records
        .iter()
        .map(|record| (record.scheme, record.implementation_revision.as_str()))
        .collect();
    for (scheme, revision) in pins {
        let short = revision.get(..8).unwrap_or(revision);
        let url = format!("{}/commit/{revision}", scheme.source_repo());
        let _ = writeln!(
            out,
            "\\item {}: \\href{{{}}}{{\\texttt{{{}}}}}",
            scheme.latex_name(),
            url,
            short
        );
    }
    out.push_str("\\end{itemize}\n");
    out
}

fn oom_clause(provenance: &Provenance, latex: bool) -> String {
    match provenance.resolved_memory_limit_bytes() {
        Some(limit) => {
            let gib = format_limit_gib(limit);
            if latex {
                format!("is a confirmed allocation failure under the {gib}~GiB virtual-address-space ceiling")
            } else {
                format!("is a confirmed allocation failure under the {gib} GiB virtual-address-space ceiling")
            }
        }
        None => "is a confirmed allocation failure (the virtual-address-space ceiling is unknown)"
            .into(),
    }
}

fn format_limit_gib(bytes: u64) -> String {
    format!("{:.0}", bytes as f64 / 1_073_741_824.0)
}

fn markdown_reproduction(provenance: &Provenance) -> String {
    let mut out = String::from("### Reproduction template\n\n");
    out.push_str(&with_memory_limit(REPRODUCTION_PROSE_MARKDOWN, provenance));
    out.push_str("\n\n```bash\n");
    out.push_str(&with_memory_limit(REPRODUCTION_COMMANDS, provenance));
    out.push_str("```\n\n");
    out.push_str(&with_memory_limit(SANITY_PROSE_MARKDOWN, provenance));
    out.push_str("\n\n```bash\n");
    out.push_str(&with_memory_limit(SANITY_COMMANDS, provenance));
    out.push_str("```\n");
    out
}

fn latex_reproduction(provenance: &Provenance) -> String {
    format!(
        "\\medskip\n\\noindent\\textbf{{Reproduction template.}}\n{}\n\
         \\begin{{verbatim}}\n{}\n\\end{{verbatim}}\n\
         {}\n\
         \\begin{{verbatim}}\n{}\n\\end{{verbatim}}\n",
        with_memory_limit(REPRODUCTION_PROSE_LATEX, provenance),
        with_memory_limit(REPRODUCTION_COMMANDS, provenance).trim_end(),
        SANITY_PROSE_LATEX,
        with_memory_limit(SANITY_COMMANDS, provenance).trim_end(),
    )
}

fn with_memory_limit(text: &str, provenance: &Provenance) -> String {
    let limit = provenance.resolved_memory_limit_bytes().unwrap_or(0);
    let rustflags = if provenance.rustflags.is_empty() {
        "(none)"
    } else {
        &provenance.rustflags
    };
    text.replace("{MEMORY_LIMIT_GIB}", &format_limit_gib(limit))
        .replace("{MEMORY_LIMIT_BYTES}", &limit.to_string())
        .replace("{RUSTFLAGS}", rustflags)
        .replace(
            "{RUN_COMMAND}",
            provenance.run_command.as_deref().unwrap_or("unknown"),
        )
}

const REPRODUCTION_PROSE_MARKDOWN: &str = "\
Machine, ISA, compiler, executable, lockfile, command, and timestamp provenance
for this dataset are recorded with each observation. The infrastructure
toolchain pin is Rust **1.95** (`rust-toolchain.toml`).
Recorded runner command: `{RUN_COMMAND}`. The commands below are a template,
not reconstructed provenance.
Workers are built from checked-in lockfiles before sampling. Every timed
execution is a fresh process wrapped in `scripts/with-memlimit.sh` with
a {MEMORY_LIMIT_GIB}~GiB virtual-address-space ceiling (`ulimit -v`, numerically
90% of host RAM). Raw records identify
warmup and measured processes separately; warmup rows are stored with
`warmup: true` and excluded from the aggregate. Workload seeds and the
`vary`/`fixed` seed mode are recorded per observation. Recorded worker flags
for this dataset: `{RUSTFLAGS}`. Isolated Cargo trees under `benchmarks/`
fetch the pinned git revisions (Plonky3, SP1, plonky2, Binius64, Flock,
WorldFnd/WHIR) so they do not unify with the lattice workspace. Cargo fetches
those revisions on first build.

Non-interactive shells may not put Cargo on `PATH`; `source ~/.cargo/env`
is required in that case. `CARGO_NET_GIT_FETCH_WITH_CLI=true` avoids libgit2 auth
failures when fetching the pinned git dependencies.
Published numbers live in `results/hash-x86_64/`.";

const REPRODUCTION_PROSE_LATEX: &str = "\
Machine, ISA, compiler, executable, lockfile, command, and timestamp provenance
for this dataset are recorded with each observation. The infrastructure
toolchain pin is Rust 1.95 (\\texttt{rust-toolchain.toml}).
The exact runner command is recorded in the JSONL; the commands below are a
template, not reconstructed provenance.
Workers are built from checked-in lockfiles before sampling. Every timed worker
is a fresh process wrapped in \\texttt{scripts/with-memlimit.sh} under a
{MEMORY_LIMIT_GIB}~GiB virtual-address-space ceiling (\\texttt{ulimit -v}).
Raw records
identify warmup and measured processes separately. Recorded RUSTFLAGS:
\\texttt{{{RUSTFLAGS}}}. Checked-in numbers live in
\\texttt{results/hash-x86\\_64/}.";

const REPRODUCTION_COMMANDS: &str = "\
# On an AVX-512 Linux x86_64 host
source \"$HOME/.cargo/env\"   # if cargo is not on PATH
cd /path/to/akita-benchmark

export CARGO_NET_GIT_FETCH_WITH_CLI=true
export RUSTFLAGS=\"-C target-cpu=native\"

./scripts/fetch-vendors.sh --akita   # Akita pin + nv=22/24 + fp64/fp128 + offload catalogs

# Full 140-cell matrix (14 schemes × 5 payloads × {1,8} threads)
./scripts/hash-eval.sh run --out results/hash-x86_64

# Rebuild Markdown + LaTeX from the JSONL already in that directory
cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval compare \\
  results/hash-x86_64 --out-dir results/hash-x86_64
";

const SANITY_PROSE_MARKDOWN: &str = "\
**Sanity-check the harness before trusting a full run.** `hash-eval matrix`
prints the 140-cell plan. A single supported cell should verify and emit JSON
with `status: ok`. Each sample the runner launches is equivalent to the worker
commands below (still under the 90%-of-RAM cap).";

const SANITY_PROSE_LATEX: &str = "\
\\noindent Sanity-check the harness before a full run.
\\texttt{hash-eval matrix} prints the 140-cell plan.
A single supported cell should verify and emit JSON with \\texttt{status: ok}.";

const SANITY_COMMANDS: &str = "\
export RUSTFLAGS=\"-C target-cpu=native\"

cargo test -p pcs-bench-core -p pcs-bench-runner --locked
cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval matrix

# One measured sample of a supported cell (payload 2^31, log2 N = 26, 1 thread)
./scripts/hash-eval.sh run --scheme akita --payload 31 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme akita-fp64 --payload 31 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme akita-fp128 --payload 31 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme akita-offload --payload 31 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme akita-fp64-offload --payload 31 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme whir --payload 31 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme basefold --payload 31 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme plonky2-fri --payload 27 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme plonky3-fri --payload 27 --threads 1 --runs 1 --warmups 0
./scripts/hash-eval.sh run --scheme flock --payload 27 --threads 1 --runs 1 --warmups 0
";

fn escape_tex(text: &str) -> String {
    text.replace('\\', r"\textbackslash{}")
        .replace('_', r"\_")
        .replace('%', r"\%")
        .replace('#', r"\#")
        .replace('&', r"\&")
}

#[cfg(test)]
mod tests {
    use super::render_markdown_hash_eval_report;
    use super::{displayed_harness_revision, RESULTS_ONLY_REVISIONS};
    use crate::hash::HashSchemeId;
    use crate::lattice::worker_memory_limit_bytes;
    use crate::observation::{HashRecord, Provenance, RunStatus};
    use std::collections::BTreeMap;

    #[test]
    fn only_audited_results_only_revisions_are_normalized() {
        for revision in RESULTS_ONLY_REVISIONS {
            assert_eq!(displayed_harness_revision(revision), &revision[..40]);
        }
        let unknown = "37a528a43333cd54d9b8a9a6fa0382676c636d98+dirty.unknown";
        assert_eq!(displayed_harness_revision(unknown), unknown);
        assert_eq!(displayed_harness_revision("clean"), "clean");
    }

    #[test]
    fn hash_report_names_the_machine_and_omits_hostnames() {
        let mut timings_ns = BTreeMap::new();
        timings_ns.insert("commit".into(), 159_000_000);
        timings_ns.insert("open".into(), 2_070_000_000);
        timings_ns.insert("verify".into(), 41_900_000);
        let record = HashRecord {
            status: RunStatus::Ok,
            status_detail: None,
            scheme: HashSchemeId::Akita,
            implementation_revision: HashSchemeId::Akita.revision().into(),
            payload_log2: 27,
            log2_n: Some(22),
            field: "2^{32}-99".into(),
            native_param: Some("fp32-dense".into()),
            threads: 1,
            sample: 0,
            warmup: false,
            timings_ns,
            proof_bytes: Some(61_337),
            commitment_bytes: Some(343),
            evaluation_bytes: Some(16),
            public_context_bytes: Some(215),
            state_bytes: Some(18_563_072),
            peak_rss_bytes: Some(119_000_000),
            provenance: Provenance {
                cpu_model: "AMD Ryzen 9 9950X 16-Core Processor".into(),
                target: "Linux x86_64".into(),
                avx512: true,
                isa_notes: "AVX-512F activated (-C target-cpu=native)".into(),
                logical_cpus: 32,
                memory_bytes: Some(121u64 * 1024 * 1024 * 1024),
                rustflags: "-C target-cpu=native".into(),
                ..Provenance::test_fixture()
            },
        };
        let report = render_markdown_hash_eval_report(std::slice::from_ref(&record));
        assert!(report.contains("AMD Ryzen 9 9950X"));
        assert!(report.contains("AVX-512F"));
        assert!(report.contains("native"));
        assert!(report.contains("128-bit"));
        assert!(report.contains("unique decoding"));
        assert!(report.contains("WHIR"));
        assert!(report.contains("BaseFold"));
        assert!(report.contains("113.744-bit conjectural random-words"));
        assert!(report.contains("96-bit unique-decoding query target"));
        assert!(report.contains("WorldFnd WHIR | CLI defaults: rate 1/2"));
        assert!(!report.contains("133-bit round-by-round"));
        assert!(report.contains("140-cell"));
        assert!(report.contains("results/hash-x86_64"));
        assert!(report.contains("Linux x86_64"));
        assert!(!report.contains("leopard"));
        assert!(report.contains("hash-eval matrix"));
        let host_ram = 121u64 * 1024 * 1024 * 1024;
        let _ = worker_memory_limit_bytes(host_ram);
        assert!(report.contains("90% of host RAM"));

        let mut audited = record.clone();
        audited.provenance.harness_revision = RESULTS_ONLY_REVISIONS[0].into();
        let raw = serde_json::to_string(&audited).unwrap();
        for rendered in [
            render_markdown_hash_eval_report(std::slice::from_ref(&audited)),
            super::render_latex_hash_eval_report(std::slice::from_ref(&audited)),
        ] {
            assert!(rendered.contains(&RESULTS_ONLY_REVISIONS[0][..40]));
            assert!(!rendered.contains(RESULTS_ONLY_REVISIONS[0]));
            assert!(rendered.contains("result-file-only"));
        }
        assert_eq!(serde_json::to_string(&audited).unwrap(), raw);
        audited.provenance.harness_revision = "unknown+dirty.123".into();
        assert!(render_markdown_hash_eval_report(&[audited]).contains("unknown+dirty.123"));

        let mut recorded_pin = record.clone();
        recorded_pin.implementation_revision = "2222222222222222222222222222222222222222".into();
        let pinned_report = render_markdown_hash_eval_report(&[recorded_pin]);
        assert!(pinned_report.contains(
            "https://github.com/LayerZero-Labs/akita/commit/2222222222222222222222222222222222222222"
        ));

        let mut no_flags = record.clone();
        no_flags.provenance.rustflags.clear();
        let no_flags_report = render_markdown_hash_eval_report(&[no_flags]);
        assert!(no_flags_report.contains("Recorded worker flags"));
        assert!(no_flags_report.contains("`(none)`"));

        let mut other_machine = record.clone();
        other_machine.sample = 1;
        other_machine.provenance.cpu_model = "Machine B".into();
        let mixed_report = render_markdown_hash_eval_report(&[record, other_machine]);
        assert!(mixed_report.contains("multiple machine descriptions"));
        assert!(!mixed_report.contains("single Machine B"));
    }
}
