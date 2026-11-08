"""Linguistic Gold Benchmark Dataset Validation Suite.

Kiểm chuẩn toàn diện 38 ca trích xuất từ các công trình nghiên cứu kinh điển:
- GS. Cao Xuân Hạo: Tiếng Việt - Sơ thảo ngữ pháp chức năng & Văn Việt Người Việt.
- Học giả Nguyễn Hiến Lê: Cổ văn Trung Quốc.
- Nhà nghiên cứu Trương Thái Du: Ngữ nguyên tiếng Việt: 600 từ vựng cơ bản.
"""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Dict, List
import pytest

from vn_writing_intel import V3Orchestrator


GOLD_DATASET_PATH = Path(__file__).parent / "data" / "linguistic_gold_standard.jsonl"


def load_gold_cases() -> List[Dict[str, Any]]:
    cases: List[Dict[str, Any]] = []
    assert GOLD_DATASET_PATH.exists(), f"Không tìm thấy file dataset tại {GOLD_DATASET_PATH}"
    with open(GOLD_DATASET_PATH, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                cases.append(json.loads(line))
    return cases


ALL_GOLD_CASES = load_gold_cases()

DEFAULT_VOICE_PROFILE: Dict[str, Any] = {
    "profile_id": "benchmark_validator",
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
    "pattern_overrides": {},
}

DEFAULT_BENCHMARK_MANIFEST: Dict[str, Any] = {
    "manifest_id": "benchmark_manifest",
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


@pytest.mark.parametrize("case", ALL_GOLD_CASES, ids=[c["id"] for c in ALL_GOLD_CASES])
def test_individual_linguistic_case(case: Dict[str, Any]) -> None:
    orchestrator = V3Orchestrator(DEFAULT_VOICE_PROFILE, validate=True)
    manifest = case.get("manifest") or DEFAULT_BENCHMARK_MANIFEST

    result = orchestrator.process(case["input_text"], manifest, validate=True)
    expected_action = case["expected_action"]

    # 1. Kiểm tra action_mode khớp tuyệt đối với kỳ vọng
    assert result["action_mode"] == expected_action, (
        f"[{case['id']}] Sai action_mode! Kỳ vọng: '{expected_action}', Thực tế: '{result['action_mode']}'. "
        f"Lý do: {case.get('linguistic_rationale')}"
    )

    # 2. Kiểm tra theo từng loại action
    if expected_action == "clean_rewrite":
        assert result["output_text"].strip() == case["expected_output"].strip(), (
            f"[{case['id']}] Kết quả viết lại không khớp với expected_output."
        )

    elif expected_action == "no_change":
        # Không được phép có false positive
        detected_pids = [p["pattern_id"] for p in result["detected_patterns"]]
        assert len(result["detected_patterns"]) == 0, (
            f"[{case['id']}] False Positive! Bắt nhầm câu hợp lệ với patterns: {detected_pids}."
        )
        assert result["output_text"] == case["input_text"]

    elif expected_action == "review_comment":
        detected_pids = [p["pattern_id"] for p in result["detected_patterns"]]
        expected_rule = case["rule_id"]
        assert expected_rule in detected_pids, (
            f"[{case['id']}] Không bắt được rule '{expected_rule}'. Các rule bắt được: {detected_pids}."
        )

    elif expected_action == "needs_author_decision":
        assert not result["semantic_audit"]["facts_preserved"] or not result["semantic_audit"]["entities_intact"], (
            f"[{case['id']}] Vi phạm anchor không được audit_semantic_integrity ghi nhận cảnh báo!"
        )


def test_benchmark_aggregate_metrics(capsys: pytest.CaptureFixture[str]) -> None:
    """Tính toán ma trận nhầm lẫn (Confusion Matrix) và in báo cáo định lượng."""
    orchestrator = V3Orchestrator(DEFAULT_VOICE_PROFILE, validate=True)

    total = len(ALL_GOLD_CASES)
    tp = 0  # Bắt đúng lỗi (cần can thiệp: clean_rewrite, review_comment, needs_author_decision)
    fp = 0  # Bắt nhầm câu hợp lệ (kỳ vọng no_change nhưng lại can thiệp)
    tn = 0  # Bỏ qua đúng câu hợp lệ (no_change)
    fn = 0  # Bỏ sót lỗi (kỳ vọng can thiệp nhưng lại no_change)

    rule_coverage: Dict[str, int] = {}

    for case in ALL_GOLD_CASES:
        manifest = case.get("manifest") or DEFAULT_BENCHMARK_MANIFEST
        result = orchestrator.process(case["input_text"], manifest, validate=True)

        is_anomaly = case["expected_action"] != "no_change"
        predicted_anomaly = result["action_mode"] != "no_change"

        if is_anomaly and predicted_anomaly:
            tp += 1
            rule_coverage[case["rule_id"]] = rule_coverage.get(case["rule_id"], 0) + 1
        elif not is_anomaly and not predicted_anomaly:
            tn += 1
        elif not is_anomaly and predicted_anomaly:
            fp += 1
        elif is_anomaly and not predicted_anomaly:
            fn += 1

    precision = tp / (tp + fp) if (tp + fp) > 0 else 1.0
    recall = tp / (tp + fn) if (tp + fn) > 0 else 1.0
    fpr = fp / (fp + tn) if (fp + tn) > 0 else 0.0
    f1 = 2 * (precision * recall) / (precision + recall) if (precision + recall) > 0 else 0.0

    report = f"""
================================================================================
          VIETNAMESE WRITING INTELLIGENCE - LINGUISTIC BENCHMARK REPORT
================================================================================
 Tổng số ca kiểm chuẩn : {total}
 True Positives  (TP)   : {tp}
 True Negatives  (TN)   : {tn}
 False Positives (FP)   : {fp}
 False Negatives (FN)   : {fn}
--------------------------------------------------------------------------------
 Precision              : {precision * 100:.2f}%
 Recall (Độ phủ)        : {recall * 100:.2f}%
 False Positive Rate    : {fpr * 100:.2f}%
 F1-Score               : {f1 * 100:.2f}%
 Số lượng Rules phủ     : {len(rule_coverage)} rules
================================================================================
"""
    with capsys.disabled():
        try:
            import sys
            sys.stdout.reconfigure(encoding="utf-8")
            print(report)
        except Exception:
            try:
                import sys
                sys.stdout.buffer.write(report.encode("utf-8") + b"\n")
            except Exception:
                pass

    assert precision == 1.0, f"Precision chưa đạt 100% (hiện tại: {precision:.2%})"
    assert recall == 1.0, f"Recall chưa đạt 100% (hiện tại: {recall:.2%})"
    assert fpr == 0.0, f"FPR chưa đạt 0% (hiện tại: {fpr:.2%})"
