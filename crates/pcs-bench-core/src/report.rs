//! Paper-style lattice evaluation report (prose + both tables).

use crate::lattice::{SchemeId, AKITA_PR466_URL};
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
    let timing = aggregate_timing_rows(records);
    let resources = aggregate_resource_rows(records);
    format!(
        "{}\n\n{}\n\n{}\n\n{}\n\n{}\n",
        markdown_prose(&provenance),
        render_markdown_timing_table(&timing),
        render_markdown_resource_table(&resources),
        markdown_pins(),
        markdown_reproduction(&provenance)
    )
}

/// LaTeX report matching `tab:eval-lattice-time` and `tab:eval-lattice-resources`.
#[must_use]
pub fn render_latex_eval_report(records: &[LatticeRecord]) -> String {
    let provenance = first_provenance(records);
    let timing = aggregate_timing_rows(records);
    let resources = aggregate_resource_rows(records);
    format!(
        "{}\n\n{}\n\n{}\n\n{}\n\n{}\n",
        latex_prose(&provenance),
        render_latex_timing_table(&timing),
        render_latex_resource_table(&resources),
        latex_pins(),
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

fn machine_sentence(provenance: &Provenance, latex: bool) -> String {
    let avx512 = provenance.avx512
        || provenance
            .isa_notes
            .to_ascii_uppercase()
            .contains("AVX-512")
        || provenance.rustflags.contains("target-cpu=native");
    let avx = if avx512 {
        if latex {
            "AVX-512F is advertised by the CPU and was activated for this run (Greyhound \\texttt{-march=native}; Akita and RoKoKo \\texttt{-C target-cpu=native})."
        } else {
            "AVX-512F is advertised by the CPU and was activated for this run (Greyhound `-march=native`; Akita and RoKoKo `-C target-cpu=native`)."
        }
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
    let rustflags = if provenance.rustflags.is_empty() {
        String::new()
    } else if latex {
        format!(
            " Compiler flags: \\texttt{{{}}}.",
            escape_tex(&provenance.rustflags)
        )
    } else {
        format!(" Compiler flags: `{}`.", provenance.rustflags)
    };
    if latex {
        format!(
            "Measurements were collected on a single {} ({}{cpus}{mem}). {avx}{rustflags}",
            escape_tex(&provenance.cpu_model),
            escape_tex(&provenance.target),
        )
    } else {
        format!(
            "Measurements were collected on a single {} ({}{cpus}{mem}). {avx}{rustflags}",
            provenance.cpu_model, provenance.target,
        )
    }
}

fn markdown_prose(provenance: &Provenance) -> String {
    format!(
        "{}\n\n\
         Our first experiment compares Akita with prior lattice-based PCSs on dense\n\
         polynomial data at the target volumes above. For each input, Akita\n\
         uses the validated planner schedule selected for that field and size.\n\
         The pinned catalogs omit $n_v=22$ and $n_v=24$; those rows are generated\n\
         with that same planner at the measured commit.\n\
         The comparison is exclusively single-threaded because neither Greyhound nor RoKoKo\n\
         natively supports multithreading. Timing cells report median ± sample standard\n\
         deviation across fresh processes after warmup. Scheme names link to the exact\n\
         git commit that was measured. Akita is reported both at the pinned `main` commit\n\
         and at the tip of [PR #466]({AKITA_PR466_URL}).\n\n\
         The timing comparison separates commitment, opening, and verification, while the\n\
         resources table reports communication, memory, and preprocessing. Released but\n\
         non-normalized measurements are labeled historical when retained and are never used\n\
         to form headline ratios.\n\n\
         RoKoKo uses the field $\\mathbb{{F}}_{{2^{{50}}-2687}}$ and fixed native parameter\n\
         sets, so we report its closest supported input at each target payload. An OOM entry\n\
         {oom}. The populated RoKoKo rows contain approximately\n\
         $25/16$ times the target number of logical bits, since their native field has about\n\
         50 bits rather than 32. They report the cost of those native instances.",
        machine_sentence(provenance, false),
        oom = oom_clause(provenance, false),
    )
}

fn latex_prose(provenance: &Provenance) -> String {
    format!(
        "{}\n\n\
         Our first experiment compares Akita with prior lattice-based PCSs on dense\n\
         polynomial data at the target volumes above. For each input, Akita\n\
         uses the validated planner schedule selected for that field and size.\n\
         The pinned catalogs omit $n_v=22$ and $n_v=24$; those rows are generated\n\
         with that same planner at the measured commit.\n\
         The comparison is exclusively single-threaded because neither Greyhound nor RoKoKo\n\
         natively supports multithreading.\n\
         Timing cells report median $\\pm$ sample standard deviation across fresh processes\n\
         after warmup. Scheme names are hyperlinks to the exact git commit that was measured.\n\
         Akita is reported both at the pinned \\texttt{{main}} commit and at the tip of\n\
         \\href{{{}}}{{PR \\#466}}.\n\n\
         The timing comparison in \\Cref{{tab:eval-lattice-time}} separates commitment,\n\
         opening, and verification, while \\Cref{{tab:eval-lattice-resources}} reports\n\
         communication, memory, and preprocessing.  Released but non-normalized\n\
         measurements are labeled historical when retained and are never used to form\n\
         headline ratios.\n\n\
         RoKoKo uses the field $\\mathbb F_{{2^{{50}}-2687}}$ and fixed native parameter\n\
         sets, so we report its closest supported input at each target payload.  An\n\
         \\evaloom{{}} entry {oom}.\n\
         The populated RoKoKo rows contain approximately $25/16$ times the target\n\
         number of logical bits, since their native field has about $50$ bits rather\n\
         than $32$. They report the cost of those native instances.",
        machine_sentence(provenance, true),
        AKITA_PR466_URL,
        oom = oom_clause(provenance, true),
    )
}

fn markdown_pins() -> String {
    let mut out = String::from("### Measured commits\n\n");
    for scheme in SchemeId::all() {
        let _ = writeln!(
            out,
            "- {} [`{}`]({})",
            scheme.display_name(),
            scheme.short_sha(),
            scheme.commit_url()
        );
    }
    let _ = writeln!(out, "- Akita PR: <{AKITA_PR466_URL}>");
    out
}

fn latex_pins() -> String {
    let mut out = String::from(
        "\\medskip\n\\noindent\\textbf{Measured commits.}\n\\begin{itemize}\\setlength{\\itemsep}{0pt}\n",
    );
    for scheme in SchemeId::all() {
        let _ = writeln!(
            out,
            "\\item {}: \\href{{{}}}{{\\texttt{{{}}}}}",
            scheme.latex_name(),
            scheme.commit_url(),
            scheme.short_sha()
        );
    }
    let _ = writeln!(
        out,
        "\\item Akita PR: \\url{{{AKITA_PR466_URL}}}\n\\end{{itemize}}"
    );
    out
}

fn oom_clause(provenance: &Provenance, latex: bool) -> String {
    match provenance.resolved_memory_limit_bytes() {
        Some(limit) => {
            let gib = format_limit_gib(limit);
            if latex {
                format!("exceeds the {gib}~GiB worker memory limit (90\\% of host RAM)")
            } else {
                format!("exceeds the {gib} GiB worker memory limit (90% of host RAM)")
            }
        }
        None => {
            if latex {
                "exceeds the worker memory limit (90\\% of host RAM when known)".into()
            } else {
                "exceeds the worker memory limit (90% of host RAM when known)".into()
            }
        }
    }
}

fn format_limit_gib(bytes: u64) -> String {
    format!("{:.0}", bytes as f64 / 1_073_741_824.0)
}

fn markdown_reproduction(provenance: &Provenance) -> String {
    let mut out = String::from("### Commands used for these numbers\n\n");
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
        "\\medskip\n\\noindent\\textbf{{Commands used for these numbers.}}\n{}\n\
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
    text.replace("{MEMORY_LIMIT_GIB}", &format_limit_gib(limit))
        .replace("{MEMORY_LIMIT_BYTES}", &limit.to_string())
}

/// How to reproduce the checked-in Linux x86_64 headline tables.
const REPRODUCTION_PROSE_MARKDOWN: &str = "\
These tables were produced on a Linux **x86_64** AVX-512 host (AMD Ryzen 9 9950X)
from this repository. The toolchain pin is Rust **1.95** (`rust-toolchain.toml`).
RoKoKo uses `rustup` **nightly**. Every timed worker is a fresh process wrapped
in `scripts/with-memlimit.sh` at {MEMORY_LIMIT_GIB}~GiB (`ulimit -v`, 90% of host RAM) with `RAYON_NUM_THREADS=1`.
The runner defaults are **1 warmup + 3 measured** samples per cell; warmup rows
are stored with `warmup: true` and excluded from the median. Greyhound is built
with `-march=native -O3 -flto`. Akita and RoKoKo inherit `RUSTFLAGS=-C target-cpu=native`.
Akita PR #466 is a separate Cargo tree (`benchmarks/akita-pr466`,
`CARGO_TARGET_DIR=target/akita-pr466`) so it does not unify with the pinned
`main` revision. `./scripts/fetch-vendors.sh` clones both Akita pins,
installs planner-generated `fp32-dense` rows for `nv=22` and `nv=24`, and
patches RoKoKo so the executor prints commitment, CRS, and peak RSS.

Non-interactive shells may not put Cargo on `PATH`; `source ~/.cargo/env`
is required in that case. `CARGO_NET_GIT_FETCH_WITH_CLI=true` avoids libgit2 auth
failures when fetching the pinned git dependencies.
Checked-in numbers live in `results/lattice-x86_64/`.";

const REPRODUCTION_PROSE_LATEX: &str = "\
These tables were produced on a Linux x86\\_64 AVX-512 host (AMD Ryzen~9 9950X) from
this repository. The toolchain pin is Rust 1.95 (\\texttt{rust-toolchain.toml}).
RoKoKo uses \\texttt{rustup} nightly. Every timed worker is a fresh process wrapped
in \\texttt{scripts/with-memlimit.sh} at {MEMORY_LIMIT_GIB}~GiB (\\texttt{ulimit -v}, 90\\% of host RAM) with
\\texttt{RAYON\\_NUM\\_THREADS=1}. The runner defaults are 1 warmup and 3 measured
samples per cell; warmup rows are stored with \\texttt{warmup: true} and excluded
from the median. Checked-in numbers live in \\texttt{results/lattice-x86\\_64/}.";

const REPRODUCTION_COMMANDS: &str = "\
# On an AVX-512 Linux x86_64 host
source \"$HOME/.cargo/env\"   # if cargo is not on PATH
cd /path/to/akita-benchmark

rustup toolchain install nightly -c rustc,cargo   # once, for RoKoKo
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export RUSTFLAGS=\"-C target-cpu=native\"
export RAYON_NUM_THREADS=1

./scripts/fetch-vendors.sh          # Greyhound, RoKoKo, Akita pins + nv=22/24 catalogs
./scripts/build-greyhound.sh

# Full 20-cell matrix (Akita main, Akita #466, Greyhound, RoKoKo)
./scripts/lattice-eval.sh run --out results/lattice-x86_64

# Rebuild Markdown + LaTeX from the JSONL already in that directory
cargo run -p pcs-bench-runner --bin pcs-bench -- lattice-eval compare \\
  results/lattice-x86_64 --out-dir results/lattice-x86_64
";

const SANITY_PROSE_MARKDOWN: &str = "\
**Sanity-check the harness before trusting a full run.** `lattice-eval matrix`
prints the 20-cell plan (unsupported RoKoKo sizes, Akita/Greyhound `log2 N`,
RoKoKo `p-26`/`p-28`/`p-30`). A single supported cell should verify and emit
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

# One measured sample of a supported cell (payload 2^31, log2 N = 26)
./scripts/lattice-eval.sh run --scheme akita --payload 31 --runs 1 --warmups 0
./scripts/lattice-eval.sh run --scheme greyhound --payload 31 --runs 1 --warmups 0
./scripts/lattice-eval.sh run --scheme rokoko --payload 31 --runs 1 --warmups 0
./scripts/lattice-eval.sh run --scheme akita-pr466 --payload 31 --runs 1 --warmups 0

# Direct workers (what each harness sample wraps with with-memlimit.sh)
./scripts/with-memlimit.sh {MEMORY_LIMIT_BYTES} \\
  env RAYON_NUM_THREADS=1 AKITA_PARALLEL=0 \\
  cargo run --release -p pcs-bench-akita --bin lattice-eval -- \\
    --log2-n 26 --payload-log2 31
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
    use crate::observation::{LatticeRecord, Provenance, RunStatus, RESULT_SCHEMA_VERSION};
    use std::collections::BTreeMap;

    #[test]
    fn report_names_the_machine_and_avx512() {
        let mut timings_ns = BTreeMap::new();
        timings_ns.insert("commit".into(), 159_000_000);
        timings_ns.insert("open".into(), 2_070_000_000);
        timings_ns.insert("verify".into(), 41_900_000);
        let record = LatticeRecord {
            schema_version: RESULT_SCHEMA_VERSION,
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
            historical: false,
            timings_ns,
            proof_bytes: Some(61_337),
            commitment_bytes: Some(3072),
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
        let report = render_markdown_eval_report(&[record]);
        assert!(report.contains("AMD Ryzen 9 9950X"));
        assert!(report.contains("AVX-512F"));
        assert!(report.contains("-C target-cpu=native"));
        assert!(report.contains("PR #466"));
        assert!(report.contains(&SchemeId::Akita.commit_url()));
        assert!(report.contains("Commands used for these numbers"));
        assert!(report.contains("./scripts/fetch-vendors.sh"));
        assert!(report.contains("./scripts/build-greyhound.sh"));
        assert!(report.contains("results/lattice-x86_64"));
        assert!(report.contains("Linux **x86_64**"));
        assert!(!report.contains("leopard"));
        assert!(report.contains("lattice-eval matrix"));
        assert!(report.contains("RUSTFLAGS=\"-C target-cpu=native\""));
        let host_ram = 121u64 * 1024 * 1024 * 1024;
        let limit = worker_memory_limit_bytes(host_ram);
        assert!(report.contains(&format!("with-memlimit.sh {limit}")));
        assert!(report.contains("90% of host RAM"));
        assert!(report.contains("109 GiB worker memory limit"));
    }
}
