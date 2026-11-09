"""Bộ tách từ tiếng Việt dựa trên quy tắc (Rule-based Vietnamese Word Segmenter)."""

from __future__ import annotations

import importlib.util
import re

from ..interfaces import BaseSegmenter, TokenSpan
from .normalization import normalize_nfc

# Tập từ vựng và cụm từ ghép tiêu biểu phục vụ phân tích ngữ pháp chức năng
DEFAULT_COMPOUND_VOCAB: set[str] = {
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

    def __init__(self, custom_vocab: set[str] | None = None) -> None:
        self.vocab = set(DEFAULT_COMPOUND_VOCAB)
        if custom_vocab:
            self.vocab.update(v.lower().strip() for v in custom_vocab)

        # Tính độ dài âm tiết tối đa trong từ điển
        self.max_compound_words = max((len(w.split()) for w in self.vocab), default=4)

    def tokenize(self, text: str) -> list[TokenSpan]:
        norm_text = normalize_nfc(text)
        tokens: list[TokenSpan] = []

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
                    window = raw_spans[idx : idx + size]
                    # Nếu giữa các token có dấu câu, bỏ qua
                    if any(re.match(r"[^\w\d\-]", s[0]) for s in window):
                        continue
                    compound_candidate = norm_text[window[0][1] : window[-1][2]].lower()
                    if compound_candidate in self.vocab:
                        tokens.append(
                            TokenSpan(
                                text=norm_text[window[0][1] : window[-1][2]],
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


class UndertheseaSegmenter(BaseSegmenter):
    """Segmenter nâng cao kết nối với Underthesea (khi người dùng cài [nlp])."""

    def __init__(self) -> None:
        if not importlib.util.find_spec("underthesea"):
            raise ImportError(
                "Thư viện 'underthesea' chưa được cài đặt. "
                "Vui lòng chạy: pip install 'vietnamese-writing-intelligence[nlp]'"
            )
        import underthesea  # type: ignore[import-untyped]

        self._uts = underthesea

    def tokenize(self, text: str) -> list[TokenSpan]:
        norm = normalize_nfc(text)
        raw_tokens: list[str] = self._uts.word_tokenize(norm)

        spans: list[TokenSpan] = []
        current_idx = 0
        for tok in raw_tokens:
            start = norm.find(tok, current_idx)
            if start == -1:
                space_tok = tok.replace("_", " ")
                start = norm.find(space_tok, current_idx)
                if start != -1:
                    tok = space_tok

            if start == -1:
                start = current_idx

            end = start + len(tok)
            current_idx = end

            is_compound = " " in tok or "_" in tok
            spans.append(TokenSpan(text=tok, start=start, end=end, is_compound=is_compound))

        return spans


def get_default_segmenter(backend: str = "auto") -> BaseSegmenter:
    """Factory function tự động chọn backend phân đoạn từ tối ưu nhất."""
    match backend:
        case "underthesea":
            return UndertheseaSegmenter()
        case "rule_based":
            return RuleBasedSegmenter()
        case "auto":
            if importlib.util.find_spec("underthesea"):
                try:
                    return UndertheseaSegmenter()
                except (ImportError, AttributeError):
                    return RuleBasedSegmenter()
            return RuleBasedSegmenter()
        case _:
            raise ValueError(
                f"Backend không hợp lệ: '{backend}'. Chọn 'auto', 'underthesea', hoặc 'rule_based'."
            )
