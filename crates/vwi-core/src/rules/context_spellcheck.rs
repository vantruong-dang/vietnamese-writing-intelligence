//! # CONTEXT SPELLCHECKER (BIGRAM LM, ADVISORY)
//!
//! Chỉ xử lý **real-word**: từ có thật trong từ điển nhưng sai ngữ cảnh
//! (`cơn bảo`, `buổi sang`). OOV do `spellcheck.rs` xử lý.
//!
//! Quy tắc an toàn:
//! - ứng viên phải cùng lớp confusion (tone/phụ âm đầu/vần) và có thật;
//! - **observed-side guardrail**: ứng viên phải có ít nhất một bigram thật;
//! - chỉ flag khi điểm ngữ cảnh hiện tại thấp và ứng viên tốt hơn `MARGIN`;
//! - luôn `Advisory` (0 điểm phạt).

use crate::rules::phonotactics::{is_exempt_loanword, is_valid_vietnamese_word};
use crate::rules::spellcheck::valid_candidates;
use crate::segmentation::bigram_lm;
use crate::types::Finding;
use crate::unicode_utils::normalize_nfc;

pub const CONTEXT_SPELLCHECK_RULE_ID: &str = "CONTEXT_SPELLING_ERROR";

/// Ngưỡng mặc định; sẽ được chọn lại bằng `examples/context_tune.rs` trên dev.
pub const DEFAULT_MIN_ABS: f32 = -3.0;
pub const DEFAULT_MARGIN: f32 = 8.0;

/// Điểm ngữ cảnh = trung bình log P trên các phía **có bigram thật**.
/// Trả về (score, observed_sides). observed_sides == 0 → không đủ tin cậy.
fn ctx_score(prev: Option<&str>, word: &str, next: Option<&str>) -> (f32, usize) {
    let mut sum = 0.0f32;
    let mut observed = 0usize;

    if let Some(prev) = prev {
        if let Some(lp) = bigram_lm::log_p_bigram(prev, word) {
            sum += lp;
            observed += 1;
        }
    }
    if let Some(next) = next {
        if let Some(lp) = bigram_lm::log_p_bigram(word, next) {
            sum += lp;
            observed += 1;
        }
    }

    if observed == 0 {
        (-15.0, 0)
    } else {
        (sum / observed as f32, observed)
    }
}

/// Ứng viên confusion cho một từ (dùng cho cả module và tune example).
pub fn candidate_words(word: &str) -> Vec<String> {
    valid_candidates(word)
}

/// Word-level tokenization: tách theo khoảng trắng/dấu câu, KHÔNG dùng
/// Viterbi compound. Context spellchecker cần thấy từng âm tiết (`buổi`,
/// `sang`) thay vì cụm bị gộp (`buổi sang`).
fn word_spans(text: &str) -> Vec<(String, usize, usize)> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_alphabetic() {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && chars[i].is_alphabetic() {
            i += 1;
        }
        out.push((chars[start..i].iter().collect(), start, i));
    }
    out
}

/// Phiên bản có ngưỡng tùy chỉnh, phục vụ grid search trên dev.
pub fn evaluate_context_spellcheck_with(text: &str, min_abs: f32, margin: f32) -> Vec<Finding> {
    let normalized = normalize_nfc(text);
    let words = word_spans(&normalized);
    let mut findings = Vec::new();

    for i in 0..words.len() {
        let (word, start, end) = &words[i];
        if word.chars().count() < 2 {
            continue;
        }
        let lower = word.to_lowercase();
        if word.chars().next().is_some_and(|c| c.is_uppercase())
            || word.chars().all(|c| c.is_uppercase())
            || is_exempt_loanword(&lower)
            || !is_valid_vietnamese_word(&lower)
        {
            continue;
        }

        let prev = words
            .get(i.wrapping_sub(1))
            .map(|(w, _, _)| w.to_lowercase());
        let next = words.get(i + 1).map(|(w, _, _)| w.to_lowercase());

        let (current_score, _) = ctx_score(prev.as_deref(), &lower, next.as_deref());
        if current_score >= min_abs {
            continue;
        }

        let mut best: Option<(String, f32)> = None;
        for candidate in valid_candidates(&lower) {
            let (score, observed) = ctx_score(prev.as_deref(), &candidate, next.as_deref());
            // Observed-side guardrail: không đoán nếu cả hai phía đều backoff.
            if observed == 0 || score <= min_abs {
                continue;
            }
            if best
                .as_ref()
                .map_or(true, |(_, best_score)| score > *best_score)
            {
                best = Some((candidate, score));
            }
        }

        let Some((replacement, best_score)) = best else {
            continue;
        };
        if best_score - current_score <= margin {
            continue;
        }

        findings.push(
            Finding::new(
                CONTEXT_SPELLCHECK_RULE_ID,
                "spellcheck",
                "advisory",
                *start,
                *end,
                0.8,
                format!(
                    "'{}' không khớp ngữ cảnh; ứng viên '{}' có xác suất bigram cao hơn rõ rệt.",
                    word, replacement
                ),
                "bigram_lm",
                word,
            )
            // Chỉ advisory, KHÔNG auto-fix: LM còn lệch register, 1.4% FP.
            .with_note(format!("Có thể bạn muốn dùng '{}'.", replacement))
            .with_stage("bigram"),
        );
    }

    findings
}

pub fn evaluate_context_spellcheck(text: &str) -> Vec<Finding> {
    evaluate_context_spellcheck_with(text, DEFAULT_MIN_ABS, DEFAULT_MARGIN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidate_generation_includes_tone_variants() {
        // Bug cũ: vần "ao" đặt dấu sai nên "bảo" không sinh ra "bão".
        assert!(candidate_words("bảo").contains(&"bão".to_string()));
        assert!(candidate_words("sang").contains(&"sáng".to_string()));
        assert!(candidate_words("canh").contains(&"cạnh".to_string()));
    }

    #[test]
    fn test_does_not_flag_clean_sentences() {
        for text in [
            "Cô giáo cho cả lớp nghỉ sớm vì trời mưa to.",
            "Anh ấy chạy rất nhanh nên về nhất.",
            "Chính sách này do nhà nước ban hành.",
            "Hôm qua tôi mới biết mai trời sẽ mưa.",
        ] {
            let findings = evaluate_context_spellcheck_with(text, DEFAULT_MIN_ABS, DEFAULT_MARGIN);
            assert!(findings.is_empty(), "Bắt oan {:?}: {:?}", text, findings);
        }
    }
}
