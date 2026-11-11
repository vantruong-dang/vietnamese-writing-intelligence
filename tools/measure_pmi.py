#!/usr/bin/env python3
"""Đo PMI / NPMI / PMI+gate trên 10 ca real-word trước khi viết Rust.

Dùng bigram_lm_c1.tsv (count>=1) + unigram_lm.tsv (count thật).
Không dùng H để tune threshold; đây là chẩn đoán khả năng của PMI.

Kết quả: PMI thô đúng 10/10 trên 10 ca H, nhưng trên VSEC:
  min_pmi=2/margin=4: recall 37,2% ở FPR 37,0%
  min_pmi=6/margin=10: recall 5,6% ở FPR 3,0%
  min_pmi=8/margin=10: recall 0,4% ở FPR 1,33%
→ PMI bias cặp hiếm, FPR nổ; không dùng.
"""
from __future__ import annotations

import math
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BIGRAM = ROOT / "data_processed" / "bigram_lm_c1.tsv"
UNIGRAM = ROOT / "data_processed" / "unigram_lm.tsv"

# (id, prev, correct, wrong, next)
CASES = [
    ("H29", "cơn", "bão", "bảo", "đang"),
    ("H30", "đi", "tắm", "tám", "biển"),
    ("H31", "thuốc", "sau", "sao", "bữa"),
    ("H32", "buổi", "sáng", "sang", None),
    ("H33", "bên", "cạnh", "canh", None),
    ("H34", "chiếc", "váy", "vay", "màu"),
    ("H35", "sớm", "để", "đẻ", "nấu"),
    ("H36", "nay", "lạnh", "nạnh", None),
    ("H37", "theo", "giấy", "dấy", "tờ"),
    ("H38", "một", "cuốn", "cuống", "sách"),
]


def load():
    uni: dict[str, int] = {}
    for line in UNIGRAM.read_text(encoding="utf-8").splitlines():
        if line.startswith("#") or not line.strip():
            continue
        w, c = line.split("\t")
        uni[w] = int(c)
    big: dict[tuple[str, str], int] = {}
    for line in BIGRAM.read_text(encoding="utf-8").splitlines():
        if line.startswith("#") or not line.strip():
            continue
        a, b, c = line.split("\t")
        big[(a, b)] = int(c)
    return uni, big, sum(uni.values())


def pmi(uni, big, n, a, b):
    c = big.get((a, b), 0)
    ca = uni.get(a, 0)
    cb = uni.get(b, 0)
    if c == 0 or ca == 0 or cb == 0:
        return None, c
    return math.log(c * n / (ca * cb)), c


def npmi(uni, big, n, a, b):
    p, c = pmi(uni, big, n, a, b)
    if p is None or c == 0:
        return None, c
    # NPMI = PMI / -log P(a,b)
    denom = -math.log(c / n)
    return (p / denom if denom else None), c


def best_side(uni, big, n, prev, word, nxt, fn, gate=0):
    vals = []
    for a, b in ((prev, word), (word, nxt)):
        if not a or not b:
            continue
        val, c = fn(uni, big, n, a, b)
        if val is not None and c >= gate:
            vals.append((val, c, a, b))
    if not vals:
        return None
    return max(vals, key=lambda x: x[0])


def main():
    uni, big, n = load()
    print(f"N={n:,} bigram_pairs={len(big):,} unigram={len(uni):,}")

    for label, fn, gate in [
        ("PMI raw", pmi, 0),
        ("PMI+gate>=3", pmi, 3),
        ("NPMI raw", npmi, 0),
        ("NPMI+gate>=3", npmi, 3),
    ]:
        print(f"\n=== {label} ===")
        wins = 0
        margins = []
        for cid, prev, correct, wrong, nxt in CASES:
            bc = best_side(uni, big, n, prev, correct, nxt, fn, gate)
            bw = best_side(uni, big, n, prev, wrong, nxt, fn, gate)
            sc = bc[0] if bc else None
            sw = bw[0] if bw else None
            ok = sc is not None and (sw is None or sc > sw)
            if ok:
                wins += 1
            if sc is not None and sw is not None:
                margins.append(sc - sw)
            def fmt(x):
                return "None" if x is None else f"{x:+.3f}"
            print(
                f"  {cid}: correct={fmt(sc)} wrong={fmt(sw)} "
                f"{'OK' if ok else 'FAIL'}"
            )
        mean_margin = sum(margins) / len(margins) if margins else 0
        print(f"  -> đúng {wins}/10 | mean_margin={mean_margin:.3f}")


if __name__ == "__main__":
    import sys

    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    main()
