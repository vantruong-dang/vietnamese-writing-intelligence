//! # NEURAL CASCADE (Tầng 3 ↔ Tầng 4)
//!
//! Hợp đồng giữa `vwi-core` và runtime neural:
//! - Tầng 1/2 khóa byte bằng `locked_mask` (hard guard + symbolic rules).
//! - Tầng 3 trả `NeuralSpanHint` (byte span + confidence + error kind).
//! - Tầng 4 snap về ranh giới âm tiết, sinh ứng viên có ràng buộc, kiểm tra
//!   `Viet74K` + bigram ngữ cảnh, và chỉ xuất diagnostic advisory nếu có ứng viên.
//!
//! Module này **không** phụ thuộc `vsm-runtime`; runtime chỉ cần implement trait
//! `NeuralContextScorer` và được nối ở `vwi-wasm`.

use crate::rules::lookup_common_typo;
use crate::segmentation::EMBEDDED_DICTIONARY;
use crate::types::Finding;
use crate::unicode_utils::normalize_nfc;

pub const NEURAL_CONTEXT_RULE_ID: &str = "NEURAL_CONTEXT_SPELLING";

#[derive(Debug, Clone, Copy)]
pub struct NeuralSpanHint {
    pub start: usize,
    pub end: usize,
    pub error_kind: u8,
    pub confidence: f32,
    pub byte_logits_idx: usize,
}

pub trait NeuralContextScorer {
    fn score_unmasked_regions(
        &self,
        text: &str,
        locked_mask: &[bool],
        out_hints: &mut Vec<NeuralSpanHint>,
    );

    fn decode_top_candidates(
        &self,
        span_start: usize,
        span_end: usize,
        top_k: usize,
        out_buf: &mut Vec<String>,
    );
}

#[derive(Debug, Clone, Copy)]
pub struct CascadeConfig {
    pub trigger: f32,
    pub syllable_mean: f32,
    pub min_confidence: f32,
    pub valid_raw_confidence: f32,
    pub max_candidates: usize,
}

impl Default for CascadeConfig {
    fn default() -> Self {
        Self {
            trigger: 0.92,
            syllable_mean: 0.85,
            min_confidence: 0.85,
            valid_raw_confidence: 0.90,
            max_candidates: 12,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NeuralDiagnostic {
    pub start: usize,
    pub end: usize,
    pub original: String,
    pub suggestion: String,
    pub error_kind: u8,
    pub confidence: f32,
}

impl NeuralDiagnostic {
    pub fn to_finding(&self, text: &str) -> Finding {
        let start_char = text[..self.start].chars().count();
        let end_char = text[..self.end].chars().count();
        Finding::new(
            NEURAL_CONTEXT_RULE_ID,
            "orthography",
            "advisory",
            start_char,
            end_char,
            self.confidence as f64,
            format!(
                "Ngữ cảnh gợi ý '{}' nên viết là '{}'.",
                self.original, self.suggestion
            ),
            "neural_context",
            self.original.clone(),
        )
        .with_fix(start_char, end_char, &self.suggestion)
        .with_stage("neural-cascade")
    }
}

fn is_word_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

/// Snap một byte span về trọn token/âm tiết bao quanh.
pub fn snap_span_to_token(text: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    if start > text.len() || end > text.len() || start >= end {
        return None;
    }
    let mut left = start;
    while left > 0 && !text.is_char_boundary(left) {
        left -= 1;
    }
    let mut right = end;
    while right < text.len() && !text.is_char_boundary(right) {
        right += 1;
    }
    while left > 0 {
        let previous = text[..left].chars().next_back()?;
        if !is_word_char(previous) {
            break;
        }
        left -= previous.len_utf8();
    }
    while right < text.len() {
        let next = text[right..].chars().next()?;
        if !is_word_char(next) {
            break;
        }
        right += next.len_utf8();
    }
    if left >= right {
        return None;
    }
    Some((left, right))
}

fn previous_token(text: &str, start: usize) -> Option<&str> {
    let mut cursor = start;
    while cursor > 0 {
        let character = text[..cursor].chars().next_back()?;
        if is_word_char(character) {
            break;
        }
        cursor -= character.len_utf8();
    }
    if cursor == 0 {
        return None;
    }
    let end = cursor;
    while cursor > 0 {
        let character = text[..cursor].chars().next_back()?;
        if !is_word_char(character) {
            break;
        }
        cursor -= character.len_utf8();
    }
    if cursor == end {
        None
    } else {
        Some(&text[cursor..end])
    }
}

fn next_token(text: &str, end: usize) -> Option<&str> {
    let mut cursor = end;
    while cursor < text.len() {
        let character = text[cursor..].chars().next()?;
        if is_word_char(character) {
            break;
        }
        cursor += character.len_utf8();
    }
    if cursor >= text.len() {
        return None;
    }
    let start = cursor;
    while cursor < text.len() {
        let character = text[cursor..].chars().next()?;
        if !is_word_char(character) {
            break;
        }
        cursor += character.len_utf8();
    }
    Some(&text[start..cursor])
}

/// Tra cứu chính xác một mục từ trong Viet74K nhúng.
pub fn is_lexicon_word(word: &str) -> bool {
    let normalized = normalize_nfc(word).to_lowercase();
    if normalized.is_empty() {
        return false;
    }
    EMBEDDED_DICTIONARY
        .find_overlapping_iter(&normalized)
        .any(|matched| matched.start() == 0 && matched.end() == normalized.len())
}

fn spans_overlap(left: (usize, usize), right: (usize, usize)) -> bool {
    left.0 < right.1 && right.0 < left.1
}

fn has_valid_bigram(candidate: &str, previous: Option<&str>, next: Option<&str>) -> bool {
    if let Some(previous) = previous {
        if is_lexicon_word(&format!("{previous} {candidate}")) {
            return true;
        }
    }
    if let Some(next) = next {
        if is_lexicon_word(&format!("{candidate} {next}")) {
            return true;
        }
    }
    false
}

const INITIALS: &[&str] = &[
    "ngh", "ng", "nh", "ch", "tr", "gi", "ph", "th", "kh", "qu", "b", "c", "d", "đ", "g", "h", "k",
    "l", "m", "n", "p", "r", "s", "t", "v", "x",
];
const INITIAL_GROUPS: &[(&str, &str)] = &[
    ("s", "x"),
    ("tr", "ch"),
    ("l", "n"),
    ("r", "d"),
    ("gi", "d"),
    ("v", "d"),
];
const CODA_GROUPS: &[(&str, &str)] = &[
    ("ng", "n"),
    ("ch", "c"),
    ("c", "t"),
    ("n", "ng"),
    ("t", "c"),
];
const VOWEL_GROUPS: &[(&str, &str)] = &[("iêu", "ưu"), ("ươu", "ưu"), ("ưu", "iêu")];

const TONE_GROUPS: &[&str] = &[
    "aàáảãạ",
    "ăằắẳẵặ",
    "âầấẩẫậ",
    "eèéẻẽẹ",
    "êềếểễệ",
    "iìíỉĩị",
    "oòóỏõọ",
    "ôồốổỗộ",
    "ơờớởỡợ",
    "uùúủũụ",
    "ưừứửữự",
    "yỳýỷỹỵ",
];

fn split_initial(syllable: &str) -> (&str, &str) {
    for initial in INITIALS {
        if syllable.starts_with(initial) && syllable.len() > initial.len() {
            return (initial, &syllable[initial.len()..]);
        }
    }
    ("", syllable)
}

fn split_coda(syllable: &str) -> (&str, &str) {
    for (left, right) in CODA_GROUPS {
        for coda in [*left, *right] {
            if syllable.ends_with(coda) && syllable.len() > coda.len() {
                return (&syllable[..syllable.len() - coda.len()], coda);
            }
        }
    }
    (syllable, "")
}

fn tone_variants(syllable: &str) -> Vec<String> {
    let mut variants = Vec::new();
    for (index, character) in syllable.char_indices() {
        if let Some(group) = TONE_GROUPS.iter().find(|group| group.contains(character)) {
            let characters: Vec<char> = group.chars().collect();
            let position = characters.iter().position(|item| *item == character);
            let mut order: Vec<usize> = Vec::new();
            if let Some(position) = position {
                // Ưu tiên hỏi ↔ ngã (nhóm 3 ↔ 4) cho nhãn TONE_CONFUSION.
                let preferred = match position {
                    3 => Some(4),
                    4 => Some(3),
                    _ => None,
                };
                if let Some(preferred) = preferred {
                    order.push(preferred);
                }
                for candidate_index in 0..characters.len() {
                    if candidate_index != position && Some(candidate_index) != preferred {
                        order.push(candidate_index);
                    }
                }
            }
            for candidate_index in order {
                let replacement = characters[candidate_index];
                let mut candidate = String::with_capacity(syllable.len());
                candidate.push_str(&syllable[..index]);
                candidate.push(replacement);
                candidate.push_str(&syllable[index + character.len_utf8()..]);
                if is_lexicon_word(&candidate) && !variants.contains(&candidate) {
                    variants.push(candidate);
                }
            }
            break;
        }
    }
    variants
}

fn phonetic_variants(syllable: &str) -> Vec<String> {
    let mut variants = Vec::new();
    let (initial, rest) = split_initial(syllable);
    for (left, right) in INITIAL_GROUPS {
        let replacement_initial = if initial == *left {
            Some(*right)
        } else if initial == *right {
            Some(*left)
        } else {
            None
        };
        if let Some(replacement_initial) = replacement_initial {
            let candidate = format!("{replacement_initial}{rest}");
            if is_lexicon_word(&candidate) && !variants.contains(&candidate) {
                variants.push(candidate);
            }
        }
    }
    let (base, coda) = split_coda(syllable);
    for (left, right) in CODA_GROUPS {
        let replacement_coda = if coda == *left {
            Some(*right)
        } else if coda == *right {
            Some(*left)
        } else {
            None
        };
        if let Some(replacement_coda) = replacement_coda {
            let candidate = format!("{base}{replacement_coda}");
            if is_lexicon_word(&candidate) && !variants.contains(&candidate) {
                variants.push(candidate);
            }
        }
    }
    for (left, right) in VOWEL_GROUPS {
        if rest.contains(left) {
            let candidate = format!("{initial}{}", rest.replace(left, right));
            if is_lexicon_word(&candidate) && !variants.contains(&candidate) {
                variants.push(candidate);
            }
        }
    }
    variants
}

const EDIT_LETTERS: &str = "aăâbcdđeêghiklmnoôơpqrstuưvxy";

const SLANG_ABBREVIATIONS: &[&str] = &[
    "ko", "k", "đc", "dc", "fb", "sg", "ib", "rep", "stt", "cmt", "oke", "ok", "vãi", "vl",
];

fn edit_one_variants(syllable: &str) -> Vec<String> {
    let mut variants = Vec::new();
    let characters: Vec<char> = syllable.chars().collect();
    // Thay thế 1 ký tự.
    for (index, character) in syllable.char_indices() {
        for replacement in EDIT_LETTERS.chars() {
            if replacement == character {
                continue;
            }
            let mut candidate = String::with_capacity(syllable.len());
            candidate.push_str(&syllable[..index]);
            candidate.push(replacement);
            candidate.push_str(&syllable[index + character.len_utf8()..]);
            if is_lexicon_word(&candidate) && !variants.contains(&candidate) {
                variants.push(candidate);
            }
        }
    }
    // Xóa 1 ký tự (IME thừa phím).
    for index in 0..characters.len() {
        let candidate: String = characters
            .iter()
            .enumerate()
            .filter(|(position, _)| *position != index)
            .map(|(_, character)| *character)
            .collect();
        if is_lexicon_word(&candidate) && !variants.contains(&candidate) {
            variants.push(candidate);
        }
    }
    // Chèn 1 ký tự (IME thiếu phím).
    for index in 0..=characters.len() {
        for replacement in EDIT_LETTERS.chars() {
            let mut candidate: String = characters[..index].iter().collect();
            candidate.push(replacement);
            candidate.extend(&characters[index..]);
            if is_lexicon_word(&candidate) && !variants.contains(&candidate) {
                variants.push(candidate);
            }
        }
    }
    // Hoán vị 2 ký tự liền kề.
    for index in 0..characters.len().saturating_sub(1) {
        let mut candidate = characters.clone();
        candidate.swap(index, index + 1);
        let candidate: String = candidate.into_iter().collect();
        if is_lexicon_word(&candidate) && !variants.contains(&candidate) {
            variants.push(candidate);
        }
    }
    variants
}

fn common_typo_candidate(raw: &str, previous: Option<&str>, next: Option<&str>) -> Option<String> {
    if let Some(previous) = previous {
        let phrase = format!("{previous} {raw}");
        if let Some(corrected) = lookup_common_typo(&phrase) {
            let parts: Vec<&str> = corrected.split_whitespace().collect();
            if parts.len() == 2 {
                return Some(parts[1].to_string());
            }
        }
    }
    if let Some(next) = next {
        let phrase = format!("{raw} {next}");
        if let Some(corrected) = lookup_common_typo(&phrase) {
            let parts: Vec<&str> = corrected.split_whitespace().collect();
            if parts.len() == 2 {
                return Some(parts[0].to_string());
            }
        }
    }
    None
}

#[derive(Debug, Clone)]
struct Candidate {
    text: String,
    rank: u8,
}

fn candidates_for(raw: &str, error_kind: u8, limit: usize) -> Vec<Candidate> {
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut push = |text: String, rank: u8| {
        if text == raw || !is_lexicon_word(&text) {
            return;
        }
        if let Some(existing) = candidates
            .iter_mut()
            .find(|candidate| candidate.text == text)
        {
            existing.rank = existing.rank.max(rank);
        } else {
            candidates.push(Candidate { text, rank });
        }
    };
    for candidate in tone_variants(raw) {
        push(candidate, if error_kind == 2 { 3 } else { 2 });
    }
    for candidate in phonetic_variants(raw) {
        push(candidate, if error_kind == 1 { 3 } else { 2 });
    }
    for candidate in edit_one_variants(raw) {
        push(candidate, 1);
    }
    candidates.sort_by_key(|candidate| {
        (
            std::cmp::Reverse(candidate.rank),
            candidate.text.chars().count(),
        )
    });
    candidates.truncate(limit);
    candidates
}

fn edit_distance(left: &str, right: &str) -> usize {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    let mut current = vec![0usize; right.len() + 1];
    for (row, left_char) in left.iter().enumerate() {
        current[0] = row + 1;
        for (column, right_char) in right.iter().enumerate() {
            let cost = usize::from(left_char != right_char);
            current[column + 1] = (previous[column + 1] + 1)
                .min(current[column] + 1)
                .min(previous[column] + cost);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}

/// Tầng 4: lọc hint neural qua Viet74K + bigram ngữ cảnh.
pub fn run_tier4(
    text: &str,
    hints: &[NeuralSpanHint],
    locked_mask: &[bool],
    config: CascadeConfig,
) -> Vec<NeuralDiagnostic> {
    run_tier4_with_candidates(text, hints, locked_mask, config, &[])
}

/// Như `run_tier4` nhưng nhận thêm ứng viên từ neural byte head (Nguồn A).
pub fn run_tier4_with_candidates(
    text: &str,
    hints: &[NeuralSpanHint],
    locked_mask: &[bool],
    config: CascadeConfig,
    external_candidates: &[(usize, usize, String)],
) -> Vec<NeuralDiagnostic> {
    let mut diagnostics: Vec<NeuralDiagnostic> = Vec::new();

    for hint in hints {
        if hint.error_kind == 0 || hint.error_kind > 3 {
            continue;
        }
        if hint.confidence < config.min_confidence {
            continue;
        }
        let Some((start, end)) = snap_span_to_token(text, hint.start, hint.end) else {
            continue;
        };
        if locked_mask
            .get(start..end)
            .is_some_and(|locked| locked.iter().any(|value| *value))
        {
            continue;
        }
        let raw = &text[start..end];
        // Tầng 4 chỉ xử lý token chữ thuần; số/mã alphanumeric thuộc Tầng 1.
        if !raw.chars().all(char::is_alphabetic) {
            continue;
        }
        if SLANG_ABBREVIATIONS.contains(&raw.to_lowercase().as_str()) {
            continue;
        }
        let raw_valid = is_lexicon_word(raw);
        if raw_valid && hint.confidence < config.valid_raw_confidence {
            continue;
        }

        let previous = previous_token(text, start);
        let next = next_token(text, end);
        let common_candidate = common_typo_candidate(raw, previous, next);
        let mut best: Option<(bool, u8, usize, String)> = common_candidate
            .as_ref()
            .map(|candidate| (true, 4, edit_distance(raw, candidate), candidate.clone()));
        for (external_start, external_end, candidate) in external_candidates {
            if !spans_overlap((*external_start, *external_end), (start, end))
                || candidate == raw
                || !is_lexicon_word(candidate)
            {
                continue;
            }
            let bigram = has_valid_bigram(candidate, previous, next);
            let distance = edit_distance(raw, candidate);
            let better = match &best {
                None => true,
                Some((best_bigram, best_rank, best_distance, _)) => {
                    (bigram, 5u8, std::cmp::Reverse(distance))
                        > (*best_bigram, *best_rank, std::cmp::Reverse(*best_distance))
                }
            };
            if better {
                best = Some((bigram, 5, distance, candidate.clone()));
            }
        }
        for candidate in candidates_for(raw, hint.error_kind, config.max_candidates) {
            if common_candidate.as_deref() == Some(candidate.text.as_str()) {
                continue;
            }
            let bigram = has_valid_bigram(&candidate.text, previous, next);
            let distance = edit_distance(raw, &candidate.text);
            let better = match &best {
                None => true,
                Some((best_bigram, best_rank, best_distance, _)) => {
                    (bigram, candidate.rank, std::cmp::Reverse(distance))
                        > (*best_bigram, *best_rank, std::cmp::Reverse(*best_distance))
                }
            };
            if better {
                best = Some((bigram, candidate.rank, distance, candidate.text));
            }
        }

        if let Some((_, _, _, suggestion)) = best {
            diagnostics.push(NeuralDiagnostic {
                start,
                end,
                original: raw.to_string(),
                suggestion,
                error_kind: hint.error_kind,
                confidence: hint.confidence,
            });
        }
    }

    diagnostics.sort_by_key(|diagnostic| (diagnostic.start, diagnostic.end));
    diagnostics.dedup_by(|left, right| left.start == right.start && left.end == right.end);
    diagnostics
}

/// Sinh hint từ xác suất per-byte `[clean, phonetic, tone, word]`.
pub fn hints_from_byte_probs(
    text: &str,
    probabilities: &[[f32; 4]],
    locked_mask: &[bool],
    config: CascadeConfig,
) -> Vec<NeuralSpanHint> {
    let bytes = text.as_bytes();
    let mut triggered = vec![false; bytes.len()];
    for index in 0..bytes.len().min(probabilities.len()) {
        if locked_mask.get(index).copied().unwrap_or(false) {
            continue;
        }
        if 1.0 - probabilities[index][0] >= config.trigger {
            triggered[index] = true;
        }
    }

    let mut hints = Vec::new();
    let mut seen = Vec::new();
    for index in 0..triggered.len() {
        if !triggered[index] {
            continue;
        }
        let Some((start, end)) = snap_span_to_token(text, index, index + 1) else {
            continue;
        };
        if seen.contains(&(start, end)) {
            continue;
        }
        seen.push((start, end));

        let mut sum = 0.0f32;
        let mut count = 0usize;
        let mut maximum = 0.0f32;
        let mut label_sums = [0.0f32; 4];
        for (byte_index, _byte) in bytes.iter().enumerate().take(end).skip(start) {
            if byte_index >= probabilities.len() {
                continue;
            }
            let probability = probabilities[byte_index];
            let error = (1.0 - probability[0]).clamp(0.0, 1.0);
            sum += error;
            count += 1;
            maximum = maximum.max(error);
            for label in 1..4 {
                label_sums[label] += probability[label];
            }
        }
        if count == 0 {
            continue;
        }
        let mean = sum / count as f32;
        if mean < config.syllable_mean || maximum < config.trigger {
            continue;
        }
        let error_kind = (1..4)
            .max_by(|left, right| label_sums[*left].total_cmp(&label_sums[*right]))
            .unwrap_or(1) as u8;
        hints.push(NeuralSpanHint {
            start,
            end,
            error_kind,
            confidence: mean,
            byte_logits_idx: start,
        });
    }
    hints
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hint_for(text: &str, needle: &str, kind: u8, confidence: f32) -> NeuralSpanHint {
        let start = text.find(needle).expect("needle");
        NeuralSpanHint {
            start,
            end: start + needle.len(),
            error_kind: kind,
            confidence,
            byte_logits_idx: start,
        }
    }

    #[test]
    fn test_snap_span_to_token() {
        let text = "Đây là chính tã.";
        let start = text.find("tã").unwrap();
        assert_eq!(
            snap_span_to_token(text, start, start + 1),
            Some((start, start + "tã".len()))
        );
        let partial = text.find("chính").unwrap() + 2;
        assert_eq!(
            snap_span_to_token(text, partial, partial + 1),
            Some((
                text.find("chính").unwrap(),
                text.find("chính").unwrap() + "chính".len()
            ))
        );
    }

    #[test]
    fn test_tier4_fixes_tone_word() {
        assert!(is_lexicon_word("chính tả"), "Viet74K thiếu 'chính tả'");
        assert!(is_lexicon_word("tả"));
        let text = "Đây là lỗi chính tã.";
        let hint = hint_for(text, "tã", 2, 0.97);
        let diagnostics = run_tier4(
            text,
            &[hint],
            &vec![false; text.len()],
            CascadeConfig::default(),
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].suggestion, "tả");
        assert_eq!(diagnostics[0].original, "tã");
    }

    #[test]
    fn test_tier4_fixes_context_bigram() {
        let text = "Cuối tuần đi thăm quan.";
        let hint = hint_for(text, "thăm", 3, 0.97);
        let diagnostics = run_tier4(
            text,
            &[hint],
            &vec![false; text.len()],
            CascadeConfig::default(),
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].suggestion, "tham");
    }

    #[test]
    fn test_tier4_drops_unknown_and_locked() {
        let text = "Docker là công cụ.";
        let hint = hint_for(text, "Docker", 1, 0.99);
        let diagnostics = run_tier4(
            text,
            &[hint],
            &vec![false; text.len()],
            CascadeConfig::default(),
        );
        assert!(diagnostics.is_empty());

        let text = "B12 là mã tài sản.";
        let hint = hint_for(text, "B12", 1, 0.99);
        let mut locked = vec![false; text.len()];
        let start = text.find("B12").unwrap();
        for value in &mut locked[start..start + 3] {
            *value = true;
        }
        assert!(run_tier4(text, &[hint], &locked, CascadeConfig::default()).is_empty());
    }

    #[test]
    fn test_tier4_drops_numeric_and_codes() {
        let text = "Báo động 2 và mã B12.";
        let start = text.find('2').unwrap();
        let hint = NeuralSpanHint {
            start,
            end: start + 1,
            error_kind: 1,
            confidence: 0.99,
            byte_logits_idx: start,
        };
        assert!(run_tier4(
            text,
            &[hint],
            &vec![false; text.len()],
            CascadeConfig::default()
        )
        .is_empty());
        let start = text.find("B12").unwrap();
        let hint = NeuralSpanHint {
            start,
            end: start + 3,
            error_kind: 1,
            confidence: 0.99,
            byte_logits_idx: start,
        };
        assert!(run_tier4(
            text,
            &[hint],
            &vec![false; text.len()],
            CascadeConfig::default()
        )
        .is_empty());

        let text = "Tùng ơi, sg gửi deadline nhé.";
        let start = text.find("sg").unwrap();
        let hint = NeuralSpanHint {
            start,
            end: start + 2,
            error_kind: 1,
            confidence: 0.99,
            byte_logits_idx: start,
        };
        assert!(run_tier4(
            text,
            &[hint],
            &vec![false; text.len()],
            CascadeConfig::default()
        )
        .is_empty());
    }

    #[test]
    fn test_hints_from_byte_probs() {
        let text = "chính tã";
        let mut probabilities = vec![[0.99, 0.003, 0.004, 0.003]; text.len()];
        let start = text.find("tã").unwrap();
        for probability in &mut probabilities[start..start + "tã".len()] {
            *probability = [0.01, 0.01, 0.97, 0.01];
        }
        let hints = hints_from_byte_probs(
            text,
            &probabilities,
            &vec![false; text.len()],
            CascadeConfig::default(),
        );
        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0].error_kind, 2);
        assert!(hints[0].confidence >= 0.9);
        assert_eq!(&text[hints[0].start..hints[0].end], "tã");
    }

    #[test]
    fn test_diagnostic_to_finding_uses_char_offsets() {
        let text = "Đây là chính tã.";
        let hint = hint_for(text, "tã", 2, 0.97);
        let diagnostics = run_tier4(
            text,
            &[hint],
            &vec![false; text.len()],
            CascadeConfig::default(),
        );
        let finding = diagnostics[0].to_finding(text);
        let chars: Vec<char> = text.chars().collect();
        let span: String = chars[finding.start..finding.end].iter().collect();
        assert_eq!(span, "tã");
        assert_eq!(finding.severity, "advisory");
        assert_eq!(finding.suggestion.as_deref(), Some("tả"));
    }
}
