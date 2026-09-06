#!/usr/bin/env python3
"""Copy lattice JSONL records, dropping selected scheme/payload pairs."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def parse_drop(spec: str) -> tuple[str, set[int]]:
    scheme, _, payloads = spec.partition(":")
    if not scheme or not payloads:
        raise argparse.ArgumentTypeError(
            f"expected scheme:payload,payload got {spec!r}"
        )
    return scheme, {int(item) for item in payloads.split(",") if item}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--drop",
        action="append",
        default=[],
        metavar="SCHEME:PAYLOADS",
        help="scheme token and comma-separated payload exponents to omit",
    )
    parser.add_argument("src", type=Path)
    parser.add_argument("dst", type=Path)
    args = parser.parse_args()
    drops = [parse_drop(item) for item in args.drop]
    lines: list[str] = []
    for line in args.src.read_text().splitlines():
        record = json.loads(line)
        if any(
            record.get("scheme") == scheme
            and record.get("payload_log2") in payloads
            for scheme, payloads in drops
        ):
            continue
        lines.append(line)
    args.dst.parent.mkdir(parents=True, exist_ok=True)
    args.dst.write_text("\n".join(lines) + ("\n" if lines else ""))


if __name__ == "__main__":
    main()
