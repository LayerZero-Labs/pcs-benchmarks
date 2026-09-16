//! Akita benchmark adapter.
//!
//! End-to-end Criterion benchmarks live in `benches/e2e.rs`. Keeping this
//! crate separate prevents future PCS dependencies and feature sets from
//! contaminating one another.

/// Exact Akita source revision measured by this adapter.
pub const IMPLEMENTATION_REVISION: &str = "c0cb822f28b7b9efe85b1924b029d36e13cdf516";

/// Pinned upstream or supplemental artifact embedded so the executable hash also
/// identifies the schedules used by setup and verification.
pub fn schedule_catalog<Cfg: akita_config::CommitmentConfig>(
    num_vars: usize,
) -> Result<akita_config::TrustedScheduleCatalog<Cfg>, String> {
    let bytes: &[u8] = match (Cfg::schedule_family_name(), num_vars) {
        ("fp32_dense", 22) => {
            include_bytes!("../../../vendor/akita-catalogs/c0cb822f/fp32-nv22/fp32_dense.aks")
        }
        ("fp32_dense", 24) => {
            include_bytes!("../../../vendor/akita-catalogs/c0cb822f/fp32-nv24/fp32_dense.aks")
        }
        ("fp64_dense", 21) => {
            include_bytes!("../../../vendor/akita-catalogs/c0cb822f/fp64-nv21/fp64_dense.aks")
        }
        ("fp64_dense", 23) => {
            include_bytes!("../../../vendor/akita-catalogs/c0cb822f/fp64-nv23/fp64_dense.aks")
        }
        ("fp64_dense", 25) => {
            include_bytes!("../../../vendor/akita-catalogs/c0cb822f/fp64-nv25/fp64_dense.aks")
        }
        ("fp64_dense", 27) => {
            include_bytes!("../../../vendor/akita-catalogs/c0cb822f/fp64-nv27/fp64_dense.aks")
        }
        ("fp128_dense", 20) => {
            include_bytes!("../../../vendor/akita-catalogs/c0cb822f/fp128-nv20/fp128_dense.aks")
        }
        ("fp128_dense", 22) => {
            include_bytes!("../../../vendor/akita-catalogs/c0cb822f/fp128-nv22/fp128_dense.aks")
        }
        ("fp32_dense", _) => {
            include_bytes!("../../../third_party/akita/artifacts/schedules/fp32_dense.aks")
        }
        ("fp64_dense", _) => {
            include_bytes!("../../../third_party/akita/artifacts/schedules/fp64_dense.aks")
        }
        ("fp128_dense", _) => {
            include_bytes!("../../../third_party/akita/artifacts/schedules/fp128_dense.aks")
        }
        ("fp32_dense_recursive", _) => include_bytes!(
            "../../../third_party/akita/artifacts/schedules/fp32_dense_recursive.aks"
        ),
        ("fp64_dense_recursive", _) => include_bytes!(
            "../../../third_party/akita/artifacts/schedules/fp64_dense_recursive.aks"
        ),
        ("fp128_dense_recursive", _) => include_bytes!(
            "../../../third_party/akita/artifacts/schedules/fp128_dense_recursive.aks"
        ),
        (family, _) => return Err(format!("unsupported Akita schedule family: {family}")),
    };
    akita_config::TrustedScheduleCatalog::<Cfg>::from_artifact_bytes(bytes)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::schedule_catalog;
    use akita_config::{
        proof_optimized::{fp128, fp32, fp64},
        CommitmentConfig, RecursiveCommitmentConfig,
    };
    use akita_types::{AkitaScheduleLookupKey, PolynomialGroupLayout};

    fn check<Cfg: CommitmentConfig>(offset: usize, first_offload: Option<usize>) {
        for payload in [27, 29, 31, 33, 35] {
            let nv = payload - offset;
            let catalog = schedule_catalog::<Cfg>(nv).expect("valid pinned artifact");
            let key = AkitaScheduleLookupKey::single(PolynomialGroupLayout::singleton(nv));
            let row = catalog.resolve_key(&key).expect("benchmark row exists");
            let offload = row
                .schedule()
                .recursive_folds
                .iter()
                .any(|fold| fold.params.setup_prefix().is_some());
            assert_eq!(offload, first_offload.is_some_and(|first| payload >= first));
        }
    }

    #[test]
    fn pinned_artifacts_cover_benchmark_sizes_and_offload_availability() {
        check::<fp32::Dense>(5, None);
        check::<fp64::Dense>(6, None);
        check::<fp128::Dense>(7, None);
        check::<RecursiveCommitmentConfig<fp32::Dense>>(5, Some(29));
        check::<RecursiveCommitmentConfig<fp64::Dense>>(6, Some(31));
        check::<RecursiveCommitmentConfig<fp128::Dense>>(7, Some(33));
    }
}
