use pyo3::prelude::*;
use std::collections::HashMap;
use vwi_core::{
    audit_linguistics as core_audit_linguistics,
    evaluate_ai_opening_cliche as core_eval_ai_opening,
    evaluate_ai_summary_tail as core_eval_ai_summary,
    evaluate_artificial_passive as core_eval_passive,
    evaluate_artificial_passive_fast as core_eval_passive_fast,
    evaluate_aspect_mechanics as core_eval_aspect,
    evaluate_bureaucratic_rules as core_eval_bureaucracy,
    evaluate_concessive_inversion as core_eval_concessive_inversion,
    evaluate_crippled_prepositional_topic as core_eval_crippled_topic,
    evaluate_dummy_it_translationese as core_eval_dummy_it,
    evaluate_factual_concessive_misuse as core_eval_factual_concessive,
    evaluate_formulaic_metaphor as core_eval_formulaic_metaphor,
    evaluate_hypothetical_concessive_misuse as core_eval_hypo_concessive,
    evaluate_logical_pleonasm as core_eval_pleonasm,
    evaluate_nominalization_heavy as core_eval_nominalization,
    evaluate_preposition_conjunction_conflict as core_eval_prep_conj,
    evaluate_redundant_beneficiary_passive as core_eval_redundant_beneficiary,
    evaluate_superlative_misuse as core_eval_superlative,
    evaluate_suyt_nua_thi as core_eval_suyt_nua_thi,
    evaluate_syntax_cliche as core_eval_syntax_cliche,
    evaluate_syntax_parallel as core_eval_syntax_parallel,
    evaluate_syntax_reporting as core_eval_syntax_reporting,
    evaluate_temporal_framing as core_eval_temporal_framing,
    evaluate_translationese_soleil as core_eval_translationese_soleil,
    normalize_nfc as core_norm_nfc, tokenize_viterbi as core_tokenize_viterbi,
    Auditor as CoreAuditor, AuditorConfig as CoreAuditorConfig, DaachorseSegmenter,
    Finding as CoreFinding, TokenSpan as CoreTokenSpan,
};

#[pyclass(name = "TokenSpan")]
#[derive(Clone, Debug)]
pub struct PyTokenSpan {
    #[pyo3(get)]
    pub text: String,
    #[pyo3(get)]
    pub start: usize,
    #[pyo3(get)]
    pub end: usize,
    #[pyo3(get)]
    pub is_compound: bool,
    #[pyo3(get)]
    pub pos: Option<String>,
    #[pyo3(get)]
    pub pos_tag: Option<String>,
}

impl From<CoreTokenSpan> for PyTokenSpan {
    fn from(span: CoreTokenSpan) -> Self {
        let pos_str = span.pos.map(|p| p.as_str().to_string());
        Self {
            text: span.text,
            start: span.start,
            end: span.end,
            is_compound: span.is_compound,
            pos: pos_str.clone(),
            pos_tag: pos_str,
        }
    }
}

#[pyclass(name = "Finding")]
#[derive(Clone, Debug)]
pub struct PyFinding {
    #[pyo3(get)]
    pub pattern_id: String,
    #[pyo3(get)]
    pub category: String,
    #[pyo3(get)]
    pub severity: String,
    #[pyo3(get)]
    pub start: usize,
    #[pyo3(get)]
    pub end: usize,
    #[pyo3(get)]
    pub confidence: f64,
    #[pyo3(get)]
    pub rationale: String,
    #[pyo3(get)]
    pub source: String,
    #[pyo3(get)]
    pub matched_span: String,
    #[pyo3(get)]
    pub suggestion: Option<String>,
}

impl From<CoreFinding> for PyFinding {
    fn from(f: CoreFinding) -> Self {
        Self {
            pattern_id: f.pattern_id,
            category: f.category,
            severity: f.severity,
            start: f.start,
            end: f.end,
            confidence: f.confidence,
            rationale: f.rationale,
            source: f.source,
            matched_span: f.matched_span,
            suggestion: f.suggestion,
        }
    }
}

/// Cấu hình kiểm toán ngôn ngữ học (AuditorConfig) cho phép vô hiệu hóa quy tắc,
/// lọc theo nhóm thể loại hoặc ghi đè mức độ nghiêm trọng.
#[pyclass(name = "AuditorConfig")]
#[derive(Clone, Debug, Default)]
pub struct PyAuditorConfig {
    pub inner: CoreAuditorConfig,
}

#[pymethods]
impl PyAuditorConfig {
    #[new]
    #[pyo3(signature = (disabled_rules=None, enabled_categories=None, severity_overrides=None))]
    fn new(
        disabled_rules: Option<Vec<String>>,
        enabled_categories: Option<Vec<String>>,
        severity_overrides: Option<HashMap<String, String>>,
    ) -> Self {
        let mut inner = CoreAuditorConfig::new();
        if let Some(rules) = disabled_rules {
            for r in rules {
                inner.disable_rule(r);
            }
        }
        if let Some(cats) = enabled_categories {
            inner.set_enabled_categories(cats);
        }
        if let Some(overrides) = severity_overrides {
            for (k, v) in overrides {
                inner.override_severity(k, v);
            }
        }
        Self { inner }
    }

    /// Vô hiệu hóa một quy tắc theo pattern_id.
    fn disable_rule(&mut self, rule_id: String) {
        self.inner.disable_rule(rule_id);
    }

    /// Kích hoạt lại một quy tắc.
    fn enable_rule(&mut self, rule_id: String) {
        self.inner.enable_rule(&rule_id);
    }

    /// Thiết lập danh sách các nhóm thể loại được kích hoạt.
    fn set_enabled_categories(&mut self, categories: Vec<String>) {
        self.inner.set_enabled_categories(categories);
    }

    /// Ghi đè mức độ nghiêm trọng cho một pattern_id.
    fn override_severity(&mut self, rule_id: String, severity: String) {
        self.inner.override_severity(rule_id, severity);
    }
}

/// Động cơ kiểm toán ngôn ngữ học cấp cao (Auditor) chạy hoàn toàn trong Rust,
/// hỗ trợ cấu hình tùy biến và kiểm toán song song đa tài liệu qua Rayon (giải phóng Python GIL).
#[pyclass(name = "Auditor")]
#[derive(Clone, Debug)]
pub struct PyAuditor {
    pub inner: CoreAuditor,
}

#[pymethods]
impl PyAuditor {
    #[new]
    #[pyo3(signature = (config=None))]
    fn new(config: Option<PyAuditorConfig>) -> Self {
        let core_config = config.map(|c| c.inner).unwrap_or_default();
        Self {
            inner: CoreAuditor::with_config(core_config),
        }
    }

    /// Kiểm toán một văn bản đơn lẻ, trả về danh sách các vi phạm ngôn ngữ.
    fn audit(&self, py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
        let findings = py.allow_threads(|| self.inner.audit(text));
        Ok(findings.into_iter().map(PyFinding::from).collect())
    }

    /// Kiểm toán song song danh sách nhiều văn bản trên đa luồng CPU (Rayon).
    fn audit_batch(&self, py: Python<'_>, texts: Vec<String>) -> PyResult<Vec<Vec<PyFinding>>> {
        let findings_batch = py.allow_threads(|| {
            let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
            self.inner.audit_batch(&text_refs)
        });
        Ok(findings_batch
            .into_iter()
            .map(|f_list| f_list.into_iter().map(PyFinding::from).collect())
            .collect())
    }
}

/// Chuẩn hóa chuỗi ký tự sang Unicode NFC từ Rust core.
#[pyfunction]
fn normalize_nfc(text: &str) -> PyResult<String> {
    Ok(core_norm_nfc(text))
}

/// Tách từ nhanh bằng thuật toán Double-Array Aho-Corasick (daachorse) trong Rust.
/// Sử dụng py.allow_threads để giải phóng Python GIL cho đa luồng thực thụ.
#[pyfunction]
fn tokenize(py: Python<'_>, text: &str) -> PyResult<Vec<PyTokenSpan>> {
    let tokens = py.allow_threads(|| {
        let segmenter = DaachorseSegmenter::new();
        segmenter.tokenize(text)
    });
    Ok(tokens.into_iter().map(PyTokenSpan::from).collect())
}

/// Tách từ với từ điển tùy biến (ví dụ nạp toàn bộ danh từ ghép chuyên ngành).
#[pyfunction]
fn tokenize_with_vocab(
    py: Python<'_>,
    text: &str,
    vocab: Vec<String>,
) -> PyResult<Vec<PyTokenSpan>> {
    let vocab_refs: Vec<&str> = vocab.iter().map(|s| s.as_str()).collect();
    let segmenter = DaachorseSegmenter::with_vocab(&vocab_refs)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?;
    let tokens = py.allow_threads(|| segmenter.tokenize(text));
    Ok(tokens.into_iter().map(PyTokenSpan::from).collect())
}

/// Tách từ bằng thuật toán Viterbi Trellis dựa trên Language Model thống kê (Unigram + Bigram + Length-Bonus).
/// Giải quyết triệt để các ca nhập nhằng hóc búa (như "bàn là" vs "bàn" + "là", "học sinh học").
#[pyfunction]
fn tokenize_viterbi(py: Python<'_>, text: &str) -> PyResult<Vec<PyTokenSpan>> {
    let tokens = py.allow_threads(|| core_tokenize_viterbi(text));
    Ok(tokens.into_iter().map(PyTokenSpan::from).collect())
}

/// Động cơ cảm biến ngôn ngữ tổng hợp: chạy toàn bộ các luật trong Rust với tốc độ native.
#[pyfunction]
fn audit_linguistics(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_audit_linguistics(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá quy tắc ARTIFICIAL_PASSIVE_BE_BY (câu bị động nhân tạo "được/bị ... bởi").
#[pyfunction]
fn evaluate_artificial_passive(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_passive(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá quy tắc ARTIFICIAL_PASSIVE_BE_BY bằng thuật toán quét thuần Rust (Zero-Regex).
#[pyfunction]
fn evaluate_artificial_passive_fast(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_passive_fast(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá lỗi Thụ hưởng Dư thừa (REDUNDANT_BENEFICIARY_PASSIVE).
#[pyfunction]
fn evaluate_redundant_beneficiary_passive(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_redundant_beneficiary(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá lỗi Khung đề què (CRIPPLED_PREPOSITIONAL_TOPIC).
#[pyfunction]
fn evaluate_crippled_prepositional_topic(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_crippled_topic(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá lỗi Xung đột Cú pháp Giới từ - Nhưng (PREPOSITION_CONJUNCTION_CONFLICT).
#[pyfunction]
fn evaluate_preposition_conjunction_conflict(
    py: Python<'_>,
    text: &str,
) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_prep_conj(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá lỗi Trùng ngữ logic (LOGICAL_PLEONASM: 'tối ưu nhất', 'ánh nắng mặt trời'...).
#[pyfunction]
fn evaluate_logical_pleonasm(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_pleonasm(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá lỗi Danh từ hóa nặng nề (NOMINALIZATION_HEAVY).
#[pyfunction]
fn evaluate_nominalization_heavy(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_nominalization(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá lỗi Khung thời gian dịch ngữ (TEMPORAL_IN_TRANSLATIONESE).
#[pyfunction]
fn evaluate_temporal_framing(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_temporal_framing(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá lỗi Soleil/Sunshine (TRANSLATIONESE_SOLEIL_SUNSHINE).
#[pyfunction]
fn evaluate_translationese_soleil(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_translationese_soleil(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá lỗi Nhượng bộ Hiện thực cho Tương lai (FACTUAL_CONCESSIVE_MISUSE).
#[pyfunction]
fn evaluate_factual_concessive_misuse(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_factual_concessive(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá lỗi Nhượng bộ Giả định cho Quá khứ (HYPOTHETICAL_CONCESSIVE_MISUSE).
#[pyfunction]
fn evaluate_hypothetical_concessive_misuse(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_hypo_concessive(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá lỗi Thời thể lai căng 'đã và đang' (CXH_VERBAL_SERIES_TENSE_FALSE_MARKER).
#[pyfunction]
fn evaluate_aspect_mechanics(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_aspect(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá lỗi Vị từ trần thuật đi kèm danh ngữ cụt (CXH_REPORTING_VERB_INCOMPLETE_CLAUSE).
#[pyfunction]
fn evaluate_syntax_reporting(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_syntax_reporting(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá lỗi Đối ngẫu bất đối xứng (CXH_PARALLEL_CONJUNCTION_ASYMMETRY).
#[pyfunction]
fn evaluate_syntax_parallel(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_syntax_parallel(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá lỗi Cú pháp lai ghép (CXH_HYBRID_COMPLEX_CONJUNCTION).
#[pyfunction]
fn evaluate_syntax_cliche(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_syntax_cliche(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá lỗi Dùng 'chí ít' (CXH_SUPERLATIVE_CHI_IT_MISUSE).
#[pyfunction]
fn evaluate_superlative_misuse(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_superlative(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá lỗi Lai ghép 'suýt nữa thì' (CXH_SUYT_NUA_THI_CONTAMINATION).
#[pyfunction]
fn evaluate_suyt_nua_thi(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_suyt_nua_thi(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá Mở đầu sáo ngữ AI (AI_OPENING_CLICHE).
#[pyfunction]
fn evaluate_ai_opening_cliche(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_ai_opening(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá Đuôi tổng kết máy móc AI (AI_SUMMARY_TAIL).
#[pyfunction]
fn evaluate_ai_summary_tail(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_ai_summary(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá Ẩn dụ sáo rỗng AI (FORMULAIC_METAPHOR).
#[pyfunction]
fn evaluate_formulaic_metaphor(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_formulaic_metaphor(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá Sáo ngữ & Đệm hành chính bằng Aho-Corasick đa mẫu (BUREAUCRATIC_VERB_PADDING, REDUNDANT_CONNECTORS, VAGUE_INTENSIFIER).
#[pyfunction]
fn evaluate_bureaucratic_rules(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_bureaucracy(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá Chủ ngữ giả "Nó là..." & Tính từ hóa nhân tạo (DUMMY_IT_TRANSLATIONESE, ADJECTIVAL_SUFFIX_FAKE).
#[pyfunction]
fn evaluate_dummy_it_translationese(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_dummy_it(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Đánh giá Xung đột nhượng bộ kép "Mặc dù... tuy nhiên..." (CONCESSIVE_INVERSION_CONFLICT).
#[pyfunction]
fn evaluate_concessive_inversion(py: Python<'_>, text: &str) -> PyResult<Vec<PyFinding>> {
    let findings = py.allow_threads(|| core_eval_concessive_inversion(text));
    Ok(findings.into_iter().map(PyFinding::from).collect())
}

/// Lõi Rust tăng tốc cho Vietnamese Writing Intelligence (vwi-py).
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyTokenSpan>()?;
    m.add_class::<PyFinding>()?;
    m.add_class::<PyAuditorConfig>()?;
    m.add_class::<PyAuditor>()?;
    m.add_function(wrap_pyfunction!(normalize_nfc, m)?)?;
    m.add_function(wrap_pyfunction!(tokenize, m)?)?;
    m.add_function(wrap_pyfunction!(tokenize_viterbi, m)?)?;
    m.add_function(wrap_pyfunction!(tokenize_with_vocab, m)?)?;
    m.add_function(wrap_pyfunction!(audit_linguistics, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_artificial_passive, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_artificial_passive_fast, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_redundant_beneficiary_passive, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_crippled_prepositional_topic, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_preposition_conjunction_conflict, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_logical_pleonasm, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_nominalization_heavy, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_temporal_framing, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_translationese_soleil, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_factual_concessive_misuse, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_hypothetical_concessive_misuse, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_aspect_mechanics, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_syntax_reporting, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_syntax_parallel, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_syntax_cliche, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_superlative_misuse, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_suyt_nua_thi, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_ai_opening_cliche, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_ai_summary_tail, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_formulaic_metaphor, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_bureaucratic_rules, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_dummy_it_translationese, m)?)?;
    m.add_function(wrap_pyfunction!(evaluate_concessive_inversion, m)?)?;
    Ok(())
}
