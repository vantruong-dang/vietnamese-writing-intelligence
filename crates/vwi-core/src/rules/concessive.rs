use crate::types::Finding;
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};
use regex::RegexBuilder;
use std::sync::LazyLock;

static FACTUAL_CONCESSIVE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(
        r"(?:\b(ngày mai|tuần tới|năm sau|sắp tới|sau này)[,\s]+(tuy|mặc dù|mặc dầu)\b|\b(tuy|mặc dù|mặc dầu)\b[^,.]+?\b(đi chăng nữa|chăng nữa|đến mấy|thế nào chăng nữa)\b)",
    )
    .case_insensitive(true)
    .build()
    .expect("Regex FACTUAL_CONCESSIVE_MISUSE không hợp lệ")
});

static HYPOTHETICAL_CONCESSIVE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\b(hôm qua|năm ngoái|tuần trước|vừa qua|thực tế là|rõ ràng là)[,\s]+dù\b")
        .case_insensitive(true)
        .build()
        .expect("Regex HYPOTHETICAL_CONCESSIVE_MISUSE không hợp lệ")
});

static CONCESSIVE_INVERSION_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\b(mặc\s+dù|dẫu\s+rằng|cho\s+dù)\b([^,.!?]+?)\b(tuy\s+nhiên|nhưng\s+mà)\b")
        .case_insensitive(true)
        .build()
        .expect("Regex CONCESSIVE_INVERSION_CONFLICT không hợp lệ")
});

pub const FACTUAL_CONCESSIVE_RULE_ID: &str = "FACTUAL_CONCESSIVE_MISUSE";
pub const HYPOTHETICAL_CONCESSIVE_RULE_ID: &str = "HYPOTHETICAL_CONCESSIVE_MISUSE";
pub const CONCESSIVE_INVERSION_RULE_ID: &str = "CONCESSIVE_INVERSION_CONFLICT";

/// Đánh giá lỗi Nhượng bộ Hiện thực dùng cho Giả định tương lai (Factual Concessive Misuse).
pub fn evaluate_factual_concessive_misuse(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in FACTUAL_CONCESSIVE_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        findings.push(Finding::new(
            FACTUAL_CONCESSIVE_RULE_ID,
            "concessive_aspect",
            "major",
            start_char,
            end_char,
            1.0,
            "Sự việc tương lai giả định (phi hiện thực). 'Tuy' chỉ dùng cho sự việc đã/đang diễn ra; giả định bắt buộc dùng 'Dù'.",
            "linguistic",
            norm[m.start()..m.end()].to_string(),
        ));
    }

    findings
}

/// Đánh giá lỗi Nhượng bộ Giả định dùng cho Hiện thực quá khứ (Hypothetical Concessive Misuse).
pub fn evaluate_hypothetical_concessive_misuse(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in HYPOTHETICAL_CONCESSIVE_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        findings.push(Finding::new(
            HYPOTHETICAL_CONCESSIVE_RULE_ID,
            "concessive_aspect",
            "major",
            start_char,
            end_char,
            0.95,
            "Sự việc quá khứ là hiện thực xác định. Bắt buộc dùng 'Tuy/Mặc dù', không dùng liên từ giả định 'Dù'.",
            "linguistic",
            norm[m.start()..m.end()].to_string(),
        ));
    }

    findings
}

/// Đánh giá lỗi xung đột nhượng bộ kép "Mặc dù/Dẫu rằng ... tuy nhiên/nhưng mà".
pub fn evaluate_concessive_inversion(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    evaluate_concessive_inversion_with_map(&norm, &mapper)
}

/// Phiên bản tối ưu nhận sẵn CharOffsetMap.
pub fn evaluate_concessive_inversion_with_map(norm: &str, mapper: &CharOffsetMap) -> Vec<Finding> {
    let mut findings = Vec::new();

    for m in CONCESSIVE_INVERSION_RE.find_iter(norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        findings.push(Finding::new(
            CONCESSIVE_INVERSION_RULE_ID,
            "concessive",
            "major",
            start_char,
            end_char,
            0.95,
            "Xung đột nhượng bộ kép ('Mặc dù... tuy nhiên'). Đã có 'mặc dù' thì không dùng 'tuy nhiên' ở vế đối lập.",
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
    fn test_factual_concessive_future() {
        let text = "Ngày mai, tuy trời mưa to, tôi cũng sẽ đến đúng giờ.";
        let findings = evaluate_factual_concessive_misuse(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, FACTUAL_CONCESSIVE_RULE_ID);
        assert_eq!(findings[0].matched_span, "Ngày mai, tuy");
        assert_eq!(findings[0].confidence, 1.0);
    }

    #[test]
    fn test_hypothetical_concessive_past() {
        let text = "Hôm qua, dù trời mưa rất to, trận đấu vẫn diễn ra sôi nổi.";
        let findings = evaluate_hypothetical_concessive_misuse(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, HYPOTHETICAL_CONCESSIVE_RULE_ID);
        assert_eq!(findings[0].matched_span, "Hôm qua, dù");
    }

    #[test]
    fn test_concessive_inversion() {
        let text = "Mặc dù anh ấy rất cố gắng tuy nhiên kết quả vẫn chưa đạt.";
        let findings = evaluate_concessive_inversion(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, CONCESSIVE_INVERSION_RULE_ID);
    }
}
