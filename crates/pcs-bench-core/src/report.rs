//! Paper-style lattice evaluation report (prose + both tables).

use crate::observation::{LatticeRecord, Provenance};
use crate::table::{
    aggregate_resource_rows, aggregate_timing_rows, render_latex_resource_table,
    render_latex_timing_table, render_markdown_resource_table, render_markdown_timing_table,
};
use std::fmt::Write as _;

/// Markdown report matching the paper's evaluation write-up.
#[must_use]
pub fn render_markdown_eval_report(records: &[LatticeRecord]) -> String {
    let provenance = first_provenance(records);
    let homogeneous_machine = records_share_machine(records, &provenance);
    let timing = aggregate_timing_rows(records);
    let resources = aggregate_resource_rows(records);
    format!(
        "{}\n\n{}\n\n{}\n\n{}\n\n{}\n",
        markdown_prose(&provenance, homogeneous_machine),
        render_markdown_timing_table(&timing),
        render_markdown_resource_table(&resources),
        markdown_pins(records),
        markdown_reproduction(&provenance)
    )
}

/// LaTeX report matching `tab:eval-lattice-time` and `tab:eval-lattice-resources`.
#[must_use]
pub fn render_latex_eval_report(records: &[LatticeRecord]) -> String {
    let provenance = first_provenance(records);
    let homogeneous_machine = records_share_machine(records, &provenance);
    let timing = aggregate_timing_rows(records);
    let resources = aggregate_resource_rows(records);
    format!(
        "{}\n\n{}\n\n{}\n\n{}\n\n{}\n",
        latex_prose(&provenance, homogeneous_machine),
        render_latex_timing_table(&timing),
        render_latex_resource_table(&resources),
        latex_pins(records),
        latex_reproduction(&provenance)
    )
}

fn first_provenance(records: &[LatticeRecord]) -> Provenance {
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

fn records_share_machine(records: &[LatticeRecord], expected: &Provenance) -> bool {
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
        "This host does not advertise AVX-512F; Greyhound numbers from this machine are not valid headline results."
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
         Our first experiment compares Akita with prior lattice-based PCSs on dense\n\
         polynomial data at the target volumes above. Akita samples uniform full-field\n\
         coefficients and uniform extension-field opening points. This table is a declared\n\
         native-workload survey unless all displayed security and statement assumptions match.\n\
         For each input, Akita\n\
         uses the validated planner schedule selected for that field and size.\n\
         The pinned catalogs omit $n_v=22$ and $n_v=24$; those rows are generated\n\
         with that same planner at the measured commit.\n\
         Akita appears twice: the direct `fp32-dense` catalog, and the same pin\n\
         with recursive setup offloading (`fp32-dense-recursive`).\n\
         The comparison is exclusively single-threaded: RoKoKo has no native multithreaded\n\
         prover, and Greyhound is pinned to `LATTICE_DOGS_THREADS=1` even though the\n\
         reference can parallelize extension products. Greyhound uses the\n\
         `l2-quantum128-adps16` Euclidean SIS policy and reports contextual proof bytes.\n\
         Timing cells report the median and, when supported by the sample count, a\n\
         conservative distribution-free 95% confidence interval. Scheme names link to the exact\n\
         git commit that was measured.\n\n\
         The timing comparison separates commitment, opening, and verification, while the\n\
         cold total includes setup plus commitment and opening. If an implementation embeds\n\
         reusable setup inside commitment, that cost remains in its cold total and its separate\n\
         setup entry is reported as unknown; reusable state size is reported when measurable.\n\
         The resources table reports communication, memory, and preprocessing.\n\n\
         RoKoKo uses the field $\\mathbb{{F}}_{{2^{{50}}-2687}}$ and fixed native parameter\n\
         sets, so we report its closest supported input at each target payload. An OOM entry\n\
         {oom}. RoKoKo's native field has about 50 bits, but its sampler is bounded to\n\
         31-bit coefficients. The displayed payload is nominal field capacity and must not\n\
         be interpreted as sampled information content or used to rescale throughput.",
        machine_sentence(provenance, false, homogeneous_machine),
        oom = oom_clause(provenance, false),
    )
}

fn latex_prose(provenance: &Provenance, homogeneous_machine: bool) -> String {
    format!(
        "{}\n\n\
         Our first experiment compares Akita with prior lattice-based PCSs on dense\n\
         polynomial data at the target volumes above. Akita samples uniform full-field\n\
         coefficients and uniform extension-field opening points. This table is a declared\n\
         native-workload survey unless all displayed security and statement assumptions match.\n\
         For each input, Akita\n\
         uses the validated planner schedule selected for that field and size.\n\
         The pinned catalogs omit $n_v=22$ and $n_v=24$; those rows are generated\n\
         with that same planner at the measured commit.\n\
         Akita appears twice: the direct \\texttt{{fp32-dense}} catalog, and the same pin\n\
         with recursive setup offloading (\\texttt{{fp32-dense-recursive}}).\n\
         The comparison is exclusively single-threaded: RoKoKo has no native multithreaded\n\
         prover, and Greyhound is pinned to \\texttt{{LATTICE\\_DOGS\\_THREADS=1}} even though the\n\
         reference can parallelize extension products. Greyhound uses the\n\
         \\texttt{{l2-quantum128-adps16}} Euclidean SIS policy and reports contextual proof bytes.\n\
         Timing cells report the median and, when supported by the sample count, a\n\
         conservative distribution-free 95\\% confidence interval. Scheme names are hyperlinks to the exact git commit that was measured.\n\n\
         The timing comparison in \\Cref{{tab:eval-lattice-time}} separates commitment,\n\
         opening, and verification; cold total includes setup, commitment, and opening.\n\
         Embedded reusable setup remains charged to commitment and is otherwise reported as unknown.\n\
         \\Cref{{tab:eval-lattice-resources}} reports communication, memory, and preprocessing.\n\n\
         RoKoKo uses the field $\\mathbb F_{{2^{{50}}-2687}}$ and fixed native parameter\n\
         sets, so we report its closest supported input at each target payload.  An\n\
         \\evaloom{{}} entry {oom}.\n\
         RoKoKo's native field has about $50$ bits, but its sampler is bounded to\n\
         31-bit coefficients. The displayed payload is nominal field capacity and must not\n\
         be interpreted as sampled information content or used to rescale throughput.",
        machine_sentence(provenance, true, homogeneous_machine),
        oom = oom_clause(provenance, true),
    )
}

fn markdown_pins(records: &[LatticeRecord]) -> String {
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

fn latex_pins(records: &[LatticeRecord]) -> String {
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

/// How to reproduce the checked-in Linux x86_64 headline tables.
const REPRODUCTION_PROSE_MARKDOWN: &str = "\
Machine, ISA, compiler, executable, lockfile, command, and timestamp provenance
for this dataset are recorded with each observation. The infrastructure
toolchain pin is Rust **1.95** (`rust-toolchain.toml`).
Recorded runner command: `{RUN_COMMAND}`. The commands below are a template,
not reconstructed provenance.
RoKoKo uses `rustup` **nightly-2026-09-03**. Workers are built before sampling;
every timed execution is then a fresh process wrapped
in `scripts/with-memlimit.sh` with a {MEMORY_LIMIT_GIB}~GiB virtual-address-space ceiling (`ulimit -v`, numerically 90% of host RAM) and `RAYON_NUM_THREADS=1`.
Raw records identify warmup and measured processes separately; warmup rows
are stored with `warmup: true` and excluded from the aggregate. Workload seeds
and the `vary`/`fixed` seed mode are recorded per observation. Greyhound is
`LayerZero-Labs/greyhound-reference`, built with `-march=native -O3 -flto`,
and run with `LATTICE_DOGS_THREADS=1` and `LABRADOR_SIS_SECURITY=l2-quantum128-adps16`.
Proof sizes are contextual wire bytes. Recorded worker flags for this dataset:
`{RUSTFLAGS}`.
`./scripts/fetch-vendors.sh` clones the pinned implementations,
installs planner-generated `fp32-dense` rows for `nv=22` and `nv=24`, installs
the recursive `fp32-dense` setup-offload catalog, and
patches RoKoKo so the executor prints commitment, CRS, and peak RSS.

Non-interactive shells may not put Cargo on `PATH`; `source ~/.cargo/env`
is required in that case. `CARGO_NET_GIT_FETCH_WITH_CLI=true` avoids libgit2 auth
failures when fetching the pinned git dependencies.
Published numbers live in `results/lattice-x86_64/`.";

const REPRODUCTION_PROSE_LATEX: &str = "\
Machine, ISA, compiler, executable, lockfile, command, and timestamp provenance
for this dataset are recorded with each observation. The infrastructure
toolchain pin is Rust 1.95 (\\texttt{rust-toolchain.toml}).
The exact runner command is recorded in the JSONL; the commands below are a
template, not reconstructed provenance.
RoKoKo uses \\texttt{rustup} nightly-2026-09-03. Workers are built before
sampling; each timed worker is a fresh process wrapped in
\\texttt{scripts/with-memlimit.sh} under a {MEMORY_LIMIT_GIB}~GiB
virtual-address-space ceiling (\\texttt{ulimit -v}) with
\\texttt{RAYON\\_NUM\\_THREADS=1}. Raw records identify warmup and measured
processes separately; warmup rows are stored with \\texttt{warmup: true} and excluded.
Recorded RUSTFLAGS: \\texttt{{{RUSTFLAGS}}}.
Checked-in numbers live in \\texttt{results/lattice-x86\\_64/}.";

const REPRODUCTION_COMMANDS: &str = "\
# On an AVX-512 Linux x86_64 host
source \"$HOME/.cargo/env\"   # if cargo is not on PATH
cd /path/to/akita-benchmark

rustup toolchain install nightly-2026-09-03 -c rustc,cargo   # once, for RoKoKo
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export RUSTFLAGS=\"-C target-cpu=native\"
export RAYON_NUM_THREADS=1

./scripts/fetch-vendors.sh          # Greyhound, RoKoKo, Akita pins + nv=22/24 + offload catalogs
./scripts/extend-akita-fp32-dense-offload.sh third_party/akita   # once; fills the offload catalog
./scripts/build-greyhound.sh

# Full 20-cell matrix (Akita, Akita offload, Greyhound, RoKoKo)
./scripts/lattice-eval.sh run --out results/lattice-x86_64

# Rebuild Markdown + LaTeX from the JSONL already in that directory
cargo run -p pcs-bench-runner --bin pcs-bench -- lattice-eval compare \\
  results/lattice-x86_64 --out-dir results/lattice-x86_64
";

const SANITY_PROSE_MARKDOWN: &str = "\
**Sanity-check the harness before trusting a full run.** `lattice-eval matrix`
prints the 20-cell plan (unsupported RoKoKo sizes, Akita/Greyhound `log2 N`,
RoKoKo `p-26`/`p-28`/`p-30`, and the Akita setup-offload row). A single supported cell should verify and emit
JSON with `status: ok`. Unit tests cover the RoKoKo log parser, OOM
classification, and table tokens. Each sample the runner launches is equivalent
to the worker commands below (still under the 90%-of-RAM cap).";

const SANITY_PROSE_LATEX: &str = "\
\\noindent Sanity-check the harness before a full run.
\\texttt{lattice-eval matrix} prints the 20-cell plan.
A single supported cell should verify and emit JSON with \\texttt{status: ok}.
Unit tests cover the RoKoKo log parser, OOM classification, and table tokens.";

const SANITY_COMMANDS: &str = "\
export RUSTFLAGS=\"-C target-cpu=native\"
export RAYON_NUM_THREADS=1

cargo test --workspace --locked
cargo run -p pcs-bench-runner --bin pcs-bench -- lattice-eval matrix
CARGO_TARGET_DIR=target/akita cargo build --release --locked \\
  --manifest-path benchmarks/akita/Cargo.toml --bin lattice-eval

# One measured sample of a supported cell (payload 2^31, log2 N = 26)
./scripts/lattice-eval.sh run --scheme akita --payload 31 --runs 1 --warmups 0
./scripts/lattice-eval.sh run --scheme akita-offload --payload 31 --runs 1 --warmups 0
./scripts/lattice-eval.sh run --scheme greyhound --payload 31 --runs 1 --warmups 0
./scripts/lattice-eval.sh run --scheme rokoko --payload 31 --runs 1 --warmups 0

# Direct workers (what each harness sample wraps with with-memlimit.sh)
./scripts/with-memlimit.sh {MEMORY_LIMIT_BYTES} \\
  env RAYON_NUM_THREADS=1 AKITA_PARALLEL=0 PCS_BENCH_SEED=1 \\
  target/akita/release/lattice-eval \\
    --log2-n 26 --payload-log2 31
./scripts/with-memlimit.sh {MEMORY_LIMIT_BYTES} \\
  env RAYON_NUM_THREADS=1 AKITA_PARALLEL=0 PCS_BENCH_SEED=1 \\
  target/akita/release/lattice-eval \\
    --log2-n 26 --payload-log2 31 --offload
./scripts/with-memlimit.sh {MEMORY_LIMIT_BYTES} target/greyhound/lattice-eval --log2-n 26
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
    use super::render_markdown_eval_report;
    use crate::lattice::{worker_memory_limit_bytes, SchemeId};
    use crate::observation::{LatticeRecord, Provenance, RunStatus};
    use std::collections::BTreeMap;

    #[test]
    fn report_names_the_machine_and_avx512() {
        let mut timings_ns = BTreeMap::new();
        timings_ns.insert("commit".into(), 159_000_000);
        timings_ns.insert("open".into(), 2_070_000_000);
        timings_ns.insert("verify".into(), 41_900_000);
        let record = LatticeRecord {
            status: RunStatus::Ok,
            status_detail: None,
            scheme: SchemeId::Akita,
            implementation_revision: SchemeId::Akita.revision().into(),
            payload_log2: 27,
            log2_n: Some(22),
            field: "2^{32}-99".into(),
            native_param: Some("fp32-dense".into()),
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
        let report = render_markdown_eval_report(std::slice::from_ref(&record));
        assert!(report.contains("AMD Ryzen 9 9950X"));
        assert!(report.contains("AVX-512F"));
        assert!(report.contains("-C target-cpu=native"));
        assert!(report.contains("Akita (offload)"));
        assert!(report.contains("recursive setup offloading"));
        assert!(report.contains(&SchemeId::Akita.commit_url()));
        assert!(report.contains("Reproduction template"));
        assert!(report.contains("./scripts/fetch-vendors.sh"));
        assert!(report.contains("./scripts/extend-akita-fp32-dense-offload.sh"));
        assert!(report.contains("./scripts/build-greyhound.sh"));
        assert!(report.contains("results/lattice-x86_64"));
        assert!(report.contains("Linux x86_64"));
        assert!(!report.contains("leopard"));

        let mut recorded_pin = record.clone();
        recorded_pin.implementation_revision = "1111111111111111111111111111111111111111".into();
        let pinned_report = render_markdown_eval_report(&[recorded_pin]);
        assert!(pinned_report.contains(
            "https://github.com/LayerZero-Labs/akita/commit/1111111111111111111111111111111111111111"
        ));

        let mut other_machine = record.clone();
        other_machine.sample = 1;
        other_machine.provenance.cpu_model = "Machine B".into();
        let mixed_report = render_markdown_eval_report(&[record, other_machine]);
        assert!(mixed_report.contains("multiple machine descriptions"));
        assert!(!mixed_report.contains("single Machine B"));
        assert!(report.contains("lattice-eval matrix"));
        assert!(report.contains("RUSTFLAGS=\"-C target-cpu=native\""));
        let host_ram = 121u64 * 1024 * 1024 * 1024;
        let limit = worker_memory_limit_bytes(host_ram);
        assert!(report.contains(&format!("with-memlimit.sh {limit}")));
        assert!(report.contains("90% of host RAM"));
        assert!(report.contains("109 GiB virtual-address-space ceiling"));
    }
}
