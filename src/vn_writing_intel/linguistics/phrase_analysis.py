"""Bộ phân tích cụm từ và cấu trúc ngữ pháp chức năng (GS. Cao Xuân Hạo)."""
from __future__ import annotations

from typing import Any, Dict, List
from ..interfaces import BasePhraseAnalyzer, TokenSpan
from .normalization import normalize_nfc


class PhraseAnalyzer(BasePhraseAnalyzer):
    """Phân tích cấu trúc cụm từ và cấu trúc Đề - Thuyết của câu tiếng Việt."""

    def analyze_phrases(self, text: str, tokens: List[TokenSpan]) -> List[Dict[str, Any]]:
        norm_text = normalize_nfc(text)
        phrases: List[Dict[str, Any]] = []

        # 1. Nhận diện cụm mở đầu bằng giới từ (Range Topic / Khung đề tiềm năng)
        if tokens and tokens[0].pos_tag == "E":
            # Tìm xem có dấu phẩy ngắt hay vị từ liền kề
            has_comma = any(t.text == "," for t in tokens[:10])
            phrases.append({
                "type": "prepositional_opening",
                "start": tokens[0].start,
                "end": tokens[min(len(tokens) - 1, 5)].end,
                "has_comma_boundary": has_comma,
                "preposition": tokens[0].text,
            })

        # 2. Nhận diện chuỗi danh từ hóa liên tiếp (Nominalization Chain)
        for i, tok in enumerate(tokens[:-1]):
            if tok.text.lower() in {"việc", "sự", "cuộc", "quá trình"}:
                # Kiểm tra xem theo sau có tiếp tục là danh từ hóa hoặc động từ danh hóa không
                next_toks = tokens[i + 1:i + 4]
                sub_chain = [t.text.lower() for t in next_toks]
                if any(w in {"thực hiện", "quá trình", "tối ưu hóa", "gia tăng"} for w in sub_chain):
                    phrases.append({
                        "type": "nominalization_chain",
                        "start": tok.start,
                        "end": next_toks[-1].end if next_toks else tok.end,
                        "lead_word": tok.text,
                        "chain_tokens": [tok.text] + [t.text for t in next_toks],
                    })

        # 3. Nhận diện cấu trúc đối ngẫu / biền ngẫu cổ điển (Antithesis Parallelism)
        if ";" in norm_text or ("không phải là khó" in norm_text and "mới thật là khó" in norm_text):
            phrases.append({
                "type": "classical_antithesis",
                "start": 0,
                "end": len(norm_text),
                "is_symmetric": True,
            })

        # 4. Nhận diện thể bị động chính danh (Adversative Passive) vs Bị động nhân tạo (Artificial Passive)
        for i, tok in enumerate(tokens):
            if tok.text.lower() in {"bị", "được"}:
                # Bị động nhân tạo có tác tử "bởi"
                has_by = any(t.text.lower() == "bởi" for t in tokens[i + 1:i + 8])
                # Bị động rủi ro chính danh (bị ngã, bị thương, bị đứt)
                is_adversative_genuine = any(
                    t.text.lower() in {"ngã", "gãy", "thương", "đứt", "phạt", "mắng"}
                    for t in tokens[i + 1:i + 4]
                )
                phrases.append({
                    "type": "passive_structure",
                    "start": tok.start,
                    "marker": tok.text,
                    "has_by_agent": has_by,
                    "is_adversative_genuine": is_adversative_genuine,
                })

        return phrases
