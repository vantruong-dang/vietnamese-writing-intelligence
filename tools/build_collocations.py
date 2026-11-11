#!/usr/bin/env python3
"""Sinh whitelist collocation mạnh từ corpus (không viết tay).

Tiêu chí:
  count(w1,w2) >= 10
  PMI(w1,w2) >= 3.0
  w1, w2 là âm tiết hợp lệ (Viet74K)
  ít nhất một trong hai từ có neighbor cùng họ ngữ âm trong từ điển

Xuất data_processed/collocations.tsv: w1, w2, count, pmi.
Đo size + coverage trên 10 ca H và 500 ca VSEC trước khi quyết định viết module.

Kết quả (không ship module):
- count>=10, PMI>=3: 21.980 cặp; cover H 7/10; VSEC strong 337/500.
  Nhưng module advisory thử nghiệm: recall 23%, FPR 47% → không dùng.
- count>=50, PMI>=6 + chỉ tone-variant: 1.760 cặp; cover H 4/10;
  VSEC strong 115/500; module recall 5,8% ở FPR 0,67%.
  Dưới ngưỡng cover H >=5/10 → không ship.
"""
from __future__ import annotations

import math
import re
import unicodedata
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BIGRAM = ROOT / "data_processed" / "bigram_lm_c1.tsv"
UNIGRAM = ROOT / "data_processed" / "unigram_lm.tsv"
OUT = ROOT / "data_processed" / "collocations.tsv"
VSEC = ROOT / "data_processed" / "vsec_realword.tsv"

MIN_COUNT = 50
MIN_PMI = 6.0
WORD_RE = re.compile(r"[^\W\d_]+", re.UNICODE)

# Nhóm phụ âm đầu dùng để tìm "neighbor cùng họ ngữ âm".
INITIAL_GROUPS = [
    ("ngh", "NG"), ("ng", "NG"),
    ("gh", "G"), ("g", "G"),
    ("ch", "TR"), ("tr", "TR"),
    ("gi", "D"), ("d", "D"), ("r", "D"), ("v", "D"),
    ("s", "X"), ("x", "X"),
    ("l", "N"), ("n", "N"),
    ("c", "K"), ("k", "K"), ("q", "K"),
    ("ph", "PH"), ("th", "TH"), ("kh", "KH"), ("nh", "NH"),
]

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


def strip_tones(text: str) -> str:
    return "".join(
        c
        for c in unicodedata.normalize("NFD", text)
        if c not in "\u0300\u0301\u0303\u0309\u0323"
    )


def phonetic_key(word: str) -> str:
    base = unicodedata.normalize("NFC", strip_tones(word.lower()))
    for prefix, group in INITIAL_GROUPS:
        if base.startswith(prefix):
            return group + base[len(prefix) :]
    return "?" + base


def load_valid() -> set[str]:
    valid = set()
    for line in (ROOT / "data_raw" / "Viet74K.txt").read_text(encoding="utf-8").splitlines():
        for w in re.split(r"[\s-]+", unicodedata.normalize("NFC", line.strip().lower())):
            if w:
                valid.add(w)
    return valid


def main():
    uni: dict[str, int] = {}
    for line in UNIGRAM.read_text(encoding="utf-8").splitlines():
        if line.startswith("#") or not line.strip():
            continue
        w, c = line.split("\t")
        uni[w] = int(c)
    n = sum(uni.values())

    big: list[tuple[str, str, int]] = []
    for line in BIGRAM.read_text(encoding="utf-8").splitlines():
        if line.startswith("#") or not line.strip():
            continue
        a, b, c = line.split("\t")
        big.append((a, b, int(c)))

    valid = load_valid()

    # Bản đồ họ ngữ âm: key -> set các từ
    buckets: dict[str, set[str]] = defaultdict(set)
    for w in valid:
        buckets[phonetic_key(w)].add(w)
    has_neighbor = {w for bucket in buckets.values() if len(bucket) > 1 for w in bucket}

    whitelist: dict[tuple[str, str], tuple[int, float]] = {}
    for w1, w2, c in big:
        if c < MIN_COUNT or w1 not in valid or w2 not in valid:
            continue
        if w1 not in has_neighbor and w2 not in has_neighbor:
            continue
        c1, c2 = uni.get(w1, 0), uni.get(w2, 0)
        if c1 == 0 or c2 == 0:
            continue
        pmi = math.log(c * n / (c1 * c2))
        if pmi >= MIN_PMI:
            whitelist[(w1, w2)] = (c, pmi)

    with OUT.open("w", encoding="utf-8") as f:
        f.write("# w1\tw2\tcount\tpmi\n")
        for (w1, w2), (c, pmi) in sorted(
            whitelist.items(), key=lambda kv: (-kv[1][0], -kv[1][1])
        ):
            f.write(f"{w1}\t{w2}\t{c}\t{pmi:.4f}\n")

    print(f"whitelist_size={len(whitelist):,} -> {OUT}")
    print("top20:")
    for (w1, w2), (c, pmi) in sorted(
        whitelist.items(), key=lambda kv: (-kv[1][1], -kv[1][0])
    )[:20]:
        print(f"  {w1} {w2}: count={c} pmi={pmi:.2f}")

    hit_h = 0
    for cid, prev, correct, _wrong, nxt in CASES:
        ok = (prev, correct) in whitelist or (correct, nxt) in whitelist
        hit_h += 1 if ok else 0
        print(f"  {cid}: {'YES' if ok else 'no'} ({prev} {correct} / {correct} {nxt})")
    print(f"covers_H={hit_h}/10")

    rows = []
    for line in VSEC.read_text(encoding="utf-8").splitlines():
        parts = line.split("\t")
        if len(parts) != 4:
            continue
        _, sent, wrong, correct = parts
        rows.append((sent, wrong.lower(), correct.lower()))
        if len(rows) >= 500:
            break
    hit_v = 0
    for sent, wrong, correct in rows:
        words = [w.lower() for w in WORD_RE.findall(sent)]
        if wrong not in words:
            continue
        idx = words.index(wrong)
        prev = words[idx - 1] if idx > 0 else None
        nxt = words[idx + 1] if idx + 1 < len(words) else None
        if (prev and (prev, correct) in whitelist) or (nxt and (correct, nxt) in whitelist):
            hit_v += 1
    print(f"covers_VSEC_strong={hit_v}/{len(rows)}")


if __name__ == "__main__":
    import sys

    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    main()
