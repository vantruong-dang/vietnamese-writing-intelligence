#!/usr/bin/env python3
"""Đo gate trước khi viết context_spellchecker.

- dev_general chỉ từ human_gold + journalism
- OOV bigram rate, unigram coverage
- margin per-case cho 10 real-word H
- unigram sau prune
- perplexity k=0.01/0.05/0.1/0.5 trên dev_general
"""
from __future__ import annotations

import math
import re
import unicodedata
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TSV = ROOT / "data_processed" / "bigram_lm.tsv"
UNI_TSV = ROOT / "data_processed" / "unigram_lm.tsv"
DEV = ROOT / "data_processed" / "dev_hj.txt"
TOKEN_RE = re.compile(r"[^\W\d_]+", re.UNICODE)

# (ca, prev, correct, wrong, next)
CASES = [
    ("H29", "cơn", "bão", "bảo", "đang"),
    ("H30", "đi", "tắm", "tám", "biển"),
    ("H31", "thuốc", "sau", "sao", "bữa"),
    ("H32", "buổi", "sáng", "sang", None),
    ("H33", "bên", "cạnh", "canh", None),
    ("H34", "chiếc", "váy", "vay", "màu"),
    ("H35", "sớm", "để", "đẻ", "nấu"),
    ("H36", "rất", "lạnh", "nạnh", None),
    ("H37", "theo", "giấy", "dấy", "tờ"),
    ("H38", "một", "cuốn", "cuống", "sách"),
]


def load_bigrams(threshold: int):
    bg: dict[tuple[str, str], int] = {}
    out: Counter[str] = Counter()
    with TSV.open(encoding="utf-8") as f:
        for line in f:
            if line.startswith("#"):
                continue
            a, b, c = line.rstrip("\n").split("\t")
            n = int(c)
            if n >= threshold:
                bg[(a, b)] = n
                out[a] += n
    return bg, out


def dev_lines():
    return [
        TOKEN_RE.findall(unicodedata.normalize("NFC", line).lower())
        for line in DEV.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]


def ctx_score(bg, out, prev, word, next_word):
    total = 0.0
    observed = 0
    if prev and (prev, word) in bg and out[prev] > 0:
        total += math.log(bg[(prev, word)] / out[prev])
        observed += 1
    if next_word and (word, next_word) in bg and out[word] > 0:
        total += math.log(bg[(word, next_word)] / out[word])
        observed += 1
    if observed == 0:
        return -15.0, 0
    return total / observed, observed


def main():
    bg, out = load_bigrams(3)
    V = len(out)
    print(f"bigram_count>=3={len(bg)} out_vocab={V}")

    # unigram sau prune
    uni_rows = []
    for line in UNI_TSV.read_text(encoding="utf-8").splitlines():
        if line.startswith("#") or not line.strip():
            continue
        w, c = line.split("\t")
        uni_rows.append((w, int(c)))
    for thr in (2, 3, 5):
        print(f"unigram count>={thr}: {sum(1 for _, c in uni_rows if c >= thr):,}")

    # dev general
    dev = dev_lines()
    tokens = [t for sent in dev for t in sent]
    print(f"dev_general lines={len(dev)} tokens={len(tokens)}")

    pairs = 0
    missing = 0
    covered_tokens = 0
    for sent in dev:
        for t in sent:
            if t in out:
                covered_tokens += 1
        for a, b in zip(sent, sent[1:]):
            pairs += 1
            if (a, b) not in bg:
                missing += 1
    print(
        f"OOV_bigram_rate={missing}/{pairs}={missing/pairs*100:.1f}% "
        f"unigram_coverage={covered_tokens/len(tokens)*100:.1f}%"
    )

    # perplexity
    total_log = 0.0
    count = 0
    for sent in dev:
        prev = "<S>"
        for w in sent:
            if prev in out:
                p = (bg.get((prev, w), 0) + 0.01) / (out[prev] + 0.01 * V)
            else:
                p = 1.0 / V
            # dùng p > 0; với k thay đổi bên dưới
            count += 1
            prev = w
    for k in (0.01, 0.05, 0.1, 0.5):
        total_log = 0.0
        count = 0
        for sent in dev:
            prev = "<S>"
            for w in sent:
                if prev in out:
                    p = (bg.get((prev, w), 0) + k) / (out[prev] + k * V)
                else:
                    p = 1.0 / V
                total_log += math.log(p)
                count += 1
                prev = w
        print(f"perplexity k={k:<4} = {math.exp(-total_log/count):.2f}")

    # margin per case
    print("\ncase  prev      correct  wrong    score_corr  score_wrong  margin  obs_c obs_w")
    margins = []
    for cid, prev, correct, wrong, nxt in CASES:
        sc, oc = ctx_score(bg, out, prev, correct, nxt)
        sw, ow = ctx_score(bg, out, prev, wrong, nxt)
        margins.append(sc - sw)
        print(
            f"{cid:<5} {prev:<9} {correct:<8} {wrong:<8} "
            f"{sc:>10.2f} {sw:>11.2f} {sc-sw:>7.2f} {oc:>5} {ow:>5}"
        )
    print(f"mean_margin={sum(margins)/len(margins):.2f} min={min(margins):.2f}")


if __name__ == "__main__":
    import sys

    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    main()
