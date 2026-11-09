"""Bộ đánh giá định lượng chuẩn học thuật: Span-Level IoU & Anchor Preservation."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .orchestrator import V3Orchestrator
from .resources import get_real_world_benchmark_path


def compute_iou(start1: int, end1: int, start2: int, end2: int) -> float:
    """Tính Intersection-over-Union (IoU) giữa hai khoảng ký tự."""
    intersection = max(0, min(end1, end2) - max(start1, start2))
    union = max(end1, end2) - min(start1, start2)
    return intersection / union if union > 0 else 0.0


def evaluate_case_spans(
    predicted_findings: list[dict[str, Any]],
    gold_spans: list[dict[str, Any]],
    iou_threshold: float = 0.5,
) -> tuple[int, int, int]:
    """So khớp danh sách span dự đoán với nhãn vàng của con người theo ngưỡng IoU.

    Trả về: (True Positives, False Positives, False Negatives)
    """
    tp = 0
    fp = 0
    matched_gold_indices: set[int] = set()

    for pred in predicted_findings:
        p_start = pred.get("start", -1)
        p_end = pred.get("end", -1)
        p_rule = pred.get("pattern_id")

        best_match_idx = -1
        best_iou = 0.0

        for idx, gold in enumerate(gold_spans):
            if idx in matched_gold_indices:
                continue
            if gold["rule_id"] != p_rule:
                continue

            iou = compute_iou(p_start, p_end, gold["start"], gold["end"])
            if iou >= iou_threshold and iou > best_iou:
                best_iou = iou
                best_match_idx = idx

        if best_match_idx != -1:
            tp += 1
            matched_gold_indices.add(best_match_idx)
        else:
            fp += 1

    fn = len(gold_spans) - len(matched_gold_indices)
    return tp, fp, fn


def evaluate_real_world_corpus(dataset_path: Path | None = None) -> dict[str, Any]:
    """Đánh giá định lượng Span-Level & Anchor Retention trên ngữ liệu thực tế."""
    path = dataset_path or get_real_world_benchmark_path()
    if not path.exists():
        raise FileNotFoundError(f"Không tìm thấy file dữ liệu tại {path}")

    with open(path, encoding="utf-8") as f:
        cases: list[dict[str, Any]] = [json.loads(line) for line in f if line.strip()]

    orch = V3Orchestrator(
        {
            "profile_id": "benchmark_eval",
            "version": "1.0.0",
            "syntactic_signature": {
                "clause_chaining_style": "hypotactic_causal",
                "allow_long_sentences": True,
            },
            "lexical_preferences": {
                "sino_vietnamese_density": "balanced",
                "colloquial_particles_allowed": True,
                "english_term_retention": "strict_preserve_technical",
            },
        },
        validate=False,
    )

    total_tp = 0
    total_fp = 0
    total_fn = 0

    clean_human_total = 0
    clean_human_falsely_flagged = 0

    total_anchors = 0
    preserved_anchors = 0

    fp_details: list[dict[str, Any]] = []
    anchor_losses: list[dict[str, Any]] = []

    for case in cases:
        input_text = case.get("input_text") or case.get("text", "")
        anchors = case.get("anchors_to_preserve", [])
        manifest = {
            "discourse_spec": {
                "genre": "general_prose",
                "register": "professional_sharp",
                "target_audience": "general_public",
            },
            "immutable_anchors": {
                "metrics": anchors,
                "domain_terms": [],
            },
        }

        res = orch.process(input_text, manifest, validate=False)
        detected = res.get("detected_patterns", [])
        gold_spans = case.get("gold_spans", [])

        # 1. Đo lường trên câu chuẩn mực của người thật (Negative Controls)
        source_type = case.get("source_type", "")
        is_human_clean = source_type in (
            "human_journalism",
            "human_classical_masterpiece",
        ) or (not source_type and case.get("expected_action") == "no_change")

        if is_human_clean:
            clean_human_total += 1
            if detected:
                clean_human_falsely_flagged += 1
                fp_details.append(
                    {
                        "id": case["id"],
                        "source_type": source_type or case.get("source", ""),
                        "input_text": input_text,
                        "detected": [d["pattern_id"] for d in detected],
                    }
                )

        # 2. Đo lường Span Precision / Recall
        tp, fp, fn = evaluate_case_spans(detected, gold_spans, iou_threshold=0.5)
        total_tp += tp
        total_fp += fp
        total_fn += fn

        # 3. Đo lường bảo toàn mỏ neo (Anchor Retention)
        out_text = res.get("output_text", "")
        for anchor in anchors:
            total_anchors += 1
            if anchor in out_text:
                preserved_anchors += 1
            else:
                anchor_losses.append({"id": case["id"], "anchor": anchor})

    precision = total_tp / (total_tp + total_fp) if (total_tp + total_fp) > 0 else 1.0
    recall = total_tp / (total_tp + total_fn) if (total_tp + total_fn) > 0 else 1.0
    f1 = (2 * precision * recall) / (precision + recall) if (precision + recall) > 0 else 0.0
    human_fpr = (clean_human_falsely_flagged / clean_human_total) if clean_human_total > 0 else 0.0
    anchor_retention = (preserved_anchors / total_anchors) if total_anchors > 0 else 1.0

    return {
        "benchmark_type": "real_world_span_level",
        "total_cases": len(cases),
        "flawed_samples": len(cases) - clean_human_total,
        "clean_samples": clean_human_total,
        "clean_flagged_wrongly": clean_human_falsely_flagged,
        "metrics": {
            "true_positives": total_tp,
            "false_positives": total_fp,
            "false_negatives": total_fn,
            "precision": precision,
            "recall": recall,
            "f1_score": f1,
            "fpr": human_fpr,
            "anchor_retention": anchor_retention,
            "total_anchors": total_anchors,
            "preserved_anchors": preserved_anchors,
        },
        "fp_details": fp_details,
        "anchor_losses": anchor_losses,
    }
