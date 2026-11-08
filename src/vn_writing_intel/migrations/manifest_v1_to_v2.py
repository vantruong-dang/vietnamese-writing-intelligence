"""Module di chuyển hợp đồng dữ liệu Manifest từ v0.1.0 sang v0.2.0."""
from __future__ import annotations

from typing import Any, Dict, List, Tuple

GENRE_MAPPING = {
    "technical_architecture": "technical_architecture",
    "financial_analysis": "financial_analysis",
    "policy_brief": "policy_brief",
    "essay": "essay",
    "interdisciplinary_essay": "essay",
    "administrative": "administrative",
    "social_copy": "social_copy",
}


def is_v1_manifest(manifest: Dict[str, Any]) -> bool:
    """Kiểm tra xem manifest có phải định dạng v0.1.0 hay không."""
    return "discourse_type" in manifest and "discourse_spec" not in manifest


def migrate_manifest_v1_to_v2(v1_data: Dict[str, Any]) -> Tuple[Dict[str, Any], List[str]]:
    """Chuyển đổi Manifest v0.1.0 sang canonical v0.2.0.

    Tuân thủ các nguyên tắc:
    1. Không tự suy diễn register hoặc target_audience nếu v1 không cung cấp -> gán 'unspecified'.
    2. Chỉ ánh xạ genre khi có ánh xạ chắc chắn (1-to-1). Nếu không chắc chắn, đặt genre: 'unspecified'
       và lưu giữ giá trị discourse_type gốc trong v1_discourse_type.
    3. Bảo toàn 100% immutable_anchors (metrics, domain_terms).
    4. Bổ sung các mảng mặc định an toàn (named_entities: [], causal_invariants: []).
    """
    notices: List[str] = []
    v2_data: Dict[str, Any] = dict(v1_data)

    if not is_v1_manifest(v1_data):
        return v2_data, notices

    discourse_type = str(v1_data.get("discourse_type", "")).strip()

    # 1. Ánh xạ genre an toàn
    if discourse_type in GENRE_MAPPING:
        genre = GENRE_MAPPING[discourse_type]
        notices.append(f"Ánh xạ discourse_type '{discourse_type}' thành genre '{genre}'.")
    else:
        genre = "unspecified"
        notices.append(
            f"discourse_type '{discourse_type}' không có ánh xạ 1-1 chắc chắn; "
            "đặt genre='unspecified' và lưu v1_discourse_type để tác giả quyết định."
        )

    # 2. Xây dựng discourse_spec (register và target_audience không tự bịa mà gán 'unspecified')
    v2_data["discourse_spec"] = {
        "genre": genre,
        "register": "unspecified",
        "target_audience": "unspecified",
        "dialect_bias": "all_neutral",
    }
    v2_data["v1_discourse_type"] = discourse_type
    v2_data.pop("discourse_type", None)

    # 3. Bảo toàn và chuẩn hóa immutable_anchors
    anchors = dict(v1_data.get("immutable_anchors", {}))
    anchors.setdefault("metrics", [])
    anchors.setdefault("domain_terms", [])
    anchors.setdefault("named_entities", [])
    anchors.setdefault("causal_invariants", [])
    v2_data["immutable_anchors"] = anchors

    # 4. Đánh dấu phiên bản schema
    v2_data["schema_version"] = "0.2.0"
    v2_data.setdefault("topic_comment_plan", [])

    return v2_data, notices
