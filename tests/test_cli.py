"""Kiểm thử đơn vị cho giao diện dòng lệnh CLI vn-intel (Subcommands & Exit Codes)."""
from __future__ import annotations

import json
from pathlib import Path
import pytest

from vn_writing_intel.cli import (
    DEFAULT_LINT_PROFILE,
    TerminalColor,
    build_parser,
    run_audit,
    run_benchmark,
    run_draft,
    run_lint,
)


@pytest.fixture
def temp_workspace(tmp_path: Path):
    """Tạo các file cấu hình và văn bản mẫu cho kiểm thử CLI."""
    clean_file = tmp_path / "clean.txt"
    clean_file.write_text("Tự học là con đường gian khổ nhưng mở ra tri thức thực sự.", encoding="utf-8")

    dirty_file = tmp_path / "dirty.txt"
    dirty_file.write_text(
        "Đây là giải pháp kỹ thuật tối ưu nhất cho hạ tầng mạng.",
        encoding="utf-8",
    )

    profile_file = tmp_path / "profile.json"
    profile_file.write_text(json.dumps(DEFAULT_LINT_PROFILE, ensure_ascii=False), encoding="utf-8")

    manifest_file = tmp_path / "manifest.json"
    manifest_data = {
        "manifest_id": "test_manifest",
        "schema_version": "0.2.0",
        "discourse_spec": {
            "genre": "technical_architecture",
            "register": "professional_sharp",
            "dialect_bias": "all_neutral",
            "target_audience": "domain_engineer",
        },
        "topic_comment_plan": [
            {
                "section_id": "sec_1",
                "topic": "Hạ tầng phân tán",
                "intended_comment": "Phân tích 100 Gbps và FFI",
            }
        ],
        "immutable_anchors": {
            "metrics": ["100 Gbps"],
            "domain_terms": ["FFI"],
            "named_entities": [],
            "causal_invariants": [],
        },
    }
    manifest_file.write_text(json.dumps(manifest_data, ensure_ascii=False), encoding="utf-8")

    anchored_text_file = tmp_path / "anchored.txt"
    anchored_text_file.write_text("Hạ tầng phân tán đạt tốc độ 100 Gbps qua ranh giới FFI an toàn.", encoding="utf-8")

    missing_anchor_file = tmp_path / "missing_anchor.txt"
    missing_anchor_file.write_text("Hạ tầng phân tán đạt tốc độ rất cao qua giao diện an toàn.", encoding="utf-8")

    return {
        "clean_file": clean_file,
        "dirty_file": dirty_file,
        "profile_file": profile_file,
        "manifest_file": manifest_file,
        "anchored_text_file": anchored_text_file,
        "missing_anchor_file": missing_anchor_file,
        "tmp_path": tmp_path,
    }


def test_terminal_color_respects_no_color(monkeypatch: pytest.MonkeyPatch):
    monkeypatch.setenv("NO_COLOR", "1")
    tc = TerminalColor()
    assert tc.enabled is False
    assert tc.red("Error") == "Error"

    monkeypatch.delenv("NO_COLOR", raising=False)
    tc_forced = TerminalColor(force_disable=True)
    assert tc_forced.enabled is False


def test_cli_parser_registered_commands():
    parser = build_parser()
    assert parser.parse_args(["lint", "clean.txt"]).command == "lint"
    assert parser.parse_args(["audit", "t.txt", "--profile", "p.json", "--manifest", "m.json"]).command == "audit"
    assert parser.parse_args(["draft", "--manifest", "m.json"]).command == "draft"
    assert parser.parse_args(["benchmark"]).command == "benchmark"
    assert parser.parse_args(["mcp"]).command == "mcp"


def test_cli_lint_clean_file(temp_workspace, capsys: pytest.CaptureFixture[str]):
    parser = build_parser()
    args = parser.parse_args(["lint", str(temp_workspace["clean_file"])])
    color = TerminalColor(force_disable=True)

    exit_code = run_lint(args, color)
    assert exit_code == 0
    captured = capsys.readouterr()
    assert "Văn bản sạch" in captured.out


def test_cli_lint_with_findings(temp_workspace, capsys: pytest.CaptureFixture[str]):
    parser = build_parser()
    args = parser.parse_args(["lint", str(temp_workspace["dirty_file"])])
    color = TerminalColor(force_disable=True)

    exit_code = run_lint(args, color)
    # dirty_file chứa LOGICAL_PLEONASM (tối ưu nhất) -> review_comment (exit code = 1)
    assert exit_code == 1
    captured = capsys.readouterr()
    assert "LOGICAL_PLEONASM" in captured.out


def test_cli_lint_json_mode(temp_workspace, capsys: pytest.CaptureFixture[str]):
    parser = build_parser()
    args = parser.parse_args(["lint", str(temp_workspace["dirty_file"]), "--json"])
    color = TerminalColor(force_disable=True)

    exit_code = run_lint(args, color)
    assert exit_code in {0, 1}
    captured = capsys.readouterr()
    data = json.loads(captured.out)
    assert "action_mode" in data
    assert "detected_patterns" in data


def test_cli_lint_missing_file(capsys: pytest.CaptureFixture[str]):
    parser = build_parser()
    args = parser.parse_args(["lint", "nonexistent_file_xyz.txt"])
    color = TerminalColor(force_disable=True)

    exit_code = run_lint(args, color)
    assert exit_code == 2
    captured = capsys.readouterr()
    assert "Không tìm thấy file" in captured.err


def test_cli_audit_intact(temp_workspace, capsys: pytest.CaptureFixture[str]):
    parser = build_parser()
    args = parser.parse_args([
        "audit",
        str(temp_workspace["anchored_text_file"]),
        "--profile", str(temp_workspace["profile_file"]),
        "--manifest", str(temp_workspace["manifest_file"]),
    ])
    color = TerminalColor(force_disable=True)

    exit_code = run_audit(args, color)
    assert exit_code == 0
    captured = capsys.readouterr()
    assert "100% Nguyên vẹn" in captured.out


def test_cli_audit_missing_metric_triggers_exit_1(temp_workspace, capsys: pytest.CaptureFixture[str]):
    parser = build_parser()
    args = parser.parse_args([
        "audit",
        str(temp_workspace["missing_anchor_file"]),
        "--profile", str(temp_workspace["profile_file"]),
        "--manifest", str(temp_workspace["manifest_file"]),
    ])
    color = TerminalColor(force_disable=True)

    exit_code = run_audit(args, color)
    assert exit_code == 1
    captured = capsys.readouterr()
    assert "Cảnh báo sai lệch dữ kiện" in captured.out
    assert "100 Gbps" in captured.out


def test_cli_draft_mock_provider(temp_workspace, capsys: pytest.CaptureFixture[str]):
    output_file = temp_workspace["tmp_path"] / "draft_output.txt"
    parser = build_parser()
    args = parser.parse_args([
        "draft",
        "--manifest", str(temp_workspace["manifest_file"]),
        "--provider", "mock",
        "--output", str(output_file),
    ])
    color = TerminalColor(force_disable=True)

    exit_code = run_draft(args, color)
    assert exit_code in {0, 1}
    assert output_file.exists()
    assert len(output_file.read_text(encoding="utf-8")) > 0


def test_cli_benchmark_quota_progress(capsys: pytest.CaptureFixture[str]):
    parser = build_parser()
    args = parser.parse_args(["benchmark", "--json"])
    color = TerminalColor(force_disable=True)

    exit_code = run_benchmark(args, color)
    assert exit_code == 0
    captured = capsys.readouterr()
    bench_data = json.loads(captured.out)

    assert bench_data["total_cases"] == 38
    assert bench_data["target_total"] == 200
    assert bench_data["metrics"]["precision"] == 1.0
    assert bench_data["metrics"]["recall"] == 1.0
    assert "category_quotas" in bench_data
    assert len(bench_data["category_quotas"]) == 4
