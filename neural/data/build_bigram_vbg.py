#!/usr/bin/env python3
"""Trích xuất bảng Syllable Bigram nén (.vbg) từ corpus tiếng Việt sạch.

Layout VBG1 (little-endian):
  [0..4] magic "VBG1"; [4..6] version; [6..8] flags;
  [8..12] count; [12..16] reserved;
  [16..16+8N] hashes u64 sorted; [16+8N..16+9N] scores u8.
"""

from __future__ import annotations

import argparse
import bisect
import math
import re
import struct
import unicodedata
from collections import Counter
from pathlib import Path

VBG_MAGIC = b"VBG1"
VBG_VERSION = 1
FNV_OFFSET_BASIS = 0xCBF29CE484222325
FNV_PRIME = 0x100000001B3
U64_MASK = 0xFFFFFFFFFFFFFFFF

CLAUSE_SPLIT_RE = re.compile(r"[^\w\s]+|\d+", re.UNICODE)
SYLLABLE_RE = re.compile(
    r"^[a-zàáảãạăằắẳẵặâầấẩẫậđèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵ]+$"
)


def fnv1a_64_bigram(w1: str, w2: str) -> int:
    h = FNV_OFFSET_BASIS
    for b in w1.encode("utf-8"):
        h ^= b
        h = (h * FNV_PRIME) & U64_MASK
    h ^= 0
    h = (h * FNV_PRIME) & U64_MASK
    for b in w2.encode("utf-8"):
        h ^= b
        h = (h * FNV_PRIME) & U64_MASK
    return h


def load_viet74k(path: Path | None) -> set[str] | None:
    if path is None or not path.exists():
        return None
    syllables: set[str] = set()
    with path.open("r", encoding="utf-8") as handle:
        for line in handle:
            normalized = unicodedata.normalize("NFC", line.strip().lower())
            for token in normalized.split():
                if SYLLABLE_RE.match(token):
                    syllables.add(token)
    return syllables


def extract_bigrams(
    input_paths: list[Path],
    valid_syllables: set[str] | None = None,
) -> Counter[tuple[str, str]]:
    counts: Counter[tuple[str, str]] = Counter()
    total_lines = 0
    for path in input_paths:
        with path.open("r", encoding="utf-8", errors="ignore") as handle:
            for line in handle:
                total_lines += 1
                normalized = unicodedata.normalize("NFC", line.strip().lower())
                if not normalized:
                    continue
                for clause in CLAUSE_SPLIT_RE.split(normalized):
                    tokens = clause.split()
                    if len(tokens) < 2:
                        continue
                    previous: str | None = None
                    for token in tokens:
                        valid = bool(SYLLABLE_RE.match(token)) and (
                            valid_syllables is None or token in valid_syllables
                        )
                        if valid:
                            if previous is not None:
                                counts[(previous, token)] += 1
                            previous = token
                        else:
                            previous = None
    print(f"[build_bigram_vbg] {total_lines:,} dòng -> {len(counts):,} bigram.")
    return counts


def quantize_log_freq(count: int, minimum: int, maximum: int) -> int:
    if maximum <= minimum:
        return 255
    ratio = (math.log(count) - math.log(minimum)) / (math.log(maximum) - math.log(minimum))
    value = round(255.0 * ratio)
    return max(1, min(255, value))


def write_vbg(
    out_path: Path,
    counts: Counter[tuple[str, str]],
    min_count: int,
    max_entries: int,
) -> tuple[int, int, int]:
    filtered = [(pair, count) for pair, count in counts.items() if count >= min_count]
    filtered.sort(key=lambda item: item[1], reverse=True)
    if max_entries > 0:
        filtered = filtered[:max_entries]
    if not filtered:
        raise ValueError("Không có bigram nào thỏa min_count.")
    maximum = filtered[0][1]
    minimum = filtered[-1][1]

    hash_map: dict[int, int] = {}
    collisions = 0
    for (w1, w2), count in filtered:
        hashed = fnv1a_64_bigram(w1, w2)
        if hashed in hash_map:
            collisions += 1
            hash_map[hashed] = max(hash_map[hashed], count)
        else:
            hash_map[hashed] = count
    if collisions:
        print(f"[!] {collisions} va chạm hash (giữ count lớn hơn).")

    items = sorted(hash_map.items(), key=lambda item: item[0])
    count = len(items)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with out_path.open("wb") as handle:
        handle.write(struct.pack("<4sHHII", VBG_MAGIC, VBG_VERSION, 0, count, 0))
        handle.write(struct.pack(f"<{count}Q", *(hash_value for hash_value, _ in items)))
        handle.write(bytes(quantize_log_freq(value, minimum, maximum) for _, value in items))
    return count, minimum, maximum


def verify_vbg(vbg_path: Path, probes: list[tuple[str, str]]) -> None:
    data = vbg_path.read_bytes()
    magic, version, flags, count, _ = struct.unpack_from("<4sHHII", data, 0)
    assert magic == VBG_MAGIC and version == VBG_VERSION
    hashes_end = 16 + count * 8
    scores_end = hashes_end + count
    assert len(data) == scores_end, "Sai kích thước file"
    hashes = struct.unpack_from(f"<{count}Q", data, 16)
    scores = data[hashes_end:scores_end]
    print(f"[Verify] entries={count:,} size={len(data)/1024:.1f} KB flags={flags}")
    for w1, w2 in probes:
        hashed = fnv1a_64_bigram(w1, w2)
        index = bisect.bisect_left(hashes, hashed)
        score = scores[index] if index < count and hashes[index] == hashed else 0
        print(f'  ("{w1}", "{w2}") -> hash=0x{hashed:016x}, score={score}')


def main() -> None:
    parser = argparse.ArgumentParser(description="Build .vbg Syllable Bigram table.")
    parser.add_argument("--inputs", nargs="+", type=Path, required=True)
    parser.add_argument("--viet74k", type=Path, default=None)
    parser.add_argument("--min-count", type=int, default=3)
    parser.add_argument("--max-entries", type=int, default=30000)
    parser.add_argument("--out", type=Path, default=Path("neural/fixtures/vi_bigram.vbg"))
    args = parser.parse_args()

    valid = load_viet74k(args.viet74k)
    counts = extract_bigrams(args.inputs, valid)
    count, minimum, maximum = write_vbg(args.out, counts, args.min_count, args.max_entries)
    print(f"[Done] {count:,} bigram (min={minimum}, max={maximum}) -> {args.out}")
    verify_vbg(
        args.out,
        probes=[
            ("giành", "giật"),
            ("dành", "giật"),
            ("sáp", "nhập"),
            ("sát", "nhập"),
            ("bàng", "quan"),
            ("bàng", "quang"),
            ("chính", "tả"),
        ],
    )


if __name__ == "__main__":
    main()
