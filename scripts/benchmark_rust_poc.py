"""Microbenchmark toàn diện: Python thuần vs Rust Core Engine (25 Rules + Viterbi + Rayon)."""

from __future__ import annotations

import sys
import time

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")

from vn_writing_intel import _core
from vn_writing_intel.linguistics import (
    RuleBasedSegmenter,
    RustViterbiSegmenter,
    UndertheseaSegmenter,
)
from vn_writing_intel.linguistics.context_rules import (
    AdjectivalSuffixFakeRule,
    AIOpeningClicheRule,
    AISummaryTailRule,
    ArtificialPassiveRule,
    AspectMechanicsRule,
    BureaucraticVerbPaddingRule,
    ConcessiveInversionConflictRule,
    CrippledPrepositionalTopicRule,
    DummyItTranslationeseRule,
    FactualConcessiveMisuseRule,
    FormulaicMetaphorRule,
    HypotheticalConcessiveMisuseRule,
    LogicalPleonasmRule,
    NominalizationHeavyRule,
    PrepositionConjunctionConflictRule,
    RedundantBeneficiaryPassiveRule,
    RedundantConnectorsRule,
    SuperlativeMisuseRule,
    SuytNuaThiRule,
    SyntaxClicheRule,
    SyntaxParallelRule,
    SyntaxReportingRule,
    TemporalFramingRule,
    TranslationeseSoleilRule,
    VagueIntensifierRule,
)


def benchmark():
    text_tokenize = (
        "Trí tuệ nhân tạo và chuyển đổi số đang tối ưu hóa hệ thống phân tán trong kỷ nguyên số. "
        "Mẹ mua một chiếc bàn là mới để là quần áo cho học sinh học bài."
    )
    text_complex = (
        "Không thể phủ nhận rằng trong tình hình kinh tế hiện nay đòi hỏi giải pháp tối ưu nhất, "
        "nhưng việc thực hiện được phê duyệt bởi ban giám đốc nhằm tiến hành thực hiện phương án "
        "mang tính chất đột phá và đồng thời đảm bảo ánh nắng mặt trời chiếu sáng."
    )

    iterations = 10_000

    print("=" * 80)
    print(f"      VWI COMPREHENSIVE BENCHMARK: PURE PYTHON vs RUST ENGINE ({iterations:,} iterations)")
    print("=" * 80)

    # -------------------------------------------------------------
    # 1. Benchmark Tokenization
    # -------------------------------------------------------------
    py_segmenter = RuleBasedSegmenter()
    rust_viterbi_segmenter = RustViterbiSegmenter()
    underthesea_segmenter = None
    try:
        underthesea_segmenter = UndertheseaSegmenter()
    except Exception:
        pass

    # A) Python Regex Segmenter
    t0 = time.perf_counter()
    for _ in range(iterations):
        _ = py_segmenter.tokenize(text_tokenize)
    py_tok_time = time.perf_counter() - t0

    # B) Rust Greedy Daachorse (embedded 72.5k dict)
    t0 = time.perf_counter()
    for _ in range(iterations):
        _ = _core.tokenize(text_tokenize)
    rust_daachorse_time = time.perf_counter() - t0

    # C) Rust Viterbi Trellis (72.5k dict + Bigram LM + Length Bonus)
    t0 = time.perf_counter()
    for _ in range(iterations):
        _ = rust_viterbi_segmenter.tokenize(text_tokenize)
    rust_viterbi_time = time.perf_counter() - t0

    # D) Underthesea (CRF / Python)
    uts_tok_time = None
    if underthesea_segmenter:
        # Run smaller sample for underthesea due to slower python inference
        uts_iters = min(1_000, iterations)
        t0 = time.perf_counter()
        for _ in range(uts_iters):
            _ = underthesea_segmenter.tokenize(text_tokenize)
        uts_tok_time = (time.perf_counter() - t0) * (iterations / uts_iters)

    print("\n1. BỘ TÁCH TỪ (TOKENIZATION):")
    print(f"   - Python Regex (64 từ ghép thủ công)     : {py_tok_time:8.4f} s | {iterations / py_tok_time:10,.0f} ops/sec")
    if uts_tok_time:
        print(f"   - Underthesea (CRF/Python)              : {uts_tok_time:8.4f} s | {iterations / uts_tok_time:10,.0f} ops/sec")
    print(f"   - Rust Daachorse (72,564 từ nhúng, Greedy): {rust_daachorse_time:8.4f} s | {iterations / rust_daachorse_time:10,.0f} ops/sec")
    print(f"   - Rust Viterbi Trellis (72,564 từ + Bigram): {rust_viterbi_time:8.4f} s | {iterations / rust_viterbi_time:10,.0f} ops/sec")

    speedup_greedy = py_tok_time / rust_daachorse_time
    speedup_viterbi = py_tok_time / rust_viterbi_time
    print(f"   ==> Rust Daachorse nhanh hơn Python Regex: {speedup_greedy:.2f}x")
    print(f"   ==> Rust Viterbi Trellis nhanh hơn Python:  {speedup_viterbi:.2f}x (Độ chính xác ngữ cảnh vượt trội)")
    if uts_tok_time:
        print(f"   ==> Rust Viterbi nhanh hơn Underthesea:     {uts_tok_time / rust_viterbi_time:.2f}x")

    # -------------------------------------------------------------
    # 2. Benchmark Full Linguistic Audit (All 25 Rules)
    # -------------------------------------------------------------
    all_py_rules = [
        CrippledPrepositionalTopicRule(),
        PrepositionConjunctionConflictRule(),
        FactualConcessiveMisuseRule(),
        HypotheticalConcessiveMisuseRule(),
        TemporalFramingRule(),
        ArtificialPassiveRule(),
        RedundantBeneficiaryPassiveRule(),
        LogicalPleonasmRule(),
        AspectMechanicsRule(),
        SyntaxReportingRule(),
        SyntaxParallelRule(),
        SyntaxClicheRule(),
        SuperlativeMisuseRule(),
        AIOpeningClicheRule(),
        AISummaryTailRule(),
        FormulaicMetaphorRule(),
        NominalizationHeavyRule(),
        SuytNuaThiRule(),
        TranslationeseSoleilRule(),
        BureaucraticVerbPaddingRule(),
        RedundantConnectorsRule(),
        VagueIntensifierRule(),
        DummyItTranslationeseRule(),
        AdjectivalSuffixFakeRule(),
        ConcessiveInversionConflictRule(),
    ]

    t0 = time.perf_counter()
    for _ in range(iterations):
        findings = []
        for r in all_py_rules:
            findings.extend(r.evaluate(text_complex, [], {}))
    py_audit_time = time.perf_counter() - t0

    # Single-thread Rust audit
    auditor = _core.Auditor()
    t0 = time.perf_counter()
    for _ in range(iterations):
        _ = auditor.audit(text_complex)
    rust_audit_time = time.perf_counter() - t0

    # Multi-thread Rayon Batch audit (100 docs per batch)
    batch_docs = [text_complex] * 100
    batch_iters = iterations // 100
    t0 = time.perf_counter()
    for _ in range(batch_iters):
        _ = auditor.audit_batch(batch_docs)
    rust_batch_time = time.perf_counter() - t0

    audit_speedup = py_audit_time / rust_audit_time
    batch_speedup = py_audit_time / rust_batch_time

    print("\n2. KIỂM TOÁN TỔNG HỢP TOÀN DIỆN (FULL AUDIT - 25 LUẬT NGÔN NGỮ HỌC):")
    print(f"   - Python thuần (Vòng lặp 25 Rules)       : {py_audit_time:8.4f} s | {iterations / py_audit_time:10,.0f} ops/sec")
    print(f"   - Rust Auditor (Đơn luồng native C-ABI)   : {rust_audit_time:8.4f} s | {iterations / rust_audit_time:10,.0f} ops/sec")
    print(f"   - Rust Auditor Batch (Rayon đa nhân CPU) : {rust_batch_time:8.4f} s | {iterations / rust_batch_time:10,.0f} ops/sec")
    print(f"   ==> RUST CORE ĐƠN LUỒNG NHANH HƠN: {audit_speedup:.2f}x")
    print(f"   ==> RUST CORE RAYON BATCH NHANH HƠN: {batch_speedup:.2f}x")
    print("=" * 80)


if __name__ == "__main__":
    benchmark()
