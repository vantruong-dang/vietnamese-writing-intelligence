# Quyết định: KHÔNG port Viterbi HMM cho POS (2026-11)

## Bối cảnh

Trước khi thay tagger lexicon hiện tại bằng Viterbi HMM, đo 3 baseline trên
UD_Vietnamese-VTB (dữ liệu trong `data_raw/`):

- **A — most-frequent tag**: tương đương lexicon hiện tại.
- **B — HMM Viterbi**, emission add-α.
- **C — HMM + suffix OOV** (backoff hậu tố 3→2→1).

Script: `tools/pos_eval.py`.
Split: gộp train+dev rồi chia 80/20 theo câu (seed 42).
Kích thước: 2.019 câu train / 504 câu dev; 36.778 / 9.599 token.

## Kết quả (tagset 13 nhãn — đúng mapping của repo)

| Baseline | Accuracy tổng | Accuracy từ đa nghĩa | n_amb |
|---|---:|---:|---:|
| A most-frequent | **89.32%** | 64.95% | 1.241 |
| B HMM (α=1) | 86.17% | 65.03% | 1.241 |
| C HMM + suffix (α=1) | 87.40% | 65.91% | 1.241 |
| C HMM + suffix (α=0.1) | 88.51% | **66.16%** | 1.241 |
| C HMM + suffix (α=0.01) | 88.52% | 65.67% | 1.241 |

Tagset 17 nhãn (UPOS gốc, không gộp PROPN/AUX):

| Baseline | Accuracy tổng | Accuracy từ đa nghĩa |
|---|---:|---:|
| A most-frequent | 87.51% | 62.87% |
| B HMM | 85.58% | 67.17% |
| C HMM + suffix | 87.00% | 67.68% |

## Kết luận

- **C không thắng A về accuracy tổng** (thấp hơn 0.8–1.9 điểm), kể cả khi
  chỉnh α.
- Trên **từ đa nghĩa**, C chỉ hơn A ~1.2 điểm (tagset 13) hoặc ~4.8 điểm
  (tagset 17), dưới ngưỡng 5 điểm đã đặt ra.
- Theo tiêu chí "chỉ port nếu thắng ≥ 5 điểm cả tổng lẫn đa nghĩa":
  **KHÔNG port Viterbi HMM**.

## Hệ quả

1. Giữ tagger lexicon + heuristic hiện tại. Không thêm dependency vào Viterbi.
2. Không dùng POS để sửa real-word (`bảo/bão`, `sang/sáng`). Tầng đó cần
   bigram từ vựng/corpus lớn hơn.
3. Classifier (`cây chó → con chó`) giữ là **lexicon lớp ngữ nghĩa + rule**,
   không trộn vào POS.
4. POS hiện chỉ nên dùng cho các pattern nông, có đo lường:
   `ADVERB_PLACEMENT` (Adj + rất → rất + Adj), `SUBJECT_VERB_GAP`,
   `LIST_STRUCTURE_MISMATCH`, `CLASSIFIER_MISMATCH` — tất cả `Advisory`.
5. Muốn POS thực sự đóng góp: cần corpus gán nhãn nội bộ 200–500 câu
   (hành chính/báo chí) và tagset mịn hơn (classifier, modal, temporal adverb).

## Ghi chú kỹ thuật

Lần chạy đầu, baseline C cho 26.20% vì lỗi xác định OOV theo **tag** thay vì
theo **từ**: tag chưa thấy từ dùng suffix (scale khác) và thắng oan. Sửa bằng
cách xác định OOV ở mức từ (`word not in word_tag`) trước khi chọn emission.
Đây đúng là loại lỗi reviewer cảnh báo.
