//! # BẮT LỖI "REAL-WORD" (TỪ ĐÚNG CHÍNH TẢ NHƯNG SAI NGỮ CẢNH)
//!
//! Các từ như `năng`, `nghĩ`, `ngũ`, `sinh`, `trăm`, `chao`, `rành`, `no`,
//! `hoàng`, `cự`, `đặc`, `chì`, `chữ`, `sét` đều là từ có thật trong tiếng Việt.
//! Từ điển chính tả đơn thuần không thể bắt được. Module này dùng
//! Aho-Corasick để khớp **cụm ngữ cảnh** (collocation) đã biết là sai, kèm
//! kiểm tra ranh giới từ để không bắt oan chuỗi con của từ đúng.
//!
//! Ví dụ: `tích cự` chỉ bị bắt khi đứng độc lập, không bắt bên trong `tích cực`.

use crate::rules::trap_whitelist::mask_trap_phrases;
use crate::types::Finding;
use crate::unicode_utils::{is_word_boundary, normalize_nfc, CharOffsetMap};
use daachorse::DoubleArrayAhoCorasick;
use std::sync::LazyLock;

pub const REAL_WORD_RULE_ID: &str = "REAL_WORD_CONTEXT";

/// Danh sách cặp (cụm sai, cụm đúng).
///
/// Quy ước: cụm sai phải là cụm tối thiểu đủ để phân biệt ngữ cảnh, tránh
/// rộng quá (bắt oan) và tránh hẹp quá (bỏ sót khi câu thay đổi).
static REAL_WORD_COLLOCATIONS: &[(&str, &str)] = &[
    // ── Bộ 1: lỗi dấu thanh tạo thành từ có thật ─────────────────────────
    ("trời năng", "trời nắng"),
    ("nghĩ ngơi", "nghỉ ngơi"),
    ("đang ngũ", "đang ngủ"),
    ("sinh đẹp", "xinh đẹp"),
    ("hoàng thành", "hoàn thành"),
    ("tích cự", "tích cực"),
    ("đặc vé", "đặt vé"),
    // ── Bộ 1: lỗi chính tả âm tiết (từ sai hoàn toàn nhưng dễ lẫn) ──────
    ("giúp đở", "giúp đỡ"),
    ("cố găng", "cố gắng"),
    ("gữi", "gửi"),
    ("áo mởi", "áo mới"),
    ("xắp xếp", "sắp xếp"),
    ("ngoang ngoãn", "ngoan ngoãn"),
    // ── Bộ 1: lỗi phụ âm đầu ─────────────────────────────────────────────
    ("trăm sóc", "chăm sóc"),
    ("chao đổi", "trao đổi"),
    ("rành cả", "dành cả"),
    ("no lắng", "lo lắng"),
    // ── Bộ 1: lỗi ngữ pháp / trật tự từ ─────────────────────────────────
    ("ăn rồi cơm", "ăn cơm rồi"),
    ("mua đã", "đã mua"),
    ("quyển bút", "cây bút"),
    ("về về", "về"),
    ("không đã", "chưa"),
    ("là rất", "rất"),
    // ── Bộ 2: lỗi chính tả hành chính tạo thành từ có thật ──────────────
    ("xử dụng", "sử dụng"),
    ("chiển khai", "triển khai"),
    ("chủ chì", "chủ trì"),
    ("dải quyết", "giải quyết"),
    ("lưu chữ", "lưu trữ"),
    ("sét duyệt", "xét duyệt"),
    ("được có hiệu lực", "có hiệu lực"),
    ("đã sẽ", "đã"),
];

static REAL_WORD_TRIE: LazyLock<DoubleArrayAhoCorasick<usize>> = LazyLock::new(|| {
    let patterns: Vec<&str> = REAL_WORD_COLLOCATIONS
        .iter()
        .map(|(wrong, _)| *wrong)
        .collect();
    DoubleArrayAhoCorasick::new(patterns).expect("Không thể build Real-word Trie")
});

fn preserve_case(original: &str, replacement: &str) -> String {
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

/// Một số cụm chỉ sai trong ngữ cảnh hẹp; kiểm tra thêm từ đứng trước.
fn context_allows_match(wrong: &str, lower_text: &str, start_byte: usize) -> bool {
    let before = lower_text[..start_byte].trim_end();
    match wrong {
        // "để báo cáo" là cụm hợp lệ ("để [mục đích] báo cáo"). Chỉ coi là lỗi
        // thừa từ khi đứng sau một chủ thể số nhiều trong văn bản hành chính.
        "để báo cáo" => before.ends_with("đơn vị") || before.ends_with("các sở"),
        _ => true,
    }
}

pub fn evaluate_real_word_errors(raw_text: &str) -> Vec<Finding> {
    let normalized = normalize_nfc(raw_text);
    let masked = mask_trap_phrases(&normalized);
    let lower_text = masked.to_lowercase();
    let char_map = CharOffsetMap::new(&masked);
    let orig_chars: Vec<char> = normalized.chars().collect();
    let mut findings = Vec::new();

    for m in REAL_WORD_TRIE.find_iter(&lower_text) {
        if !is_word_boundary(&lower_text, m.start(), m.end()) {
            continue;
        }

        let rule_idx = m.value();
        let (wrong, correct) = REAL_WORD_COLLOCATIONS[rule_idx];

        // "Hoàng thành Thăng Long" là danh từ riêng hợp lệ.
        if wrong == "hoàng thành" {
            let after = lower_text[m.end()..].trim_start();
            if after.starts_with("thăng") {
                continue;
            }
        }

        if !context_allows_match(wrong, &lower_text, m.start()) {
            continue;
        }

        let (start, end) = char_map.byte_range_to_char_range(m.start(), m.end());
        if end > orig_chars.len() || start >= end {
            continue;
        }
        let original_span: String = orig_chars[start..end].iter().collect();
        let replacement = preserve_case(&original_span, correct);

        findings.push(
            Finding::new(
                REAL_WORD_RULE_ID,
                "real_word",
                "major",
                start,
                end,
                0.93,
                format!(
                    "Lỗi ngữ cảnh: '{}' là từ có thật nhưng sai ngữ cảnh; có thể bạn muốn viết '{}'.",
                    original_span, replacement
                ),
                "real_word",
                original_span,
            )
            .with_fix(start, end, replacement)
                .with_stage("aho-corasick"),
        );
    }

    // Cụm "để báo cáo" không nằm trong Trie vì phụ thuộc ngữ cảnh phía trước.
    // Quét riêng để giữ bảng dữ liệu tường minh ở trên.
    let mut search_from = 0;
    while let Some(rel) = lower_text[search_from..].find("để báo cáo") {
        let start_byte = search_from + rel;
        let end_byte = start_byte + "để báo cáo".len();
        let before = lower_text[..start_byte].trim_end();
        if (before.ends_with("đơn vị") || before.ends_with("các sở"))
            && is_word_boundary(&lower_text, start_byte, end_byte)
        {
            let (start, end) = char_map.byte_range_to_char_range(start_byte, end_byte);
            if end <= orig_chars.len() {
                let span: String = orig_chars[start..end].iter().collect();
                let replacement = preserve_case(&span, "báo cáo");
                findings.push(
                    Finding::new(
                        REAL_WORD_RULE_ID,
                        "real_word",
                        "minor",
                        start,
                        end,
                        0.85,
                        "Thừa từ 'để' trước động từ 'báo cáo' trong câu cầu khiến hành chính.",
                        "real_word",
                        span,
                    )
                    .with_fix(start, end, replacement)
                    .with_stage("aho-corasick"),
                );
            }
        }
        search_from = end_byte;
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(text: &str) -> Vec<String> {
        evaluate_real_word_errors(text)
            .into_iter()
            .map(|f| f.matched_span)
            .collect()
    }

    #[test]
    fn test_real_word_basic_pairs() {
        assert_eq!(ids("Hôm nay trời năng đẹp."), vec!["trời năng"]);
        assert_eq!(ids("Anh ấy cần nghĩ ngơi."), vec!["nghĩ ngơi"]);
        assert_eq!(ids("Em bé đang ngũ trong phòng."), vec!["đang ngũ"]);
        assert_eq!(ids("Cô ấy rất sinh đẹp."), vec!["sinh đẹp"]);
        assert_eq!(ids("Trẻ em cần được trăm sóc."), vec!["trăm sóc"]);
    }

    #[test]
    fn test_no_false_positive_on_correct_words() {
        // "tích cực" chứa chuỗi con "tích cự" nhưng KHÔNG được bắt.
        assert!(ids("Anh ấy làm việc rất tích cực.").is_empty());
        assert!(ids("Trẻ em cần được chăm sóc đầy đủ.").is_empty());
        assert!(ids("Dự án đã hoàn thành đúng hạn.").is_empty());
        assert!(ids("Hoàng thành Thăng Long là di sản.").is_empty());
    }

    #[test]
    fn test_case_preservation() {
        let f = evaluate_real_word_errors("Trăm sóc trẻ em.");
        assert_eq!(f[0].suggestion.as_deref(), Some("Chăm sóc"));
    }

    #[test]
    fn test_common_misspellings_without_real_word_collision() {
        for (text, expected) in [
            ("Cảm ơn bạn đã giúp đở tôi.", "giúp đỡ"),
            ("Chúng ta cần cố găng hơn nữa.", "cố gắng"),
            ("Tôi sẽ gữi tài liệu.", "gửi"),
            ("Chị ấy mua áo mởi.", "áo mới"),
            ("Hãy xắp xếp tài liệu.", "sắp xếp"),
            ("Em gái tôi rất ngoang ngoãn.", "ngoan ngoãn"),
        ] {
            let findings = evaluate_real_word_errors(text);
            assert!(
                findings
                    .iter()
                    .any(|f| f.suggestion.as_deref() == Some(expected)),
                "Không sửa được {:?} -> {:?}: {:?}",
                text,
                expected,
                findings
            );
        }
    }

    #[test]
    fn test_de_bao_cao_requires_admin_subject() {
        // Câu mục đích bình thường: không bắt.
        assert!(ids("Tôi để báo cáo lại sự việc.").is_empty());
        // Câu cầu khiến hành chính: bắt.
        assert_eq!(
            ids("Đề nghị các đơn vị để báo cáo trước ngày 30."),
            vec!["để báo cáo"]
        );
    }
}
