//! Host provenance for lattice-eval runs.

use anyhow::{Context, Result};
use pcs_bench_core::Provenance;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::process::Command;

pub(crate) fn capture() -> Result<Provenance> {
    let avx512 = avx512f();
    let rustflags = std::env::var("RUSTFLAGS").unwrap_or_default();
    let isa_notes = if avx512 {
        let mut note = String::from("AVX-512F");
        if rustflags.contains("target-cpu=native") || rustflags.contains("avx512") {
            note.push_str(" advertised; native-target code generation requested");
        } else {
            note.push_str(" advertised; Greyhound uses -march=native");
        }
        note
    } else {
        "AVX-512F not advertised".into()
    };
    let ram = memory_bytes();
    Ok(Provenance {
        harness_revision: git_revision().unwrap_or_else(|| "uncommitted".into()),
        timestamp_utc: command_text("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"]),
        run_command: Some(std::env::args().collect::<Vec<_>>().join(" ")),
        rustc_version: rustc_version()?,
        target: uname()?,
        cpu_model: cpu_model(),
        machine_id_hash: machine_id_hash(),
        threads: 1,
        rustflags,
        avx512,
        isa_notes,
        logical_cpus: logical_cpus(),
        memory_bytes: ram,
        memory_limit_bytes: ram.map(pcs_bench_core::worker_memory_limit_bytes),
        executable_sha256: None,
        worker_compiler_version: None,
        lockfile_sha256: None,
        build_command: None,
        workload_seed: None,
        seed_mode: None,
    })
}

fn command_text(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|text| text.trim().to_owned())
}

fn machine_id_hash() -> Option<String> {
    let hostname = command_text("hostname", &[])?;
    let digest = format!("{:x}", Sha256::digest(hostname.as_bytes()));
    Some(digest.get(..16).unwrap_or(&digest).to_owned())
}

pub(crate) trait ProvenanceExt {
    fn write(&self, path: &Path) -> Result<()>;
    fn write_hash(&self, path: &Path) -> Result<()>;
}

impl ProvenanceExt for Provenance {
    fn write(&self, path: &Path) -> Result<()> {
        let body = format!(
            "harness_revision={}\n\
             timestamp_utc={}\n\
             run_command={}\n\
             rustc_version={}\n\
             target={}\n\
             cpu_model={}\n\
             machine_id_hash={}\n\
             threads={}\n\
             rustflags={}\n\
             avx512={}\n\
             isa_notes={}\n\
             logical_cpus={}\n\
             memory_bytes={}\n\
             memory_limit_bytes={}\n\
             seed_mode={}\n\
             akita={}\n\
             akita_offload={}\n\
             greyhound={}\n\
             rokoko={}\n",
            self.harness_revision,
            self.timestamp_utc.as_deref().unwrap_or("unknown"),
            self.run_command.as_deref().unwrap_or("unknown"),
            self.rustc_version,
            self.target,
            self.cpu_model,
            self.machine_id_hash.as_deref().unwrap_or("unknown"),
            self.threads,
            self.rustflags,
            self.avx512,
            self.isa_notes,
            self.logical_cpus,
            self.memory_bytes
                .map_or_else(|| "unknown".into(), |bytes| bytes.to_string()),
            self.memory_limit_bytes
                .map_or_else(|| "unknown".into(), |bytes| bytes.to_string()),
            self.seed_mode.as_deref().unwrap_or("unknown"),
            pcs_bench_core::SchemeId::Akita.commit_url(),
            pcs_bench_core::SchemeId::AkitaOffload.commit_url(),
            pcs_bench_core::SchemeId::Greyhound.commit_url(),
            pcs_bench_core::SchemeId::Rokoko.commit_url(),
        );
        fs::write(path, body).with_context(|| format!("write {}", path.display()))
    }

    fn write_hash(&self, path: &Path) -> Result<()> {
        let body = format!(
            "harness_revision={}\n\
             timestamp_utc={}\n\
             run_command={}\n\
             rustc_version={}\n\
             target={}\n\
             cpu_model={}\n\
             machine_id_hash={}\n\
             threads={}\n\
             rustflags={}\n\
             avx512={}\n\
             isa_notes={}\n\
             logical_cpus={}\n\
             memory_bytes={}\n\
             memory_limit_bytes={}\n\
             seed_mode={}\n\
             akita={}\n\
             whir={}\n\
             basefold={}\n\
             plonky2_fri={}\n\
             plonky3_fri_stir={}\n\
             binius64={}\n\
             flock={}\n\
             whir_provekit={}\n\
             security_bits_128={}\n\
             security_bits_100={}\n\
             provekit_security_bits={}\n",
            self.harness_revision,
            self.timestamp_utc.as_deref().unwrap_or("unknown"),
            self.run_command.as_deref().unwrap_or("unknown"),
            self.rustc_version,
            self.target,
            self.cpu_model,
            self.machine_id_hash.as_deref().unwrap_or("unknown"),
            self.threads,
            self.rustflags,
            self.avx512,
            self.isa_notes,
            self.logical_cpus,
            self.memory_bytes
                .map_or_else(|| "unknown".into(), |bytes| bytes.to_string()),
            self.memory_limit_bytes
                .map_or_else(|| "unknown".into(), |bytes| bytes.to_string()),
            self.seed_mode.as_deref().unwrap_or("unknown"),
            pcs_bench_core::HashSchemeId::Akita.commit_url(),
            pcs_bench_core::HashSchemeId::Whir.commit_url(),
            pcs_bench_core::HashSchemeId::Basefold.commit_url(),
            pcs_bench_core::HashSchemeId::Plonky2Fri.commit_url(),
            pcs_bench_core::HashSchemeId::Plonky3Fri.commit_url(),
            pcs_bench_core::HashSchemeId::Binius64.commit_url(),
            pcs_bench_core::HashSchemeId::FlockLigerito.commit_url(),
            pcs_bench_core::HashSchemeId::WhirProvekit.commit_url(),
            pcs_bench_core::HASH_SECURITY_BITS,
            pcs_bench_core::HASH_SECURITY_BITS_100,
            pcs_bench_core::PROVEKIT_SECURITY_BITS,
        );
        fs::write(path, body).with_context(|| format!("write {}", path.display()))
    }
}

fn git_revision() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let revision = String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())?;
    let status = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all"])
        .output()
        .ok()?;
    if !status.status.success() || status.stdout.is_empty() {
        return Some(revision);
    }
    let diff = Command::new("git")
        .args(["diff", "--binary", "HEAD"])
        .output()
        .ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&status.stdout);
    if diff.status.success() {
        hasher.update(&diff.stdout);
    }
    let root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|path| std::path::PathBuf::from(path.trim()));
    let untracked = Command::new("git")
        .args([
            "ls-files",
            "--others",
            "--exclude-standard",
            "--full-name",
            "-z",
        ])
        .output()
        .ok();
    if let (Some(root), Some(untracked)) =
        (root, untracked.filter(|output| output.status.success()))
    {
        for path in untracked.stdout.split(|byte| *byte == 0) {
            if path.is_empty() {
                continue;
            }
            hasher.update(path);
            let relative = String::from_utf8_lossy(path);
            if let Ok(contents) = fs::read(root.join(relative.as_ref())) {
                hasher.update(&contents);
            }
        }
    }
    let dirty = format!("{:x}", hasher.finalize());
    Some(format!(
        "{revision}+dirty.{}",
        dirty.get(..12).unwrap_or(&dirty)
    ))
}

fn rustc_version() -> Result<String> {
    let output = Command::new("rustc")
        .arg("-Vv")
        .output()
        .context("rustc -Vv")?;
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn uname() -> Result<String> {
    let output = Command::new("uname")
        .args(["-sm"])
        .output()
        .context("uname")?;
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn cpu_model() -> String {
    if let Ok(output) = Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
    {
        if output.status.success() {
            if let Ok(text) = String::from_utf8(output.stdout) {
                let text = text.trim();
                if !text.is_empty() {
                    return text.to_owned();
                }
            }
        }
    }
    if let Ok(text) = fs::read_to_string("/proc/cpuinfo") {
        if let Some(line) = text.lines().find(|line| line.starts_with("model name")) {
            if let Some((_, value)) = line.split_once(':') {
                return value.trim().to_owned();
            }
        }
    }
    "unknown".into()
}

fn avx512f() -> bool {
    fs::read_to_string("/proc/cpuinfo")
        .ok()
        .is_some_and(|text| text.contains("avx512f"))
}

fn logical_cpus() -> u32 {
    std::thread::available_parallelism().map_or(0, |count| count.get() as u32)
}

fn memory_bytes() -> Option<u64> {
    if let Ok(output) = Command::new("sysctl").args(["-n", "hw.memsize"]).output() {
        if output.status.success() {
            if let Ok(text) = String::from_utf8(output.stdout) {
                if let Ok(bytes) = text.trim().parse::<u64>() {
                    if bytes > 0 {
                        return Some(bytes);
                    }
                }
            }
        }
    }
    let text = fs::read_to_string("/proc/meminfo").ok()?;
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("MemTotal:") else {
            continue;
        };
        let kb: u64 = rest.split_whitespace().next()?.parse().ok()?;
        return Some(kb.saturating_mul(1024));
    }
    None
}
