#!/usr/bin/env python3
"""Tạo dataset Curriculum Learning 2 pha từ D1 cũ, D1 v3 và VSEC.

Pha 1: 50% D1 cũ + 45% D1 v3 synthetic + 5% VSEC.
Pha 2: 92% D1 v3 synthetic + 8% VSEC.
"""

from __future__ import annotations

import argparse
import json
import random
from pathlib import Path


def read_lines(path: Path) -> list[str]:
    with path.open(encoding="utf-8") as handle:
        return [line for line in handle if line.strip()]


def filter_synthetic(lines: list[str]) -> list[str]:
    output = []
    for line in lines:
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        if record.get("source") != "vsec_human":
            output.append(line)
    return output


def sample(lines: list[str], count: int, rng: random.Random) -> list[str]:
    if not lines:
        return []
    output = []
    while len(output) < count:
        output.append(rng.choice(lines))
    return output


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--old", type=Path, default=Path("data_processed/neural_diverse/train.jsonl"))
    parser.add_argument("--new", type=Path, default=Path("data_processed/neural_mixed_v3/train.jsonl"))
    parser.add_argument("--vsec-train", type=Path, default=Path("data_processed/vsec_train.jsonl"))
    parser.add_argument("--val", type=Path, default=Path("data_processed/neural_mixed_v3/val.jsonl"))
    parser.add_argument("--vsec-holdout", type=Path, default=Path("data_processed/vsec_test.jsonl"))
    parser.add_argument("--phase1-size", type=int, default=400_000)
    parser.add_argument("--phase2-size", type=int, default=200_000)
    parser.add_argument("--out", type=Path, default=Path("data_processed/neural_curriculum"))
    parser.add_argument("--seed", type=int, default=42)
    args = parser.parse_args()

    rng = random.Random(args.seed)
    old = filter_synthetic(read_lines(args.old))
    new = filter_synthetic(read_lines(args.new))
    vsec = read_lines(args.vsec_train)
    print(f"old={len(old):,} new_synthetic={len(new):,} vsec_train={len(vsec):,}")

    phase1 = (
        sample(old, int(args.phase1_size * 0.50), rng)
        + sample(new, int(args.phase1_size * 0.45), rng)
        + sample(vsec, int(args.phase1_size * 0.05), rng)
    )
    phase2 = (
        sample(new, int(args.phase2_size * 0.92), rng)
        + sample(vsec, int(args.phase2_size * 0.08), rng)
    )
    rng.shuffle(phase1)
    rng.shuffle(phase2)

    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "phase1_train.jsonl").write_text("".join(phase1), encoding="utf-8")
    (args.out / "phase2_train.jsonl").write_text("".join(phase2), encoding="utf-8")
    (args.out / "val.jsonl").write_text("".join(read_lines(args.val)), encoding="utf-8")
    (args.out / "vsec_holdout.jsonl").write_text(
        "".join(read_lines(args.vsec_holdout)), encoding="utf-8"
    )
    print(
        json.dumps(
            {
                "phase1": len(phase1),
                "phase2": len(phase2),
                "val": len(read_lines(args.val)),
                "vsec_holdout": len(read_lines(args.vsec_holdout)),
            }
        )
    )


if __name__ == "__main__":
    main()
