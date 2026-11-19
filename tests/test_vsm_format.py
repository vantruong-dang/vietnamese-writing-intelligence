import json
from pathlib import Path

from neural.export.make_fixture import build_tensors
from neural.export.reference import forward
from neural.export.vsm_format import (
    VsmConfig,
    dequantize,
    quantize_per_output,
    read_vsm,
    write_vsm,
)


def _config() -> VsmConfig:
    return VsmConfig(
        vocab_size=256,
        d_model=8,
        n_blocks=2,
        kernel_size=3,
        expand=2,
        n_error_classes=4,
        max_seq_len=32,
        rf_radius_bytes=4,
    )


def test_vsm_roundtrip_and_reference_forward(tmp_path: Path) -> None:
    config = _config()
    path = tmp_path / "tiny.vsm"
    write_vsm(path, config, build_tensors(config), int8=True)
    model = read_vsm(path)
    assert model.config.d_model == 8
    assert model.config.n_blocks == 2
    assert model.dilations == [1, 2]
    assert "embedding.weight" in model.tensors
    assert "blocks.1.pw2.scales" in model.tensors

    input_bytes = list("sửa lỗi".encode())
    error_logits, byte_logits = forward(model, input_bytes)
    assert len(error_logits) == len(input_bytes)
    assert len(byte_logits) == len(input_bytes)
    assert all(len(row) == 4 for row in error_logits)
    assert all(len(row) == 256 for row in byte_logits)


def test_quantize_dequantize_roundtrip() -> None:
    values = [0.0, 0.5, -0.5, 1.0, -1.0, 0.25]
    quantized, scales = quantize_per_output(values, 2)
    restored = dequantize(quantized, scales, 2)
    assert len(restored) == len(values)
    for original, value in zip(values, restored, strict=True):
        assert abs(original - value) <= max(scales) / 2 + 1e-6


def test_fixture_expected_matches_reference() -> None:
    fixture_dir = Path(__file__).resolve().parent.parent / "neural" / "fixtures"
    model = read_vsm(fixture_dir / "tiny_model.vsm")
    lines = (fixture_dir / "tiny_model_expected.txt").read_text(encoding="utf-8").splitlines()
    values = {}
    for line in lines:
        key, *rest = line.split()
        values[key] = [float(item) for item in rest]
    input_bytes = [int(item) for item in values["input"]]
    expected_logits = values["error_logits"]
    error_logits, byte_logits = forward(model, input_bytes)
    flat = [value for row in error_logits for value in row]
    assert len(flat) == len(expected_logits)
    for actual, expected in zip(flat, expected_logits, strict=True):
        assert abs(actual - expected) <= 2e-6
    expected_argmax = [int(item) for item in values["byte_argmax"]]
    actual_argmax = [max(range(len(row)), key=row.__getitem__) for row in byte_logits]
    assert actual_argmax == expected_argmax


def test_manifest_is_json_serializable(tmp_path: Path) -> None:
    config = _config()
    path = tmp_path / "tiny.vsm"
    write_vsm(path, config, build_tensors(config), int8=True)
    payload = {"config": config.as_dict(), "tensors": sorted(read_vsm(path).tensors)}
    text = json.dumps(payload, ensure_ascii=False)
    assert "embedding.weight" in text
