use crate::types::Finding;
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};
use regex::RegexBuilder;
use std::sync::LazyLock;

static LOGICAL_PLEONASM_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\b(tối\s+ưu\s+nhất|hoàn\s+hảo\s+nhất|ánh\s+nắng\s+mặt\s+trời|hoàn\s+toàn\s+tuyệt\s+đối|kết\s+hợp\s+lại\s+với\s+nhau|cuối\s+cùng\s+kết\s+thúc|bắt\s+đầu\s+khởi\s+công|đồng\s+thời\s+cùng\s+lúc|tái\s+diễn\s+lại|lặp\s+lại\s+nhiều\s+lần|(?:nguyên\s+nhân|lý\s+do)(?:\s+[^,.]+?)?\s+là\s+bởi\s+vì|ăn\s+cắp\s+một\s+cách\s+gian\s+lận|lừa\s+dối\s+một\s+cách\s+thiếu\s+trung\s+thực|mục\s+đích(?:\s+[^,.]+?)?\s+là\s+nhằm\s+để)\b")
        .case_insensitive(true)
        .build()
        .expect("Regex LOGICAL_PLEONASM không hợp lệ")
});

pub const LOGICAL_PLEONASM_RULE_ID: &str = "LOGICAL_PLEONASM";
pub const EXCESSIVE_SU_RULE_ID: &str = "EXCESSIVE_SU_REPETITION";

static SU_WORD_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\bsự\b")
        .case_insensitive(true)
        .build()
        .expect("Regex EXCESSIVE_SU không hợp lệ")
});

/// Phát hiện lạm dụng danh từ hóa bằng cách đếm mật độ chữ 'sự' trong một câu / đoạn văn (>= 4 chữ sự).
pub fn evaluate_excessive_su(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    let mut su_matches: Vec<regex::Match> = Vec::new();
    for m in SU_WORD_RE.find_iter(&norm) {
        su_matches.push(m);
    }

    if su_matches.len() >= 4 {
        let first = su_matches.first().unwrap();
        let last = su_matches.last().unwrap();
        let (start_char, end_char) = mapper.convert_range(first.start(), last.end());
        let span = norm[first.start()..last.end()].to_string();

        findings.push(
            Finding::new(
                EXCESSIVE_SU_RULE_ID,
                "pleonasm",
                "minor",
                start_char,
                end_char,
                0.85,
                format!(
                    "Lạm dụng danh từ hóa: Xuất hiện {} chữ 'sự' trong một câu/đoạn khiến câu văn cồng kềnh, mất tính hành động tự nhiên.",
                    su_matches.len()
                ),
                "linguistic",
                span,
            )
            .with_suggestion("Chuyển các cụm 'sự + Động từ/Tính từ' thành động từ/tính từ trực tiếp (ví dụ: 'sự phát triển' -> 'phát triển')"),
        );
    }

    findings
}

/// Đánh giá lỗi Trùng ngữ logic (Logical Pleonasm): Lặp ý tố biểu đạt thừa ('tối ưu nhất', 'ánh nắng mặt trời'...).
pub fn evaluate_logical_pleonasm(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in LOGICAL_PLEONASM_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        let span = norm[m.start()..m.end()].to_string();
        let lower_span = span.to_lowercase();
        let (note, replacement) = if lower_span.contains("tối ưu nhất") {
            ("Nên dùng 'tối ưu' hoặc 'tốt nhất' thay vì ghép thừa chữ 'nhất'.", Some("tối ưu"))
        } else if lower_span.contains("hoàn hảo nhất") {
            ("Nên dùng 'hoàn hảo' vì bản thân đã là tuyệt đối, không ghép thêm 'nhất'.", Some("hoàn hảo"))
        } else if lower_span.contains("ánh nắng mặt trời") {
            ("Nên dùng 'ánh nắng' hoặc 'nắng' vì nắng vốn từ mặt trời.", Some("ánh nắng"))
        } else if lower_span.contains("hoàn toàn tuyệt đối") {
            ("Nên dùng 'hoàn toàn' hoặc 'tuyệt đối'.", Some("hoàn toàn"))
        } else if lower_span.contains("kết hợp lại với nhau") {
            ("Nên rút gọn thành 'kết hợp với nhau'.", Some("kết hợp với nhau"))
        } else if lower_span.contains("cuối cùng kết thúc") {
            ("Nên dùng 'kết thúc' hoặc 'cuối cùng'.", Some("kết thúc"))
        } else if lower_span.contains("bắt đầu khởi công") {
            ("Nên dùng 'khởi công' hoặc 'bắt đầu'.", Some("khởi công"))
        } else if lower_span.contains("đồng thời cùng lúc") {
            ("Nên dùng 'đồng thời' hoặc 'cùng lúc'.", Some("đồng thời"))
        } else if lower_span.contains("tái diễn lại") {
            ("Nên dùng 'tái diễn' vì 'tái' vốn đã là lặp lại.", Some("tái diễn"))
        } else if lower_span.contains("lặp lại nhiều lần") {
            ("Nên dùng 'lặp lại' hoặc 'nhiều lần'.", Some("lặp lại"))
        } else if lower_span.contains("là bởi vì") {
            ("Nên rút gọn thành 'là do' hoặc chỉ dùng 'vì'.", Some("là do"))
        } else {
            ("Lược bỏ ý tố biểu đạt thừa.", None)
        };

        let mut finding = Finding::new(
            LOGICAL_PLEONASM_RULE_ID,
            "pleonasm",
            "minor",
            start_char,
            end_char,
            0.95,
            "Trùng ngữ ngữ nghĩa (pleonasm): Lặp thừa ý tố biểu đạt (ví dụ: 'tối' đã là nhất, 'nắng' đã là của mặt trời).",
            "linguistic",
            span,
        );

        if let Some(repl) = replacement {
            finding = finding.with_note_and_fix(note, start_char, end_char, repl);
        } else {
            finding = finding.with_note(note);
        }

        findings.push(finding);
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
