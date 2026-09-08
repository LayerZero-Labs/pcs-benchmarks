#!/usr/bin/env python3
"""Patch a pinned Akita checkout so fp32-dense setup-offload catalogs can compile.

Adds the `fp32_dense_recursive` generated family, Cargo features, and
`RecursiveCommitmentConfig<fp32::Dense>` catalog wiring. Idempotent.
"""

from __future__ import annotations

import argparse
import shutil
import sys
from pathlib import Path

FAMILY_KEYS = """const FP32_DENSE_RECURSIVE_KEYS: &[PolynomialGroupLayout] = &[
    PolynomialGroupLayout::singleton(20),
    PolynomialGroupLayout::singleton(22),
    PolynomialGroupLayout::singleton(24),
    PolynomialGroupLayout::singleton(26),
    PolynomialGroupLayout::singleton(28),
    PolynomialGroupLayout::singleton(30),
];
"""

FAMILY_ROW = """    family_row!(
        recursive,
        "fp32_dense_recursive",
        "FP32_DENSE_RECURSIVE_SCHEDULES",
        "fp32-dense-recursive",
        FP32_DENSE_RECURSIVE_KEYS,
        RecursiveCommitmentConfig<fp32::Dense>,
        fp32::Dense,
        no_grouped_requests
    ),
"""

NOTICE = "// Modified for PCS Benchmarks to add the fp32-dense-recursive (setup-offload) family.\n"

STUB_CATALOG = '''// Generated stub for PCS Benchmarks until `extend-akita-fp32-dense-offload.sh` fills rows.
// Modified for PCS Benchmarks to add the nv=20,22,24,26,28,30 setup-offload schedule family.
#[allow(unused_imports)]
use super::{
    BlockGeometry, ChunkedWitnessCfg, CommitmentPayloadMode, CommitmentRingDims,
    DecompositionParams, GeneratedFoldCore, GeneratedFoldScheduleEntry, GeneratedFrozenGroup,
    GeneratedGroup, GeneratedMatrix, GeneratedPrecommittedGroup, GeneratedRecursiveFold,
    GeneratedRootFold, GeneratedScheduleCatalogIdentity, GeneratedSetupPrefix,
    GeneratedTerminalFold, GroupCommitPhaseParams, InnerCommitMatrixParams,
    OuterCommitMatrixParams, PlannerCostModelId, PolynomialGroupLayout, RingDimensionScheduleMode,
    SelectionPolicyId, SelectiveL2ResponseModelId, SisL2TableDigest, SisModulusProfileId,
    SisSecurityPolicyId, SisTableDigest,
};

#[rustfmt::skip]
pub(crate) static FP32_DENSE_RECURSIVE_SCHEDULES: &[GeneratedFoldScheduleEntry] = &[];

#[rustfmt::skip]
pub(crate) static CATALOG_SUFFIX_DIMENSIONS: &[usize] = &[64, 128];
#[rustfmt::skip]
pub(crate) static CATALOG_POTENTIAL_A_DIMENSIONS: &[usize] = &[64, 128, 256, 512, 1024, 2048];
#[rustfmt::skip]
pub(crate) static CATALOG_POTENTIAL_B_DIMENSIONS: &[usize] = &[64, 128, 256];
#[rustfmt::skip]
pub(crate) static CATALOG_POTENTIAL_D_DIMENSIONS: &[usize] = &[64, 128, 256];
#[rustfmt::skip]
pub(crate) static CATALOG_RING_DIMENSIONS: &[usize] = &[128, 256, 512, 1024, 2048];
#[rustfmt::skip]
pub(crate) static CATALOG_IDENTITY: GeneratedScheduleCatalogIdentity = GeneratedScheduleCatalogIdentity {
    family_name: "fp32_dense_recursive",
    protocol_epoch: 4,
    cost_model: PlannerCostModelId::ExactPayloadAndSetupEnvelope,
    selective_l2_response_model: SelectiveL2ResponseModelId::TypedProtocolMomentsV1,
    selection_policy: SelectionPolicyId::MinFirstDirectSetupThenPayloadV2,
    recursive_split_search_policy: crate::RecursiveSplitSearchPolicy::BoundedBalancedExtremesV1,
    recursive_setup_search_policy: crate::RecursiveSetupSearchPolicy::RootAndFirstChildV1,
    setup_field_budget: None,
    min_offloaded_witness_contraction: 3,
    sis_modulus_profile: SisModulusProfileId::Q32Offset99,
    sis_security_policy: SisSecurityPolicyId::Quantum128BitADPS16,
    sis_table_digest: SisTableDigest([0xe9, 0xf5, 0x73, 0xac, 0xce, 0xc4, 0xe5, 0xcd, 0x01, 0x7b, 0x9f, 0xc8, 0x9d, 0x69, 0x24, 0xf6, 0x9d, 0xbc, 0x75, 0x4a, 0x46, 0x5e, 0x46, 0x53, 0xd5, 0x48, 0x2f, 0x75, 0x15, 0x92, 0x7e, 0x90]),
    sis_l2_table_digest: SisL2TableDigest([0xa1, 0xcc, 0x0a, 0x06, 0x08, 0x97, 0x44, 0x14, 0x5b, 0x61, 0x91, 0x9c, 0xf0, 0x01, 0xea, 0x26, 0x0c, 0x95, 0xa4, 0xbb, 0xa1, 0x61, 0xff, 0xda, 0xff, 0x55, 0x39, 0x1d, 0x0d, 0xfa, 0x10, 0x2a]),
    decomposition: DecompositionParams { log_basis: 3, log_commit_bound: 32, log_open_bound: None },
    claim_ext_degree: 4,
    chal_ext_degree: 4,
    inner_basis_range: (3, 10),
    opening_basis_range: (3, 6),
    witness_chunk: ChunkedWitnessCfg { num_chunks: 1, num_activated_levels: 0 },
    recursive_setup_planning: true,
    ring_dimension_schedule_mode: RingDimensionScheduleMode::AdaptiveDimension { num_search_levels: 2, suffix_dimensions: CATALOG_SUFFIX_DIMENSIONS, potential_a_dimensions: CATALOG_POTENTIAL_A_DIMENSIONS, potential_b_dimensions: CATALOG_POTENTIAL_B_DIMENSIONS, potential_d_dimensions: CATALOG_POTENTIAL_D_DIMENSIONS },
    ring_dimensions: CATALOG_RING_DIMENSIONS,
    ring_challenge_config_digest: 17970171637396625756,
    key_count: 0,
    key_digest: 0,
};
'''

MOD_DECL = """#[cfg(feature = "fp32-dense-recursive")]
pub mod fp32_dense_recursive;
"""

TABLE_FN = """#[cfg(feature = "fp32-dense-recursive")]
pub fn fp32_dense_recursive_table() -> GeneratedScheduleTable {
    GeneratedScheduleTable {
        entries: fp32_dense_recursive::FP32_DENSE_RECURSIVE_SCHEDULES,
        identity: fp32_dense_recursive::CATALOG_IDENTITY,
    }
}
"""


def replace_once(text: str, old: str, new: str, path: Path) -> str:
    if new.strip() in text and old not in text:
        return text
    if old not in text:
        raise SystemExit(f"{path}: expected block not found:\n{old[:120]}")
    return text.replace(old, new, 1)


def ensure_notice(text: str) -> str:
    if NOTICE in text:
        return text
    return NOTICE + text


def patch_generated_families(root: Path) -> None:
    path = root / "crates/akita-planner/src/generated_families.rs"
    text = path.read_text()
    text = ensure_notice(text)
    if "FP32_DENSE_RECURSIVE_KEYS" not in text:
        marker = "const FP32_DENSE_KEYS:"
        idx = text.find(marker)
        if idx < 0:
            raise SystemExit(f"{path}: FP32_DENSE_KEYS not found")
        text = text[:idx] + FAMILY_KEYS + "\n" + text[idx:]
    if '"fp32_dense_recursive"' not in text:
        text = replace_once(
            text,
            """    family_row!(
        "fp32_onehot",
        "FP32_ONEHOT_SCHEDULES",
        "fp32-onehot",
        FP32_ONEHOT_KEYS,
        fp32::OneHot,
        fp32_onehot_grouped_requests
    ),
];
""",
            FAMILY_ROW
            + """    family_row!(
        "fp32_onehot",
        "FP32_ONEHOT_SCHEDULES",
        "fp32-onehot",
        FP32_ONEHOT_KEYS,
        fp32::OneHot,
        fp32_onehot_grouped_requests
    ),
];
""",
            path,
        )
    path.write_text(text)
    print(f"{path}: fp32_dense_recursive family")


def patch_config_cargo(root: Path) -> None:
    path = root / "crates/akita-config/Cargo.toml"
    text = path.read_text()
    feature = """schedules-fp32-dense-recursive = [
  "schedules-fp32-dense",
  "akita-schedules/fp32-dense-recursive",
]
"""
    if "schedules-fp32-dense-recursive" not in text:
        text = replace_once(
            text,
            'schedules-fp32-dense = ["akita-schedules/fp32-dense"]\n',
            feature + 'schedules-fp32-dense = ["akita-schedules/fp32-dense"]\n',
            path,
        )
    needle = '  "schedules-fp128-onehot-recursive",\n'
    insertion = '  "schedules-fp32-dense-recursive",\n'
    if "all-schedules" in text and insertion not in text:
        text = replace_once(text, needle, insertion + needle, path)
    path.write_text(text)
    print(f"{path}: schedules-fp32-dense-recursive")


def patch_schedules_cargo(root: Path) -> None:
    path = root / "crates/akita-schedules/Cargo.toml"
    text = path.read_text()
    if 'fp32-dense-recursive = []' not in text:
        text = replace_once(
            text,
            "fp32-dense = []\n",
            "fp32-dense = []\nfp32-dense-recursive = []\n",
            path,
        )
    insertion = '  "fp32-dense-recursive",\n'
    if insertion not in text:
        text = replace_once(
            text,
            '  "fp128-onehot-recursive",\n',
            insertion + '  "fp128-onehot-recursive",\n',
            path,
        )
    path.write_text(text)
    print(f"{path}: fp32-dense-recursive feature")


def patch_recursive_commitment(root: Path) -> None:
    path = root / "crates/akita-config/src/recursive_commitment.rs"
    text = path.read_text()
    old_cfg = """#[cfg(any(
    feature = "schedules-fp128-onehot-recursive",
    feature = "schedules-fp128-onehot-recursive-multi-chunk-w8r2"
))]
use std::any::TypeId;
"""
    new_cfg = """#[cfg(any(
    feature = "schedules-fp128-onehot-recursive",
    feature = "schedules-fp128-onehot-recursive-multi-chunk-w8r2",
    feature = "schedules-fp32-dense-recursive"
))]
use std::any::TypeId;
"""
    if 'feature = "schedules-fp32-dense-recursive"' not in text.split("use std::any::TypeId;")[0]:
        text = replace_once(text, old_cfg, new_cfg, path)
    branch = """        #[cfg(feature = "schedules-fp32-dense-recursive")]
        {
            if TypeId::of::<Cfg>() == TypeId::of::<crate::proof_optimized::fp32::Dense>() {
                return Some(akita_schedules::fp32_dense_recursive_table());
            }
        }
        None
"""
    if "fp32_dense_recursive_table" not in text:
        text = replace_once(
            text,
            """        None
    }
}
""",
            branch
            + """    }
}
""",
            path,
        )
    path.write_text(text)
    print(f"{path}: fp32 Dense recursive catalog")


def patch_generated_mod(root: Path) -> None:
    path = root / "crates/akita-schedules/src/generated/mod.rs"
    text = path.read_text()
    if "fp32_dense_recursive" not in text:
        text = replace_once(
            text,
            """#[cfg(feature = "fp32-dense")]
pub mod fp32_dense;
""",
            MOD_DECL
            + """#[cfg(feature = "fp32-dense")]
pub mod fp32_dense;
""",
            path,
        )
        text = replace_once(
            text,
            """#[cfg(feature = "fp32-dense")]
pub fn fp32_dense_table() -> GeneratedScheduleTable {
""",
            TABLE_FN
            + """
#[cfg(feature = "fp32-dense")]
pub fn fp32_dense_table() -> GeneratedScheduleTable {
""",
            path,
        )
        path.write_text(text)
        print(f"{path}: fp32_dense_recursive wiring")
    else:
        print(f"{path}: wiring already present")


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def ensure_stub_catalog(root: Path) -> None:
    path = root / "crates/akita-schedules/src/generated/fp32_dense_recursive.rs"
    if path.exists():
        print(f"{path}: keeping existing catalog")
        return
    overlay = repo_root() / "vendor/akita-catalogs/fp32_dense_recursive-main.rs"
    if overlay.exists():
        shutil.copyfile(overlay, path)
        print(f"{path}: copied overlay catalog from {overlay}")
        return
    path.write_text(STUB_CATALOG)
    print(f"{path}: wrote empty stub catalog")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("akita_checkout", type=Path)
    args = parser.parse_args()
    root = args.akita_checkout.resolve()
    if not (root / ".git").exists():
        raise SystemExit(f"{root} is not a git checkout")
    patch_generated_families(root)
    patch_config_cargo(root)
    patch_schedules_cargo(root)
    patch_recursive_commitment(root)
    patch_generated_mod(root)
    ensure_stub_catalog(root)


if __name__ == "__main__":
    try:
        main()
    except BrokenPipeError:
        sys.exit(0)
