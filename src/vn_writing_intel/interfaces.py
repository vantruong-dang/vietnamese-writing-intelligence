"""Hệ thống Protocols và Data Contracts trừu tượng cho Linguistic Sensors và Generative Drafter."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Protocol, runtime_checkable


@dataclass(frozen=True)
class TokenSpan:
    """Đại diện cho một token hoặc từ ghép với tọa độ Unicode code points trên văn bản chuẩn hóa."""

    text: str
    start: int  # Tọa độ ký tự bắt đầu (trên văn bản đã chuẩn hóa NFC)
    end: int  # Tọa độ ký tự kết thúc
    pos_tag: str | None = None  # Nhãn từ loại (NOUN, VERB, ADJ, ADP, N, V, A, R, P, E, C, M...)
    is_compound: bool = False  # True nếu là từ ghép tiếng Việt (VD: "công việc", "sự tích")

    @property
    def pos(self) -> str | None:
        """Bí danh tương thích ngược với trường `pos` của lõi Rust (`_core.TokenSpan`)."""
        return self.pos_tag


@dataclass(frozen=True)
class Finding:
    """Phát hiện kiểm định ngôn ngữ hoặc bề mặt với nguồn gốc và độ tin cậy rõ ràng."""

    pattern_id: str
    category: str  # humanizer | translationese | grammar | style | linguistic | semantic
    severity: str  # critical | major | minor | warning | suggestion
    start: int  # Offset bắt đầu trên văn bản NFC
    end: int  # Offset kết thúc trên văn bản NFC
    confidence: float  # Mức độ tin cậy của bộ phát hiện (0.0 - 1.0)
    rationale: str  # Lý giải nguyên nhân phát hiện
    source: str  # linguistic | upstream | semantic | style
    metadata: dict[str, Any] = field(default_factory=dict)

    @property
    def rule_id(self) -> str:
        """Bí danh trực quan cho `pattern_id`."""
        return self.pattern_id

    @property
    def message(self) -> str:
        """Bí danh trực quan cho `rationale`."""
        return self.rationale

    @property
    def matched_span(self) -> str:
        """Chuỗi ký tự vi phạm trích xuất từ metadata."""
        return str(self.metadata.get("matched_span", ""))

    def span(self, text: str) -> str:
        """Trích xuất chuỗi ký tự tương ứng từ văn bản nguồn dựa trên tọa độ start:end."""
        return text[self.start : self.end]

    def to_dict(self, text: str | None = None) -> dict[str, Any]:
        """Chuyển đổi thành từ điển chuẩn hóa, tự động tạo span nếu cung cấp text."""
        matched_span = (
            self.span(text) if text is not None else self.metadata.get("matched_span", "")
        )
        return {
            "pattern_id": self.pattern_id,
            "category": self.category,
            "severity": self.severity,
            "start": self.start,
            "end": self.end,
            "span": matched_span,
            "confidence": self.confidence,
            "rationale": self.rationale,
            "source": self.source,
            "metadata": self.metadata,
        }


@dataclass(frozen=True)
class GenerationResult:
    """Kết quả sinh bản thảo từ LLM Provider bao gồm metadata và mức tiêu hao token."""

    text: str
    provider: str
    model: str
    usage: dict[str, int] = field(default_factory=dict)
    finish_reason: str = "stop"
    raw_response: Any | None = None


@runtime_checkable
class BaseSegmenter(Protocol):
    """Giao diện bộ tách từ tiếng Việt (Word Segmenter)."""

    def tokenize(self, text: str) -> list[TokenSpan]: ...


@runtime_checkable
class BasePOSTagger(Protocol):
    """Giao diện gán nhãn từ loại tiếng Việt (Part-of-Speech Tagger)."""

    def tag(self, tokens: list[TokenSpan]) -> list[TokenSpan]: ...


@runtime_checkable
class BasePhraseAnalyzer(Protocol):
    """Giao diện nhận diện cụm từ và cấu trúc cú pháp (Noun Phrase, Verb Phrase, Nominalization)."""

    def analyze_phrases(self, text: str, tokens: list[TokenSpan]) -> list[dict[str, Any]]: ...


@runtime_checkable
class BaseContextRule(Protocol):
    """Giao diện luật đánh giá ngữ cảnh không dựa vào cấm đoán từ vựng cơ học."""

    rule_id: str

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]: ...


@runtime_checkable
class BaseLLMProvider(Protocol):
    """Giao diện trừu tượng độc lập model/vendor cho Native Drafter."""

    def generate(
        self,
        system_prompt: str,
        user_prompt: str,
        temperature: float = 0.5,
    ) -> GenerationResult: ...
