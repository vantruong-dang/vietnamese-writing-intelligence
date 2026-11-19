# VSM — Vietnamese Spelling Model (D1 / D2 / D3 / D4)

Gói thực nghiệm mô hình byte-level cho tiếng Việt, tách khỏi extension đang phát hành.
Không thêm dependency vào runtime hiện tại; PyTorch chỉ nằm trong optional group `neural`.

## Bản chốt D2 (Phương án A)

| Hạng mục | Thông số |
| --- | --- |
| Embedding | `256 x 160`, INT8 per-row |
| Stem | `Conv1D(160, 160, k=5)` + SiLU |
| Block | `DWConv(160, k=5)` + RMSNorm + `PW(160->320)` + ReLU + `PW(320->160)` + residual |
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
`bench_corpus` + **`viwiki_clean.txt` và `leipzig_news.txt`** (mỗi file tối đa
`max_sentences / số file` để tránh một nguồn lấn át). Câu được chia theo **nguồn**
80/10/10 cho train/val/test để tránh leakage. Các toán tử:

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

`byte_mask=0` ở các vị trí không căn chỉnh được. Quy tắc an toàn:
`byte_targets` chỉ được bật khi độ dài byte của đoạn input và đoạn clean **bằng nhau**
(ví dụ `sử ↔ xử`, Telex `đnag ↔ đang`). Các cặp lệch byte như
`nghiêng (8 byte) ↔ nghiên (7 byte)` hoặc chèn/xóa chỉ học `head_err`;
`head_byte` bỏ qua toàn bộ span để không học lệch vị trí. Tầng 4 sẽ xử lý các
trường hợp lệch byte bằng biến thể âm vị/thanh điệu + `Viet74K`.

## D3 — Huấn luyện

```bash
pip install -e ".[neural]"
# 500K–1M câu: ưu tiên viwiki_clean + leipzig_news, giữ 10K câu sạch làm holdout.
python -m neural.data.generate_errors \
  --out data_processed/neural_large \
  --train-size 500000 --val-size 10000 --test-size 10000
python -m neural.train \
  --train data_processed/neural_large/train.jsonl \
  --val data_processed/neural_large/val.jsonl \
  --out neural/checkpoints/vsm_96_4.pt \
  --d-model 96 --n-layers 4 --dilations 1,2,4,8 \
  --epochs 5 --batch-size 128 --grad-accum 2 --amp bf16 --device cuda
```

Trên GPU 4 GB (Gigabyte G5 GD), dùng `--batch-size 64 --grad-accum 4 --amp bf16`
nếu OOM. Sau mỗi epoch nên chấm FPR trên 10K câu sạch + từ vựng kỹ thuật/tên riêng;
chỉ chấp nhận checkpoint có FPR span ≤ 0.5% trước khi export.

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

## Cascade với `vwi-core`

`vwi-core::neural_cascade` định nghĩa:

- `NeuralSpanHint` (byte span + error kind + confidence).
- `NeuralContextScorer` (Tầng 3 trả hint; Tầng 4 lọc).
- `hints_from_byte_probs` (trigger 0.92, pool trung bình 0.85).
- `run_tier4` (snap token, Viet74K, bigram context, raw-valid cần 0.96).
- `NeuralDiagnostic::to_finding` (severity advisory, rule `NEURAL_CONTEXT_SPELLING`).

Runtime implement trait qua feature `cascade`:

```bash
cargo test -p vsm-runtime --features cascade
```

```rust
use vsm_runtime::VsmCascadeScorer;
use vwi_core::{run_tier4, CascadeConfig};

let scorer = VsmCascadeScorer::new(&vsm_bytes)?;
let mut hints = Vec::new();
scorer.score_unmasked_regions(text, &locked_mask, &mut hints);
let diagnostics = run_tier4(text, &hints, &locked_mask, CascadeConfig::default());
let findings: Vec<_> = diagnostics.iter().map(|d| d.to_finding(text)).collect();
```

Tầng 4 chỉ xuất diagnostic khi có ứng viên qua `Viet74K`; tên riêng/từ mượn
(`Docker`, `B12`...) bị `DROPPED` im lặng.

## Benchmark latency (wasm32 + simd128)

Harness:

```bash
wasm-pack build crates/vsm-wasm --target nodejs --out-dir pkg --release
python -m neural.export.make_fixture --full \
  --out neural/fixtures/full_random.vsm \
  --expected neural/fixtures/full_random_expected.txt
node neural/bench_wasm.cjs
```

Kết quả đo trên Node/V8 (máy có tải nền, nên dùng **tỷ lệ tương đối**; chạy lại
trên máy trống để lấy số tuyệt đối):

| Cấu hình | 2.000 byte (16 chunk) | ~ms/chunk | Ghi chú |
| --- | --- | --- | --- |
| 160/6 (bản chốt) | ~0,87–1,3 s (best, ít tải) | ~55–80 | Dùng cho batch/offline |
| 128/6 | ~0,9–1,6 s | ~55–100 | Không cải thiện rõ |
| 96/4, dilations [1,2,4,8] | ~0,20 s | **~12–14** | Sát ngưỡng, nên chấm incremental |
| 64/4, dilations [1,2,4,8] | ~0,084 s | **~5,3** | Đạt <10 ms/chunk |

Các tối ưu đã áp dụng:

- Dequantize INT8 → F32 một lần lúc `VsmModel::from_bytes`.
- Pack trọng số Pointwise thành `[out/4][in][4]` (Pack-4).
- Micro-kernel 4×4: 4 output channel × 4 timestep, không transpose, không horizontal sum.
- Fuse `PW2 + residual`; đổi GELU → ReLU để bỏ `tanh`/`exp` trên wasm.

**Kết luận:** `96/4` đạt ~12–14 ms/chunk (sát ngưỡng), `64/4` đạt ~5,3 ms/chunk.
Vẫn nên chấm incremental từng câu trong Web Worker; nếu cần chấm cả 2.000 byte
dưới 10 ms tuyệt đối, dùng `64/4` hoặc WebGPU/WebNN.

## Kết quả train `96/4` trên RTX 3050 (4 GB)

- Dữ liệu: 500K train / 10K val / 10K test (seed 43), sinh từ **63.184 câu**
  `viwiki_clean + leipzig_news + corpus hiện có`, chia câu theo nguồn 80/10/10.
- Train: 5 epoch, batch 128, grad-accum 2, BF16, Muon + AdamW.
- Val: precision 0.792, recall 0.846, **F1 0.818**, byte accuracy 0.993.
- Test FP32: precision 0.805, recall 0.848, **F1 0.826**, byte accuracy 0.993.
- Test INT8 `.vsm`: precision 0.798, recall 0.838, **F1 0.817**
  (chỉ giảm ~0.9 điểm sau lượng tử hóa).
- FPR cascade (Tier 3 + Tier 4, Viet74K):
  - `journalism`: 0/70.861 token
  - `legal`: 0/63.959 token
  - `news`: 0/44.181 token
  - Tổng: **0/178.001 token**, 0 câu bị báo oan trên 6.000 câu.

Artifact: `neural/fixtures/vsm_96_4_int8.vsm` (**258 KB**, 39 tensor).

Checkpoint FP32: `neural/checkpoints/vsm_96_4_diverse.pt` (không commit, `*.pt` bị ignore).

### Đo lại artifact đã train

```bash
cargo run -p vsm-runtime --example eval_int8 --release -- \
  neural/fixtures/vsm_96_4_int8.vsm data_processed/neural_diverse/test.jsonl 10000

cargo run -p vsm-runtime --features cascade --example fpr --release -- \
  neural/fixtures/vsm_96_4_int8.vsm data_processed/bench_corpus/news.txt 2000

node neural/bench_wasm.cjs neural/fixtures/vsm_96_4_int8.vsm
```

Benchmark hiện tại (máy có tải nền): ~215–230 ms/2.000 byte ≈ **13–14 ms/chunk**.
`64/4` vẫn là lựa chọn an toàn hơn cho mục tiêu <10 ms/chunk.

## Audit End-to-End trước khi ghép `vwi-wasm`

Chạy `eval_cascade` trên 10K test (Tier 2 orthography + Tier 3 + Tier 4):

```bash
cargo run -p vsm-runtime --features cascade --example eval_cascade --release -- \
  neural/fixtures/vsm_96_4_int8.vsm data_processed/neural_diverse/test.jsonl 10000
```

| Chỉ số | Giá trị |
| --- | --- |
| Gold spans | 8.024 |
| Cascade recall (Tier 2 + Tier 4) | **67,6%** |
| Neural-only recall | 42,6% |
| Suggestion Top-1 (span) | **56,6%** |
| Record Top-1 (cả câu) | 30,9% |
| Clean FPR (1976 câu sạch) | **0,25%** |

Ngưỡng hiện tại: `trigger=0.92`, `mean=0.85`, `valid_raw=0.90`.
Nếu hạ `valid_raw` xuống 0.85: recall tăng lên **69,5%** nhưng clean FPR lên **0,61%**
(vượt gate 0,5%). Vì vậy **chưa đạt launch gate recall ≥70%**; không nên ghép cascade
vào bản phát hành ở thời điểm này.

### Adversarial FPR (chat/code-switching)

Bộ 600 câu chat/forum trộn Anh–Việt, tên riêng, slang (`ko`, `đc`, `fb`, `sg`),
code/deadline/merge:

- **0/7.640 token** bị báo oan (0%) sau khi Tier 4 bỏ qua token số/mã và slang.
- Gate đề xuất ≤2%: **đạt**.

### Quyết định

- Giữ **96/4** (258 KB, ~13,5 ms/chunk); không cần hạ `64/4` vì 96/4 đã đủ nhanh
  khi chấm incremental trong Web Worker và có recall tốt hơn.
- **Chưa ship cascade**: recall 67,6% < 70%. Cần một trong hai:
  1. Tăng recall Tier 4 (ưu tiên candidate từ byte head, mở rộng ngữ cảnh bigram),
     rồi đo lại trên cùng test set.
  2. Hạ launch gate recall xuống ~65% và chấp nhận FPR 0,25% — chỉ nên làm nếu
     người dùng chấp nhận advisory nhiều hơn.
- `64/4` chỉ đáng thử nếu sau khi train, cascade recall giảm **<2 điểm** so với 96/4.

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
