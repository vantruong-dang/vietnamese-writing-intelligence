//! # SỬA LỖI NGỮ CẢNH (CONTEXT CONFUSION) & LẶP TỪ
//!
//! Bổ sung cho `real_word.rs` (cặp từ đơn) và `spellcheck.rs` (âm tiết OOV):
//! - cụm từ sai ngữ cảnh đã được chuẩn hóa (cơn bảo → cơn bão, bên canh → bên cạnh...),
//! - lặp từ chức năng (cần cần, đã đã, và và...),
//! - đảo trật tự phó từ (hay rất → rất hay),
//! - "được" + động từ tiêu cực (được phạt → bị phạt).
//!
//! Tất cả `Advisory` để không trừ điểm; nhưng vẫn có AutoFix để người dùng
//! áp dụng một chạm.

use crate::types::Finding;
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};
use daachorse::{DoubleArrayAhoCorasick, DoubleArrayAhoCorasickBuilder, MatchKind};
use regex::Regex;
use std::sync::LazyLock;

pub const CONTEXT_CONFUSION_RULE_ID: &str = "CONTEXT_CONFUSION";
pub const DUPLICATE_WORD_RULE_ID: &str = "DUPLICATE_FUNCTION_WORD";
pub const ADVERB_ORDER_RULE_ID: &str = "ADVERB_ORDER";
pub const VOICE_MISMATCH_RULE_ID: &str = "DUOC_BI_MISMATCH";

/// (cụm sai, cụm đúng) — các lỗi real-word thường gặp, cần ngữ cảnh 2-3 từ.
static CONTEXT_PAIRS: &[(&str, &str)] = &[
    ("cơn bảo", "cơn bão"),
    ("buổi sang", "buổi sáng"),
    ("bên canh", "bên cạnh"),
    ("chiếc vay", "chiếc váy"),
    ("đi tám biển", "đi tắm biển"),
    ("sao bữa ăn", "sau bữa ăn"),
    ("đẻ nấu", "để nấu"),
    ("vửng chắc", "vững chắc"),
    ("chình bày", "trình bày"),
    ("chiếc kặp", "chiếc cặp"),
    ("chiếc gế", "chiếc ghế"),
    ("rất nười", "rất lười"),
    ("rất nạnh", "rất lạnh"),
    ("dấy tờ", "giấy tờ"),
    ("cuống sách", "cuốn sách"),
];

static CONTEXT_TRIE: LazyLock<DoubleArrayAhoCorasick<usize>> = LazyLock::new(|| {
    let patterns: Vec<&str> = CONTEXT_PAIRS.iter().map(|(wrong, _)| *wrong).collect();
    DoubleArrayAhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostLongest)
        .build(&patterns)
        .expect("Không thể build Context Confusion Trie")
});

/// Từ chức năng không bao giờ được lặp liền nhau trong văn viết chuẩn.
static NON_REDUPLICATED: &[&str] = &["cần", "đã", "sẽ", "và", "của", "cho", "là", "với"];

static VOICE_MISMATCH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\bđược\s+(phạt|khiển trách|cảnh cáo|mắng|đuổi|trừng phạt|kỷ luật|xử lý nghiêm)\b",
    )
    .expect("Regex DUOC_BI_MISMATCH không hợp lệ")
});

static ADVERB_ORDER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bhay\s+rất\b").expect("Regex ADVERB_ORDER không hợp lệ"));

fn char_range(text: &str, start: usize, end: usize) -> (usize, usize) {
    let map = CharOffsetMap::new(text);
    map.byte_range_to_char_range(start, end)
}

fn push_pair_finding(
    findings: &mut Vec<Finding>,
    text: &str,
    start: usize,
    end: usize,
    replacement: &str,
    rule_id: &str,
) {
    let (cs, ce) = char_range(text, start, end);
    findings.push(
        Finding::new(
            rule_id,
            "syntax",
            "advisory",
            cs,
            ce,
            0.9,
            format!(
                "Cụm '{}' sai ngữ cảnh; nên viết '{}'.",
                &text[start..end],
                replacement
            ),
            "confusion",
            &text[start..end],
        )
        .with_fix(cs, ce, replacement)
        .with_stage("aho-corasick"),
    );
}

pub fn evaluate_context_confusions(raw_text: &str) -> Vec<Finding> {
    let text = normalize_nfc(raw_text);
    let lower = text.to_lowercase();
    let mut findings = Vec::new();

    for m in CONTEXT_TRIE.leftmost_find_iter(&lower) {
        let (wrong, correct) = CONTEXT_PAIRS[m.value()];
        // Giữ chữ hoa đầu cụm nếu bản gốc viết hoa.
        let original = &text[m.start()..m.end()];
        let replacement = if original.chars().next().is_some_and(|c| c.is_uppercase()) {
            let mut chars = correct.chars();
            chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str()
        } else {
            correct.to_string()
        };
        push_pair_finding(
            &mut findings,
            &text,
            m.start(),
            m.end(),
            &replacement,
            CONTEXT_CONFUSION_RULE_ID,
        );
        let _ = wrong;
    }

    findings
}

pub fn evaluate_duplicate_function_words(raw_text: &str) -> Vec<Finding> {
    let text = normalize_nfc(raw_text);
    let mut findings = Vec::new();
    let chars: Vec<char> = text.chars().collect();

    for word in NON_REDUPLICATED {
        let mut search = 0;
        while let Some(rel) = text[search..].to_lowercase().find(word) {
            // Vị trí byte trong chuỗi lowercase tương ứng 1:1 với bản gốc.
            let start = search + rel;
            let after_first = start + word.len();
            let Some(rest) = text.get(after_first..) else {
                break;
            };
            let lower_rest = rest.to_lowercase();
            let trimmed = lower_rest.trim_start();
            if trimmed.starts_with(word) {
                let spaces = lower_rest.len() - trimmed.len();
                let second_start = after_first + spaces;
                let second_end = second_start + word.len();
                let (cs, ce) = char_range(&text, start, second_end);
                let matched: String = chars[cs..ce].iter().collect();
                findings.push(
                    Finding::new(
                        DUPLICATE_WORD_RULE_ID,
                        "syntax",
                        "advisory",
                        cs,
                        ce,
                        0.92,
                        format!("Lặp từ chức năng '{}' không cần thiết.", word),
                        "confusion",
                        matched,
                    )
                    .with_fix(cs, ce, (*word).to_string())
                    .with_stage("regex"),
                );
                search = second_end;
            } else {
                search = after_first;
            }
        }
    }

    findings
}

pub fn evaluate_adverb_order(raw_text: &str) -> Vec<Finding> {
    let text = normalize_nfc(raw_text);
    let mut findings = Vec::new();
    for m in ADVERB_ORDER_RE.find_iter(&text) {
        push_pair_finding(
            &mut findings,
            &text,
            m.start(),
            m.end(),
            "rất hay",
            ADVERB_ORDER_RULE_ID,
        );
    }
    findings
}

pub fn evaluate_voice_mismatch(raw_text: &str) -> Vec<Finding> {
    let text = normalize_nfc(raw_text);
    let mut findings = Vec::new();
    for cap in VOICE_MISMATCH_RE.captures_iter(&text) {
        let full = cap.get(0).unwrap();
        let duoc = &text[full.start()..full.start() + "được".len()];
        if duoc.chars().next().is_some_and(|c| c.is_lowercase()) {
            // Giữ nguyên động từ phía sau, chỉ đổi "được" -> "bị".
            let (cs, ce) = char_range(&text, full.start(), full.start() + "được".len());
            findings.push(
                Finding::new(
                    VOICE_MISMATCH_RULE_ID,
                    "syntax",
                    "advisory",
                    cs,
                    ce,
                    0.85,
                    "'được' + động từ tiêu cực (bị phạt) là sai nghĩa; dùng 'bị'.",
                    "confusion",
                    duoc,
                )
                .with_fix(cs, ce, "bị")
                .with_stage("regex"),
            );
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spans(findings: &[Finding]) -> Vec<String> {
        findings.iter().map(|f| f.matched_span.clone()).collect()
    }

    #[test]
    fn test_context_pairs() {
        assert_eq!(
            spans(&evaluate_context_confusions("Cơn bảo đang tiến vào.")),
            vec!["Cơn bảo"]
        );
        assert_eq!(
            spans(&evaluate_context_confusions("Nhà bên canh.")),
            vec!["bên canh"]
        );
        assert_eq!(
            spans(&evaluate_context_confusions("Tôi thích chình bày.")),
            vec!["chình bày"]
        );
    }

    #[test]
    fn test_duplicate_words() {
        assert_eq!(
            spans(&evaluate_duplicate_function_words(
                "Chúng ta cần cần hoàn thành."
            )),
            vec!["cần cần"]
        );
        assert_eq!(
            spans(&evaluate_duplicate_function_words("Tôi đã đã gửi thư.")),
            vec!["đã đã"]
        );
        // Không bắt từ láy hợp lệ.
        assert!(evaluate_duplicate_function_words("Người người nhà nhà vui mừng.").is_empty());
    }

    #[test]
    fn test_adverb_order_and_voice() {
        assert_eq!(
            spans(&evaluate_adverb_order("Anh ấy hát hay rất.")),
            vec!["hay rất"]
        );
        assert_eq!(
            spans(&evaluate_voice_mismatch("Anh ấy được phạt.")),
            vec!["được"]
        );
        assert!(evaluate_voice_mismatch("Anh ấy được khen thưởng.").is_empty());
    }
}
