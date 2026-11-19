# VSM — Vietnamese Spelling Model (D1 / D2 / D3 / D4)

Gói thực nghiệm mô hình byte-level cho tiếng Việt, tách khỏi extension đang phát hành.
Không thêm dependency vào runtime hiện tại; PyTorch chỉ nằm trong optional group `neural`.

## Bản chốt D2 (Phương án A)

| Hạng mục | Thông số |
| --- | --- |
| Embedding | `256 x 160`, INT8 per-row |
| Stem | `Conv1D(160, 160, k=5)` + SiLU |
| Block | `DWConv(160, k=5)` + RMSNorm + `PW(160->320)` + GELU(tanh) + `PW(320->160)` + residual |
| Số block | 6, dilations `[1, 2, 4, 8, 4, 2]` |
| Heads | `Head_err(4)`, `Head_byte(256)` |
| Params | ~834K |
| RF | bán kính 44 byte/phía, tổng 89 byte (~19 âm tiết) |
| Chunk | `max_seq_len=256`, `stride=128` (overlap 128 > RF 89) |
| Nhãn | per-byte span: `0=CLEAN, 1=PHONETIC, 2=TONE, 3=WORD` |

### Vì sao stride = 128?

Mỗi lớp `Conv1D(k=5, d)` mở rộng bán kính `d * (k-1)/2 = 2d` byte.
Stem `d=1`; 6 block có tổng `sum(dilations)=21`:

```text
radius = 2 * (1 + 21) = 44 bytes
RF     = 2 * 44 + 1 = 89 bytes
```

Với `max_seq_len=256`, stride 128 cho overlap 128 byte mỗi phía > 44 byte RF,
nên không mất ngữ cảnh ở rìa chunk. Đoạn 2.000 byte cần ~15 chunk.

> Lưu ý: `Promise.all` trên các hàm wasm đồng bộ **không** tạo song song trên
> main thread nếu không có `SharedArrayBuffer`/COOP+COEP. Hãy chạy trong Web Worker
> và chỉ chạy lại các câu/đoạn vừa sửa (incremental diffing) để giữ latency thấp.

## D1 — Sinh dữ liệu lỗi

```bash
python -m neural.data.generate_errors \
  --out data_processed/neural \
  --train-size 20000 --val-size 2000 --test-size 2000
```

Nguồn mặc định: `Viet74K.txt` + `human_gold/journalism_corpus/news_fresh` +
`bench_corpus`. Các toán tử:

- **phonetic**: s/x, tr/ch, l/n, r/d/gi/v, n/ng, t/c, iêu/ưu... (biến thể phải là âm tiết có thật).
- **tone**: hỏi/ngã.
- **word**: cặp từ thật dùng nhầm (`sát nhập -> sáp nhập`, `chuẩn đoán -> chẩn đoán`...).
- **ime**: transpose/duplicate/delete/insert ký tự.
- **clean**: giữ nguyên, nhãn 0.

Mỗi record JSONL:

```json
{
  "id": "...",
  "text": "...",
  "clean_text": "...",
  "spans": [{"start": 12, "end": 15, "label": 1}],
  "byte_targets": [ ... ],
  "byte_mask": [ ... ],
  "meta": {"operator": "phonetic", "source": "corpus"}
}
```

`byte_mask=0` ở các vị trí không căn chỉnh được (insert/delete); loss `head_byte`
bỏ qua các vị trí này. Với lỗi thêm/xóa, mô hình vẫn học phát hiện span nhờ
`head_err`, còn `head_byte` chỉ học các vị trí thay thế căn chỉnh được.

## D3 — Huấn luyện

```bash
pip install -e ".[neural]"
python -m neural.train \
  --train data_processed/neural/train.jsonl \
  --val data_processed/neural/val.jsonl \
  --out neural/checkpoints/vsm_d2.pt \
  --epochs 20 --batch-size 32 --device cuda
```

- **Muon**: `stem.weight` và toàn bộ `pw1/pw2.weight` (các ma trận trộn kênh).
- **AdamW**: embedding, depthwise conv, RMSNorm, `head_err`, `head_byte`, bias.
- Loss: `CE(error_head) + CE(byte_head) * byte_mask`.
- Scheduler: warmup tuyến tính + cosine.
- Checkpoint lưu `model`, `config`, `metrics`, `epoch`.

`Muon` nằm trong `neural/muon.py`, dùng Newton-Schulz bậc 5, không cần package ngoài.

## D4 — Export `.vsm` INT8 + parity

```bash
python -m neural.export.export_vsm \
  --checkpoint neural/checkpoints/vsm_d2.pt \
  --out neural/fixtures/vsm_d2_int8.vsm \
  --manifest neural/fixtures/vsm_d2_manifest.json
```

Exporter tự chạy parity PyTorch FP32 vs reference INT8 trên một câu mẫu và ghi
`max_error_logit_delta`, `byte_argmax_match` vào manifest.

### Định dạng `.vsm` v1

Header 48 byte (little-endian):

| Offset | Kiểu | Trường |
| --- | --- | --- |
| 0 | `[u8;4]` | magic `VSM1` |
| 4 | `u16` | version = 1 |
| 6 | `u16` | flags (bit0 = INT8) |
| 8 | `u32` | vocab_size |
| 12 | `u32` | d_model |
| 16 | `u32` | n_blocks |
| 20 | `u32` | kernel_size |
| 24 | `u32` | expand |
| 28 | `u32` | n_error_classes |
| 32 | `u32` | max_seq_len |
| 36 | `u32` | rf_radius_bytes |
| 40 | `u32` | tensor_count |
| 44 | `u32` | reserved |

Mỗi tensor descriptor:

```text
name_len u16 | name bytes | dtype u8 (0=F32, 1=I8) | ndim u8
shape u32[ndim] | scale f32 | zero_point i32 | data_offset u64 | data_len u64
```

Sau descriptor, align 16 byte rồi tới data section. Mỗi tensor data cũng align 16.
INT8 dùng tensor `*.scales` companion (per-output-channel hoặc per-row embedding).
F32 dùng trực tiếp, không cần scales.

Tên tensor:

```text
embedding.weight, embedding.scales
stem.weight, stem.scales
blocks.{i}.dw.weight, blocks.{i}.dw.scales
blocks.{i}.norm.weight
blocks.{i}.pw1.weight, blocks.{i}.pw1.scales
blocks.{i}.pw2.weight, blocks.{i}.pw2.scales
head_err.weight, head_err.scales, head_err.bias
head_byte.weight, head_byte.scales, head_byte.bias
model.dilations
```

## Rust runtime

```bash
cargo test -p vsm-runtime
```

- `crates/vsm-runtime/src/format.rs`: parser `.vsm`, validate magic/version/shape.
- `crates/vsm-runtime/src/kernel/conv1d.rs`: `conv1d_dw_w8a32`, `conv1d_pw_w8a32`,
  `conv1d_dense_w8a32`; dùng `wide::f32x4` (map xuống `simd128` trên wasm32).
- `crates/vsm-runtime/src/model.rs`: forward + chunked inference stride 128.
- `crates/vsm-runtime/tests/parity.rs`: đọc fixture INT8 và so logits với reference Python.

Fixture nhỏ (`neural/fixtures/tiny_model.vsm`, d_model=8, 2 block) được sinh
không cần torch:

```bash
python -m neural.export.make_fixture
```

## Cấu trúc

```text
neural/
  config.py              # ModelConfig / TrainConfig
  model.py               # PyTorch D2
  muon.py                # Muon + phân nhóm tham số
  dataset.py             # byte-level dataset + collate
  train.py               # vòng huấn luyện
  data/generate_errors.py# D1
  export/vsm_format.py   # writer/reader .vsm
  export/reference.py    # forward thuần Python
  export/make_fixture.py # fixture parity
  export/export_vsm.py   # D4 exporter
  fixtures/              # fixture INT8 + expected
crates/vsm-runtime/      # parser + kernel + forward Rust
```

## Giới hạn hiện tại

- `.vsm` runtime Rust hiện chạy INT8; F32 chỉ dùng cho debug/parity qua Python.
- `head_byte` không học vị trí chèn/xóa (mask 0); các lỗi này dựa vào span detection
  và tầng sửa lỗi từ điển/Tier 4.
- Latency cần đo thực tế trên wasm32 SIMD; nếu vượt 25 ms/chunk, giảm `d_model`
  xuống 128 (params ~570K) trước khi tăng `max_seq_len`.
