"""Anti-Corruption Layer (Adapter) giao tiếp an toàn với các thư viện upstream."""

import importlib
import re
from typing import Any, ClassVar


class UpstreamAdapter:
    """Adapter cách ly hệ thống khỏi sự thay đổi API của upstream packages."""

    CORE_PATTERNS: ClassVar[list[dict[str, str]]] = [
        {
            "id": "AI_OPENING_CLICHE",
            "category": "humanizer",
            "severity": "critical",
            "regex": r"^(không thể phủ nhận rằng|có thể nói rằng|nhìn chung(?!\s+lại)|trong bối cảnh .* hiện nay)[\s,]*",
            "rationale": "Mở đầu rập khuôn kiểu RLHF, làm giảm độ tin cậy của luận điểm.",
        },
        {
            "id": "AI_SUMMARY_TAIL",
            "category": "humanizer",
            "severity": "major",
            "regex": r"\b(tóm lại|nói tóm lại|nhìn chung lại|xét cho cùng)[,\s]*",
            "rationale": "Từ nối tổng kết máy móc.",
        },
        {
            "id": "PASSIVE_TRANSLATIONESE",
            "category": "translationese",
            "severity": "major",
            "regex": r"\b(được xem là|được coi là|được thực hiện bởi)\b",
            "rationale": "Cấu trúc bị động dịch thô từ tiếng Anh.",
        },
        {
            "id": "NOMINALIZATION_HEAVY",
            "category": "translationese",
            "severity": "minor",
            "regex": r"\b(việc thực hiện|sự gia tăng của|quá trình tối ưu hóa)\b",
            "rationale": "Danh từ hóa nặng nề làm mất tính hành động của câu văn.",
        },
        {
            "id": "FORMULAIC_METAPHOR",
            "category": "humanizer",
            "severity": "critical",
            "regex": r"\b(bức tranh toàn cảnh|chiếc chìa khóa vạn năng|con dao hai lưỡi|ngọn hải đăng)\b",
            "rationale": "Ẩn dụ sáo rỗng đặc trưng của văn bản AI.",
        },
        {
            "id": "CRIPPLED_PREPOSITIONAL_TOPIC",
            "category": "topic_comment",
            "severity": "major",
            "regex": r"^(trong|qua|bằng|với)\s+[^,]+?\s+(đòi hỏi|yêu cầu|cho thấy|chứng minh|đã chứng minh|giúp|đã giúp|khiến|buộc|đã buộc|đã phát hiện|phát hiện|chứng tỏ)\b",
            "rationale": "Lẫn lộn Khung đề và Chủ đề: Giới từ biến cụm từ thành trạng ngữ/phương tiện, không thể làm chủ thể cho vị từ.",
        },
        {
            "id": "PREPOSITION_CONJUNCTION_CONFLICT",
            "category": "topic_comment",
            "severity": "major",
            "regex": r"^(với những|với|qua|bằng)\s+[^,]+?\s+nhưng\b",
            "rationale": "Xung đột cú pháp: Giới từ mở đầu trạng ngữ không thể kết hợp trực tiếp với liên từ phản đề 'nhưng'.",
        },
        {
            "id": "FACTUAL_CONCESSIVE_MISUSE",
            "category": "concessive_aspect",
            "severity": "major",
            "regex": (
                r"(?:\b(ngày mai|tuần tới|năm sau|sắp tới|sau này)[,\s]+(tuy|mặc dù|mặc dầu)\b|"
                r"\b(tuy|mặc dù|mặc dầu)\b[^,.]+?\b(đi chăng nữa|chăng nữa|đến mấy|thế nào chăng nữa)\b)"
            ),
            "rationale": "Sự việc tương lai giả định (phi hiện thực). 'Tuy' chỉ dùng cho sự việc đã/đang diễn ra; giả định bắt buộc dùng 'Dù'.",
        },
        {
            "id": "HYPOTHETICAL_CONCESSIVE_MISUSE",
            "category": "concessive_aspect",
            "severity": "major",
            "regex": r"\b(hôm qua|năm ngoái|tuần trước|vừa qua|thực tế là|rõ ràng là)[,\s]+dù\b",
            "rationale": "Sự việc quá khứ là hiện thực xác định. Bắt buộc dùng 'Tuy/Mặc dù', không dùng liên từ giả định 'Dù'.",
        },
        {
            "id": "TEMPORAL_IN_TRANSLATIONESE",
            "category": "temporal_framing",
            "severity": "minor",
            "regex": r"\bsẽ\s+[^,.]+?\s+(trong\s+(?:một|hai|ba|bốn|năm|sáu|bảy|tám|chín|mười|\d+)\s+(?:ngày|giờ|phút|tuần|tháng|năm)|sau\s+(?:một|hai|ba|bốn|năm|sáu|bảy|tám|chín|mười|\d+)\s+(?:ngày|giờ|phút|tuần|tháng|năm)\s+nữa)\b",
            "rationale": "Sao phỏng 'in X days': Khoảng cách thời gian tương lai trong tiếng Việt nên đặt làm Khung đề ở đầu câu thay vì cuối câu.",
        },
        {
            "id": "ARTIFICIAL_PASSIVE_BE_BY",
            "category": "pseudo_passive",
            "severity": "major",
            "regex": r"\b(được|bị)\s+[^,.]+?\s+bởi(?!\s+(?:vì|thế|do|tại|vậy))\b",
            "rationale": "Câu bị động nhân tạo sao phỏng công thức 'be + V3 + by' của tiếng Anh.",
        },
        {
            "id": "REDUNDANT_BENEFICIARY_PASSIVE",
            "category": "pseudo_passive",
            "severity": "minor",
            "regex": r"^(căn phòng|báo cáo|hệ thống|dữ liệu|bàn ghế|nhà cửa|cái bàn|tóc nó)\s+(?:đã|sẽ|đang)?\s*(?:được|bị)\s+(lau dọn|xem xét|khởi động|xử lý|lau|cắt)\b",
            "rationale": "Danh ngữ chỉ vật vô tri đứng làm Đề không cần vị từ chỉ sự thụ hưởng 'được'.",
        },
        {
            "id": "LOGICAL_PLEONASM",
            "category": "pleonasm",
            "severity": "minor",
            "regex": r"\b(tối ưu nhất|ánh nắng mặt trời|hoàn toàn tuyệt đối|nguyên nhân\s+[^,.]+?\s+là bởi vì)\b",
            "rationale": "Trùng ngữ ngữ nghĩa (pleonasm): Lặp thừa ý tố biểu đạt.",
        },
        {
            "id": "CXH_VERBAL_SERIES_TENSE_FALSE_MARKER",
            "category": "aspect_mechanics",
            "severity": "minor",
            "regex": r"\b(đã và đang)\b",
            "rationale": "Cụm 'đã và đang' dịch thô từ thì hoàn thành tiếp diễn tiếng Anh, tạo cảm giác lai căng và làm nặng câu văn.",
        },
        {
            "id": "CXH_REPORTING_VERB_INCOMPLETE_CLAUSE",
            "category": "syntax_reporting",
            "severity": "major",
            "regex": r"^(phải nói là|cho là|cho rằng)\s+(?:một\s+)?(?:cú|sự|cuộc|việc|quyết định)\s+[^,.]+[.?!]?$",
            "rationale": "Sau 'phải nói là' bắt buộc phải là một mệnh đề trọn vẹn, không dùng một danh ngữ đơn độc.",
        },
        {
            "id": "CXH_PARALLEL_CONJUNCTION_ASYMMETRY",
            "category": "syntax_parallel",
            "severity": "major",
            "regex": r"^(một\s+[^,]+?),?\s+nhưng\s+(không|chưa|thiếu)\b",
            "rationale": "Liên từ 'nhưng' nối hai vế bất đối xứng: Danh ngữ đứng trước đối lập với cụm vị từ cụt phía sau.",
        },
        {
            "id": "CXH_HYBRID_COMPLEX_CONJUNCTION",
            "category": "syntax_cliche",
            "severity": "minor",
            "regex": r"\bnhư\s+[^,]+?\s+đã\s+biết\s+(?:là|rằng)\b",
            "rationale": "Ghép nhầm hai kiểu câu: 'Như... đã biết' và '... đã biết rằng'.",
        },
        {
            "id": "CXH_SUPERLATIVE_CHI_IT_MISUSE",
            "category": "lexical_misuse",
            "severity": "minor",
            "regex": r"\bchí ít\b",
            "rationale": "'Chí' chỉ dùng cho cực cấp tuyệt đối; dùng 'chí ít' là lai căng; phải dùng từ thuần Việt 'ít nhất' hoặc 'tối thiểu'.",
        },
        {
            "id": "CXH_SUYT_NUA_THI_CONTAMINATION",
            "category": "lexical_misuse",
            "severity": "minor",
            "regex": r"\bsuýt nữa thì\b",
            "rationale": "Lai ghép 'suýt nữa' với 'chút nữa thì' (Cao Xuân Hạo, tr. 163).",
        },
        {
            "id": "TRANSLATIONESE_SOLEIL_SUNSHINE",
            "category": "translationese",
            "severity": "minor",
            "regex": r"^không có mặt trời\b",
            "rationale": "Dịch thô 'soleil/sun' là 'mặt trời' thay vì 'ánh nắng' (Cao Xuân Hạo, tr. 164).",
        },
    ]

    def __init__(self) -> None:
        self.upstream_module = None
        self.mode = "native_core"
        try:
            self.upstream_module = importlib.import_module("vietnamese_writing_skills")
            self.mode = "upstream_linked"
        except ImportError:
            self.mode = "native_core"

    @staticmethod
    def _mask_protected_fallback(text: str) -> str:
        """Che chắn nội dung không nên bắt lỗi (code blocks, inline code, URLs, emails, filenames)."""
        # Che code blocks ```...``` giữ nguyên số dòng và độ dài ký tự
        out = re.sub(
            r"```[\s\S]*?```", lambda m: "".join(c if c == "\n" else "x" for c in m.group(0)), text
        )
        # Che inline code `...`
        out = re.sub(r"`[^`\n]+`", lambda m: "x" * len(m.group(0)), out)
        # Che URLs
        out = re.sub(r"https?://\S+", lambda m: "x" * len(m.group(0)), out)
        # Che Emails
        out = re.sub(r"\b[\w.+-]+@[\w.-]+\.\w+\b", lambda m: "x" * len(m.group(0)), out)
        # Che File names
        out = re.sub(
            r"\b[\w.-]+\.(?:md|txt|yml|yaml|json|py|ts|tsx|js|jsx|html|css|pdf|docx|csv|png|jpg)\b",
            lambda m: "x" * len(m.group(0)),
            out,
            flags=re.IGNORECASE,
        )
        return out

    def mask_text(self, text: str, protected_terms: list[str] | None = None) -> str:
        """Ủy quyền che chắn qua upstream nếu khả dụng, hoặc dùng fallback nội bộ."""
        if self.upstream_module is not None:
            try:
                cli_lint = importlib.import_module("vietnamese_writing_skills.cli.lint")
                mask_fn = getattr(cli_lint, "mask_protected", None)
                if callable(mask_fn):
                    out = mask_fn(text)
                else:
                    out = self._mask_protected_fallback(text)
            except (ImportError, AttributeError, TypeError):
                out = self._mask_protected_fallback(text)
        else:
            out = self._mask_protected_fallback(text)

        if protected_terms:
            for term in protected_terms:
                if not term or not term.strip():
                    continue
                term_pat = r"\b" + re.escape(term.strip()) + r"\b"
                out = re.sub(term_pat, lambda m: "x" * len(m.group(0)), out, flags=re.IGNORECASE)

        return out

    def run_linter(
        self, text: str, protected_terms: list[str] | None = None
    ) -> tuple[list[dict[str, Any]], str]:
        findings: list[dict[str, Any]] = []
        mode = self.mode

        # Chuẩn bị văn bản đã che chắn các thuật ngữ bất biến (domain_terms, metrics) cho upstream
        text_for_upstream = text
        protected_set = {t.strip().lower() for t in (protected_terms or []) if t and t.strip()}
        if protected_set:
            for term in protected_set:
                term_pat = r"\b" + re.escape(term) + r"\b"
                text_for_upstream = re.sub(
                    term_pat,
                    lambda m: "x" * len(m.group(0)),
                    text_for_upstream,
                    flags=re.IGNORECASE,
                )

        if self.upstream_module is not None:
            fn = None
            for attr_name in ("lint_text", "analyze", "lint", "evaluate", "check_text"):
                fn = getattr(self.upstream_module, attr_name, None)
                if callable(fn):
                    break

            if fn is None:
                try:
                    cli_lint = importlib.import_module("vietnamese_writing_skills.cli.lint")
                    fn = getattr(cli_lint, "lint_text", None)
                except (ImportError, AttributeError, TypeError):
                    pass

            if callable(fn):
                try:
                    raw = fn(text_for_upstream)
                    if isinstance(raw, list):
                        mode = "upstream_active"
                        for item in raw:
                            if isinstance(item, dict):
                                span = item.get("span", "")
                                if not span and item.get("occurrences"):
                                    span = item["occurrences"][0].get("matched_text", "")
                                elif not span:
                                    span = item.get("excerpt", "")

                                if span.strip().lower() in protected_set:
                                    continue

                                findings.append(
                                    {
                                        "pattern_id": item.get("pattern_id", "UPSTREAM_FINDING"),
                                        "category": item.get(
                                            "category", item.get("finding_type", "upstream")
                                        ),
                                        "severity": item.get("severity", "medium"),
                                        "span": span,
                                        "start": item.get("line", 0),
                                        "end": item.get("column", 0),
                                        "rationale": item.get(
                                            "message",
                                            item.get("rationale", item.get("summary", "")),
                                        ),
                                    }
                                )
                except (ImportError, AttributeError, TypeError):
                    pass

        # Quét bổ sung các quy tắc nội tại (Core Patterns) trên văn bản đã được che chắn (masked)
        masked_text = self.mask_text(text, protected_terms=protected_terms)
        for rule in self.CORE_PATTERNS:
            for match in re.finditer(
                rule["regex"], masked_text, flags=re.IGNORECASE | re.MULTILINE
            ):
                span = text[match.start() : match.end()].strip(" ,")
                if span.strip().lower() in protected_set:
                    continue
                findings.append(
                    {
                        "pattern_id": rule["id"],
                        "category": rule["category"],
                        "severity": rule["severity"],
                        "span": span,
                        "start": match.start(),
                        "end": match.end(),
                        "rationale": rule["rationale"],
                    }
                )

        return findings, mode
