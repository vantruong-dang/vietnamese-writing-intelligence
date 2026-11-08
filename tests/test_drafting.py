"""Kiểm thử đơn vị cho hệ thống Native Drafter (Stage 4)."""
from __future__ import annotations

from vn_writing_intel import (
    BaseLLMProvider,
    DraftPlanner,
    MockLLMProvider,
    NativeDrafter,
    V3Orchestrator,
)


def get_sample_manifest_and_profile():
    voice_profile = {
        "profile_id": "tech_lead_sharp",
        "version": "1.0.0",
        "syntactic_signature": {
            "clause_chaining_style": "topic_comment_native",
            "allow_long_sentences": True,
        },
        "lexical_preferences": {
            "sino_vietnamese_density": "balanced",
            "colloquial_particles_allowed": False,
            "english_term_retention": "preserve_industry_standard",
        },
        "banned_phrases": ["bức tranh toàn cảnh", "chiếc chìa khóa vạn năng"],
    }

    manifest = {
        "manifest_id": "draft_001",
        "schema_version": "0.2.0",
        "discourse_spec": {
            "genre": "technical_architecture",
            "register": "professional_sharp",
            "dialect_bias": "all_neutral",
            "target_audience": "domain_engineer",
        },
        "topic_comment_plan": [
            {
                "section_id": "sec_intro",
                "topic": "Hạ tầng phân tán 100 Gbps",
                "comment_guideline": "Phân tích ranh giới FFI giữa Rust và C",
                "clause_chaining_preference": "topic_comment",
            },
            {
                "section_id": "sec_body",
                "topic": "Hiện tượng type erasure",
                "comment_guideline": "Đánh giá nguy cơ vi phạm bộ nhớ và suy giảm hiệu năng",
            },
        ],
        "immutable_anchors": {
            "metrics": ["100 Gbps"],
            "domain_terms": ["FFI", "type erasure"],
            "named_entities": ["Rust", "C"],
            "causal_invariants": ["type erasure làm vỡ hợp đồng an toàn bộ nhớ"],
        },
    }
    return voice_profile, manifest


def test_planner_builds_prompts():
    profile, manifest = get_sample_manifest_and_profile()
    planner = DraftPlanner()

    sys_prompt = planner.build_system_prompt(manifest, profile)
    assert "technical_architecture" in sys_prompt
    assert "domain_engineer" in sys_prompt
    assert "100 Gbps" in sys_prompt
    assert "FFI" in sys_prompt
    assert "bức tranh toàn cảnh" in sys_prompt

    user_prompt = planner.build_user_prompt(manifest, user_instructions="Viết súc tích dưới 200 từ.")
    assert "sec_intro" in user_prompt
    assert "Hạ tầng phân tán 100 Gbps" in user_prompt
    assert "Viết súc tích dưới 200 từ." in user_prompt


def test_mock_llm_provider():
    provider = MockLLMProvider(default_response="Nội dung thử nghiệm 100 Gbps.")
    assert isinstance(provider, BaseLLMProvider)

    res = provider.generate("System prompt", "User prompt")
    assert res.text == "Nội dung thử nghiệm 100 Gbps."
    assert res.provider == "mock"
    assert res.model == "mock-drafter-v1"
    assert res.usage["total_tokens"] > 0
    assert len(provider.history) == 1


def test_native_drafter_full_cycle_intact():
    profile, manifest = get_sample_manifest_and_profile()

    # Provider sinh văn bản thỏa mãn toàn bộ mỏ neo và không chứa sáo ngữ
    good_text = (
        "Hạ tầng phân tán 100 Gbps đòi hỏi sự kiểm soát nghiêm ngặt ranh giới FFI giữa Rust và C. "
        "Hiện tượng type erasure nếu không được xử lý sẽ làm vỡ hợp đồng an toàn bộ nhớ."
    )
    provider = MockLLMProvider(default_response=good_text)
    orchestrator = V3Orchestrator(profile, validate=True)

    drafter = NativeDrafter(provider=provider, orchestrator=orchestrator)
    result = drafter.draft(manifest, profile, auto_verify=True)

    assert result["status"] == "no_change"
    assert result["draft_text"] == good_text
    assert result["verification"]["semantic_audit"]["facts_preserved"] is True
    assert len(result["verification"]["detected_patterns"]) == 0


def test_native_drafter_missing_anchor_triggers_decision():
    profile, manifest = get_sample_manifest_and_profile()

    # Provider sinh văn bản làm rơi mất số liệu "100 Gbps"
    bad_text = (
        "Hạ tầng phân tán tốc độ cao đòi hỏi kiểm soát ranh giới FFI giữa Rust và C. "
        "Hiện tượng type erasure làm vỡ hợp đồng an toàn bộ nhớ."
    )
    provider = MockLLMProvider(default_response=bad_text)
    orchestrator = V3Orchestrator(profile, validate=True)

    drafter = NativeDrafter(provider=provider, orchestrator=orchestrator)
    result = drafter.draft(manifest, profile, auto_verify=True)

    assert result["status"] == "needs_author_decision"
    assert result["verification"]["semantic_audit"]["facts_preserved"] is False
    assert any("100 Gbps" in w for w in result["verification"]["semantic_audit"]["drift_warnings"])


def test_native_drafter_unverified_mode():
    profile, manifest = get_sample_manifest_and_profile()
    provider = MockLLMProvider(default_response="Bản thảo thô chưa qua bộ lọc.")

    drafter = NativeDrafter(provider=provider)
    result = drafter.draft(manifest, profile, auto_verify=False)

    assert result["status"] == "unverified"
    assert result["verification"] is None
