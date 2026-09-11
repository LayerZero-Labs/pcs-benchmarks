//! PCS comparison CLI.

mod provenance;
mod worker;

use crate::provenance::ProvenanceExt;
use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use pcs_bench_core::{
    aggregate_hash_resource_rows, aggregate_hash_timing_rows, aggregate_resource_rows,
    aggregate_timing_rows, hash_case, hash_matrix, lattice_case, lattice_matrix,
    render_latex_eval_report, render_latex_hash_eval_report, render_latex_hash_resource_table,
    render_latex_hash_timing_table, render_latex_resource_table, render_latex_timing_table,
    render_markdown_eval_report, render_markdown_hash_eval_report,
    render_markdown_hash_resource_table, render_markdown_hash_timing_table,
    render_markdown_resource_table, render_markdown_timing_table, HashRecord, HashSchemeId,
    LatticeRecord, SchemeId, HASH_THREADS, PAYLOAD_LOG2,
};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "pcs-bench", about = "Reproducible PCS comparison harness")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Dense lattice PCS comparison (Akita, Akita offload, Greyhound, RoKoKo).
    LatticeEval {
        #[command(subcommand)]
        command: LatticeCommand,
    },
    /// Dense hash PCS comparison (Akita, WHIR, BaseFold).
    HashEval {
        #[command(subcommand)]
        command: HashCommand,
    },
}

#[derive(Subcommand)]
enum LatticeCommand {
    /// Print the planned comparison matrix without running it.
    Matrix,
    /// Run selected cells and write JSONL plus tables.
    Run(RunArgs),
    /// Rebuild tables from existing JSONL records.
    Compare(CompareArgs),
}

#[derive(Subcommand)]
enum HashCommand {
    /// Print the planned comparison matrix without running it.
    Matrix,
    /// Run selected cells and write JSONL plus tables.
    Run(HashRunArgs),
    /// Rebuild tables from existing JSONL records.
    Compare(CompareArgs),
}

#[derive(Clone, Copy, Default, ValueEnum)]
enum TableFormat {
    Markdown,
    Latex,
    #[default]
    Both,
}

#[derive(Clone, Copy, Default, ValueEnum)]
enum SeedMode {
    /// Use a different recorded workload seed for every process.
    #[default]
    Vary,
    /// Reuse one recorded seed to isolate machine/runtime noise.
    Fixed,
}

impl SeedMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Vary => "vary",
            Self::Fixed => "fixed",
        }
    }
}

#[derive(clap::Args)]
struct RunArgs {
    /// Comma-separated schemes: akita,akita-offload,greyhound,rokoko (default: all).
    #[arg(long, value_delimiter = ',')]
    scheme: Vec<String>,
    /// Comma-separated payload exponents (default: 27,29,31,33,35).
    #[arg(long, value_delimiter = ',')]
    payload: Vec<u32>,
    /// Measured processes per cell after warmup.
    #[arg(long, default_value_t = 10)]
    runs: u32,
    /// Discarded processes per cell.
    #[arg(long, default_value_t = 1)]
    warmups: u32,
    /// Workload variation policy: vary inputs per process or hold them fixed.
    #[arg(long, value_enum, default_value = "vary")]
    seed_mode: SeedMode,
    /// Output directory. Defaults to results/lattice-<utc>.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(clap::Args)]
struct HashRunArgs {
    /// Comma-separated schemes: akita,akita-fp64,akita-fp128,plonky2-fri,plonky3-fri,plonky3-stir,whir,binius64,flock,whir-provekit,basefold (default: all).
    #[arg(long, value_delimiter = ',')]
    scheme: Vec<String>,
    /// Comma-separated payload exponents (default: 27,29,31,33,35).
    #[arg(long, value_delimiter = ',')]
    payload: Vec<u32>,
    /// Comma-separated thread counts (default: 1,8).
    #[arg(long, value_delimiter = ',')]
    threads: Vec<u32>,
    /// Measured processes per cell after warmup.
    #[arg(long, default_value_t = 10)]
    runs: u32,
    /// Discarded processes per cell.
    #[arg(long, default_value_t = 1)]
    warmups: u32,
    /// Workload variation policy: vary inputs per process or hold them fixed.
    #[arg(long, value_enum, default_value = "vary")]
    seed_mode: SeedMode,
    /// Output directory. Defaults to results/hash-<utc>.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(clap::Args)]
struct CompareArgs {
    /// JSONL file or directory of `records.jsonl` files.
    input: PathBuf,
    #[arg(long, value_enum, default_value_t = TableFormat::Both)]
    format: TableFormat,
    #[arg(long)]
    out_dir: Option<PathBuf>,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::LatticeEval { command } => match command {
            LatticeCommand::Matrix => {
                print_matrix();
                Ok(())
            }
            LatticeCommand::Run(args) => run_lattice(args),
            LatticeCommand::Compare(args) => compare_lattice(args),
        },
        Command::HashEval { command } => match command {
            HashCommand::Matrix => {
                print_hash_matrix();
                Ok(())
            }
            HashCommand::Run(args) => run_hash(args),
            HashCommand::Compare(args) => compare_hash(args),
        },
    }
}

fn print_matrix() {
    println!("payload\tscheme\tfield\tlog2_n\tparam\tnotes");
    for case in lattice_matrix() {
        println!(
            "2^{}\t{}\t{}\t{}\t{}\t{}",
            case.payload_log2,
            case.scheme.display_name(),
            case.field.name,
            case.log2_n
                .map_or_else(|| "—".into(), |value| value.to_string()),
            case.native_param.unwrap_or("—"),
            case.unsupported_reason.unwrap_or(""),
        );
    }
}

fn run_lattice(args: RunArgs) -> Result<()> {
    reject_negative_check_performance_run()?;
    let schemes = parse_schemes(&args.scheme)?;
    let payloads = if args.payload.is_empty() {
        PAYLOAD_LOG2.to_vec()
    } else {
        args.payload.clone()
    };
    let mut cases = Vec::new();
    for payload in payloads {
        for scheme in &schemes {
            cases.push(
                lattice_case(payload, *scheme)
                    .with_context(|| format!("unknown payload 2^{payload} for {scheme:?}"))?,
            );
        }
    }
    let mut prepared = std::collections::BTreeSet::new();
    for case in &cases {
        if case.log2_n.is_some() && prepared.insert((case.scheme, case.native_param)) {
            worker::prepare_lattice_case(case).with_context(|| {
                format!(
                    "prepare {} worker for payload 2^{}",
                    case.scheme.display_name(),
                    case.payload_log2
                )
            })?;
        }
    }

    let out_dir = match args.out {
        Some(path) => path,
        None => default_out_dir()?,
    };
    let mut provenance = provenance::capture()?;
    provenance.seed_mode = Some(args.seed_mode.as_str().into());
    fs::create_dir_all(&out_dir)?;
    let records_path = out_dir.join("records.jsonl");
    let mut records_file = create_records_file(&records_path)?;
    provenance.write(&out_dir.join("provenance.txt"))?;

    let mut records = Vec::new();
    for case in cases {
        let samples = args.warmups.saturating_add(args.runs);
        for sample in 0..samples {
            let warmup = sample < args.warmups;
            eprintln!(
                "[{} 2^{} {} sample {}{}]",
                case.scheme.token(),
                case.payload_log2,
                case.native_param.unwrap_or("-"),
                sample,
                if warmup { " warmup" } else { "" }
            );
            let seed = workload_seed(args.seed_mode, case.payload_log2, sample);
            let record = worker::run_case(&case, sample, warmup, seed, provenance.clone());
            serde_json::to_writer(&mut records_file, &record)?;
            records_file.write_all(b"\n")?;
            records_file.flush()?;
            records.push(record);
        }
    }

    write_tables(&out_dir, &records)?;
    eprintln!("records: {}", records_path.display());
    eprintln!("markdown: {}", out_dir.join("table.md").display());
    eprintln!("latex: {}", out_dir.join("table.tex").display());
    eprintln!("report: {}", out_dir.join("report.md").display());
    Ok(())
}

fn compare_lattice(args: CompareArgs) -> Result<()> {
    let records = load_records(&args.input)?;
    if records.is_empty() {
        bail!("no lattice records in {}", args.input.display());
    }
    validate_lattice_cohorts(&records)?;
    let out_dir = args.out_dir.unwrap_or_else(|| {
        args.input
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    });
    fs::create_dir_all(&out_dir)?;
    let rows = aggregate_timing_rows(&records);
    match args.format {
        TableFormat::Markdown => {
            fs::write(
                out_dir.join("table.md"),
                render_markdown_timing_table(&rows),
            )?;
            fs::write(
                out_dir.join("report.md"),
                render_markdown_eval_report(&records),
            )?;
        }
        TableFormat::Latex => {
            fs::write(out_dir.join("table.tex"), render_latex_timing_table(&rows))?;
            fs::write(
                out_dir.join("report.tex"),
                render_latex_eval_report(&records),
            )?;
        }
        TableFormat::Both => write_tables(&out_dir, &records)?,
    }
    println!("{}", render_markdown_eval_report(&records));
    Ok(())
}

fn write_tables(out_dir: &Path, records: &[LatticeRecord]) -> Result<()> {
    validate_lattice_records(records, Path::new("in-memory lattice records"))?;
    validate_lattice_seed_schedule(records)?;
    validate_lattice_build_identities(records)?;
    validate_lattice_cohorts(records)?;
    let rows = aggregate_timing_rows(records);
    let resources = aggregate_resource_rows(records);
    fs::write(
        out_dir.join("table.md"),
        render_markdown_timing_table(&rows),
    )?;
    fs::write(
        out_dir.join("table-resources.md"),
        render_markdown_resource_table(&resources),
    )?;
    fs::write(out_dir.join("table.tex"), render_latex_timing_table(&rows))?;
    fs::write(
        out_dir.join("table-resources.tex"),
        render_latex_resource_table(&resources),
    )?;
    fs::write(
        out_dir.join("report.md"),
        render_markdown_eval_report(records),
    )?;
    fs::write(
        out_dir.join("report.tex"),
        render_latex_eval_report(records),
    )?;
    Ok(())
}

fn print_hash_matrix() {
    println!("payload\tscheme\tfield\tlog2_n\tthreads\tparam");
    for case in hash_matrix() {
        println!(
            "2^{}\t{}\t{}\t{}\t{}\t{}",
            case.payload_log2,
            case.scheme.display_name(),
            case.field.name,
            case.log2_n,
            case.threads,
            case.native_param,
        );
    }
}

fn run_hash(args: HashRunArgs) -> Result<()> {
    reject_negative_check_performance_run()?;
    let schemes = parse_hash_schemes(&args.scheme)?;
    let payloads = if args.payload.is_empty() {
        PAYLOAD_LOG2.to_vec()
    } else {
        args.payload.clone()
    };
    let threads = if args.threads.is_empty() {
        HASH_THREADS.to_vec()
    } else {
        args.threads.clone()
    };
    let mut cases = Vec::new();
    for payload in payloads {
        for scheme in &schemes {
            for thread_count in &threads {
                cases.push(hash_case(payload, *scheme, *thread_count).with_context(|| {
                    format!("unknown payload 2^{payload} for {scheme:?} threads={thread_count}")
                })?);
            }
        }
    }
    let mut prepared = std::collections::BTreeSet::new();
    for case in &cases {
        if prepared.insert(case.scheme) {
            worker::prepare_hash_case(case).with_context(|| {
                format!(
                    "prepare {} worker for payload 2^{}",
                    case.scheme.display_name(),
                    case.payload_log2
                )
            })?;
        }
    }

    let out_dir = match args.out {
        Some(path) => path,
        None => default_hash_out_dir()?,
    };
    let mut provenance = provenance::capture()?;
    provenance.seed_mode = Some(args.seed_mode.as_str().into());
    fs::create_dir_all(&out_dir)?;
    let records_path = out_dir.join("records.jsonl");
    let mut records_file = create_records_file(&records_path)?;
    provenance.write_hash(&out_dir.join("provenance.txt"))?;

    let mut records = Vec::new();
    for case in cases {
        let samples = args.warmups.saturating_add(args.runs);
        for sample in 0..samples {
            let warmup = sample < args.warmups;
            eprintln!(
                "[{} 2^{} {} t{} sample {}{}]",
                case.scheme.token(),
                case.payload_log2,
                case.native_param,
                case.threads,
                sample,
                if warmup { " warmup" } else { "" }
            );
            let seed = workload_seed(args.seed_mode, case.payload_log2, sample);
            let record = worker::run_hash_case(&case, sample, warmup, seed, provenance.clone());
            serde_json::to_writer(&mut records_file, &record)?;
            records_file.write_all(b"\n")?;
            records_file.flush()?;
            records.push(record);
        }
    }

    write_hash_tables(&out_dir, &records)?;
    eprintln!("records: {}", records_path.display());
    eprintln!("markdown: {}", out_dir.join("table.md").display());
    eprintln!(
        "markdown resources: {}",
        out_dir.join("table-resources.md").display()
    );
    eprintln!("latex: {}", out_dir.join("table.tex").display());
    eprintln!(
        "latex resources: {}",
        out_dir.join("table-resources.tex").display()
    );
    eprintln!("report: {}", out_dir.join("report.md").display());
    Ok(())
}

fn create_records_file(path: &Path) -> Result<File> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| {
            format!(
                "create new records file {}; refusing to overwrite an existing run",
                path.display()
            )
        })
}

fn reject_negative_check_performance_run() -> Result<()> {
    if std::env::var("PCS_BENCH_NEGATIVE_CHECK").as_deref() == Ok("1") {
        bail!(
            "PCS_BENCH_NEGATIVE_CHECK=1 is a correctness-only worker mode and cannot be persisted as a performance run"
        );
    }
    Ok(())
}

const fn workload_seed(mode: SeedMode, payload_log2: u32, sample: u32) -> u64 {
    let base = 0x5043_5342_454e_4348u64 ^ ((payload_log2 as u64) << 32);
    match mode {
        SeedMode::Vary => base ^ sample as u64,
        SeedMode::Fixed => base,
    }
}

fn validate_cohort<'a>(
    label: &str,
    mut provenances: impl Iterator<Item = &'a pcs_bench_core::Provenance>,
) -> Result<()> {
    let Some(expected) = provenances.next() else {
        return Ok(());
    };
    for candidate in provenances {
        if let Some(field) = cohort_mismatch(expected, candidate) {
            bail!(
                "{label} records do not form one comparable cohort: `{field}` differs; split the records into separate reports"
            );
        }
    }
    Ok(())
}

fn validate_lattice_cohorts(records: &[LatticeRecord]) -> Result<()> {
    validate_environment("lattice", records.iter().map(|record| &record.provenance))?;
    for scheme in SchemeId::all() {
        validate_cohort(
            &format!("lattice {}", scheme.token()),
            records
                .iter()
                .filter(|record| record.scheme == scheme)
                .map(|record| &record.provenance),
        )?;
    }
    Ok(())
}

fn validate_environment<'a>(
    label: &str,
    mut provenances: impl Iterator<Item = &'a pcs_bench_core::Provenance>,
) -> Result<()> {
    let Some(expected) = provenances.next() else {
        return Ok(());
    };
    for candidate in provenances {
        if let Some(field) = environment_mismatch(expected, candidate) {
            bail!(
                "{label} records were not measured in one comparable environment: `{field}` differs"
            );
        }
    }
    Ok(())
}

fn cohort_mismatch(
    expected: &pcs_bench_core::Provenance,
    candidate: &pcs_bench_core::Provenance,
) -> Option<&'static str> {
    if expected.harness_revision != candidate.harness_revision {
        Some("harness_revision")
    } else if expected.timestamp_utc != candidate.timestamp_utc {
        Some("timestamp_utc")
    } else if expected.run_command != candidate.run_command {
        Some("run_command")
    } else {
        environment_mismatch(expected, candidate)
    }
}

fn environment_mismatch(
    expected: &pcs_bench_core::Provenance,
    candidate: &pcs_bench_core::Provenance,
) -> Option<&'static str> {
    if expected.rustc_version != candidate.rustc_version {
        Some("rustc_version")
    } else if expected.target != candidate.target {
        Some("target")
    } else if expected.cpu_model != candidate.cpu_model {
        Some("cpu_model")
    } else if expected.cpu_scaling_driver != candidate.cpu_scaling_driver {
        Some("cpu_scaling_driver")
    } else if expected.cpu_governor != candidate.cpu_governor {
        Some("cpu_governor")
    } else if expected.cpu_energy_preference != candidate.cpu_energy_preference {
        Some("cpu_energy_preference")
    } else if expected.machine_id_hash != candidate.machine_id_hash {
        Some("machine_id_hash")
    } else if expected.logical_cpus != candidate.logical_cpus {
        Some("logical_cpus")
    } else if expected.memory_bytes != candidate.memory_bytes {
        Some("memory_bytes")
    } else if expected.memory_limit_bytes != candidate.memory_limit_bytes {
        Some("memory_limit_bytes")
    } else if expected.rustflags != candidate.rustflags {
        Some("rustflags")
    } else if expected.avx512 != candidate.avx512 {
        Some("avx512")
    } else if expected.isa_notes != candidate.isa_notes {
        Some("isa_notes")
    } else if expected.seed_mode != candidate.seed_mode {
        Some("seed_mode")
    } else {
        None
    }
}

fn compare_hash(args: CompareArgs) -> Result<()> {
    let records = load_hash_records(&args.input)?;
    if records.is_empty() {
        bail!("no hash records in {}", args.input.display());
    }
    validate_cohort("hash", records.iter().map(|record| &record.provenance))?;
    let out_dir = args.out_dir.unwrap_or_else(|| {
        args.input
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    });
    fs::create_dir_all(&out_dir)?;
    match args.format {
        TableFormat::Markdown => {
            write_hash_markdown(&out_dir, &records)?;
        }
        TableFormat::Latex => {
            write_hash_latex(&out_dir, &records)?;
        }
        TableFormat::Both => write_hash_tables(&out_dir, &records)?,
    }
    println!("{}", render_markdown_hash_eval_report(&records));
    Ok(())
}

fn write_hash_tables(out_dir: &Path, records: &[HashRecord]) -> Result<()> {
    validate_hash_records(records, Path::new("in-memory hash records"))?;
    validate_hash_seed_schedule(records)?;
    validate_hash_build_identities(records)?;
    validate_cohort("hash", records.iter().map(|record| &record.provenance))?;
    write_hash_markdown(out_dir, records)?;
    write_hash_latex(out_dir, records)?;
    Ok(())
}

fn write_hash_markdown(out_dir: &Path, records: &[HashRecord]) -> Result<()> {
    let timing = aggregate_hash_timing_rows(records);
    let resources = aggregate_hash_resource_rows(records);
    fs::write(
        out_dir.join("table.md"),
        render_markdown_hash_timing_table(&timing),
    )?;
    fs::write(
        out_dir.join("table-resources.md"),
        render_markdown_hash_resource_table(&resources),
    )?;
    fs::write(
        out_dir.join("report.md"),
        render_markdown_hash_eval_report(records),
    )?;
    Ok(())
}

fn write_hash_latex(out_dir: &Path, records: &[HashRecord]) -> Result<()> {
    let timing = aggregate_hash_timing_rows(records);
    let resources = aggregate_hash_resource_rows(records);
    fs::write(
        out_dir.join("table.tex"),
        render_latex_hash_timing_table(&timing),
    )?;
    fs::write(
        out_dir.join("table-resources.tex"),
        render_latex_hash_resource_table(&resources),
    )?;
    fs::write(
        out_dir.join("report.tex"),
        render_latex_hash_eval_report(records),
    )?;
    Ok(())
}

fn parse_schemes(tokens: &[String]) -> Result<Vec<SchemeId>> {
    if tokens.is_empty() {
        return Ok(SchemeId::all().to_vec());
    }
    let mut schemes = Vec::new();
    for token in tokens {
        let scheme =
            SchemeId::parse_token(token).with_context(|| format!("unknown scheme '{token}'"))?;
        if !schemes.contains(&scheme) {
            schemes.push(scheme);
        }
    }
    Ok(schemes)
}

fn parse_hash_schemes(tokens: &[String]) -> Result<Vec<HashSchemeId>> {
    if tokens.is_empty() {
        return Ok(HashSchemeId::all().to_vec());
    }
    let mut schemes = Vec::new();
    for token in tokens {
        let scheme = HashSchemeId::parse_token(token)
            .with_context(|| format!("unknown hash scheme '{token}'"))?;
        if !schemes.contains(&scheme) {
            schemes.push(scheme);
        }
    }
    Ok(schemes)
}

fn default_out_dir() -> Result<PathBuf> {
    let stamp = timestamp_utc()?;
    Ok(workspace_root()?
        .join("results")
        .join(format!("lattice-{stamp}")))
}

fn default_hash_out_dir() -> Result<PathBuf> {
    let stamp = timestamp_utc()?;
    Ok(workspace_root()?
        .join("results")
        .join(format!("hash-{stamp}")))
}

fn timestamp_utc() -> Result<String> {
    let output = std::process::Command::new("date")
        .args(["-u", "+%Y%m%dT%H%M%SZ"])
        .output()
        .context("date")?;
    if !output.status.success() {
        bail!("date failed");
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn workspace_root() -> Result<PathBuf> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?)
}

fn load_records(path: &Path) -> Result<Vec<LatticeRecord>> {
    let records = if path.is_dir() {
        let mut records = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let nested = entry.path();
            if nested.file_name().and_then(|name| name.to_str()) == Some("records.jsonl")
                || nested.extension().and_then(|ext| ext.to_str()) == Some("jsonl")
            {
                records.extend(load_jsonl(&nested)?);
            }
        }
        records
    } else {
        load_jsonl(path)?
    };
    validate_unique_lattice_records(&records)?;
    validate_lattice_seed_schedule(&records)?;
    validate_lattice_build_identities(&records)?;
    Ok(records)
}

fn load_jsonl(path: &Path) -> Result<Vec<LatticeRecord>> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut records = Vec::new();
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: LatticeRecord = serde_json::from_str(&line)
            .with_context(|| format!("{}:{}", path.display(), index + 1))?;
        validate_record_timings(record.status, &record.timings_ns, path, index + 1)?;
        validate_record_communication(
            record.status,
            record.proof_bytes,
            record.commitment_bytes,
            record.evaluation_bytes,
            record.public_context_bytes,
            path,
            index + 1,
        )?;
        validate_lattice_record_case(&record, path, index + 1)?;
        validate_record_provenance(
            record.status,
            &record.provenance,
            record.scheme != SchemeId::Greyhound,
            path,
            index + 1,
        )?;
        records.push(record);
    }
    Ok(records)
}

fn load_hash_records(path: &Path) -> Result<Vec<HashRecord>> {
    let records = if path.is_dir() {
        let mut records = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let nested = entry.path();
            if nested.file_name().and_then(|name| name.to_str()) == Some("records.jsonl")
                || nested.extension().and_then(|ext| ext.to_str()) == Some("jsonl")
            {
                records.extend(load_hash_jsonl(&nested)?);
            }
        }
        records
    } else {
        load_hash_jsonl(path)?
    };
    validate_unique_hash_records(&records)?;
    validate_hash_seed_schedule(&records)?;
    validate_hash_build_identities(&records)?;
    Ok(records)
}

fn load_hash_jsonl(path: &Path) -> Result<Vec<HashRecord>> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut records = Vec::new();
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: HashRecord = serde_json::from_str(&line)
            .with_context(|| format!("{}:{}", path.display(), index + 1))?;
        validate_record_timings(record.status, &record.timings_ns, path, index + 1)?;
        validate_record_communication(
            record.status,
            record.proof_bytes,
            record.commitment_bytes,
            record.evaluation_bytes,
            record.public_context_bytes,
            path,
            index + 1,
        )?;
        validate_hash_record_case(&record, path, index + 1)?;
        validate_record_provenance(record.status, &record.provenance, true, path, index + 1)?;
        records.push(record);
    }
    Ok(records)
}

fn validate_record_timings(
    status: pcs_bench_core::RunStatus,
    timings: &std::collections::BTreeMap<String, u64>,
    path: &Path,
    line: usize,
) -> Result<()> {
    if status != pcs_bench_core::RunStatus::Ok {
        return Ok(());
    }
    for phase in ["commit", "open", "verify"] {
        if !timings.contains_key(phase) {
            bail!(
                "{}:{line}: successful record is missing required phase `{phase}`",
                path.display()
            );
        }
    }
    Ok(())
}

fn validate_lattice_records(records: &[LatticeRecord], path: &Path) -> Result<()> {
    for (index, record) in records.iter().enumerate() {
        let line = index + 1;
        validate_record_timings(record.status, &record.timings_ns, path, line)?;
        validate_record_communication(
            record.status,
            record.proof_bytes,
            record.commitment_bytes,
            record.evaluation_bytes,
            record.public_context_bytes,
            path,
            line,
        )?;
        validate_lattice_record_case(record, path, line)?;
        validate_record_provenance(
            record.status,
            &record.provenance,
            record.scheme != SchemeId::Greyhound,
            path,
            line,
        )?;
    }
    Ok(())
}

fn validate_hash_records(records: &[HashRecord], path: &Path) -> Result<()> {
    for (index, record) in records.iter().enumerate() {
        let line = index + 1;
        validate_record_timings(record.status, &record.timings_ns, path, line)?;
        validate_record_communication(
            record.status,
            record.proof_bytes,
            record.commitment_bytes,
            record.evaluation_bytes,
            record.public_context_bytes,
            path,
            line,
        )?;
        validate_hash_record_case(record, path, line)?;
        validate_record_provenance(record.status, &record.provenance, true, path, line)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_record_communication(
    status: pcs_bench_core::RunStatus,
    proof_bytes: Option<u64>,
    commitment_bytes: Option<u64>,
    evaluation_bytes: Option<u64>,
    public_context_bytes: Option<u64>,
    path: &Path,
    line: usize,
) -> Result<()> {
    if status != pcs_bench_core::RunStatus::Ok {
        return Ok(());
    }
    for (field, value) in [
        ("proof_bytes", proof_bytes),
        ("commitment_bytes", commitment_bytes),
        ("evaluation_bytes", evaluation_bytes),
        ("public_context_bytes", public_context_bytes),
    ] {
        if value.is_none() {
            bail!(
                "{}:{line}: successful record is missing required field `{field}`",
                path.display()
            );
        }
    }
    Ok(())
}

fn validate_record_provenance(
    status: pcs_bench_core::RunStatus,
    provenance: &pcs_bench_core::Provenance,
    requires_lockfile: bool,
    path: &Path,
    line: usize,
) -> Result<()> {
    for (field, value) in [
        (
            "harness_revision",
            Some(provenance.harness_revision.as_str()),
        ),
        ("rustc_version", Some(provenance.rustc_version.as_str())),
        ("target", Some(provenance.target.as_str())),
        ("cpu_model", Some(provenance.cpu_model.as_str())),
        ("isa_notes", Some(provenance.isa_notes.as_str())),
        ("timestamp_utc", provenance.timestamp_utc.as_deref()),
        ("run_command", provenance.run_command.as_deref()),
        ("machine_id_hash", provenance.machine_id_hash.as_deref()),
    ] {
        if value.is_none_or(str::is_empty) {
            bail!(
                "{}:{line}: record is missing required provenance `{field}`",
                path.display()
            );
        }
    }
    if provenance.logical_cpus == 0
        || provenance.memory_bytes.is_none_or(|bytes| bytes == 0)
        || provenance.memory_limit_bytes.is_none_or(|bytes| bytes == 0)
    {
        bail!(
            "{}:{line}: record has incomplete machine resource provenance",
            path.display()
        );
    }
    if provenance.target.contains("Linux")
        && provenance.target.contains("x86_64")
        && provenance.cpu_governor.as_deref().is_none_or(str::is_empty)
    {
        bail!(
            "{}:{line}: Linux x86_64 record is missing CPU governor provenance",
            path.display()
        );
    }
    if status != pcs_bench_core::RunStatus::Ok {
        return Ok(());
    }
    for (field, value) in [
        ("executable_sha256", provenance.executable_sha256.as_deref()),
        (
            "worker_compiler_version",
            provenance.worker_compiler_version.as_deref(),
        ),
        ("build_command", provenance.build_command.as_deref()),
    ] {
        if value.is_none_or(str::is_empty) {
            bail!(
                "{}:{line}: successful record is missing build provenance `{field}`",
                path.display()
            );
        }
    }
    if requires_lockfile
        && provenance
            .lockfile_sha256
            .as_deref()
            .is_none_or(str::is_empty)
    {
        bail!(
            "{}:{line}: successful record is missing build provenance `lockfile_sha256`",
            path.display()
        );
    }
    Ok(())
}

fn validate_lattice_record_case(record: &LatticeRecord, path: &Path, line: usize) -> Result<()> {
    let expected = lattice_case(record.payload_log2, record.scheme).with_context(|| {
        format!(
            "{}:{line}: no canonical lattice case for {} payload=2^{}",
            path.display(),
            record.scheme.display_name(),
            record.payload_log2
        )
    })?;
    if record.log2_n != expected.log2_n
        || record.implementation_revision != record.scheme.revision()
        || record.field != expected.field.name
        || record.native_param.as_deref() != expected.native_param
        || record.threads != pcs_bench_core::THREADS_LATTICE_EVAL
        || record.provenance.threads != record.threads
    {
        bail!(
            "{}:{line}: lattice record does not match canonical matrix case",
            path.display()
        );
    }
    Ok(())
}

fn validate_hash_record_case(record: &HashRecord, path: &Path, line: usize) -> Result<()> {
    let expected =
        hash_case(record.payload_log2, record.scheme, record.threads).with_context(|| {
            format!(
                "{}:{line}: no canonical hash case for {} payload=2^{} threads={}",
                path.display(),
                record.scheme.display_name(),
                record.payload_log2,
                record.threads
            )
        })?;
    if record.log2_n != Some(expected.log2_n)
        || record.implementation_revision != record.scheme.revision()
        || record.field != expected.field.name
        || record.native_param.as_deref() != Some(expected.native_param)
        || record.provenance.threads != record.threads
    {
        bail!(
            "{}:{line}: hash record does not match canonical matrix case",
            path.display()
        );
    }
    Ok(())
}

fn validate_unique_lattice_records(records: &[LatticeRecord]) -> Result<()> {
    let mut seen = std::collections::BTreeSet::new();
    for record in records {
        let identity = (
            record.scheme,
            record.payload_log2,
            record.threads,
            record.sample,
            record.warmup,
            record.provenance.harness_revision.as_str(),
            record.provenance.executable_sha256.as_deref(),
        );
        if !seen.insert(identity) {
            bail!(
                "duplicate lattice observation: {} payload=2^{} threads={} sample={} warmup={}",
                record.scheme.display_name(),
                record.payload_log2,
                record.threads,
                record.sample,
                record.warmup
            );
        }
    }
    Ok(())
}

fn validate_unique_hash_records(records: &[HashRecord]) -> Result<()> {
    let mut seen = std::collections::BTreeSet::new();
    for record in records {
        let identity = (
            record.scheme,
            record.payload_log2,
            record.threads,
            record.sample,
            record.warmup,
            record.provenance.harness_revision.as_str(),
            record.provenance.executable_sha256.as_deref(),
        );
        if !seen.insert(identity) {
            bail!(
                "duplicate hash observation: {} payload=2^{} threads={} sample={} warmup={}",
                record.scheme.display_name(),
                record.payload_log2,
                record.threads,
                record.sample,
                record.warmup
            );
        }
    }
    Ok(())
}

fn validate_lattice_seed_schedule(records: &[LatticeRecord]) -> Result<()> {
    for record in records {
        validate_record_seed(
            record.payload_log2,
            record.sample,
            &record.provenance,
            record.scheme.display_name(),
        )?;
    }
    Ok(())
}

fn validate_hash_seed_schedule(records: &[HashRecord]) -> Result<()> {
    for record in records {
        validate_record_seed(
            record.payload_log2,
            record.sample,
            &record.provenance,
            record.scheme.display_name(),
        )?;
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BuildIdentity {
    implementation_revision: String,
    executable_sha256: Option<String>,
    lockfile_sha256: Option<String>,
    worker_compiler_version: Option<String>,
    build_command: Option<String>,
}

fn record_build_identity(
    implementation_revision: &str,
    provenance: &pcs_bench_core::Provenance,
) -> BuildIdentity {
    BuildIdentity {
        implementation_revision: implementation_revision.to_owned(),
        executable_sha256: provenance.executable_sha256.clone(),
        lockfile_sha256: provenance.lockfile_sha256.clone(),
        worker_compiler_version: provenance.worker_compiler_version.clone(),
        build_command: provenance.build_command.clone(),
    }
}

fn validate_lattice_build_identities(records: &[LatticeRecord]) -> Result<()> {
    let entries = records
        .iter()
        .filter(|record| record.status == pcs_bench_core::RunStatus::Ok)
        .map(|record| {
            let group = match record.scheme {
                SchemeId::Akita | SchemeId::AkitaOffload => "akita".to_owned(),
                SchemeId::Greyhound => "greyhound".to_owned(),
                SchemeId::Rokoko => format!(
                    "rokoko:{}",
                    record.native_param.as_deref().unwrap_or("unknown")
                ),
            };
            (
                group,
                record_build_identity(&record.implementation_revision, &record.provenance),
            )
        });
    validate_build_identity_groups("lattice", entries)
}

fn validate_hash_build_identities(records: &[HashRecord]) -> Result<()> {
    let entries = records
        .iter()
        .filter(|record| record.status == pcs_bench_core::RunStatus::Ok)
        .map(|record| {
            let group = match record.scheme {
                HashSchemeId::Akita | HashSchemeId::AkitaFp64 | HashSchemeId::AkitaFp128 => {
                    "akita".to_owned()
                }
                other => other.token().to_owned(),
            };
            (
                group,
                record_build_identity(&record.implementation_revision, &record.provenance),
            )
        });
    validate_build_identity_groups("hash", entries)
}

fn validate_build_identity_groups(
    label: &str,
    entries: impl Iterator<Item = (String, BuildIdentity)>,
) -> Result<()> {
    let mut identities = std::collections::BTreeMap::new();
    for (group, identity) in entries {
        if let Some(expected) = identities.get(&group) {
            if expected != &identity {
                bail!(
                    "{label} worker `{group}` used multiple implementation/build identities; split the report"
                );
            }
        } else {
            identities.insert(group, identity);
        }
    }
    Ok(())
}

fn validate_record_seed(
    payload_log2: u32,
    sample: u32,
    provenance: &pcs_bench_core::Provenance,
    label: &str,
) -> Result<()> {
    let mode = match provenance.seed_mode.as_deref() {
        Some("vary") => SeedMode::Vary,
        Some("fixed") => SeedMode::Fixed,
        Some(other) => bail!("{label} record has unknown seed mode `{other}`"),
        None => bail!("{label} record is missing its seed mode"),
    };
    let expected = workload_seed(mode, payload_log2, sample);
    if provenance.workload_seed != Some(expected) {
        bail!(
            "{label} record has workload seed {:?}; expected {expected} for payload=2^{payload_log2} sample={sample}",
            provenance.workload_seed
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        validate_build_identity_groups, validate_cohort, validate_environment,
        validate_hash_record_case, validate_record_seed, validate_record_timings,
        validate_unique_hash_records, workload_seed, BuildIdentity, SeedMode,
    };
    use pcs_bench_core::{HashRecord, HashSchemeId, Provenance, RunStatus};
    use std::collections::BTreeMap;
    use std::path::Path;

    #[test]
    fn rejects_cross_machine_report_cohort() {
        let one = Provenance::test_fixture();
        let mut two = one.clone();
        two.cpu_model = "different machine".into();
        let error =
            validate_cohort("test", [&one, &two].into_iter()).expect_err("must reject mismatch");
        assert!(error.to_string().contains("cpu_model"));
    }

    #[test]
    fn accepts_distinct_runs_in_the_same_environment() {
        let one = Provenance::test_fixture();
        let mut two = one.clone();
        two.harness_revision = "refreshed".into();
        two.timestamp_utc = Some("later".into());
        two.run_command = Some("lattice-eval run --scheme rokoko".into());
        validate_environment("test", [&one, &two].into_iter()).expect("same environment");
    }

    #[test]
    fn rejects_imported_success_with_missing_phase() {
        let mut timings = BTreeMap::new();
        timings.insert("commit".into(), 1);
        timings.insert("open".into(), 2);
        let error = validate_record_timings(RunStatus::Ok, &timings, Path::new("records.jsonl"), 7)
            .expect_err("must reject incomplete success");
        assert!(error.to_string().contains("required phase `verify`"));
    }

    #[test]
    fn rejects_imported_record_with_wrong_case_identity() {
        let record = HashRecord {
            status: RunStatus::Ok,
            status_detail: None,
            scheme: HashSchemeId::Binius64,
            implementation_revision: HashSchemeId::Binius64.revision().into(),
            payload_log2: 27,
            log2_n: Some(19),
            field: "F_{2^{128}}".into(),
            native_param: Some("binius64-basefold-100".into()),
            threads: 1,
            sample: 0,
            warmup: false,
            timings_ns: BTreeMap::from([
                ("commit".into(), 1),
                ("open".into(), 2),
                ("verify".into(), 3),
            ]),
            proof_bytes: None,
            commitment_bytes: None,
            evaluation_bytes: None,
            public_context_bytes: None,
            state_bytes: None,
            peak_rss_bytes: None,
            provenance: Provenance::test_fixture(),
        };
        let error = validate_hash_record_case(&record, Path::new("records.jsonl"), 1)
            .expect_err("wrong log2_n must be rejected");
        assert!(error.to_string().contains("canonical matrix case"));

        let mut thread_mismatch = record.clone();
        thread_mismatch.log2_n = Some(20);
        thread_mismatch.threads = 8;
        let error = validate_hash_record_case(&thread_mismatch, Path::new("records.jsonl"), 1)
            .expect_err("thread provenance mismatch must be rejected");
        assert!(error.to_string().contains("canonical matrix case"));

        let mut same_coordinate = record.clone();
        same_coordinate.provenance.workload_seed = Some(99);
        let error = validate_unique_hash_records(&[record, same_coordinate])
            .expect_err("sample coordinate must remain unique across seeds");
        assert!(error.to_string().contains("duplicate hash observation"));
    }

    #[test]
    fn rejects_seed_that_does_not_match_declared_mode() {
        let mut provenance = Provenance::test_fixture();
        provenance.seed_mode = Some("vary".into());
        provenance.workload_seed = Some(7);
        let error =
            validate_record_seed(27, 3, &provenance, "test").expect_err("wrong seed must fail");
        assert!(error.to_string().contains("expected"));

        provenance.workload_seed = Some(workload_seed(SeedMode::Vary, 27, 3));
        validate_record_seed(27, 3, &provenance, "test").expect("scheduled seed");
    }

    #[test]
    fn rejects_multiple_builds_for_one_worker_group() {
        let one = BuildIdentity {
            implementation_revision: "revision".into(),
            executable_sha256: Some("one".into()),
            lockfile_sha256: Some("lock".into()),
            worker_compiler_version: Some("compiler".into()),
            build_command: Some("build".into()),
        };
        let mut two = one.clone();
        two.executable_sha256 = Some("two".into());
        let error = validate_build_identity_groups(
            "test",
            [("worker".into(), one), ("worker".into(), two)].into_iter(),
        )
        .expect_err("mixed build identities must be rejected");
        assert!(error
            .to_string()
            .contains("multiple implementation/build identities"));
    }
}
