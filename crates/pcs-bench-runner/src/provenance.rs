//! Host provenance for lattice-eval runs.

use anyhow::{Context, Result};
use pcs_bench_core::Provenance;
use std::fs;
use std::path::Path;
use std::process::Command;

pub(crate) fn capture() -> Result<Provenance> {
    let avx512 = avx512f();
    let rustflags = std::env::var("RUSTFLAGS").unwrap_or_default();
    let isa_notes = if avx512 {
        let mut note = String::from("AVX-512F");
        if rustflags.contains("target-cpu=native") || rustflags.contains("avx512") {
            note.push_str(" activated (-C target-cpu=native)");
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
        rustc_version: rustc_version()?,
        target: uname()?,
        cpu_model: cpu_model(),
        threads: 1,
        rustflags,
        avx512,
        isa_notes,
        logical_cpus: logical_cpus(),
        memory_bytes: ram,
        memory_limit_bytes: ram.map(pcs_bench_core::worker_memory_limit_bytes),
    })
}

impl ProvenanceExt for Provenance {
    fn write(&self, path: &Path) -> Result<()> {
        let body = format!(
            "harness_revision={}\n\
             rustc_version={}\n\
             target={}\n\
             cpu_model={}\n\
             threads={}\n\
             rustflags={}\n\
             avx512={}\n\
             isa_notes={}\n\
             logical_cpus={}\n\
             memory_bytes={}\n\
             memory_limit_bytes={}\n\
             akita={}\n\
             akita_pr466={}\n\
             greyhound={}\n\
             rokoko={}\n",
            self.harness_revision,
            self.rustc_version,
            self.target,
            self.cpu_model,
            self.threads,
            self.rustflags,
            self.avx512,
            self.isa_notes,
            self.logical_cpus,
            self.memory_bytes
                .map_or_else(|| "unknown".into(), |bytes| bytes.to_string()),
            self.memory_limit_bytes
                .map_or_else(|| "unknown".into(), |bytes| bytes.to_string()),
            pcs_bench_core::SchemeId::Akita.commit_url(),
            pcs_bench_core::SchemeId::AkitaPr466.commit_url(),
            pcs_bench_core::SchemeId::Greyhound.commit_url(),
            pcs_bench_core::SchemeId::Rokoko.commit_url(),
        );
        fs::write(path, body).with_context(|| format!("write {}", path.display()))
    }
}

pub(crate) trait ProvenanceExt {
    fn write(&self, path: &Path) -> Result<()>;
}

fn git_revision() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
}

fn rustc_version() -> Result<String> {
    let output = Command::new("rustc")
        .arg("-V")
        .output()
        .context("rustc -V")?;
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
