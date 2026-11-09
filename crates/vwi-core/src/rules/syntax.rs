use crate::types::Finding;
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};
use regex::RegexBuilder;
use std::sync::LazyLock;

static SYNTAX_REPORTING_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"^(phải nói là|cho là|cho rằng)\s+(?:một\s+)?(?:cú|sự|cuộc|việc|quyết định)\s+[^,.]+[.?!]?$")
        .case_insensitive(true)
        .build()
        .expect("Regex CXH_REPORTING_VERB_INCOMPLETE_CLAUSE không hợp lệ")
});

static SYNTAX_PARALLEL_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"^(một\s+[^,]+?),?\s+nhưng\s+(không|chưa|thiếu)\b")
        .case_insensitive(true)
        .build()
        .expect("Regex CXH_PARALLEL_CONJUNCTION_ASYMMETRY không hợp lệ")
});

static SYNTAX_CLICHE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\bnhư\s+[^,]+?\s+đã\s+biết\s+(?:là|rằng)\b")
        .case_insensitive(true)
        .build()
        .expect("Regex CXH_HYBRID_COMPLEX_CONJUNCTION không hợp lệ")
});

pub const SYNTAX_REPORTING_RULE_ID: &str = "CXH_REPORTING_VERB_INCOMPLETE_CLAUSE";
pub const SYNTAX_PARALLEL_RULE_ID: &str = "CXH_PARALLEL_CONJUNCTION_ASYMMETRY";
pub const SYNTAX_CLICHE_RULE_ID: &str = "CXH_HYBRID_COMPLEX_CONJUNCTION";

/// Bắt lỗi sau vị từ trần thuật dùng một danh ngữ đơn độc thay vì mệnh đề trọn vẹn.
pub fn evaluate_syntax_reporting(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in SYNTAX_REPORTING_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        findings.push(Finding::new(
            SYNTAX_REPORTING_RULE_ID,
            "syntax_reporting",
            "major",
            start_char,
            end_char,
            0.95,
            "Sau 'phải nói là' bắt buộc phải là một mệnh đề trọn vẹn, không dùng một danh ngữ đơn độc.",
            "linguistic",
            norm[m.start()..m.end()].to_string(),
        ));
    }

    findings
}

/// Bắt lỗi liên từ 'nhưng' nối hai vế bất đối xứng (danh ngữ đối lập vị từ cụt).
pub fn evaluate_syntax_parallel(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in SYNTAX_PARALLEL_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        findings.push(Finding::new(
            SYNTAX_PARALLEL_RULE_ID,
            "syntax_parallel",
            "major",
            start_char,
            end_char,
            0.95,
            "Liên từ 'nhưng' nối hai vế bất đối xứng: Danh ngữ đứng trước đối lập với cụm vị từ cụt phía sau.",
            "linguistic",
            norm[m.start()..m.end()].to_string(),
        ));
    }

    findings
}

/// Bắt lỗi ghép nhầm hai cấu trúc diễn ngôn 'Như... đã biết' và '... đã biết rằng'.
pub fn evaluate_syntax_cliche(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in SYNTAX_CLICHE_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        findings.push(Finding::new(
            SYNTAX_CLICHE_RULE_ID,
            "syntax_cliche",
            "minor",
            start_char,
            end_char,
            0.95,
            "Ghép nhầm hai kiểu câu: 'Như... đã biết' và '... đã biết rằng'.",
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
    fn test_syntax_reporting() {
        let text = "Phải nói là một quyết định sáng suốt.";
        let findings = evaluate_syntax_reporting(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, SYNTAX_REPORTING_RULE_ID);
    }

    #[test]
    fn test_syntax_parallel() {
        let text = "Một kế hoạch hoàn hảo nhưng không khả thi.";
        let findings = evaluate_syntax_parallel(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, SYNTAX_PARALLEL_RULE_ID);
    }

    #[test]
    fn test_syntax_cliche() {
        let text = "Như chúng ta đã biết rằng việc này rất khó.";
        let findings = evaluate_syntax_cliche(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, SYNTAX_CLICHE_RULE_ID);
    }
}
