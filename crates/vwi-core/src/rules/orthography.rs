//! Module kiểm tra lỗi chính tả cơ bản, dấu câu và quy chuẩn văn thư tiếng Việt
//! Căn cứ: Nghị định 30/2020/NĐ-CP (Công tác văn thư) & Thông tư 20/2018/TT-BGDĐT

use crate::types::Finding;
use crate::unicode_utils::normalize_nfc;

pub const PUNCTUATION_SPACE_BEFORE_RULE_ID: &str = "PUNCTUATION_SPACE_BEFORE";
pub const PUNCTUATION_NO_SPACE_AFTER_RULE_ID: &str = "PUNCTUATION_NO_SPACE_AFTER";
pub const BRACKET_SPACE_INSIDE_RULE_ID: &str = "BRACKET_SPACE_INSIDE";
pub const SENTENCE_CAPITALIZATION_RULE_ID: &str = "SENTENCE_CAPITALIZATION";
pub const TELEX_TYPO_ERROR_RULE_ID: &str = "TELEX_TYPO_ERROR";

pub struct OrthographyChecker;

impl OrthographyChecker {
    /// Đánh dấu các khoảng ký tự là URL, email hoặc tên miền (không áp dụng luật dấu câu / viết hoa)
    fn compute_ignored_char_mask(chars: &[char]) -> Vec<bool> {
        let len = chars.len();
        let mut mask = vec![false; len];

        let s: String = chars.iter().collect();
        // Tìm các URL bắt đầu bằng http://, https://, ftp://, www.
        let prefixes = ["http://", "https://", "ftp://", "www."];
        
        let mut byte_idx = 0;
        for (char_idx, &c) in chars.iter().enumerate() {
            let rem_str = &s[byte_idx..];
            for prefix in &prefixes {
                if rem_str.starts_with(prefix) {
                    // Đánh dấu từ vị trí này cho đến khi gặp khoảng trắng hoặc ngoặc nhọn/ngoặc đơn ở cuối
                    let mut k = char_idx;
                    while k < len && !chars[k].is_whitespace() && !matches!(chars[k], '>' | '"' | '\'') {
                        mask[k] = true;
                        k += 1;
                    }
                }
            }
            byte_idx += c.len_utf8();
        }

        mask
    }

    /// Kiểm tra lỗi khoảng trắng quanh các dấu câu: , . ; : ! ? và trong ngoặc đơn
    pub fn check_punctuation_spacing(raw_text: &str) -> Vec<Finding> {
        let norm = normalize_nfc(raw_text);
        let chars: Vec<char> = norm.chars().collect();
        let len = chars.len();
        let ignored_mask = Self::compute_ignored_char_mask(&chars);
        let mut findings = Vec::new();

        for i in 0..len {
            if ignored_mask[i] {
                continue;
            }

            let c = chars[i];

            // 1. Lỗi: Có khoảng trắng TRƯỚC dấu câu: "chào , bạn" hoặc "sao ?"
            if matches!(c, ',' | '.' | ';' | ':' | '!' | '?') {
                if i > 0 && chars[i - 1] == ' ' {
                    // Ngoại lệ: Dấu ba chấm "..."
                    let is_ellipsis = c == '.'
                        && ((i + 1 < len && chars[i + 1] == '.')
                            || (i > 1 && chars[i - 2] == '.'));
                    if !is_ellipsis {
                        let matched_span: String = chars[i - 1..=i].iter().collect();
                        findings.push(
                            Finding::new(
                                PUNCTUATION_SPACE_BEFORE_RULE_ID,
                                "orthography",
                                "major",
                                i - 1,
                                i + 1,
                                0.98,
                                format!(
                                    "Thừa khoảng trắng trước dấu '{}'. Đúng chuẩn văn thư: viết sát vào chữ phía trước.",
                                    c
                                ),
                                "orthography",
                                matched_span,
                            )
                            .with_suggestion(c.to_string()),
                        );
                    }
                }

                // 2. Lỗi: THIẾU khoảng trắng SAU dấu câu: "chào bạn.tôi là"
                if i + 1 < len {
                    let next_c = chars[i + 1];
                    // Phía sau dấu câu phải là khoảng trắng, xuống dòng, dấu ngoặc hoặc dấu câu khác
                    if !next_c.is_whitespace()
                        && !matches!(
                            next_c,
                            ')' | ']' | '}' | '"' | '”' | '»' | '\'' | '’' | ',' | '.' | '!' | '?' | ';' | ':' | '/' | '-' | '…'
                        )
                    {
                        // Ngoại lệ 1: Số thập phân hoặc phân cách hàng nghìn (3.14 hoặc 10,5 hoặc 150.000)
                        let is_numeric = (c == '.' || c == ',')
                            && i > 0
                            && chars[i - 1].is_ascii_digit()
                            && next_c.is_ascii_digit();

                        // Ngoại lệ 2: Dấu ba chấm "..."
                        let is_ellipsis = c == '.'
                            && (next_c == '.' || (i > 0 && chars[i - 1] == '.'));

                        if !is_numeric && !is_ellipsis {
                            let matched_span: String = chars[i..=i + 1].iter().collect();
                            findings.push(
                                Finding::new(
                                    PUNCTUATION_NO_SPACE_AFTER_RULE_ID,
                                    "orthography",
                                    "major",
                                    i,
                                    i + 2,
                                    0.96,
                                    format!(
                                        "Thiếu khoảng trắng sau dấu '{}'. Cần cách một dấu cách trước từ tiếp theo.",
                                        c
                                    ),
                                    "orthography",
                                    matched_span,
                                )
                                .with_suggestion(format!("{} {}", c, next_c)),
                            );
                        }
                    }
                }
            }

            // 3. Lỗi: Khoảng trắng thừa bên trong ngoặc đơn: "( nội dung )"
            if c == '(' && i + 1 < len && chars[i + 1] == ' ' {
                let matched_span: String = chars[i..=i + 1].iter().collect();
                findings.push(
                    Finding::new(
                        BRACKET_SPACE_INSIDE_RULE_ID,
                        "orthography",
                        "minor",
                        i,
                        i + 2,
                        0.95,
                        "Thừa khoảng trắng sau dấu mở ngoặc '('. Ký tự đầu phải viết sát dấu ngoặc.",
                        "orthography",
                        matched_span,
                    )
                    .with_suggestion("(".to_string()),
                );
            }
            if c == ')' && i > 0 && chars[i - 1] == ' ' {
                let matched_span: String = chars[i - 1..=i].iter().collect();
                findings.push(
                    Finding::new(
                        BRACKET_SPACE_INSIDE_RULE_ID,
                        "orthography",
                        "minor",
                        i - 1,
                        i + 1,
                        0.95,
                        "Thừa khoảng trắng trước dấu đóng ngoặc ')'. Chữ cuối phải viết sát dấu ngoặc.",
                        "orthography",
                        matched_span,
                    )
                    .with_suggestion(")".to_string()),
                );
            }
        }

        findings
    }

    /// Kiểm tra viết hoa đầu câu sau các dấu kết thúc: . ! ? hoặc ở đầu văn bản
    pub fn check_sentence_capitalization(raw_text: &str) -> Vec<Finding> {
        let norm = normalize_nfc(raw_text);
        let chars: Vec<char> = norm.chars().collect();
        let len = chars.len();
        let ignored_mask = Self::compute_ignored_char_mask(&chars);
        let mut findings = Vec::new();

        let mut require_cap = true;
        let mut idx = 0;

        while idx < len {
            if ignored_mask[idx] {
                idx += 1;
                continue;
            }

            let c = chars[idx];

            if matches!(c, '.' | '!' | '?') {
                // Ngoại lệ 1: Số thập phân hoặc số phân cách hàng nghìn (ví dụ: 150.000.000 hoặc 3.14)
                if c == '.' && idx > 0 && chars[idx - 1].is_ascii_digit() && (idx + 1 < len && chars[idx + 1].is_ascii_digit()) {
                    idx += 1;
                    continue;
                }

                // Kiểm tra xem có phải dấu ba chấm không
                let mut next_idx = idx + 1;
                while next_idx < len && chars[next_idx] == '.' {
                    next_idx += 1;
                }

                // Ngoại lệ: Viết tắt "v.v."
                let is_vv = idx >= 3
                    && chars[idx - 3] == 'v'
                    && chars[idx - 2] == '.'
                    && chars[idx - 1] == 'v'
                    && c == '.';

                if !is_vv {
                    require_cap = true;
                }
                idx = next_idx;
                continue;
            }

            // Bỏ qua khoảng trắng, ngoặc, nháy
            if c.is_whitespace() || matches!(c, '(' | '[' | '{' | '"' | '”' | '“' | '«' | '»' | '\'' | '’' | '`') {
                idx += 1;
                continue;
            }

            if c.is_alphabetic() {
                if require_cap && c.is_lowercase() {
                    findings.push(
                        Finding::new(
                            SENTENCE_CAPITALIZATION_RULE_ID,
                            "orthography",
                            "major",
                            idx,
                            idx + 1,
                            0.95,
                            format!("Chữ cái đầu câu '{}' chưa được viết hoa.", c),
                            "orthography",
                            c.to_string(),
                        )
                        .with_suggestion(c.to_uppercase().to_string()),
                    );
                }
                require_cap = false;
            } else if c.is_ascii_digit() {
                // Nếu câu bắt đầu bằng số (ví dụ: "10 người..."), bỏ qua yêu cầu viết hoa
                require_cap = false;
            }

            idx += 1;
        }

        findings
    }

    /// Bắt lỗi dính phím Telex kinh điển: dduowjc, cuar, bieets, toi...
    pub fn check_telex_accidents(raw_text: &str) -> Vec<Finding> {
        let norm = normalize_nfc(raw_text);
        let mut findings = Vec::new();

        let telex_patterns: &[(&str, &str)] = &[
            ("dduowjc", "được"),
            ("dduwowjc", "được"),
            ("dduocj", "được"),
            ("cuar", "của"),
            ("bieets", "biết"),
            ("tooi", "tôi"),
            ("nguwowfi", "người"),
            ("nhuwng", "nhưng"),
            ("khoong", "không"),
            ("phair", "phải"),
            ("ddang", "đang"),
            ("dax", "đã"),
            ("moij", "mọi"),
            ("lafm", "làm"),
            ("roofi", "rồi"),
            ("nhaat", "nhất"),
            ("luoon", "luôn"),
            ("trangj", "trạng"),
            ("ddiefe", "điều"),
            ("nghiax", "nghĩa"),
        ];

        let chars: Vec<char> = norm.chars().collect();
        let text_lower = norm.to_lowercase();
        let lower_chars: Vec<char> = text_lower.chars().collect();
        let total_chars = chars.len();

        for &(typo, fix) in telex_patterns {
            let typo_len = typo.chars().count();
            if total_chars < typo_len {
                continue;
            }

            for start in 0..=(total_chars - typo_len) {
                let slice: String = lower_chars[start..start + typo_len].iter().collect();
                if slice == typo {
                    // Kiểm tra ranh giới từ (Word Boundary) để tránh bắt nhầm trong từ tiếng Anh
                    let prev_ok = start == 0 || !lower_chars[start - 1].is_alphabetic();
                    let next_ok = (start + typo_len == total_chars)
                        || !lower_chars[start + typo_len].is_alphabetic();

                    if prev_ok && next_ok {
                        let matched_span: String = chars[start..start + typo_len].iter().collect();
                        findings.push(
                            Finding::new(
                                TELEX_TYPO_ERROR_RULE_ID,
                                "orthography",
                                "critical",
                                start,
                                start + typo_len,
                                0.99,
                                format!(
                                    "Lỗi dính phím gõ Telex: '{}'. Bạn có ý muốn gõ '{}'?",
                                    matched_span, fix
                                ),
                                "orthography",
                                matched_span,
                            )
                            .with_suggestion(fix.to_string()),
                        );
                    }
                }
            }
        }

        findings
    }
}

/// Hàm thẩm định tổng hợp toàn bộ các quy tắc chính tả văn thư
pub fn evaluate_orthography(text: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(OrthographyChecker::check_punctuation_spacing(text));
    findings.extend(OrthographyChecker::check_sentence_capitalization(text));
    findings.extend(OrthographyChecker::check_telex_accidents(text));
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orthography_spacing_before_punct() {
        let text = "xin chào , bạn";
        let findings = OrthographyChecker::check_punctuation_spacing(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, PUNCTUATION_SPACE_BEFORE_RULE_ID);
        assert_eq!(findings[0].suggestion.as_deref(), Some(","));
    }

    #[test]
    fn test_orthography_spacing_after_punct() {
        let text = "hàng đầu.Đỉnh cao";
        let findings = OrthographyChecker::check_punctuation_spacing(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, PUNCTUATION_NO_SPACE_AFTER_RULE_ID);
        assert_eq!(findings[0].suggestion.as_deref(), Some(". Đ"));
    }

    #[test]
    fn test_orthography_bracket_space() {
        let text = "( công viên )";
        let findings = OrthographyChecker::check_punctuation_spacing(text);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].pattern_id, BRACKET_SPACE_INSIDE_RULE_ID);
        assert_eq!(findings[1].pattern_id, BRACKET_SPACE_INSIDE_RULE_ID);
    }

    #[test]
    fn test_orthography_capitalization() {
        let text = "hôm nay trời đẹp. mai trời mưa.";
        let findings = OrthographyChecker::check_sentence_capitalization(text);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].suggestion.as_deref(), Some("H"));
        assert_eq!(findings[1].suggestion.as_deref(), Some("M"));
    }

    #[test]
    fn test_orthography_telex() {
        let text = "Tôi không dduowjc đi chơi.";
        let findings = OrthographyChecker::check_telex_accidents(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, TELEX_TYPO_ERROR_RULE_ID);
        assert_eq!(findings[0].suggestion.as_deref(), Some("được"));
    }

    #[test]
    fn test_orthography_ultimate_sentence() {
        let text = "hôm nay trời đẹp quá .tôi muốn đi chơi ( công viên ) mà không dduowjc";
        let findings = evaluate_orthography(text);
        let ids: Vec<&str> = findings.iter().map(|f| f.pattern_id.as_str()).collect();

        assert!(ids.contains(&SENTENCE_CAPITALIZATION_RULE_ID));
        assert!(ids.contains(&PUNCTUATION_SPACE_BEFORE_RULE_ID));
        assert!(ids.contains(&BRACKET_SPACE_INSIDE_RULE_ID));
        assert!(ids.contains(&TELEX_TYPO_ERROR_RULE_ID));
    }
}
