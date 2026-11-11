# Đóng góp cho VWI

## Cài đặt môi trường

```bash
python -m venv .venv
# Windows
.venv\Scripts\activate
# macOS / Linux
source .venv/bin/activate

pip install -e ".[dev,mcp]"
```

## Trước khi push

```bash
pwsh scripts/check.ps1          # fast, < 15s: ruff + build examples
```

Windows PowerShell 5.1 (không có `pwsh`):

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
```

Trước khi mở PR hoặc release:

```bash
pwsh scripts/check.ps1 -Full    # fast + cargo test --all-targets + pytest
```

CI gọi **đúng hai lệnh trên** (`Check (fast)` và `Test (full)`). Không thêm bước
kiểm tra riêng trong `ci.yml`: nếu cần check mới, thêm vào `scripts/check.ps1`
để local và CI không drift.

> **CI không chạy gì khác ngoài `check.ps1`.** Nếu bạn cần thêm check, thêm vào
> script — không thêm step riêng vào `ci.yml`. Nếu CI fail mà local pass, chạy
> `check.ps1 -Full`; nếu vẫn pass, báo maintainer vì script và CI đã drift.

`check.ps1` cần PowerShell 7 (`pwsh`). GitHub-hosted runners (Ubuntu, Windows,
macOS) đều có sẵn; nếu thêm runner mới, cài `pwsh` trước khi gọi script.

## Checklist khi loại bỏ một experiment

- [ ] Xoá example / eval script đi kèm.
- [ ] Xoá data file đi kèm (nếu có).
- [ ] Giữ generator để tái lập, nhưng đảm bảo nó không build vào product.
- [ ] Ghi số đo và lý do loại vào docstring module/test tương ứng.
- [ ] Chạy `scripts/check.ps1` trước khi push.

## Feature `context-lm`

Feature mặc định **tắt**. Khi bật, cần dữ liệu bigram thật:

```
crates/vwi-core/data/bigram_lm.tsv
crates/vwi-core/data/unigram_lm.tsv
```

Hai file này không commit (dữ liệu lớn). Nếu thiếu, `build.rs` tự dùng fixture
nhỏ đã commit (`bigram_lm_test.tsv`, `unigram_lm_test.tsv`) và in cảnh báo.
Fixture chỉ để CI/build sạch compile được, **không dùng cho production**.

Tạo lại fixture từ dữ liệu thật:

```bash
python tools/gen_lm_fixture.py
```

## Benchmark

```bash
python tools/gen_benchmark.py --corpus <dir> --out <dir> --salt <secret>
python tools/predict_benchmark.py --bench <dir>/blind.jsonl --out <dir>/blind_pred.jsonl
python tools/score_benchmark.py --bench <dir>/blind.jsonl --pred <dir>/blind_pred.jsonl
```

- Salt và blind set **không commit**; salt phải giữ ngoài repo.
- Không tune trên blind set. Blind chỉ dùng để đo, một lần cho mỗi phiên bản.
- Baseline sản phẩm trên blind (2026-11, 800 ca): FPR 7/300, detection 182/500,
  sửa đúng 148/500, precision 96,3%. Số chi tiết nằm trong docstring
  `tools/score_benchmark.py`.
