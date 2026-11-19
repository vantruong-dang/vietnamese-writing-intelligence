import json
from itertools import pairwise
from pathlib import Path

from neural.data.generate_errors import align_segment, generate_dataset


def _write_lexicon(path: Path) -> None:
    words = [
        "xử lý",
        "sử lý",
        "bổ sung",
        "bổ xung",
        "chỉn chu",
        "chỉnh chu",
        "chẩn đoán",
        "chuẩn đoán",
        "tham quan",
        "thăm quan",
        "sáp nhập",
        "sát nhập",
        "sửa lỗi",
        "dữ liệu",
        "báo cáo",
        "hồ sơ",
        "bệnh nhân",
    ]
    path.write_text("\n".join(words) + "\n", encoding="utf-8")


def test_generate_dataset_schema_and_determinism(tmp_path: Path) -> None:
    lexicon = tmp_path / "lexicon.txt"
    _write_lexicon(lexicon)
    corpus = tmp_path / "corpus.txt"
    corpus.write_text(
        "\n".join(
            [
                "Chúng ta cần xử lý hồ sơ sáp nhập.",
                "Báo cáo cần chỉn chu và bổ sung dữ liệu.",
                "Bác sĩ chẩn đoán bệnh nhân.",
                "Cuối tuần đi tham quan.",
            ]
        )
        + "\n",
        encoding="utf-8",
    )

    stats = generate_dataset(
        tmp_path / "out",
        lexicon,
        [corpus],
        train_size=60,
        val_size=10,
        test_size=10,
        clean_ratio=0.2,
        seed=7,
    )
    assert stats["train"]["records"] > 0
    assert stats["train"]["operators"]

    for split in ("train", "val", "test"):
        records = [
            json.loads(line)
            for line in (tmp_path / "out" / f"{split}.jsonl").read_text(encoding="utf-8").splitlines()
        ]
        assert records
        for record in records:
            text_bytes = record["text"].encode("utf-8")
            assert len(record["byte_targets"]) == len(text_bytes)
            assert len(record["byte_mask"]) == len(text_bytes)
            assert all(0 <= value <= 255 for value in record["byte_targets"])
            spans = record["spans"]
            for span in spans:
                assert 1 <= span["label"] <= 3
                assert 0 <= span["start"] < span["end"] <= len(text_bytes)
            for left, right in pairwise(spans):
                assert left["end"] <= right["start"]

    # Cùng seed -> cùng dữ liệu.
    generate_dataset(
        tmp_path / "out2",
        lexicon,
        [corpus],
        train_size=60,
        val_size=10,
        test_size=10,
        clean_ratio=0.2,
        seed=7,
    )
    assert (tmp_path / "out" / "train.jsonl").read_bytes() == (
        tmp_path / "out2" / "train.jsonl"
    ).read_bytes()


def test_align_segment_only_masks_equal_byte_lengths() -> None:
    # "sử" -> "xử": cùng độ dài byte, mask bật và target đúng.
    prefix = "sử".encode()
    text = prefix + b"a"
    targets, mask = align_segment(list(text), 0, len(prefix), "xử")
    assert mask[: len(prefix)] == [1] * len(prefix)
    assert bytes(targets[: len(prefix)]) == "xử".encode()

    # "nghiêng" (8 byte) -> "nghiên" (7 byte): lệch độ dài -> mask tắt toàn span.
    text = "nghiêng".encode()
    targets, mask = align_segment(list(text), 0, len(text), "nghiên")
    assert len(targets) == len(text)
    assert all(value == 0 for value in mask)

    # Telex "đnag" -> "đang": cùng 5 byte -> mask bật 1-1.
    text = "đnag".encode()
    targets, mask = align_segment(list(text), 0, len(text), "đang")
    assert mask == [1] * len(text)
    assert bytes(targets) == "đang".encode()


def test_error_records_have_aligned_targets(tmp_path: Path) -> None:
    lexicon = tmp_path / "lexicon.txt"
    _write_lexicon(lexicon)
    corpus = tmp_path / "corpus.txt"
    corpus.write_text("Chúng ta cần xử lý hồ sơ sáp nhập.\n", encoding="utf-8")
    generate_dataset(
        tmp_path / "out",
        lexicon,
        [corpus],
        train_size=40,
        val_size=5,
        test_size=5,
        clean_ratio=0.0,
        seed=11,
    )
    records = [
        json.loads(line)
        for line in (tmp_path / "out" / "train.jsonl").read_text(encoding="utf-8").splitlines()
    ]
    assert any(record["spans"] for record in records)
    error_records = [record for record in records if record["spans"]]
    assert any(any(record["byte_mask"]) for record in error_records)
