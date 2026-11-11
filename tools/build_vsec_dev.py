#!/usr/bin/env python3
"""Trích lỗi real-word đơn từ từ VSEC làm dev lỗi gán tay.

VSEC: mỗi record có annotations theo âm tiết; lỗi có is_correct=False và
alternative_syllables. Chỉ giữ ca mà cả từ sai và từ đúng đều là một âm tiết
hợp lệ trong Viet74K (đây mới là real-word, không phải typo OOV).

Kết quả: 9.341 record → 11.202 lỗi → 7.641 ca real-word đơn từ.
Dùng để đo recall context bigram; kết quả hiện tại 3,2% @ FPR 0,67% → không ship.
"""
from __future__ import annotations

import json
import re
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VSEC = ROOT / "data_raw" / "vsec.jsonl"
OUT = ROOT / "data_processed" / "vsec_realword.tsv"
WORD_RE = re.compile(r"^[^\W\d_]+$", re.UNICODE)


def load_valid() -> set[str]:
    valid = set()
    for line in (ROOT / "data_raw" / "Viet74K.txt").read_text(encoding="utf-8").splitlines():
        for w in re.split(r"[\s-]+", unicodedata.normalize("NFC", line.strip().lower())):
            if w:
                valid.add(w)
    return valid


def main():
    valid = load_valid()
    rows = []
    stats = {"records": 0, "errors": 0, "single_realword": 0}
    for line in VSEC.open(encoding="utf-8"):
        record = json.loads(line)
        stats["records"] += 1
        anns = record.get("annotations", [])
        sentence = " ".join(a.get("current_syllable", "") for a in anns).strip()
        for a in anns:
            if a.get("is_correct", True):
                continue
            stats["errors"] += 1
            wrong = unicodedata.normalize("NFC", a.get("current_syllable", "")).strip()
            alts = a.get("alternative_syllables") or []
            if not alts:
                continue
            correct = unicodedata.normalize("NFC", alts[0]).strip()
            if not WORD_RE.match(wrong) or not WORD_RE.match(correct):
                continue
            wl, cl = wrong.lower(), correct.lower()
            if wl == cl or wl not in valid or cl not in valid:
                continue
            rows.append((sentence, wrong, correct))
            stats["single_realword"] += 1

    OUT.parent.mkdir(exist_ok=True)
    with OUT.open("w", encoding="utf-8") as f:
        for i, (sent, wrong, correct) in enumerate(rows, 1):
            f.write(f"{i}\t{sent}\t{wrong}\t{correct}\n")
    print(stats)
    print(f"wrote {len(rows)} -> {OUT}")


if __name__ == "__main__":
    import sys

    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    main()
