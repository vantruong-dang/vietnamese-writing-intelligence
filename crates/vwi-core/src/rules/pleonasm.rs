use crate::types::Finding;
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};
use regex::RegexBuilder;
use std::sync::LazyLock;

static LOGICAL_PLEONASM_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\b(tối ưu nhất|ánh nắng mặt trời|hoàn toàn tuyệt đối|nguyên nhân\s+[^,.]+?\s+là bởi vì)\b")
        .case_insensitive(true)
        .build()
        .expect("Regex LOGICAL_PLEONASM không hợp lệ")
});

pub const LOGICAL_PLEONASM_RULE_ID: &str = "LOGICAL_PLEONASM";

/// Đánh giá lỗi Trùng ngữ logic (Logical Pleonasm): Lặp ý tố biểu đạt thừa ('tối ưu nhất', 'ánh nắng mặt trời'...).
pub fn evaluate_logical_pleonasm(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in LOGICAL_PLEONASM_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        let span = norm[m.start()..m.end()].to_string();
        let lower_span = span.to_lowercase();
        let suggestion = if lower_span.contains("tối ưu nhất") {
            "tối ưu (hoặc 'tốt nhất')"
        } else if lower_span.contains("ánh nắng mặt trời") {
            "ánh nắng (hoặc 'nắng')"
        } else if lower_span.contains("hoàn toàn tuyệt đối") {
            "hoàn toàn (hoặc 'tuyệt đối')"
        } else if lower_span.contains("là bởi vì") {
            "là do (hoặc chỉ dùng 'vì')"
        } else {
            "Lược bỏ ý tố biểu đạt thừa"
        };

        findings.push(
            Finding::new(
                LOGICAL_PLEONASM_RULE_ID,
                "pleonasm",
                "minor",
                start_char,
                end_char,
                0.95,
                "Trùng ngữ ngữ nghĩa (pleonasm): Lặp thừa ý tố biểu đạt (ví dụ: 'tối' đã là nhất, 'nắng' đã là của mặt trời).",
                "linguistic",
                span,
            )
            .with_suggestion(suggestion),
        );
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toi_uu_nhat_detection() {
        let text = "Đây là giải pháp kỹ thuật tối ưu nhất cho bài toán mở rộng hạ tầng.";
        let findings = evaluate_logical_pleonasm(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, LOGICAL_PLEONASM_RULE_ID);
        assert_eq!(findings[0].matched_span, "tối ưu nhất");
    }

    #[test]
    fn test_anh_nang_mat_troi_detection() {
        let text = "Ánh nắng mặt trời buổi sớm chiếu qua khe cửa sổ rực rỡ.";
        let findings = evaluate_logical_pleonasm(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].matched_span, "Ánh nắng mặt trời");
    }

    #[test]
    fn test_nguyen_nhan_la_boi_vi_detection() {
        let text = "Nguyên nhân sự cố mạng là bởi vì máy chủ quá tải.";
        let findings = evaluate_logical_pleonasm(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].matched_span, "Nguyên nhân sự cố mạng là bởi vì");
    }
}
