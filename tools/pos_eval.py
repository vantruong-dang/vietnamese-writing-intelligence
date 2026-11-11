#!/usr/bin/env python3
"""
Đo 3 baseline POS trên UD_Vietnamese-VTB trước khi port sang Rust.
Baseline A: most-frequent tag (lexicon hiện tại)
Baseline B: HMM Viterbi, OOV = uniform (add-1)
Baseline C: HMM Viterbi + suffix-based OOV emission (len 1..3)

Chỉ emit .rs nếu C >= A + 5 điểm accuracy tổng VÀ C >= A + 5 điểm trên subset đa nghĩa.

Kết quả repo (UD_Vietnamese-VTB, 13 nhãn, train+dev 80/20):
  A most-frequent 89,32 / 64,95; B HMM 86,17 / 65,03; C HMM+suffix 87,40 / 65,91
  (acc / ambig). C không thắng A → KHÔNG port Viterbi.
"""
from __future__ import annotations

import argparse
import math
import random
import sys
from collections import Counter, defaultdict
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

UPOS = [
    "ADJ", "ADP", "ADV", "AUX", "CCONJ", "DET", "INTJ", "NOUN", "NUM",
    "PART", "PRON", "PROPN", "PUNCT", "SCONJ", "SYM", "VERB", "X",
]
UPOS13 = [
    "NOUN", "VERB", "ADJ", "ADP", "ADV", "PRON", "CCONJ", "SCONJ",
    "PART", "NUM", "PUNCT", "SYM", "OTHER",
]
REPO_MAP = {
    "NOUN": "NOUN", "PROPN": "NOUN", "VERB": "VERB", "AUX": "VERB",
    "ADJ": "ADJ", "ADP": "ADP", "ADV": "ADV", "PRON": "PRON",
    "CCONJ": "CCONJ", "SCONJ": "SCONJ", "PART": "PART", "NUM": "NUM",
    "PUNCT": "PUNCT", "SYM": "SYM",
}
PUNCT_TAGS = {"PUNCT", "SYM"}

# Được set trong main theo --tagset
ACTIVE_TAGS = UPOS
TAG_MAP: dict[str, str] = {}
LOWER_FORMS = False
EMIT_ALPHA = 1.0
TRANS_ALPHA = 1.0


def read_conllu(path: str) -> list[list[tuple[str, str]]]:
    sents: list[list[tuple[str, str]]] = []
    cur: list[tuple[str, str]] = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n")
            if not line:
                if cur:
                    sents.append(cur)
                    cur = []
                continue
            if line.startswith("#"):
                continue
            cols = line.split("\t")
            if len(cols) < 4:
                continue
            form, upos = cols[1], cols[3]
            if upos == "_":
                upos = "X"
            upos = TAG_MAP.get(upos, upos)
            if LOWER_FORMS:
                form = form.lower()
            cur.append((form, upos))
    if cur:
        sents.append(cur)
    return sents


def split(sents, dev_ratio=0.2, seed=42):
    rng = random.Random(seed)
    idx = list(range(len(sents)))
    rng.shuffle(idx)
    n_dev = int(len(sents) * dev_ratio)
    dev_set = set(idx[:n_dev])
    train = [sents[i] for i in range(len(sents)) if i not in dev_set]
    dev = [sents[i] for i in idx[:n_dev]]
    return train, dev


def build_model(train):
    emit = defaultdict(Counter)
    trans = defaultdict(Counter)
    tag_count = Counter()
    word_tag = defaultdict(Counter)
    suffix = {k: defaultdict(Counter) for k in (1, 2, 3)}

    for sent in train:
        prev = "<S>"
        for w, t in sent:
            emit[t][w] += 1
            trans[prev][t] += 1
            tag_count[t] += 1
            word_tag[w][t] += 1
            for k in (1, 2, 3):
                suffix[k][w[-k:]][t] += 1
            prev = t
        trans[prev]["</S>"] += 1
    return emit, trans, tag_count, word_tag, suffix


def most_frequent_tag(word_tag):
    return {w: c.most_common(1)[0][0] for w, c in word_tag.items()}


def log_emission(emit, tag_count, V, tag, word):
    c = emit[tag].get(word, 0)
    return math.log((c + EMIT_ALPHA) / (tag_count.get(tag, 0) + EMIT_ALPHA * V))


def log_transition(trans, tag):
    total = sum(trans[tag].values()) + TRANS_ALPHA * (len(ACTIVE_TAGS) + 2)
    return {
        t: math.log((trans[tag].get(t, 0) + TRANS_ALPHA) / total)
        for t in ACTIVE_TAGS
    }


def log_suffix_emission(suffix, tag_count, tag, word):
    for k in (3, 2, 1):
        s = word[-k:]
        if s in suffix[k]:
            c = suffix[k][s].get(tag, 0)
            tot = sum(suffix[k][s].values())
            return math.log((c + 1) / (tot + len(ACTIVE_TAGS)))
    return math.log(1 / len(ACTIVE_TAGS))


def viterbi(sent, emit, trans, tag_count, V, suffix=None, word_tag=None):
    n = len(sent)
    tags = ACTIVE_TAGS
    dp = [[-1e18] * len(tags) for _ in range(n)]
    bp = [[0] * len(tags) for _ in range(n)]
    start = log_transition(trans, "<S>")
    trans_cache = {t: log_transition(trans, t) for t in tags}

    def emit_cost(word, tag):
        # OOV phải được xác định ở mức TỪ (không phải mức tag), nếu không
        # các tag chưa thấy từ sẽ dùng suffix có scale khác và thắng oan.
        is_oov = word_tag is None or word not in word_tag
        if suffix is not None and is_oov:
            return log_suffix_emission(suffix, tag_count, tag, word)
        return log_emission(emit, tag_count, V, tag, word)

    for j, t in enumerate(tags):
        dp[0][j] = start[t] + emit_cost(sent[0][0], t)
    for i in range(1, n):
        for j, t in enumerate(tags):
            e = emit_cost(sent[i][0], t)
            best, arg = -1e18, 0
            for k, pt in enumerate(tags):
                v = dp[i - 1][k] + trans_cache[pt][t]
                if v > best:
                    best, arg = v, k
            dp[i][j] = best + e
            bp[i][j] = arg

    j = max(range(len(tags)), key=lambda k: dp[n - 1][k])
    out = [tags[j]]
    for i in range(n - 1, 0, -1):
        j = bp[i][j]
        out.append(tags[j])
    return list(reversed(out))


def ambiguous_words(word_tag, min_count=3, minority_ratio=0.2):
    amb = set()
    for w, c in word_tag.items():
        tot = sum(c.values())
        if tot < min_count:
            continue
        top = c.most_common(1)[0][1]
        if top / tot < 1 - minority_ratio:
            amb.add(w)
    return amb


def eval_baseline(name, dev, predict, amb):
    correct = tot = 0
    amb_correct = amb_tot = 0
    for sent in dev:
        for (w, gold), pred in zip(sent, predict(sent)):
            tot += 1
            if w in amb:
                amb_tot += 1
            if pred == gold:
                correct += 1
                if w in amb:
                    amb_correct += 1
    acc = correct / tot * 100 if tot else 0
    aacc = amb_correct / amb_tot * 100 if amb_tot else 0
    print(f"{name:<24} acc={acc:6.2f}%  ambig_acc={aacc:6.2f}%  (n_amb={amb_tot})")
    return acc, aacc


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--train", required=True)
    ap.add_argument("--dev", required=True)
    ap.add_argument("--test", default=None, help="Gộp thêm test vào corpus trước khi chia 80/20")
    ap.add_argument("--include-test", action="store_true")
    ap.add_argument("--emit", action="store_true")
    ap.add_argument("--out", default="crates/vwi-core/src/pos")
    ap.add_argument("--tagset", choices=["13", "17"], default="17")
    ap.add_argument("--alpha", type=float, default=1.0)
    args = ap.parse_args()

    global ACTIVE_TAGS, TAG_MAP, LOWER_FORMS, EMIT_ALPHA, TRANS_ALPHA
    EMIT_ALPHA = args.alpha
    TRANS_ALPHA = args.alpha
    if args.tagset == "13":
        ACTIVE_TAGS = UPOS13
        TAG_MAP = REPO_MAP
        LOWER_FORMS = True

    train = read_conllu(args.train) + read_conllu(args.dev)
    if args.include_test and args.test:
        train += read_conllu(args.test)
    tr, dv = split(train)
    print(
        f"train_sents={len(tr)} dev_sents={len(dv)} "
        f"train_tokens={sum(len(s) for s in tr)} dev_tokens={sum(len(s) for s in dv)}"
    )

    emit, trans, tag_count, word_tag, suffix = build_model(tr)
    V = len(word_tag)
    mft = most_frequent_tag(word_tag)
    amb = ambiguous_words(word_tag)
    print(f"vocab={V} ambiguous_words={len(amb)}")

    eval_baseline("A most-frequent", dv, lambda s: [mft.get(w, "NOUN") for w, _ in s], amb)
    eval_baseline(
        "B HMM Viterbi",
        dv,
        lambda s: viterbi(s, emit, trans, tag_count, V, word_tag=word_tag),
        amb,
    )
    accC, ambC = eval_baseline(
        "C HMM + suffix OOV",
        dv,
        lambda s: viterbi(s, emit, trans, tag_count, V, suffix, word_tag=word_tag),
        amb,
    )

    if args.emit:
        print("\n--emit: chỉ ghi file, KHÔNG tự kết luận.")
        Path(args.out).mkdir(parents=True, exist_ok=True)
        print(f"acc(C)={accC:.2f} ambig(C)={ambC:.2f}")


if __name__ == "__main__":
    main()
