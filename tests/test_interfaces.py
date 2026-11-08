"""Kiểm định hệ thống Interfaces và Data Contracts của v0.2.0."""

from vn_writing_intel.interfaces import (
    BaseContextRule,
    BaseLLMProvider,
    BasePhraseAnalyzer,
    BasePOSTagger,
    BaseSegmenter,
    Finding,
    GenerationResult,
    TokenSpan,
)


def test_token_span_creation():
    span = TokenSpan(text="công việc", start=0, end=9, pos_tag="N", is_compound=True)
    assert span.text == "công việc"
    assert span.start == 0
    assert span.end == 9
    assert span.pos_tag == "N"
    assert span.is_compound is True


def test_finding_span_derivation():
    full_text = "Hệ thống phát hiện bức tranh toàn cảnh một cách chính xác."
    finding = Finding(
        pattern_id="FORMULAIC_METAPHOR",
        category="humanizer",
        severity="critical",
        start=19,
        end=38,
        confidence=0.95,
        rationale="Sáo ngữ ẩn dụ",
        source="linguistic",
        metadata={"token_count": 4},
    )

    # span() trích xuất chính xác chuỗi con dựa vào tọa độ start:end
    assert finding.span(full_text) == "bức tranh toàn cảnh"

    # to_dict() serialize đầy đủ
    d = finding.to_dict(full_text)
    assert d["span"] == "bức tranh toàn cảnh"
    assert d["confidence"] == 0.95
    assert d["source"] == "linguistic"


def test_generation_result():
    res = GenerationResult(
        text="Văn bản đã tạo",
        provider="mock",
        model="mock-v1",
        usage={"prompt_tokens": 10, "completion_tokens": 20},
    )
    assert res.text == "Văn bản đã tạo"
    assert res.finish_reason == "stop"
    assert res.usage["completion_tokens"] == 20


def test_protocols_runtime_checkable():
    class DummySegmenter:
        def tokenize(self, text: str):
            return []

    class DummyPOSTagger:
        def tag(self, tokens):
            return tokens

    class DummyPhraseAnalyzer:
        def analyze_phrases(self, text: str, tokens):
            return []

    class DummyContextRule:
        rule_id = "TEST_RULE"

        def evaluate(self, text: str, tokens, context):
            return []

    class DummyLLM:
        def generate(self, system_prompt: str, user_prompt: str, temperature: float = 0.5):
            return GenerationResult(text="", provider="dummy", model="test")

    assert isinstance(DummySegmenter(), BaseSegmenter)
    assert isinstance(DummyPOSTagger(), BasePOSTagger)
    assert isinstance(DummyPhraseAnalyzer(), BasePhraseAnalyzer)
    assert isinstance(DummyContextRule(), BaseContextRule)
    assert isinstance(DummyLLM(), BaseLLMProvider)
