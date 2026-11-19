#!/usr/bin/env python3
"""Phân tích phân bố lỗi thực tế trên VSEC và (tùy chọn) log miss của Cascade."""

from __future__ import annotations

import argparse
import json
import unicodedata
from collections import Counter, defaultdict
from pathlib import Path

PHONETIC_PAIRS = [
    ("s", "x"),
    ("l", "n"),
    ("ch", "tr"),
    ("d", "gi"),
    ("d", "r"),
    ("gi", "r"),
    ("c", "k"),
    ("c", "q"),
    ("i", "y"),
    ("n", "ng"),
    ("t", "c"),
    ("nh", "n"),
]


def strip_accents(value: str) -> str:
    nfd = unicodedata.normalize("NFD", value)
    out = "".join(ch for ch in nfd if unicodedata.category(ch) != "Mn")
    return out.replace("đ", "d").replace("Đ", "D")


def strip_tone_only(value: str) -> str:
    tone_marks = {"\u0300", "\u0301", "\u0303", "\u0309", "\u0323"}
    nfd = unicodedata.normalize("NFD", value)
    return unicodedata.normalize("NFC", "".join(ch for ch in nfd if ch not in tone_marks))


def levenshtein(left: str, right: str) -> int:
    if left == right:
        return 0
    if not left:
        return len(right)
    if not right:
        return len(left)
    previous = list(range(len(right) + 1))
    for row, left_char in enumerate(left, 1):
        current = [row]
        for column, right_char in enumerate(right, 1):
            cost = 0 if left_char == right_char else 1
            current.append(
                min(current[-1] + 1, previous[column] + 1, previous[column - 1] + cost)
            )
        previous = current
    return previous[-1]


def load_viet74k(path: Path | None) -> set[str]:
    if path is None or not path.exists():
        return set()
    vocab: set[str] = set()
    with path.open(encoding="utf-8") as handle:
        for line in handle:
            normalized = unicodedata.normalize("NFC", line.strip().lower())
            if normalized:
                vocab.add(normalized)
                vocab.update(normalized.split())
    return vocab


def classify_error(raw: str, clean: str) -> str:
    raw_lc = unicodedata.normalize("NFC", raw).lower()
    clean_lc = unicodedata.normalize("NFC", clean).lower()
    if raw_lc == clean_lc:
        return "CASE_ONLY"
    if " " in raw_lc or " " in clean_lc:
        return "SPACE_MERGE_SPLIT"
    if strip_tone_only(raw_lc) == strip_tone_only(clean_lc):
        return "TONE_ONLY"
    if strip_accents(raw_lc) == strip_accents(clean_lc):
        return "DIACRITIC_VOWEL"
    for left, right in PHONETIC_PAIRS:
        if (raw_lc.startswith(left) and clean_lc.startswith(right) and raw_lc[len(left):] == clean_lc[len(right):]) or (
            raw_lc.startswith(right) and clean_lc.startswith(left) and raw_lc[len(right):] == clean_lc[len(left):]
        ):
            return "PHONETIC_ONSET"
        if (raw_lc.endswith(left) and clean_lc.endswith(right) and raw_lc[:-len(left)] == clean_lc[:-len(right)]) or (
            raw_lc.endswith(right) and clean_lc.endswith(left) and raw_lc[:-len(right)] == clean_lc[:-len(left)]
        ):
            return "PHONETIC_CODA"
    distance = levenshtein(raw_lc, clean_lc)
    if distance == 1:
        if len(raw_lc) < len(clean_lc):
            return "CHAR_MISSING_1"
        if len(raw_lc) > len(clean_lc):
            return "CHAR_EXTRA_1"
        return "CHAR_SUB_1"
    return "MULTI_EDIT_2PLUS"


def extract_spans(row: dict) -> list[tuple[int, int, str, str]]:
    text = row.get("text", "")
    text_bytes = text.encode("utf-8")
    out: list[tuple[int, int, str, str]] = []
    for span in row.get("spans", []):
        start = int(span.get("start", 0))
        end = int(span.get("end", 0))
        raw = span.get("raw") or text_bytes[start:end].decode("utf-8", errors="replace")
        clean = span.get("clean", "")
        if raw and clean and raw != clean:
            out.append((start, end, raw, clean))
    return out


def audit(vsec_path: Path, viet74k: set[str], cascade_log: Path | None) -> None:
    total_rows = 0
    total_spans = 0
    byte_len_stats: Counter[str] = Counter()
    vocab_stats: Counter[str] = Counter()
    kind_stats: Counter[str] = Counter()
    pairs: defaultdict[str, Counter[tuple[str, str]]] = defaultdict(Counter)

    cascade_by_key: dict[tuple[int, int, int], dict] = {}
    if cascade_log and cascade_log.exists():
        with cascade_log.open(encoding="utf-8") as handle:
            for line in handle:
                if line.strip():
                    record = json.loads(line)
                    cascade_by_key[
                        (record["record_idx"], record["start"], record["end"])
                    ] = record

    miss_reasons: Counter[str] = Counter()
    miss_by_kind: defaultdict[str, Counter[str]] = defaultdict(Counter)

    with vsec_path.open(encoding="utf-8") as handle:
        for index, line in enumerate(handle):
            if not line.strip():
                continue
            row = json.loads(line)
            total_rows += 1
            for start, end, raw, clean in extract_spans(row):
                total_spans += 1
                raw_bytes = len(raw.encode("utf-8"))
                clean_bytes = len(clean.encode("utf-8"))
                if raw_bytes == clean_bytes:
                    byte_len_stats["SAME_BYTE_LEN"] += 1
                elif raw_bytes < clean_bytes:
                    byte_len_stats["RAW_SHORTER_BYTES"] += 1
                else:
                    byte_len_stats["RAW_LONGER_BYTES"] += 1

                if viet74k:
                    raw_in = unicodedata.normalize("NFC", raw.lower()) in viet74k
                    clean_in = unicodedata.normalize("NFC", clean.lower()) in viet74k
                    if not raw_in and clean_in:
                        vocab_stats["RAW_OOV -> CLEAN_IN_VOCAB"] += 1
                    elif raw_in and clean_in:
                        vocab_stats["RAW_IN_VOCAB -> CLEAN_IN_VOCAB"] += 1
                    elif not clean_in:
                        vocab_stats["CLEAN_OOV"] += 1

                kind = classify_error(raw, clean)
                kind_stats[kind] += 1
                pairs[kind][(raw, clean)] += 1

                if cascade_by_key:
                    diagnostic = cascade_by_key.get((index, start, end))
                    status = diagnostic.get("status", "NO_LOG_ENTRY") if diagnostic else "NO_LOG_ENTRY"
                    miss_reasons[status] += 1
                    if status != "HIT":
                        miss_by_kind[kind][status] += 1

    def print_table(title: str, counter: Counter[str], denominator: int) -> None:
        print(f"\n=== {title} (n={denominator:,}) ===")
        for key, value in counter.most_common():
            percent = value / denominator * 100.0 if denominator else 0.0
            print(f"  {key:<55} {value:>7,} ({percent:5.1f}%)")

    print(f"[VSEC] rows={total_rows:,} spans={total_spans:,}")
    print_table("1. BYTE LENGTH", byte_len_stats, total_spans)
    if viet74k:
        print_table("2. VIET74K VOCAB", vocab_stats, total_spans)
    print_table("3. ERROR KIND", kind_stats, total_spans)

    print("\n=== 4. TOP EXAMPLES ===")
    for kind, _ in kind_stats.most_common(8):
        top = ", ".join(
            f"'{raw}'->'{clean}'({count})"
            for (raw, clean), count in pairs[kind].most_common(4)
        )
        print(f"  {kind:<22}: {top}")

    if cascade_by_key:
        print_table("5. CASCADE STATUS", miss_reasons, total_spans)
        print("\n=== 6. MISS BY KIND ===")
        for kind, sub in sorted(
            miss_by_kind.items(), key=lambda item: sum(item[1].values()), reverse=True
        )[:8]:
            summary = ", ".join(f"{status}={count}" for status, count in sub.most_common(3))
            print(f"  {kind:<22} (miss={sum(sub.values()):>5,}): {summary}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--vsec", type=Path, required=True)
    parser.add_argument("--viet74k", type=Path, default=Path("data_raw/Viet74K.txt"))
    parser.add_argument("--cascade-log", type=Path, default=None)
    args = parser.parse_args()
    audit(args.vsec, load_viet74k(args.viet74k), args.cascade_log)


if __name__ == "__main__":
    main()
