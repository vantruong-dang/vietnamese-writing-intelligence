"""Reference forward thuần Python cho .vsm (dùng cho parity test, không cần torch).

Layout hoạt động: [T][C] (time-major). Trọng số conv đã ở layout [out][k][in].
"""

from __future__ import annotations

import math
from collections.abc import Sequence

from .vsm_format import VsmFile


def _dequant_i8(values: Sequence[int], scales: Sequence[float], channels: int) -> list[float]:
    inner = len(values) // channels
    output: list[float] = []
    for channel in range(channels):
        scale = scales[channel]
        output.extend(value * scale for value in values[channel * inner: (channel + 1) * inner])
    return output


def silu(value: float) -> float:
    return value / (1.0 + math.exp(-value))


def gelu_tanh(value: float) -> float:
    coefficient = math.sqrt(2.0 / math.pi)
    return 0.5 * value * (1.0 + math.tanh(coefficient * (value + 0.044715 * value**3)))


def dense_conv(
    x: Sequence[float],
    weight: Sequence[float],
    out_channels: int,
    in_channels: int,
    kernel: int,
    dilation: int,
) -> list[float]:
    length = len(x) // in_channels
    output = [0.0] * (length * out_channels)
    center = kernel // 2
    for time in range(length):
        for out_channel in range(out_channels):
            total = 0.0
            for tap in range(kernel):
                source = time + (tap - center) * dilation
                if source < 0 or source >= length:
                    continue
                weight_base = (out_channel * kernel + tap) * in_channels
                input_base = source * in_channels
                for in_channel in range(in_channels):
                    total += x[input_base + in_channel] * weight[weight_base + in_channel]
            output[time * out_channels + out_channel] = total
    return output


def depthwise_conv(
    x: Sequence[float],
    weight: Sequence[float],
    channels: int,
    kernel: int,
    dilation: int,
) -> list[float]:
    length = len(x) // channels
    output = [0.0] * len(x)
    center = kernel // 2
    for time in range(length):
        for channel in range(channels):
            total = 0.0
            for tap in range(kernel):
                source = time + (tap - center) * dilation
                if source < 0 or source >= length:
                    continue
                total += x[source * channels + channel] * weight[channel * kernel + tap]
            output[time * channels + channel] = total
    return output


def pointwise_conv(
    x: Sequence[float],
    weight: Sequence[float],
    out_channels: int,
    in_channels: int,
) -> list[float]:
    length = len(x) // in_channels
    output = [0.0] * (length * out_channels)
    for time in range(length):
        input_base = time * in_channels
        output_base = time * out_channels
        for out_channel in range(out_channels):
            weight_base = out_channel * in_channels
            total = 0.0
            for in_channel in range(in_channels):
                total += x[input_base + in_channel] * weight[weight_base + in_channel]
            output[output_base + out_channel] = total
    return output


def rms_norm(x: list[float], weight: Sequence[float], channels: int, eps: float = 1e-5) -> list[float]:
    length = len(x) // channels
    output = [0.0] * len(x)
    for time in range(length):
        base = time * channels
        mean_square = sum(x[base + channel] ** 2 for channel in range(channels)) / channels
        scale = 1.0 / math.sqrt(mean_square + eps)
        for channel in range(channels):
            output[base + channel] = x[base + channel] * scale * weight[channel]
    return output


def _tensor_f32(model: VsmFile, name: str) -> list[float]:
    tensor = model.tensor(name)
    if tensor.dtype != "f32":
        raise ValueError(f"{name} phải là F32.")
    return list(tensor.data)


def _tensor_i8_dequant(model: VsmFile, name: str, scales_name: str, channels: int) -> list[float]:
    tensor = model.tensor(name)
    if tensor.dtype != "i8":
        raise ValueError(f"{name} phải là I8.")
    scales = _tensor_f32(model, scales_name)
    return _dequant_i8(tensor.data, scales, channels)


def forward(model: VsmFile, input_bytes: Sequence[int]) -> tuple[list[list[float]], list[list[float]]]:
    config = model.config
    length = len(input_bytes)
    channels = config.d_model
    dilations = model.dilations

    embedding = model.tensor("embedding.weight")
    embedding_scales = _tensor_f32(model, "embedding.scales")
    x: list[float] = []
    for byte in input_bytes:
        row = byte * channels
        scale = embedding_scales[byte]
        x.extend(value * scale for value in embedding.data[row: row + channels])

    stem_weight = _tensor_i8_dequant(model, "stem.weight", "stem.scales", channels)
    x = dense_conv(x, stem_weight, channels, channels, config.kernel_size, 1)
    x = [silu(value) for value in x]

    for index in range(config.n_blocks):
        prefix = f"blocks.{index}"
        residual = list(x)
        dw_weight = _tensor_i8_dequant(
            model, f"{prefix}.dw.weight", f"{prefix}.dw.scales", channels
        )
        h = depthwise_conv(
            x, dw_weight, channels, config.kernel_size, dilations[index]
        )
        h = rms_norm(h, _tensor_f32(model, f"{prefix}.norm.weight"), channels, config.rms_eps)
        pw1_channels = config.expanded_dim
        pw1_weight = _tensor_i8_dequant(
            model, f"{prefix}.pw1.weight", f"{prefix}.pw1.scales", pw1_channels
        )
        h = pointwise_conv(h, pw1_weight, pw1_channels, channels)
        h = [gelu_tanh(value) for value in h]
        pw2_weight = _tensor_i8_dequant(
            model, f"{prefix}.pw2.weight", f"{prefix}.pw2.scales", channels
        )
        h = pointwise_conv(h, pw2_weight, channels, pw1_channels)
        x = [left + right for left, right in zip(residual, h, strict=True)]

    error_weight = _tensor_i8_dequant(
        model, "head_err.weight", "head_err.scales", config.n_error_classes
    )
    error_bias = _tensor_f32(model, "head_err.bias")
    error_logits = pointwise_conv(x, error_weight, config.n_error_classes, channels)

    byte_weight = _tensor_i8_dequant(
        model, "head_byte.weight", "head_byte.scales", config.vocab_size
    )
    byte_bias = _tensor_f32(model, "head_byte.bias")
    byte_logits = pointwise_conv(x, byte_weight, config.vocab_size, channels)

    error_rows = []
    byte_rows = []
    for time in range(length):
        error_row = [
            error_logits[time * config.n_error_classes + index] + error_bias[index]
            for index in range(config.n_error_classes)
        ]
        byte_row = [
            byte_logits[time * config.vocab_size + index] + byte_bias[index]
            for index in range(config.vocab_size)
        ]
        error_rows.append(error_row)
        byte_rows.append(byte_row)
    return error_rows, byte_rows
