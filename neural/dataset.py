"""Dataset byte-level cho VSM (D1 -> D3).

Mỗi record JSONL:
    text, clean_text, spans[{start,end,label}], byte_targets[], byte_mask[]

- labels: 0=CLEAN, 1=PHONETIC, 2=TONE, 3=WORD (per-byte span tagging).
- byte_targets: byte đúng tại từng vị trí input (dùng cho head_byte).
- byte_mask: 1 nếu vị trí đó có target đáng học, 0 nếu bỏ qua.
"""

from __future__ import annotations

import json
import random
from pathlib import Path
from typing import Any

import torch
from torch import Tensor
from torch.utils.data import Dataset


class ByteErrorDataset(Dataset):
    def __init__(
        self,
        path: str | Path,
        max_seq_len: int = 256,
        random_crop: bool = True,
        seed: int = 42,
    ) -> None:
        self.path = Path(path)
        self.max_seq_len = max_seq_len
        self.random_crop = random_crop
        self.rng = random.Random(seed)
        self.offsets: list[int] = []
        with self.path.open("rb") as handle:
            while True:
                offset = handle.tell()
                line = handle.readline()
                if not line:
                    break
                if line.strip():
                    self.offsets.append(offset)
        if not self.offsets:
            raise ValueError(f"Dataset rỗng: {self.path}")
        self._handle = self.path.open("rb")

    def __len__(self) -> int:
        return len(self.offsets)

    def __del__(self) -> None:
        handle = getattr(self, "_handle", None)
        if handle is not None:
            handle.close()

    def _read(self, index: int) -> dict[str, Any]:
        self._handle.seek(self.offsets[index])
        line = self._handle.readline()
        return json.loads(line.decode("utf-8"))

    def _window(self, raw_length: int, spans: list[dict[str, int]]) -> tuple[int, int]:
        if raw_length <= self.max_seq_len:
            return 0, raw_length
        if not self.random_crop and not spans:
            return 0, self.max_seq_len
        anchor = spans[0]["start"] if spans else 0
        if self.random_crop:
            anchor = max(0, anchor - self.rng.randint(0, self.max_seq_len // 4))
        start = min(max(0, anchor - self.max_seq_len // 4), raw_length - self.max_seq_len)
        return start, start + self.max_seq_len

    def __getitem__(self, index: int) -> dict[str, Tensor]:
        record = self._read(index)
        text_bytes = list(record["text"].encode("utf-8"))
        raw_length = len(text_bytes)
        spans = [
            {
                "start": int(span["start"]),
                "end": int(span["end"]),
                "label": int(span["label"]),
            }
            for span in record.get("spans", [])
        ]
        start, end = self._window(raw_length, spans)
        input_ids = text_bytes[start:end]
        length = len(input_ids)

        labels = [0] * length
        for span in spans:
            span_start = max(0, span["start"] - start)
            span_end = min(length, span["end"] - start)
            if span_start < span_end:
                for position in range(span_start, span_end):
                    labels[position] = span["label"]

        raw_targets = record.get("byte_targets")
        raw_mask = record.get("byte_mask")
        if raw_targets is None:
            raw_targets = text_bytes
        if raw_mask is None:
            raw_mask = [0] * raw_length
        byte_targets = list(raw_targets[start:end])
        byte_mask = list(raw_mask[start:end])
        if len(byte_targets) < length:
            byte_targets.extend(input_ids[len(byte_targets):])
        if len(byte_mask) < length:
            byte_mask.extend([0] * (length - len(byte_mask)))

        return {
            "input_ids": torch.tensor(input_ids, dtype=torch.long),
            "labels": torch.tensor(labels, dtype=torch.long),
            "byte_targets": torch.tensor(byte_targets[:length], dtype=torch.long),
            "byte_mask": torch.tensor(byte_mask[:length], dtype=torch.long),
            "length": torch.tensor(length, dtype=torch.long),
        }


def collate_batch(batch: list[dict[str, Tensor]], pad_token_id: int = 0) -> dict[str, Tensor]:
    max_length = max(int(item["length"]) for item in batch)
    batch_size = len(batch)
    input_ids = torch.full((batch_size, max_length), pad_token_id, dtype=torch.long)
    labels = torch.full((batch_size, max_length), -100, dtype=torch.long)
    byte_targets = torch.full((batch_size, max_length), pad_token_id, dtype=torch.long)
    byte_mask = torch.zeros((batch_size, max_length), dtype=torch.long)
    lengths = torch.zeros(batch_size, dtype=torch.long)

    for row, item in enumerate(batch):
        length = int(item["length"])
        lengths[row] = length
        input_ids[row, :length] = item["input_ids"]
        labels[row, :length] = item["labels"]
        byte_targets[row, :length] = item["byte_targets"]
        byte_mask[row, :length] = item["byte_mask"]

    return {
        "input_ids": input_ids,
        "labels": labels,
        "byte_targets": byte_targets,
        "byte_mask": byte_mask,
        "lengths": lengths,
    }
