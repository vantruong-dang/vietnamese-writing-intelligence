"""D1 — Pipeline sinh lỗi tiếng Việt có chủ đích cho VSM.

Nhãn per-byte span (Phương án A):
    0 = CLEAN
    1 = PHONETIC_CONFUSION  (s/x, tr/ch, l/n, r/d/gi, n/ng, t/c...)
    2 = TONE_CONFUSION      (hỏi/ngã, dấu thanh sai)
    3 = WORD_CONFUSION      (dùng nhầm từ thật, tách/gộp từ)

Mỗi record JSONL:
    text, clean_text, spans[{start,end,label}], byte_targets[], byte_mask[], meta

- spans: toàn bộ byte UTF-8 của âm tiết/từ bị lỗi cùng nhãn.
- byte_targets: byte đúng tại từng vị trí input (cho head_byte).
- byte_mask: 1 nếu vị trí đó có target đáng học (căn chỉnh được), 0 nếu bỏ qua.

Không dùng LLM, không phụ thuộc thư viện ngoài.
"""

from __future__ import annotations

import argparse
import difflib
import hashlib
import json
import random
import re
import unicodedata
from collections import Counter
from pathlib import Path
from typing import Any

LABEL_CLEAN = 0
LABEL_PHONETIC = 1
LABEL_TONE = 2
LABEL_WORD = 3

TOKEN_RE = re.compile(r"[^\W\d_]+", re.UNICODE)
URL_RE = re.compile(r"https?://|www\.|[\w-]+\.(?:com|vn|org|net)\b", re.IGNORECASE)

HOI_TO_NGA = str.maketrans(
    "ảẳẩẻểỉỏổởủửỷẢẲẨẺỂỈỎỔỞỦỬỶ",
    "ãẵẫẽễĩõỗỡũữỹÃẴẪẼỄĨÕỖỠŨỮỸ",
)
NGA_TO_HOI = str.maketrans(
    "ãẵẫẽễĩõỗỡũữỹÃẴẪẼỄĨÕỖỠŨỮỸ",
    "ảẳẩẻểỉỏổởủửỷẢẲẨẺỂỈỎỔỞỦỬỶ",
)
INITIAL_GROUPS = [
    ("s", "x"),
    ("tr", "ch"),
    ("l", "n"),
    ("r", "d"),
    ("gi", "d"),
    ("v", "d"),
]
INITIALS = sorted({item for group in INITIAL_GROUPS for item in group}, key=len, reverse=True)
CODA_GROUPS = [("ng", "n"), ("ch", "c"), ("c", "t"), ("n", "ng")]
CODAS = sorted({item for group in CODA_GROUPS for item in group}, key=len, reverse=True)
VOWEL_GROUPS = [("iêu", "ưu"), ("ươu", "ưu"), ("ưu", "iêu")]

WORD_CONFUSIONS: list[tuple[str, str]] = [
    ("sát nhập", "sáp nhập"),
    ("chuẩn đoán", "chẩn đoán"),
    ("thăm quan", "tham quan"),
    ("sử lý", "xử lý"),
    ("bổ xung", "bổ sung"),
    ("chỉnh chu", "chỉn chu"),
    ("dành giật", "giành giật"),
    ("vô hình chung", "vô hình trung"),
    ("chín mùi", "chín muồi"),
    ("mẫu thuẫn", "mâu thuẫn"),
]
WORD_TEMPLATES = [
    "Chúng tôi đang bàn về {phrase} trong cuộc họp.",
    "Đây là vấn đề {phrase} mà ai cũng quan tâm.",
    "Báo cáo đề cập đến {phrase} của dự án.",
    "Chuyên gia giải thích rõ hơn về {phrase}.",
]

DEFAULT_CORPUS = [
    "data_raw/human_gold.txt",
    "data_raw/journalism_corpus.txt",
    "data_raw/news_fresh.txt",
    "data_processed/bench_corpus/journalism.txt",
    "data_processed/bench_corpus/legal.txt",
    "data_processed/bench_corpus/news.txt",
]


def nfc(text: str) -> str:
    return unicodedata.normalize("NFC", text.strip().lower())


def load_lexicon(path: str | Path) -> tuple[set[str], set[str], set[str]]:
    """Trả về (all entries, single syllables, two-syllable words)."""
    entries: set[str] = set()
    for line in Path(path).read_text(encoding="utf-8").splitlines():
        word = nfc(line)
        if word:
            entries.add(word)
    syllables = {word for word in entries if " " not in word and word.isalpha()}
    two_syllable = {
        word
        for word in entries
        if len(word.split()) == 2 and all(part.isalpha() for part in word.split())
    }
    return entries, syllables, two_syllable


def load_sentences(paths: list[str | Path], max_sentences: int = 200_000) -> list[str]:
    sentences: list[str] = []
    seen: set[str] = set()
    for raw_path in paths:
        path = Path(raw_path)
        if not path.exists():
            continue
        for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
            sentence = line.strip()
            if not sentence or URL_RE.search(sentence):
                continue
            words = TOKEN_RE.findall(sentence)
            if not 4 <= len(words) <= 40:
                continue
            key = sentence.lower()
            if key in seen:
                continue
            seen.add(key)
            sentences.append(sentence)
            if len(sentences) >= max_sentences:
                return sentences
    return sentences


def split_initial(syllable: str) -> tuple[str, str]:
    for initial in INITIALS:
        if syllable.startswith(initial) and len(syllable) > len(initial):
            return initial, syllable[len(initial):]
    return "", syllable


def split_coda(syllable: str) -> tuple[str, str]:
    for coda in CODAS:
        if syllable.endswith(coda) and len(syllable) > len(coda):
            return syllable[: -len(coda)], coda
    return syllable, ""


def phonetic_variants(syllable: str, syllables: set[str]) -> list[str]:
    variants: set[str] = set()
    initial, rest = split_initial(syllable)
    for left, right in INITIAL_GROUPS:
        if initial == left:
            variants.add(right + rest)
        elif initial == right:
            variants.add(left + rest)
    base, coda = split_coda(syllable)
    for left, right in CODA_GROUPS:
        if coda == left:
            variants.add(base + right)
        elif coda == right:
            variants.add(base + left)
    for left, right in VOWEL_GROUPS:
        if left in rest:
            variants.add(initial + rest.replace(left, right, 1))
    variants.discard(syllable)
    return sorted(variant for variant in variants if variant in syllables)


def tone_variants(syllable: str, syllables: set[str]) -> list[str]:
    variants = {syllable.translate(HOI_TO_NGA), syllable.translate(NGA_TO_HOI)}
    variants.discard(syllable)
    return sorted(variant for variant in variants if variant in syllables)


def match_case(original: str, replacement: str) -> str:
    if original.isupper():
        return replacement.upper()
    if original[:1].isupper():
        return replacement[:1].upper() + replacement[1:]
    return replacement


def align_segment(
    text_bytes: list[int],
    start: int,
    end: int,
    clean_segment: str,
) -> tuple[list[int], list[int]]:
    """Căn chỉnh byte input -> byte đúng trong đoạn lỗi.

    Vị trí không căn chỉnh được (insert/delete) nhận mask = 0.
    """
    targets = list(text_bytes)
    mask = [0] * len(text_bytes)
    clean_bytes = list(clean_segment.encode("utf-8"))
    input_segment = text_bytes[start:end]
    matcher = difflib.SequenceMatcher(None, input_segment, clean_bytes, autojunk=False)
    for tag, i1, i2, j1, j2 in matcher.get_opcodes():
        if tag not in {"equal", "replace"}:
            continue
        aligned = min(i2 - i1, j2 - j1)
        for offset in range(aligned):
            targets[start + i1 + offset] = clean_bytes[j1 + offset]
            mask[start + i1 + offset] = 1
    return targets, mask


def _record(
    text: str,
    clean_text: str,
    start_char: int,
    end_char: int,
    label: int,
    operator: str,
    source: str,
) -> dict[str, Any] | None:
    if text == clean_text:
        return None
    prefix = text[:start_char]
    suffix = text[end_char:]
    clean_segment = clean_text[len(prefix): len(clean_text) - len(suffix)]
    start = len(prefix.encode("utf-8"))
    end = len((prefix + text[start_char:end_char]).encode("utf-8"))
    text_bytes = list(text.encode("utf-8"))
    if not 0 <= start < end <= len(text_bytes):
        return None
    targets, mask = align_segment(text_bytes, start, end, clean_segment)
    identifier = hashlib.sha1(
        f"{operator}|{text}|{start}|{end}".encode()
    ).hexdigest()[:16]
    return {
        "id": identifier,
        "text": text,
        "clean_text": clean_text,
        "spans": [{"start": start, "end": end, "label": label}],
        "byte_targets": targets,
        "byte_mask": mask,
        "meta": {"operator": operator, "source": source},
    }


def _clean_record(text: str, source: str) -> dict[str, Any]:
    text_bytes = list(text.encode("utf-8"))
    identifier = hashlib.sha1(f"clean|{text}".encode()).hexdigest()[:16]
    return {
        "id": identifier,
        "text": text,
        "clean_text": text,
        "spans": [],
        "byte_targets": text_bytes,
        "byte_mask": [0] * len(text_bytes),
        "meta": {"operator": "clean", "source": source},
    }


def _find_phrase(sentence: str, phrase: str) -> re.Match[str] | None:
    pattern = re.compile(rf"(?<!\w){re.escape(phrase)}(?!\w)", re.IGNORECASE)
    return pattern.search(sentence)


def _make_word_record(sentence: str, rng: random.Random, source: str) -> dict[str, Any]:
    pairs = list(WORD_CONFUSIONS)
    rng.shuffle(pairs)
    for wrong_phrase, correct_phrase in pairs:
        match = _find_phrase(sentence, correct_phrase)
        if match is None:
            continue
        original = match.group(0)
        replacement = match_case(original, wrong_phrase)
        text = sentence[: match.start()] + replacement + sentence[match.end():]
        record = _record(
            text,
            sentence,
            match.start(),
            match.start() + len(replacement),
            LABEL_WORD,
            "word",
            source,
        )
        if record is not None:
            return record

    # Không có cặp nào trong câu -> dựng câu mẫu để đảm bảo đủ nhãn WORD.
    wrong_phrase, correct_phrase = rng.choice(WORD_CONFUSIONS)
    template = rng.choice(WORD_TEMPLATES)
    clean_text = template.format(phrase=correct_phrase)
    start = clean_text.index(correct_phrase)
    replacement = match_case(correct_phrase, wrong_phrase)
    text = clean_text[:start] + replacement + clean_text[start + len(correct_phrase):]
    record = _record(
        text,
        clean_text,
        start,
        start + len(replacement),
        LABEL_WORD,
        "word_template",
        "template",
    )
    if record is None:
        raise RuntimeError("Không sinh được word record.")
    return record


def _make_phonetic_record(
    sentence: str,
    rng: random.Random,
    syllables: set[str],
    source: str,
    tone: bool,
) -> dict[str, Any] | None:
    matches = list(TOKEN_RE.finditer(sentence))
    rng.shuffle(matches)
    for match in matches:
        base = nfc(match.group(0))
        if base not in syllables:
            continue
        variants = tone_variants(base, syllables) if tone else phonetic_variants(base, syllables)
        if not variants:
            continue
        replacement = match_case(match.group(0), rng.choice(variants))
        record = _record(
            sentence[: match.start()] + replacement + sentence[match.end():],
            sentence,
            match.start(),
            match.start() + len(replacement),
            LABEL_TONE if tone else LABEL_PHONETIC,
            "tone" if tone else "phonetic",
            source,
        )
        if record is not None:
            return record
    return None


def _make_ime_record(
    sentence: str,
    rng: random.Random,
    source: str,
) -> dict[str, Any] | None:
    matches = [match for match in TOKEN_RE.finditer(sentence) if len(match.group(0)) >= 3]
    if not matches:
        return None
    rng.shuffle(matches)
    for match in matches:
        replacement = _corrupt_ime(match.group(0), rng)
        if replacement == match.group(0):
            continue
        record = _record(
            sentence[: match.start()] + replacement + sentence[match.end():],
            sentence,
            match.start(),
            match.start() + len(replacement),
            LABEL_PHONETIC,
            "ime",
            source,
        )
        if record is not None:
            return record
    return None


def make_error_record(
    sentence: str,
    rng: random.Random,
    syllables: set[str],
    source: str,
    operator: str,
) -> dict[str, Any] | None:
    if operator == "word":
        return _make_word_record(sentence, rng, source)
    if operator == "tone":
        return _make_phonetic_record(sentence, rng, syllables, source, tone=True)
    if operator == "phonetic":
        return _make_phonetic_record(sentence, rng, syllables, source, tone=False)
    if operator == "ime":
        return _make_ime_record(sentence, rng, source)
    return None


def _corrupt_ime(word: str, rng: random.Random) -> str:
    operation = rng.choice(["transpose", "duplicate", "delete", "insert"])
    chars = list(word)
    if operation == "transpose" and len(chars) >= 3:
        index = rng.randrange(0, len(chars) - 1)
        chars[index], chars[index + 1] = chars[index + 1], chars[index]
    elif operation == "duplicate":
        index = rng.randrange(0, len(chars))
        chars.insert(index, chars[index])
    elif operation == "delete" and len(chars) >= 3:
        del chars[rng.randrange(0, len(chars))]
    else:
        index = rng.randrange(0, len(chars) + 1)
        chars.insert(index, rng.choice("abcdeghiklmnopqrstuvxy"))
    return "".join(chars)


def generate_records(
    count: int,
    rng: random.Random,
    sentences: list[str],
    lexicon_words: list[str],
    syllables: set[str],
    clean_ratio: float = 0.2,
) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    attempts = 0
    max_attempts = count * 20
    operators = ["phonetic", "tone", "word", "ime"]
    operator_index = 0
    while len(records) < count and attempts < max_attempts:
        attempts += 1
        if rng.random() < clean_ratio:
            if sentences:
                sentence = rng.choice(sentences)
            else:
                sentence = " ".join(rng.sample(lexicon_words, k=min(2, len(lexicon_words))))
            records.append(_clean_record(sentence, "corpus" if sentences else "lexicon"))
            continue
        if sentences:
            sentence = rng.choice(sentences)
            source = "corpus"
        else:
            sentence = " ".join(rng.sample(lexicon_words, k=min(3, len(lexicon_words))))
            source = "lexicon"

        preferred = operators[operator_index % len(operators)]
        operator_index += 1
        order = [preferred] + [item for item in operators if item != preferred]
        for operator in order:
            record = make_error_record(sentence, rng, syllables, source, operator)
            if record is not None:
                records.append(record)
                break
    return records


def _stats(records: list[dict[str, Any]]) -> dict[str, Any]:
    labels = Counter()
    operators = Counter()
    for record in records:
        operators[record["meta"]["operator"]] += 1
        if not record["spans"]:
            labels["clean"] += 1
        else:
            labels[str(record["spans"][0]["label"])] += 1
    return {"labels": dict(labels), "operators": dict(operators), "records": len(records)}


def write_jsonl(path: Path, records: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        for record in records:
            handle.write(json.dumps(record, ensure_ascii=False) + "\n")


def generate_dataset(
    output_dir: str | Path,
    lexicon_path: str | Path,
    corpus_paths: list[str | Path],
    train_size: int = 20_000,
    val_size: int = 2_000,
    test_size: int = 2_000,
    clean_ratio: float = 0.2,
    seed: int = 42,
) -> dict[str, Any]:
    _, syllables, two_syllable = load_lexicon(lexicon_path)
    lexicon_words = sorted(two_syllable)
    sentences = load_sentences(corpus_paths)
    if not sentences and not lexicon_words:
        raise ValueError("Không có câu corpus lẫn từ 2 âm tiết trong từ điển.")
    rng = random.Random(seed)

    train_records = generate_records(
        train_size, rng, sentences, lexicon_words, syllables, clean_ratio
    )
    val_records = generate_records(
        val_size, random.Random(seed + 1), sentences, lexicon_words, syllables, clean_ratio
    )
    test_records = generate_records(
        test_size, random.Random(seed + 2), sentences, lexicon_words, syllables, clean_ratio
    )

    output = Path(output_dir)
    write_jsonl(output / "train.jsonl", train_records)
    write_jsonl(output / "val.jsonl", val_records)
    write_jsonl(output / "test.jsonl", test_records)

    stats = {
        "train": _stats(train_records),
        "val": _stats(val_records),
        "test": _stats(test_records),
        "sentences": len(sentences),
        "lexicon_two_syllable": len(lexicon_words),
        "seed": seed,
    }
    (output / "stats.json").write_text(
        json.dumps(stats, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    return stats


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="D1 — sinh dữ liệu lỗi tiếng Việt cho VSM")
    parser.add_argument("--out", default="data_processed/neural")
    parser.add_argument("--lexicon", default="data_raw/Viet74K.txt")
    parser.add_argument("--corpus", action="append", default=None)
    parser.add_argument("--train-size", type=int, default=20_000)
    parser.add_argument("--val-size", type=int, default=2_000)
    parser.add_argument("--test-size", type=int, default=2_000)
    parser.add_argument("--clean-ratio", type=float, default=0.2)
    parser.add_argument("--seed", type=int, default=42)
    return parser.parse_args()


def main() -> None:
    args = _parse_args()
    corpus = args.corpus or DEFAULT_CORPUS
    stats = generate_dataset(
        output_dir=args.out,
        lexicon_path=args.lexicon,
        corpus_paths=corpus,
        train_size=args.train_size,
        val_size=args.val_size,
        test_size=args.test_size,
        clean_ratio=args.clean_ratio,
        seed=args.seed,
    )
    print(json.dumps(stats, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
