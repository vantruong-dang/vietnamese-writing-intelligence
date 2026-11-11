#!/usr/bin/env python3
"""Chạy sản phẩm thật (_core) trên benchmark và xuất predictions cho score_benchmark.

Mặc định chỉ tính finding có severity critical/major/minor là "flagged"
(bỏ advisory/suggestion để không tính gợi ý phong cách là lỗi).

  python tools/predict_benchmark.py --bench data_processed/bench/blind.jsonl \
      --out data_processed/bench/blind_pred.jsonl
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "src"))

from vn_writing_intel import _core  # noqa: E402

ACTIONABLE = {"critical", "major", "minor"}


def apply_fixes(text: str, findings) -> str:
    fixes = []
    for f in findings:
        fx = getattr(f, "fix", None)
        if fx is None:
            continue
        if f.severity not in ACTIONABLE:
            continue
        fixes.append((fx.start, fx.end, fx.replacement))
    fixes.sort(key=lambda x: (x[0], -(x[1] - x[0])))
    chosen = []
    last = 0
    for s, e, rep in fixes:
        if s >= last and e > s:
            chosen.append((s, e, rep))
            last = e
    for s, e, rep in reversed(chosen):
        text = text[:s] + rep + text[e:]
    return text


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bench", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    auditor = _core.Auditor()
    rows = [
        json.loads(line)
        for line in Path(args.bench).read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8") as f:
        for row in rows:
            findings = auditor.audit(row["text"])
            actionable = [x for x in findings if x.severity in ACTIONABLE]
            spans = [[x.start, x.end] for x in actionable]
            pred = {
                "id": row["id"],
                "flagged": bool(actionable),
                "fixed_text": apply_fixes(row["text"], actionable),
                "spans": spans,
            }
            f.write(json.dumps(pred, ensure_ascii=False) + "\n")
    print(f"wrote {len(rows)} predictions -> {out}")


if __name__ == "__main__":
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    main()
