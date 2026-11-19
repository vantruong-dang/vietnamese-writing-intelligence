"""Muon optimizer tối giản cho các ma trận 2D (stem + pointwise conv).

Tham khảo: Keller Jordan, "Muon: An optimizer for hidden layers in neural networks".
Chỉ dùng cho trọng số >= 2D; embedding, depthwise conv, RMSNorm và head dùng AdamW.
"""

from __future__ import annotations

import torch
from torch import Tensor, nn


def _zeropower_via_newtonschulz5(matrix: Tensor, steps: int = 5, eps: float = 1e-7) -> Tensor:
    """Xấp xỉ ma trận trực giao gần nhất bằng Newton-Schulz bậc 5."""
    a, b, c = 3.4445, -4.7750, 2.0315
    x = matrix.float()
    transposed = False
    if x.size(0) > x.size(1):
        x = x.T
        transposed = True
    x = x / (x.norm() + eps)
    for _ in range(steps):
        gram = x @ x.T
        update = b * gram + c * (gram @ gram)
        x = a * x + update @ x
    if transposed:
        x = x.T
    return x


class Muon(torch.optim.Optimizer):
    """Muon với momentum + Newton-Schulz, chỉ nhận tham số >= 2D."""

    def __init__(
        self,
        params,
        lr: float = 2e-2,
        momentum: float = 0.95,
        nesterov: bool = True,
        ns_steps: int = 5,
        weight_decay: float = 0.0,
    ) -> None:
        defaults = {
            "lr": lr,
            "momentum": momentum,
            "nesterov": nesterov,
            "ns_steps": ns_steps,
            "weight_decay": weight_decay,
        }
        super().__init__(params, defaults)

    @torch.no_grad()
    def step(self, closure=None):  # type: ignore[override]
        loss = None
        if closure is not None:
            with torch.enable_grad():
                loss = closure()

        for group in self.param_groups:
            lr = group["lr"]
            momentum = group["momentum"]
            nesterov = group["nesterov"]
            ns_steps = group["ns_steps"]
            weight_decay = group["weight_decay"]

            for parameter in group["params"]:
                if parameter.grad is None:
                    continue
                gradient = parameter.grad
                if gradient.ndim < 2:
                    raise ValueError("Muon chỉ hỗ trợ tham số có >= 2 chiều.")

                state = self.state[parameter]
                if "momentum_buffer" not in state:
                    state["momentum_buffer"] = torch.zeros_like(gradient)
                buffer = state["momentum_buffer"]
                buffer.mul_(momentum).add_(gradient)
                update = gradient.add(buffer, alpha=momentum) if nesterov else buffer

                if weight_decay:
                    parameter.mul_(1.0 - lr * weight_decay)

                original_shape = update.shape
                flattened = update.reshape(original_shape[0], -1)
                orthogonal = _zeropower_via_newtonschulz5(flattened, steps=ns_steps)
                orthogonal = orthogonal.reshape(original_shape)
                parameter.add_(orthogonal.to(parameter.dtype), alpha=-lr)
        return loss


def split_muon_adamw_parameters(model: nn.Module) -> tuple[list[nn.Parameter], list[nn.Parameter]]:
    """Tách tham số theo quy tắc D2.

    Muon: stem conv + toàn bộ pointwise conv (pw1, pw2) — các ma trận trộn kênh.
    AdamW: embedding, depthwise conv, RMSNorm, head_err, head_byte, bias.
    """
    muon: list[nn.Parameter] = []
    adamw: list[nn.Parameter] = []
    for name, parameter in model.named_parameters():
        if not parameter.requires_grad:
            continue
        is_stem = name.startswith("stem.")
        is_pointwise = ".pw1." in name or ".pw2." in name
        if (is_stem or is_pointwise) and parameter.ndim >= 2:
            muon.append(parameter)
        else:
            adamw.append(parameter)
    return muon, adamw
