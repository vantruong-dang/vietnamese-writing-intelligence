#!/usr/bin/env python3
"""
TRÍCH XUẤT POS LEXICON & BIGRAM TRANSITION MATRIX TỪ UNIVERSAL DEPENDENCIES TREEBANK
Đọc dữ liệu vi_vtb-ud-*.conllu trong data_raw/ và xuất mã nguồn Rust tĩnh
crates/vwi-core/src/segmentation/generated_pos.rs.
"""

from __future__ import annotations

import math
import sys
from collections import Counter, defaultdict
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

ROOT_DIR = Path(__file__).resolve().parent.parent
DATA_RAW_DIR = ROOT_DIR / "data_raw"
OUTPUT_RUST_FILE = (
    ROOT_DIR / "crates" / "vwi-core" / "src" / "segmentation" / "generated_pos.rs"
)

# 13 nhãn chuẩn Universal Dependencies (UPOS) ánh xạ sang Rust enum PosTag
UPOS_MAP = {
    "NOUN": "Noun",
    "PROPN": "Noun",
    "VERB": "Verb",
    "AUX": "Verb",
    "ADJ": "Adj",
    "ADP": "Adp",
    "ADV": "Adv",
    "PRON": "Pron",
    "CCONJ": "Cconj",
    "SCONJ": "Sconj",
    "PART": "Part",
    "NUM": "Num",
    "PUNCT": "Punct",
    "SYM": "Sym",
}

TAGS = [
    "Noun",
    "Verb",
    "Adj",
    "Adp",
    "Adv",
    "Pron",
    "Cconj",
    "Sconj",
    "Part",
    "Num",
    "Punct",
    "Sym",
    "Other",
]
TAG_TO_ID = {t: i for i, t in enumerate(TAGS)}

# Bổ sung danh sách từ chức năng đóng (Closed-class Function Words) cốt lõi của tiếng Việt
MANDATORY_LEXICON = {
    # Giới từ (Adp)
    "trong": "Adp",
    "ngoài": "Adp",
    "bằng": "Adp",
    "với": "Adp",
    "qua": "Adp",
    "tại": "Adp",
    "từ": "Adp",
    "đến": "Adp",
    "theo": "Adp",
    "vào": "Adp",
    "lên": "Adp",
    "xuống": "Adp",
    "về": "Adp",
    "ở": "Adp",
    "do": "Adp",
    "bởi": "Adp",
    "giữa": "Adp",
    "suốt": "Adp",
    "dưới": "Adp",
    "trên": "Adp",
    "cho": "Adp",
    # Liên từ phụ thuộc (Sconj)
    "nếu": "Sconj",
    "hễ": "Sconj",
    "giá như": "Sconj",
    "miễn là": "Sconj",
    "vì": "Sconj",
    "bởi vì": "Sconj",
    "do đó": "Sconj",
    "tuy": "Sconj",
    "dù": "Sconj",
    "mặc dù": "Sconj",
    "mặc dầu": "Sconj",
    "để": "Sconj",
    "nhằm": "Sconj",
    "cho nên": "Sconj",
    "thành thử": "Sconj",
    "rốt cuộc": "Sconj",
    # Liên từ đẳng lập (Cconj)
    "và": "Cconj",
    "hoặc": "Cconj",
    "hay": "Cconj",
    "nhưng": "Cconj",
    "song": "Cconj",
    "thế nhưng": "Cconj",
    "tuy nhiên": "Cconj",
    "còn": "Cconj",
    "đồng thời": "Cconj",
    # Trợ từ & Tình thái từ (Part)
    "thì": "Part",
    "mà": "Part",
    "nhé": "Part",
    "nhỉ": "Part",
    "nha": "Part",
    "cơ": "Part",
    "đấy": "Part",
    "thôi": "Part",
    "chính": "Part",
    "ngay": "Part",
    "đích thị": "Part",
    # Đại từ (Pron)
    "tôi": "Pron",
    "ta": "Pron",
    "chúng ta": "Pron",
    "chúng tôi": "Pron",
    "chúng mình": "Pron",
    "nó": "Pron",
    "họ": "Pron",
    "anh ấy": "Pron",
    "chị ấy": "Pron",
    "ông ấy": "Pron",
    "bà ấy": "Pron",
    "hắn": "Pron",
    "mình": "Pron",
    "ai": "Pron",
    "gì": "Pron",
    "đâu": "Pron",
    "nào": "Pron",
    "sao": "Pron",
    # Vị từ nòng cốt hay gặp trong lỗi Khung đề què (Transitive Verbs)
    "đòi hỏi": "Verb",
    "yêu cầu": "Verb",
    "cho thấy": "Verb",
    "chứng minh": "Verb",
    "khẳng định": "Verb",
    "chứng tỏ": "Verb",
    "buộc": "Verb",
    "khiến": "Verb",
    "giúp": "Verb",
    "làm cho": "Verb",
    "phát hiện": "Verb",
    "chỉ ra": "Verb",
    "nêu rõ": "Verb",
    "nhấn mạnh": "Verb",
    "minh chứng": "Verb",
}


def parse_conllu_files() -> tuple[dict[str, Counter], defaultdict[str, Counter]]:
    """Đọc toàn bộ file CoNLL-U trong data_raw/."""
    conllu_files = list(DATA_RAW_DIR.glob("*.conllu"))
    if not conllu_files:
        print(f"[!] Không tìm thấy file .conllu trong {DATA_RAW_DIR}")
        return {}, defaultdict(Counter)

    word_counts: dict[str, Counter] = defaultdict(Counter)
    transition_counts: defaultdict[str, Counter] = defaultdict(Counter)

    for cf in conllu_files:
        print(f"[*] Đang đọc: {cf.name}...")
        prev_tag = "Punct"
        for line in cf.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if not line:
                prev_tag = "Punct"
                continue
            if line.startswith("#"):
                continue
            parts = line.split("\t")
            if len(parts) >= 4:
                form = parts[1].strip().lower()
                upos = parts[3].strip()
                mapped_tag = UPOS_MAP.get(upos, "Other")

                word_counts[form][mapped_tag] += 1
                transition_counts[prev_tag][mapped_tag] += 1
                prev_tag = mapped_tag

    return word_counts, transition_counts


def compute_transition_matrix(
    transition_counts: defaultdict[str, Counter],
) -> list[list[float]]:
    """Tính ma trận xác suất chuyển đổi log-probability: P(tag_j | tag_i)."""
    num_tags = len(TAGS)
    matrix = [[0.0] * num_tags for _ in range(num_tags)]

    for i, tag_from in enumerate(TAGS):
        counts = transition_counts[tag_from]
        total_transitions = sum(counts.values()) + num_tags  # Laplace smoothing +1
        for j, tag_to in enumerate(TAGS):
            c = counts[tag_to] + 1.0  # smoothing
            prob = c / total_transitions
            matrix[i][j] = round(math.log(prob), 4)

    return matrix


def generate_rust_code(
    lexicon: list[tuple[str, str]],
    matrix: list[list[float]],
    out_path: Path,
):
    """Xuất file Rust chứa POS_LEXICON và POS_TRANSITIONS."""
    out_path.parent.mkdir(parents=True, exist_ok=True)
    print(f"[*] Xuất mã nguồn Rust sang: {out_path}...")

    # Sắp xếp từ điển theo bảng chữ cái để binary_search_by_key O(log N)
    lexicon.sort(key=lambda x: x[0])

    with open(out_path, "w", encoding="utf-8") as f:
        f.write("// FILE TỰ ĐỘNG SINH BỞI scripts/build_pos_lexicon.py\n")
        f.write("// KHÔNG SỬA TAY. Được trích xuất từ Universal Dependencies Treebank (VTB).\n\n")

        # 1. POS Lexicon (Sorted Slice for Binary Search)
        f.write(f"/// Bảng từ vựng gắn nhãn POS chuẩn ({len(lexicon)} mục từ, sắp xếp Alphabet O(log N))\n")
        f.write("pub static POS_LEXICON: &[(&str, PosTag)] = &[\n")
        for word, tag in lexicon:
            # Escape dấu ngoặc kép nếu có
            clean_word = word.replace('"', '\\"')
            f.write(f'    ("{clean_word}", PosTag::{tag}),\n')
        f.write("];\n\n")

        # 2. Transition Matrix (13x13 f32 Log-probabilities)
        f.write("/// Ma trận xác suất chuyển trạng thái Bigram POS log-probability: P(tag_to | tag_from)\n")
        f.write("/// Thứ tự hàng/cột: Noun, Verb, Adj, Adp, Adv, Pron, Cconj, Sconj, Part, Num, Punct, Sym, Other\n")
        f.write("pub static POS_TRANSITIONS: [[f32; 13]; 13] = [\n")
        for row in matrix:
            row_str = ", ".join(f"{v:.4f}" for v in row)
            f.write(f"    [{row_str}],\n")
        f.write("];\n\n")

        # 3. Helper Lookup
        f.write("/// Tra cứu O(log N) nhãn từ loại từ bảng tra cứu tĩnh\n")
        f.write("#[inline]\n")
        f.write("pub fn lookup_pos_tag(word: &str) -> Option<PosTag> {\n")
        f.write("    let lower = word.to_lowercase();\n")
        f.write("    POS_LEXICON\n")
        f.write("        .binary_search_by_key(&lower.as_str(), |&(k, _)| k)\n")
        f.write("        .ok()\n")
        f.write("        .map(|idx| POS_LEXICON[idx].1)\n")
        f.write("}\n")

    print(f"[+] Đã hoàn thành! Đã ghi {len(lexicon)} mục từ POS vào lõi Rust.")


def main():
    word_counts, transition_counts = parse_conllu_files()

    lexicon_map: dict[str, str] = {}

    # 1. Nạp từ UD Treebank (chọn nhãn có tần suất xuất hiện cao nhất)
    for word, tag_counter in word_counts.items():
        if len(word) >= 1:
            top_tag, _ = tag_counter.most_common(1)[0]
            lexicon_map[word] = top_tag

    # 2. Ghi đè bằng các từ loại bắt buộc (Closed-class override)
    for word, tag in MANDATORY_LEXICON.items():
        lexicon_map[word] = tag

    lexicon_list = list(lexicon_map.items())
    print(f"[+] Tổng số từ vựng POS thu thập được: {len(lexicon_list)}")

    # 3. Tính toán ma trận chuyển đổi Bigram
    matrix = compute_transition_matrix(transition_counts)

    # 4. Xuất mã nguồn Rust
    generate_rust_code(lexicon_list, matrix, OUTPUT_RUST_FILE)


if __name__ == "__main__":
    main()
