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

Đã tích hợp `BigramTable` `.vbg` (100.000 bigram, 879 KB, FNV-1a 64-bit) và
dual-path gatekeeper. Tạo lại bảng:

```bash
python neural/data/build_bigram_vbg.py \
  --inputs data_raw/viwiki_clean.txt data_raw/leipzig_news.txt \
  --viet74k data_raw/Viet74K.txt --min-count 2 --max-entries 100000 \
  --out neural/fixtures/vi_bigram.vbg
```

Chạy `eval_cascade` trên 10K test:

```bash
cargo run -p vsm-runtime --features cascade --example eval_cascade --release -- \
  neural/fixtures/vsm_96_4_int8.vsm data_processed/neural_diverse/test.jsonl 10000
```

| Chỉ số | Không bigram | Có `.vbg` |
| --- | --- | --- |
| Cascade recall | 67,6% | **65,6%** |
| Neural-only recall | 42,6% | 40,6% |
| Suggestion Top-1 (span) | 56,6% | 52,9% |
| Record Top-1 | 30,9% | 32,2% |
| Clean FPR (1.976 câu) | 0,25% | **0,05%** |
| FPR corpus (178K token) | ~0,008% | **0,0084%** |
| Adversarial FPR (7.640 token) | 0% | **0%** |

Ngưỡng: `trigger=0.88`, `mean=0.80`, `Tier4 invalid=0.85`, `valid=0.88`.

**Kết luận:** bigram giảm FPR mạnh nhưng **không kéo recall vượt 70%** trên
checkpoint `96/4` hiện tại; trần recall nằm ở độ phủ hint/candidate, không phải
ở gate Viet74K. Vì vậy:

- Giữ **96/4** (258 KB, ~13,5 ms/chunk); **không cần 64/4**.
- **Chưa ship cascade** vì recall 65,6% < 70%.
- Bigram `.vbg` được giữ làm tùy chọn (`run_tier4_with_bigram`) để giảm FPR khi
  cần; `run_tier4` không bigram vẫn tương thích ngược.
- Để đạt 70% cần tăng độ phủ dữ liệu/hint (300K–500K câu gốc, Focal Loss /
  temperature scaling) hoặc hạ launch gate xuống ~65%.

### VSEC — benchmark lỗi người thật

Chuyển `data_raw/vsec.jsonl` thành JSONL benchmark (9.330 câu, 11.180 lỗi) bằng
`neural/data/build_vsec_bench.py`, rồi chạy `eval_cascade` trên checkpoint
`96/4` hiện tại:

| Chỉ số | Giá trị |
| --- | --- |
| Cascade recall | **15,8%** |
| Neural-only recall | 2,9% |
| Suggestion Top-1 | 7,2% |
| Record Top-1 | 1,8% |

Khoảng cách với synthetic test (65,6% recall) cho thấy domain gap rất lớn:
VSEC là lỗi người thật, nhiều ca real-word/ngữ cảnh mà D1 synthetic chưa phủ.

Thử fine-tune thuần VSEC 3 epoch (7.477 train / 1.853 holdout):

- VSEC holdout recall tăng **15,8% → 26,5%**
- Nhưng synthetic recall sụt **65,6% → 43,8%**, clean FPR tăng **0,66%**
- Kết luận: **catastrophic forgetting**; không ship checkpoint này.

Hướng đúng: trộn VSEC với synthetic (ví dụ 10–30% VSEC) thay vì fine-tune thuần,
và giữ VSEC holdout riêng để đo. Các corpus HF (`fsnaix/vietnamese-corpus-large`,
UTS_VLC, VietVault) có thể mở rộng câu gốc, nhưng cần streaming + lọc NFC/HTML
trước khi đưa vào D1.

### Chẩn đoán 100% VSEC holdout (`--dump-trace`)

`eval_cascade --dump-trace` + `audit_vsec_misses.py` trên 2.168 lỗi holdout:

| Nguyên nhân rớt | Tỷ lệ |
| --- | --- |
| `MISS_TIER3_LOW_CONF` | **78,6%** |
| `HIT_WRONG_SUGGESTION` | 9,6% |
| `HIT` | 7,4% |
| `MISS_TIER4_NO_CANDIDATE` | 4,4% |

Phân bố lỗi VSEC:

- **TONE_ONLY 55,8%**, **DIACRITIC_VOWEL 11,5%**
- **CHAR_MISSING_1 9,9%**, **CHAR_EXTRA_1 6,3%**, `SPACE_MERGE_SPLIT` 2,4%
- **74,1%** là real-word (raw và clean đều có trong Viet74K)
- **68,2%** lỗi làm thay đổi độ dài byte (`byte_mask=0`)

Kết luận: nút thắt chính là **Tầng 3 thiếu tự tin trên phân phối lỗi người thật**,
không phải Tầng 4 thiếu ứng viên. D1 cần bổ sung mạnh:
tone 6 mức, biến âm nguyên âm (`a/ă/â`, `e/ê`, `o/ô/ơ`, `u/ư`), thiếu/thừa ký tự,
dính/tách từ; sau đó **mixed training** synthetic + ~15% VSEC train.

### D1 v3 + Mixed Training (600K)

- 600K train: 510K synthetic (42% tone, 18% diacritic vowel, 20% char drop/dup,
  15% phonetic/word, 5% space edit) + **90K VSEC** (15%).
- VSEC holdout 1.866 câu khóa riêng.
- Train `96/4` 5 epoch, BF16, Muon + AdamW.

| Tập | Cascade recall | Top-1 | Clean FPR |
| --- | --- | --- | --- |
| Synthetic v3 (12K) | 56,7% | 11,4% | 0,23% |
| VSEC holdout | **28,5%** (baseline 16,8%) | 13,8% | — |

Synthetic v3 thấp hơn test cũ vì tập mới chứa nhiều lỗi lệch byte/space khó hơn;
VSEC tăng gần 1,7× nhưng vẫn dưới gate 70%. Chưa ship cascade.

Đã sửa thêm panic `is_char_boundary` trong `viterbi_segmenter` khi corpus có
ký tự Unicode mở rộng (`İ`) làm offset lowercase lệch byte.

### Curriculum Learning 2 pha

- Phase 1: 50% D1 cũ + 45% D1 v3 + 5% VSEC, 4 epoch, LR chuẩn.
- Phase 2: 92% D1 v3 + 8% VSEC, 2 epoch, LR 0.15×.
- `neural/data/build_curriculum.py` + `neural/train.py --lr-scale`.

| Tập | Baseline | V3 15% VSEC | Curriculum |
| --- | --- | --- | --- |
| Synthetic cũ (10K) | **65,6%** | 49,0% | **62,6%** |
| Synthetic v3 (12K) | — | 56,7% | 55,6% |
| VSEC holdout | 16,8% | **28,5%** | **25,1%** |
| Clean FPR | 0,25% | 0,23% | **0,10%** |

Curriculum là Pareto tốt hơn V3 (giữ synthetic tốt hơn, VSEC vẫn 25,1%) nhưng
**synthetic 62,6% vẫn dưới gate 65%**. Chưa đủ điều kiện bật cascade mặc định;
có thể cân nhắc cờ optional cho tester nếu chấp nhận synthetic ~62%.

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
