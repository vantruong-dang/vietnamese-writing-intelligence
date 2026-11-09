use crate::types::Finding;
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};
use regex::RegexBuilder;
use std::sync::LazyLock;

static CRIPPLED_TOPIC_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(
        r"^(trong|qua|bằng|với)\s+[^,]+?\s+(đòi hỏi|yêu cầu|cho thấy|chứng minh|đã chứng minh|giúp|đã giúp|khiến|buộc|đã buộc|đã phát hiện|phát hiện|chứng tỏ)\b",
    )
    .case_insensitive(true)
    .multi_line(true)
    .build()
    .expect("Regex CRIPPLED_PREPOSITIONAL_TOPIC không hợp lệ")
});

static PREP_CONJ_CONFLICT_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"^(với những|với|qua|bằng)\s+[^,]+?\s+nhưng\b")
        .case_insensitive(true)
        .multi_line(true)
        .build()
        .expect("Regex PREPOSITION_CONJUNCTION_CONFLICT không hợp lệ")
});

pub const CRIPPLED_TOPIC_RULE_ID: &str = "CRIPPLED_PREPOSITIONAL_TOPIC";
pub const PREP_CONJ_RULE_ID: &str = "PREPOSITION_CONJUNCTION_CONFLICT";

/// Đánh giá lỗi Khung đề què (Crippled Prepositional Topic): Giới ngữ bị ép làm chủ thể cho vị từ.
pub fn evaluate_crippled_prepositional_topic(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in CRIPPLED_TOPIC_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        findings.push(Finding::new(
            CRIPPLED_TOPIC_RULE_ID,
            "topic_comment",
            "major",
            start_char,
            end_char,
            0.95,
            "Lẫn lộn Khung đề và Chủ đề: Giới từ biến cụm từ thành trạng ngữ/phương tiện, không thể làm chủ thể cho vị từ.",
            "linguistic",
            norm[m.start()..m.end()].to_string(),
        ));
    }

    findings
}

/// Đánh giá lỗi Xung đột Cú pháp (Preposition Conjunction Conflict): Giới từ mở đầu kết hợp với 'nhưng'.
pub fn evaluate_preposition_conjunction_conflict(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in PREP_CONJ_CONFLICT_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        findings.push(Finding::new(
            PREP_CONJ_RULE_ID,
            "topic_comment",
            "major",
            start_char,
            end_char,
            0.95,
            "Xung đột cú pháp: Giới từ mở đầu trạng ngữ không thể kết hợp trực tiếp với liên từ phản đề 'nhưng'.",
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
    fn test_crippled_topic_detection() {
        let text = "Trong tình hình kinh tế hiện nay đòi hỏi doanh nghiệp phải tái cơ cấu toàn diện.";
        let findings = evaluate_crippled_prepositional_topic(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, CRIPPLED_TOPIC_RULE_ID);
        assert_eq!(
            findings[0].matched_span,
            "Trong tình hình kinh tế hiện nay đòi hỏi"
        );
    }

    #[test]
    fn test_prep_conj_conflict_detection() {
        let text = "Với những nỗ lực phi thường nhưng họ vẫn không thể hoàn thành dự án đúng hạn.";
        let findings = evaluate_preposition_conjunction_conflict(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, PREP_CONJ_RULE_ID);
        assert_eq!(
            findings[0].matched_span,
            "Với những nỗ lực phi thường nhưng"
        );
    }

    #[test]
    fn test_clean_topic_sentence() {
        let text = "Tình hình kinh tế hiện nay đòi hỏi doanh nghiệp phải tái cơ cấu.";
        let findings = evaluate_crippled_prepositional_topic(text);
        assert!(findings.is_empty());
    }
}
