#!/usr/bin/env python3
"""D1 v3: Mixed synthetic + VSEC generator aligned with VSEC error distribution.

Chạy:
    python -m neural.data.generate_mixed \
      --corpus data_raw/viwiki_clean.txt data_raw/leipzig_news.txt \
      --vsec data_processed/vsec_bench.jsonl \
      --num-train 600000 --vsec-mix-ratio 0.15 \
      --out data_processed/neural_mixed_v3
"""

from __future__ import annotations

import argparse
import json
import random
import re
import unicodedata
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

CLEAN = 0
PHONETIC = 1
TONE = 2
WORD = 3

SYLLABLE_RE = re.compile(r"^[A-Za-zÀ-ỹĐđ]+$", re.UNICODE)

TONE_SET = {"\u0300", "\u0301", "\u0303", "\u0309", "\u0323"}
DIACRITIC_SET = {"\u0302", "\u0306", "\u031B"}

VOWEL_DIACRITIC_VARIANTS: dict[str, list[str]] = {
    "a": ["\u0306", "\u0302"],
    "e": ["\u0302"],
    "o": ["\u0302", "\u031B"],
    "u": ["\u031B"],
}

TONE_TRANSITIONS: dict[str, list[tuple[str, float]]] = {
    "": [("\u0300", 0.30), ("\u0301", 0.30), ("\u0323", 0.20), ("\u0309", 0.10), ("\u0303", 0.10)],
    "\u0300": [("", 0.45), ("\u0323", 0.25), ("\u0309", 0.15), ("\u0301", 0.15)],
    "\u0301": [("", 0.40), ("\u0309", 0.25), ("\u0303", 0.20), ("\u0323", 0.15)],
    "\u0309": [("\u0303", 0.50), ("\u0301", 0.20), ("", 0.15), ("\u0323", 0.15)],
    "\u0303": [("\u0309", 0.50), ("\u0323", 0.25), ("\u0301", 0.15), ("", 0.10)],
    "\u0323": [("\u0300", 0.35), ("", 0.30), ("\u0303", 0.20), ("\u0309", 0.15)],
}

ONSET_SWAPS = [
    ("s", "x"), ("x", "s"), ("l", "n"), ("n", "l"), ("ch", "tr"), ("tr", "ch"),
    ("d", "gi"), ("gi", "d"), ("d", "r"), ("r", "d"), ("gi", "r"), ("r", "gi"),
    ("c", "k"), ("k", "c"), ("ngh", "ng"), ("ng", "ngh"), ("gh", "g"), ("g", "gh"),
]
CODA_SWAPS = [
    ("n", "ng"), ("ng", "n"), ("t", "c"), ("c", "t"), ("nh", "n"), ("n", "nh"),
    ("ch", "c"), ("c", "ch"), ("i", "y"), ("y", "i"),
]
VSEC_CODA_VOWEL_DROPS = [
    ("nghiệm", "nghiêm"), ("nghiêm", "nghiệm"), ("nghiêng", "nghiên"),
    ("nghiên", "nghiêng"), ("được", "đươc"), ("nước", "nươc"), ("trước", "trươc"),
    ("người", "ngươi"), ("mười", "mươi"), ("cười", "cươi"), ("tiếng", "tiến"),
    ("việc", "viêc"), ("biết", "biêt"), ("học", "họ"), ("thực", "thưc"),
    ("chức", "chưc"), ("phát", "phá"), ("nhất", "nhâ"), ("mất", "mâ"),
]
WORD_CONFUSIONS = [
    ("dành", "giành"), ("giành", "dành"), ("sáp", "sát"), ("sát", "sáp"),
    ("quan", "quang"), ("quang", "quan"), ("trí", "tri"), ("tri", "trí"),
    ("suất", "xuất"), ("xuất", "suất"), ("sử", "xử"), ("xử", "sử"),
    ("chí", "trí"), ("trí", "chí"), ("bổ", "bỗ"), ("bỗ", "bổ"),
    ("dự", "dữ"), ("dữ", "dự"), ("tả", "tã"), ("tã", "tả"),
    ("chính", "chín"), ("chín", "chính"), ("tính", "tín"), ("tín", "tính"),
]


@dataclass
class TokenSpan:
    text: str
    is_word: bool


def tokenize_sentence(sentence: str) -> list[TokenSpan]:
    tokens = []
    for match in re.finditer(r"[^\W\d_]+|[^\w\s]+|\s+", sentence, re.UNICODE):
        token = match.group(0)
        tokens.append(TokenSpan(token, bool(SYLLABLE_RE.match(token)) and len(token) >= 2))
    return tokens


def preserve_case(source: str, target: str) -> str:
    if source.isupper() and len(source) > 1:
        return target.upper()
    if source[0].isupper():
        return target[0].upper() + target[1:]
    return target


def mutate_tone(word: str, rng: random.Random) -> str | None:
    nfd = unicodedata.normalize("NFD", word)
    current_tone = ""
    chars: list[str] = []
    tone_index = -1
    for character in nfd:
        if character in TONE_SET:
            current_tone = character
            tone_index = len(chars)
        else:
            chars.append(character)
    transitions = TONE_TRANSITIONS.get(current_tone, [])
    if not transitions:
        return None
    tones, weights = zip(*transitions, strict=True)
    new_tone = rng.choices(tones, weights=weights, k=1)[0]
    if new_tone:
        if tone_index >= 0:
            chars.insert(tone_index, new_tone)
        else:
            insert_at = -1
            for index, character in enumerate(chars):
                if character.lower() in "aeiouy":
                    insert_at = index + 1
                    while insert_at < len(chars) and chars[insert_at] in DIACRITIC_SET:
                        insert_at += 1
            if insert_at < 0:
                return None
            chars.insert(insert_at, new_tone)
    mutated = unicodedata.normalize("NFC", "".join(chars))
    return mutated if mutated != word else None


def mutate_diacritic_vowel(word: str, rng: random.Random) -> str | None:
    if rng.random() < 0.25:
        if word.startswith("đ"):
            return "d" + word[1:]
        if word.startswith("Đ"):
            return "D" + word[1:]
        if word.startswith("d"):
            return "đ" + word[1:]
        if word.startswith("D"):
            return "Đ" + word[1:]

    nfd = unicodedata.normalize("NFD", word)
    clusters: list[tuple[str, str, str]] = []
    for character in nfd:
        if character in DIACRITIC_SET and clusters:
            base, _, tone = clusters[-1]
            clusters[-1] = (base, character, tone)
        elif character in TONE_SET and clusters:
            base, diacritic, _ = clusters[-1]
            clusters[-1] = (base, diacritic, character)
        else:
            clusters.append((character, "", ""))

    eligible = [
        index for index, (base, _, _) in enumerate(clusters)
        if base.lower() in VOWEL_DIACRITIC_VARIANTS
    ]
    if not eligible:
        return None
    target = rng.choice(eligible)
    base, current, tone = clusters[target]
    variants = [""] + VOWEL_DIACRITIC_VARIANTS[base.lower()]
    choices = [variant for variant in variants if variant != current]
    if not choices:
        return None
    clusters[target] = (base, rng.choice(choices), tone)
    rebuilt = "".join(base + diacritic + tone for base, diacritic, tone in clusters)
    mutated = unicodedata.normalize("NFC", rebuilt)
    return mutated if mutated != word else None


def mutate_char_drop_or_dup(word: str, rng: random.Random) -> str | None:
    characters = list(word)
    if len(characters) < 3:
        return None
    if rng.random() < 0.62 and len(characters) >= 4:
        index = len(characters) - 1 if rng.random() < 0.55 else rng.randint(1, len(characters) - 2)
        del characters[index]
        result = "".join(characters)
        return result if result != word else None
    if rng.random() < 0.70:
        index = rng.randint(1, len(characters) - 1)
        characters.insert(index, characters[index])
        return "".join(characters)
    return word + rng.choice(["s", "f", "r", "x", "j", "w", "n", "g"])


def mutate_phonetic_or_word(word: str, rng: random.Random) -> tuple[str | None, int]:
    lower = word.lower()
    roll = rng.random()
    if roll < 0.35:
        for wrong, correct in WORD_CONFUSIONS:
            if lower == wrong:
                return preserve_case(word, correct), WORD
    if roll < 0.60:
        for wrong, correct in VSEC_CODA_VOWEL_DROPS:
            if lower == wrong:
                return preserve_case(word, correct), PHONETIC
    shuffled = ONSET_SWAPS[:]
    rng.shuffle(shuffled)
    for left, right in shuffled:
        if lower.startswith(left) and len(lower) > len(left):
            return preserve_case(word, right + lower[len(left):]), PHONETIC
    shuffled = CODA_SWAPS[:]
    rng.shuffle(shuffled)
    for left, right in shuffled:
        if lower.endswith(left) and len(lower) > len(left):
            return preserve_case(word, lower[:-len(left)] + right), PHONETIC
    return None, PHONETIC


def build_alignment_record(clean_text: str, corrupted_text: str, spans: list[dict], source: str) -> dict | None:
    clean_nfc = unicodedata.normalize("NFC", clean_text)
    corrupted_nfc = unicodedata.normalize("NFC", corrupted_text)
    corrupted_bytes = corrupted_nfc.encode("utf-8")
    clean_bytes = clean_nfc.encode("utf-8")
    length = len(corrupted_bytes)
    if length == 0 or length > 512:
        return None
    labels = [CLEAN] * length
    byte_targets = list(corrupted_bytes)
    byte_mask = [1] * length
    if len(corrupted_bytes) == len(clean_bytes):
        byte_targets = list(clean_bytes)
    for span in spans:
        start, end, label = span["start"], span["end"], span["label"]
        raw_bytes = span["raw"].encode("utf-8")
        clean_span_bytes = span["clean"].encode("utf-8")
        for index in range(start, min(end, length)):
            labels[index] = label
        if len(raw_bytes) == len(clean_span_bytes):
            for offset, value in enumerate(clean_span_bytes):
                if start + offset < length:
                    byte_targets[start + offset] = value
                    byte_mask[start + offset] = 1
        else:
            for index in range(start, min(end, length)):
                byte_mask[index] = 0
    return {
        "text": corrupted_nfc,
        "clean_text": clean_nfc,
        "source": source,
        "spans": spans,
        "labels": labels,
        "byte_targets": byte_targets,
        "byte_mask": byte_mask,
    }


def synthesize_one(clean_sentence: str, rng: random.Random) -> dict | None:
    clean_sentence = unicodedata.normalize("NFC", clean_sentence.strip())
    tokens = tokenize_sentence(clean_sentence)
    word_indices = [index for index, token in enumerate(tokens) if token.is_word]
    if len(word_indices) < 3:
        return None
    if rng.random() < 0.18:
        return build_alignment_record(clean_sentence, clean_sentence, [], "synthetic_clean")

    category = rng.choices(
        ["tone", "diacritic_vowel", "char_drop_dup", "phonetic_word", "space_edit"],
        weights=[0.42, 0.18, 0.20, 0.15, 0.05],
        k=1,
    )[0]

    if category == "space_edit":
        if rng.random() < 0.65 and len(word_indices) >= 2:
            position = rng.choice(range(len(word_indices) - 1))
            first, second = word_indices[position], word_indices[position + 1]
            if second == first + 2 and tokens[first + 1].text == " ":
                merged = tokens[first].text + tokens[second].text
                original = tokens[first].text + " " + tokens[second].text
                prefix = "".join(token.text for token in tokens[:first])
                suffix = "".join(token.text for token in tokens[second + 1:])
                corrupted = prefix + merged + suffix
                start = len(prefix.encode("utf-8"))
                end = start + len(merged.encode("utf-8"))
                span = [{"start": start, "end": end, "label": WORD, "raw": merged, "clean": original}]
                return build_alignment_record(clean_sentence, corrupted, span, "synthetic_space_merge")
        else:
            long_words = [index for index in word_indices if len(tokens[index].text) >= 4]
            if long_words:
                index = rng.choice(long_words)
                word = tokens[index].text
                cut = rng.randint(2, len(word) - 2)
                split_word = word[:cut] + " " + word[cut:]
                prefix = "".join(token.text for token in tokens[:index])
                suffix = "".join(token.text for token in tokens[index + 1:])
                corrupted = prefix + split_word + suffix
                start = len(prefix.encode("utf-8"))
                end = start + len(split_word.encode("utf-8"))
                span = [{"start": start, "end": end, "label": WORD, "raw": split_word, "clean": word}]
                return build_alignment_record(clean_sentence, corrupted, span, "synthetic_space_split")

    edits = 2 if (len(word_indices) >= 8 and rng.random() < 0.25) else 1
    positions = word_indices[:]
    rng.shuffle(positions)
    replacements: dict[int, tuple[str, int]] = {}
    for index in positions:
        if len(replacements) >= edits:
            break
        original = tokens[index].text
        label = WORD
        if category == "tone":
            mutated = mutate_tone(original, rng)
            label = TONE
        elif category == "diacritic_vowel":
            mutated = mutate_diacritic_vowel(original, rng)
            label = PHONETIC
        elif category == "char_drop_dup":
            mutated = mutate_char_drop_or_dup(original, rng)
            label = WORD
        else:
            mutated, label = mutate_phonetic_or_word(original, rng)
        if mutated and mutated != original:
            replacements[index] = (mutated, label)

    if not replacements:
        return None
    parts: list[str] = []
    spans: list[dict] = []
    cursor = 0
    for index, token in enumerate(tokens):
        if index in replacements:
            mutated, label = replacements[index]
            mutated_bytes = len(mutated.encode("utf-8"))
            spans.append({
                "start": cursor,
                "end": cursor + mutated_bytes,
                "label": label,
                "raw": mutated,
                "clean": token.text,
            })
            parts.append(mutated)
            cursor += mutated_bytes
        else:
            parts.append(token.text)
            cursor += len(token.text.encode("utf-8"))
    return build_alignment_record(clean_sentence, "".join(parts), spans, f"synthetic_{category}")


def reservoir_sample_sentences(paths: list[Path], target: int, seed: int) -> list[str]:
    rng = random.Random(seed)
    reservoir: list[str] = []
    seen = 0
    for path in paths:
        if not path.exists():
            print(f"[!] Không thấy corpus: {path}")
            continue
        with path.open(encoding="utf-8", errors="ignore") as handle:
            for line in handle:
                sentence = unicodedata.normalize("NFC", line.strip())
                if not (20 <= len(sentence) <= 240) or not any(char.isalpha() for char in sentence):
                    continue
                seen += 1
                if len(reservoir) < target:
                    reservoir.append(sentence)
                else:
                    index = rng.randint(0, seen - 1)
                    if index < target:
                        reservoir[index] = sentence
    print(f"[Corpus] {seen:,} câu hợp lệ -> reservoir {len(reservoir):,}")
    rng.shuffle(reservoir)
    return reservoir


def load_and_split_vsec(path: Path, holdout_ratio: float, seed: int) -> tuple[list[dict], list[dict]]:
    rows: list[dict] = []
    with path.open(encoding="utf-8") as handle:
        for line in handle:
            if not line.strip():
                continue
            raw = json.loads(line)
            spans = []
            for span in raw.get("spans", []):
                label = span.get("label", WORD)
                if label not in (PHONETIC, TONE, WORD):
                    label = WORD
                spans.append({
                    "start": span["start"],
                    "end": span["end"],
                    "label": label,
                    "raw": span.get("raw", ""),
                    "clean": span.get("clean", ""),
                })
            record = build_alignment_record(raw["clean_text"], raw["text"], spans, "vsec_human")
            if record is not None:
                rows.append(record)
    rng = random.Random(seed)
    rng.shuffle(rows)
    cut = int(len(rows) * (1.0 - holdout_ratio))
    return rows[:cut], rows[cut:]


def write_jsonl(path: Path, records: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        for record in records:
            handle.write(json.dumps(record, ensure_ascii=False) + "\n")


def main() -> None:
    parser = argparse.ArgumentParser(description="D1 v3 Mixed generator")
    parser.add_argument("--corpus", nargs="+", type=Path, default=[
        Path("data_raw/viwiki_clean.txt"), Path("data_raw/leipzig_news.txt")
    ])
    parser.add_argument("--vsec", type=Path, default=Path("data_processed/vsec_bench.jsonl"))
    parser.add_argument("--num-train", type=int, default=600_000)
    parser.add_argument("--num-val", type=int, default=12_000)
    parser.add_argument("--num-test", type=int, default=12_000)
    parser.add_argument("--vsec-mix-ratio", type=float, default=0.15)
    parser.add_argument("--vsec-holdout-ratio", type=float, default=0.20)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--out", type=Path, default=Path("data_processed/neural_mixed_v3"))
    args = parser.parse_args()

    rng = random.Random(args.seed)
    n_vsec = int(args.num_train * args.vsec_mix_ratio) if args.vsec.exists() else 0
    n_syn = args.num_train - n_vsec
    needed = int((n_syn + args.num_val + args.num_test) * 1.15)
    pool = reservoir_sample_sentences(args.corpus, needed, args.seed)
    if not pool:
        raise SystemExit("Không có câu corpus.")
    val_cut = int(len(pool) * 0.03)
    test_cut = int(len(pool) * 0.06)
    val_sentences = pool[:val_cut]
    test_sentences = pool[val_cut:test_cut]
    train_sentences = pool[test_cut:]

    def build_split(sentences: list[str], target: int) -> list[dict]:
        output: list[dict] = []
        index = 0
        while len(output) < target and sentences:
            record = synthesize_one(sentences[index % len(sentences)], rng)
            index += 1
            if record is not None:
                output.append(record)
        return output

    syn_train = build_split(train_sentences, n_syn)
    syn_val = build_split(val_sentences, args.num_val)
    syn_test = build_split(test_sentences, args.num_test)

    vsec_train_upsampled: list[dict] = []
    vsec_holdout: list[dict] = []
    if args.vsec.exists() and n_vsec:
        train_rows, holdout_rows = load_and_split_vsec(
            args.vsec, args.vsec_holdout_ratio, seed=2026
        )
        vsec_holdout = holdout_rows
        while len(vsec_train_upsampled) < n_vsec and train_rows:
            vsec_train_upsampled.append(rng.choice(train_rows))

    mixed = syn_train + vsec_train_upsampled
    rng.shuffle(mixed)
    write_jsonl(args.out / "train.jsonl", mixed)
    write_jsonl(args.out / "val.jsonl", syn_val)
    write_jsonl(args.out / "test.jsonl", syn_test)
    write_jsonl(args.out / "vsec_holdout.jsonl", vsec_holdout)

    stats = {
        "train_total": len(mixed),
        "synthetic_train": len(syn_train),
        "vsec_train_upsampled": len(vsec_train_upsampled),
        "vsec_holdout_locked": len(vsec_holdout),
        "val_total": len(syn_val),
        "test_total": len(syn_test),
        "sources": dict(Counter(record["source"] for record in mixed)),
    }
    (args.out / "stats.json").write_text(
        json.dumps(stats, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(json.dumps(stats, ensure_ascii=False))


if __name__ == "__main__":
    main()
