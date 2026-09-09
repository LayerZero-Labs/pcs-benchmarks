//! Persisted benchmark records.

use crate::hash::HashSchemeId;
use crate::lattice::{worker_memory_limit_bytes, SchemeId};
use crate::workload::Workload;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Outcome of one attempted lattice-eval cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    /// Timed phases completed and the proof verified.
    Ok,
    /// The implementation has no native parameter set for this payload.
    Unsupported,
    /// The process exceeded the documented memory ceiling.
    Oom,
    /// The worker failed for a reason other than an expected gap.
    Error,
}

/// True when a worker error string is an out-of-memory failure, including
/// Labrador's "Not enough memory" and Rust's allocation panic.
#[must_use]
pub fn looks_like_oom(detail: Option<&str>) -> bool {
    let Some(text) = detail else {
        return false;
    };
    let text = text.to_ascii_lowercase();
    text.contains("memory limit")
        || text.contains("cannot allocate")
        || text.contains("not enough memory")
        || text.contains("memory allocation")
        || text.contains("out of memory")
        || text.contains("oom")
}

/// True when Greyhound/Labrador rejected the instance because the inner
/// Ajtai commitment is not SIS-secure (not an out-of-memory failure).
#[must_use]
pub fn looks_like_greyhound_sis(detail: Option<&str>) -> bool {
    let Some(text) = detail else {
        return false;
    };
    let text = text.to_ascii_lowercase();
    text.contains("inner commitments not secure")
        || text.contains("cannot make inner commitments secure")
        || text.contains("cannot make outer commitments secure")
}

/// One raw, process-isolated benchmark observation from a Criterion path.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    /// Scheme and implementation name.
    pub implementation: String,
    /// Exact source revision of the implementation.
    pub implementation_revision: String,
    /// Workload used for this observation.
    pub workload: Workload,
    /// Zero-based measured sample index.
    pub sample: u32,
    /// Phase timings in nanoseconds.
    pub timings_ns: BTreeMap<String, u64>,
    /// Serialized proof size, when supported.
    pub proof_bytes: Option<u64>,
    /// Peak resident set size, when available.
    pub peak_rss_bytes: Option<u64>,
    /// Machine and build provenance.
    pub provenance: Provenance,
}

/// Slim JSON object emitted by a scheme worker. The runner fills provenance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerOutput {
    /// Cell outcome.
    pub status: RunStatus,
    /// Human-readable reason for a non-ok status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_detail: Option<String>,
    /// Native `log2 N` actually executed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log2_n: Option<u32>,
    /// Phase timings in nanoseconds.
    #[serde(default)]
    pub timings_ns: BTreeMap<String, u64>,
    /// Serialized proof size, when supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_bytes: Option<u64>,
    /// Serialized commitment size, when supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commitment_bytes: Option<u64>,
    /// Claimed evaluation bytes sent separately from commitment and proof.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evaluation_bytes: Option<u64>,
    /// Excluded public/verifier context bytes, when materialized and known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_context_bytes: Option<u64>,
    /// Reusable preprocessing / CRS state size, when supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_bytes: Option<u64>,
    /// Peak resident set size, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peak_rss_bytes: Option<u64>,
}

/// One lattice-eval cell attempt. This is the unit the comparison tables consume.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LatticeRecord {
    /// Cell outcome.
    pub status: RunStatus,
    /// Human-readable reason for a non-ok status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_detail: Option<String>,
    /// Scheme identity.
    pub scheme: SchemeId,
    /// Exact source revision of the implementation.
    pub implementation_revision: String,
    /// Target payload exponent: payload = `2^{payload_log2}` bits.
    pub payload_log2: u32,
    /// Native `log2 N` when the scheme ran or would run a supported size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log2_n: Option<u32>,
    /// Field label, for example `2^{32}-99`.
    pub field: String,
    /// Implementation parameter name such as `p-26` or `fp32-dense`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_param: Option<String>,
    /// Worker thread count. Lattice-eval is always single-threaded.
    pub threads: u32,
    /// Zero-based measured sample index.
    pub sample: u32,
    /// Whether this sample was discarded as warmup.
    pub warmup: bool,
    /// Phase timings in nanoseconds (`setup`, `commit`, `open`, `verify`).
    #[serde(default)]
    pub timings_ns: BTreeMap<String, u64>,
    /// Serialized opening-proof size, when supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_bytes: Option<u64>,
    /// Serialized commitment size, when supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commitment_bytes: Option<u64>,
    /// Claimed evaluation bytes sent separately from commitment and proof.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evaluation_bytes: Option<u64>,
    /// Excluded public/verifier context bytes, when materialized and known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_context_bytes: Option<u64>,
    /// Reusable preprocessing / CRS state size, when supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_bytes: Option<u64>,
    /// Peak resident set size, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peak_rss_bytes: Option<u64>,
    /// Machine and build provenance.
    pub provenance: Provenance,
}

/// One hash-eval cell attempt. Same measurement contract as [`LatticeRecord`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HashRecord {
    /// Cell outcome.
    pub status: RunStatus,
    /// Human-readable reason for a non-ok status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_detail: Option<String>,
    /// Scheme identity.
    pub scheme: HashSchemeId,
    /// Exact source revision of the implementation.
    pub implementation_revision: String,
    /// Target payload exponent: payload = `2^{payload_log2}` bits.
    pub payload_log2: u32,
    /// Native `log2 N` actually executed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log2_n: Option<u32>,
    /// Field label, for example `2^{32}-99` or `2^{31}-2^{24}+1`.
    pub field: String,
    /// Implementation parameter name such as `fp32-dense` or `whir-capacity-128`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_param: Option<String>,
    /// Worker thread count (1 or 8 in the headline table).
    pub threads: u32,
    /// Zero-based measured sample index.
    pub sample: u32,
    /// Whether this sample was discarded as warmup.
    pub warmup: bool,
    /// Phase timings in nanoseconds (`setup`, `commit`, `open`, `verify`).
    #[serde(default)]
    pub timings_ns: BTreeMap<String, u64>,
    /// Serialized opening-proof size, when supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_bytes: Option<u64>,
    /// Serialized commitment size, when supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commitment_bytes: Option<u64>,
    /// Claimed evaluation bytes sent separately from commitment and proof.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evaluation_bytes: Option<u64>,
    /// Excluded public/verifier context bytes, when materialized and known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_context_bytes: Option<u64>,
    /// Reusable preprocessing / CRS state size, when supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_bytes: Option<u64>,
    /// Peak resident set size, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peak_rss_bytes: Option<u64>,
    /// Machine and build provenance.
    pub provenance: Provenance,
}

/// Environment fields needed to decide whether observations are comparable.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    /// Benchmark harness revision.
    pub harness_revision: String,
    /// UTC timestamp at which the run was started.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp_utc: Option<String>,
    /// Complete runner command for this run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_command: Option<String>,
    /// Rust compiler version.
    pub rustc_version: String,
    /// Operating system and architecture.
    pub target: String,
    /// CPU model.
    pub cpu_model: String,
    /// Privacy-preserving hash used to distinguish physical benchmark hosts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine_id_hash: Option<String>,
    /// Number of worker threads.
    pub threads: u32,
    /// Effective compiler flags.
    pub rustflags: String,
    /// Whether the host advertised AVX-512F.
    #[serde(default)]
    pub avx512: bool,
    /// Human-readable ISA note, for example `AVX-512F (-C target-cpu=native)`.
    #[serde(default)]
    pub isa_notes: String,
    /// Logical CPU count of the host.
    #[serde(default)]
    pub logical_cpus: u32,
    /// Host RAM in bytes, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_bytes: Option<u64>,
    /// Worker `ulimit -v` ceiling in bytes (90% of host RAM when measured).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_limit_bytes: Option<u64>,
    /// SHA-256 digest of the exact executable invoked for this observation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executable_sha256: Option<String>,
    /// Compiler version that built this worker (Rust or C, as applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker_compiler_version: Option<String>,
    /// SHA-256 digest of the dependency lockfile used for the worker build.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lockfile_sha256: Option<String>,
    /// Exact command used to build the worker executable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_command: Option<String>,
    /// Deterministic workload seed requested for this observation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workload_seed: Option<u64>,
    /// Whether repetitions vary workload seeds or hold one workload fixed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed_mode: Option<String>,
}

impl Provenance {
    /// Placeholder provenance for unit tests and dry-run matrix rows.
    #[must_use]
    pub fn test_fixture() -> Self {
        Self {
            harness_revision: "test".into(),
            timestamp_utc: None,
            run_command: None,
            rustc_version: "test".into(),
            target: "test".into(),
            cpu_model: "test".into(),
            machine_id_hash: None,
            threads: 1,
            rustflags: String::new(),
            avx512: true,
            isa_notes: "AVX-512F".into(),
            logical_cpus: 1,
            memory_bytes: None,
            memory_limit_bytes: None,
            executable_sha256: None,
            worker_compiler_version: None,
            lockfile_sha256: None,
            build_command: None,
            workload_seed: None,
            seed_mode: None,
        }
    }

    /// Worker virtual-memory ceiling actually applied, or 90% of recorded host RAM.
    #[must_use]
    pub fn resolved_memory_limit_bytes(&self) -> Option<u64> {
        self.memory_limit_bytes
            .filter(|bytes| *bytes > 0)
            .or_else(|| {
                self.memory_bytes
                    .filter(|bytes| *bytes > 0)
                    .map(worker_memory_limit_bytes)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{looks_like_greyhound_sis, looks_like_oom};

    #[test]
    fn sis_rejection_is_not_classified_as_oom() {
        let detail = Some(
            "Greyhound failed with status Some(1): ERROR in polcom_reduce(): Inner commitments not secure",
        );
        assert!(looks_like_greyhound_sis(detail));
        assert!(!looks_like_oom(detail));
        assert!(looks_like_greyhound_sis(Some(
            "Cannot make inner commitments secure"
        )));
        assert!(looks_like_greyhound_sis(Some(
            "Cannot make outer commitments secure"
        )));
        assert!(looks_like_oom(Some("ERROR: Not enough memory")));
        assert!(!looks_like_greyhound_sis(Some("ERROR: Not enough memory")));
    }
}
