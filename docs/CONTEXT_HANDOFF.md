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
  news_extra, news_fresh, viwiki dump/extract, Leipzig news, legal sample,
  VSEC, Viet74K.
- `data_processed/` (ignored): `bigram_lm.tsv` (9,60 MB count≥2),
  `unigram_lm.tsv`, `dev_hj.txt` (1215 dòng), `dev_general.txt`,
  `dev_viwiki.txt`, `vsec_realword.tsv` (7.641 lỗi), `build_log.txt`.
- `crates/vwi-core/data/bigram_lm.tsv` **đã xoá/ignore**; muốn bật feature:
  copy bản count≥3 (~7 MB) từ `data_processed` vào rồi build `--features context-lm`.
- viwiki: CC BY-SA, cần attribution nếu phân phối lại.

## 6. Số bigram quan trọng

```
train_tokens=15.908.343 (wiki cap 5M + Leipzig 10M + legal 2M)
unigram_distinct=156.729
count>=2: 662.995 (9,60 MB)
count>=3: 447.704 (6,51 MB)
count>=5: 286.541
dev_hj: 1.215 dòng / 65.199 token
OOV dev_hj (count>=3)=17,0% | H clean OOV=4,9% | unigram coverage=99,0%
perplexity dev_hj: k=0.01 583,70 | 0.05 546,22 | 0.1 563,25 | 0.5 750,64
unigram prune: >=2 66.631 | >=3 45.229 | >=5 29.773
margin 10 ca H: mean 10,25 | min 5,25
```

## 7. Context bigram — kết quả âm tính

- LM cân bằng: VSEC -3/8 → recall 3,2%, FPR 0,67%; -4/10 → 9,0%, FPR 3,67%;
  -5/10 → 16,4%, FPR 9,67%; -6/4 → 29,2%, FPR 30,0%.
- Không cấu hình nào đạt recall 40–50% ở FPR ≤1,5%. Nút thắt còn lại:
  dev_hj OOV 17% (human_gold literary) + absolute score, không phải thuật toán.
- Feature `context-lm` vẫn tắt; defaults -3/8 (an toàn, recall thấp).

## 8. Việc tiếp theo (thứ tự)

1. Cân bằng corpus **đã làm**: wiki 5M + Leipzig 10M + legal 2M. Perplexity
   giảm 828 → 546, H clean OOV 12,8% → 4,9%; nhưng dev_hj OOV vẫn 17% vì
   human_gold là văn phê bình/literary.
2. Nếu muốn tiếp: tăng trọng số báo chí/essay hiện đại hoặc thêm nguồn
   literary; đo lại dev_hj OOV (mục tiêu <10%).
3. VSEC đã dùng làm dev recall; grid vẫn không đạt 40–50% ở FPR ≤1,5%.
4. Giữ `context-lm` tắt; chỉ bật nếu một LM mới đạt ngưỡng. Không AutoFix.

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
