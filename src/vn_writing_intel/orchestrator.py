"""Bộ điều phối trung tâm v0.2.0 tích hợp kiến trúc Pipeline đa tầng."""

from __future__ import annotations

import re
import unicodedata
from typing import Any

from .pipeline import RewritePass, SemanticAuditPass, WritingIntelligencePipeline
from .upstream_adapter import UpstreamAdapter
from .validators import default_registry


class V3Orchestrator:
    """Bộ điều phối trung tâm thực thi quy trình phân tích và kiểm chuẩn văn bản tiếng Việt."""

    def __init__(self, voice_profile: dict[str, Any], validate: bool = True) -> None:
        if validate:
            default_registry.validate_voice_profile(voice_profile)
        self.profile = voice_profile
        self.adapter = UpstreamAdapter()
        self.pipeline = WritingIntelligencePipeline(adapter=self.adapter)

    @staticmethod
    def _normalize_whitespace(s: str) -> str:
        s = unicodedata.normalize("NFC", s)
        return re.sub(r"\s+", " ", s).strip()

    def audit_semantic_integrity(
        self, text: str, manifest: dict[str, Any]
    ) -> tuple[bool, list[str]]:
        """Kiểm tra sự bảo toàn số liệu và thuật ngữ bất biến từ manifest."""
        from .pipeline import PipelineContext

        ctx = PipelineContext(
            raw_text=text,
            manifest=manifest,
            voice_profile=self.profile,
            cleaned_text=text,
        )
        SemanticAuditPass().run(ctx)
        return ctx.facts_preserved, ctx.drift_warnings

    def _clean_opening_cliches(self, text: str) -> str:
        """Dọn dẹp sáo ngữ mở đầu rập khuôn."""
        return RewritePass._clean_opening_cliches(text)

    def process(
        self, draft_text: str, manifest: dict[str, Any], validate: bool = True
    ) -> dict[str, Any]:
        """Quy trình xử lý hoàn chỉnh thông qua Pipeline Engine."""
        return self.pipeline.execute(
            draft_text=draft_text,
            voice_profile=self.profile,
            manifest=manifest,
            validate=validate,
        )
