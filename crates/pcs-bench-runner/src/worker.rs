//! Process-isolated scheme workers.

use anyhow::{bail, Context, Result};
use pcs_bench_core::{
    greyhound_ring_len, parse_rokoko_stdout, HashCase, HashRecord, HashSchemeId, LatticeCase,
    LatticeRecord, Provenance, RunStatus, SchemeId, WorkerOutput, GREYHOUND_SIS_POLICY,
    RESULT_SCHEMA_VERSION, THREADS_LATTICE_EVAL,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub(crate) fn run_case(
    case: &LatticeCase,
    sample: u32,
    warmup: bool,
    provenance: Provenance,
) -> LatticeRecord {
    let mut provenance = provenance;
    provenance.threads = THREADS_LATTICE_EVAL;

    if case.log2_n.is_none() {
        return record(
            case,
            sample,
            warmup,
            provenance,
            WorkerOutput {
                status: RunStatus::Unsupported,
                status_detail: case.unsupported_reason.map(str::to_owned),
                log2_n: None,
                timings_ns: BTreeMap::new(),
                proof_bytes: None,
                commitment_bytes: None,
                state_bytes: None,
                peak_rss_bytes: None,
            },
        );
    }

    let mem_limit = provenance.resolved_memory_limit_bytes().unwrap_or(0);
    let output = match case.scheme {
        SchemeId::Akita => run_akita(case, "pcs-bench-akita", mem_limit),
        SchemeId::AkitaPr466 => run_akita_pr466(case, mem_limit),
        SchemeId::Greyhound => run_greyhound(case, mem_limit),
        SchemeId::Rokoko => run_rokoko(case, mem_limit),
    };

    let worker = match output {
        Ok(worker) => worker,
        Err(error) if is_oom(&error) => WorkerOutput {
            status: RunStatus::Oom,
            status_detail: Some(error.to_string()),
            log2_n: case.log2_n,
            timings_ns: BTreeMap::new(),
            proof_bytes: None,
            commitment_bytes: None,
            state_bytes: None,
            peak_rss_bytes: None,
        },
        Err(error) => WorkerOutput {
            status: RunStatus::Error,
            status_detail: Some(error.to_string()),
            log2_n: case.log2_n,
            timings_ns: BTreeMap::new(),
            proof_bytes: None,
            commitment_bytes: None,
            state_bytes: None,
            peak_rss_bytes: None,
        },
    };

    record(case, sample, warmup, provenance, worker)
}

pub(crate) fn run_hash_case(
    case: &HashCase,
    sample: u32,
    warmup: bool,
    provenance: Provenance,
) -> HashRecord {
    let mut provenance = provenance;
    provenance.threads = case.threads;

    let mem_limit = provenance.resolved_memory_limit_bytes().unwrap_or(0);
    let output = match case.scheme {
        HashSchemeId::Akita => run_akita_hash(case, mem_limit),
        HashSchemeId::Whir => run_whir(case, mem_limit),
        HashSchemeId::Basefold => run_basefold(case, mem_limit),
    };

    let worker = match output {
        Ok(worker) => worker,
        Err(error) if is_oom(&error) => WorkerOutput {
            status: RunStatus::Oom,
            status_detail: Some(error.to_string()),
            log2_n: Some(case.log2_n),
            timings_ns: BTreeMap::new(),
            proof_bytes: None,
            commitment_bytes: None,
            state_bytes: None,
            peak_rss_bytes: None,
        },
        Err(error) => WorkerOutput {
            status: RunStatus::Error,
            status_detail: Some(error.to_string()),
            log2_n: Some(case.log2_n),
            timings_ns: BTreeMap::new(),
            proof_bytes: None,
            commitment_bytes: None,
            state_bytes: None,
            peak_rss_bytes: None,
        },
    };

    hash_record(case, sample, warmup, provenance, worker)
}

fn hash_record(
    case: &HashCase,
    sample: u32,
    warmup: bool,
    provenance: Provenance,
    worker: WorkerOutput,
) -> HashRecord {
    HashRecord {
        schema_version: RESULT_SCHEMA_VERSION,
        status: worker.status,
        status_detail: worker.status_detail,
        scheme: case.scheme,
        implementation_revision: case.scheme.revision().to_owned(),
        payload_log2: case.payload_log2,
        log2_n: worker.log2_n.or(Some(case.log2_n)),
        field: case.field.name.to_owned(),
        native_param: Some(case.native_param.to_owned()),
        threads: case.threads,
        sample,
        warmup,
        historical: false,
        timings_ns: worker.timings_ns,
        proof_bytes: worker.proof_bytes,
        commitment_bytes: worker.commitment_bytes,
        state_bytes: worker.state_bytes,
        peak_rss_bytes: worker.peak_rss_bytes,
        provenance,
    }
}

fn record(
    case: &LatticeCase,
    sample: u32,
    warmup: bool,
    provenance: Provenance,
    worker: WorkerOutput,
) -> LatticeRecord {
    LatticeRecord {
        schema_version: RESULT_SCHEMA_VERSION,
        status: worker.status,
        status_detail: worker.status_detail,
        scheme: case.scheme,
        implementation_revision: case.scheme.revision().to_owned(),
        payload_log2: case.payload_log2,
        log2_n: worker.log2_n.or(case.log2_n),
        field: case.field.name.to_owned(),
        native_param: case.native_param.map(str::to_owned),
        threads: THREADS_LATTICE_EVAL,
        sample,
        warmup,
        historical: false,
        timings_ns: worker.timings_ns,
        proof_bytes: worker.proof_bytes,
        commitment_bytes: worker.commitment_bytes,
        state_bytes: worker.state_bytes,
        peak_rss_bytes: worker.peak_rss_bytes,
        provenance,
    }
}

fn run_akita(case: &LatticeCase, package: &str, mem_limit: u64) -> Result<WorkerOutput> {
    let log2_n = case.log2_n.context("akita cell is supported")?;
    let output = limited_command(workspace_root()?, mem_limit)
        .args([
            "cargo",
            "run",
            "--release",
            "-p",
            package,
            "--bin",
            "lattice-eval",
            "--",
            "--log2-n",
            &log2_n.to_string(),
            "--payload-log2",
            &case.payload_log2.to_string(),
        ])
        .env("RAYON_NUM_THREADS", "1")
        .env("AKITA_PARALLEL", "0")
        .output()
        .context("spawn Akita lattice-eval")?;
    parse_worker_json(&output, "Akita", mem_limit)
}

fn run_akita_hash(case: &HashCase, mem_limit: u64) -> Result<WorkerOutput> {
    let threads = case.threads.to_string();
    let mut command = limited_command(workspace_root()?, mem_limit);
    command.args([
        "cargo",
        "run",
        "--release",
        "-p",
        "pcs-bench-akita",
        "--bin",
        "lattice-eval",
        "--",
        "--log2-n",
        &case.log2_n.to_string(),
        "--payload-log2",
        &case.payload_log2.to_string(),
        "--threads",
        &threads,
    ]);
    command.env("RAYON_NUM_THREADS", &threads);
    if case.threads <= 1 {
        command.env("AKITA_PARALLEL", "0");
    }
    let output = command.output().context("spawn Akita hash-eval")?;
    parse_worker_json(&output, "Akita", mem_limit)
}

fn run_whir(case: &HashCase, mem_limit: u64) -> Result<WorkerOutput> {
    let manifest = workspace_root()?.join("benchmarks/whir/Cargo.toml");
    if !manifest.exists() {
        bail!(
            "WHIR adapter missing at {}. Restore benchmarks/whir",
            manifest.display()
        );
    }
    let target_dir = workspace_root()?.join("target/whir");
    let threads = case.threads.to_string();
    let output = limited_command(workspace_root()?, mem_limit)
        .args([
            "cargo",
            "run",
            "--release",
            "--manifest-path",
            manifest.to_str().context("whir manifest path")?,
            "--bin",
            "hash-eval",
            "--",
            "--log2-n",
            &case.log2_n.to_string(),
            "--threads",
            &threads,
        ])
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RAYON_NUM_THREADS", &threads)
        .output()
        .context("spawn WHIR hash-eval")?;
    parse_worker_json(&output, "WHIR", mem_limit)
}

fn run_basefold(case: &HashCase, mem_limit: u64) -> Result<WorkerOutput> {
    let manifest = workspace_root()?.join("benchmarks/basefold/Cargo.toml");
    if !manifest.exists() {
        bail!(
            "BaseFold adapter missing at {}. Restore benchmarks/basefold",
            manifest.display()
        );
    }
    let target_dir = workspace_root()?.join("target/basefold");
    let threads = case.threads.to_string();
    let output = limited_command(workspace_root()?, mem_limit)
        .args([
            "cargo",
            "run",
            "--release",
            "--manifest-path",
            manifest.to_str().context("basefold manifest path")?,
            "--bin",
            "hash-eval",
            "--",
            "--log2-n",
            &case.log2_n.to_string(),
            "--threads",
            &threads,
        ])
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RAYON_NUM_THREADS", &threads)
        .output()
        .context("spawn BaseFold hash-eval")?;
    parse_worker_json(&output, "BaseFold", mem_limit)
}

fn run_akita_pr466(case: &LatticeCase, mem_limit: u64) -> Result<WorkerOutput> {
    let log2_n = case.log2_n.context("akita-pr466 cell is supported")?;
    let manifest = workspace_root()?.join("benchmarks/akita-pr466/Cargo.toml");
    if !manifest.exists() {
        bail!(
            "Akita PR #466 adapter missing at {}. Restore benchmarks/akita-pr466",
            manifest.display()
        );
    }
    let target_dir = workspace_root()?.join("target/akita-pr466");
    let output = limited_command(workspace_root()?, mem_limit)
        .args([
            "cargo",
            "run",
            "--release",
            "--manifest-path",
            manifest.to_str().context("akita-pr466 manifest path")?,
            "--bin",
            "lattice-eval",
            "--",
            "--log2-n",
            &log2_n.to_string(),
            "--payload-log2",
            &case.payload_log2.to_string(),
        ])
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RAYON_NUM_THREADS", "1")
        .env("AKITA_PARALLEL", "0")
        .output()
        .context("spawn Akita PR #466 lattice-eval")?;
    parse_worker_json(&output, "Akita PR #466", mem_limit)
}

fn run_greyhound(case: &LatticeCase, mem_limit: u64) -> Result<WorkerOutput> {
    let log2_n = case.log2_n.context("greyhound cell is supported")?;
    let _ = greyhound_ring_len(log2_n).context("Greyhound ring length")?;
    let binary = greyhound_binary()?;
    let output = limited_command(workspace_root()?, mem_limit)
        .arg(binary.as_os_str())
        .args(["--log2-n", &log2_n.to_string()])
        .env("LATTICE_DOGS_THREADS", "1")
        .env("LABRADOR_SIS_SECURITY", GREYHOUND_SIS_POLICY)
        .output()
        .context("spawn Greyhound lattice-eval")?;
    parse_worker_json(&output, "Greyhound", mem_limit)
}

fn run_rokoko(case: &LatticeCase, mem_limit: u64) -> Result<WorkerOutput> {
    let feature = case.native_param.context("RoKoKo native feature")?;
    let rokoko_root = workspace_root()?.join("third_party/rokoko");
    if !rokoko_root.join("Cargo.toml").exists() {
        bail!(
            "RoKoKo vendor missing at {}. Run ./scripts/fetch-vendors.sh",
            rokoko_root.display()
        );
    }
    let target_dir = workspace_root()?.join(format!("target/rokoko-{feature}"));
    let features = format!("incomplete-rexl,unsafe-sumcheck,{feature}");
    let status = Command::new("cargo")
        .current_dir(&rokoko_root)
        .args([
            "+nightly",
            "build",
            "--release",
            "--no-default-features",
            "--features",
            &features,
        ])
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RAYON_NUM_THREADS", "1")
        .status()
        .context("build RoKoKo")?;
    if !status.success() {
        bail!("RoKoKo build failed with status {status:?}");
    }
    let binary = target_dir.join("release/rokoko");
    if !binary.exists() {
        bail!("RoKoKo binary missing after build: {}", binary.display());
    }
    let output = limited_command(&rokoko_root, mem_limit)
        .arg(binary.as_os_str())
        .env("MIMALLOC_PURGE_DELAY", "-1")
        .env("RAYON_NUM_THREADS", "1")
        .output()
        .context("spawn RoKoKo")?;
    classify_status(&output, "RoKoKo", mem_limit)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");
    let parsed = parse_rokoko_stdout(&combined).context("parse RoKoKo timings from stdout")?;
    if parsed.commitment_bytes.is_none()
        || parsed.state_bytes.is_none()
        || parsed.peak_rss_bytes.is_none()
    {
        bail!("RoKoKo stdout missing commitment/CRS/RSS lines; run ./scripts/fetch-vendors.sh");
    }
    Ok(WorkerOutput {
        status: RunStatus::Ok,
        status_detail: None,
        log2_n: case.log2_n,
        timings_ns: parsed.timings_ns,
        proof_bytes: parsed.proof_bytes,
        commitment_bytes: parsed.commitment_bytes,
        state_bytes: parsed.state_bytes,
        peak_rss_bytes: parsed.peak_rss_bytes,
    })
}

fn greyhound_binary() -> Result<PathBuf> {
    let root = workspace_root()?;
    let binary = root.join("target/greyhound/lattice-eval");
    let status = Command::new(root.join("scripts/build-greyhound.sh"))
        .status()
        .context("build Greyhound")?;
    if !status.success() {
        bail!("Greyhound build failed; AVX-512 Linux is required. See docs/lattice-eval.md");
    }
    if binary.exists() {
        Ok(binary)
    } else {
        bail!("Greyhound binary missing after build: {}", binary.display())
    }
}

fn limited_command(dir: impl AsRef<Path>, mem_limit: u64) -> Command {
    let root = workspace_root().unwrap_or_else(|_| PathBuf::from("."));
    let mut command = Command::new("bash");
    command
        .current_dir(dir)
        .arg(root.join("scripts/with-memlimit.sh"))
        .arg(mem_limit.to_string());
    command
}

fn parse_worker_json(output: &Output, label: &str, mem_limit: u64) -> Result<WorkerOutput> {
    classify_status(output, label, mem_limit)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_line = stdout
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('{'))
        .with_context(|| format!("{label} produced no JSON object on stdout"))?;
    serde_json::from_str(json_line).with_context(|| format!("parse {label} worker JSON"))
}

fn classify_status(output: &Output, label: &str, mem_limit: u64) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }
    if is_signal(output, 9) || output.status.code() == Some(137) {
        let gib = (mem_limit as f64 / 1_073_741_824.0).round();
        bail!("{label} exceeded the {gib:.0} GiB memory limit");
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    bail!(
        "{label} failed with status {:?}: {}",
        output.status.code(),
        stderr.trim()
    )
}

fn is_oom(error: &anyhow::Error) -> bool {
    let text = error.to_string().to_ascii_lowercase();
    pcs_bench_core::looks_like_oom(Some(&text))
}

fn is_signal(output: &Output, signal: i32) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        output.status.signal() == Some(signal)
    }
    #[cfg(not(unix))]
    {
        let _ = signal;
        false
    }
}

fn workspace_root() -> Result<PathBuf> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?)
}
