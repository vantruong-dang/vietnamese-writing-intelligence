"""Thin Python Wrapper ủy quyền 100% kiểm toán ngôn ngữ học sang lõi Rust (_core)."""

from __future__ import annotations

from typing import Any

from .interfaces import Finding

try:
    from . import _core

    _HAS_RUST_CORE = True
except (ImportError, ModuleNotFoundError):
    _HAS_RUST_CORE = False


def _convert_rust_finding(f: Any) -> Finding:
    """Chuyển đổi đối tượng PyFinding từ FFI Rust sang dataclass Finding chuẩn của Python."""
    return Finding(
        pattern_id=f.pattern_id,
        category=f.category,
        severity=f.severity,
        start=f.start,
        end=f.end,
        confidence=f.confidence,
        rationale=f.rationale,
        source=f.source,
        metadata={"matched_span": f.matched_span},
        suggestion=getattr(f, "suggestion", None),
    )


class AuditorConfig:
    """Cấu hình kiểm toán ngôn ngữ học (ủy quyền trực tiếp sang _core.AuditorConfig)."""

    def __init__(
        self,
        disabled_rules: list[str] | None = None,
        enabled_categories: list[str] | None = None,
        severity_overrides: dict[str, str] | None = None,
    ) -> None:
        if not _HAS_RUST_CORE:
            raise ImportError("Module lõi Rust (_core) chưa được biên dịch.")
        self._inner = _core.AuditorConfig(
            disabled_rules=disabled_rules,
            enabled_categories=enabled_categories,
            severity_overrides=severity_overrides,
        )

    def disable_rule(self, rule_id: str) -> None:
        """Vô hiệu hóa một quy tắc theo pattern_id."""
        self._inner.disable_rule(rule_id)

    def enable_rule(self, rule_id: str) -> None:
        """Kích hoạt lại một quy tắc."""
        self._inner.enable_rule(rule_id)

    def set_enabled_categories(self, categories: list[str]) -> None:
        """Thiết lập danh sách các nhóm thể loại được kích hoạt."""
        self._inner.set_enabled_categories(categories)

    def override_severity(self, rule_id: str, severity: str) -> None:
        """Ghi đè mức độ nghiêm trọng cho một pattern_id."""
        self._inner.override_severity(rule_id, severity)


class Auditor:
    """Động cơ kiểm toán ngôn ngữ học hiệu năng cao chạy hoàn toàn trên lõi Rust.

    Hỗ trợ kiểm toán đơn văn bản (`audit`) và kiểm toán song song đa luồng giải phóng
    Python GIL thông qua Rayon (`audit_batch`).
    """

    def __init__(self, config: AuditorConfig | Any | None = None) -> None:
        if not _HAS_RUST_CORE:
            raise ImportError("Module lõi Rust (_core) chưa được biên dịch.")
        raw_cfg = getattr(config, "_inner", config) if config is not None else None
        self._inner = _core.Auditor(raw_cfg)

    def audit(self, text: str) -> list[Finding]:
        """Kiểm toán một văn bản đơn lẻ, trả về danh sách các phát hiện ngôn ngữ."""
        rust_findings = self._inner.audit(text)
        return [_convert_rust_finding(f) for f in rust_findings]

    def audit_batch(self, texts: list[str]) -> list[list[Finding]]:
        """Kiểm toán song song danh sách nhiều văn bản trên đa luồng CPU (Rayon, no-GIL)."""
        batch_results = self._inner.audit_batch(texts)
        return [[_convert_rust_finding(f) for f in doc_findings] for doc_findings in batch_results]


def audit_linguistics(text: str) -> list[Finding]:
    """Hàm tiện ích chạy toàn bộ 25+ quy tắc kiểm toán ngôn ngữ học trực tiếp từ lõi Rust."""
    if not _HAS_RUST_CORE:
        raise ImportError("Module lõi Rust (_core) chưa được biên dịch.")
    return [_convert_rust_finding(f) for f in _core.audit_linguistics(text)]
