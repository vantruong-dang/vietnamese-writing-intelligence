"""D4 — định dạng .vsm, export INT8 và fixture parity."""

from .vsm_format import TensorSpec, VsmConfig, VsmFile, quantize_per_output, read_vsm, write_vsm

__all__ = [
    "TensorSpec",
    "VsmConfig",
    "VsmFile",
    "quantize_per_output",
    "read_vsm",
    "write_vsm",
]
