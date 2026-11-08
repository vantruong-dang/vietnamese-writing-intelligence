"""Bộ điều phối trung tâm v0.1.0."""
import re
import unicodedata
from typing import Dict, Any, List, Tuple
from .upstream_adapter import UpstreamAdapter
from .validators import default_registry


class V3Orchestrator:
    def __init__(self, voice_profile: Dict[str, Any], validate: bool = True) -> None:
        if validate:
            default_registry.validate_voice_profile(voice_profile)
        self.profile = voice_profile
        self.adapter = UpstreamAdapter()

    @staticmethod
    def _normalize_whitespace(s: str) -> str:
        s = unicodedata.normalize("NFC", s)
        return re.sub(r"\s+", " ", s).strip()

    def audit_semantic_integrity(self, text: str, manifest: Dict[str, Any]) -> Tuple[bool, List[str]]:
        warnings: List[str] = []
        anchors = manifest.get("immutable_anchors", {})
        norm_text = self._normalize_whitespace(text)

        # Kiểm tra số liệu định lượng (cho phép sai khác khoảng trắng linh hoạt)
        for metric in anchors.get("metrics", []):
            pattern = re.escape(self._normalize_whitespace(metric)).replace(r"\ ", r"\s*")
            if not re.search(pattern, norm_text, flags=re.IGNORECASE):
                warnings.append(f"Mất mát hoặc sai lệch số liệu bất biến: '{metric}'")

        # Kiểm tra thuật ngữ chuyên môn (giữ nguyên ranh giới từ \b)
        for term in anchors.get("domain_terms", []):
            pattern = r"\b" + re.escape(self._normalize_whitespace(term)) + r"\b"
            if not re.search(pattern, norm_text, flags=re.IGNORECASE):
                warnings.append(f"Thuật ngữ chuyên môn bị dịch sai hoặc lược bỏ: '{term}'")

        return len(warnings) == 0, warnings

    def _clean_opening_cliches(self, text: str) -> str:
        text = unicodedata.normalize("NFC", text)
        pattern = r"^(Không thể phủ nhận rằng|Có thể nói rằng|Nhìn chung|Thực tế cho thấy rằng)[\s,]*"
        cleaned = re.sub(pattern, "", text.strip(), flags=re.IGNORECASE | re.MULTILINE)
        cleaned = cleaned.lstrip(" ,;-")
        if cleaned and cleaned[0].islower():
            cleaned = cleaned[0].upper() + cleaned[1:]
        return cleaned

    def process(self, draft_text: str, manifest: Dict[str, Any], validate: bool = True) -> Dict[str, Any]:
        if validate:
            default_registry.validate_draft_manifest(manifest)

        overrides = self.profile.get("pattern_overrides", {})
        raw_findings, adapter_status = self.adapter.run_linter(draft_text)

        active_findings: List[Dict[str, Any]] = []
        for finding in raw_findings:
            pid = finding.get("pattern_id", "")
            rule_cfg = overrides.get(pid, {"enforcement": "default"})

            if rule_cfg.get("enforcement") == "disabled":
                continue

            if "severity_override" in rule_cfg:
                finding = {**finding, "severity": rule_cfg["severity_override"]}

            active_findings.append(finding)

        cleaned_text = draft_text
        if overrides.get("AI_OPENING_CLICHE", {}).get("enforcement") != "disabled":
            cleaned_text = self._clean_opening_cliches(draft_text)

        is_intact, warnings = self.audit_semantic_integrity(cleaned_text, manifest)

        if not is_intact:
            action_mode = "needs_author_decision"
        elif cleaned_text == draft_text and not active_findings:
            action_mode = "no_change"
        elif cleaned_text != draft_text:
            action_mode = "clean_rewrite"
        else:
            action_mode = "review_comment"

        return {
            "engine_mode": adapter_status,
            "profile_applied": self.profile.get("profile_id", "default"),
            "action_mode": action_mode,
            "original_text": draft_text,
            "output_text": cleaned_text if is_intact else draft_text,
            "detected_patterns": active_findings,
            "semantic_audit": {
                "facts_preserved": is_intact,
                "entities_intact": is_intact,
                "drift_warnings": warnings
            }
        }
