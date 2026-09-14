#!/usr/bin/env python3
"""Patch a pinned Akita checkout so a `fp{32,64,128}`-dense setup-offload catalog can compile.

Adds the `fp<width>_dense_recursive` generated family, Cargo features, and
`RecursiveCommitmentConfig<fp<width>::Dense>` catalog wiring. Idempotent.

Upstream ships no dense recursive family for any width; `fp32` was added first
and this generalises the same wiring to `fp64` and `fp128`.
"""

from __future__ import annotations

import argparse
import re
import shutil
import sys
from pathlib import Path

# Nominal payloads 2^27..2^35 map to nv = payload - 5/6/7 for fp32/fp64/fp128.
# fp32 keeps the wider set it was generated with so its catalog is unchanged.
FIELD_KEYS = {
    "fp32": [20, 22, 24, 26, 28, 30],
    "fp64": [21, 23, 25, 27, 29],
    "fp128": [20, 22, 24, 26, 28],
}


class Field:
    def __init__(self, name: str) -> None:
        self.name = name
        self.upper = name.upper()
        self.keys = FIELD_KEYS[name]
        self.module = f"{name}_dense_recursive"
        self.feature = f"{name}-dense-recursive"
        self.schedules = f"{self.upper}_DENSE_RECURSIVE_SCHEDULES"
        self.keys_const = f"{self.upper}_DENSE_RECURSIVE_KEYS"
        self.catalog_id = f"{name}-dense-recursive"
        self.base_module = f"{name}_dense"
        self.notice = (
            f"// Modified for PCS Benchmarks to add the {name}-dense-recursive"
            " (setup-offload) family.\n"
        )


def replace_once(text: str, old: str, new: str, path: Path) -> str:
    if new.strip() in text and old not in text:
        return text
    if old not in text:
        raise SystemExit(f"{path}: expected block not found:\n{old[:160]}")
    return text.replace(old, new, 1)


def patch_generated_families(root: Path, f: Field) -> None:
    path = root / "crates/akita-planner/src/generated_families.rs"
    text = path.read_text()
    if f.notice not in text:
        text = f.notice + text
    if f.keys_const not in text:
        marker = f"const {f.upper}_DENSE_KEYS:"
        idx = text.find(marker)
        if idx < 0:
            raise SystemExit(f"{path}: {marker} not found")
        rows = "".join(f"    PolynomialGroupLayout::singleton({nv}),\n" for nv in f.keys)
        block = f"const {f.keys_const}: &[PolynomialGroupLayout] = &[\n{rows}];\n\n"
        text = text[:idx] + block + text[idx:]
    if f'"{f.module}"' not in text:
        start = text.find(f'    family_row!(\n        "{f.base_module}",\n')
        if start < 0:
            raise SystemExit(f"{path}: {f.base_module} family_row not found")
        end = text.find("    ),\n", start)
        if end < 0:
            raise SystemExit(f"{path}: unterminated {f.base_module} family_row")
        end += len("    ),\n")
        row = (
            "    family_row!(\n"
            "        recursive,\n"
            f'        "{f.module}",\n'
            f'        "{f.schedules}",\n'
            f'        "{f.catalog_id}",\n'
            f"        {f.keys_const},\n"
            f"        RecursiveCommitmentConfig<{f.name}::Dense>,\n"
            f"        {f.name}::Dense,\n"
            "        no_grouped_requests\n"
            "    ),\n"
        )
        text = text[:end] + row + text[end:]
    path.write_text(text)
    print(f"{path}: {f.module} family")


def patch_config_cargo(root: Path, f: Field) -> None:
    path = root / "crates/akita-config/Cargo.toml"
    text = path.read_text()
    feature = (
        f"schedules-{f.feature} = [\n"
        f'  "schedules-{f.name}-dense",\n'
        f'  "akita-schedules/{f.feature}",\n'
        "]\n"
    )
    if f"schedules-{f.feature}" not in text:
        anchor = f'schedules-{f.name}-dense = ["akita-schedules/{f.name}-dense"]\n'
        text = replace_once(text, anchor, feature + anchor, path)
    insertion = f'  "schedules-{f.feature}",\n'
    needle = '  "schedules-fp128-onehot-recursive",\n'
    if "all-schedules" in text and insertion not in text:
        text = replace_once(text, needle, insertion + needle, path)
    path.write_text(text)
    print(f"{path}: schedules-{f.feature}")


def patch_schedules_cargo(root: Path, f: Field) -> None:
    path = root / "crates/akita-schedules/Cargo.toml"
    text = path.read_text()
    if f"{f.feature} = []" not in text:
        anchor = f"{f.name}-dense = []\n"
        text = replace_once(text, anchor, anchor + f"{f.feature} = []\n", path)
    insertion = f'  "{f.feature}",\n'
    if insertion not in text:
        text = replace_once(
            text, '  "fp128-onehot-recursive",\n', insertion + '  "fp128-onehot-recursive",\n', path
        )
    path.write_text(text)
    print(f"{path}: {f.feature} feature")


def patch_recursive_commitment(root: Path, f: Field) -> None:
    path = root / "crates/akita-config/src/recursive_commitment.rs"
    text = path.read_text()
    head = text.split("use std::any::TypeId;")[0]
    if f'feature = "schedules-{f.feature}"' not in head:
        old = '    feature = "schedules-fp128-onehot-recursive-multi-chunk-w8r2"'
        new = (
            '    feature = "schedules-fp128-onehot-recursive-multi-chunk-w8r2",\n'
            f'    feature = "schedules-{f.feature}"'
        )
        text = replace_once(text, old, new, path)
    if f"{f.module}_table" not in text:
        branch = (
            f'        #[cfg(feature = "schedules-{f.feature}")]\n'
            "        {\n"
            f"            if TypeId::of::<Cfg>() == TypeId::of::<crate::proof_optimized::{f.name}::Dense>() {{\n"
            f"                return Some(akita_schedules::{f.module}_table());\n"
            "            }\n"
            "        }\n"
        )
        text = replace_once(text, "        None\n    }\n}\n", branch + "        None\n    }\n}\n", path)
    path.write_text(text)
    print(f"{path}: {f.name} Dense recursive catalog")


def patch_generated_mod(root: Path, f: Field) -> None:
    path = root / "crates/akita-schedules/src/generated/mod.rs"
    text = path.read_text()
    if f.module in text:
        print(f"{path}: wiring already present")
        return
    mod_anchor = f'#[cfg(feature = "{f.name}-dense")]\npub mod {f.base_module};\n'
    mod_decl = f'#[cfg(feature = "{f.feature}")]\npub mod {f.module};\n'
    text = replace_once(text, mod_anchor, mod_anchor + mod_decl, path)
    fn_anchor = (
        f'#[cfg(feature = "{f.name}-dense")]\npub fn {f.base_module}_table() -> GeneratedScheduleTable {{\n'
    )
    table_fn = (
        f'#[cfg(feature = "{f.feature}")]\n'
        f"pub fn {f.module}_table() -> GeneratedScheduleTable {{\n"
        "    GeneratedScheduleTable {\n"
        f"        entries: {f.module}::{f.schedules},\n"
        f"        identity: {f.module}::CATALOG_IDENTITY,\n"
        "    }\n"
        "}\n"
    )
    text = replace_once(text, fn_anchor, table_fn + "\n" + fn_anchor, path)
    path.write_text(text)
    print(f"{path}: {f.module} wiring")


def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


def derive_stub(root: Path, f: Field) -> str:
    """Build a compilable empty catalog from the base dense family of the same width.

    Only the three recursive-specific identity fields differ; the planner
    overwrites the whole file during generation, so this only has to compile.
    """
    base = root / f"crates/akita-schedules/src/generated/{f.base_module}.rs"
    text = base.read_text()
    pattern = re.compile(
        rf"pub\(crate\) static {f.upper}_DENSE_SCHEDULES: &\[GeneratedFoldScheduleEntry\] = &\[.*?\n\];\n",
        re.S,
    )
    replacement = (
        f"pub(crate) static {f.schedules}: &[GeneratedFoldScheduleEntry] = &[];\n"
    )
    text, count = pattern.subn(replacement, text)
    if count != 1:
        raise SystemExit(f"{base}: expected one schedules array, found {count}")
    subs = [
        (f'family_name: "{f.base_module}",', f'family_name: "{f.module}",'),
        (
            "recursive_setup_search_policy: crate::RecursiveSetupSearchPolicy::Exhaustive,",
            "recursive_setup_search_policy: crate::RecursiveSetupSearchPolicy::RootAndFirstChildV1,",
        ),
        ("recursive_setup_planning: false,", "recursive_setup_planning: true,"),
        (
            "selection_policy: SelectionPolicyId::MinFirstDirectSetupThenPayloadV2,",
            "selection_policy: SelectionPolicyId::MinPaddedSetupEnvelopeThenFirstDirectThenPayloadV3,",
        ),
    ]
    for old, new in subs:
        if old not in text:
            raise SystemExit(f"{base}: identity field not found: {old}")
        text = text.replace(old, new, 1)
    text = re.sub(r"key_count: \d+,", "key_count: 0,", text, count=1)
    text = re.sub(r"key_digest: \d+,", "key_digest: 0,", text, count=1)
    header = (
        f"// Generated stub for PCS Benchmarks until extend-akita-dense-offload.sh fills rows.\n"
        f"// Modified for PCS Benchmarks to add the nv="
        f"{','.join(str(nv) for nv in f.keys)} setup-offload schedule family.\n"
    )
    return header + text


def ensure_stub_catalog(root: Path, f: Field) -> None:
    path = root / f"crates/akita-schedules/src/generated/{f.module}.rs"
    if path.exists():
        print(f"{path}: keeping existing catalog")
        return
    overlay = repo_root() / f"vendor/akita-catalogs/{f.module}-main.rs"
    if overlay.exists():
        shutil.copyfile(overlay, path)
        print(f"{path}: copied overlay catalog from {overlay}")
        return
    path.write_text(derive_stub(root, f))
    print(f"{path}: wrote empty stub catalog derived from {f.base_module}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("akita_checkout", type=Path)
    parser.add_argument("--field", choices=sorted(FIELD_KEYS), default="fp32")
    args = parser.parse_args()
    root = args.akita_checkout.resolve()
    if not (root / ".git").exists():
        raise SystemExit(f"{root} is not a git checkout")
    f = Field(args.field)
    patch_generated_families(root, f)
    patch_config_cargo(root, f)
    patch_schedules_cargo(root, f)
    patch_recursive_commitment(root, f)
    patch_generated_mod(root, f)
    ensure_stub_catalog(root, f)


if __name__ == "__main__":
    try:
        main()
    except BrokenPipeError:
        sys.exit(0)
