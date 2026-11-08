"""Unit tests for Stateless & Token-Efficient FastMCP Server."""

from __future__ import annotations

import json

from vn_writing_intel.mcp_server import (
    _MCP_AVAILABLE,
    audit_text,
    audit_vietnamese_text,
    mcp,
    plan_topic_comment_outline,
)


def test_mcp_server_module_instance():
    if _MCP_AVAILABLE:
        assert mcp is not None
        assert mcp.name == "vietnamese-writing-intelligence"


def test_audit_text_compact_mode():
    sample_text = "Không thể phủ nhận rằng đây là giải pháp tối ưu nhất cho hệ thống."
    res_str = audit_text(sample_text, compact=True)

    # Không có khoảng trắng thừa sau dấu phẩy và hai chấm
    assert ", " not in res_str
    assert ": " not in res_str

    data = json.loads(res_str)
    assert "mode" in data
    assert "facts_preserved" in data
    assert "findings" in data

    # Không echo lại original_text để tiết kiệm token
    assert "original_text" not in data
    assert "semantic_audit" not in data

    # Findings được nén gọn
    if data["findings"]:
        f0 = data["findings"][0]
        assert "rule" in f0
        assert "severity" in f0
        assert "span" in f0
        assert "rationale" not in f0


def test_audit_text_full_mode():
    sample_text = "Không thể phủ nhận rằng đây là giải pháp tối ưu nhất cho hệ thống."
    res_str = audit_text(sample_text, compact=False)
    data = json.loads(res_str)

    assert "action_mode" in data
    assert "original_text" in data
    assert "semantic_audit" in data


def test_audit_vietnamese_text_alias():
    sample_text = "Hệ thống vận hành trơn tru."
    res_compact = audit_vietnamese_text(sample_text, compact=True)
    data = json.loads(res_compact)
    assert data["mode"] == "no_change"
    assert data["facts_preserved"] is True
    assert data["findings"] == []


def test_audit_text_with_custom_manifest():
    sample_text = "Chỉ số VN-Index đóng cửa tại 1250 điểm."
    manifest = {
        "discourse_spec": {
            "genre": "financial_analysis",
            "register": "professional_sharp",
            "target_audience": "domain_engineer",
        },
        "immutable_anchors": {
            "metrics": ["1250 điểm"],
            "domain_terms": ["VN-Index"],
        },
    }
    res_str = audit_text(sample_text, manifest_json=json.dumps(manifest), compact=True)
    data = json.loads(res_str)
    assert data["mode"] == "no_change"
    assert data["facts_preserved"] is True


def test_audit_text_invalid_manifest_json():
    res_str = audit_text("Văn bản kiểm tra.", manifest_json="invalid{json")
    data = json.loads(res_str)
    assert "error" in data


def test_plan_topic_comment_outline():
    topic = "Hạ tầng phân tán"
    points = ["Đảm bảo khả năng chịu lỗi cao", "Tối ưu hóa độ trễ truy vấn"]
    res_str = plan_topic_comment_outline("technical_architecture", topic, points)

    # Nén compact JSON không khoảng trắng phân tách giữa các key
    assert res_str.startswith('{"sys":"')
    assert ',"user":"' in res_str

    data = json.loads(res_str)
    assert "sys" in data
    assert "user" in data
    assert "Topic-Comment Plan" in data["user"]
    assert "Hạ tầng phân tán" in data["user"]
