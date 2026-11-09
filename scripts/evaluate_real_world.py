"""Script kiểm định khách quan trên ngữ liệu thực tế (Real-world Benchmark)."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

from vn_writing_intel.orchestrator import V3Orchestrator
from vn_writing_intel.resources import get_real_world_benchmark_path

DATASET_FILE = Path(__file__).resolve().parents[1] / "tests" / "data" / "real_world_benchmark.jsonl"


def evaluate_real_world_corpus(dataset_path: Path | None = None) -> dict[str, Any]:
    """Đánh giá định lượng trên tập dữ liệu thực tế và trả về dictionary kết quả."""
    path = dataset_path or (
        DATASET_FILE if DATASET_FILE.exists() else get_real_world_benchmark_path()
    )
    if not path.exists():
        raise FileNotFoundError(f"Không tìm thấy file dữ liệu tại {path}")

    with open(path, encoding="utf-8") as f:
        cases = [json.loads(line) for line in f if line.strip()]

    profile: dict[str, Any] = {
        "profile_id": "real_world_evaluator",
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
    }
    manifest: dict[str, Any] = {
        "discourse_spec": {
            "genre": "general_prose",
            "register": "professional_sharp",
            "target_audience": "general_public",
        },
        "immutable_anchors": {"metrics": [], "domain_terms": []},
    }

    orch = V3Orchestrator(profile, validate=False)

    total_expected_violations = 0
    total_detected_violations = 0
    true_positives = 0
    false_positives = 0
    false_negatives = 0

    clean_sentences_total = 0
    clean_sentences_flagged_wrongly = 0
    fp_details: list[dict[str, Any]] = []
    fn_details: list[dict[str, Any]] = []

    for case in cases:
        res = orch.process(case["text"], manifest, validate=False)
        detected_rules = {f["pattern_id"] for f in res["detected_patterns"]}
        expected_rules = set(case.get("expected_violations", []))

        total_expected_violations += len(expected_rules)
        total_detected_violations += len(detected_rules)

        # 1. Đối với câu văn sạch (Người viết chuẩn)
        if case["expected_action"] == "no_change":
            clean_sentences_total += 1
            if detected_rules:
                clean_sentences_flagged_wrongly += 1
                false_positives += len(detected_rules)
                fp_details.append(
                    {
                        "id": case["id"],
                        "source": case["source"],
                        "text": case["text"],
                        "detected": sorted(detected_rules),
                    }
                )
        # 2. Đối với câu có lỗi (Văn bản AI / dịch thô)
        else:
            tp = len(detected_rules.intersection(expected_rules))
            fp = len(detected_rules - expected_rules)
            fn = len(expected_rules - detected_rules)

            true_positives += tp
            false_positives += fp
            false_negatives += fn

            if fn > 0:
                fn_details.append(
                    {
                        "id": case["id"],
                        "source": case["source"],
                        "text": case["text"],
                        "missed": sorted(expected_rules - detected_rules),
                    }
                )

    precision = (
        (true_positives / (true_positives + false_positives))
        if (true_positives + false_positives) > 0
        else 1.0
    )
    recall = (
        (true_positives / (true_positives + false_negatives))
        if (true_positives + false_negatives) > 0
        else 1.0
    )
    f1 = (2 * precision * recall / (precision + recall)) if (precision + recall) > 0 else 0.0
    fpr = (
        (clean_sentences_flagged_wrongly / clean_sentences_total)
        if clean_sentences_total > 0
        else 0.0
    )

    return {
        "total_cases": len(cases),
        "flawed_samples": len(cases) - clean_sentences_total,
        "clean_samples": clean_sentences_total,
        "clean_flagged_wrongly": clean_sentences_flagged_wrongly,
        "metrics": {
            "true_positives": true_positives,
            "false_positives": false_positives,
            "false_negatives": false_negatives,
            "precision": precision,
            "recall": recall,
            "f1_score": f1,
            "fpr": fpr,
        },
        "fp_details": fp_details,
        "fn_details": fn_details,
    }


def run_real_world_evaluation() -> int:
    """Chạy đánh giá và in báo cáo ra màn hình console."""
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")

    try:
        report = evaluate_real_world_corpus()
    except FileNotFoundError as e:
        print(f"Lỗi: {e}")
        return 2

    m = report["metrics"]
    print("=" * 80)
    print("      ĐÁNH GIÁ ĐỊNH LƯỢNG TRÊN NGỮ LIỆU THỰC TẾ (REAL-WORLD BENCHMARK)")
    print("=" * 80)

    for item in report["fp_details"]:
        print(f"[BẮT OAN - FP] Ca: {item['id']} ({item['source']})")
        print(f'  Văn bản : "{item["text"]}"')
        print(f"  Bắt nhầm: {item['detected']}\n")

    for item in report["fn_details"]:
        print(f"[BỎ SÓT - FN] Ca: {item['id']} ({item['source']})")
        print(f'  Văn bản: "{item["text"]}"')
        print(f"  Bỏ lọt : {item['missed']}\n")

    print("-" * 80)
    print(f"Tổng số mẫu thực tế kiểm nghiệm: {report['total_cases']}")
    print(f"  • Mẫu lỗi (AI / Dịch máy)      : {report['flawed_samples']}")
    print(f"  • Mẫu chuẩn mực (Người thật)   : {report['clean_samples']}")
    print("-" * 80)
    print(f"True Positives  (Bắt đúng lỗi)  : {m['true_positives']}")
    print(
        f"False Positives (Bắt sai / Oan) : {m['false_positives']} "
        f"(trong đó {report['clean_flagged_wrongly']} câu người thật bị bắt nhầm)"
    )
    print(f"False Negatives (Bỏ lọt lỗi)    : {m['false_negatives']}")
    print("=" * 80)
    print("🎯 KẾT QUẢ THỰC TẾ:")
    print(f"   Precision (Độ chuẩn xác)      : {m['precision']:.2%}")
    print(f"   Recall (Độ bao phủ)           : {m['recall']:.2%}")
    print(f"   F1-Score                      : {m['f1_score']:.2%}")
    print(f"   False Positive Rate (Bắt oan) : {m['fpr']:.2%}")
    print("=" * 80)
    return 0


if __name__ == "__main__":
    sys.exit(run_real_world_evaluation())
