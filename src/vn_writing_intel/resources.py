"""Tiện ích nạp tài nguyên tĩnh (Schemas, Gold Benchmark) tuân thủ Zero-Duplication SSOT."""

from __future__ import annotations

import importlib.resources
from pathlib import Path


def get_schema_dir() -> Path:
    """Trả về đường dẫn tới thư mục schemas.

    Ưu tiên tài nguyên đóng gói bên trong wheel qua importlib.resources,
    tự động fallback về thư mục schemas ở root repository khi chạy editable (pip install -e .).
    """
    try:
        bundled = importlib.resources.files("vn_writing_intel") / "schemas"
        p = Path(str(bundled))
        if p.exists() and (p / "voice-profile.schema.json").exists():
            return p
    except Exception:
        pass

    # Fallback về root repo
    repo_root = Path(__file__).resolve().parents[2] / "schemas"
    if repo_root.exists():
        return repo_root

    raise FileNotFoundError("Không thể tìm thấy thư mục schemas ở cả package bundle lẫn repo root.")


def get_gold_standard_path() -> Path:
    """Trả về đường dẫn tới file linguistic_gold_standard.jsonl."""
    try:
        bundled = (
            importlib.resources.files("vn_writing_intel")
            / "data"
            / "linguistic_gold_standard.jsonl"
        )
        p = Path(str(bundled))
        if p.exists():
            return p
    except Exception:
        pass

    # Fallback về tests/data tại root repo
    repo_path = (
        Path(__file__).resolve().parents[2] / "tests" / "data" / "linguistic_gold_standard.jsonl"
    )
    if repo_path.exists():
        return repo_path

    raise FileNotFoundError(
        "Không thể tìm thấy linguistic_gold_standard.jsonl ở cả package bundle lẫn repo root."
    )


def get_real_world_benchmark_path() -> Path:
    """Trả về đường dẫn tới file real_world_benchmark.jsonl."""
    try:
        bundled = (
            importlib.resources.files("vn_writing_intel") / "data" / "real_world_benchmark.jsonl"
        )
        p = Path(str(bundled))
        if p.exists():
            return p
    except Exception:
        pass

    repo_path = (
        Path(__file__).resolve().parents[2] / "tests" / "data" / "real_world_benchmark.jsonl"
    )
    if repo_path.exists():
        return repo_path

    raise FileNotFoundError(
        "Không thể tìm thấy real_world_benchmark.jsonl ở cả package bundle lẫn repo root."
    )
