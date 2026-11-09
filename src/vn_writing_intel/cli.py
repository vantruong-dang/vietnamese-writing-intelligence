"""Giao diện dòng lệnh CLI chuẩn hóa cho Vietnamese Writing Intelligence."""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any

from .drafting import (
    AnthropicProvider,
    MockLLMProvider,
    NativeDrafter,
    OllamaProvider,
    OpenAIProvider,
)
from .orchestrator import V3Orchestrator
from .real_world_eval import evaluate_real_world_corpus
from .resources import get_gold_standard_path
from .validators import SchemaValidationError

# ==============================================================================
# Hỗ trợ Mã màu ANSI và Tương thích Terminal (NO_COLOR, isatty)
# ==============================================================================


class TerminalColor:
    """Điều phối màu sắc terminal tôn trọng tiêu chuẩn NO_COLOR và TTY."""

    def __init__(self, force_disable: bool = False) -> None:
        no_color_env = bool(os.environ.get("NO_COLOR"))
        is_tty = hasattr(sys.stdout, "isatty") and sys.stdout.isatty()
        self.enabled = (not no_color_env) and is_tty and (not force_disable)

    def red(self, text: str) -> str:
        return f"\033[91m{text}\033[0m" if self.enabled else text

    def green(self, text: str) -> str:
        return f"\033[92m{text}\033[0m" if self.enabled else text

    def yellow(self, text: str) -> str:
        return f"\033[93m{text}\033[0m" if self.enabled else text

    def cyan(self, text: str) -> str:
        return f"\033[96m{text}\033[0m" if self.enabled else text

    def bold(self, text: str) -> str:
        return f"\033[1m{text}\033[0m" if self.enabled else text


# ==============================================================================
# Cấu hình Mặc định cho chế độ Lint nhanh
# ==============================================================================

DEFAULT_LINT_PROFILE: dict[str, Any] = {
    "profile_id": "cli_default_profile",
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

DEFAULT_LINT_MANIFEST: dict[str, Any] = {
    "manifest_id": "cli_default_manifest",
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

# ==============================================================================
# Xử lý các Lệnh con (Subcommands)
# ==============================================================================


def run_lint(args: argparse.Namespace, color: TerminalColor) -> int:
    """Thực thi quét nhanh (lint) một file văn bản với cấu hình mặc định."""
    target_path = Path(args.file)
    if not target_path.exists():
        sys.stderr.write(f"Lỗi: Không tìm thấy file '{args.file}'\n")
        return 2

    try:
        with open(target_path, encoding="utf-8") as f:
            text = f.read()
    except OSError as e:
        sys.stderr.write(f"Lỗi đọc file: {e}\n")
        return 2

    orch = V3Orchestrator(DEFAULT_LINT_PROFILE, validate=False)
    result = orch.process(text, DEFAULT_LINT_MANIFEST, validate=False)

    if args.json:
        print(json.dumps(result, ensure_ascii=False, indent=2))
    else:
        action = result["action_mode"]
        patterns = result["detected_patterns"]

        print(color.bold(f"==> Kết quả quét lint: {args.file}"))
        print(
            f"Trạng thái (Action Mode): {color.green(action) if action == 'no_change' else color.yellow(action)}"
        )
        print(f"Tổng số phát hiện: {len(patterns)}")

        if patterns:
            print("\nChi tiết các phát hiện:")
            for p in patterns:
                sev = p.get("severity", "minor")
                sev_colored = (
                    color.red(sev)
                    if sev == "critical"
                    else color.yellow(sev)
                    if sev == "major"
                    else sev
                )
                pid = p.get("pattern_id", "UNKNOWN")
                span = p.get("span", "")
                rationale = p.get("rationale", "")
                print(f'  • [{sev_colored}] {color.bold(pid)}: "{span}"')
                print(f"    └─ {rationale}")
        else:
            print(color.green("✓ Văn bản sạch, không phát hiện vi phạm phong cách hay cú pháp."))

    return 0 if result["action_mode"] in {"no_change", "clean_rewrite"} else 1


def run_audit(args: argparse.Namespace, color: TerminalColor) -> int:
    """Thực thi kiểm toán chuyên sâu bảo toàn dữ kiện theo Manifest và Voice Profile."""
    target_path = Path(args.text_file)
    profile_path = Path(args.profile)
    manifest_path = Path(args.manifest)

    for p, name in [
        (target_path, "văn bản"),
        (profile_path, "profile"),
        (manifest_path, "manifest"),
    ]:
        if not p.exists():
            sys.stderr.write(f"Lỗi: Không tìm thấy file {name} tại '{p}'\n")
            return 2

    try:
        with open(target_path, encoding="utf-8") as f:
            text = f.read()
        with open(profile_path, encoding="utf-8") as f:
            profile = json.load(f)
        with open(manifest_path, encoding="utf-8") as f:
            manifest = json.load(f)
    except (OSError, json.JSONDecodeError) as e:
        sys.stderr.write(f"Lỗi nạp cấu hình JSON: {e}\n")
        return 2

    try:
        orch = V3Orchestrator(profile, validate=True)
        result = orch.process(text, manifest, validate=True)
    except (SchemaValidationError, ValueError, KeyError) as e:
        sys.stderr.write(f"Lỗi kiểm định: {e}\n")
        return 2

    if args.json:
        print(json.dumps(result, ensure_ascii=False, indent=2))
    else:
        audit = result["semantic_audit"]
        action = result["action_mode"]
        facts_ok = audit["facts_preserved"]
        patterns = result["detected_patterns"]

        print(color.bold(f"==> Kết quả Kiểm toán Dữ kiện: {args.text_file}"))
        print(
            f"Hành động khuyến nghị: {color.green(action) if action == 'no_change' else color.red(action) if action == 'needs_author_decision' else color.yellow(action)}"
        )
        print(
            f"Bảo toàn số liệu & thực thể: {color.green('100% Nguyên vẹn') if facts_ok else color.red('Phát hiện mất mát / sai lệch')}"
        )

        if audit.get("drift_warnings"):
            print(color.red("\nCảnh báo sai lệch dữ kiện (Semantic Drift):"))
            for w in audit["drift_warnings"]:
                print(f"  ⚠ {w}")

        if patterns:
            print(f"\nPhát hiện ngôn ngữ & phong cách ({len(patterns)}):")
            for p in patterns:
                pid = p.get("pattern_id", "UNKNOWN")
                span = p.get("span", "")
                rationale = p.get("rationale", "")
                print(f'  • {color.bold(pid)}: "{span}" -> {rationale}')

        if result.get("output_text") != text and action == "clean_rewrite":
            print(color.green("\nVăn bản sau khi làm sạch:"))
            print(result["output_text"])

    return 0 if result["action_mode"] in {"no_change", "clean_rewrite"} else 1


def run_draft(args: argparse.Namespace, color: TerminalColor) -> int:
    """Thực thi sinh bản thảo (Native Drafter) từ DraftManifest."""
    manifest_path = Path(args.manifest)
    if not manifest_path.exists():
        sys.stderr.write(f"Lỗi: Không tìm thấy manifest tại '{manifest_path}'\n")
        return 2

    try:
        with open(manifest_path, encoding="utf-8") as f:
            manifest = json.load(f)
    except (OSError, json.JSONDecodeError) as e:
        sys.stderr.write(f"Lỗi nạp manifest: {e}\n")
        return 2

    profile = DEFAULT_LINT_PROFILE
    if args.profile:
        profile_path = Path(args.profile)
        if not profile_path.exists():
            sys.stderr.write(f"Lỗi: Không tìm thấy profile tại '{profile_path}'\n")
            return 2
        with open(profile_path, encoding="utf-8") as f:
            profile = json.load(f)

    # Khởi tạo LLM Provider tương ứng
    provider_type = args.provider.lower()
    try:
        match provider_type:
            case "mock":
                provider = MockLLMProvider()
            case "ollama":
                provider = OllamaProvider(model=args.model or "llama3:latest")
            case "anthropic":
                provider = AnthropicProvider(model=args.model or "claude-3-5-sonnet-20241022")
            case "openai":
                provider = OpenAIProvider(model=args.model or "gpt-4o")
            case _:
                sys.stderr.write(
                    f"Lỗi: Provider '{args.provider}' không được hỗ trợ. Chọn mock, ollama, anthropic, openai.\n"
                )
                return 2
    except Exception as e:
        sys.stderr.write(f"Lỗi khởi tạo LLM Provider: {e}\n")
        return 2

    drafter = NativeDrafter(provider=provider)
    try:
        draft_res = drafter.draft(
            manifest=manifest,
            voice_profile=profile,
            instructions=args.instructions or "",
            auto_verify=True,
        )
    except Exception as e:
        sys.stderr.write(f"Lỗi trong quá trình sinh bản thảo: {e}\n")
        return 1

    if args.output:
        try:
            with open(args.output, "w", encoding="utf-8") as f:
                f.write(draft_res["draft_text"])
            if not args.json:
                print(color.green(f"Đã lưu bản thảo vào file: {args.output}"))
        except OSError as e:
            sys.stderr.write(f"Lỗi ghi file output: {e}\n")
            return 2

    if args.json:
        # Chuẩn hóa đối tượng để serialize JSON
        gen_data = draft_res["generation"]
        serializable = {
            "draft_text": draft_res["draft_text"],
            "status": draft_res["status"],
            "provider": gen_data.provider,
            "model": gen_data.model,
            "usage": gen_data.usage,
            "verification": draft_res["verification"],
        }
        print(json.dumps(serializable, ensure_ascii=False, indent=2))
    else:
        print(color.bold("==> Bản thảo sinh bởi Native Drafter:"))
        print(draft_res["draft_text"])
        print(
            "\nTrạng thái kiểm toán tự động: "
            + (
                color.green(draft_res["status"])
                if draft_res["status"] == "no_change"
                else color.yellow(draft_res["status"])
            )
        )

    return 0 if draft_res["status"] in {"no_change", "clean_rewrite", "unverified"} else 1


def run_benchmark(args: argparse.Namespace, color: TerminalColor) -> int:
    """Thực thi kiểm chuẩn định lượng và báo cáo tiến độ hạn ngạch (Quota Progress)."""
    if getattr(args, "real", False):
        return run_real_world_benchmark(args, color)

    dataset_path = get_gold_standard_path()
    cases: list[dict[str, Any]] = []

    with open(dataset_path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                cases.append(json.loads(line))

    orch = V3Orchestrator(DEFAULT_LINT_PROFILE, validate=False)

    total = len(cases)
    tp = 0
    fp = 0
    tn = 0
    fn = 0
    rule_coverage: dict[str, int] = {}

    # Phân nhóm tiến độ 4 cụm theo cam kết KNOWLEDGE_ARCHITECTURE_MAP.md (mục tiêu 200 câu = 4 nhóm x 50 câu)
    category_quotas: dict[str, dict[str, Any]] = {
        "group_1_nominalization": {
            "name": "Nhóm 1: Danh từ hóa 3 tầng (GS. Cao Xuân Hạo)",
            "categories": {"nominalization"},
            "target": 50,
            "current": 0,
        },
        "group_2_topic_comment": {
            "name": "Nhóm 2: Cấu trúc Đề - Thuyết & Cú pháp bản địa (GS. Cao Xuân Hạo)",
            "categories": {"topic_comment", "syntax_reporting", "syntax_parallel", "syntax_cliche"},
            "target": 50,
            "current": 0,
        },
        "group_3_concessive_temporal_pleonasm": {
            "name": "Nhóm 3: Hiện thực/Giả định (Tuy - Dù) & Trùng ngữ (Humanizer)",
            "categories": {
                "concessive_aspect",
                "temporal_framing",
                "aspect_mechanics",
                "pleonasm",
                "humanizer",
                "lexical_misuse",
                "pragmatics_negation",
            },
            "target": 50,
            "current": 0,
        },
        "group_4_anchors_classical": {
            "name": "Nhóm 4: Bảo toàn Dữ kiện & Tiết tấu Đối ngẫu Cổ văn (Nguyễn Hiến Lê)",
            "categories": {
                "semantic_anchor",
                "protected_block",
                "classical_rhythm",
                "pseudo_passive",
                "translationese",
            },
            "target": 50,
            "current": 0,
        },
    }

    for case in cases:
        cat = case.get("category", "")
        for g in category_quotas.values():
            if cat in g["categories"]:
                g["current"] += 1
                break

        manifest = case.get("manifest") or DEFAULT_LINT_MANIFEST
        result = orch.process(case["input_text"], manifest, validate=False)

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

    if args.json:
        serializable_quotas = {
            k: {**v, "categories": sorted(v["categories"])} for k, v in category_quotas.items()
        }
        out = {
            "total_cases": total,
            "target_total": 200,
            "progress_percent": round(total / 200 * 100, 2),
            "metrics": {
                "precision": precision,
                "recall": recall,
                "fpr": fpr,
                "f1_score": f1,
                "true_positives": tp,
                "true_negatives": tn,
                "false_positives": fp,
                "false_negatives": fn,
                "rules_covered": len(rule_coverage),
            },
            "category_quotas": serializable_quotas,
        }
        print(json.dumps(out, ensure_ascii=False, indent=2))
    else:
        print(color.bold("=" * 80))
        print(color.bold("       VIETNAMESE WRITING INTELLIGENCE - LINGUISTIC BENCHMARK REPORT"))
        print(color.bold("=" * 80))
        print(
            f" Tổng số ca hiện có       : {color.bold(str(total))} / 200 ({total / 200 * 100:.1f}% tiến độ mục tiêu Spec v0.2.0)"
        )
        print(f" True Positives (TP)      : {color.green(str(tp))}")
        print(f" True Negatives (TN)      : {color.green(str(tn))}")
        print(
            f" False Positives (FP)     : {color.green(str(fp)) if fp == 0 else color.red(str(fp))}"
        )
        print(
            f" False Negatives (FN)     : {color.green(str(fn)) if fn == 0 else color.red(str(fn))}"
        )
        print("-" * 80)
        print(f" Precision (Độ chính xác) : {color.green(f'{precision * 100:.2f}%')}")
        print(f" Recall (Độ phủ phát hiện): {color.green(f'{recall * 100:.2f}%')}")
        print(f" False Positive Rate (FPR): {color.green(f'{fpr * 100:.2f}%')}")
        print(f" F1-Score                 : {color.green(f'{f1 * 100:.2f}%')}")
        print(f" Số lượng Rules được phủ  : {color.bold(str(len(rule_coverage)))} rules")
        print("=" * 80)
        print(color.bold("\nTIẾN ĐỘ ĐỘ PHỦ THEO HẠN NGẠCH CÁC NHÓM (QUOTA PROGRESS):"))
        for g in category_quotas.values():
            curr = g["current"]
            tgt = g["target"]
            pct = curr / tgt * 100
            bar = "█" * int(pct // 5) + "░" * (20 - int(pct // 5))
            print(f" • {g['name']}")
            print(f"   [{bar}] {curr}/{tgt} câu ({pct:.1f}%)")

    return 0 if (precision == 1.0 and recall == 1.0 and fpr == 0.0) else 1


def run_real_world_benchmark(args: argparse.Namespace, color: TerminalColor) -> int:
    """Thực thi đánh giá định lượng Span-Level & Anchor Retention trên ngữ liệu thực tế."""
    try:
        report = evaluate_real_world_corpus()
    except FileNotFoundError as e:
        sys.stderr.write(f"Lỗi: {e}\n")
        return 2

    if args.json:
        print(json.dumps(report, ensure_ascii=False, indent=2))
        return 0

    m = report["metrics"]
    print(color.bold("=" * 80))
    print(color.bold("      REAL-WORLD BENCHMARK REPORT (SPAN-LEVEL & ANCHOR EVALUATION)"))
    print(color.bold("=" * 80))

    for item in report["fp_details"]:
        print(f"[FALSE POSITIVE ON HUMAN PROSE] Ca {item['id']}:")
        print(f'  Văn bản : "{item["input_text"]}"')
        print(f"  Bắt nhầm: {item['detected']}\n")

    for loss in report["anchor_losses"]:
        print(f"[ANCHOR LOSS] Ca {loss['id']}: Mất mỏ neo '{loss['anchor']}'")

    print("-" * 80)
    print(f"Tổng số mẫu thực tế đánh giá     : {color.bold(str(report['total_cases']))}")
    print(f"  • Mẫu lỗi thật (MT / Raw LLM)  : {report['flawed_samples']}")
    print(f"  • Mẫu chuẩn mực (Báo chí/Cổ văn): {report['clean_samples']}")
    print("-" * 80)
    print(f"Span True Positives (Bắt trúng)  : {color.green(str(m['true_positives']))}")
    print(f"Span False Positives (Bắt sai/lệch): {color.yellow(str(m['false_positives']))}")
    print(f"Span False Negatives (Bỏ lọt)    : {color.yellow(str(m['false_negatives']))}")
    print("=" * 80)
    print(color.bold("🎯 KẾT QUẢ ĐO LƯỜNG ĐỊNH LƯỢNG:"))
    prec = f"{m['precision']:.2%}"
    rec = f"{m['recall']:.2%}"
    f1_str = f"{m['f1_score']:.2%}"
    fpr_str = f"{m['fpr']:.2%}"
    anch_str = f"{m['anchor_retention']:.2%}"
    print(f"   Span Precision (Độ chính xác) : {color.green(prec)}")
    print(f"   Span Recall (Độ bao phủ)      : {color.green(rec)}")
    print(f"   Span F1-Score                 : {color.green(f1_str)}")
    print(f"   Human Text False Alarm (FPR)  : {color.green(fpr_str)} (Mục tiêu: < 5%)")
    print(f"   Anchor Retention (Bảo toàn)   : {color.green(anch_str)} (Mục tiêu: 100%)")
    print("=" * 80)
    return 0


def run_mcp_command(args: argparse.Namespace) -> int:
    """Khởi động Model Context Protocol (MCP) server qua stdio."""
    try:
        from .mcp_server import run_mcp_cli

        run_mcp_cli()
        return 0
    except ImportError as e:
        sys.stderr.write(f"{e}\n")
        return 2
    except Exception as e:
        sys.stderr.write(f"Lỗi khởi động MCP Server: {e}\n")
        return 1


# ==============================================================================
# Điểm Nhập Trung Tâm (CLI Entrypoint)
# ==============================================================================


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="vn-intel",
        description="Vietnamese Writing Intelligence CLI - Công cụ phân tích và viết văn tiếng Việt bản địa.",
    )
    parser.add_argument("--no-color", action="store_true", help="Vô hiệu hóa toàn bộ mã màu ANSI")
    subparsers = parser.add_subparsers(dest="command", required=True)

    # Subcommand: lint
    p_lint = subparsers.add_parser("lint", help="Quét nhanh văn bản với cấu hình mặc định")
    p_lint.add_argument("file", help="File văn bản (.txt / .md)")
    p_lint.add_argument("--json", action="store_true", help="Xuất kết quả định dạng JSON thuần")

    # Subcommand: audit
    p_audit = subparsers.add_parser(
        "audit", help="Kiểm toán chuyên sâu với Voice Profile và Draft Manifest"
    )
    p_audit.add_argument("text_file", help="File văn bản đầu vào (.txt / .md)")
    p_audit.add_argument("--profile", required=True, help="Đường dẫn file voice-profile.json")
    p_audit.add_argument("--manifest", required=True, help="Đường dẫn file draft-manifest.json")
    p_audit.add_argument("--json", action="store_true", help="Xuất kết quả định dạng JSON thuần")

    # Subcommand: draft
    p_draft = subparsers.add_parser(
        "draft", help="Phác thảo văn bản từ DraftManifest qua LLM Provider"
    )
    p_draft.add_argument("--manifest", required=True, help="Đường dẫn file draft-manifest.json")
    p_draft.add_argument("--profile", help="Đường dẫn file voice-profile.json (tùy chọn)")
    p_draft.add_argument(
        "--provider",
        default="mock",
        help="Nhà cung cấp LLM: mock (mặc định), ollama, anthropic, openai",
    )
    p_draft.add_argument(
        "--model", help="Tên model (ví dụ: llama3, claude-3-5-sonnet-20241022, gpt-4o)"
    )
    p_draft.add_argument("--instructions", help="Yêu cầu bổ sung cho bài viết")
    p_draft.add_argument("--output", "-o", help="Đường dẫn lưu file kết quả")
    p_draft.add_argument("--json", action="store_true", help="Xuất kết quả định dạng JSON thuần")

    # Subcommand: benchmark
    p_bench = subparsers.add_parser(
        "benchmark", help="Chạy kiểm chuẩn định lượng và báo cáo tiến độ hạn ngạch"
    )
    p_bench.add_argument("--json", action="store_true", help="Xuất báo cáo định dạng JSON thuần")
    p_bench.add_argument(
        "--real",
        action="store_true",
        help="Chạy kiểm nghiệm trên tập văn bản thực tế (Real-World Corpus)",
    )

    # Subcommand: mcp
    subparsers.add_parser("mcp", help="Khởi động Model Context Protocol (MCP) server qua stdio")

    return parser


def main() -> None:
    # 1. Ép mã hóa chuẩn UTF-8 cho stdout và stderr trên Windows
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(encoding="utf-8")

    parser = build_parser()
    args = parser.parse_args()

    color = TerminalColor(force_disable=args.no_color or getattr(args, "json", False))

    match args.command:
        case "lint":
            sys.exit(run_lint(args, color))
        case "audit":
            sys.exit(run_audit(args, color))
        case "draft":
            sys.exit(run_draft(args, color))
        case "benchmark":
            sys.exit(run_benchmark(args, color))
        case "mcp":
            sys.exit(run_mcp_command(args))
        case _:
            parser.print_help()
            sys.exit(2)


if __name__ == "__main__":
    main()
