"""Bộ kiểm thử mở rộng cho các tình huống biên (Edge Cases & Robustness)."""

import unicodedata

from vn_writing_intel.orchestrator import V3Orchestrator


def get_base_fixtures():
    profile = {
        "profile_id": "test_profile",
        "version": "1.0.0",
        "syntactic_signature": {
            "clause_chaining_style": "hypotactic_causal",
            "allow_long_sentences": True,
            "max_sentence_word_budget": 80,
        },
        "lexical_preferences": {
            "sino_vietnamese_density": "balanced",
            "colloquial_particles_allowed": False,
            "english_term_retention": "strict_preserve_technical",
        },
        "pattern_overrides": {"NOMINALIZATION_HEAVY": {"enforcement": "disabled"}},
    }
    manifest = {
        "discourse_type": "technical_report",
        "immutable_anchors": {
            "metrics": ["100 Gbps", "3.14159 rad/s"],
            "domain_terms": ["zero-cost abstraction", "TP.HCM", "Nguyễn Văn A"],
        },
    }
    return profile, manifest


def test_edge_case_1_protected_blocks_no_false_positives():
    """Tình huống 1: Sáo ngữ nằm trong code block, URL, email hoặc tên file không được báo lỗi."""
    profile, manifest = get_base_fixtures()
    text = (
        "Hệ thống vận hành với tốc độ 100 Gbps tại TP.HCM theo thiết kế của Nguyễn Văn A.\n"
        "Tham khảo tài liệu tại https://example.com/buc-tranh-toan-canh hoặc liên hệ contact@con-dao-hai-luoi.com.\n"
        "Xem cấu hình trong file buc-tranh-toan-canh.json và đoạn mã:\n"
        "```python\n"
        "# con dao hai lưỡi trong code\n"
        "speed = 3.14159 rad/s\n"
        "abstraction = 'zero-cost abstraction'\n"
        "```\n"
    )

    orch = V3Orchestrator(profile)
    res = orch.process(text, manifest)

    # Đảm bảo không bắt nhầm sáo ngữ bên trong URL, email, file name hay code fence
    pids = [p["pattern_id"] for p in res["detected_patterns"]]
    assert "FORMULAIC_METAPHOR" not in pids
    assert res["semantic_audit"]["facts_preserved"] is True


def test_edge_case_2_single_occurrence_vs_frequency():
    """Tình huống 2: Phân biệt quy tắc sáo ngữ tuyệt đối (bắt ngay lần đầu) và quy tắc tần suất."""
    profile, manifest = get_base_fixtures()
    # "Không thể phủ nhận rằng" là sáo ngữ mở đầu tuyệt đối cần dọn dẹp ngay
    draft = "Không thể phủ nhận rằng tốc độ đạt 100 Gbps tại TP.HCM của Nguyễn Văn A với 3.14159 rad/s và zero-cost abstraction."

    orch = V3Orchestrator(profile)
    res = orch.process(draft, manifest)

    assert res["action_mode"] == "clean_rewrite"
    assert not res["output_text"].startswith("Không thể phủ nhận rằng")
    assert res["semantic_audit"]["facts_preserved"] is True


def test_edge_case_3_override_allows_valid_context():
    """Tình huống 3: Pattern xuất hiện nhiều lần nhưng nằm trong phạm vi profile cho phép (override: disabled)."""
    profile, manifest = get_base_fixtures()
    text = (
        "Việc thực hiện kiểm thử tốc độ 100 Gbps và quá trình tối ưu hóa 3.14159 rad/s "
        "đảm bảo zero-cost abstraction tại TP.HCM do Nguyễn Văn A chủ trì."
    )

    orch = V3Orchestrator(profile)
    res = orch.process(text, manifest)

    pids = [p["pattern_id"] for p in res["detected_patterns"]]
    # Rule NOMINALIZATION_HEAVY đã bị disable trong profile nên không được flag
    assert "NOMINALIZATION_HEAVY" not in pids
    assert res["semantic_audit"]["facts_preserved"] is True


def test_edge_case_4_names_and_technical_terms():
    """Tình huống 4: Văn bản chứa tên riêng, thuật ngữ kỹ thuật và số liệu được bảo toàn nguyên vẹn."""
    profile, manifest = get_base_fixtures()
    text = (
        "Báo cáo kỹ thuật: GS. Nguyễn Văn A xác nhận kiến trúc zero-cost abstraction "
        "đạt hiệu năng 100 Gbps và 3.14159 rad/s tại trạm TP.HCM."
    )

    orch = V3Orchestrator(profile)
    res = orch.process(text, manifest)

    audit = res["semantic_audit"]
    assert audit["facts_preserved"] is True
    assert audit["entities_intact"] is True
    assert len(audit["drift_warnings"]) == 0


def test_edge_case_5_unicode_nfd_and_crlf_endings():
    """Tình huống 5: Văn bản tiếng Việt dùng Unicode tổ hợp (NFD) và xuống dòng Windows CRLF (\r\n)."""
    profile, manifest = get_base_fixtures()

    raw_text = (
        "Báo cáo tiến độ hạ tầng:\r\n"
        "Nguyễn Văn A công bố kết quả tại TP.HCM.\r\n"
        "Băng thông 100 Gbps, vận tốc góc 3.14159 rad/s, tối ưu zero-cost abstraction."
    )
    # Ép toàn bộ văn bản sang dạng tổ hợp NFD
    nfd_text = unicodedata.normalize("NFD", raw_text)

    orch = V3Orchestrator(profile)
    res = orch.process(nfd_text, manifest)

    # Dù input là NFD và CRLF, hệ thống chuẩn hóa NFC để đối soát trúng đích
    assert res["semantic_audit"]["facts_preserved"] is True
    assert len(res["semantic_audit"]["drift_warnings"]) == 0


def test_edge_case_6_upstream_graceful_recovery():
    """Tình huống 6: Upstream không khả dụng hoặc trả về cấu trúc lỗi, Adapter tự động phục hồi."""
    profile, manifest = get_base_fixtures()
    text = "Thực tế cho thấy rằng, việc đo đạc 100 Gbps và 3.14159 rad/s tại TP.HCM của Nguyễn Văn A áp dụng zero-cost abstraction."

    orch = V3Orchestrator(profile)
    # Giả lập lỗi upstream
    orch.adapter.upstream_module = None
    orch.adapter.mode = "native_core"

    res = orch.process(text, manifest)
    assert res["engine_mode"] == "native_core"
    assert res["semantic_audit"]["facts_preserved"] is True
    assert res["output_text"].startswith("Việc đo đạc")
