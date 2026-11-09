use crate::types::Finding;
use crate::unicode_utils::{is_word_boundary, normalize_nfc, CharOffsetMap};
use daachorse::DoubleArrayAhoCorasick;
use std::sync::LazyLock;

pub const BUREAUCRATIC_VERB_PADDING_RULE_ID: &str = "BUREAUCRATIC_VERB_PADDING";
pub const REDUNDANT_CONNECTORS_RULE_ID: &str = "REDUNDANT_CONNECTORS";
pub const VAGUE_INTENSIFIER_RULE_ID: &str = "VAGUE_INTENSIFIER";

/// Danh sách sáo ngữ hành chính & đệm từ rỗng
struct ClicheRule {
    pattern: &'static str,
    #[allow(dead_code)]
    suggestion: &'static str,
    message: &'static str,
    rule_id: &'static str,
    severity: &'static str,
}

static BUREAUCRATIC_PATTERNS: &[ClicheRule] = &[
    // 1. Đệm động từ rỗng
    ClicheRule {
        pattern: "tiến hành thực hiện",
        suggestion: "thực hiện",
        message: "Lạm dụng đệm động từ rỗng hành chính ('tiến hành thực hiện'). Hãy dùng trực tiếp động từ chính.",
        rule_id: "BUREAUCRATIC_VERB_PADDING",
        severity: "minor",
    },
    ClicheRule {
        pattern: "triển khai công tác",
        suggestion: "triển khai",
        message: "Sáo ngữ hành chính rườm rà ('triển khai công tác').",
        rule_id: "BUREAUCRATIC_VERB_PADDING",
        severity: "minor",
    },
    ClicheRule {
        pattern: "đưa vào áp dụng",
        suggestion: "áp dụng",
        message: "Đệm động từ thừa ('đưa vào áp dụng'). Hãy dùng ngắn gọn 'áp dụng'.",
        rule_id: "BUREAUCRATIC_VERB_PADDING",
        severity: "minor",
    },
    // 2. Chêm liên từ thừa
    ClicheRule {
        pattern: "và đồng thời",
        suggestion: "đồng thời",
        message: "Chêm liên từ trùng lặp ('và đồng thời'). Chỉ cần dùng 'và' hoặc 'đồng thời'.",
        rule_id: "REDUNDANT_CONNECTORS",
        severity: "minor",
    },
    ClicheRule {
        pattern: "cũng như là",
        suggestion: "cũng như",
        message: "Thừa hư từ 'là' sau liên từ đẳng lập ('cũng như là').",
        rule_id: "REDUNDANT_CONNECTORS",
        severity: "minor",
    },
    // 3. Trợ từ tăng cường sáo rỗng
    ClicheRule {
        pattern: "vô cùng hết sức",
        suggestion: "rất",
        message: "Xếp chồng trợ từ cực cấp sáo rỗng ('vô cùng hết sức').",
        rule_id: "VAGUE_INTENSIFIER",
        severity: "minor",
    },
    ClicheRule {
        pattern: "thực sự rất là",
        suggestion: "rất",
        message: "Khẩu ngữ rườm rà trong văn viết chính luận ('thực sự rất là').",
        rule_id: "VAGUE_INTENSIFIER",
        severity: "minor",
    },
];

// Khởi tạo Trie Aho-Corasick tĩnh quét toàn bộ sáo ngữ trong một lượt O(N)
static CLICHE_TRIE: LazyLock<DoubleArrayAhoCorasick<usize>> = LazyLock::new(|| {
    let patterns: Vec<&str> = BUREAUCRATIC_PATTERNS.iter().map(|r| r.pattern).collect();
    DoubleArrayAhoCorasick::new(patterns).expect("Không thể build Bureaucracy Trie")
});

pub fn evaluate_bureaucratic_rules(raw_text: &str) -> Vec<Finding> {
    let normalized = normalize_nfc(raw_text);
    let lower_text = normalized.to_lowercase();
    let char_map = CharOffsetMap::new(&normalized);
    let mut findings = Vec::new();

    for m in CLICHE_TRIE.find_iter(&lower_text) {
        if !is_word_boundary(&lower_text, m.start(), m.end()) {
            continue;
        }
        let rule_idx = m.value();
        let rule = &BUREAUCRATIC_PATTERNS[rule_idx];
        let (start, end) = char_map.byte_range_to_char_range(m.start(), m.end());

        findings.push(Finding::new(
            rule.rule_id,
            "bureaucracy",
            rule.severity,
            start,
            end,
            0.95,
            rule.message,
            "linguistic",
            normalized[m.start()..m.end()].to_string(),
        ));
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bureaucratic_trie_scanner() {
        let text = "Ban chỉ đạo yêu cầu tiến hành thực hiện kế hoạch và đồng thời đưa vào áp dụng công nghệ mới.";
        let findings = evaluate_bureaucratic_rules(text);

        assert_eq!(findings.len(), 3);
        assert_eq!(findings[0].pattern_id, "BUREAUCRATIC_VERB_PADDING");
        assert_eq!(findings[1].pattern_id, "REDUNDANT_CONNECTORS");
        assert_eq!(findings[2].pattern_id, "BUREAUCRATIC_VERB_PADDING");
    }

    #[test]
    fn test_bureaucratic_intensifier() {
        let text = "Vấn đề này vô cùng hết sức cấp bách.";
        let findings = evaluate_bureaucratic_rules(text);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, "VAGUE_INTENSIFIER");
        assert_eq!(findings[0].matched_span, "vô cùng hết sức");
    }
}
