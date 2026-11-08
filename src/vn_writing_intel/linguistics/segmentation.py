"""Bộ tách từ tiếng Việt dựa trên quy tắc (Rule-based Vietnamese Word Segmenter)."""
from __future__ import annotations

import re
from typing import List, Set
from ..interfaces import BaseSegmenter, TokenSpan
from .normalization import normalize_nfc


# Tập từ vựng và cụm từ ghép tiêu biểu phục vụ phân tích ngữ pháp chức năng
DEFAULT_COMPOUND_VOCAB: Set[str] = {
    "trí tuệ nhân tạo",
    "chuyển đổi số",
    "tối ưu hóa",
    "ban kiểm soát",
    "báo cáo tài chính",
    "chuyên gia nước ngoài",
    "căn phòng",
    "hệ thống phân tán",
    "đã và đang",
    "ngày mai",
    "hôm qua",
    "tuần trước",
    "tuần tới",
    "năm sau",
    "năm ngoái",
    "bởi vì",
    "không thể phủ nhận rằng",
    "có thể nói rằng",
    "nói tóm lại",
    "chí ít",
    "ít nhất",
    "anh trai",
    "chị gái",
    "em gái",
    "thực hiện",
    "quá trình",
    "nguyên nhân",
    "giải pháp",
    "doanh nghiệp",
    "giả thuyết",
    "số liệu",
    "thực nghiệm",
    "tái cơ cấu",
    "bức tranh toàn cảnh",
    "chiếc chìa khóa vạn năng",
    "con dao hai lưỡi",
    "ngọn hải đăng",
    "lau dọn",
    "khởi động",
    "xem xét",
    "áp lực",
    "ký quỹ",
    "cuộc gọi",
    "thuyền trưởng",
    "quyết định",
    "giải thuật",
    "bị can",
    "hiện trường",
    "mặt trời",
    "cáp quang",
    "đường truyền",
}


class RuleBasedSegmenter(BaseSegmenter):
    """Bộ tách từ rule-based tuân thủ giao diện BaseSegmenter.

    Sử dụng giải thuật so khớp tiền tố cực đại (Maximum Matching) trên danh từ/cụm từ
    ghép tiếng Việt, kết hợp phân rã âm tiết cơ bản để luôn bảo toàn tọa độ Unicode NFC chính xác.
    """

    def __init__(self, custom_vocab: Set[str] | None = None) -> None:
        self.vocab = set(DEFAULT_COMPOUND_VOCAB)
        if custom_vocab:
            self.vocab.update(v.lower().strip() for v in custom_vocab)

        # Tính độ dài âm tiết tối đa trong từ điển
        self.max_compound_words = max((len(w.split()) for w in self.vocab), default=4)

    def tokenize(self, text: str) -> List[TokenSpan]:
        norm_text = normalize_nfc(text)
        tokens: List[TokenSpan] = []

        # Tách chuỗi theo ranh giới âm tiết / từ và dấu câu
        # Giữ lại vị trí bắt đầu/kết thúc trên text gốc
        word_iter = re.finditer(r"[\w\d\-]+|[^\w\s]", norm_text, flags=re.UNICODE)
        raw_spans = [(m.group(0), m.start(), m.end()) for m in word_iter]

        idx = 0
        n = len(raw_spans)

        while idx < n:
            matched_compound = False
            # Thử ghép từ max_compound_words giảm dần về 2
            for size in range(self.max_compound_words, 1, -1):
                if idx + size <= n:
                    window = raw_spans[idx:idx + size]
                    # Nếu giữa các token có dấu câu, bỏ qua
                    if any(re.match(r"[^\w\d\-]", s[0]) for s in window):
                        continue
                    compound_candidate = norm_text[window[0][1]:window[-1][2]].lower()
                    if compound_candidate in self.vocab:
                        tokens.append(
                            TokenSpan(
                                text=norm_text[window[0][1]:window[-1][2]],
                                start=window[0][1],
                                end=window[-1][2],
                                is_compound=True,
                            )
                        )
                        idx += size
                        matched_compound = True
                        break

            if not matched_compound:
                tok_text, start, end = raw_spans[idx]
                tokens.append(
                    TokenSpan(
                        text=tok_text,
                        start=start,
                        end=end,
                        is_compound=False,
                    )
                )
                idx += 1

        return tokens
