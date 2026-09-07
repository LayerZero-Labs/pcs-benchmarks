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
use std::fs::{self, File};
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
    /// Dense lattice PCS comparison (Akita, Greyhound, RoKoKo).
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

#[derive(clap::Args)]
struct RunArgs {
    /// Comma-separated schemes: akita,greyhound,rokoko (default: all).
    #[arg(long, value_delimiter = ',')]
    scheme: Vec<String>,
    /// Comma-separated payload exponents (default: 27,29,31,33,35).
    #[arg(long, value_delimiter = ',')]
    payload: Vec<u32>,
    /// Measured processes per cell after warmup.
    #[arg(long, default_value_t = 3)]
    runs: u32,
    /// Discarded processes per cell.
    #[arg(long, default_value_t = 1)]
    warmups: u32,
    /// Output directory. Defaults to results/lattice-<utc>.
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(clap::Args)]
struct HashRunArgs {
    /// Comma-separated schemes: akita,plonky2-fri,plonky3-fri,plonky3-stir,whir,binius64,flock,whir-provekit,basefold (default: all).
    #[arg(long, value_delimiter = ',')]
    scheme: Vec<String>,
    /// Comma-separated payload exponents (default: 27,29,31,33,35).
    #[arg(long, value_delimiter = ',')]
    payload: Vec<u32>,
    /// Comma-separated thread counts (default: 1,8).
    #[arg(long, value_delimiter = ',')]
    threads: Vec<u32>,
    /// Measured processes per cell after warmup.
    #[arg(long, default_value_t = 3)]
    runs: u32,
    /// Discarded processes per cell.
    #[arg(long, default_value_t = 1)]
    warmups: u32,
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
    let schemes = parse_schemes(&args.scheme)?;
    let payloads = if args.payload.is_empty() {
        PAYLOAD_LOG2.to_vec()
    } else {
        args.payload.clone()
    };
    let out_dir = match args.out {
        Some(path) => path,
        None => default_out_dir()?,
    };
    fs::create_dir_all(&out_dir)?;
    let records_path = out_dir.join("records.jsonl");
    let mut records_file = File::create(&records_path)?;
    let provenance = provenance::capture()?;
    provenance.write(&out_dir.join("provenance.txt"))?;

    let mut records = Vec::new();
    for payload in payloads {
        for scheme in &schemes {
            let Some(case) = lattice_case(payload, *scheme) else {
                bail!("unknown payload 2^{payload} for {scheme:?}");
            };
            let samples = args.warmups.saturating_add(args.runs);
            for sample in 0..samples {
                let warmup = sample < args.warmups;
                eprintln!(
                    "[{} 2^{} {} sample {}{}]",
                    case.scheme.token(),
                    payload,
                    case.native_param.unwrap_or("-"),
                    sample,
                    if warmup { " warmup" } else { "" }
                );
                let record = worker::run_case(&case, sample, warmup, provenance.clone());
                serde_json::to_writer(&mut records_file, &record)?;
                records_file.write_all(b"\n")?;
                records_file.flush()?;
                records.push(record);
            }
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
    let out_dir = match args.out {
        Some(path) => path,
        None => default_hash_out_dir()?,
    };
    fs::create_dir_all(&out_dir)?;
    let records_path = out_dir.join("records.jsonl");
    let mut records_file = File::create(&records_path)?;
    let provenance = provenance::capture()?;
    provenance.write_hash(&out_dir.join("provenance.txt"))?;

    let mut records = Vec::new();
    for payload in payloads {
        for scheme in &schemes {
            for thread_count in &threads {
                let Some(case) = hash_case(payload, *scheme, *thread_count) else {
                    bail!("unknown payload 2^{payload} for {scheme:?} threads={thread_count}");
                };
                let samples = args.warmups.saturating_add(args.runs);
                for sample in 0..samples {
                    let warmup = sample < args.warmups;
                    eprintln!(
                        "[{} 2^{} {} t{} sample {}{}]",
                        case.scheme.token(),
                        payload,
                        case.native_param,
                        case.threads,
                        sample,
                        if warmup { " warmup" } else { "" }
                    );
                    let record = worker::run_hash_case(&case, sample, warmup, provenance.clone());
                    serde_json::to_writer(&mut records_file, &record)?;
                    records_file.write_all(b"\n")?;
                    records_file.flush()?;
                    records.push(record);
                }
            }
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

fn compare_hash(args: CompareArgs) -> Result<()> {
    let records = load_hash_records(&args.input)?;
    if records.is_empty() {
        bail!("no hash records in {}", args.input.display());
    }
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
    if path.is_dir() {
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
        return Ok(records);
    }
    load_jsonl(path)
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
        records.push(record);
    }
    Ok(records)
}

fn load_hash_records(path: &Path) -> Result<Vec<HashRecord>> {
    if path.is_dir() {
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
        return Ok(records);
    }
    load_hash_jsonl(path)
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
        records.push(record);
    }
    Ok(records)
}
