"""Mô hình VSM (D2) bằng PyTorch.

Kiến trúc chốt Phương án A:
    Embedding(256, 160)
    -> Stem Conv1D(160, 160, k=5) + SiLU
    -> 6 x [DWConv(160, k=5) + RMSNorm + PW(160->320) + GELU + PW(320->160) + residual]
    -> Head_err(4) + Head_byte(256)

Mọi conv đều không bias. GELU dùng xấp xỉ tanh để parity với Rust.
"""

from __future__ import annotations

from dataclasses import asdict

import torch
from torch import Tensor, nn

from .config import ModelConfig


class RMSNorm(nn.Module):
    """RMSNorm trên trục kênh của tensor [B, C, T]."""

    def __init__(self, channels: int, eps: float = 1e-5) -> None:
        super().__init__()
        self.weight = nn.Parameter(torch.ones(channels))
        self.eps = eps

    def forward(self, x: Tensor) -> Tensor:
        variance = x.pow(2).mean(dim=1, keepdim=True)
        return x * torch.rsqrt(variance + self.eps) * self.weight.view(1, -1, 1)


class DepthwiseSeparableBlock(nn.Module):
    def __init__(self, config: ModelConfig, dilation: int) -> None:
        super().__init__()
        kernel = config.kernel_size
        padding = dilation * (kernel - 1) // 2
        self.dw = nn.Conv1d(
            config.d_model,
            config.d_model,
            kernel_size=kernel,
            padding=padding,
            dilation=dilation,
            groups=config.d_model,
            bias=False,
        )
        self.norm = RMSNorm(config.d_model, eps=config.rms_eps)
        self.pw1 = nn.Conv1d(config.d_model, config.expanded_dim, 1, bias=False)
        self.act = nn.GELU(approximate="tanh")
        self.pw2 = nn.Conv1d(config.expanded_dim, config.d_model, 1, bias=False)

    def forward(self, x: Tensor) -> Tensor:
        residual = x
        h = self.dw(x)
        h = self.norm(h)
        h = self.pw1(h)
        h = self.act(h)
        h = self.pw2(h)
        return residual + h


class VSMModel(nn.Module):
    def __init__(self, config: ModelConfig | None = None) -> None:
        super().__init__()
        self.config = config or ModelConfig()
        cfg = self.config

        self.embedding = nn.Embedding(cfg.vocab_size, cfg.d_model)
        self.stem = nn.Conv1d(
            cfg.d_model,
            cfg.d_model,
            kernel_size=cfg.kernel_size,
            padding=cfg.kernel_size // 2,
            bias=False,
        )
        self.stem_act = nn.SiLU()
        self.blocks = nn.ModuleList(
            DepthwiseSeparableBlock(cfg, dilation) for dilation in cfg.dilations
        )
        self.head_err = nn.Conv1d(cfg.d_model, cfg.n_error_classes, 1)
        self.head_byte = nn.Conv1d(cfg.d_model, cfg.vocab_size, 1)
        self.apply(self._init_weights)

    @staticmethod
    def _init_weights(module: nn.Module) -> None:
        if isinstance(module, (nn.Conv1d, nn.Linear)):
            nn.init.kaiming_normal_(module.weight, nonlinearity="linear")
            if module.bias is not None:
                nn.init.zeros_(module.bias)
        elif isinstance(module, nn.Embedding):
            nn.init.normal_(module.weight, mean=0.0, std=0.02)

    def forward(self, input_ids: Tensor) -> tuple[Tensor, Tensor]:
        """Trả về (error_logits [B,T,4], byte_logits [B,T,256])."""
        if input_ids.ndim != 2:
            raise ValueError("input_ids phải có shape [B, T].")

        x = self.embedding(input_ids).transpose(1, 2)  # [B, C, T]
        x = self.stem_act(self.stem(x))
        for block in self.blocks:
            x = block(x)

        error_logits = self.head_err(x).transpose(1, 2)  # [B, T, 4]
        byte_logits = self.head_byte(x).transpose(1, 2)  # [B, T, 256]
        return error_logits, byte_logits

    def config_dict(self) -> dict[str, object]:
        return asdict(self.config)

    def parameter_count(self) -> int:
        return sum(parameter.numel() for parameter in self.parameters())
