//! # TỪ ĐIỂN LỖI CHÍNH TẢ PHỔ BIẾN (COMMON MISSPELLING DICTIONARY)
//!
//! Bắt các lỗi chính tả / gõ nhầm **đã được kiểm duyệt thủ công** mà từ điển
//! âm tiết và phonotactics không thể phát hiện: `sử lý` → `xử lý`,
//! `bổ xung` → `bổ sung`, `chỉnh chu` → `chỉn chu`, `chuẩn đoán` → `chẩn đoán`,
//! `thăm quan` → `tham quan`, `dành giật` → `giành giật`...
//!
//! ## Vì sao dùng binary search thay vì Aho-Corasick?
//! Bảng chỉ vài trăm cặp, tra cứu trên cửa sổ 2–3 từ liên tiếp là đủ và rất rẻ:
//! `O(số_từ × log N)`, không cấp phát trie, không tăng RAM đáng kể.
//!
//! ## Chống bắt oan
//! - Chỉ so khớp **nguyên cụm từ** trên ranh giới từ (token), không so chuỗi con.
//! - Bảng dữ liệu đã loại các cặp "cả hai đều đúng nhưng khác nghĩa"
//!   (`bàng quan`/`bàng quang`, `dành dụm`/`giành giật`, `trí thức`/`tri thức`...).
//! - Không sửa khi cụm nằm trong vùng đã được masking (URL, code, markdown...).

use crate::types::Finding;
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};

include!("common_typos_data.rs");

pub const COMMON_MISSPELLING_RULE_ID: &str = "COMMON_MISSPELLING";

#[derive(Debug)]
struct Token<'a> {
    text: &'a str,
    start_byte: usize,
    end_byte: usize,
}

/// Tách token theo runs of alphanumeric. Giữ nguyên byte offset để quy đổi
/// sang char offset (Rust API dùng char, không dùng byte).
fn tokenize(text: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut start: Option<usize> = None;

    for (idx, c) in text.char_indices() {
        if c.is_alphanumeric() {
            if start.is_none() {
                start = Some(idx);
            }
        } else if let Some(s) = start.take() {
            tokens.push(Token {
                text: &text[s..idx],
                start_byte: s,
                end_byte: idx,
            });
        }
    }
    if let Some(s) = start {
        tokens.push(Token {
            text: &text[s..],
            start_byte: s,
            end_byte: text.len(),
        });
    }
    tokens
}

/// Bảo toàn kiểu chữ của một từ: ALL CAPS → ALL CAPS, Title → Title, còn lại giữ.
fn preserve_word_case(original: &str, replacement: &str) -> String {
    let alpha = original.chars().filter(|c| c.is_alphabetic()).count();
    let upper = original.chars().filter(|c| c.is_uppercase()).count();
    if alpha > 0 && upper == alpha {
        return replacement.to_uppercase();
    }
    let mut chars = replacement.chars();
    match chars.next() {
        Some(first) if original.chars().next().is_some_and(|c| c.is_uppercase()) => {
            first.to_uppercase().collect::<String>() + chars.as_str()
        }
        _ => replacement.to_string(),
    }
}

/// Bảo toàn kiểu chữ theo từng từ của cả cụm (`Sử Lý` → `Xử Lý`, `BỔ XUNG` → `BỔ SUNG`).
fn preserve_phrase_case(original: &str, replacement: &str) -> String {
    let orig_words: Vec<&str> = original.split_whitespace().collect();
    let repl_words: Vec<&str> = replacement.split_whitespace().collect();
    if !orig_words.is_empty() && orig_words.len() == repl_words.len() {
        return repl_words
            .iter()
            .zip(orig_words.iter())
            .map(|(repl, orig)| preserve_word_case(orig, repl))
            .collect::<Vec<_>>()
            .join(" ");
    }
    preserve_word_case(original, replacement)
}

fn lookup(phrase: &str) -> Option<&'static str> {
    COMMON_TYPO_PAIRS
        .binary_search_by_key(&phrase, |(wrong, _)| *wrong)
        .ok()
        .map(|idx| COMMON_TYPO_PAIRS[idx].1)
}

/// Tra cứu công khai cho tầng cascade: cụm sai -> cụm đúng.
pub fn lookup_common_typo(phrase: &str) -> Option<&'static str> {
    let normalized = normalize_nfc(phrase).to_lowercase();
    lookup(&normalized)
}

/// Quét toàn văn bản, trả về các finding lỗi chính tả phổ biến.
pub fn evaluate_common_misspellings(raw_text: &str) -> Vec<Finding> {
    let text = normalize_nfc(raw_text);
    let tokens = tokenize(&text);
    let char_map = CharOffsetMap::new(&text);
    let mut findings = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        // Ưu tiên cụm dài (3 từ) trước để `dành chiến thắng` không bị bắt
        // thành `dành chiến` + phần còn lại.
        let mut matched: Option<(usize, &'static str)> = None;
        for n in [3usize, 2, 1] {
            if i + n > tokens.len() {
                continue;
            }
            let phrase = tokens[i..i + n]
                .iter()
                .map(|t| t.text.to_lowercase())
                .collect::<Vec<_>>()
                .join(" ");
            if let Some(correct) = lookup(&phrase) {
                matched = Some((n, correct));
                break;
            }
        }

        let Some((n, correct)) = matched else {
            i += 1;
            continue;
        };

        let start_byte = tokens[i].start_byte;
        let end_byte = tokens[i + n - 1].end_byte;
        let original = &text[start_byte..end_byte];
        let replacement = preserve_phrase_case(original, correct);
        let (start, end) = char_map.byte_range_to_char_range(start_byte, end_byte);

        if start < end {
            findings.push(
                Finding::new(
                    COMMON_MISSPELLING_RULE_ID,
                    "orthography",
                    "major",
                    start,
                    end,
                    0.97,
                    format!(
                        "Lỗi chính tả phổ biến: '{}' nên viết là '{}'.",
                        original, replacement
                    ),
                    "misspelling",
                    original,
                )
                .with_fix(start, end, &replacement)
                .with_stage("binary-search"),
            );
        }
        i += n;
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replacements(text: &str) -> Vec<(String, String)> {
        evaluate_common_misspellings(text)
            .into_iter()
            .map(|f| (f.matched_span, f.suggestion.unwrap_or_default()))
            .collect()
    }

    #[test]
    fn test_basic_pairs() {
        assert_eq!(
            replacements("Chúng ta cần sử lý việc này."),
            vec![("sử lý".to_string(), "xử lý".to_string())]
        );
        assert_eq!(
            replacements("Hãy bổ xung thêm dữ liệu."),
            vec![("bổ xung".to_string(), "bổ sung".to_string())]
        );
        assert_eq!(
            replacements("Báo cáo cần chỉnh chu hơn."),
            vec![("chỉnh chu".to_string(), "chỉn chu".to_string())]
        );
        assert_eq!(
            replacements("Bác sĩ chuẩn đoán bệnh."),
            vec![("chuẩn đoán".to_string(), "chẩn đoán".to_string())]
        );
        assert_eq!(
            replacements("Cuối tuần đi thăm quan."),
            vec![("thăm quan".to_string(), "tham quan".to_string())]
        );
    }

    #[test]
    fn test_case_preservation() {
        assert_eq!(
            replacements("Sử lý hồ sơ."),
            vec![("Sử lý".to_string(), "Xử lý".to_string())]
        );
        assert_eq!(
            replacements("BỔ XUNG tài liệu."),
            vec![("BỔ XUNG".to_string(), "BỔ SUNG".to_string())]
        );
        assert_eq!(
            replacements("Chuẩn đoán bệnh."),
            vec![("Chuẩn đoán".to_string(), "Chẩn đoán".to_string())]
        );
    }

    #[test]
    fn test_no_substring_false_positive() {
        // "sử lý" không được khớp bên trong "xử lý" hay "sử dụng".
        assert!(evaluate_common_misspellings("Anh ấy xử lý công việc.").is_empty());
        assert!(evaluate_common_misspellings("Chúng ta sử dụng công cụ.").is_empty());
        // Từ đúng chứa chuỗi gần giống cũng không bị bắt.
        assert!(evaluate_common_misspellings("Bổ sung thêm dữ liệu.").is_empty());
        assert!(evaluate_common_misspellings("Chỉn chu trong từng chi tiết.").is_empty());
    }

    #[test]
    fn test_does_not_match_inside_longer_word() {
        // "sỉa" chỉ là token độc lập, không khớp trong "sỉa" của từ khác.
        assert!(evaluate_common_misspellings("Tôi xỉa răng.").is_empty());
    }

    #[test]
    fn test_three_word_phrase_wins() {
        // Ưu tiên 3-gram: phải khớp trọn cụm, không cắt thành 2-gram.
        assert_eq!(
            replacements("Đây là vô hình chung một cách hiểu sai."),
            vec![("vô hình chung".to_string(), "vô hình trung".to_string())]
        );
    }

    #[test]
    fn test_no_false_positive_on_valid_prefix_phrases() {
        // Các cụm từng bị nhầm là lỗi nhưng thực chất là phần đầu hợp lệ.
        assert!(evaluate_common_misspellings(
            "Bác trai và bác gái mới từ quê ra thăm gia đình tôi."
        )
        .is_empty());
        assert!(
            evaluate_common_misspellings("Chúng tôi chuyền tay nhau xem bức ảnh cũ.").is_empty()
        );
        assert!(evaluate_common_misspellings("Tôi đang nghĩ việc này nên làm thế nào.").is_empty());
        assert!(evaluate_common_misspellings("Cuối tuần cả nhà đi thăm khảo cổ.").is_empty());
        // Các cụm rất dễ bắt oan trong văn bản thật (đã bắt được khi quét corpus).
        assert!(evaluate_common_misspellings("Nhà máy có công suất phát điện rất lớn.").is_empty());
        assert!(evaluate_common_misspellings("Dự báo có khả năng xuất hiện mưa lớn.").is_empty());
        assert!(evaluate_common_misspellings("Tôi cũng cố gắng hoàn thành đúng hạn.").is_empty());
        assert!(
            evaluate_common_misspellings("Trang trại thu gom trứng từ các ao nuôi.").is_empty()
        );
    }

    #[test]
    fn test_offsets_are_char_based() {
        let text = "😀 Sử lý hồ sơ.";
        let f = evaluate_common_misspellings(text);
        assert_eq!(f.len(), 1);
        let chars: Vec<char> = text.chars().collect();
        let span: String = chars[f[0].start..f[0].end].iter().collect();
        assert_eq!(span, "Sử lý");
    }
}
