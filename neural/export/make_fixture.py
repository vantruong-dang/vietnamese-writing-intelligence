"""Sinh fixture .vsm nhỏ + expected output cho parity test Rust (không cần torch).

Chạy:
    python -m neural.export.make_fixture
"""

from __future__ import annotations

import argparse
import random
from pathlib import Path

from .reference import forward
from .vsm_format import TensorSpec, VsmConfig, quantize_per_output, read_vsm, write_vsm

FIXTURE_DIR = Path(__file__).resolve().parent.parent / "fixtures"
FIXTURE_VSM = FIXTURE_DIR / "tiny_model.vsm"
FIXTURE_EXPECTED = FIXTURE_DIR / "tiny_model_expected.txt"


def _random_values(rng: random.Random, count: int, scale: float = 0.5) -> list[float]:
    return [rng.uniform(-scale, scale) for _ in range(count)]


def _quantized_tensor(
    name: str,
    values: list[float],
    output_channels: int,
    shape: tuple[int, ...],
) -> list[TensorSpec]:
    quantized, scales = quantize_per_output(values, output_channels)
    scales_name = name[: -len(".weight")] + ".scales" if name.endswith(".weight") else f"{name}.scales"
    return [
        TensorSpec(name=name, dtype="i8", shape=shape, data=quantized),
        TensorSpec(
            name=scales_name,
            dtype="f32",
            shape=(output_channels,),
            data=scales,
        ),
    ]


def build_tensors(config: VsmConfig, seed: int = 1234) -> list[TensorSpec]:
    rng = random.Random(seed)
    d_model = config.d_model
    kernel = config.kernel_size
    expanded = config.expanded_dim
    tensors: list[TensorSpec] = []

    tensors += _quantized_tensor(
        "embedding.weight",
        _random_values(rng, config.vocab_size * d_model, 0.4),
        config.vocab_size,
        (config.vocab_size, d_model),
    )
    tensors += _quantized_tensor(
        "stem.weight",
        _random_values(rng, d_model * kernel * d_model, 0.3),
        d_model,
        (d_model, kernel, d_model),
    )
    for index in range(config.n_blocks):
        prefix = f"blocks.{index}"
        tensors += _quantized_tensor(
            f"{prefix}.dw.weight",
            _random_values(rng, d_model * kernel, 0.3),
            d_model,
            (d_model, kernel),
        )
        tensors.append(
            TensorSpec(
                name=f"{prefix}.norm.weight",
                dtype="f32",
                shape=(d_model,),
                data=[1.0 + rng.uniform(-0.05, 0.05) for _ in range(d_model)],
            )
        )
        tensors += _quantized_tensor(
            f"{prefix}.pw1.weight",
            _random_values(rng, expanded * d_model, 0.3),
            expanded,
            (expanded, 1, d_model),
        )
        tensors += _quantized_tensor(
            f"{prefix}.pw2.weight",
            _random_values(rng, d_model * expanded, 0.3),
            d_model,
            (d_model, 1, expanded),
        )
    tensors += _quantized_tensor(
        "head_err.weight",
        _random_values(rng, config.n_error_classes * d_model, 0.3),
        config.n_error_classes,
        (config.n_error_classes, 1, d_model),
    )
    tensors.append(
        TensorSpec(
            name="head_err.bias",
            dtype="f32",
            shape=(config.n_error_classes,),
            data=_random_values(rng, config.n_error_classes, 0.1),
        )
    )
    tensors += _quantized_tensor(
        "head_byte.weight",
        _random_values(rng, config.vocab_size * d_model, 0.3),
        config.vocab_size,
        (config.vocab_size, 1, d_model),
    )
    tensors.append(
        TensorSpec(
            name="head_byte.bias",
            dtype="f32",
            shape=(config.vocab_size,),
            data=_random_values(rng, config.vocab_size, 0.1),
        )
    )
    tensors.append(
        TensorSpec(
            name="model.dilations",
            dtype="i8",
            shape=(config.n_blocks,),
            data=[1, 2][: config.n_blocks],
        )
    )
    return tensors


def main() -> None:
    parser = argparse.ArgumentParser(description="Sinh fixture .vsm cho parity test")
    parser.add_argument("--out", default=str(FIXTURE_VSM))
    parser.add_argument("--expected", default=str(FIXTURE_EXPECTED))
    args = parser.parse_args()

    config = VsmConfig(
        vocab_size=256,
        d_model=8,
        n_blocks=2,
        kernel_size=3,
        expand=2,
        n_error_classes=4,
        max_seq_len=32,
        rf_radius_bytes=4,
    )
    tensors = build_tensors(config)
    # dilations của fixture là [1, 2]
    for tensor in tensors:
        if tensor.name == "model.dilations":
            tensor.data = [1, 2]
    write_vsm(args.out, config, tensors, int8=True)

    model = read_vsm(args.out)
    input_bytes = list("sửa lỗi".encode())
    error_logits, byte_logits = forward(model, input_bytes)
    byte_argmax = [max(range(len(row)), key=row.__getitem__) for row in byte_logits]

    lines = [
        "input " + " ".join(str(value) for value in input_bytes),
        "error_logits " + " ".join(f"{value:.6f}" for row in error_logits for value in row),
        "byte_argmax " + " ".join(str(value) for value in byte_argmax),
    ]
    Path(args.expected).write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"[+] fixture: {args.out}")
    print(f"[+] expected: {args.expected}")


if __name__ == "__main__":
    main()
