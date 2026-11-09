use crate::types::Finding;
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};
use regex::RegexBuilder;
use std::sync::LazyLock;

static ASPECT_MECHANICS_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\b(đã và đang)\b")
        .case_insensitive(true)
        .build()
        .expect("Regex CXH_VERBAL_SERIES_TENSE_FALSE_MARKER không hợp lệ")
});

pub const ASPECT_MECHANICS_RULE_ID: &str = "CXH_VERBAL_SERIES_TENSE_FALSE_MARKER";

/// Bắt lỗi cụm 'đã và đang' dịch thô từ thì hoàn thành tiếp diễn tiếng Anh.
pub fn evaluate_aspect_mechanics(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in ASPECT_MECHANICS_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        findings.push(Finding::new(
            ASPECT_MECHANICS_RULE_ID,
            "aspect_mechanics",
            "minor",
            start_char,
            end_char,
            0.95,
            "Cụm 'đã và đang' dịch thô từ thì hoàn thành tiếp diễn tiếng Anh, tạo cảm giác lai căng và làm nặng câu văn.",
            "linguistic",
            norm[m.start()..m.end()].to_string(),
        ));
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_da_va_dang() {
        let text = "Chúng tôi đã và đang nỗ lực cải thiện hiệu năng.";
        let findings = evaluate_aspect_mechanics(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, ASPECT_MECHANICS_RULE_ID);
        assert_eq!(findings[0].matched_span, "đã và đang");
    }
}
