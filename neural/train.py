"""Huấn luyện VSM (D3) với Muon cho stem/pointwise và AdamW cho phần còn lại.

Ví dụ:
    python -m neural.train \
        --train data_processed/neural/train.jsonl \
        --val data_processed/neural/val.jsonl \
        --out neural/checkpoints/vsm_d2.pt
"""

from __future__ import annotations

import argparse
import json
import math
import random
from pathlib import Path
from typing import Any

import torch
from torch import Tensor
from torch.utils.data import DataLoader

from .config import DEFAULT_TRAIN_CONFIG, ModelConfig, TrainConfig
from .dataset import ByteErrorDataset, collate_batch
from .model import VSMModel
from .muon import Muon, split_muon_adamw_parameters


def set_seed(seed: int) -> None:
    random.seed(seed)
    torch.manual_seed(seed)
    if torch.cuda.is_available():
        torch.cuda.manual_seed_all(seed)


def build_optimizers(model: VSMModel, config: TrainConfig) -> tuple[torch.optim.Optimizer, torch.optim.Optimizer]:
    muon_parameters, adamw_parameters = split_muon_adamw_parameters(model)
    if not muon_parameters:
        raise RuntimeError("Không tìm thấy tham số nào cho Muon.")
    muon = Muon(
        muon_parameters,
        lr=config.muon_lr,
        momentum=config.muon_momentum,
        ns_steps=config.muon_ns_steps,
        weight_decay=config.weight_decay,
    )
    adamw = torch.optim.AdamW(
        adamw_parameters,
        lr=config.learning_rate,
        weight_decay=config.weight_decay,
        betas=(0.9, 0.95),
    )
    return muon, adamw


def build_scheduler(
    optimizer: torch.optim.Optimizer, total_steps: int, warmup_ratio: float
) -> torch.optim.lr_scheduler.LambdaLR:
    warmup_steps = max(1, int(total_steps * warmup_ratio))

    def lr_lambda(step: int) -> float:
        if step < warmup_steps:
            return (step + 1) / warmup_steps
        progress = (step - warmup_steps) / max(1, total_steps - warmup_steps)
        return 0.5 * (1.0 + math.cos(math.pi * min(1.0, progress)))

    return torch.optim.lr_scheduler.LambdaLR(optimizer, lr_lambda)


def compute_loss(
    model: VSMModel,
    batch: dict[str, Tensor],
    config: TrainConfig,
) -> tuple[Tensor, Tensor, Tensor]:
    input_ids = batch["input_ids"]
    error_logits, byte_logits = model(input_ids)
    error_loss = torch.nn.functional.cross_entropy(
        error_logits.reshape(-1, error_logits.size(-1)),
        batch["labels"].reshape(-1),
        ignore_index=-100,
        label_smoothing=config.label_smoothing,
    )
    byte_loss_all = torch.nn.functional.cross_entropy(
        byte_logits.reshape(-1, byte_logits.size(-1)),
        batch["byte_targets"].reshape(-1),
        reduction="none",
    ).reshape_as(batch["byte_mask"])
    mask = batch["byte_mask"].float()
    byte_loss = (byte_loss_all * mask).sum() / mask.sum().clamp_min(1.0)
    total = config.error_loss_weight * error_loss + config.byte_loss_weight * byte_loss
    return total, error_loss, byte_loss


def decode_error_spans(logits: Tensor) -> list[tuple[int, int, int]]:
    """Gộp các vị trí liên tiếp cùng nhãn lỗi thành span."""
    predictions = logits.argmax(dim=-1).tolist()
    spans: list[tuple[int, int, int]] = []
    start: int | None = None
    current = 0
    for position, label in enumerate(predictions):
        if label != current:
            if current != 0 and start is not None:
                spans.append((start, position, current))
            start = position if label != 0 else None
            current = label
    if current != 0 and start is not None:
        spans.append((start, len(predictions), current))
    return spans


def gold_spans(labels: Tensor) -> list[tuple[int, int, int]]:
    values = labels.tolist()
    spans: list[tuple[int, int, int]] = []
    start: int | None = None
    current = 0
    for position, label in enumerate(values):
        if label != current:
            if current != 0 and start is not None:
                spans.append((start, position, current))
            start = position if label != 0 else None
            current = label
    if current != 0 and start is not None:
        spans.append((start, len(values), current))
    return spans


def _overlap(left: tuple[int, int, int], right: tuple[int, int, int]) -> bool:
    return left[2] == right[2] and left[0] < right[1] and right[0] < left[1]


def span_f1(
    predictions: list[list[tuple[int, int, int]]],
    golds: list[list[tuple[int, int, int]]],
) -> dict[str, float]:
    true_positive = false_positive = false_negative = 0
    for predicted, gold in zip(predictions, golds, strict=True):
        matched_gold: set[int] = set()
        for span in predicted:
            match = next(
                (idx for idx, candidate in enumerate(gold) if idx not in matched_gold and _overlap(span, candidate)),
                None,
            )
            if match is None:
                false_positive += 1
            else:
                matched_gold.add(match)
                true_positive += 1
        false_negative += len(gold) - len(matched_gold)
    precision = true_positive / max(1, true_positive + false_positive)
    recall = true_positive / max(1, true_positive + false_negative)
    f1 = 2 * precision * recall / max(1e-9, precision + recall)
    return {"precision": precision, "recall": recall, "f1": f1}


@torch.no_grad()
def evaluate(
    model: VSMModel,
    loader: DataLoader,
    config: TrainConfig,
    device: torch.device,
) -> dict[str, float]:
    model.eval()
    total_loss = 0.0
    total_batches = 0
    byte_correct = 0
    byte_total = 0
    predicted_spans: list[list[tuple[int, int, int]]] = []
    golds: list[list[tuple[int, int, int]]] = []

    for batch in loader:
        batch = {key: value.to(device) for key, value in batch.items()}
        loss, _, _ = compute_loss(model, batch, config)
        total_loss += float(loss)
        total_batches += 1
        error_logits, byte_logits = model(batch["input_ids"])
        byte_predictions = byte_logits.argmax(dim=-1)
        mask = batch["byte_mask"].bool()
        byte_correct += int((byte_predictions[mask] == batch["byte_targets"][mask]).sum())
        byte_total += int(mask.sum())
        for row in range(error_logits.size(0)):
            length = int(batch["lengths"][row])
            predicted_spans.append(decode_error_spans(error_logits[row, :length]))
            golds.append(gold_spans(batch["labels"][row, :length]))

    metrics = span_f1(predicted_spans, golds)
    metrics["loss"] = total_loss / max(1, total_batches)
    metrics["byte_accuracy"] = byte_correct / max(1, byte_total)
    return metrics


def train(
    train_path: str,
    val_path: str,
    output_path: str,
    model_config: ModelConfig,
    train_config: TrainConfig,
) -> dict[str, Any]:
    set_seed(train_config.seed)
    device = torch.device(train_config.device)

    train_dataset = ByteErrorDataset(train_path, max_seq_len=train_config.max_seq_len)
    val_dataset = ByteErrorDataset(val_path, max_seq_len=train_config.max_seq_len, random_crop=False)
    train_loader = DataLoader(
        train_dataset,
        batch_size=train_config.batch_size,
        shuffle=True,
        collate_fn=collate_batch,
        num_workers=train_config.num_workers,
    )
    val_loader = DataLoader(
        val_dataset,
        batch_size=train_config.batch_size,
        shuffle=False,
        collate_fn=collate_batch,
        num_workers=train_config.num_workers,
    )

    model = VSMModel(model_config).to(device)
    muon, adamw = build_optimizers(model, train_config)
    total_steps = max(1, len(train_loader) * train_config.epochs)
    muon_scheduler = build_scheduler(muon, total_steps, train_config.warmup_ratio)
    adamw_scheduler = build_scheduler(adamw, total_steps, train_config.warmup_ratio)

    output = Path(output_path)
    output.parent.mkdir(parents=True, exist_ok=True)
    best_f1 = -1.0
    history: list[dict[str, float]] = []
    accumulation = max(1, train_config.grad_accum_steps)
    amp_enabled = train_config.use_amp and device.type == "cuda"
    amp_dtype = torch.bfloat16 if train_config.amp_dtype == "bf16" else torch.float16
    scaler = torch.amp.GradScaler(
        device.type,
        enabled=amp_enabled and train_config.amp_dtype == "fp16",
    )

    for epoch in range(1, train_config.epochs + 1):
        model.train()
        epoch_loss = 0.0
        for step, batch in enumerate(train_loader, start=1):
            batch = {key: value.to(device) for key, value in batch.items()}
            muon.zero_grad(set_to_none=True)
            adamw.zero_grad(set_to_none=True)
            with torch.autocast(device_type=device.type, dtype=amp_dtype, enabled=amp_enabled):
                loss, _, _ = compute_loss(model, batch, train_config)
            scaler.scale(loss / accumulation).backward()
            should_step = step % accumulation == 0 or step == len(train_loader)
            if should_step:
                if scaler.is_enabled():
                    scaler.unscale_(muon)
                    scaler.unscale_(adamw)
                torch.nn.utils.clip_grad_norm_(model.parameters(), train_config.grad_clip)
                scaler.step(muon)
                scaler.step(adamw)
                scaler.update()
                muon_scheduler.step()
                adamw_scheduler.step()
            epoch_loss += float(loss.detach())
            if step % train_config.log_every == 0:
                print(f"epoch {epoch} step {step}/{len(train_loader)} loss {float(loss.detach()):.4f}")

        metrics = evaluate(model, val_loader, train_config, device)
        metrics["epoch"] = float(epoch)
        metrics["train_loss"] = epoch_loss / max(1, len(train_loader))
        history.append(metrics)
        print(json.dumps(metrics, ensure_ascii=False))

        if metrics["f1"] > best_f1:
            best_f1 = metrics["f1"]
            torch.save(
                {
                    "model": model.state_dict(),
                    "config": model.config_dict(),
                    "metrics": metrics,
                    "epoch": epoch,
                },
                output,
            )

    return {"best_f1": best_f1, "history": history, "checkpoint": str(output)}


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Huấn luyện VSM D2")
    parser.add_argument("--train", required=True)
    parser.add_argument("--val", required=True)
    parser.add_argument("--out", default="neural/checkpoints/vsm_d2.pt")
    parser.add_argument("--epochs", type=int, default=DEFAULT_TRAIN_CONFIG.epochs)
    parser.add_argument("--batch-size", type=int, default=DEFAULT_TRAIN_CONFIG.batch_size)
    parser.add_argument("--lr", type=float, default=DEFAULT_TRAIN_CONFIG.learning_rate)
    parser.add_argument("--muon-lr", type=float, default=DEFAULT_TRAIN_CONFIG.muon_lr)
    parser.add_argument("--device", default=DEFAULT_TRAIN_CONFIG.device)
    parser.add_argument("--max-seq-len", type=int, default=DEFAULT_TRAIN_CONFIG.max_seq_len)
    parser.add_argument("--seed", type=int, default=DEFAULT_TRAIN_CONFIG.seed)
    parser.add_argument("--amp", choices=["off", "bf16", "fp16"], default="off")
    parser.add_argument("--grad-accum", type=int, default=1)
    parser.add_argument("--d-model", type=int, default=160)
    parser.add_argument("--n-layers", type=int, default=6)
    parser.add_argument("--dilations", default="1,2,4,8,4,2")
    parser.add_argument("--expand", type=int, default=2)
    parser.add_argument("--kernel-size", type=int, default=5)
    return parser.parse_args()


def main() -> None:
    args = _parse_args()
    train_config = TrainConfig(
        epochs=args.epochs,
        batch_size=args.batch_size,
        learning_rate=args.lr,
        muon_lr=args.muon_lr,
        device=args.device,
        max_seq_len=args.max_seq_len,
        seed=args.seed,
        use_amp=args.amp != "off",
        amp_dtype="bf16" if args.amp == "off" else args.amp,
        grad_accum_steps=args.grad_accum,
    )
    model_config = ModelConfig(
        d_model=args.d_model,
        n_blocks=args.n_layers,
        dilations=tuple(int(value) for value in args.dilations.split(",") if value.strip()),
        expand=args.expand,
        kernel_size=args.kernel_size,
        max_seq_len=args.max_seq_len,
    )
    result = train(args.train, args.val, args.out, model_config, train_config)
    print(json.dumps(result, ensure_ascii=False))


if __name__ == "__main__":
    main()
