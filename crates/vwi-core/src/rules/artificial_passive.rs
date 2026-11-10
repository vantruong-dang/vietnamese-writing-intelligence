use crate::types::Finding;
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};
use fancy_regex::Regex;
use std::sync::LazyLock;

static ARTIFICIAL_PASSIVE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(được|bị)\s+[^,.]+?\s+bởi(?!\s+(?:vì|thế|do|tại|vậy))\b")
        .expect("Mẫu regex ARTIFICIAL_PASSIVE_BE_BY không hợp lệ")
});

static REDUNDANT_BENEFICIARY_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::RegexBuilder::new(
        r"^(căn phòng|báo cáo|hệ thống|dữ liệu|bàn ghế|nhà cửa|cái bàn|tóc nó)\s+(?:đã|sẽ|đang)?\s*(?:được|bị)\s+(lau dọn|xem xét|khởi động|xử lý|lau|cắt)\b",
    )
    .case_insensitive(true)
    .build()
    .expect("Regex REDUNDANT_BENEFICIARY_PASSIVE không hợp lệ")
});

pub const RULE_ID: &str = "ARTIFICIAL_PASSIVE_BE_BY";
pub const ARTIFICIAL_PASSIVE_RULE_ID: &str = RULE_ID;
pub const REDUNDANT_BENEFICIARY_RULE_ID: &str = "REDUNDANT_BENEFICIARY_PASSIVE";
pub const CATEGORY: &str = "pseudo_passive";
pub const SEVERITY: &str = "major";
pub const RATIONALE: &str = "Câu bị động nhân tạo sao phỏng công thức 'be + V3 + by' của tiếng Anh.";
pub const SOURCE: &str = "linguistic";

/// Đánh giá câu bị động nhân tạo dùng fancy-regex (hỗ trợ negative lookahead tổng quát).
pub fn evaluate_artificial_passive(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in ARTIFICIAL_PASSIVE_RE.find_iter(&norm).flatten() {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        let matched_span = norm[m.start()..m.end()].to_string();

        findings.push(
            Finding::new(
                RULE_ID,
                CATEGORY,
                SEVERITY,
                start_char,
                end_char,
                0.95,
                RATIONALE,
                SOURCE,
                matched_span,
            )
            .with_suggestion("Chuyển sang câu chủ động hoặc dùng 'do [tác nhân] thực hiện' thay vì cấu trúc 'được/bị ... bởi'"),
        );
    }

    findings
}

/// Thuật toán quét thuần Rust (Zero-Regex Scanner) tối ưu cực độ:
/// Tránh hoàn toàn overhead backtracking của fancy-regex.
pub fn evaluate_artificial_passive_fast(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let lower = norm.to_lowercase();
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    let bytes = lower.as_bytes();
    let n = bytes.len();
    let mut i = 0;

    while i < n {
        // Tìm "được" hoặc "bị" ở ranh giới từ
        let is_duoc = lower[i..].starts_with("được");
        let is_bi = lower[i..].starts_with("bị");

        if (is_duoc || is_bi) && (i == 0 || is_word_boundary_char(lower[..i].chars().next_back().unwrap())) {
            let start_byte = i;
            let lead_len = if is_duoc { "được".len() } else { "bị".len() };
            let after_lead = i + lead_len;

            if after_lead < n && is_word_boundary_char(lower[after_lead..].chars().next().unwrap()) {
                // Quét tới dấu phẩy, dấu chấm hoặc tìm "bởi"
                let mut search_idx = after_lead;
                let mut found_boi = None;

                while search_idx < n {
                    let ch = lower[search_idx..].chars().next().unwrap();
                    if ch == ',' || ch == '.' || ch == ';' || ch == '?' || ch == '!' {
                        break;
                    }
                    if lower[search_idx..].starts_with("bởi") {
                        let prev_ok = is_word_boundary_char(lower[..search_idx].chars().next_back().unwrap());
                        let after_boi = search_idx + "bởi".len();
                        let next_ok = after_boi >= n || is_word_boundary_char(lower[after_boi..].chars().next().unwrap());

                        if prev_ok && next_ok {
                            found_boi = Some(after_boi);
                            break;
                        }
                    }
                    search_idx += ch.len_utf8();
                }

                if let Some(end_boi_byte) = found_boi {
                    // Kiểm tra bẫy liên từ: "bởi vì", "bởi thế", "bởi do", "bởi tại", "bởi vậy"
                    let rest = lower[end_boi_byte..].trim_start();
                    let is_trap = rest.starts_with("vì")
                        || rest.starts_with("thế")
                        || rest.starts_with("do")
                        || rest.starts_with("tại")
                        || rest.starts_with("vậy");

                    if !is_trap {
                        let (start_char, end_char) = mapper.convert_range(start_byte, end_boi_byte);
                        findings.push(
                            Finding::new(
                                RULE_ID,
                                CATEGORY,
                                SEVERITY,
                                start_char,
                                end_char,
                                0.95,
                                RATIONALE,
                                SOURCE,
                                norm[start_byte..end_boi_byte].to_string(),
                            )
                            .with_suggestion("Chuyển sang câu chủ động hoặc dùng 'do [tác nhân] thực hiện' thay vì cấu trúc 'được/bị ... bởi'"),
                        );
                    }
                }
            }
        }
        if let Some(ch) = lower[i..].chars().next() {
            i += ch.len_utf8();
        } else {
            break;
        }
    }

    findings
}

/// Đánh giá lỗi Thụ hưởng Dư thừa (Redundant Beneficiary Passive): Danh từ chỉ vật vô tri không cần 'được'.
pub fn evaluate_redundant_beneficiary_passive(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in REDUNDANT_BENEFICIARY_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        findings.push(
            Finding::new(
                REDUNDANT_BENEFICIARY_RULE_ID,
                "pseudo_passive",
                "minor",
                start_char,
                end_char,
                0.95,
                "Danh ngữ chỉ vật vô tri đứng làm Đề không cần vị từ chỉ sự thụ hưởng 'được'.",
                "linguistic",
                norm[m.start()..m.end()].to_string(),
            )
            .with_suggestion("Lược bỏ 'được/bị' vì chủ thể tự mang ý nghĩa thụ hưởng tự nhiên theo ngữ pháp GS. Cao Xuân Hạo"),
        );
    }

    findings
}

#[inline]
fn is_word_boundary_char(c: char) -> bool {
    !c.is_alphanumeric() && c != '_' && c != '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redundant_beneficiary_passive() {
        let text = "Căn phòng đã được lau dọn sạch sẽ trước khi khách đến.";
        let findings = evaluate_redundant_beneficiary_passive(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, REDUNDANT_BENEFICIARY_RULE_ID);
        assert_eq!(findings[0].matched_span, "Căn phòng đã được lau dọn");
    }

    #[test]
    fn test_catch_artificial_passive() {
        let text = "Dự án này được phê duyệt bởi hội đồng quản trị.";
        let findings = evaluate_artificial_passive(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, RULE_ID);
        assert_eq!(findings[0].matched_span, "được phê duyệt bởi");
        assert_eq!(findings[0].start, 10);
        assert_eq!(findings[0].end, 28);

        let fast_findings = evaluate_artificial_passive_fast(text);
        assert_eq!(fast_findings, findings);
    }

    #[test]
    fn test_catch_bi_passive() {
        let text = "Vấn đề này bị phát hiện bởi đội ngũ kiểm toán độc lập.";
        let findings = evaluate_artificial_passive(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].matched_span, "bị phát hiện bởi");

        let fast_findings = evaluate_artificial_passive_fast(text);
        assert_eq!(fast_findings, findings);
    }

    #[test]
    fn test_bypass_boi_vi_trap() {
        let text = "Dự án bị đình trệ bởi vì thiếu hụt nguồn vốn đầu tư.";
        let findings = evaluate_artificial_passive(text);
        assert!(findings.is_empty());

        let fast_findings = evaluate_artificial_passive_fast(text);
        assert!(fast_findings.is_empty());
    }

    #[test]
    fn test_bypass_boi_do_trap() {
        let text = "Anh ấy được hoan nghênh bởi do có nhiều đóng góp lớn.";
        let findings = evaluate_artificial_passive(text);
        assert!(findings.is_empty());

        let fast_findings = evaluate_artificial_passive_fast(text);
        assert!(fast_findings.is_empty());
    }

    #[test]
    fn test_clean_sentence() {
        let text = "Hội đồng quản trị đã phê duyệt dự án này.";
        let findings = evaluate_artificial_passive(text);
        assert!(findings.is_empty());

        let fast_findings = evaluate_artificial_passive_fast(text);
        assert!(fast_findings.is_empty());
    }
}
