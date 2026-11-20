#!/usr/bin/env python3
"""Chuyển Viwiki-Spelling (JSON/JSONL) sang JSONL benchmark cho eval_cascade."""

from __future__ import annotations

import argparse
import difflib
import json
import re
import unicodedata
from collections import Counter
from pathlib import Path

CLEAN, PHONETIC, TONE, WORD = 0, 1, 2, 3
TONE_SET = {"\u0300", "\u0301", "\u0303", "\u0309", "\u0323"}
DIACRITIC_SET = {"\u0302", "\u0306", "\u031B"}


def strip_marks(value: str, tone_only: bool) -> str:
    nfd = unicodedata.normalize("NFD", value.lower())
    marks = TONE_SET if tone_only else (TONE_SET | DIACRITIC_SET)
    result = "".join(ch for ch in nfd if ch not in marks)
    return unicodedata.normalize("NFC", result).replace("đ", "d")


def infer_kind(raw: str, clean: str) -> int:
    raw_lc = unicodedata.normalize("NFC", raw.strip().lower())
    clean_lc = unicodedata.normalize("NFC", clean.strip().lower())
    if not raw_lc or not clean_lc or " " in raw_lc or " " in clean_lc:
        return WORD
    if strip_marks(raw_lc, True) == strip_marks(clean_lc, True):
        return TONE
    if strip_marks(raw_lc, False) == strip_marks(clean_lc, False):
        return PHONETIC
    return WORD


def char_to_byte(text: str, index: int) -> int:
    return len(text[: max(0, min(index, len(text)))].encode("utf-8"))


def split_sentences(text: str, spans: list[tuple[int, int, str]]) -> list[dict]:
    records = []
    for match in re.finditer(r"[^.!?\n]+(?:[.!?]+|\n|$)", text):
        sentence = match.group(0).strip()
        if len(sentence) < 15:
            continue
        start, end = match.start(), match.end()
        local = []
        for span_start, span_end, suggestion in spans:
            if span_start >= start and span_end <= end and suggestion:
                local.append((span_start - start, span_end - start, suggestion.strip()))
        if not local:
            continue
        local.sort(key=lambda item: (item[0], item[1]))
        merged = []
        for item in local:
            if not merged or item[0] >= merged[-1][1]:
                merged.append(item)
        clean_parts = []
        cursor = 0
        for span_start, span_end, suggestion in merged:
            clean_parts.append(sentence[cursor:span_start])
            clean_parts.append(suggestion)
            cursor = span_end
        clean_parts.append(sentence[cursor:])
        clean_sentence = unicodedata.normalize("NFC", "".join(clean_parts))
        corrupted_sentence = unicodedata.normalize("NFC", sentence)
        corrupted_bytes = corrupted_sentence.encode("utf-8")
        clean_bytes = clean_sentence.encode("utf-8")
        if not corrupted_bytes or len(corrupted_bytes) > 512:
            continue
        labels = [CLEAN] * len(corrupted_bytes)
        byte_targets = (
            list(clean_bytes)
            if len(corrupted_bytes) == len(clean_bytes)
            else list(corrupted_bytes)
        )
        byte_mask = [1] * len(corrupted_bytes)
        spans_out = []
        for span_start, span_end, suggestion in merged:
            byte_start = char_to_byte(corrupted_sentence, span_start)
            byte_end = char_to_byte(corrupted_sentence, span_end)
            if byte_start >= byte_end or byte_end > len(corrupted_bytes):
                continue
            raw_token = corrupted_bytes[byte_start:byte_end].decode("utf-8", errors="replace")
            clean_token = unicodedata.normalize("NFC", suggestion)
            label = infer_kind(raw_token, clean_token)
            spans_out.append(
                {
                    "start": byte_start,
                    "end": byte_end,
                    "label": label,
                    "raw": raw_token,
                    "clean": clean_token,
                }
            )
            for index in range(byte_start, byte_end):
                labels[index] = label
            clean_token_bytes = clean_token.encode("utf-8")
            if byte_end - byte_start == len(clean_token_bytes):
                for offset, value in enumerate(clean_token_bytes):
                    byte_targets[byte_start + offset] = value
                    byte_mask[byte_start + offset] = 1
            else:
                for index in range(byte_start, byte_end):
                    byte_mask[index] = 0
        if spans_out:
            records.append(
                {
                    "text": corrupted_sentence,
                    "clean_text": clean_sentence,
                    "source": "viwiki_spelling",
                    "spans": spans_out,
                    "labels": labels,
                    "byte_targets": byte_targets,
                    "byte_mask": byte_mask,
                }
            )
    return records


def pair_spans(noisy: str, clean: str) -> list[tuple[int, int, str]]:
    matcher = difflib.SequenceMatcher(None, noisy, clean, autojunk=False)
    spans = []
    for tag, i1, i2, j1, j2 in matcher.get_opcodes():
        if tag in ("replace", "delete", "insert"):
            raw = noisy[i1:i2]
            target = clean[j1:j2]
            if raw.strip() and len(raw) <= 24 and len(target) <= 24:
                spans.append((i1, i2, target))
    return spans


def parse_file(path: Path) -> list[dict]:
    content = path.read_text(encoding="utf-8", errors="ignore").strip()
    if not content:
        return []
    docs = []
    if content.startswith("["):
        loaded = json.loads(content)
        if isinstance(loaded, list):
            docs = [item for item in loaded if isinstance(item, dict)]
    else:
        for line in content.splitlines():
            try:
                obj = json.loads(line)
                if isinstance(obj, dict):
                    docs.append(obj)
            except json.JSONDecodeError:
                continue
    records = []
    for doc in docs:
        text = doc.get("text") or doc.get("noisy_text") or doc.get("content") or ""
        mistakes = doc.get("mistakes") or doc.get("errors") or doc.get("spans")
        if isinstance(mistakes, list) and text:
            spans = []
            for mistake in mistakes:
                start = mistake.get("start_offset", mistake.get("start"))
                end = mistake.get("end_offset", mistake.get("end"))
                if isinstance(start, str) and start.isdigit():
                    start = int(start)
                if isinstance(end, str) and end.isdigit():
                    end = int(end)
                wrong = mistake.get("text")
                if end is None and isinstance(start, int) and isinstance(wrong, str):
                    end = start + len(wrong)
                suggestion = (
                    mistake.get("suggest")
                    or mistake.get("correction")
                    or mistake.get("clean")
                    or mistake.get("target")
                )
                if isinstance(suggestion, list) and suggestion:
                    suggestion = suggestion[0]
                if isinstance(start, int) and isinstance(end, int) and isinstance(suggestion, str):
                    spans.append((start, end, suggestion))
            if spans:
                records.extend(split_sentences(unicodedata.normalize("NFC", text), spans))
                continue
        clean = doc.get("clean_text") or doc.get("target") or doc.get("corrected_text")
        if text and isinstance(clean, str) and text != clean:
            noisy_nfc = unicodedata.normalize("NFC", text)
            clean_nfc = unicodedata.normalize("NFC", clean)
            spans = pair_spans(noisy_nfc, clean_nfc)
            if spans:
                records.extend(split_sentences(noisy_nfc, spans))
    return records


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--inputs", nargs="+", type=Path, required=True)
    parser.add_argument("--out", type=Path, default=Path("data_processed/viwiki_spelling_bench.jsonl"))
    args = parser.parse_args()

    files = []
    for path in args.inputs:
        if path.is_dir():
            files.extend(sorted(path.rglob("*.json*")))
        elif path.exists():
            files.append(path)
    records = []
    for path in files:
        records.extend(parse_file(path))
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("w", encoding="utf-8") as handle:
        for record in records:
            handle.write(json.dumps(record, ensure_ascii=False) + "\n")

    labels = Counter()
    total = same = 0
    for record in records:
        for span in record["spans"]:
            total += 1
            labels[span["label"]] += 1
            if len(span["raw"].encode("utf-8")) == len(span["clean"].encode("utf-8")):
                same += 1
    print(
        json.dumps(
            {
                "files": len(files),
                "sentences": len(records),
                "spans": total,
                "same_byte_pct": round(same / max(1, total) * 100, 1),
                "labels": dict(labels),
            },
            ensure_ascii=False,
        )
    )


if __name__ == "__main__":
    main()
