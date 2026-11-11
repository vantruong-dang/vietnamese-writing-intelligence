#!/usr/bin/env python3
"""Xây Bigram LM từ corpus người thật trong data_raw/.

Nguyên tắc:
- Chỉ dùng văn xuôi người viết: human_gold, journalism, literature, news_extra,
  Leipzig news, legal sample; loại ai_synthetic/từ điển/wordlist.
- Cap Wikipedia 5M token để không chi phối phân phối bigram.
- Split 90/10 theo từng nguồn; dev_hj giữ nguyên human_gold+journalism.
- Lưu TSV count thô; Rust tự tính smoothing.

Kết quả với mix hiện tại (wiki 5M + Leipzig 10M + legal 2M):
  train 15,9M token; bigram count>=3 447.704; dev_hj OOV 17,0%;
  H clean OOV 4,9%; perplexity dev_hj k=0.05 546,22.
  VSEC real-word chỉ recall 3,2% ở FPR 0,67% → context-lm vẫn tắt.

--trigram: đo trước khi triển khai. Kết quả: 5.582.761 trigram; count>=3
  734.380 (~8,4 MB); dev_hj OOV 50,0% (>=1) / 66,9% (>=3); 4/7 ca thất bại
  có trigram đúng mạnh. OOV >40% → không triển khai trigram.
"""
from __future__ import annotations

import itertools
import math
import random
import re
import sys
import unicodedata
from collections import Counter
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

ROOT = Path(__file__).resolve().parent.parent
RAW = ROOT / "data_raw"
OUT = ROOT / "data_processed"
TOKEN_RE = re.compile(r"[^\W\d_]+", re.UNICODE)

# Phụ âm đầu có thể bị tách khỏi vần trong human_gold ("m ột", "c ầu").
INITIAL_CLUSTERS = {
    "b", "c", "ch", "d", "đ", "g", "gh", "gi", "h", "k", "kh", "l", "m",
    "n", "ng", "ngh", "nh", "p", "ph", "q", "qu", "r", "s", "t", "th",
    "tr", "v", "x",
}
VOWEL_START = set(
    "aăâeêioôơuưy"
    "àáảãạằắẳẵặầấẩẫậèéẻẽẹềếểễệìíỉĩị"
    "òóỏõọồốổỗộờớởỡợùúủũụừứửữựỳýỷỹỵ"
)


def load_valid_syllables() -> set[str]:
    path = RAW / "Viet74K.txt"
    if not path.exists():
        return set()
    valid: set[str] = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        for word in re.split(r"[\s-]+", unicodedata.normalize("NFC", line.strip().lower())):
            if word:
                valid.add(word)
    return valid


def repair_tokens(tokens: list[str], valid: set[str]) -> tuple[list[str], int]:
    """Ghép lại phụ âm đầu bị tách khỏi vần nếu kết quả là âm tiết hợp lệ."""
    out: list[str] = []
    repaired = 0
    i = 0
    while i < len(tokens):
        if i + 1 < len(tokens):
            merged = tokens[i] + tokens[i + 1]
            if (
                tokens[i] in INITIAL_CLUSTERS
                and tokens[i + 1][0] in VOWEL_START
                and merged in valid
            ):
                out.append(merged)
                repaired += 1
                i += 2
                continue
        out.append(tokens[i])
        i += 1
    return out, repaired

# (tên nguồn, danh sách file glob)
# (tên, files, max_tokens). max_tokens=None giữ toàn bộ.
SOURCES = [
    ("human_gold", [RAW / "human_gold.txt"], None),
    ("journalism", [RAW / "journalism_corpus.txt"], None),
    ("literature", sorted((RAW / "literature_wikisource").glob("*.txt")), None),
    ("news_extra", sorted((RAW / "news_extra").glob("*.txt")), None),
    (
        "ud_text",
        [
            RAW / "vi_vtb-ud-train.conllu",
            RAW / "vi_vtb-ud-dev.conllu",
            RAW / "vi_vtb-ud-test.conllu",
        ],
        None,
    ),
    ("news_fresh", [RAW / "news_fresh.txt"] if (RAW / "news_fresh.txt").exists() else [], None),
    # Cap Wikipedia để không chi phối phân phối bigram.
    (
        "viwiki",
        [RAW / "viwiki_clean.txt"] if (RAW / "viwiki_clean.txt").exists() else [],
        5_000_000,
    ),
    (
        "leipzig_news",
        [RAW / "leipzig_news.txt"] if (RAW / "leipzig_news.txt").exists() else [],
        10_000_000,
    ),
    (
        "legal",
        [RAW / "legal_sample.txt"] if (RAW / "legal_sample.txt").exists() else [],
        2_000_000,
    ),
]

# Cặp đúng cần cho 10 ca real-word H (chẩn đoán coverage, không dùng để tune)
REAL_WORD_PAIRS = [
    ("cơn", "bão"),
    ("đi", "tắm"),
    ("sau", "bữa"),
    ("buổi", "sáng"),
    ("bên", "cạnh"),
    ("chiếc", "váy"),
    ("để", "nấu"),
    ("rất", "lạnh"),
    ("giấy", "tờ"),
    ("cuốn", "sách"),
]

# 20 câu đúng H để ước lượng OOV bigram rate
CLEAN_SENTS = [
    "Anh ấy đứng thứ ba trong cuộc thi chạy bộ.",
    "Các cơ quan nhà nước phải công khai thông tin.",
    "Cuộc họp bắt đầu lúc 8 giờ 30 phút sáng nay.",
    "Công ty vừa ký hợp đồng trị giá 2,5 tỷ đồng.",
    "Hà Nội, ngày 03 tháng 12 năm 2026",
    "Căn cứ khoản 2 Điều 7 Luật Lưu trữ năm 2024.",
    "Trận Hà Nội FC vs Hải Phòng diễn ra tối nay.",
    "Bà ngoại kể chuyện cổ tích cho các cháu nghe.",
    "Thủ tướng Chính phủ chủ trì cuộc họp sáng nay.",
    "Thành phố Hồ Chí Minh là trung tâm kinh tế lớn của cả nước.",
    "Bà cụ khóa cửa rồi đi chợ, tay cầm một chiếc khóa cũ.",
    "Hôm qua tôi mới biết mai trời sẽ mưa.",
    "Quý khách vui lòng đợi trong giây lát.",
    "Tổng chi phí khoảng 1.250.000 đồng.",
    "Nhân dân ta luôn đoàn kết một lòng.",
    "Cô giáo cho cả lớp nghỉ sớm vì trời mưa to.",
    "Anh ấy chạy rất nhanh nên về nhất.",
    "Kính gửi: các sở, ban, ngành thuộc tỉnh.",
    "Kính gửi: Các sở, ban, ngành thuộc tỉnh.",
    "Ông Trần Quang Minh sinh năm 1960 tại Cần Thơ.",
]


def normalize(line: str) -> list[str]:
    line = unicodedata.normalize("NFC", line).lower()
    return TOKEN_RE.findall(line)


def valid_line(tokens: list[str]) -> bool:
    if len(tokens) < 3:
        return False
    letter_ratio = sum(1 for t in tokens if any(c.isalpha() for c in t)) / len(tokens)
    return letter_ratio >= 0.9


def collect_source(paths: list[Path], valid: set[str]) -> tuple[list[list[str]], int]:
    lines: list[list[str]] = []
    repaired_total = 0
    for path in paths:
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError as exc:
            print(f"[!] Bỏ qua {path}: không phải UTF-8 ({exc})")
            continue
        for raw_line in text.splitlines():
            if path.suffix == ".conllu":
                if not raw_line.startswith("# text = "):
                    continue
                raw_line = raw_line[len("# text = ") :]
            tokens = normalize(raw_line)
            tokens, repaired = repair_tokens(tokens, valid)
            repaired_total += repaired
            if valid_line(tokens):
                lines.append(tokens)
    return lines, repaired_total


def split_source(lines: list[list[str]], dev_ratio: float, rng: random.Random):
    idx = list(range(len(lines)))
    rng.shuffle(idx)
    n_dev = max(1, int(len(lines) * dev_ratio))
    dev_idx = set(idx[:n_dev])
    train = [lines[i] for i in range(len(lines)) if i not in dev_idx]
    dev = [lines[i] for i in idx[:n_dev]]
    return train, dev


def perplexity(dev, unigram, bigram, k, V, N):
    total_log = 0.0
    count = 0
    for sent in dev:
        prev = "<S>"
        for w in sent:
            if prev in unigram:
                p = (bigram.get((prev, w), 0) + k) / (unigram[prev] + k * V)
            else:
                p = (unigram.get(w, 0) + k) / (N + k * V)
            total_log += math.log(p)
            count += 1
            prev = w
    return math.exp(-total_log / count)


def main():
    rng = random.Random(42)
    train_all: list[list[str]] = []
    dev_all: list[list[str]] = []
    dev_general: list[list[str]] = []
    dev_hj: list[list[str]] = []
    dev_viwiki: list[list[str]] = []
    seen: set[tuple[str, ...]] = set()
    valid = load_valid_syllables()
    print(f"valid_syllables={len(valid)}")
    print("=== SPLIT THEO NGUỒN (90/10, dedup toàn cục) ===")
    for name, paths, max_tokens in SOURCES:
        existing = [p for p in paths if p.exists()]
        if not existing:
            print(f"{name:<14} (không có file)")
            continue
        raw_lines, repaired = collect_source(existing, valid)
        lines: list[list[str]] = []
        duplicates = 0
        for tokens in raw_lines:
            key = tuple(tokens)
            if key in seen:
                duplicates += 1
                continue
            seen.add(key)
            lines.append(tokens)
        if max_tokens is not None:
            rng.shuffle(lines)
            capped: list[list[str]] = []
            total = 0
            for sent in lines:
                if total + len(sent) > max_tokens:
                    break
                capped.append(sent)
                total += len(sent)
            lines = capped

        train, dev = split_source(lines, 0.10, rng)
        train_all.extend(train)
        dev_all.extend(dev)
        if name == "viwiki":
            dev_viwiki.extend(dev)
        else:
            dev_general.extend(dev)
        if name in ("human_gold", "journalism"):
            dev_hj.extend(dev)
        tokens = sum(len(s) for s in lines)
        print(
            f"{name:<14} files={len(existing):<4} lines={len(lines):<6} "
            f"dup={duplicates:<6} repaired={repaired:<6} tokens={tokens:<8} "
            f"train={len(train):<6} dev={len(dev)}"
        )

    print(
        f"\nTỔNG train_sents={len(train_all)} dev_sents={len(dev_all)} "
        f"train_tokens={sum(len(s) for s in train_all)} "
        f"dev_tokens={sum(len(s) for s in dev_all)}"
    )

    unigram: Counter[str] = Counter()
    bigram: Counter[tuple[str, str]] = Counter()
    for sent in train_all:
        unigram.update(sent)
        for a, b in itertools.pairwise(sent):
            bigram[(a, b)] += 1

    N = sum(unigram.values())
    V = len(unigram)
    print(f"unigram_distinct={V} tokens={N}")

    print("\n=== BIGRAM THEO NGƯỠNG PRUNE ===")
    sizes = {}
    for threshold in (1, 2, 3, 5):
        count = sum(1 for c in bigram.values() if c >= threshold)
        sizes[threshold] = count
        print(f"count>={threshold}: {count:,}")

    OUT.mkdir(exist_ok=True)
    with (OUT / "dev_general.txt").open("w", encoding="utf-8") as f:
        for sent in dev_general:
            f.write(" ".join(sent) + "\n")
    with (OUT / "dev_viwiki.txt").open("w", encoding="utf-8") as f:
        for sent in dev_viwiki:
            f.write(" ".join(sent) + "\n")
    with (OUT / "dev_hj.txt").open("w", encoding="utf-8") as f:
        for sent in dev_hj:
            f.write(" ".join(sent) + "\n")
    print(f"dev_hj={len(dev_hj)} lines")
    print(
        f"dev_general={len(dev_general)} lines | dev_viwiki={len(dev_viwiki)} lines"
    )

    kept: Counter[tuple[str, str]] = Counter(
        {k: c for k, c in bigram.items() if c >= 2}
    )
    tsv = OUT / "bigram_lm.tsv"
    with tsv.open("w", encoding="utf-8") as f:
        f.write("# w1\tw2\tcount\n")
        for (a, b), c in kept.most_common():
            f.write(f"{a}\t{b}\t{c}\n")
    size_mb = tsv.stat().st_size / 1024 / 1024
    print(f"TSV size (count>=2): {size_mb:.2f} MB -> {tsv}")

    if "--emit-c1" in sys.argv:
        c1 = OUT / "bigram_lm_c1.tsv"
        with c1.open("w", encoding="utf-8") as f:
            f.write("# w1\tw2\tcount\n")
            for (a, b), c in bigram.most_common():
                f.write(f"{a}\t{b}\t{c}\n")
        print(
            f"TSV size (count>=1): {c1.stat().st_size/1024/1024:.2f} MB -> {c1}"
        )

    uni_tsv = OUT / "unigram_lm.tsv"
    with uni_tsv.open("w", encoding="utf-8") as f:
        f.write("# word\tcount\n")
        for w, c in unigram.most_common():
            f.write(f"{w}\t{c}\n")

    print("\n=== OOV BIGRAM RATE TRÊN DEV (câu sạch) ===")
    total = missing = 0
    for sent in dev_all:
        for a, b in itertools.pairwise(sent):
            total += 1
            if (a, b) not in kept:
                missing += 1
    dev_oov = missing / total if total else 0
    print(f"dev_pairs={total} missing={missing} ({dev_oov*100:.1f}%)")

    print("\n=== OOV BIGRAM RATE TRÊN 20 CÂU H SẠCH ===")
    total = missing = 0
    for sent in CLEAN_SENTS:
        tokens = normalize(sent)
        for a, b in itertools.pairwise(tokens):
            total += 1
            if (a, b) not in kept:
                missing += 1
    h_oov = missing / total if total else 0
    print(f"h_pairs={total} missing={missing} ({h_oov*100:.1f}%)")

    print("\n=== REAL-WORD COVERAGE (cặp đúng có trong train LM) ===")
    covered = 0
    for a, b in REAL_WORD_PAIRS:
        ok = (a, b) in kept
        covered += 1 if ok else 0
        print(f"  {a} {b}: {ok}")
    print(f"coverage={covered}/{len(REAL_WORD_PAIRS)}")

    print("\n=== PERPLEXITY DEV ===")
    for k in (0.1, 0.5, 1.0):
        ppl = perplexity(dev_all, unigram, bigram, k, V, N)
        print(f"k={k:<4} perplexity={ppl:,.2f}")

    print("\n=== MẪU BIGRAM PHỔ BIẾN ===")
    for (a, b), c in bigram.most_common(10):
        print(f"  {a} {b}: {c}")

    if "--trigram" in sys.argv:
        measure_trigrams(train_all, dev_hj)


# Các trigram của 7 ca bigram thất bại: (đúng, sai)
TRIGRAM_CASES = [
    ("H30", ("đi", "tắm", "biển"), ("đi", "tám", "biển")),
    ("H31", ("thuốc", "sau", "bữa"), ("thuốc", "sao", "bữa")),
    ("H34", ("chiếc", "váy", "màu"), ("chiếc", "vay", "màu")),
    ("H35", ("sớm", "để", "nấu"), ("sớm", "đẻ", "nấu")),
    ("H36", ("nay", "rất", "lạnh"), ("nay", "rất", "nạnh")),
    ("H37", ("theo", "giấy", "tờ"), ("theo", "dấy", "tờ")),
    ("H38", ("một", "cuốn", "sách"), ("một", "cuống", "sách")),
]


def measure_trigrams(train_all: list[list[str]], dev_hj: list[list[str]]) -> None:
    """Đo trigram OOV/threshold và coverage 7 ca, dùng hash để tiết kiệm RAM."""
    print("\n=== TRIGRAM ===")
    counts: dict[int, int] = {}
    targets: Counter[tuple[str, str, str]] = Counter()
    target_set = {t for _, good, bad in TRIGRAM_CASES for t in (good, bad)}

    for sent in train_all:
        for triple in zip(sent, sent[1:], sent[2:], strict=False):
            h = hash(triple)
            counts[h] = counts.get(h, 0) + 1
            if triple in target_set:
                targets[triple] += 1

    print(f"trigram_distinct(>=1)={len(counts):,}")
    for thr in (2, 3, 5):
        n = sum(1 for c in counts.values() if c >= thr)
        print(f"trigram count>={thr}: {n:,} | binary ~{n*12/1024/1024:.1f} MB")

    for thr in (1, 3):
        total = missing = 0
        for sent in dev_hj:
            for triple in zip(sent, sent[1:], sent[2:], strict=False):
                total += 1
                if counts.get(hash(triple), 0) < thr:
                    missing += 1
        print(
            f"dev_hj trigram OOV (count>={thr}) = {missing}/{total} "
            f"({missing/total*100:.1f}%)"
        )

    print("\nTrigram 7 ca (đúng / sai):")
    viable = 0
    for cid, good, bad in TRIGRAM_CASES:
        gc, bc = targets[good], targets[bad]
        # coi là khả thi nếu trigram đúng có count >=3 và hơn trigram sai
        if gc >= 3 and gc > bc:
            viable += 1
        print(f"  {cid}: đúng={good} count={gc} | sai={bad} count={bc}")
    print(f"trigram khả thi (đúng>=3 và > sai): {viable}/7")


if __name__ == "__main__":
    main()
