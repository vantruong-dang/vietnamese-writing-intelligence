"""Hệ thống Pipeline thực thi đa tầng (Multi-pass Pipeline Architecture) v0.2.0."""

from __future__ import annotations

import re
import unicodedata
from dataclasses import dataclass, field
from typing import Any

from .interfaces import BaseSegmenter, Finding
from .linguistics import (
    LinguisticSensorEngine,
    get_default_segmenter,
    normalize_nfc,
)
from .migrations import is_v1_manifest, migrate_manifest_v1_to_v2
from .upstream_adapter import UpstreamAdapter
from .validators import default_registry


@dataclass
class PipelineContext:
    """Ngữ cảnh truyền tải dữ liệu xuyên suốt các tầng lọc (Passes) của pipeline."""

    raw_text: str
    manifest: dict[str, Any]
    voice_profile: dict[str, Any]
    normalized_text: str = ""
    cleaned_text: str = ""
    findings: list[Finding] = field(default_factory=list)
    active_findings: list[dict[str, Any]] = field(default_factory=list)
    adapter_status: str = "native_core"
    facts_preserved: bool = True
    entities_intact: bool = True
    drift_warnings: list[str] = field(default_factory=list)
    action_mode: str = "no_change"
    metadata: dict[str, Any] = field(default_factory=dict)


class NormalizationPass:
    """Tầng 1: Chuẩn hóa Unicode NFC và chuẩn bị văn bản phân tích."""

    def run(self, ctx: PipelineContext) -> None:
        ctx.normalized_text = normalize_nfc(ctx.raw_text)
        ctx.cleaned_text = ctx.normalized_text


class SensorAuditPass:
    """Tầng 2: Thu thập phát hiện ngôn ngữ học từ LinguisticSensorEngine và UpstreamAdapter."""

    def __init__(
        self,
        adapter: UpstreamAdapter,
        sensor_engine: LinguisticSensorEngine,
        segmenter: BaseSegmenter | None = None,
    ) -> None:
        self.adapter = adapter
        self.sensor_engine = sensor_engine
        self.segmenter = segmenter or get_default_segmenter()

    def run(self, ctx: PipelineContext) -> None:
        anchors = ctx.manifest.get("immutable_anchors", {})
        protected_terms = list(anchors.get("domain_terms", [])) + list(anchors.get("metrics", []))

        # 1. Thu thập từ UpstreamAdapter (kèm Core Patterns trên văn bản đã mask)
        raw_upstream, status = self.adapter.run_linter(
            ctx.normalized_text, protected_terms=protected_terms
        )
        ctx.adapter_status = status

        # 2. Thu thập từ LinguisticSensorEngine trên văn bản đã mask bảo vệ các khối mã/link
        masked_text = self.adapter.mask_text(ctx.normalized_text, protected_terms=protected_terms)
        tokens = self.segmenter.tokenize(masked_text)
        linguistic_findings = self.sensor_engine.analyze(masked_text, tokens, context=ctx.manifest)

        # Hợp nhất findings tránh trùng lặp pattern_id tại cùng tọa độ
        merged_findings: list[dict[str, Any]] = list(raw_upstream)
        seen_keys = {(f["pattern_id"], f["start"], f["end"]) for f in raw_upstream}

        for lf in linguistic_findings:
            key = (lf.pattern_id, lf.start, lf.end)
            if key not in seen_keys:
                merged_findings.append(lf.to_dict(ctx.normalized_text))
                seen_keys.add(key)

        ctx.active_findings = merged_findings


class ProfileFilterPass:
    """Tầng 3: Áp dụng Voice Profile, lọc rule bị vô hiệu hóa hoặc ghi đè severity."""

    def run(self, ctx: PipelineContext) -> None:
        overrides = ctx.voice_profile.get("pattern_overrides", {})
        filtered: list[dict[str, Any]] = []

        for item in ctx.active_findings:
            pid = item.get("pattern_id", "")
            rule_cfg = overrides.get(pid, {"enforcement": "default"})

            if rule_cfg.get("enforcement") == "disabled":
                continue

            if "severity_override" in rule_cfg:
                item = {**item, "severity": rule_cfg["severity_override"]}

            filtered.append(item)

        ctx.active_findings = filtered


class RewritePass:
    """Tầng 4: Biên tập và làm sạch cơ bản (Clean rewrite) nếu cấu hình cho phép."""

    @staticmethod
    def _clean_opening_cliches(text: str) -> str:
        text = unicodedata.normalize("NFC", text)
        pattern = r"^(Không thể phủ nhận rằng|Có thể nói rằng|Nhìn chung(?!\s+lại)|Thực tế cho thấy rằng)[\s,]*"
        cleaned = re.sub(pattern, "", text.strip(), flags=re.IGNORECASE | re.MULTILINE)
        cleaned = cleaned.lstrip(" ,;-")
        if cleaned and cleaned[0].islower():
            cleaned = cleaned[0].upper() + cleaned[1:]
        return cleaned

    def run(self, ctx: PipelineContext) -> None:
        overrides = ctx.voice_profile.get("pattern_overrides", {})
        if overrides.get("AI_OPENING_CLICHE", {}).get("enforcement") != "disabled":
            ctx.cleaned_text = self._clean_opening_cliches(ctx.raw_text)
        else:
            ctx.cleaned_text = ctx.raw_text


class SemanticAuditPass:
    """Tầng 5: Kiểm định tính toàn vẹn ngữ nghĩa và các mỏ neo bất biến (Immutable Anchors)."""

    @staticmethod
    def _normalize_whitespace(s: str) -> str:
        s = unicodedata.normalize("NFC", s)
        return re.sub(r"\s+", " ", s).strip()

    def run(self, ctx: PipelineContext) -> None:
        warnings: list[str] = []
        anchors = ctx.manifest.get("immutable_anchors", {})
        norm_text = self._normalize_whitespace(ctx.cleaned_text)

        # Kiểm tra số liệu định lượng
        for metric in anchors.get("metrics", []):
            pattern = re.escape(self._normalize_whitespace(metric)).replace(r"\ ", r"\s*")
            if not re.search(pattern, norm_text, flags=re.IGNORECASE):
                warnings.append(f"Mất mát hoặc sai lệch số liệu bất biến: '{metric}'")

        # Kiểm tra thuật ngữ chuyên môn
        for term in anchors.get("domain_terms", []):
            pattern = r"\b" + re.escape(self._normalize_whitespace(term)) + r"\b"
            if not re.search(pattern, norm_text, flags=re.IGNORECASE):
                warnings.append(f"Thuật ngữ chuyên môn bị dịch sai hoặc lược bỏ: '{term}'")

        ctx.drift_warnings = warnings
        ctx.facts_preserved = len(warnings) == 0
        ctx.entities_intact = len(warnings) == 0


class DecisionStateMachinePass:
    """Tầng 6: Máy trạng thái quyết định hành động xuất bản (Action Mode)."""

    def run(self, ctx: PipelineContext) -> None:
        match (ctx.facts_preserved, ctx.cleaned_text == ctx.raw_text, bool(ctx.active_findings)):
            case (False, _, _):
                ctx.action_mode = "needs_author_decision"
            case (True, True, False):
                ctx.action_mode = "no_change"
            case (True, False, _):
                ctx.action_mode = "clean_rewrite"
            case _:
                ctx.action_mode = "review_comment"


class WritingIntelligencePipeline:
    """Động cơ điều phối Pipeline đa tầng."""

    def __init__(
        self,
        adapter: UpstreamAdapter | None = None,
        sensor_engine: LinguisticSensorEngine | None = None,
    ) -> None:
        self.adapter = adapter or UpstreamAdapter()
        self.sensor_engine = sensor_engine or LinguisticSensorEngine()
        self.passes = [
            NormalizationPass(),
            RewritePass(),
            SensorAuditPass(self.adapter, self.sensor_engine),
            ProfileFilterPass(),
            SemanticAuditPass(),
            DecisionStateMachinePass(),
        ]

    def execute(
        self,
        draft_text: str,
        voice_profile: dict[str, Any],
        manifest: dict[str, Any],
        validate: bool = True,
    ) -> dict[str, Any]:
        # Chuẩn hóa manifest và profile
        if validate:
            manifest = default_registry.validate_draft_manifest(manifest)
            default_registry.validate_voice_profile(voice_profile)
        elif is_v1_manifest(manifest):
            manifest, _ = migrate_manifest_v1_to_v2(manifest)

        ctx = PipelineContext(
            raw_text=draft_text,
            manifest=manifest,
            voice_profile=voice_profile,
        )

        for p in self.passes:
            p.run(ctx)

        return {
            "engine_mode": ctx.adapter_status,
            "profile_applied": voice_profile.get("profile_id", "default"),
            "action_mode": ctx.action_mode,
            "original_text": ctx.raw_text,
            "output_text": ctx.cleaned_text if ctx.facts_preserved else ctx.raw_text,
            "detected_patterns": ctx.active_findings,
            "semantic_audit": {
                "facts_preserved": ctx.facts_preserved,
                "entities_intact": ctx.entities_intact,
                "drift_warnings": ctx.drift_warnings,
            },
        }
