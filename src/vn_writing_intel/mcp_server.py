"""FastMCP Server tối ưu hóa không trạng thái (Stateless & Token-Efficient) cho Claude Desktop / Cursor."""
from __future__ import annotations

import json
from typing import Any, Dict, List, Optional

from .drafting.planner import DraftPlanner
from .orchestrator import V3Orchestrator

try:
    from mcp.server.fastmcp import FastMCP  # type: ignore[import-not-found]
    _MCP_AVAILABLE = True
except (ImportError, ModuleNotFoundError):
    try:
        from mcp.server.mcpserver import MCPServer as FastMCP  # type: ignore[import-not-found]
        _MCP_AVAILABLE = True
    except (ImportError, ModuleNotFoundError):
        FastMCP = None  # type: ignore[misc,assignment]
        _MCP_AVAILABLE = False


def _compress_finding_for_llm(finding: Dict[str, Any]) -> Dict[str, Any]:
    """Cắt tỉa các trường rườm rà, chỉ giữ lại thông tin cốt lõi để tiết kiệm token."""
    return {
        "rule": finding.get("pattern_id"),
        "severity": finding.get("severity"),
        "span": finding.get("span"),
    }


def audit_text(
    text: str,
    profile_id: str = "systems_architect",
    manifest_json: Optional[str] = None,
    compact: bool = True,
) -> str:
    """Kiểm toán văn bản tiếng Việt.

    Quét sáo ngữ AI, câu khuyết tật Khung đề vs Chủ đề, bị động nhân tạo, trùng ngữ
    và đối soát bảo toàn 100% dữ kiện mỏ neo bất biến.

    Khi compact=True, tool cắt bỏ toàn bộ văn bản gốc và khoảng trắng thừa để tiết kiệm token tối đa.
    """
    profile: Dict[str, Any] = {
        "profile_id": profile_id,
        "version": "1.0.0",
        "syntactic_signature": {
            "clause_chaining_style": "topic_comment_native",
            "allow_long_sentences": True,
        },
        "lexical_preferences": {
            "sino_vietnamese_density": "balanced",
            "colloquial_particles_allowed": True,
            "english_term_retention": "preserve_industry_standard",
        },
    }

    if manifest_json:
        try:
            manifest: Dict[str, Any] = json.loads(manifest_json)
        except Exception as e:
            return json.dumps({"error": f"Lỗi cú pháp manifest_json: {e}"}, ensure_ascii=False, separators=(",", ":"))
    else:
        manifest = {
            "discourse_spec": {
                "genre": "essay",
                "register": "professional_sharp",
                "target_audience": "general_public",
            },
            "immutable_anchors": {
                "metrics": [],
                "domain_terms": [],
            },
        }

    orch = V3Orchestrator(profile, validate=False)
    res = orch.process(text, manifest, validate=False)

    if not compact:
        return json.dumps(res, ensure_ascii=False)

    # Payload siêu tối ưu token cho LLM
    compact_payload: Dict[str, Any] = {
        "mode": res["action_mode"],
        "facts_preserved": res["semantic_audit"]["facts_preserved"],
        "findings": [_compress_finding_for_llm(f) for f in res.get("detected_patterns", [])],
    }

    # Nếu có thất thoát dữ kiện, chỉ trả về cảnh báo cụ thể
    if res["semantic_audit"].get("drift_warnings"):
        compact_payload["warnings"] = res["semantic_audit"]["drift_warnings"]

    # Nếu có bản thảo sạch đã biên tập và khác với gốc, chỉ trả về output_text
    if res["action_mode"] == "clean_rewrite" and res.get("output_text"):
        compact_payload["clean_text"] = res["output_text"]

    # Xuất JSON nén (separators không có khoảng trắng thừa)
    return json.dumps(compact_payload, ensure_ascii=False, separators=(",", ":"))


def audit_vietnamese_text(
    text: str,
    profile_id: str = "systems_architect",
    manifest_json: Optional[str] = None,
    compact: bool = True,
) -> str:
    """Alias của audit_text đảm bảo tương thích ngược hoàn toàn."""
    return audit_text(text=text, profile_id=profile_id, manifest_json=manifest_json, compact=compact)


def plan_topic_comment_outline(
    genre: str,
    topic: str,
    intended_points: List[str],
) -> str:
    """Tạo dàn ý Đề - Thuyết (Topic - Comment) thuần Việt từ danh sách luận điểm.

    Ngăn ngừa lối tư duy dịch câu Chủ - Vị gượng ép từ tiếng Anh.
    """
    planner = DraftPlanner()
    genre_valid = genre if genre in planner.GENRE_PROMPT_DESCRIPTIONS else "technical_architecture"
    manifest: Dict[str, Any] = {
        "discourse_spec": {
            "genre": genre_valid,
            "register": "professional_sharp",
            "target_audience": "general_public",
        },
        "topic_comment_plan": [
            {"topic": topic, "intended_comment": p}
            for p in intended_points
        ],
        "immutable_anchors": {
            "metrics": [],
            "domain_terms": [],
        },
    }
    sys_prompt, user_prompt = planner.build_prompts(manifest)

    # Payload nén gọn cho model
    return json.dumps(
        {"sys": sys_prompt, "user": user_prompt},
        ensure_ascii=False,
        separators=(",", ":"),
    )


# Khởi tạo MCP instance module-level nếu thư viện khả dụng
if _MCP_AVAILABLE and FastMCP is not None:
    mcp: Any = FastMCP("vietnamese-writing-intelligence")
    mcp.tool()(audit_text)
    mcp.tool()(audit_vietnamese_text)
    mcp.tool()(plan_topic_comment_outline)
else:
    mcp = None


def run_mcp_cli() -> None:
    """Chạy MCP Server qua giao thức stdio."""
    if mcp is None:
        raise ImportError(
            "Thư viện 'mcp' chưa được cài đặt. "
            "Vui lòng cài đặt bổ sung bằng lệnh: pip install 'vietnamese-writing-intelligence[mcp]'"
        )
    mcp.run(transport="stdio")


if __name__ == "__main__":
    run_mcp_cli()
