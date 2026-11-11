//! # SỬA LỖI ÂM TIẾT OOV BẰNG TỪ ĐIỂN + BIGRAM LM
//!
//! Khác với `real_word.rs` (tra cụm sai đã biết), module này xử lý các âm tiết
//! **không tồn tại trong từ điển** (OOV) do gõ sai dấu thanh hoặc phụ âm đầu.
//!
//! Quy trình:
//! 1. Chỉ xét token có dấu tiếng Việt (hoặc `đ`), không viết hoa, không phải
//!    từ mượn/teencode → tránh bắt oan tên riêng và tiếng Anh.
//! 2. Sinh ứng viên bằng các nhóm lỗi gõ phổ biến:
//!    - biến đổi dấu thanh (ngang/huyền/sắc/hỏi/ngã/nặng),
//!    - nhầm phụ âm đầu (ng/ngh, g/gh, c/k, ch/tr, s/x, l/n, d/gi, r/d...).
//! 3. Giữ ứng viên **có thật trong từ điển 10.760 âm tiết**.
//! 4. Chọn ứng viên bằng **bigram language model** với từ đứng trước/sau.
//!    Nếu không đủ ngữ cảnh và có nhiều ứng viên → bỏ qua (thà bỏ sót).
//!
//! Luôn `Advisory`: đây là gợi ý chính tả, không trừ điểm.

use crate::rules::phonotactics::{
    is_exempt_loanword, is_valid_vietnamese_word, place_vietnamese_tone_decree20,
};
use crate::segmentation::generated_lm::{BIGRAM_DATA, UNIGRAM_DATA};
use crate::types::Finding;
use crate::unicode_utils::normalize_nfc;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use unicode_normalization::UnicodeNormalization;

pub const SPELLCHECK_RULE_ID: &str = "SPELLCHECK_SYLLABLE";

static UNIGRAM: LazyLock<HashMap<&'static str, f32>> =
    LazyLock::new(|| UNIGRAM_DATA.iter().copied().collect());

static BIGRAM: LazyLock<HashMap<(&'static str, &'static str), f32>> =
    LazyLock::new(|| BIGRAM_DATA.iter().map(|(a, b, s)| ((*a, *b), *s)).collect());

/// Các cặp phụ âm đầu dễ nhầm khi gõ.
const INITIAL_SWAPS: &[(&str, &str)] = &[
    ("ng", "ngh"),
    ("g", "gh"),
    ("c", "k"),
    ("ch", "tr"),
    ("s", "x"),
    ("l", "n"),
    ("d", "gi"),
    ("r", "d"),
    ("v", "d"),
];

/// Nhóm nguyên âm gần nhau (chỉ thay 1 ký tự mỗi lần).
const VOWEL_GROUPS: &[&[char]] = &[&['a', 'ă', 'â'], &['o', 'ô', 'ơ'], &['e', 'ê'], &['i', 'y']];

/// Cặp phụ âm cuối dễ nhầm.
const FINAL_SWAPS: &[(&str, &str)] = &[("ng", "n"), ("nh", "n"), ("ch", "c"), ("c", "t")];

/// Bỏ các dấu thanh (grave, acute, tilde, hook above, dot below) nhưng giữ
/// dấu mũ/móc/trăng (ô, ơ, â, ă, ư...).
fn strip_tones(text: &str) -> String {
    text.nfd()
        .filter(|c| {
            !matches!(
                *c,
                '\u{0300}' | '\u{0301}' | '\u{0303}' | '\u{0309}' | '\u{0323}'
            )
        })
        .nfc()
        .collect()
}

fn has_vietnamese_mark(token: &str) -> bool {
    token.chars().any(|c| c == 'đ' || c == 'Đ' || !c.is_ascii())
}

fn swap_initial(token: &str, from: &str, to: &str) -> Option<String> {
    token
        .strip_prefix(from)
        .map(|rest| format!("{}{}", to, rest))
        .or_else(|| {
            token
                .strip_prefix(to)
                .map(|rest| format!("{}{}", from, rest))
        })
}

/// Sinh các biến thể cấu trúc âm tiết (phụ âm cuối, nguyên âm) trên nền không dấu.
fn base_variants(base: &str) -> Vec<String> {
    let mut out = vec![base.to_string()];
    let chars: Vec<char> = base.chars().collect();

    for (from, to) in FINAL_SWAPS {
        if let Some(stem) = base.strip_suffix(from) {
            out.push(format!("{}{}", stem, to));
        }
        if let Some(stem) = base.strip_suffix(to) {
            out.push(format!("{}{}", stem, from));
        }
    }

    for (idx, c) in chars.iter().enumerate() {
        for group in VOWEL_GROUPS {
            if group.contains(c) {
                for alt in group.iter() {
                    if alt != c {
                        let mut variant = chars.clone();
                        variant[idx] = *alt;
                        out.push(variant.iter().collect());
                    }
                }
            }
        }
    }

    out.sort_unstable();
    out.dedup();
    out
}

/// Sinh toàn bộ ứng viên hợp lệ, không trùng lặp.
/// `pub(crate)` để context spellchecker tái sử dụng cùng confusion set.
pub(crate) fn valid_candidates(token: &str) -> Vec<String> {
    let mut seeds = vec![token.to_string()];
    for (a, b) in INITIAL_SWAPS {
        if let Some(swapped) = swap_initial(token, a, b) {
            seeds.push(swapped);
        }
    }
    let mut candidates: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for seed in &seeds {
        // Chính biến thể phụ âm đầu (giữ nguyên dấu thanh) cũng là ứng viên,
        // ví dụ "kặp" -> "cặp", "gế" -> "ghế", "ngĩ" -> "nghĩ".
        if seed != token && is_valid_vietnamese_word(seed) && seen.insert(seed.clone()) {
            candidates.push(seed.clone());
        }
        // Biến đổi cấu trúc âm tiết rồi mới đặt lại thanh điệu.
        for base in base_variants(&strip_tones(seed)) {
            for tone in 0..=5 {
                let candidate = place_vietnamese_tone_decree20(&base, tone);
                if candidate != token
                    && is_valid_vietnamese_word(&candidate)
                    && seen.insert(candidate.clone())
                {
                    candidates.push(candidate);
                }
            }
        }
    }
    candidates
}

fn context_score(candidate: &str, prev: Option<&str>, next: Option<&str>) -> (f32, bool) {
    let unigram = UNIGRAM.get(candidate).copied().unwrap_or(-15.0);
    let mut score = unigram * 0.1;
    let mut has_bigram = false;

    if let Some(prev) = prev {
        if let Some(v) = BIGRAM.get(&(prev, candidate)) {
            score += *v;
            has_bigram = true;
        }
    }
    if let Some(next) = next {
        if let Some(v) = BIGRAM.get(&(candidate, next)) {
            score += *v;
            has_bigram = true;
        }
    }
    // Ứng viên không khớp bất kỳ bigram nào bị phạt nặng: thà chọn ứng viên
    // có ngữ cảnh thật, không chọn theo unigram đơn thuần.
    if !has_bigram {
        score -= 12.0;
    }
    (score, has_bigram)
}

/// Chi phí sửa (prior): 1 = chỉ khác dấu thanh, 2 = khác phụ âm đầu,
/// 3 = khác nguyên âm/phụ âm cuối. Dùng khi LM không có bigram hỗ trợ.
fn operation_cost(token: &str, candidate: &str) -> u8 {
    let base_token = strip_tones(token);
    let base_candidate = strip_tones(candidate);
    if base_token == base_candidate {
        return 1;
    }
    for (a, b) in INITIAL_SWAPS {
        if swap_initial(&base_token, a, b).as_deref() == Some(base_candidate.as_str()) {
            return 2;
        }
    }
    3
}

/// Chọn ứng viên tốt nhất. Trả về None nếu không đủ chắc chắn.
fn choose_candidate(token: &str, prev: Option<&str>, next: Option<&str>) -> Option<String> {
    let candidates = valid_candidates(token);
    if candidates.is_empty() {
        return None;
    }
    if candidates.len() == 1 {
        return Some(candidates[0].clone());
    }

    let mut scored: Vec<(String, f32, bool, u8)> = candidates
        .into_iter()
        .map(|c| {
            let (mut score, has_bigram) = context_score(&c, prev, next);
            let cost = operation_cost(token, &c);
            // Ưu tiên nhẹ các sửa đổi rẻ hơn khi điểm ngữ cảnh sát nhau.
            score -= cost as f32 * 0.2;
            (c, score, has_bigram, cost)
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    if scored.iter().any(|(_, _, has_bigram, _)| *has_bigram) {
        let (best, best_score, _, _) = &scored[0];
        let second_score = scored
            .get(1)
            .map(|(_, s, _, _)| *s)
            .unwrap_or(f32::NEG_INFINITY);
        if best_score - second_score >= 1.5 {
            return Some(best.clone());
        }
        return None;
    }

    // Không có bigram: chọn theo chi phí sửa thấp nhất, rồi unigram.
    let present: Vec<&(String, f32, bool, u8)> = scored
        .iter()
        .filter(|(c, _, _, _)| UNIGRAM.contains_key(c.as_str()))
        .collect();
    if present.is_empty() {
        return None;
    }
    let min_cost = present.iter().map(|(_, _, _, cost)| *cost).min().unwrap();
    let cheapest: Vec<&(String, f32, bool, u8)> = present
        .into_iter()
        .filter(|(_, _, _, cost)| *cost == min_cost)
        .collect();
    if cheapest.len() == 1 {
        return Some(cheapest[0].0.clone());
    }
    // Nhiều ứng viên cùng chi phí: chỉ chọn khi unigram chênh lệch rõ.
    let mut by_unigram: Vec<(&String, f32)> = cheapest
        .iter()
        .map(|(c, _, _, _)| (c, *UNIGRAM.get(c.as_str()).unwrap_or(&-15.0)))
        .collect();
    by_unigram.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    if by_unigram.len() >= 2 && by_unigram[0].1 - by_unigram[1].1 >= 1.0 {
        Some(by_unigram[0].0.clone())
    } else {
        None
    }
}

/// Tách token từ văn bản (giữ nguyên char offset) và trả về danh sách
/// (token, start, end, prev, next) đã lowercase.
fn surrounding_words(chars: &[char], start: usize, end: usize) -> (Option<String>, Option<String>) {
    let read_left = |mut i: isize| -> Option<String> {
        while i >= 0 && !chars[i as usize].is_alphabetic() {
            i -= 1;
        }
        if i < 0 {
            return None;
        }
        let end = i as usize + 1;
        while i >= 0 && chars[i as usize].is_alphabetic() {
            i -= 1;
        }
        let start = (i + 1) as usize;
        Some(chars[start..end].iter().collect::<String>().to_lowercase())
    };
    let read_right = |mut i: usize| -> Option<String> {
        while i < chars.len() && !chars[i].is_alphabetic() {
            i += 1;
        }
        if i >= chars.len() {
            return None;
        }
        let start = i;
        while i < chars.len() && chars[i].is_alphabetic() {
            i += 1;
        }
        Some(chars[start..i].iter().collect::<String>().to_lowercase())
    };
    (read_left(start as isize - 1), read_right(end))
}

pub fn evaluate_spellcheck(raw_text: &str) -> Vec<Finding> {
    let normalized = normalize_nfc(raw_text);
    let chars: Vec<char> = normalized.chars().collect();
    let mut findings = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        if !chars[i].is_alphabetic() {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && chars[i].is_alphabetic() {
            i += 1;
        }
        let end = i;
        let token: String = chars[start..end].iter().collect();
        let lower = token.to_lowercase();

        // Chỉ xét âm tiết đơn có dấu tiếng Việt, không viết hoa, không phải từ mượn.
        if token.chars().count() < 2
            || !has_vietnamese_mark(&token)
            || token.chars().next().is_some_and(|c| c.is_uppercase())
            || token.chars().all(|c| c.is_uppercase())
            || is_exempt_loanword(&lower)
        {
            continue;
        }

        // Chỉ sửa OOV. Real-word (từ hợp lệ nhưng sai ngữ cảnh) phải xử lý bằng
        // bigram/context có kiểm soát (xem confusion.rs), vì LM hiện tại quá
        // thưa để phân biệt "giáo" / "giao" mà không tạo false positive.
        if is_valid_vietnamese_word(&lower) {
            continue;
        }

        let (prev, next) = surrounding_words(&chars, start, end);
        let Some(replacement) = choose_candidate(&lower, prev.as_deref(), next.as_deref()) else {
            continue;
        };

        // start/end đã là char offset của normalized.
        let (cstart, cend) = (start, end);
        let rationale = format!(
            "'{}' không có trong từ điển âm tiết; có thể bạn muốn viết '{}'.",
            token, replacement
        );

        findings.push(
            Finding::new(
                SPELLCHECK_RULE_ID,
                "orthography",
                "advisory",
                cstart,
                cend,
                0.85,
                rationale,
                "spellcheck",
                token,
            )
            .with_fix(cstart, cend, replacement)
            .with_stage("spellcheck"),
        );
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(text: &str) -> Vec<(String, String)> {
        evaluate_spellcheck(text)
            .into_iter()
            .map(|f| {
                (
                    f.matched_span,
                    f.fix.map(|x| x.replacement).unwrap_or_default(),
                )
            })
            .collect()
    }

    #[test]
    fn test_tone_and_initial_confusions() {
        assert!(fixed("Bạn hãy ngồi xuông ghế chờ nhé.")
            .contains(&("xuông".to_string(), "xuống".to_string())));
        // "vửng", "chình" là âm tiết có thật trong từ điển → do
        // confusion.rs (ngữ cảnh) xử lý, không phải spellcheck OOV.
        assert!(fixed("Tôi thích món phỡ bò.").contains(&("phỡ".to_string(), "phở".to_string())));
        assert!(fixed("Bà nội tôi ngĩ rằng trời mưa.")
            .contains(&("ngĩ".to_string(), "nghĩ".to_string())));
        // "kặp"/"gế"/"nười" là các ca ngữ cảnh (confusion.rs), không phải OOV đơn thuần.
    }

    #[test]
    fn test_does_not_flag_valid_words_or_names() {
        for text in [
            "Anh ấy đứng thứ ba trong cuộc thi.",
            "Cô giáo cho cả lớp nghỉ sớm.",
            "Google Docs và Microsoft Teams.",
            "Nguyễn Văn An sinh năm 1990.",
        ] {
            assert!(
                evaluate_spellcheck(text).is_empty(),
                "Bắt oan: {:?} -> {:?}",
                text,
                fixed(text)
            );
        }
    }
}
