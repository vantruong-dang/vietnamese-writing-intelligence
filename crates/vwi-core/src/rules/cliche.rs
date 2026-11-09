use crate::types::Finding;
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};
use fancy_regex::Regex as FancyRegex;
use regex::RegexBuilder;
use std::sync::LazyLock;

static AI_OPENING_RE: LazyLock<FancyRegex> = LazyLock::new(|| {
    FancyRegex::new(r"(?im)^(không thể phủ nhận rằng|có thể nói rằng|nhìn chung(?!\s+lại)|trong bối cảnh .* hiện nay)[\s,]*")
        .expect("Mẫu regex AI_OPENING_CLICHE không hợp lệ")
});

static AI_SUMMARY_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\b(tóm lại|nói tóm lại|nhìn chung lại|xét cho cùng)[,\s]*")
        .case_insensitive(true)
        .build()
        .expect("Regex AI_SUMMARY_TAIL không hợp lệ")
});

static FORMULAIC_METAPHOR_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\b(bức tranh toàn cảnh|chiếc chìa khóa vạn năng|con dao hai lưỡi|ngọn hải đăng)\b")
        .case_insensitive(true)
        .build()
        .expect("Regex FORMULAIC_METAPHOR không hợp lệ")
});

pub const AI_OPENING_RULE_ID: &str = "AI_OPENING_CLICHE";
pub const AI_SUMMARY_RULE_ID: &str = "AI_SUMMARY_TAIL";
pub const FORMULAIC_METAPHOR_RULE_ID: &str = "FORMULAIC_METAPHOR";

/// Bắt mở đầu sáo rỗng rập khuôn của AI (AI Opening Cliche).
pub fn evaluate_ai_opening_cliche(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in AI_OPENING_RE.find_iter(&norm).flatten() {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        findings.push(Finding::new(
            AI_OPENING_RULE_ID,
            "humanizer",
            "critical",
            start_char,
            end_char,
            0.95,
            "Cụm rào đón mở đầu rập khuôn kiểu RLHF, làm giảm độ tin cậy của luận điểm.",
            "linguistic",
            norm[m.start()..m.end()].to_string(),
        ));
    }

    findings
}

/// Bắt từ nối tổng kết máy móc (AI Summary Tail).
pub fn evaluate_ai_summary_tail(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in AI_SUMMARY_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        findings.push(Finding::new(
            AI_SUMMARY_RULE_ID,
            "humanizer",
            "major",
            start_char,
            end_char,
            0.95,
            "Từ nối tổng kết máy móc làm giảm sức nặng của nhận định.",
            "linguistic",
            norm[m.start()..m.end()].to_string(),
        ));
    }

    findings
}

/// Bắt các ẩn dụ công thức sáo rỗng (Formulaic Metaphor).
pub fn evaluate_formulaic_metaphor(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in FORMULAIC_METAPHOR_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        findings.push(Finding::new(
            FORMULAIC_METAPHOR_RULE_ID,
            "humanizer",
            "critical",
            start_char,
            end_char,
            0.95,
            "Sử dụng ẩn dụ sáo rỗng đặc trưng của mô hình ngôn ngữ lớn.",
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
    fn test_ai_opening() {
        let text = "Không thể phủ nhận rằng công nghệ đang thay đổi thế giới.";
        let findings = evaluate_ai_opening_cliche(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, AI_OPENING_RULE_ID);
        assert!(findings[0].matched_span.starts_with("Không thể phủ nhận rằng"));
    }

    #[test]
    fn test_ai_summary() {
        let text = "Tóm lại, dự án đã thành công tốt đẹp.";
        let findings = evaluate_ai_summary_tail(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, AI_SUMMARY_RULE_ID);
        assert!(findings[0].matched_span.starts_with("Tóm lại"));
    }

    #[test]
    fn test_formulaic_metaphor() {
        let text = "Đây là bức tranh toàn cảnh về thị trường bán lẻ.";
        let findings = evaluate_formulaic_metaphor(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, FORMULAIC_METAPHOR_RULE_ID);
        assert_eq!(findings[0].matched_span, "bức tranh toàn cảnh");
    }
}
