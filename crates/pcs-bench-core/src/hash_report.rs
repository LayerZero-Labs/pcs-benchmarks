//! Paper-style hash evaluation report (prose + both tables).

use crate::hash::HashSchemeId;
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
    let timing = aggregate_hash_timing_rows(records);
    let resources = aggregate_hash_resource_rows(records);
    format!(
        "{}\n\n{}\n\n{}\n\n{}\n\n{}\n",
        markdown_prose(&provenance),
        render_markdown_hash_timing_table(&timing),
        render_markdown_hash_resource_table(&resources),
        markdown_pins(),
        markdown_reproduction(&provenance)
    )
}

/// LaTeX report matching `tab:eval-hash-time` and `tab:eval-hash-resources`.
#[must_use]
pub fn render_latex_hash_eval_report(records: &[HashRecord]) -> String {
    let provenance = first_provenance(records);
    let timing = aggregate_hash_timing_rows(records);
    let resources = aggregate_hash_resource_rows(records);
    format!(
        "{}\n\n{}\n\n{}\n\n{}\n\n{}\n",
        latex_prose(&provenance),
        render_latex_hash_timing_table(&timing),
        render_latex_hash_resource_table(&resources),
        latex_pins(),
        latex_reproduction(&provenance)
    )
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

fn machine_sentence(provenance: &Provenance, latex: bool) -> String {
    let avx512 = provenance.avx512
        || provenance
            .isa_notes
            .to_ascii_uppercase()
            .contains("AVX-512")
        || provenance.rustflags.contains("target-cpu=native");
    let avx = if avx512 {
        if latex {
            "AVX-512F is advertised by the CPU and was activated for this run (\\texttt{-C target-cpu=native})."
        } else {
            "AVX-512F is advertised by the CPU and was activated for this run (`-C target-cpu=native`)."
        }
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
         Our second experiment compares Akita with other high-performance hash-based PCSs\n\
         on the same dense standalone payload ladder ($2^{{27}}$ through $2^{{35}}$ bits).\n\
         Each scheme uses its **native** security target, hash, field, and rate rather than a\n\
         common 128-bit retune, so cells are **not** $\\lambda$-comparable.\n\
         Akita, Plonky3 WHIR, and SP1 BaseFold stay at the 128-bit transcript-error target:\n\
         WHIR is Plonky3 `p3-whir` at `security_level=128`. Capacity bound at rate $1/2$ is used\n\
         when that instance fits a 30-bit KoalaBear grind ($\\log_2 N \\le 26$);\n\
         unique decoding at rate $1/2$ is used at $\\log_2 N=28$ and $30$, where list-decoding\n\
         bounds on KoalaBear cannot close 128 bits within that grind limit. BaseFold (SP1) is\n\
         SLOP stacked BaseFold with FRI parameters `log_blowup=1`, 112 queries, and\n\
         16 bits of grinding (conjectured soundness $1\\cdot 112+16=128$).\n\
         Plonky2 FRI, Plonky3 FRI/STIR, Binius64 BaseFold, and Flock Ligerito Fast use native\n\
         **100-bit** targets. ProveKit WHIR uses Johnson-bound **133-bit** Goldilocks degree-3\n\
         challenges with base-field coefficients. KoalaBear univariate FRI/STIR pack into a\n\
         $2^{{23}}\\times 2^{{n-23}}$ matrix when $\\log_2 N>23$ (two-adicity 24 at rate $1/2$).\n\
         Timing cells report median ± sample standard deviation across fresh processes\n\
         after warmup, at **1 and 8 threads**. Scheme names link to the exact git commit\n\
         that was measured. Unmeasured roster cells are `pending`.\n\n\
         The timing comparison separates commitment, opening, and verification, while the\n\
         resources table reports communication, memory (1-thread and 8-thread peak RSS),\n\
         and preprocessing. An OOM entry {oom}.",
        machine_sentence(provenance, false),
        oom = oom_clause(provenance, false),
    )
}

fn latex_prose(provenance: &Provenance) -> String {
    format!(
        "{}\n\n\
         Our second experiment compares Akita with other high-performance hash-based PCSs\n\
         on the same dense standalone payload ladder ($2^{{27}}$ through $2^{{35}}$ bits).\n\
         Each scheme uses its native security target, hash, field, and rate rather than a\n\
         common 128-bit retune, so cells are not $\\lambda$-comparable.\n\
         Akita, Plonky3 WHIR, and SP1 BaseFold stay at the 128-bit transcript-error target:\n\
         WHIR is Plonky3 \\texttt{{p3-whir}} at \\texttt{{security\\_level=128}}. Capacity bound at\n\
         rate $1/2$ is used when that instance fits a 30-bit KoalaBear grind\n\
         ($\\log_2 N \\le 26$); unique decoding at rate $1/2$ is used at $\\log_2 N=28$\n\
         and $30$, where list-decoding bounds on KoalaBear cannot close 128 bits within\n\
         that grind limit. BaseFold (SP1) is SLOP stacked BaseFold with FRI parameters\n\
         $\\log_2(1/\\rho)=1$, 112 queries, and 16 bits of grinding (conjectured soundness\n\
         $1\\cdot 112+16=128$). Plonky2 FRI, Plonky3 FRI/STIR, Binius64 BaseFold, and Flock\n\
         Ligerito Fast use native 100-bit targets. ProveKit WHIR uses Johnson-bound 133-bit\n\
         Goldilocks degree-3 challenges with base-field coefficients. KoalaBear univariate\n\
         FRI/STIR pack into a $2^{{23}}\\times 2^{{n-23}}$ matrix when $\\log_2 N>23$. Timing cells\n\
         report median $\\pm$ sample standard deviation across fresh processes after warmup,\n\
         at 1 and 8 threads. Scheme names are hyperlinks to the exact git commit that was\n\
         measured. Unmeasured roster cells are \\evalpending{{}}.\n\n\
         The timing comparison in \\Cref{{tab:eval-hash-time}} separates commitment,\n\
         opening, and verification, while \\Cref{{tab:eval-hash-resources}} reports\n\
         communication, memory, and preprocessing.  An \\evaloom{{}} entry {oom}.",
        machine_sentence(provenance, true),
        oom = oom_clause(provenance, true),
    )
}

fn markdown_pins() -> String {
    let mut out = String::from("### Measured commits\n\n");
    for scheme in HashSchemeId::all() {
        let extra = extra_pin_suffix(scheme, false);
        let _ = writeln!(
            out,
            "- {} [`{}`]({}){extra}",
            scheme.display_name(),
            scheme.short_sha(),
            scheme.commit_url(),
        );
    }
    out
}

fn latex_pins() -> String {
    let mut out = String::from(
        "\\medskip\n\\noindent\\textbf{Measured commits.}\n\\begin{itemize}\\setlength{\\itemsep}{0pt}\n",
    );
    for scheme in HashSchemeId::all() {
        let extra = extra_pin_suffix(scheme, true);
        let _ = writeln!(
            out,
            "\\item {}: \\href{{{}}}{{\\texttt{{{}}}}}{extra}",
            scheme.latex_name(),
            scheme.commit_url(),
            scheme.short_sha()
        );
    }
    out.push_str("\\end{itemize}\n");
    out
}

fn extra_pin_suffix(scheme: HashSchemeId, latex: bool) -> String {
    let Some(url) = scheme.extra_commit_url() else {
        return String::new();
    };
    let sha = url.rsplit('/').next().unwrap_or("");
    let short = sha.get(..8).unwrap_or(sha);
    if latex {
        format!(r"; whir \href{{{url}}}{{\texttt{{{short}}}}}")
    } else {
        format!("; whir [`{short}`]({url})")
    }
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

const REPRODUCTION_PROSE_MARKDOWN: &str = "\
These tables were produced on a Linux **x86_64** AVX-512 host (AMD Ryzen 9 9950X)
from this repository. The toolchain pin is Rust **1.95** (`rust-toolchain.toml`).
Every timed worker is a fresh process wrapped in `scripts/with-memlimit.sh` at
{MEMORY_LIMIT_GIB}~GiB (`ulimit -v`, 90% of host RAM). The runner defaults are
**1 warmup + 3 measured** samples per cell; warmup rows are stored with
`warmup: true` and excluded from the median. Workers inherit
`RUSTFLAGS=-C target-cpu=native`. Isolated Cargo trees under `benchmarks/`
fetch the pinned git revisions (Plonky3, SP1, plonky2, Binius64, Flock,
ProveKit/whir) so they do not unify with the lattice workspace. Cargo fetches
those revisions on first build.

Non-interactive shells may not put Cargo on `PATH`; `source ~/.cargo/env`
is required in that case. `CARGO_NET_GIT_FETCH_WITH_CLI=true` avoids libgit2 auth
failures when fetching the pinned git dependencies.
Checked-in numbers live in `results/hash-x86_64/`.";

const REPRODUCTION_PROSE_LATEX: &str = "\
These tables were produced on a Linux x86\\_64 AVX-512 host (AMD Ryzen~9 9950X) from
this repository. The toolchain pin is Rust 1.95 (\\texttt{rust-toolchain.toml}).
Every timed worker is a fresh process wrapped in \\texttt{scripts/with-memlimit.sh}
at {MEMORY_LIMIT_GIB}~GiB (\\texttt{ulimit -v}, 90\\% of host RAM). The runner defaults
are 1 warmup and 3 measured samples per cell. Checked-in numbers live in
\\texttt{results/hash-x86\\_64/}.";

const REPRODUCTION_COMMANDS: &str = "\
# On an AVX-512 Linux x86_64 host
source \"$HOME/.cargo/env\"   # if cargo is not on PATH
cd /path/to/akita-benchmark

export CARGO_NET_GIT_FETCH_WITH_CLI=true
export RUSTFLAGS=\"-C target-cpu=native\"

./scripts/fetch-vendors.sh --akita   # Akita pin + nv=22/24 catalogs

# Full 90-cell matrix (9 schemes × 5 payloads × {1,8} threads)
./scripts/hash-eval.sh run --out results/hash-x86_64

# Rebuild Markdown + LaTeX from the JSONL already in that directory
cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval compare \\
  results/hash-x86_64 --out-dir results/hash-x86_64
";

const SANITY_PROSE_MARKDOWN: &str = "\
**Sanity-check the harness before trusting a full run.** `hash-eval matrix`
prints the 90-cell plan. A single supported cell should verify and emit JSON
with `status: ok`. Each sample the runner launches is equivalent to the worker
commands below (still under the 90%-of-RAM cap).";

const SANITY_PROSE_LATEX: &str = "\
\\noindent Sanity-check the harness before a full run.
\\texttt{hash-eval matrix} prints the 90-cell plan.
A single supported cell should verify and emit JSON with \\texttt{status: ok}.";

const SANITY_COMMANDS: &str = "\
export RUSTFLAGS=\"-C target-cpu=native\"

cargo test -p pcs-bench-core -p pcs-bench-runner --locked
cargo run -p pcs-bench-runner --bin pcs-bench -- hash-eval matrix

# One measured sample of a supported cell (payload 2^31, log2 N = 26, 1 thread)
./scripts/hash-eval.sh run --scheme akita --payload 31 --threads 1 --runs 1 --warmups 0
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
    use crate::hash::HashSchemeId;
    use crate::lattice::worker_memory_limit_bytes;
    use crate::observation::{HashRecord, Provenance, RunStatus, RESULT_SCHEMA_VERSION};
    use std::collections::BTreeMap;

    #[test]
    fn hash_report_names_the_machine_and_omits_hostnames() {
        let mut timings_ns = BTreeMap::new();
        timings_ns.insert("commit".into(), 159_000_000);
        timings_ns.insert("open".into(), 2_070_000_000);
        timings_ns.insert("verify".into(), 41_900_000);
        let record = HashRecord {
            schema_version: RESULT_SCHEMA_VERSION,
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
            historical: false,
            timings_ns,
            proof_bytes: Some(61_337),
            commitment_bytes: Some(343),
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
        let report = render_markdown_hash_eval_report(&[record]);
        assert!(report.contains("AMD Ryzen 9 9950X"));
        assert!(report.contains("AVX-512F"));
        assert!(report.contains("native"));
        assert!(report.contains("128-bit"));
        assert!(report.contains("unique decoding"));
        assert!(report.contains("WHIR"));
        assert!(report.contains("BaseFold"));
        assert!(report.contains("90-cell"));
        assert!(report.contains("results/hash-x86_64"));
        assert!(report.contains("Linux **x86_64**"));
        assert!(!report.contains("leopard"));
        assert!(report.contains("hash-eval matrix"));
        let host_ram = 121u64 * 1024 * 1024 * 1024;
        let _ = worker_memory_limit_bytes(host_ram);
        assert!(report.contains("90% of host RAM"));
    }
}
