"""Cấu hình D2/D3 cho mô hình VSM (Vietnamese Spelling Model).

Bản chốt Phương án A:
- Embedding(256, 160) INT8 per-row
- Stem Conv1D(160, 160, k=5) + SiLU
- 6 x DW-Sep Block: DW(160, k=5) + RMSNorm + PW(160->320) + ReLU + PW(320->160)
  với dilations [1, 2, 4, 8, 4, 2]
- Head_err Linear(160, 4), Head_byte Linear(160, 256)

RF: bán kính 44 byte mỗi phía, tổng 89 byte. Chunk stride = 128 byte.
"""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass(frozen=True)
class ModelConfig:
    vocab_size: int = 256
    d_model: int = 160
    n_blocks: int = 6
    kernel_size: int = 5
    dilations: tuple[int, ...] = (1, 2, 4, 8, 4, 2)
    expand: int = 2
    n_error_classes: int = 4
    max_seq_len: int = 256
    rms_eps: float = 1e-5

    def __post_init__(self) -> None:
        if self.vocab_size != 256:
            raise ValueError("VSM byte-level yêu cầu vocab_size = 256.")
        if self.d_model % 16 != 0:
            raise ValueError("d_model phải chia hết cho 16 để căn lề SIMD.")
        if self.kernel_size % 2 == 0:
            raise ValueError("kernel_size phải là số lẻ.")
        if len(self.dilations) != self.n_blocks:
            raise ValueError("Số dilations phải bằng n_blocks.")
        if self.expand < 1:
            raise ValueError("expand phải >= 1.")

    @property
    def rf_radius_bytes(self) -> int:
        """Bán kính receptive field mỗi phía (byte).

        Mỗi lớp Conv1D(k=5, d) đóng góp d * (k - 1) / 2 = 2d byte.
        Stem d=1, các block d=[1,2,4,8,4,2].
        """
        per_side = self.kernel_size // 2
        return per_side * (1 + sum(self.dilations))

    @property
    def rf_bytes(self) -> int:
        return 2 * self.rf_radius_bytes + 1

    @property
    def chunk_stride(self) -> int:
        """Stride an toàn: 128 byte, overlap 128 > RF 89."""
        stride = 128
        if stride + 2 * self.rf_radius_bytes > self.max_seq_len:
            raise ValueError("max_seq_len quá nhỏ so với RF; cần >= stride + 2*RF.")
        return stride

    @property
    def expanded_dim(self) -> int:
        return self.d_model * self.expand

    def estimated_parameters(self) -> int:
        embedding = self.vocab_size * self.d_model
        stem = self.d_model * self.d_model * self.kernel_size
        per_block = (
            self.d_model * self.kernel_size
            + self.d_model
            + self.expanded_dim * self.d_model
            + self.d_model * self.expanded_dim
        )
        heads = self.d_model * self.n_error_classes + self.d_model * self.vocab_size
        return embedding + stem + self.n_blocks * per_block + heads


@dataclass(frozen=True)
class TrainConfig:
    batch_size: int = 32
    epochs: int = 20
    max_seq_len: int = 256
    learning_rate: float = 3e-3
    muon_lr: float = 2e-2
    muon_momentum: float = 0.95
    muon_ns_steps: int = 5
    weight_decay: float = 0.01
    warmup_ratio: float = 0.05
    grad_clip: float = 1.0
    error_loss_weight: float = 1.0
    byte_loss_weight: float = 1.0
    num_workers: int = 0
    seed: int = 42
    device: str = "cpu"
    use_amp: bool = False
    amp_dtype: str = "bf16"
    grad_accum_steps: int = 1
    checkpoint_dir: str = "neural/checkpoints"
    log_every: int = 50
    val_every: int = 1
    label_smoothing: float = 0.0
    extra: dict[str, float] = field(default_factory=dict)


DEFAULT_MODEL_CONFIG = ModelConfig()
DEFAULT_TRAIN_CONFIG = TrainConfig()
