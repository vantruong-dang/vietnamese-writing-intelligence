"""Model Context Protocol (MCP) Server cho Vietnamese Writing Intelligence."""
from __future__ import annotations

import json
from typing import Any, Dict, List, Optional

from .drafting.planner import DraftPlanner
from .orchestrator import V3Orchestrator

try:
    from mcp.server.fastmcp import FastMCP
    _MCP_AVAILABLE = True
except ImportError:
    FastMCP = None  # type: ignore[misc,assignment]
    _MCP_AVAILABLE = False


def create_mcp_server() -> Any:
    """Khởi tạo và cấu hình FastMCP server nếu thư viện mcp khả dụng."""
    if not _MCP_AVAILABLE:
        raise ImportError(
            "Thư viện 'mcp' chưa được cài đặt. "
            "Vui lòng cài đặt bổ sung bằng lệnh: pip install 'vietnamese-writing-intelligence[mcp]'"
        )

    server = FastMCP("vietnamese-writing-intelligence")

    @server.tool()
    def audit_vietnamese_text(
        text: str,
        profile_id: str = "systems_architect",
        manifest_json: Optional[str] = None,
    ) -> str:
        """Kiểm toán văn bản tiếng Việt.

        Quét sáo ngữ AI, câu khuyết tật Khung đề vs Chủ đề, bệnh bị động nhân tạo,
        trùng ngữ Hán - Việt và đối soát bảo toàn mỏ neo dữ kiện bất biến.
        """
        profile = {
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
                return json.dumps({"error": f"Lỗi cú pháp manifest_json: {e}"}, ensure_ascii=False)
        else:
            manifest = {
                "discourse_spec": {
                    "genre": "essay",
                    "register": "professional_sharp",
                    "target_audience": "domain_engineer",
                    "dialect_bias": "all_neutral",
                },
                "immutable_anchors": {
                    "metrics": [],
                    "domain_terms": [],
                },
            }

        orch = V3Orchestrator(profile, validate=False)
        result = orch.process(text, manifest, validate=False)
        return json.dumps(result, ensure_ascii=False, indent=2)

    @server.tool()
    def plan_topic_comment_outline(
        genre: str,
        topic: str,
        intended_points: List[str],
    ) -> str:
        """Lập dàn ý cấu trúc Đề - Thuyết (Topic-Comment) thuần Việt cho một chủ đề.

        Ngăn ngừa lối tư duy dịch câu Chủ - Vị thô từ tiếng Anh.
        """
        planner = DraftPlanner()
        manifest: Dict[str, Any] = {
            "discourse_spec": {
                "genre": genre if genre in planner.GENRE_PROMPT_DESCRIPTIONS else "essay",
                "register": "professional_sharp",
                "target_audience": "domain_engineer",
                "dialect_bias": "all_neutral",
            },
            "topic_comment_plan": [
                {
                    "section_id": f"sec_{idx + 1}",
                    "topic": topic if idx == 0 else f"{topic} (khía cạnh {idx + 1})",
                    "comment_guideline": point,
                    "clause_chaining_preference": "topic_comment",
                }
                for idx, point in enumerate(intended_points)
            ],
            "immutable_anchors": {
                "metrics": [],
                "domain_terms": [],
            },
        }

        default_profile: Dict[str, Any] = {
            "profile_id": "mcp_planner_profile",
            "banned_phrases": [],
        }

        sys_prompt = planner.build_system_prompt(manifest, default_profile)
        user_prompt = planner.build_user_prompt(manifest)

        return json.dumps(
            {
                "system_prompt": sys_prompt,
                "user_prompt": user_prompt,
                "topic_comment_plan": manifest["topic_comment_plan"],
            },
            ensure_ascii=False,
            indent=2,
        )

    return server


def run_mcp_cli() -> None:
    """Chạy MCP Server qua giao thức stdio."""
    server = create_mcp_server()
    server.run(transport="stdio")


if __name__ == "__main__":
    run_mcp_cli()
