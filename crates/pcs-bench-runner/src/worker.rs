//! Process-isolated scheme workers.

use anyhow::{bail, Context, Result};
use pcs_bench_core::{
    greyhound_ring_len, parse_rokoko_stdout, HashCase, HashRecord, HashSchemeId, LatticeCase,
    LatticeRecord, Provenance, RunStatus, SchemeId, WorkerOutput, GREYHOUND_SIS_POLICY,
    THREADS_LATTICE_EVAL,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ROKOKO_TOOLCHAIN: &str = "+nightly-2026-09-03";

pub(crate) fn run_case(
    case: &LatticeCase,
    sample: u32,
    warmup: bool,
    seed: u64,
    provenance: Provenance,
) -> LatticeRecord {
    let mut provenance = provenance;
    provenance.threads = THREADS_LATTICE_EVAL;
    provenance.workload_seed = Some(seed);

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
                evaluation_bytes: None,
                public_context_bytes: None,
                state_bytes: None,
                peak_rss_bytes: None,
            },
        );
    }

    if let Err(error) = attach_lattice_build_identity(case, &mut provenance) {
        return record(
            case,
            sample,
            warmup,
            provenance,
            WorkerOutput {
                status: RunStatus::Error,
                status_detail: Some(error.to_string()),
                log2_n: case.log2_n,
                timings_ns: BTreeMap::new(),
                proof_bytes: None,
                commitment_bytes: None,
                evaluation_bytes: None,
                public_context_bytes: None,
                state_bytes: None,
                peak_rss_bytes: None,
            },
        );
    }

    let mem_limit = provenance.resolved_memory_limit_bytes().unwrap_or(0);
    let output = match case.scheme {
        SchemeId::Akita => run_akita(case, mem_limit, seed, false),
        SchemeId::AkitaOffload => run_akita(case, mem_limit, seed, true),
        SchemeId::Greyhound => run_greyhound(case, mem_limit, seed),
        SchemeId::Rokoko => run_rokoko(case, mem_limit, seed),
    }
    .and_then(|worker| validate_case_log2(worker, case.log2_n, case.scheme.display_name()));

    let worker = match output {
        Ok(worker) => worker,
        Err(error) if is_oom(&error) => WorkerOutput {
            status: RunStatus::Oom,
            status_detail: Some(error.to_string()),
            log2_n: case.log2_n,
            timings_ns: BTreeMap::new(),
            proof_bytes: None,
            commitment_bytes: None,
            evaluation_bytes: None,
            public_context_bytes: None,
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
            evaluation_bytes: None,
            public_context_bytes: None,
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
    seed: u64,
    provenance: Provenance,
) -> HashRecord {
    let mut provenance = provenance;
    provenance.threads = case.threads;
    provenance.workload_seed = Some(seed);

    let mem_limit = provenance.resolved_memory_limit_bytes().unwrap_or(0);
    let output = attach_hash_build_identity(case, &mut provenance)
        .and_then(|()| spawn_hash_worker(case, mem_limit, seed))
        .and_then(|worker| {
            validate_case_log2(worker, Some(case.log2_n), case.scheme.display_name())
        });

    let worker = match output {
        Ok(worker) => worker,
        Err(error) if is_oom(&error) => WorkerOutput {
            status: RunStatus::Oom,
            status_detail: Some(error.to_string()),
            log2_n: Some(case.log2_n),
            timings_ns: BTreeMap::new(),
            proof_bytes: None,
            commitment_bytes: None,
            evaluation_bytes: None,
            public_context_bytes: None,
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
            evaluation_bytes: None,
            public_context_bytes: None,
            state_bytes: None,
            peak_rss_bytes: None,
        },
    };

    hash_record(case, sample, warmup, provenance, worker)
}

pub(crate) fn prepare_lattice_case(case: &LatticeCase) -> Result<()> {
    if case.log2_n.is_none() {
        return Ok(());
    }
    match case.scheme {
        SchemeId::Akita | SchemeId::AkitaOffload => build_akita(),
        SchemeId::Greyhound => build_greyhound(),
        SchemeId::Rokoko => build_rokoko(case),
    }
}

pub(crate) fn prepare_hash_case(case: &HashCase) -> Result<()> {
    match case.scheme {
        HashSchemeId::Akita | HashSchemeId::AkitaFp64 | HashSchemeId::AkitaFp128 => build_akita(),
        HashSchemeId::Whir => {
            build_isolated_hash("benchmarks/whir", "hash-eval", &[], "WHIR (Plonky3)")
        }
        HashSchemeId::Basefold => {
            build_isolated_hash("benchmarks/basefold", "hash-eval", &[], "BaseFold (SP1)")
        }
        HashSchemeId::Plonky2Fri => build_isolated_hash(
            "benchmarks/plonky2-fri",
            "hash-eval",
            &[("RUSTC_BOOTSTRAP", "1")],
            "Plonky2 FRI",
        ),
        HashSchemeId::Plonky3Fri => build_isolated_hash(
            "benchmarks/plonky3-uni",
            "hash-eval-fri",
            &[],
            "Plonky3 FRI",
        ),
        HashSchemeId::Plonky3Stir => build_isolated_hash(
            "benchmarks/plonky3-uni",
            "hash-eval-stir",
            &[],
            "Plonky3 STIR",
        ),
        HashSchemeId::Binius64 => {
            build_isolated_hash("benchmarks/binius64", "hash-eval", &[], "Binius64 BaseFold")
        }
        HashSchemeId::FlockLigerito => build_isolated_hash(
            "benchmarks/flock-ligerito",
            "hash-eval",
            &[],
            "Flock Ligerito",
        ),
        HashSchemeId::WhirProvekit => build_isolated_hash(
            "benchmarks/whir-provekit",
            "hash-eval",
            &[],
            "WHIR (ProveKit)",
        ),
    }
}

fn attach_lattice_build_identity(case: &LatticeCase, provenance: &mut Provenance) -> Result<()> {
    let root = workspace_root()?;
    match case.scheme {
        SchemeId::Akita | SchemeId::AkitaOffload => attach_build_identity(
            provenance,
            &akita_binary(&root)?,
            Some(&root.join("benchmarks/akita/Cargo.lock")),
            "CARGO_TARGET_DIR=target/akita cargo build --release --locked --manifest-path benchmarks/akita/Cargo.toml --bin lattice-eval",
        ),
        SchemeId::Greyhound => {
            attach_build_identity(
                provenance,
                &greyhound_binary()?,
                None,
                "./scripts/build-greyhound.sh",
            )?;
            let compiler = std::env::var("CC").unwrap_or_else(|_| "cc".into());
            provenance.worker_compiler_version = command_version(&compiler, &["--version"]);
            Ok(())
        }
        SchemeId::Rokoko => {
            let feature = case.native_param.context("RoKoKo native feature")?;
            attach_build_identity(
                provenance,
                &root.join(format!("target/rokoko-{feature}/release/rokoko")),
                Some(&root.join("third_party/rokoko/Cargo.lock")),
                &format!(
                    "CARGO_TARGET_DIR=target/rokoko-{feature} cargo {ROKOKO_TOOLCHAIN} build --release --locked --no-default-features --features incomplete-rexl,unsafe-sumcheck,{feature}"
                ),
            )?;
            provenance.worker_compiler_version =
                command_version("rustc", &[ROKOKO_TOOLCHAIN, "-Vv"]);
            Ok(())
        }
    }
}

fn attach_hash_build_identity(case: &HashCase, provenance: &mut Provenance) -> Result<()> {
    let root = workspace_root()?;
    if matches!(
        case.scheme,
        HashSchemeId::Akita | HashSchemeId::AkitaFp64 | HashSchemeId::AkitaFp128
    ) {
        return attach_build_identity(
            provenance,
            &akita_binary(&root)?,
            Some(&root.join("benchmarks/akita/Cargo.lock")),
            "CARGO_TARGET_DIR=target/akita cargo build --release --locked --manifest-path benchmarks/akita/Cargo.toml --bin lattice-eval",
        );
    }
    let (crate_dir, bin, extra_env) = match case.scheme {
        HashSchemeId::Whir => ("benchmarks/whir", "hash-eval", ""),
        HashSchemeId::Basefold => ("benchmarks/basefold", "hash-eval", ""),
        HashSchemeId::Plonky2Fri => ("benchmarks/plonky2-fri", "hash-eval", "RUSTC_BOOTSTRAP=1 "),
        HashSchemeId::Plonky3Fri => ("benchmarks/plonky3-uni", "hash-eval-fri", ""),
        HashSchemeId::Plonky3Stir => ("benchmarks/plonky3-uni", "hash-eval-stir", ""),
        HashSchemeId::Binius64 => ("benchmarks/binius64", "hash-eval", ""),
        HashSchemeId::FlockLigerito => ("benchmarks/flock-ligerito", "hash-eval", ""),
        HashSchemeId::WhirProvekit => ("benchmarks/whir-provekit", "hash-eval", ""),
        HashSchemeId::Akita | HashSchemeId::AkitaFp64 | HashSchemeId::AkitaFp128 => {
            unreachable!("Akita handled above")
        }
    };
    let target_dir = root
        .join("target")
        .join(crate_dir.trim_start_matches("benchmarks/"));
    attach_build_identity(
        provenance,
        &target_dir.join("release").join(bin),
        Some(&root.join(crate_dir).join("Cargo.lock")),
        &format!(
            "{extra_env}CARGO_TARGET_DIR=target/{} cargo build --release --locked --manifest-path {crate_dir}/Cargo.toml --bin {bin}",
            crate_dir.trim_start_matches("benchmarks/")
        ),
    )
}

fn attach_build_identity(
    provenance: &mut Provenance,
    executable: &Path,
    lockfile: Option<&Path>,
    build_command: &str,
) -> Result<()> {
    provenance.executable_sha256 = Some(
        sha256_file(executable)
            .with_context(|| format!("hash worker executable {}", executable.display()))?,
    );
    provenance.lockfile_sha256 = lockfile
        .filter(|path| path.is_file())
        .map(|path| {
            sha256_file(path)
                .with_context(|| format!("hash dependency lockfile {}", path.display()))
        })
        .transpose()?;
    provenance.build_command = Some(build_command.to_owned());
    provenance.worker_compiler_version = Some(provenance.rustc_version.clone());
    Ok(())
}

fn command_version(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|version| version.trim().to_owned())
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024].into_boxed_slice();
    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("read {}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn hash_record(
    case: &HashCase,
    sample: u32,
    warmup: bool,
    provenance: Provenance,
    worker: WorkerOutput,
) -> HashRecord {
    HashRecord {
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
        timings_ns: worker.timings_ns,
        proof_bytes: worker.proof_bytes,
        commitment_bytes: worker.commitment_bytes,
        evaluation_bytes: worker.evaluation_bytes,
        public_context_bytes: worker.public_context_bytes,
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
        timings_ns: worker.timings_ns,
        proof_bytes: worker.proof_bytes,
        commitment_bytes: worker.commitment_bytes,
        evaluation_bytes: worker.evaluation_bytes,
        public_context_bytes: worker.public_context_bytes,
        state_bytes: worker.state_bytes,
        peak_rss_bytes: worker.peak_rss_bytes,
        provenance,
    }
}

fn build_akita() -> Result<()> {
    let root = workspace_root()?;
    let manifest = root.join("benchmarks/akita/Cargo.toml");
    let target_dir = root.join("target/akita");
    let status = Command::new("cargo")
        .current_dir(root.join("benchmarks/akita"))
        .args([
            "build",
            "--release",
            "--locked",
            "--manifest-path",
            manifest
                .to_str()
                .context("Akita manifest path is not valid UTF-8")?,
            "--bin",
            "lattice-eval",
        ])
        .env("CARGO_TARGET_DIR", &target_dir)
        .status()
        .context("build Akita worker")?;
    if !status.success() {
        bail!("Akita worker build failed with status {status:?}");
    }
    let _ = akita_binary(&root)?;
    Ok(())
}

fn akita_binary(root: &Path) -> Result<PathBuf> {
    let binary = root.join("target/akita/release/lattice-eval");
    if binary.is_file() {
        Ok(binary)
    } else {
        bail!("Akita worker executable missing at {}", binary.display())
    }
}

fn build_isolated_hash(
    crate_dir: &str,
    bin: &str,
    extra_env: &[(&str, &str)],
    label: &str,
) -> Result<()> {
    let root = workspace_root()?;
    let manifest = root.join(crate_dir).join("Cargo.toml");
    if !manifest.exists() {
        bail!(
            "{label} adapter missing at {}. Restore {crate_dir}",
            manifest.display()
        );
    }
    let target_dir = root
        .join("target")
        .join(crate_dir.trim_start_matches("benchmarks/"));
    let mut command = Command::new("cargo");
    command
        .current_dir(root.join(crate_dir))
        .args([
            "build",
            "--release",
            "--locked",
            "--manifest-path",
            manifest
                .to_str()
                .with_context(|| format!("{label} manifest path"))?,
            "--bin",
            bin,
        ])
        .env("CARGO_TARGET_DIR", &target_dir);
    for (key, value) in extra_env {
        command.env(key, value);
    }
    let status = command
        .status()
        .with_context(|| format!("build {label} worker"))?;
    if !status.success() {
        bail!("{label} worker build failed with status {status:?}");
    }
    let binary = target_dir.join("release").join(bin);
    if !binary.is_file() {
        bail!("{label} executable missing at {}", binary.display());
    }
    Ok(())
}

fn build_greyhound() -> Result<()> {
    let root = workspace_root()?;
    let status = Command::new(root.join("scripts/build-greyhound.sh"))
        .status()
        .context("build Greyhound")?;
    if !status.success() {
        bail!("Greyhound build failed; AVX-512 Linux is required. See docs/lattice-eval.md");
    }
    let _ = greyhound_binary()?;
    Ok(())
}

fn build_rokoko(case: &LatticeCase) -> Result<()> {
    let feature = case.native_param.context("RoKoKo native feature")?;
    let root = workspace_root()?;
    let rokoko_root = root.join("third_party/rokoko");
    if !rokoko_root.join("Cargo.toml").exists() {
        bail!(
            "RoKoKo vendor missing at {}. Run ./scripts/fetch-vendors.sh",
            rokoko_root.display()
        );
    }
    let target_dir = root.join(format!("target/rokoko-{feature}"));
    let features = format!("incomplete-rexl,unsafe-sumcheck,{feature}");
    let status = Command::new("cargo")
        .current_dir(&rokoko_root)
        .args([
            ROKOKO_TOOLCHAIN,
            "build",
            "--release",
            "--locked",
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
    if !binary.is_file() {
        bail!("RoKoKo binary missing after build: {}", binary.display());
    }
    Ok(())
}

fn run_akita(case: &LatticeCase, mem_limit: u64, seed: u64, offload: bool) -> Result<WorkerOutput> {
    let log2_n = case.log2_n.context("akita cell is supported")?;
    let root = workspace_root()?;
    let binary = akita_binary(&root)?;
    let mut command = limited_command(&root, mem_limit);
    command.arg(binary).args([
        "--log2-n",
        &log2_n.to_string(),
        "--payload-log2",
        &case.payload_log2.to_string(),
    ]);
    if offload {
        command.arg("--offload");
    }
    let output = command
        .env("RAYON_NUM_THREADS", "1")
        .env("AKITA_PARALLEL", "0")
        .env("PCS_BENCH_SEED", seed.to_string())
        .output()
        .context("spawn Akita lattice-eval")?;
    parse_worker_json(&output, "Akita", mem_limit)
}

fn spawn_hash_worker(case: &HashCase, mem_limit: u64, seed: u64) -> Result<WorkerOutput> {
    match case.scheme {
        HashSchemeId::Akita | HashSchemeId::AkitaFp64 | HashSchemeId::AkitaFp128 => {
            run_akita_hash(case, mem_limit, seed)
        }
        HashSchemeId::Whir => run_isolated_hash(
            case,
            mem_limit,
            "benchmarks/whir",
            "hash-eval",
            &[],
            &[],
            seed,
            "WHIR (Plonky3)",
        ),
        HashSchemeId::Basefold => run_isolated_hash(
            case,
            mem_limit,
            "benchmarks/basefold",
            "hash-eval",
            &[],
            &[],
            seed,
            "BaseFold (SP1)",
        ),
        HashSchemeId::Plonky2Fri => run_isolated_hash(
            case,
            mem_limit,
            "benchmarks/plonky2-fri",
            "hash-eval",
            &[],
            &[("RUSTC_BOOTSTRAP", "1")],
            seed,
            "Plonky2 FRI",
        ),
        HashSchemeId::Plonky3Fri => run_isolated_hash(
            case,
            mem_limit,
            "benchmarks/plonky3-uni",
            "hash-eval-fri",
            &[],
            &[],
            seed,
            "Plonky3 FRI",
        ),
        HashSchemeId::Plonky3Stir => run_isolated_hash(
            case,
            mem_limit,
            "benchmarks/plonky3-uni",
            "hash-eval-stir",
            &[],
            &[],
            seed,
            "Plonky3 STIR",
        ),
        HashSchemeId::Binius64 => run_isolated_hash(
            case,
            mem_limit,
            "benchmarks/binius64",
            "hash-eval",
            &[],
            &[],
            seed,
            "Binius64 BaseFold",
        ),
        HashSchemeId::FlockLigerito => run_isolated_hash(
            case,
            mem_limit,
            "benchmarks/flock-ligerito",
            "hash-eval",
            &[],
            &[],
            seed,
            "Flock Ligerito",
        ),
        HashSchemeId::WhirProvekit => run_isolated_hash(
            case,
            mem_limit,
            "benchmarks/whir-provekit",
            "hash-eval",
            &[],
            &[],
            seed,
            "WHIR (ProveKit)",
        ),
    }
}

fn run_akita_hash(case: &HashCase, mem_limit: u64, seed: u64) -> Result<WorkerOutput> {
    let threads = case.threads.to_string();
    let field = case.scheme.akita_field_arg().unwrap_or("fp32").to_string();
    let root = workspace_root()?;
    let binary = akita_binary(&root)?;
    let mut command = limited_command(&root, mem_limit);
    command.arg(binary).args([
        "--log2-n",
        &case.log2_n.to_string(),
        "--payload-log2",
        &case.payload_log2.to_string(),
        "--threads",
        &threads,
        "--field",
        &field,
    ]);
    command.env("RAYON_NUM_THREADS", &threads);
    command.env("PCS_BENCH_SEED", seed.to_string());
    if case.threads <= 1 {
        command.env("AKITA_PARALLEL", "0");
    }
    let output = command.output().context("spawn Akita hash-eval")?;
    parse_worker_json(&output, "Akita", mem_limit)
}

#[allow(clippy::too_many_arguments)]
fn run_isolated_hash(
    case: &HashCase,
    mem_limit: u64,
    crate_dir: &str,
    bin: &str,
    extra_args: &[&str],
    extra_env: &[(&str, &str)],
    seed: u64,
    label: &str,
) -> Result<WorkerOutput> {
    let root = workspace_root()?;
    let target_dir = root
        .join("target")
        .join(crate_dir.trim_start_matches("benchmarks/"));
    let binary = target_dir.join("release").join(bin);
    if !binary.is_file() {
        bail!(
            "{label} executable missing at {}; build preparation did not complete",
            binary.display()
        );
    }
    let threads = case.threads.to_string();
    let log2_n = case.log2_n.to_string();
    let mut args = vec!["--log2-n", &log2_n, "--threads", &threads];
    args.extend(extra_args.iter().copied());
    let mut command = limited_command(root.join(crate_dir), mem_limit);
    command
        .arg(binary)
        .args(&args)
        .env("RAYON_NUM_THREADS", &threads)
        .env("PCS_BENCH_SEED", seed.to_string());
    for (key, value) in extra_env {
        command.env(key, value);
    }
    let output = command
        .output()
        .with_context(|| format!("spawn {label} hash-eval"))?;
    parse_worker_json(&output, label, mem_limit)
}

fn run_greyhound(case: &LatticeCase, mem_limit: u64, seed: u64) -> Result<WorkerOutput> {
    let log2_n = case.log2_n.context("greyhound cell is supported")?;
    let _ = greyhound_ring_len(log2_n).context("Greyhound ring length")?;
    let binary = greyhound_binary()?;
    let output = limited_command(workspace_root()?, mem_limit)
        .arg(binary.as_os_str())
        .args(["--log2-n", &log2_n.to_string()])
        .env("LATTICE_DOGS_THREADS", "1")
        .env("LABRADOR_SIS_SECURITY", GREYHOUND_SIS_POLICY)
        .env("PCS_BENCH_SEED", seed.to_string())
        .output()
        .context("spawn Greyhound lattice-eval")?;
    parse_worker_json(&output, "Greyhound", mem_limit)
}

fn run_rokoko(case: &LatticeCase, mem_limit: u64, seed: u64) -> Result<WorkerOutput> {
    let feature = case.native_param.context("RoKoKo native feature")?;
    let rokoko_root = workspace_root()?.join("third_party/rokoko");
    if !rokoko_root.join("Cargo.toml").exists() {
        bail!(
            "RoKoKo vendor missing at {}. Run ./scripts/fetch-vendors.sh",
            rokoko_root.display()
        );
    }
    let target_dir = workspace_root()?.join(format!("target/rokoko-{feature}"));
    let binary = target_dir.join("release/rokoko");
    if !binary.is_file() {
        bail!(
            "RoKoKo executable missing at {}; build preparation did not complete",
            binary.display()
        );
    }
    let output = limited_command(&rokoko_root, mem_limit)
        .arg(binary.as_os_str())
        .env("MIMALLOC_PURGE_DELAY", "-1")
        .env("RAYON_NUM_THREADS", "1")
        .env("PCS_BENCH_SEED", seed.to_string())
        .output()
        .context("spawn RoKoKo")?;
    classify_status(&output, "RoKoKo", mem_limit)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");
    let parsed = parse_rokoko_stdout(&combined).context("parse RoKoKo timings from stdout")?;
    if !parsed.timings_ns.contains_key("setup")
        || parsed.proof_bytes.is_none()
        || parsed.commitment_bytes.is_none()
        || parsed.evaluation_bytes.is_none()
        || parsed.state_bytes.is_none()
        || parsed.peak_rss_bytes.is_none()
    {
        bail!(
            "RoKoKo stdout missing setup/proof/commitment/evaluation/CRS/RSS lines; run ./scripts/fetch-vendors.sh"
        );
    }
    Ok(WorkerOutput {
        status: RunStatus::Ok,
        status_detail: Some(
            "statement=multilinear,distribution=bounded-signed-31-bit,point=native-random,evaluation=separate"
                .into(),
        ),
        log2_n: case.log2_n,
        timings_ns: parsed.timings_ns,
        proof_bytes: parsed.proof_bytes,
        commitment_bytes: parsed.commitment_bytes,
        evaluation_bytes: parsed.evaluation_bytes,
        public_context_bytes: Some(0),
        state_bytes: parsed.state_bytes,
        peak_rss_bytes: parsed.peak_rss_bytes,
    })
}

fn greyhound_binary() -> Result<PathBuf> {
    let root = workspace_root()?;
    let binary = root.join("target/greyhound/lattice-eval");
    if binary.is_file() {
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| line.trim_start().starts_with('{'))
        .collect();
    let json_line = match json_lines.as_slice() {
        [] => None,
        [line] => Some(*line),
        _ => bail!(
            "{label} produced {} JSON objects on stdout; expected exactly one",
            json_lines.len()
        ),
    };
    let parsed = json_line
        .map(|line| {
            serde_json::from_str::<WorkerOutput>(line)
                .with_context(|| format!("parse {label} worker JSON"))
        })
        .transpose()?;

    if output.status.success() {
        let worker =
            parsed.with_context(|| format!("{label} produced no JSON object on stdout"))?;
        match worker.status {
            RunStatus::Ok => validate_success_output(&worker, label)?,
            RunStatus::Unsupported => {}
            RunStatus::Oom | RunStatus::Error => {
                bail!(
                    "{label} exited successfully but reported status {:?}",
                    worker.status
                );
            }
        }
        return Ok(worker);
    }

    if let Some(mut worker) = parsed {
        if worker.status == RunStatus::Ok {
            let exit_error = classify_status(output, label, mem_limit)
                .expect_err("non-successful process must have an exit error");
            bail!("{label} reported success after an unsuccessful process exit: {exit_error}");
        }
        if worker.status == RunStatus::Error
            && pcs_bench_core::looks_like_oom(worker.status_detail.as_deref())
        {
            worker.status = RunStatus::Oom;
        }
        return Ok(worker);
    }

    classify_status(output, label, mem_limit)?;
    unreachable!("unsuccessful process status was accepted")
}

fn validate_success_output(worker: &WorkerOutput, label: &str) -> Result<()> {
    for phase in ["commit", "open", "verify"] {
        if !worker.timings_ns.contains_key(phase) {
            bail!("{label} success response is missing required phase `{phase}`");
        }
    }
    for (field, value) in [
        ("proof_bytes", worker.proof_bytes),
        ("commitment_bytes", worker.commitment_bytes),
        ("evaluation_bytes", worker.evaluation_bytes),
        ("public_context_bytes", worker.public_context_bytes),
    ] {
        if value.is_none() {
            bail!("{label} success response is missing required field `{field}`");
        }
    }
    Ok(())
}

fn validate_case_log2(
    worker: WorkerOutput,
    expected_log2_n: Option<u32>,
    label: &str,
) -> Result<WorkerOutput> {
    if worker.status == RunStatus::Ok && worker.log2_n != expected_log2_n {
        bail!(
            "{label} response identifies log2_n={:?}, expected {expected_log2_n:?}",
            worker.log2_n
        );
    }
    Ok(worker)
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    #[cfg(unix)]
    use super::{classify_status, is_oom, parse_worker_json, validate_case_log2};

    #[cfg(unix)]
    fn shell_output(script: &str) -> std::process::Output {
        std::process::Command::new("sh")
            .args(["-c", script])
            .output()
            .expect("run synthetic worker")
    }

    #[cfg(unix)]
    const SUCCESS: &str = r#"{"status":"ok","log2_n":1,"timings_ns":{"setup":0,"commit":1,"open":2,"verify":3},"proof_bytes":1,"commitment_bytes":1,"evaluation_bytes":1,"public_context_bytes":0}"#;

    #[cfg(unix)]
    #[test]
    fn rejects_success_json_after_failed_exit() {
        let output = shell_output(&format!("printf '%s\\n' '{SUCCESS}'; exit 1"));
        let error = parse_worker_json(&output, "synthetic", 1024).expect_err("must reject");
        assert!(error.to_string().contains("reported success"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_success_with_missing_phase() {
        let output = shell_output(
            r#"printf '%s\n' '{"status":"ok","timings_ns":{"setup":0,"commit":1,"open":2}}'"#,
        );
        let error = parse_worker_json(&output, "synthetic", 1024).expect_err("must reject");
        assert!(error
            .to_string()
            .contains("missing required phase `verify`"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_duplicate_worker_responses() {
        let output = shell_output(&format!("printf '%s\\n%s\\n' '{SUCCESS}' '{SUCCESS}'"));
        let error = parse_worker_json(&output, "synthetic", 1024).expect_err("must reject");
        assert!(error.to_string().contains("expected exactly one"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_unknown_worker_fields() {
        let output = shell_output(
            r#"printf '%s\n' '{"status":"ok","log2_n":1,"timings_ns":{"commit":1,"open":2,"verify":3},"proof_bytes":1,"commitment_bytes":1,"evaluation_bytes":1,"public_context_bytes":0,"typo":1}'"#,
        );
        let error = parse_worker_json(&output, "synthetic", 1024).expect_err("must reject");
        assert!(format!("{error:#}").contains("unknown field"));
    }

    #[cfg(unix)]
    #[test]
    fn preserves_structured_failure() {
        let output = shell_output(
            r#"printf '%s\n' '{"status":"error","status_detail":"proof rejected"}'; exit 1"#,
        );
        let worker = parse_worker_json(&output, "synthetic", 1024).expect("structured error");
        assert_eq!(worker.status, pcs_bench_core::RunStatus::Error);
        assert_eq!(worker.status_detail.as_deref(), Some("proof rejected"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_wrong_case_response() {
        let output = shell_output(&format!("printf '%s\\n' '{SUCCESS}'"));
        let worker = parse_worker_json(&output, "synthetic", 1024).expect("valid response");
        let error =
            validate_case_log2(worker, Some(2), "synthetic").expect_err("wrong case must fail");
        assert!(error.to_string().contains("expected Some(2)"));
    }

    #[cfg(unix)]
    #[test]
    fn sigkill_is_not_assumed_to_be_oom() {
        let output = shell_output("kill -9 $$");
        let error =
            classify_status(&output, "synthetic", 1024).expect_err("signal must be an error");
        assert!(error.to_string().contains("cause is unknown"));
        assert!(!is_oom(&error));
    }
}

fn classify_status(output: &Output, label: &str, mem_limit: u64) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }
    if is_signal(output, 9) || output.status.code() == Some(137) {
        let gib = (mem_limit as f64 / 1_073_741_824.0).round();
        bail!(
            "{label} terminated by SIGKILL/137 under a {gib:.0} GiB address-space ceiling; termination cause is unknown"
        );
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
