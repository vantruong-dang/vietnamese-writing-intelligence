"""Bộ gán nhãn từ loại tiếng Việt theo ngữ pháp chức năng (GS. Cao Xuân Hạo)."""
from __future__ import annotations

from typing import Dict, List, Set
from ..interfaces import BasePOSTagger, TokenSpan


# Từ điển phân loại từ vựng phục vụ gán nhãn chức năng
PREPOSITIONS: Set[str] = {
    "trong", "ngoài", "bằng", "với", "qua", "tại", "sau", "trước", "từ", "đến", "theo", "vào"
}

CONJUNCTIONS: Set[str] = {
    "tuy", "dù", "mặc dù", "mặc dầu", "nhưng", "mà", "thì", "và", "hoặc", "bởi vì",
    "rằng", "là", "nên", "cho nên", "do đó", "nếu", "hễ", "giá như"
}

NOMINALIZERS: Set[str] = {
    "việc", "sự", "cuộc", "cú", "quá trình", "chiếc", "căn", "ngọn", "bức", "tấm", "con"
}

KINSHIP_TERMS: Set[str] = {
    "anh trai", "chị gái", "em gái", "em trai", "anh", "chị", "em", "cha", "mẹ", "bố", "con", "anh em"
}

ASPECT_MARKERS: Set[str] = {
    "đã", "đang", "sẽ", "đã và đang", "vừa", "mới", "từng", "sắp"
}

ADVERBS_AND_NEGATION: Set[str] = {
    "không", "chưa", "chẳng", "cũng", "đều", "rất", "quá", "lắm", "nhất", "tất", "cả", "hãy", "chớ", "đừng"
}

PRONOUNS: Set[str] = {
    "tôi", "ta", "chúng ta", "chúng tôi", "họ", "anh ấy", "cô ấy", "ông ấy", "bà ta", "nó", "mình",
    "gì", "ai", "đâu", "nào", "sao"
}

KEY_VERBS: Set[str] = {
    "đòi hỏi", "chứng minh", "mua", "bán", "sửa", "đến", "nỗ lực", "tối ưu", "tối ưu hóa",
    "xem xét", "lau dọn", "khởi động", "chối", "nhận", "ngã", "gãy", "thành", "làm", "tạo",
    "tăng trưởng", "thủng", "quét", "phủ nhận", "nói", "biết", "cho là", "cho rằng", "phải nói là"
}


class RuleBasedPOSTagger(BasePOSTagger):
    """Bộ gán nhãn từ loại rule-based tuân thủ BasePOSTagger."""

    def __init__(self, custom_tags: Dict[str, str] | None = None) -> None:
        self.custom_tags = custom_tags or {}

    def tag(self, tokens: List[TokenSpan]) -> List[TokenSpan]:
        tagged_tokens: List[TokenSpan] = []

        for tok in tokens:
            lower = tok.text.lower()
            tag = self.custom_tags.get(lower)

            if tag is None:
                if lower in PREPOSITIONS:
                    tag = "E"  # Giới từ
                elif lower in CONJUNCTIONS:
                    tag = "C"  # Liên từ
                elif lower in NOMINALIZERS:
                    tag = "Nc"  # Loại từ / Danh từ hóa
                elif lower in KINSHIP_TERMS:
                    tag = "Kin"  # Thân tộc
                elif lower in ASPECT_MARKERS:
                    tag = "Asp"  # Tác tử thể / thì
                elif lower in ADVERBS_AND_NEGATION:
                    tag = "R"  # Phó từ / phủ định
                elif lower in PRONOUNS:
                    tag = "P"  # Đại từ
                elif lower in KEY_VERBS:
                    tag = "V"  # Vị từ
                elif tok.text.isdigit():
                    tag = "M"  # Số từ
                elif len(tok.text) == 1 and not tok.text.isalnum():
                    tag = "CH"  # Dấu câu
                else:
                    tag = "N"  # Danh từ hoặc từ vựng thực từ mặc định

            tagged_tokens.append(
                TokenSpan(
                    text=tok.text,
                    start=tok.start,
                    end=tok.end,
                    pos_tag=tag,
                    is_compound=tok.is_compound,
                )
            )

        return tagged_tokens
