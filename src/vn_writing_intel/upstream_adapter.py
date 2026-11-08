"""Anti-Corruption Layer (Adapter) giao tiếp an toàn với các thư viện upstream."""
from typing import List, Dict, Any, Tuple
import importlib
import re


class UpstreamAdapter:
    """Adapter cách ly hệ thống khỏi sự thay đổi API của upstream packages."""

    CORE_PATTERNS = [
        {
            "id": "AI_OPENING_CLICHE",
            "category": "humanizer",
            "severity": "critical",
            "regex": r"^(không thể phủ nhận rằng|có thể nói rằng|nhìn chung|trong bối cảnh .* hiện nay)[\s,]*",
            "rationale": "Mở đầu rập khuôn kiểu RLHF, làm giảm độ tin cậy của luận điểm."
        },
        {
            "id": "AI_SUMMARY_TAIL",
            "category": "humanizer",
            "severity": "major",
            "regex": r"(tóm lại,|nhìn chung lại,|xét cho cùng,)[\s,]*",
            "rationale": "Từ nối tổng kết máy móc."
        },
        {
            "id": "PASSIVE_TRANSLATIONESE",
            "category": "translationese",
            "severity": "major",
            "regex": r"\b(được xem là|được coi là|được thực hiện bởi)\b",
            "rationale": "Cấu trúc bị động dịch thô từ tiếng Anh."
        },
        {
            "id": "NOMINALIZATION_HEAVY",
            "category": "translationese",
            "severity": "minor",
            "regex": r"\b(việc thực hiện|sự gia tăng của|quá trình tối ưu hóa)\b",
            "rationale": "Danh từ hóa nặng nề làm mất tính hành động của câu văn."
        },
        {
            "id": "FORMULAIC_METAPHOR",
            "category": "humanizer",
            "severity": "critical",
            "regex": r"\b(bức tranh toàn cảnh|chiếc chìa khóa vạn năng|con dao hai lưỡi|ngọn hải đăng)\b",
            "rationale": "Ẩn dụ sáo rỗng đặc trưng của văn bản AI."
        }
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
        out = re.sub(r"```[\s\S]*?```", lambda m: " " * len(m.group(0)), text)
        # Che inline code `...`
        out = re.sub(r"`[^`\n]+`", lambda m: " " * len(m.group(0)), out)
        # Che URLs
        out = re.sub(r"https?://\S+", lambda m: " " * len(m.group(0)), out)
        # Che Emails
        out = re.sub(r"\b[\w.+-]+@[\w.-]+\.\w+\b", lambda m: " " * len(m.group(0)), out)
        # Che File names
        out = re.sub(
            r"\b[\w.-]+\.(?:md|txt|yml|yaml|json|py|ts|tsx|js|jsx|html|css|pdf|docx|csv|png|jpg)\b",
            lambda m: " " * len(m.group(0)),
            out,
            flags=re.IGNORECASE,
        )
        return out

    def mask_text(self, text: str) -> str:
        """Ủy quyền che chắn qua upstream nếu khả dụng, hoặc dùng fallback nội bộ."""
        if self.upstream_module is not None:
            try:
                cli_lint = importlib.import_module("vietnamese_writing_skills.cli.lint")
                mask_fn = getattr(cli_lint, "mask_protected", None)
                if callable(mask_fn):
                    return mask_fn(text)
            except Exception:
                pass
        return self._mask_protected_fallback(text)

    def run_linter(self, text: str) -> Tuple[List[Dict[str, Any]], str]:
        findings: List[Dict[str, Any]] = []
        mode = self.mode

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
                except Exception:
                    pass

            if callable(fn):
                try:
                    raw = fn(text)
                    if isinstance(raw, list):
                        mode = "upstream_active"
                        for item in raw:
                            if isinstance(item, dict):
                                span = item.get("span", "")
                                if not span and item.get("occurrences"):
                                    span = item["occurrences"][0].get("matched_text", "")
                                elif not span:
                                    span = item.get("excerpt", "")
                                findings.append({
                                    "pattern_id": item.get("pattern_id", "UPSTREAM_FINDING"),
                                    "category": item.get("category", item.get("finding_type", "upstream")),
                                    "severity": item.get("severity", "medium"),
                                    "span": span,
                                    "start": item.get("line", 0),
                                    "end": item.get("column", 0),
                                    "rationale": item.get("message", item.get("rationale", item.get("summary", "")))
                                })
                except Exception:
                    pass

        # Quét bổ sung các quy tắc nội tại (Core Patterns) trên văn bản đã được che chắn (masked)
        masked_text = self.mask_text(text)
        for rule in self.CORE_PATTERNS:
            for match in re.finditer(rule["regex"], masked_text, flags=re.IGNORECASE | re.MULTILINE):
                findings.append({
                    "pattern_id": rule["id"],
                    "category": rule["category"],
                    "severity": rule["severity"],
                    "span": text[match.start():match.end()].strip(" ,"),
                    "start": match.start(),
                    "end": match.end(),
                    "rationale": rule["rationale"]
                })

        return findings, mode
