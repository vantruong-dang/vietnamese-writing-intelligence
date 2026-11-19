"""Đo FPR Tier 3 trên văn bản sạch (upper bound trước Tầng 4 Viet74K).

    python -m neural.eval_fpr \
        --checkpoint neural/checkpoints/vsm_96_4.pt \
        --corpus data_processed/bench_corpus/journalism.txt \
        --corpus data_processed/bench_corpus/legal.txt \
        --max-sentences 10000

Một token bị coi là candidate nếu:
    mean(P_err trên các byte chữ cái) >= 0.85
    và max(P_err) >= 0.92
Đây là ngưỡng Tầng 3; Tầng 4 (Viet74K + bigram) sẽ lọc tiếp.
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

import torch

from .config import ModelConfig
from .model import VSMModel

TOKEN_RE = re.compile(r"[^\W\d_]+", re.UNICODE)


def load_sentences(paths: list[str], max_sentences: int) -> list[str]:
    sentences: list[str] = []
    seen: set[str] = set()
    for raw_path in paths:
        path = Path(raw_path)
        if not path.exists():
            continue
        for line in path.read_text(encoding="utf-8", errors="ignore").splitlines():
            sentence = line.strip()
            if not sentence or "http://" in sentence or "https://" in sentence:
                continue
            if not 3 <= len(TOKEN_RE.findall(sentence)) <= 60:
                continue
            key = sentence.lower()
            if key in seen:
                continue
            seen.add(key)
            sentences.append(sentence)
            if len(sentences) >= max_sentences:
                return sentences
    return sentences


def load_jsonl_clean(path: str, max_sentences: int) -> list[str]:
    sentences: list[str] = []
    with Path(path).open(encoding="utf-8") as handle:
        for line in handle:
            record = json.loads(line)
            if record.get("expected_flag", False):
                continue
            text = record.get("text", "").strip()
            if text:
                sentences.append(text)
                if len(sentences) >= max_sentences:
                    break
    return sentences


def byte_offsets(text: str) -> list[int]:
    offsets = [0]
    total = 0
    for character in text:
        total += len(character.encode("utf-8"))
        offsets.append(total)
    return offsets


@torch.no_grad()
def evaluate(
    checkpoint: str,
    sentences: list[str],
    threshold_mean: float,
    threshold_max: float,
    device: str,
) -> dict[str, object]:
    state = torch.load(checkpoint, map_location=device)
    config = ModelConfig(**state["config"])
    model = VSMModel(config).to(device)
    model.load_state_dict(state["model"])
    model.eval()

    total_tokens = 0
    flagged_tokens = 0
    flagged_sentences = 0
    examples: list[dict[str, object]] = []

    for sentence in sentences:
        text_bytes = list(sentence.encode("utf-8"))
        if not text_bytes:
            continue
        tensor = torch.tensor([text_bytes], dtype=torch.long, device=device)
        error_logits, _ = model(tensor)
        probabilities = torch.softmax(error_logits[0], dim=-1).cpu().tolist()
        offsets = byte_offsets(sentence)
        sentence_flagged = False

        for match in TOKEN_RE.finditer(sentence):
            start = offsets[match.start()]
            end = offsets[match.end()]
            if start >= end or end > len(probabilities):
                continue
            errors = [1.0 - probabilities[index][0] for index in range(start, end)]
            mean = sum(errors) / len(errors)
            maximum = max(errors)
            total_tokens += 1
            if mean >= threshold_mean and maximum >= threshold_max:
                flagged_tokens += 1
                sentence_flagged = True
                if len(examples) < 20:
                    examples.append(
                        {
                            "token": match.group(0),
                            "mean": round(mean, 4),
                            "max": round(maximum, 4),
                            "sentence": sentence[:160],
                        }
                    )
        if sentence_flagged:
            flagged_sentences += 1

    return {
        "sentences": len(sentences),
        "tokens": total_tokens,
        "flagged_tokens": flagged_tokens,
        "token_fpr": flagged_tokens / max(1, total_tokens),
        "flagged_sentences": flagged_sentences,
        "sentence_fpr": flagged_sentences / max(1, len(sentences)),
        "thresholds": {"mean": threshold_mean, "max": threshold_max},
        "examples": examples,
    }


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Đo FPR Tier 3 trên văn bản sạch")
    parser.add_argument("--checkpoint", required=True)
    parser.add_argument("--corpus", action="append", default=None)
    parser.add_argument("--jsonl-clean", default=None)
    parser.add_argument("--max-sentences", type=int, default=10000)
    parser.add_argument("--threshold-mean", type=float, default=0.85)
    parser.add_argument("--threshold-max", type=float, default=0.92)
    parser.add_argument("--device", default="cuda" if torch.cuda.is_available() else "cpu")
    parser.add_argument("--out", default=None)
    return parser.parse_args()


def main() -> None:
    args = _parse_args()
    if args.jsonl_clean:
        sentences = load_jsonl_clean(args.jsonl_clean, args.max_sentences)
    else:
        corpus = args.corpus or [
            "data_processed/bench_corpus/journalism.txt",
            "data_processed/bench_corpus/legal.txt",
            "data_processed/bench_corpus/news.txt",
        ]
        sentences = load_sentences(corpus, args.max_sentences)
    result = evaluate(
        args.checkpoint,
        sentences,
        args.threshold_mean,
        args.threshold_max,
        args.device,
    )
    text = json.dumps(result, ensure_ascii=False, indent=2)
    if args.out:
        Path(args.out).write_text(text + "\n", encoding="utf-8")
    print(text)


if __name__ == "__main__":
    main()
