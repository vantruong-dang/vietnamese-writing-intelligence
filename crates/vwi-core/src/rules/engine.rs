use crate::types::Finding;

/// Động cơ cảm biến ngôn ngữ học tổng hợp (Linguistic Sensor Engine) thuần Rust.
/// Chạy toàn bộ các luật ngữ pháp chức năng qua Pipeline tách câu độc lập (Sentence-Bounded Scope),
/// chuẩn hóa chính tả Nghị định 30 và trả về danh sách các vi phạm được sắp xếp theo vị trí ký tự.
pub fn audit_linguistics(text: &str) -> Vec<Finding> {
    crate::auditor::Auditor::default().audit(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_linguistics_multiple_findings() {
        let text = "Trong tình hình hiện nay đòi hỏi giải pháp tối ưu nhất được triển khai bởi nhóm.";
        let findings = audit_linguistics(text);

        // Phải bắt được cả 3 lỗi: Khung đề què, Tối ưu nhất, Bị động nhân tạo
        assert!(findings.len() >= 3);
        let ids: Vec<&str> = findings.iter().map(|f| f.pattern_id.as_str()).collect();
        assert!(ids.contains(&"CRIPPLED_PREPOSITIONAL_TOPIC"));
        assert!(ids.contains(&"LOGICAL_PLEONASM"));
        assert!(ids.contains(&"ARTIFICIAL_PASSIVE_BE_BY"));
    }

    #[test]
    fn test_audit_linguistics_all_categories() {
        let text = "Không thể phủ nhận rằng ngày mai, tuy trời mưa nhưng dự án đã và đang là chiếc chìa khóa vạn năng.";
        let findings = audit_linguistics(text);
        let ids: Vec<&str> = findings.iter().map(|f| f.pattern_id.as_str()).collect();

        assert!(ids.contains(&"AI_OPENING_CLICHE"));
        assert!(ids.contains(&"FACTUAL_CONCESSIVE_MISUSE"));
        assert!(ids.contains(&"CXH_VERBAL_SERIES_TENSE_FALSE_MARKER"));
        assert!(ids.contains(&"FORMULAIC_METAPHOR"));
    }

    #[test]
    fn test_audit_linguistics_bureaucracy_and_dummy_it() {
        let text = "Nó là rất quan trọng để tiến hành thực hiện phương án mang tính chất đột phá này.";
        let findings = audit_linguistics(text);
        let ids: Vec<&str> = findings.iter().map(|f| f.pattern_id.as_str()).collect();

        assert!(ids.contains(&"DUMMY_IT_TRANSLATIONESE"));
        assert!(ids.contains(&"BUREAUCRATIC_VERB_PADDING"));
        assert!(ids.contains(&"ADJECTIVAL_SUFFIX_FAKE"));
    }
}
