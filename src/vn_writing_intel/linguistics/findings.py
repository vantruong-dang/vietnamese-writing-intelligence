"""Tiện ích tạo Finding có cấu trúc cho tầng phân tích ngôn ngữ học."""

from __future__ import annotations

from typing import Any

from ..interfaces import Finding


def create_finding(
    pattern_id: str,
    category: str,
    severity: str,
    start: int,
    end: int,
    rationale: str,
    confidence: float = 1.0,
    source: str = "linguistic",
    metadata: dict[str, Any] | None = None,
) -> Finding:
    """Tạo đối tượng Finding bất biến tuân thủ giao diện Finding."""
    return Finding(
        pattern_id=pattern_id,
        category=category,
        severity=severity,
        start=start,
        end=end,
        confidence=confidence,
        rationale=rationale,
        source=source,
        metadata=metadata or {},
    )
