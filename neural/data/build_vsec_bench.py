#!/usr/bin/env python3
"""Chuyển VSEC (data_raw/vsec.jsonl) thành JSONL benchmark cho eval_cascade.

VSEC gán nhãn theo âm tiết; script tái tạo text/clean_text + byte spans.
"""

from __future__ import annotations

import argparse
import json
import re
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
DEFAULT_INPUT = ROOT / "data_raw" / "vsec.jsonl"
DEFAULT_OUTPUT = ROOT / "data_processed" / "vsec_bench.jsonl"
CORE_RE = re.compile(r"^(\W*)(.*?)(\W*)$", re.UNICODE)


def split_punctuation(syllable: str) -> tuple[str, str, str]:
    match = CORE_RE.match(syllable)
    if not match:
        return "", syllable, ""
    return match.group(1), match.group(2), match.group(3)


def label_for(wrong: str, correct: str) -> int:
    # Nếu khác dấu thanh nhưng cùng base letters → TONE (2), còn lại PHONETIC (1).
    wrong_base = "".join(
        character
        for character in unicodedata.normalize("NFD", wrong.lower())
        if unicodedata.category(character) != "Mn"
    )
    correct_base = "".join(
        character
        for character in unicodedata.normalize("NFD", correct.lower())
        if unicodedata.category(character) != "Mn"
    )
    return 2 if wrong_base == correct_base else 1


def convert(input_path: Path, output_path: Path) -> dict[str, int]:
    rows = 0
    errors = 0
    with input_path.open(encoding="utf-8") as source, output_path.open(
        "w", encoding="utf-8"
    ) as target:
        for line in source:
            record = json.loads(line)
            annotations = record.get("annotations", [])
            if not annotations:
                continue
            text_parts: list[str] = []
            clean_parts: list[str] = []
            spans: list[dict[str, int]] = []
            byte_cursor = 0
            for index, annotation in enumerate(annotations):
                syllable = unicodedata.normalize(
                    "NFC", annotation.get("current_syllable", "")
                ).strip()
                if not syllable:
                    continue
                prefix, core, suffix = split_punctuation(syllable)
                clean_core = core
                if not annotation.get("is_correct", True):
                    alternatives = annotation.get("alternative_syllables") or []
                    if alternatives:
                        _, clean_core, _ = split_punctuation(
                            unicodedata.normalize("NFC", alternatives[0]).strip()
                        )
                if index > 0 and text_parts:
                    text_parts.append(" ")
                    clean_parts.append(" ")
                    byte_cursor += 1
                start = byte_cursor + len(prefix.encode("utf-8"))
                text_parts.append(prefix)
                clean_parts.append(prefix)
                text_parts.append(core)
                clean_parts.append(clean_core)
                end = start + len(core.encode("utf-8"))
                if clean_core != core:
                    spans.append(
                        {
                            "start": start,
                            "end": end,
                            "label": label_for(core, clean_core),
                            "raw": core,
                            "clean": clean_core,
                        }
                    )
                    errors += 1
                text_parts.append(suffix)
                clean_parts.append(suffix)
                byte_cursor = len("".join(text_parts).encode("utf-8"))

            if not spans:
                continue
            text = "".join(text_parts)
            clean_text = "".join(clean_parts)
            target.write(
                json.dumps(
                    {
                        "id": f"vsec-{rows}",
                        "text": text,
                        "clean_text": clean_text,
                        "spans": spans,
                        "meta": {"source": "vsec"},
                    },
                    ensure_ascii=False,
                )
                + "\n"
            )
            rows += 1
    return {"rows": rows, "errors": errors}


def main() -> None:
    parser = argparse.ArgumentParser(description="Build VSEC benchmark JSONL")
    parser.add_argument("--input", type=Path, default=DEFAULT_INPUT)
    parser.add_argument("--out", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()
    stats = convert(args.input, args.out)
    print(json.dumps(stats, ensure_ascii=False))


if __name__ == "__main__":
    main()
