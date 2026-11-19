"""D4 — export checkpoint PyTorch sang .vsm INT8 và kiểm tra parity reference.

Ví dụ:
    python -m neural.export.export_vsm \
        --checkpoint neural/checkpoints/vsm_d2.pt \
        --out neural/fixtures/vsm_d2_int8.vsm
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import torch

from ..config import ModelConfig
from ..model import VSMModel
from .reference import forward as reference_forward
from .vsm_format import TensorSpec, VsmConfig, quantize_per_output, read_vsm, write_vsm


def _quantized(
    name: str,
    tensor: torch.Tensor,
    output_channels: int,
    shape: tuple[int, ...],
) -> list[TensorSpec]:
    values = tensor.detach().cpu().float().reshape(output_channels, -1).flatten().tolist()
    quantized, scales = quantize_per_output(values, output_channels)
    scales_name = name[: -len(".weight")] + ".scales"
    return [
        TensorSpec(name=name, dtype="i8", shape=shape, data=quantized),
        TensorSpec(name=scales_name, dtype="f32", shape=(output_channels,), data=scales),
    ]


def _f32_tensor(name: str, tensor: torch.Tensor, shape: tuple[int, ...]) -> TensorSpec:
    return TensorSpec(
        name=name,
        dtype="f32",
        shape=shape,
        data=tensor.detach().cpu().float().reshape(-1).tolist(),
    )


def _weight_tensor(
    name: str,
    tensor: torch.Tensor,
    output_channels: int,
    shape: tuple[int, ...],
    quantize: bool,
) -> list[TensorSpec]:
    if quantize:
        return _quantized(name, tensor, output_channels, shape)
    return [_f32_tensor(name, tensor, shape)]


def export_checkpoint(
    checkpoint_path: str | Path,
    output_path: str | Path,
    manifest_path: str | Path | None = None,
    skip_parity: bool = False,
    mixed_precision: bool = False,
) -> dict[str, object]:
    heads_f32 = mixed_precision
    aggressive_int8 = not mixed_precision
    checkpoint = torch.load(checkpoint_path, map_location="cpu")
    model_config = ModelConfig(**checkpoint["config"])
    model = VSMModel(model_config)
    model.load_state_dict(checkpoint["model"])
    model.eval()

    state = model.state_dict()
    tensors: list[TensorSpec] = []
    quantize_small = aggressive_int8
    tensors += _weight_tensor(
        "embedding.weight",
        state["embedding.weight"],
        model_config.vocab_size,
        (model_config.vocab_size, model_config.d_model),
        quantize_small,
    )
    # PyTorch: [out, in, k] -> runtime: [out, k, in]
    stem = state["stem.weight"].permute(0, 2, 1).contiguous()
    tensors += _weight_tensor(
        "stem.weight",
        stem,
        model_config.d_model,
        (model_config.d_model, model_config.kernel_size, model_config.d_model),
        quantize_small,
    )

    for index in range(model_config.n_blocks):
        prefix = f"blocks.{index}"
        dw = state[f"{prefix}.dw.weight"].reshape(model_config.d_model, model_config.kernel_size)
        tensors += _weight_tensor(
            f"{prefix}.dw.weight",
            dw,
            model_config.d_model,
            (model_config.d_model, model_config.kernel_size),
            quantize_small,
        )
        tensors.append(
            TensorSpec(
                name=f"{prefix}.norm.weight",
                dtype="f32",
                shape=(model_config.d_model,),
                data=state[f"{prefix}.norm.weight"].detach().cpu().float().tolist(),
            )
        )
        pw1 = state[f"{prefix}.pw1.weight"].reshape(
            model_config.expanded_dim, model_config.d_model
        )
        tensors += _quantized(
            f"{prefix}.pw1.weight",
            pw1,
            model_config.expanded_dim,
            (model_config.expanded_dim, 1, model_config.d_model),
        )
        pw2 = state[f"{prefix}.pw2.weight"].reshape(
            model_config.d_model, model_config.expanded_dim
        )
        tensors += _quantized(
            f"{prefix}.pw2.weight",
            pw2,
            model_config.d_model,
            (model_config.d_model, 1, model_config.expanded_dim),
        )

    tensors += _weight_tensor(
        "head_err.weight",
        state["head_err.weight"].reshape(model_config.n_error_classes, model_config.d_model),
        model_config.n_error_classes,
        (model_config.n_error_classes, 1, model_config.d_model),
        not heads_f32,
    )
    tensors.append(
        TensorSpec(
            name="head_err.bias",
            dtype="f32",
            shape=(model_config.n_error_classes,),
            data=state["head_err.bias"].detach().cpu().float().tolist(),
        )
    )
    tensors += _weight_tensor(
        "head_byte.weight",
        state["head_byte.weight"].reshape(model_config.vocab_size, model_config.d_model),
        model_config.vocab_size,
        (model_config.vocab_size, 1, model_config.d_model),
        not heads_f32,
    )
    tensors.append(
        TensorSpec(
            name="head_byte.bias",
            dtype="f32",
            shape=(model_config.vocab_size,),
            data=state["head_byte.bias"].detach().cpu().float().tolist(),
        )
    )
    tensors.append(
        TensorSpec(
            name="model.dilations",
            dtype="i8",
            shape=(model_config.n_blocks,),
            data=list(model_config.dilations),
        )
    )

    vsm_config = VsmConfig(
        vocab_size=model_config.vocab_size,
        d_model=model_config.d_model,
        n_blocks=model_config.n_blocks,
        kernel_size=model_config.kernel_size,
        expand=model_config.expand,
        n_error_classes=model_config.n_error_classes,
        max_seq_len=model_config.max_seq_len,
        rf_radius_bytes=model_config.rf_radius_bytes,
    )
    write_vsm(output_path, vsm_config, tensors, int8=True)
    size = Path(output_path).stat().st_size

    parity: dict[str, object] = {"checked": False}
    if not skip_parity:
        sample = "xin chào thế giới".encode()
        with torch.no_grad():
            error_logits, byte_logits = model(torch.tensor([list(sample)], dtype=torch.long))
        torch_error = error_logits[0].tolist()
        torch_byte = byte_logits[0].argmax(dim=-1).tolist()
        exported = read_vsm(output_path)
        ref_error, ref_byte = reference_forward(exported, list(sample))
        max_delta = max(
            abs(left - right)
            for torch_row, ref_row in zip(torch_error, ref_error, strict=True)
            for left, right in zip(torch_row, ref_row, strict=True)
        )
        ref_byte_argmax = [
            max(range(len(row)), key=row.__getitem__) for row in ref_byte
        ]
        parity = {
            "checked": True,
            "max_error_logit_delta": max_delta,
            "byte_argmax_match": ref_byte_argmax == torch_byte,
        }

    manifest = {
        "checkpoint": str(checkpoint_path),
        "output": str(output_path),
        "size_bytes": size,
        "config": vsm_config.as_dict(),
        "tensor_count": len(tensors),
        "mixed_precision": mixed_precision,
        "parity": parity,
    }
    if manifest_path is not None:
        Path(manifest_path).write_text(
            json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
        )
    return manifest


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Export VSM checkpoint -> .vsm INT8")
    parser.add_argument("--checkpoint", required=True)
    parser.add_argument("--out", default="neural/fixtures/vsm_d2_int8.vsm")
    parser.add_argument("--manifest", default=None)
    parser.add_argument("--skip-parity", action="store_true")
    parser.add_argument(
        "--mixed-precision",
        action="store_true",
        help="Giữ embedding/stem/DW/head ở F32; mặc định INT8 toàn bộ",
    )
    return parser.parse_args()


def main() -> None:
    args = _parse_args()
    manifest = export_checkpoint(
        args.checkpoint,
        args.out,
        manifest_path=args.manifest,
        skip_parity=args.skip_parity,
        mixed_precision=args.mixed_precision,
    )
    print(json.dumps(manifest, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
