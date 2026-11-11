# CONTEXT HANDOFF — VWI (compact)

Cập nhật: 2026-11. Tất cả thay đổi đang **uncommitted**.

## 1. Mục tiêu

Linter tiếng Việt (Rust core + CLI + WASM/extension) đạt FPR thấp trước khi
publish, xử lý được: chính tả OOV, real-word, hành chính Nghị định 30,
teencode, POS nông.

## 2. Trạng thái test hiện tại

| Bộ | Kết quả | Ghi chú |
|---|---:|---|
| `cargo test --workspace` | **0 failed** | default build |
| `golden_test_v2` (106 ca) | FPR 0/31, detection/fix 75/75, ★17/17 | dev set, không blind |
| `holdout_h` (50 ca) | FPR 0/20, detection 30/30, ★10/10 | H đã biết; bigram đóng góp 0–3 |
| Baseline H trước remediation | 5/30 (16.7%), ★0/10 | con số generalization thật duy nhất |

Tầng phát hiện trên dev 106: regex 37, Aho 32, orthography 5, spellcheck 4,
POS 1. Trên H: Aho 15, regex 9, spellcheck 6, POS 2, khác 4.

## 3. Quyết định đã chốt

1. **POS: đóng, không port Viterbi.** Đo trên UD_Vietnamese-VTB (13 nhãn):
   A most-frequent 89.32/64.95, B HMM 86.17/65.03, C HMM+suffix 87.40/65.91.
   C thua A về tổng, ambig chỉ +1. Xem `docs/pos_decision.md`.
2. **admin_format gate theo `Register::Administrative`** để không bắt oan
   văn xuôi/báo chí.
3. **Bigram LM data gate: ĐẠT.** 22,1M token train, 812k bigram count≥2,
   OOV dev_hj 18,7%, coverage 10/10.
4. **Context bigram spellchecker: KHÔNG ship.** Feature `context-lm` opt-in,
   mặc định tắt, không AutoFix. Lý do: VSEC recall 7,8% ở FPR 1,67%
   (cấu hình -4/10), dưới xa mục tiêu 50–60%. Xem `docs/bigram_decision.md`.
5. **Giữ:** phonetic OOV spellchecker, confusion lexicon có kiểm soát,
   POS nông advisory (adverb placement, classifier mismatch), teencode advisory.

## 4. File/thay đổi chính

### Sửa
- `crates/vwi-core/Cargo.toml` — feature `context-lm = []`.
- `src/auditor.rs` — wiring rules mới, admin register gate, context-lm cfg.
- `src/lib.rs`, `src/rules/mod.rs`, `src/segmentation/mod.rs` — exports/cfg.
- `src/types.rs` — `Finding.stage: Option<String>` + `with_stage`.
- `src/rules/phonotactics.rs` — sửa đặt dấu vần mở `ao/au/eo/iu`
  (`bảo → bão`), + unit test.
- `src/rules/orthography.rs` — skip sentence-cap cho accepted fragments.
- `src/rules/translationese.rs` — miễn trừ "trong vòng 24 giờ".
- `src/rules/engine.rs` — test admin gating.

### Module mới
`admin_format.rs`, `real_word.rs`, `trap_whitelist.rs`, `teencode.rs`,
`pos_structure.rs`, `tense.rs`, `confusion.rs`, `spellcheck.rs`,
`context_spellcheck.rs` (cfg), `segmentation/bigram_lm.rs` (cfg).

### Test mới
`tests/golden_test_v2.rs`, `tests/guardrail_no_false_positive.rs`,
`tests/holdout_h.rs`.

### Tools
`tools/pos_eval.py`, `bigram_coverage.py`, `build_bigram_lm.py`,
`fetch_news_rss.py`, `check_viwiki_dump.py`, `fetch_viwiki_shard.py`,
`measure_context_gate.py`, `build_vsec_dev.py`.
Examples: `context_tune.rs`, `context_eval_vsec.rs` (cfg `context-lm`).

### Docs
`docs/pos_decision.md`, `docs/bigram_decision.md`, file này.

## 5. Dữ liệu

- `data_raw/` (ignored): UD conllu, human_gold, journalism, literature,
  news_extra, news_fresh, viwiki dump/extract, VSEC, Viet74K.
- `data_processed/` (ignored): `bigram_lm.tsv` (12,24 MB count≥2),
  `unigram_lm.tsv`, `dev_hj.txt` (1215 dòng), `dev_general.txt`,
  `dev_viwiki.txt`, `vsec_realword.tsv` (7.641 lỗi).
- `crates/vwi-core/data/bigram_lm.tsv` **đã xoá/ignore**; muốn bật feature:
  copy bản count≥3 (~7 MB) từ `data_processed` vào rồi build `--features context-lm`.
- viwiki: CC BY-SA, cần attribution nếu phân phối lại.

## 6. Số bigram quan trọng

```
train_tokens=22.138.216 | unigram_distinct=341.999
count>=2: 812.078 (11,46 MB)
count>=3: 503.278 (6,96 MB)
count>=5: 310.353 (4,34 MB)
dev_hj: 1.215 dòng / 65.199 token
OOV dev_hj (count>=3)=18,7% | unigram coverage=99,0%
perplexity dev_hj: k=0.01 828,78 | 0.05 853,60 | 0.1 943,10 | 0.5 1543,45
unigram prune: >=2 146.915 | >=3 96.886 | >=5 63.263
margin 10 ca H: mean 10,43 | min 6,54
```

## 7. Context bigram — kết quả âm tính

- Grid dev_hj: -4/10 → FP 17/1215 (1,40%), H real-word 3/10.
- VSEC 500 ca: -4/10 → TP 39, FP 5, recall 7,8%, FPR 1,67%, P 88,6%, F1 14,3%.
- Nới -5/10: recall 14,4%, FPR 7,0%. -6/2: recall 39,2%, FPR 50,7%.
- Kết luận: không cấu hình nào đạt recall 50–60% ở FPR ≤1,5%. Nút thắt:
  97% Wikipedia + absolute score thấp, không phải thuật toán.

## 8. Việc tiếp theo (thứ tự)

1. Cân bằng corpus: cap Wikipedia ~5M token, thêm báo chí (Leipzig/fsnaix
   news) + hành chính (VBPL nếu truy cập được). Rebuild LM, đo lại OOV/perplexity.
2. Dùng VSEC (7.641 ca) làm dev recall; grid `MIN_ABS/MARGIN` trên dev_hj
   (FPR) + VSEC (recall). Mục tiêu: recall ≥50% ở FPR ≤1,5%.
3. Chỉ khi đạt mới bật AutoFix; nếu không, giữ feature tắt.
4. Cân nhắc sửa tokenizer chung (compound) nếu muốn POS/context tốt hơn.

## 9. Lệnh tái lập

```powershell
cargo test --workspace
cargo test -p vwi-core --test golden_test_v2 -- --nocapture --test-threads=1
cargo test -p vwi-core --test holdout_h -- --nocapture
python tools/build_bigram_lm.py
python tools/measure_context_gate.py
# feature (sau khi copy TSV count>=3 vào crates/vwi-core/data/):
cargo run -p vwi-core --features context-lm --example context_eval_vsec
```
