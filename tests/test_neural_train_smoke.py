import json
from pathlib import Path

import pytest

torch = pytest.importorskip("torch")

from neural.config import ModelConfig  # noqa: E402
from neural.export.export_vsm import export_checkpoint  # noqa: E402
from neural.export.vsm_format import read_vsm  # noqa: E402
from neural.model import VSMModel  # noqa: E402
from neural.muon import Muon, split_muon_adamw_parameters  # noqa: E402
from neural.train import compute_loss  # noqa: E402


def _tiny_model() -> VSMModel:
    config = ModelConfig(
        d_model=16,
        n_blocks=2,
        kernel_size=3,
        dilations=(1, 2),
        max_seq_len=16,
    )
    return VSMModel(config)


def test_one_muon_adamw_step() -> None:
    model = _tiny_model()
    torch.manual_seed(0)
    input_ids = torch.randint(0, 256, (2, 16), dtype=torch.long)
    labels = torch.zeros((2, 16), dtype=torch.long)
    labels[:, 5:7] = 1
    byte_targets = input_ids.clone()
    byte_mask = torch.zeros_like(input_ids)
    byte_mask[:, 5:7] = 1
    batch = {
        "input_ids": input_ids,
        "labels": labels,
        "byte_targets": byte_targets,
        "byte_mask": byte_mask,
    }

    muon_params, adamw_params = split_muon_adamw_parameters(model)
    assert muon_params
    assert adamw_params
    muon = Muon(muon_params, lr=1e-3)
    adamw = torch.optim.AdamW(adamw_params, lr=1e-3)

    class _Config:
        error_loss_weight = 1.0
        byte_loss_weight = 1.0
        label_smoothing = 0.0

    loss, error_loss, byte_loss = compute_loss(model, batch, _Config())  # type: ignore[arg-type]
    assert torch.isfinite(loss)
    assert torch.isfinite(error_loss)
    assert torch.isfinite(byte_loss)
    loss.backward()
    muon.step()
    adamw.step()


def test_export_checkpoint_roundtrip(tmp_path: Path) -> None:
    model = _tiny_model()
    checkpoint = tmp_path / "checkpoint.pt"
    torch.save({"model": model.state_dict(), "config": model.config_dict()}, checkpoint)
    output = tmp_path / "tiny_export.vsm"
    manifest = export_checkpoint(checkpoint, output, skip_parity=False)
    assert output.exists()
    parsed = read_vsm(output)
    assert parsed.config.d_model == 16
    assert parsed.config.n_blocks == 2
    assert manifest["parity"]["checked"] is True  # type: ignore[index]
    json.dumps(manifest, ensure_ascii=False)
