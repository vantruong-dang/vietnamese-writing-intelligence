from vn_writing_intel.orchestrator import V3Orchestrator


def get_base_fixtures():
    profile = {
        "profile_id": "systems_architect",
        "version": "1.0.0",
        "syntactic_signature": {
            "clause_chaining_style": "hypotactic_causal",
            "allow_long_sentences": True,
            "max_sentence_word_budget": 70,
        },
        "lexical_preferences": {
            "sino_vietnamese_density": "high_academic_precision",
            "colloquial_particles_allowed": False,
            "english_term_retention": "strict_preserve_technical",
        },
        "pattern_overrides": {"NOMINALIZATION_HEAVY": {"enforcement": "disabled"}},
    }
    manifest = {
        "discourse_type": "technical_architecture",
        "immutable_anchors": {
            "metrics": ["150W", "20V 7.5A"],
            "domain_terms": ["type erasure", "FFI"],
        },
    }
    return profile, manifest


def test_clean_rewrite_flow():
    profile, manifest = get_base_fixtures()
    draft = "Không thể phủ nhận rằng, việc kiểm tra nguồn 20V 7.5A (150W) và ranh giới FFI giúp ngăn chặn lỗi type erasure."

    orch = V3Orchestrator(profile)
    res = orch.process(draft, manifest)

    assert res["action_mode"] == "clean_rewrite"
    assert res["semantic_audit"]["facts_preserved"] is True
    assert res["output_text"].startswith("Việc kiểm tra nguồn")
    # Rule bị disable không được xuất hiện
    assert "NOMINALIZATION_HEAVY" not in [d["pattern_id"] for d in res["detected_patterns"]]


def test_missing_metric_triggers_needs_author_decision():
    profile, manifest = get_base_fixtures()
    corrupted_draft = "Kiểm tra ranh giới FFI giúp ngăn chặn lỗi type erasure hoàn toàn."

    orch = V3Orchestrator(profile)
    res = orch.process(corrupted_draft, manifest)

    assert res["action_mode"] == "needs_author_decision"
    assert res["semantic_audit"]["facts_preserved"] is False
    assert len(res["semantic_audit"]["drift_warnings"]) == 2
