"""Định dạng .vsm v1 — header, tensor descriptor, INT8/F32 payload.

Layout (little-endian):
    Header 48 byte:
        magic[4] = "VSM1", version u16, flags u16,
        vocab u32, d_model u32, n_blocks u32, kernel u32, expand u32,
        n_error_classes u32, max_seq_len u32, rf_radius u32,
        tensor_count u32, reserved u32
    tensor_count descriptor:
        name_len u16, name bytes, dtype u8, ndim u8, shape u32[ndim],
        scale f32, zero_point i32, data_offset u64, data_len u64
    align 16 -> data section.
    Mỗi tensor data được align 16; data_offset tính từ đầu data section.

INT8 dùng scale companion (tensor `*.scales`) cho per-channel/per-row.
F32 dùng trực tiếp, không cần scale companion.
"""

from __future__ import annotations

import struct
from collections.abc import Iterable, Sequence
from dataclasses import dataclass, field
from pathlib import Path

MAGIC = b"VSM1"
VERSION = 1
HEADER_SIZE = 48
DTYPE_F32 = 0
DTYPE_I8 = 1
FLAG_INT8 = 1


@dataclass(frozen=True)
class VsmConfig:
    vocab_size: int = 256
    d_model: int = 160
    n_blocks: int = 6
    kernel_size: int = 5
    expand: int = 2
    n_error_classes: int = 4
    max_seq_len: int = 256
    rf_radius_bytes: int = 44
    rms_eps: float = 1e-5

    @property
    def expanded_dim(self) -> int:
        return self.d_model * self.expand

    def as_dict(self) -> dict[str, int]:
        return {
            "vocab_size": self.vocab_size,
            "d_model": self.d_model,
            "n_blocks": self.n_blocks,
            "kernel_size": self.kernel_size,
            "expand": self.expand,
            "n_error_classes": self.n_error_classes,
            "max_seq_len": self.max_seq_len,
            "rf_radius_bytes": self.rf_radius_bytes,
        }


@dataclass
class TensorSpec:
    name: str
    dtype: str  # "f32" | "i8"
    shape: tuple[int, ...]
    data: list[float] | list[int]
    scale: float = 1.0
    zero_point: int = 0
    data_offset: int = 0
    data_len: int = 0

    @property
    def numel(self) -> int:
        total = 1
        for dimension in self.shape:
            total *= dimension
        return total


@dataclass
class VsmFile:
    config: VsmConfig
    tensors: dict[str, TensorSpec] = field(default_factory=dict)

    def tensor(self, name: str) -> TensorSpec:
        try:
            return self.tensors[name]
        except KeyError as error:
            raise KeyError(f"Thiếu tensor {name!r} trong .vsm") from error

    @property
    def dilations(self) -> list[int]:
        tensor = self.tensors.get("model.dilations")
        if tensor is not None:
            return [int(value) for value in tensor.data]
        if self.config.n_blocks == 6:
            return [1, 2, 4, 8, 4, 2]
        return [1] * self.config.n_blocks


def _align(value: int, alignment: int = 16) -> int:
    return (value + alignment - 1) // alignment * alignment


def _pack_data(tensor: TensorSpec) -> bytes:
    if tensor.dtype == "f32":
        return struct.pack(f"<{len(tensor.data)}f", *tensor.data)
    if tensor.dtype == "i8":
        return struct.pack(f"<{len(tensor.data)}b", *tensor.data)
    raise ValueError(f"dtype không hỗ trợ: {tensor.dtype}")


def write_vsm(
    path: str | Path,
    config: VsmConfig,
    tensors: Iterable[TensorSpec],
    int8: bool = True,
) -> None:
    items = sorted(tensors, key=lambda item: item.name)
    flags = FLAG_INT8 if int8 else 0

    data_parts: list[bytes] = []
    data_cursor = 0
    for tensor in items:
        payload = _pack_data(tensor)
        offset = _align(data_cursor)
        if offset > data_cursor:
            data_parts.append(b"\x00" * (offset - data_cursor))
        tensor.data_offset = offset
        tensor.data_len = len(payload)
        data_parts.append(payload)
        data_cursor = offset + len(payload)
    data_section = b"".join(data_parts)

    descriptors = bytearray()
    for tensor in items:
        name = tensor.name.encode("utf-8")
        descriptors += struct.pack("<H", len(name))
        descriptors += name
        descriptors += struct.pack("<BB", DTYPE_F32 if tensor.dtype == "f32" else DTYPE_I8, len(tensor.shape))
        for dimension in tensor.shape:
            descriptors += struct.pack("<I", dimension)
        descriptors += struct.pack("<fiiQQ", tensor.scale, tensor.zero_point, 0, tensor.data_offset, tensor.data_len)

    header = struct.pack(
        "<4sHHIIIIIIIIII",
        MAGIC,
        VERSION,
        flags,
        config.vocab_size,
        config.d_model,
        config.n_blocks,
        config.kernel_size,
        config.expand,
        config.n_error_classes,
        config.max_seq_len,
        config.rf_radius_bytes,
        len(items),
        0,
    )
    prefix = header + bytes(descriptors)
    prefix += b"\x00" * (_align(len(prefix)) - len(prefix))

    output = Path(path)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(prefix + data_section)


def read_vsm(path: str | Path) -> VsmFile:
    payload = Path(path).read_bytes()
    if len(payload) < HEADER_SIZE:
        raise ValueError("File .vsm quá ngắn.")
    (
        magic,
        version,
        _flags,
        vocab_size,
        d_model,
        n_blocks,
        kernel_size,
        expand,
        n_error_classes,
        max_seq_len,
        rf_radius,
        tensor_count,
        _reserved,
    ) = struct.unpack("<4sHHIIIIIIIIII", payload[:HEADER_SIZE])
    if magic != MAGIC:
        raise ValueError(f"Sai magic: {magic!r}")
    if version != VERSION:
        raise ValueError(f"Không hỗ trợ version {version}")

    config = VsmConfig(
        vocab_size=vocab_size,
        d_model=d_model,
        n_blocks=n_blocks,
        kernel_size=kernel_size,
        expand=expand,
        n_error_classes=n_error_classes,
        max_seq_len=max_seq_len,
        rf_radius_bytes=rf_radius,
    )
    cursor = HEADER_SIZE
    descriptors: list[tuple[str, str, tuple[int, ...], float, int, int, int]] = []
    for _ in range(tensor_count):
        (name_len,) = struct.unpack_from("<H", payload, cursor)
        cursor += 2
        name = payload[cursor: cursor + name_len].decode("utf-8")
        cursor += name_len
        dtype_id, ndim = struct.unpack_from("<BB", payload, cursor)
        cursor += 2
        shape = []
        for _dimension in range(ndim):
            (dimension,) = struct.unpack_from("<I", payload, cursor)
            cursor += 4
            shape.append(dimension)
        scale, zero_point, _unused, data_offset, data_len = struct.unpack_from(
            "<fiiQQ", payload, cursor
        )
        cursor += struct.calcsize("<fiiQQ")
        descriptors.append((name, "f32" if dtype_id == DTYPE_F32 else "i8", tuple(shape), scale, zero_point, data_offset, data_len))

    data_start = _align(cursor)
    tensors: dict[str, TensorSpec] = {}
    for name, dtype, shape, scale, zero_point, data_offset, data_len in descriptors:
        start = data_start + data_offset
        end = start + data_len
        raw = payload[start:end]
        if len(raw) != data_len:
            raise ValueError(f"Tensor {name!r} bị cắt cụt.")
        if dtype == "f32":
            values = list(struct.unpack(f"<{data_len // 4}f", raw))
        else:
            values = list(struct.unpack(f"<{data_len}b", raw))
        tensors[name] = TensorSpec(
            name=name,
            dtype=dtype,
            shape=shape,
            data=values,
            scale=scale,
            zero_point=zero_point,
            data_offset=data_offset,
            data_len=data_len,
        )
    return VsmFile(config=config, tensors=tensors)


def quantize_per_output(
    values: Sequence[float],
    output_channels: int,
) -> tuple[list[int], list[float]]:
    """Symmetric INT8 per output channel. values layout [out, inner]."""
    if output_channels <= 0 or len(values) % output_channels != 0:
        raise ValueError("Số phần tử không chia hết cho output_channels.")
    inner = len(values) // output_channels
    quantized = [0] * len(values)
    scales = [1.0] * output_channels
    for channel in range(output_channels):
        start = channel * inner
        row = values[start: start + inner]
        maximum = max((abs(value) for value in row), default=0.0)
        scale = maximum / 127.0 if maximum > 0 else 1.0
        scales[channel] = scale
        for offset, value in enumerate(row):
            quantized[start + offset] = max(-127, min(127, round(value / scale)))
    return quantized, scales


def dequantize(values: Sequence[int], scales: Sequence[float], output_channels: int) -> list[float]:
    inner = len(values) // output_channels
    output: list[float] = []
    for channel in range(output_channels):
        scale = scales[channel]
        output.extend(value * scale for value in values[channel * inner: (channel + 1) * inner])
    return output
