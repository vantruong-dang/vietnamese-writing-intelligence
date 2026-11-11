//! # TEENCODE / VIẾT TẮT MẠNG XÃ HỘI (ADVISORY)
//!
//! Người dùng thường dán comment Facebook/Voz vào. Các từ như `ko`, `dc`,
//! `mk`, `vs`, `j` không phải lỗi chính tả mà là cách viết thân mật.
//!
//! Nguyên tắc an toàn:
//! - **Luôn `Advisory`** → 0 điểm phạt, không bao giờ làm tụt điểm văn bản.
//! - Chỉ khớp **token độc lập** (word boundary), không khớp chuỗi con.
//! - Bỏ qua token viết hoa toàn bộ (K, R, VS, BT...) vì rất có thể là
//!   tên riêng/chữ viết tắt chuyên ngành, không phải teencode.

use crate::types::Finding;
use crate::unicode_utils::{is_word_boundary, normalize_nfc, CharOffsetMap};
use daachorse::{DoubleArrayAhoCorasick, DoubleArrayAhoCorasickBuilder, MatchKind};
use std::sync::LazyLock;

pub const TEENCODE_RULE_ID: &str = "TEENCODE_SHORTHAND";

/// (teencode, từ chuẩn)
static TEENCODE_MAP: &[(&str, &str)] = &[
    ("ko", "không"),
    ("hok", "không"),
    ("k", "không"),
    ("dc", "được"),
    ("đc", "được"),
    ("mk", "mình"),
    ("r", "rồi"),
    ("vs", "với"),
    ("j", "gì"),
    ("bt", "biết"),
    ("ns", "nói"),
    ("tl", "trả lời"),
];

static TEENCODE_TRIE: LazyLock<DoubleArrayAhoCorasick<usize>> = LazyLock::new(|| {
    let patterns: Vec<&str> = TEENCODE_MAP.iter().map(|(short, _)| *short).collect();
    // LeftmostLongest để "ko" thắng "k", tránh việc trie trả về "k" rồi
    // boundary check loại bỏ và làm mất luôn "ko".
    DoubleArrayAhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostLongest)
        .build(&patterns)
        .expect("Không thể build Teencode Trie")
});

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

pub fn evaluate_teencode(raw_text: &str) -> Vec<Finding> {
    let normalized = normalize_nfc(raw_text);
    let lower_text = normalized.to_lowercase();
    let char_map = CharOffsetMap::new(&normalized);
    let orig_chars: Vec<char> = normalized.chars().collect();
    let mut findings = Vec::new();

    for m in TEENCODE_TRIE.leftmost_find_iter(&lower_text) {
        if !is_word_boundary(&lower_text, m.start(), m.end()) {
            continue;
        }

        let (start, end) = char_map.byte_range_to_char_range(m.start(), m.end());
        if end > orig_chars.len() || start >= end {
            continue;
        }
        let original: String = orig_chars[start..end].iter().collect();

        // "K", "R", "VS", "BT" viết hoa toàn bộ: coi là tên riêng/chữ viết tắt.
        if original.chars().count() <= 2 && original.chars().all(|c| c.is_uppercase()) {
            continue;
        }

        let (short, standard) = TEENCODE_MAP[m.value()];
        // "A vs B" / "Hà Nội vs Sài Gòn" (đối đầu thể thao) không phải teencode:
        // cả hai từ đứng cạnh đều viết hoa.
        if short.eq_ignore_ascii_case("vs") {
            let prev_word: String = orig_chars[..start]
                .iter()
                .rev()
                .skip_while(|c| c.is_whitespace())
                .take_while(|c| c.is_alphabetic())
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            let next_word: String = orig_chars[end..]
                .iter()
                .skip_while(|c| c.is_whitespace())
                .take_while(|c| c.is_alphabetic())
                .collect();
            let starts_upper = |word: &str| word.chars().next().is_some_and(|c| c.is_uppercase());
            if starts_upper(&prev_word) && starts_upper(&next_word) {
                continue;
            }
        }
        let replacement = if original.chars().next().is_some_and(|c| c.is_uppercase()) {
            capitalize_first(standard)
        } else {
            standard.to_string()
        };

        findings.push(
            Finding::new(
                TEENCODE_RULE_ID,
                "teencode",
                "advisory",
                start,
                end,
                0.9,
                format!(
                    "'{}' là cách viết tắt thân mật; trong văn bản trang trọng nên dùng '{}'.",
                    original, replacement
                ),
                "register",
                original,
            )
            .with_fix(start, end, replacement)
            .with_stage("aho-corasick"),
        );
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spans(text: &str) -> Vec<String> {
        evaluate_teencode(text)
            .into_iter()
            .map(|f| f.matched_span)
            .collect()
    }

    #[test]
    fn test_detects_common_teencode() {
        assert_eq!(spans("Mk ko bt j ca"), vec!["Mk", "ko", "bt", "j"]);
        assert_eq!(spans("dc roi"), vec!["dc"]);
    }

    #[test]
    fn test_always_advisory() {
        for f in evaluate_teencode("mk ko dc") {
            assert_eq!(f.severity, "advisory");
        }
    }

    #[test]
    fn test_does_not_flag_inside_words_or_uppercase_acronyms() {
        assert!(spans("komkom").is_empty());
        assert!(spans("VS BT K R").is_empty());
        // Báo thể thao "A vs B" không phải teencode.
        assert!(spans("Hà Nội vs Sài Gòn").is_empty());
        // Từ tiếng Việt bình thường không bị bắt.
        assert!(spans("không được với mình rồi gì biết").is_empty());
    }
}
