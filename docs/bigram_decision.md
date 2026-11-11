# Bigram LM: từ "chưa đủ" đến "đủ điều kiện" (2026-11)

## Giai đoạn 1 — corpus nội bộ (chưa đủ)

| Chỉ số | Ngưỡng | Thực tế | Đạt? |
|---|---:|---:|---|
| Bigram distinct (count≥2) | ≥150.000 | 71.728 | ❌ |
| OOV bigram rate dev | ≤30% | 32,7% | ❌ |
| Real-word coverage | ≥6/10 | 5/10 | ❌ |

Kết luận khi đó: **chưa viết context spellchecker**.

Phát hiện dữ liệu:
- `human_gold.txt` bị tách phụ âm đầu (`m ột`, `c ầu`); đã repair 11.177 chỗ
  bằng Viet74K (95,8% cặp ghép thành âm tiết hợp lệ).
- `literature_wikisource` và `news_extra` **100% trùng** nguồn khác → dedup loại.
- `ud_text` + RSS VnExpress chỉ thêm ~37k token, không đủ.

## Giai đoạn 2 — bổ sung Wikipedia tiếng Việt (đủ)

Nguồn: dump chính thức `dumps.wikimedia.org/viwiki/20261101/`, shard
`pages-articles2.xml-p832083p1952085.bz2` (122 MB nén, CC BY-SA).
Extract bằng `wikiextractor` → `data_raw/viwiki_clean.txt`
(702.502 dòng, 149,6 MB text).

Sau khi thêm vào pipeline (split 90/10 theo nguồn, dedup toàn cục):

```
train_sents=635374 dev_sents=70594 train_tokens=22138216 dev_tokens=2450465
unigram_distinct=341999

Bigram distinct: count>=1 2,418,848 | count>=2 812,078 |
                 count>=3 503,278  | count>=5 310,353
TSV size (count>=2): 12.24 MB

Perplexity dev: k=0.1 225.83 | k=0.5 517.79 | k=1.0 805.22
OOV bigram rate dev: 10.6%   (20 câu H sạch: 12.8%)
Real-word coverage: 10/10
```

| Chỉ số | Ngưỡng | Thực tế | Đạt? |
|---|---:|---:|---|
| Bigram distinct (count≥2) | ≥150.000 | 812.078 | ✅ |
| OOV bigram rate dev | ≤30% | 10,6% | ✅ |
| Real-word coverage | ≥6/10 | 10/10 | ✅ |
| TSV size | đo thật | 12,24 MB | — |

## Gate trên dev_general (human_gold + journalism)

`dev_hj.txt` = 1.215 dòng / 65.199 token, **không một câu viwiki**:

```
OOV bigram rate (count>=3) = 18.7%   (target <= 20-25%)
unigram coverage           = 99.0%
unigram sau prune: count>=2 146.915 | count>=3 96.886 | count>=5 63.263
perplexity dev_hj:
  k=0.01 -> 828.78 | k=0.05 -> 853.60 | k=0.1 -> 943.10 | k=0.5 -> 1543.45
```

k=0.01 tốt nhất trên dev_hj (khác dev wiki vì register/sparsity). Chọn **k=0.01**.

### Margin 10 ca real-word (trên LM count>=3)

| Ca | Cặp đúng | Cặp sai | score đúng | score sai | margin |
|---|---|---|---|---|---|
| H29 | cơn bão | cơn bảo | -1.28 | -15.00 | 13.72 |
| H30 | đi tắm | đi tám | -5.09 | -15.00 | 9.91 |
| H31 | sau bữa | sao bữa | -7.17 | -15.00 | 7.83 |
| H32 | buổi sáng | buổi sang | -2.24 | -15.00 | 12.76 |
| H33 | bên cạnh | bên canh | -1.87 | -15.00 | 13.13 |
| H34 | chiếc váy | chiếc vay | -3.63 | -15.00 | 11.37 |
| H35 | để nấu | đẻ nấu | -5.45 | -15.00 | 9.55 |
| H36 | rất lạnh | rất nạnh | -6.49 | -15.00 | 8.51 |
| H37 | giấy tờ | dấy tờ | -4.07 | -15.00 | 10.93 |
| H38 | cuốn sách | cuống sách | -4.24 | -10.78 | 6.54 |

mean margin = **10.43**, min = **6.54** → về mặt scoring, LM phân biệt được cả 10.

### Sau 2 fix (word-level tokenizer + bỏ gate quá chặt)

Đã sửa:
- tokenizer riêng cho context module (split từ, không compound) → thấy
  `buổi`/`sang`, `bên`/`canh`;
- bỏ yêu cầu từ gốc phải có mặt trong LM và bỏ gate dấu tiếng Việt (từ ASCII
  như `sang`, `canh`, `vay` vẫn là từ Việt hợp lệ);
- guardrail chỉ kiểm tra **ứng viên** có bigram thật.

Grid lại trên dev_hj:

| min_abs / margin | FP lines / 1215 | H real-word hit (đo thực tế) |
|---|---:|---:|
| -4 / 10 | 17 (1,40%) | **3/10** (H29, H32, H33) |
| -5 / 10 | 65 (5,35%) | — |

Chọn **-4 / 10**: FPR 1,40%, 3/10 real-word. Đây là mức "advisory" có ích nhưng
**không đủ để auto-fix**: 7 ca còn lại vẫn phụ thuộc bảng ngữ cảnh có kiểm soát.
Vì vậy module chỉ `with_note` (gợi ý), **không** tạo AutoFix, và vẫn nằm sau
feature `context-lm` opt-in.

**Kết luận tạm thời: bigram context khả thi ở mức advisory hẹp (3/10, FPR 1,4%).**
Nhưng con số 3/10 là trên H đã biết. Khi đo trên **VSEC** (lỗi người gán tay),
kết quả thấp hơn nhiều:

## Cân bằng register (wiki cap 5M + Leipzig 10M + legal 2M)

Đã thêm:
- `leipzig_news.txt` (997.342 câu, 146 MB; cap 10M token),
- `legal_sample.txt` (46.056 dòng; cap 2M token),
- cap `viwiki_clean.txt` còn 5M token.

Corpus train: **15.908.343 token**, unigram 156.729, bigram count≥3 **447.704**
(6,51 MB). Dev_hj giữ nguyên human_gold+journalism (không trộn nguồn mới).

Kết quả:

```
dev_hj OOV (count>=3) = 17,0%   (trước: 18,7%)
20 câu H sạch OOV     = 4,9%    (trước: 12,8%)
perplexity dev_hj: k=0.01 583,70 | k=0.05 546,22 | k=0.1 563,25 | k=0.5 750,64
```

Perplexity giảm mạnh (828 → 546), H clean OOV giảm 12,8% → 4,9%. Nhưng
dev_hj OOV vẫn 17% vì human_gold là văn phê bình/literary, không giống
Leipzig/legal.

VSEC grid với LM mới:

| min_abs / margin | TP/500 | FP/300 | Recall | FPR | Precision | F1 |
|---|---:|---:|---:|---:|---:|---:|
| -3 / 8 | 16 | 2 | 3,2% | 0,67% | 88,9% | 6,2% |
| -4 / 10 | 45 | 11 | 9,0% | 3,67% | 80,4% | 16,2% |
| -5 / 10 | 82 | 29 | 16,4% | 9,67% | 73,9% | 26,8% |
| -6 / 4 | 146 | 90 | 29,2% | 30,0% | 61,9% | 39,7% |

Vẫn **không có cấu hình nào đạt recall 40–50% ở FPR ≤1,5%**. Cân bằng register
giúp LM tốt hơn rõ rệt (perplexity/H clean), nhưng chưa đủ cho real-word.

## VSEC — dev lỗi gán tay (kết quả quyết định)

VSEC: 9.341 record, 11.202 lỗi; sau khi lọc cả từ sai và từ đúng đều là âm tiết
hợp lệ (real-word đơn từ) còn **7.641 ca**. Đánh giá trên 500 ca + 300 dòng dev_hj:

| min_abs / margin | TP/500 | FP/300 | Recall | FPR | Precision | F1 |
|---|---:|---:|---:|---:|---:|---:|
| **-4 / 10** | 39 | 5 | **7,8%** | **1,67%** | 88,6% | 14,3% |
| -5 / 10 | 72 | 21 | 14,4% | 7,00% | 77,4% | 24,3% |
| -6 / 2 | 196 | 152 | 39,2% | 50,67% | 56,3% | 46,2% |

Không có cấu hình nào đạt **recall 50–60% ở FPR ≤1,5%**. Muốn recall cao phải
chấp nhận FPR 30–50% — không dùng được.

**Kết luận cuối: context bigram chưa đủ để ship, kể cả advisory.**
3/10 trên H là may mắn trên tập nhỏ đã biết, không phải năng lực tổng quát.
Module giữ sau feature `context-lm`, mặc định tắt, chỉ để tiếp tục nghiên cứu.

Bù lại, trong lúc làm đã phát hiện và sửa bug thật: `place_vietnamese_tone_decree20`
đặt sai dấu cho vần mở `ao/au/eo/iu` (`bảo` → `baỏ` thay vì `bảo`), khiến
`bảo ↔ bão` không bao giờ là ứng viên. Đã thêm rule + unit test.

## Quyết định (cập nhật)

- Giữ phonetic OOV spellchecker (6 ca H, 0 FPR).
- Giữ bảng ngữ cảnh có kiểm soát (`confusion.rs`) cho real-word.
- Context bigram: **không ship**, kể cả advisory. VSEC với LM cân bằng:
  recall 3,2% ở FPR 0,67% (an toàn) hoặc 9,0% ở FPR 3,67% — quá thấp so với
  mục tiêu 40–50% ở FPR ≤1,5%.
- Feature `context-lm` mặc định tắt, giữ cho nghiên cứu; không AutoFix.
- Muốn mở lại: cần LM cân bằng register (báo chí/hành chính) + dev lỗi gán tay
  lớn hơn, rồi đo lại recall/FPR trước khi bật.

Các bước cũ (đã hoàn thành một phần):

1. Chọn bản prune (count≥2: 812k/12,2 MB; count≥3: 503k/~7,5 MB) sau khi đo
   trade-off coverage/kích thước.
2. Viết loader Rust đọc TSV qua `include_str!` + `LazyLock`, interning từ nếu
   cần; hoặc build binary format u32 nếu kích thước WASM là ưu tiên.
3. Context spellchecker: backoff unigram + **observed-side guardrail** +
   `MARGIN`/`MIN_ABS` chọn trên dev sạch (không dùng H).
4. Tạo dev lỗi tổng hợp từ dev (thay từ bằng ứng viên phonetic) để đo recall.
5. Chỉ sau đó chạy blind H2 và báo `FPR ?/20`, `Detection ?/30`, `★ ?/10`.

Lưu ý pháp lý: viwiki là **CC BY-SA** — cần ghi attribution cho dữ liệu/nếu
phân phối lại corpus hoặc mô hình dẫn xuất. Không commit dump/thư mục extract
vào repo (`data_raw/` đã ignore; `data_processed/` cũng đã ignore).

## Cảnh báo: LM bị lệch register về Wikipedia

Sau khi thêm viwiki, **~97% token train đến từ Wikipedia** (21,5M/22,1M).
Top bigram phản ánh văn phong bách khoa: `trong họ`, `một loài`, `loài này`,
`miêu tả khoa học`... Vì vậy:

- OOV 10,6% trên dev wiki; trên **dev_hj (human_gold+journalism) là 18,7%** —
  con số đại diện văn xuôi thường.
- `MARGIN`/`MIN_ABS` của context spellchecker **phải chọn trên dev
  general** (human_gold/journalism), không phải dev viwiki.
- Nếu muốn LM cân bằng hơn: cap viwiki còn ~5M token hoặc weight theo nguồn.
  Chưa làm vì coverage real-word đã 10/10 và mục tiêu là common collocations.

## Chọn mức prune

| Prune | Số cặp | Kích thước | H clean OOV | Coverage |
|---|---:|---:|---:|---:|
| count≥2 | 812.078 | 11,46 MB | 12,8% | 10/10 |
| count≥3 | 503.278 | 6,96 MB | 15,9% | 10/10 |
| count≥5 | 310.353 | 4,34 MB | 18,3% | 10/10 |

Khuyến nghị: **một đường duy nhất count≥3** (6,96 MB) cho cả desktop và WASM.
Coverage vẫn 10/10, OOV dev_hj 18,7% (dưới ngưỡng), một bộ test/một MARGIN.
Nếu sau này desktop cần thêm, mở feature `bigram-full` với count≥2 — không
phải bây giờ.

## Ghi chú kỹ thuật

- Lần đầu, baseline POS C rơi xuống 26,20% vì xác định OOV theo **tag** thay vì
  theo **từ**, khiến suffix (scale khác) thắng oan. Đã sửa; xem
  `docs/pos_decision.md`.
- Không thêm tay 10 cặp real-word vào LM; coverage 10/10 hiện tại là do corpus,
  không phải do tune trên H.
