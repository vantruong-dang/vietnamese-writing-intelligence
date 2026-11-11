//! # CẢM BIẾN CẤU TRÚC CÂU BẰNG POS TAGGER (ADVISORY)
//!
//! Hai luật "Tầng 2" được thiết kế cực kỳ bảo thủ để không bắt oan văn xuôi
//! và văn bản hành chính:
//!
//! 1. `SUBJECT_VERB_GAP` — chủ ngữ và vị ngữ cách nhau quá xa (**> 18 token,
//!    không có dấu phẩy/ngắt câu**) → dấu hiệu "câu thọt".
//! 2. `LIST_STRUCTURE_MISMATCH` — phép liệt kê dạng `X, Y và Z` với X/Y/Z là
//!    từ đơn, trong đó trộn lẫn Danh từ và Động từ → sai cấu trúc liệt kê.
//!
//! Cả hai luật luôn `Advisory` (0 điểm phạt) vì POS tagger tiếng Việt vẫn có
//! thể nhầm trên câu phức tạp. Đây là tín hiệu tư vấn, không phải lỗi cứng.
//!
//! ## Vì sao không port Viterbi POS
//! Đo trên UD_Vietnamese-VTB (13 nhãn): most-frequent 89,32/64,95;
//! HMM Viterbi 86,17/65,03; HMM+suffix 87,40/65,91 (acc/ambig). Viterbi
//! không thắng most-frequent về tổng và chỉ hơn ~1 điểm trên từ đa nghĩa,
//! nên không đáng port. Các rule ở đây là pattern nông có đo lường, không
//! phải POS tagger đầy đủ; classifier (`cây chó → con chó`) là lexicon lớp
//! ngữ nghĩa, không phải UPOS.

use crate::segmentation::viterbi_segmenter::tokenize_viterbi;
use crate::types::{Finding, PosTag, TokenSpan};

pub const SUBJECT_VERB_GAP_RULE_ID: &str = "SUBJECT_VERB_GAP";
pub const LIST_STRUCTURE_MISMATCH_RULE_ID: &str = "LIST_STRUCTURE_MISMATCH";
pub const CLASSIFIER_MISMATCH_RULE_ID: &str = "CLASSIFIER_MISMATCH";
pub const ADVERB_PLACEMENT_RULE_ID: &str = "ADVERB_PLACEMENT";

/// Chỉ các phó từ **bắt buộc đứng trước** tính từ. "quá"/"lắm" đứng sau là
/// đúng ("hay quá", "đẹp lắm") nên không đưa vào.
static PRE_ADJECTIVE_ADVERBS: &[&str] = &["rất", "hơi"];

/// Danh từ chỉ động vật — không đi với loại từ "cây" (thực vật/đồ vật dài).
static ANIMAL_NOUNS: &[&str] = &[
    "chó", "mèo", "gà", "vịt", "ngan", "ngỗng", "trâu", "bò", "ngựa", "dê", "cừu", "lợn", "heo",
    "cá", "chim", "tôm", "cua", "ốc", "thỏ", "hổ", "voi", "khỉ",
];

/// Ngưỡng khoảng cách tối thiểu (số token) để coi là "câu thọt".
/// Đặt cao hơn đề xuất 15 để bảo vệ câu hành chính có cụm danh từ dài.
const SUBJECT_VERB_GAP_THRESHOLD: usize = 18;

fn tag_of(token: &TokenSpan) -> PosTag {
    token.pos.unwrap_or(PosTag::Other)
}

fn is_simple_list_item(token: &TokenSpan) -> bool {
    !token.is_compound
        && token.text.chars().count() >= 2
        && token.text.chars().all(|c| c.is_alphabetic())
}

fn has_clause_break(text: &str, start: usize, end: usize) -> bool {
    let chars: Vec<char> = text.chars().collect();
    if start >= end || end > chars.len() {
        return false;
    }
    chars[start..end]
        .iter()
        .any(|c| matches!(c, ',' | ';' | ':' | '.' | '!' | '?'))
}

pub fn evaluate_subject_verb_gap(text: &str) -> Vec<Finding> {
    let tokens = tokenize_viterbi(text);
    if tokens.len() < SUBJECT_VERB_GAP_THRESHOLD + 4 {
        return Vec::new();
    }

    // Chọn chủ ngữ khả dĩ đầu tiên: Danh từ/Đại từ, không nằm ngay sau giới từ.
    let mut subject_idx = None;
    for (i, token) in tokens.iter().enumerate() {
        let tag = tag_of(token);
        if !matches!(tag, PosTag::Noun | PosTag::Pron) {
            continue;
        }
        if i > 0 && tag_of(&tokens[i - 1]) == PosTag::Adp {
            continue;
        }
        subject_idx = Some(i);
        break;
    }

    let Some(subject_idx) = subject_idx else {
        return Vec::new();
    };

    let Some((verb_idx, verb)) = tokens
        .iter()
        .enumerate()
        .skip(subject_idx + 1)
        .find(|(_, token)| tag_of(token) == PosTag::Verb)
    else {
        return Vec::new();
    };

    let gap = verb_idx - subject_idx - 1;
    if gap <= SUBJECT_VERB_GAP_THRESHOLD {
        return Vec::new();
    }

    let subject = &tokens[subject_idx];
    if has_clause_break(text, subject.end, verb.start) {
        return Vec::new();
    }

    let chars: Vec<char> = text.chars().collect();
    if verb.end > chars.len() {
        return Vec::new();
    }
    let span: String = chars[subject.start..verb.end].iter().collect();

    vec![Finding::new(
        SUBJECT_VERB_GAP_RULE_ID,
        "syntax",
        "advisory",
        subject.start,
        verb.end,
        0.75,
        format!(
            "Chủ ngữ và vị ngữ cách nhau {} token mà không có dấu ngắt câu; câu có thể bị 'thọt' hoặc khó đọc.",
            gap
        ),
        "pos_structure",
        span,
    )
    .with_note("Cân nhắc rút ngắn cụm chủ ngữ hoặc tách thành hai câu.")
    .with_stage("pos")]
}

pub fn evaluate_list_structure_mismatch(text: &str) -> Vec<Finding> {
    let tokens = tokenize_viterbi(text);
    let mut findings = Vec::new();

    for k in 3..tokens.len().saturating_sub(1) {
        if !tokens[k].text.eq_ignore_ascii_case("và") {
            continue;
        }

        let first = &tokens[k - 3];
        let comma = &tokens[k - 2];
        let second = &tokens[k - 1];
        let third = &tokens[k + 1];

        if comma.text != ","
            || !is_simple_list_item(first)
            || !is_simple_list_item(second)
            || !is_simple_list_item(third)
        {
            continue;
        }

        let tags = [tag_of(first), tag_of(second), tag_of(third)];
        let has_noun = tags.contains(&PosTag::Noun);
        let has_verb = tags.contains(&PosTag::Verb);

        // Chỉ bắt khi trộn hẳn Danh từ với Động từ. Tính từ + Danh từ là
        // cụm danh từ hợp lệ trong tiếng Việt nên không coi là lỗi.
        if !(has_noun && has_verb) {
            continue;
        }

        let chars: Vec<char> = text.chars().collect();
        if third.end > chars.len() {
            continue;
        }
        let span: String = chars[first.start..third.end].iter().collect();

        findings.push(
            Finding::new(
                LIST_STRUCTURE_MISMATCH_RULE_ID,
                "syntax",
                "advisory",
                first.start,
                third.end,
                0.72,
                "Các thành phần trong phép liệt kê không cùng loại từ (Danh từ lẫn Động từ).",
                "pos_structure",
                span,
            )
            .with_note("Đưa các thành phần liệt kê về cùng từ loại để câu song song, dễ đọc.")
            .with_stage("pos"),
        );
    }

    findings
}

/// Phát hiện trật tự "Tính từ + rất/hơi" (sai) và đảo lại thành "rất/hơi + Tính từ".
///
/// Đây là chỗ POS thực sự có ích: "hay" trong từ điển bị gán `Cconj`, nhưng
/// khi đứng sau một động từ ("hát hay") nó là tính từ. Ta khử nhập nhằng bằng
/// ngữ cảnh POS cục bộ (động từ phía trước) thay vì sửa từ điển.
pub fn evaluate_adverb_placement(text: &str) -> Vec<Finding> {
    let tokens = tokenize_viterbi(text);
    let mut findings = Vec::new();

    for i in 1..tokens.len() {
        let adverb = &tokens[i];
        if !PRE_ADJECTIVE_ADVERBS
            .iter()
            .any(|a| adverb.text.eq_ignore_ascii_case(a))
        {
            continue;
        }
        // Phải kết thúc vế câu: hết câu hoặc gặp dấu câu.
        let ends_clause = tokens
            .get(i + 1)
            .map_or(true, |t| tag_of(t) == PosTag::Punct);
        if !ends_clause {
            continue;
        }

        let adjective = &tokens[i - 1];
        let tag = tag_of(adjective);
        let is_adjective = tag == PosTag::Adj
            || (adjective.text.eq_ignore_ascii_case("hay")
                && i >= 2
                && tag_of(&tokens[i - 2]) == PosTag::Verb);
        if !is_adjective {
            continue;
        }

        let chars: Vec<char> = text.chars().collect();
        if adverb.end > chars.len() {
            continue;
        }
        let span: String = chars[adjective.start..adverb.end].iter().collect();
        let replacement = format!("{} {}", adverb.text, adjective.text);

        findings.push(
            Finding::new(
                ADVERB_PLACEMENT_RULE_ID,
                "syntax",
                "advisory",
                adjective.start,
                adverb.end,
                0.9,
                format!(
                    "Trật tự phó từ sai: '{}' phải đứng trước tính từ ('{}').",
                    span, replacement
                ),
                "pos_structure",
                span,
            )
            .with_fix(adjective.start, adverb.end, replacement)
            .with_stage("pos"),
        );
    }

    findings
}

/// Loại từ "cây" không tương thích với danh từ động vật → gợi ý "con".
pub fn evaluate_classifier_mismatch(text: &str) -> Vec<Finding> {
    let tokens = tokenize_viterbi(text);
    let mut findings = Vec::new();

    for window in tokens.windows(2) {
        let classifier = &window[0];
        let noun = &window[1];
        if !classifier.text.eq_ignore_ascii_case("cây") {
            continue;
        }
        let noun_lower = noun.text.to_lowercase();
        if !ANIMAL_NOUNS.contains(&noun_lower.as_str()) {
            continue;
        }
        let chars: Vec<char> = text.chars().collect();
        if classifier.end > chars.len() {
            continue;
        }
        findings.push(
            Finding::new(
                CLASSIFIER_MISMATCH_RULE_ID,
                "syntax",
                "advisory",
                classifier.start,
                classifier.end,
                0.88,
                format!(
                    "Loại từ 'cây' không dùng cho động vật ('{}'); nên dùng 'con'.",
                    noun.text
                ),
                "pos_structure",
                classifier.text.clone(),
            )
            .with_fix(classifier.start, classifier.end, "con")
            .with_stage("pos"),
        );
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_verb_gap_flags_very_long_gap() {
        // 21 token toàn danh từ chen giữa chủ ngữ "Tôi" và vị ngữ "quyết định",
        // không có dấu phẩy — đúng dạng "câu thọt" mà luật nhắm tới.
        let text = "Tôi trời đất nước non sông núi rừng biển đảo cây cỏ hoa lá chim thú cá tôm cua ốc gà vịt ngan ngỗng chó mèo trâu bò ngựa dê cừu quyết định.";
        let findings = evaluate_subject_verb_gap(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, "advisory");
    }

    #[test]
    fn test_subject_verb_gap_ignores_normal_sentences() {
        for text in [
            "Chúng tôi đã hoàn thành dự án đúng hạn.",
            "Theo quy định của pháp luật hiện hành, các bộ ngành phải báo cáo trước ngày 30.",
            "Báo cáo tài chính đã được công bố bởi hội đồng quản trị.",
        ] {
            assert!(
                evaluate_subject_verb_gap(text).is_empty(),
                "Bắt oan câu thọt: {:?}",
                text
            );
        }
    }

    #[test]
    fn test_list_structure_flags_mixed_noun_verb() {
        let findings = evaluate_list_structure_mismatch("Cô ấy mua sách, bút và chạy.");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, "advisory");
    }

    #[test]
    fn test_adverb_placement_disambiguates_hay() {
        for (text, expected) in [
            ("Anh ấy hát hay rất.", "rất hay"),
            ("Cô ấy đẹp rất.", "rất đẹp"),
            ("Xe chạy nhanh rất.", "rất nhanh"),
        ] {
            let findings = evaluate_adverb_placement(text);
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
        // "hay quá", "đẹp lắm", "rất hay" đều đúng.
        for text in ["Hay quá!", "Đẹp lắm.", "Cô ấy rất đẹp."] {
            assert!(
                evaluate_adverb_placement(text).is_empty(),
                "Bắt oan: {:?}",
                text
            );
        }
    }

    #[test]
    fn test_classifier_mismatch_animal() {
        assert_eq!(
            evaluate_classifier_mismatch("Cô ấy nuôi một cây chó.").len(),
            1
        );
        // "cây bút", "cây cầu" hợp lệ.
        assert!(evaluate_classifier_mismatch("Trên bàn có một cây bút.").is_empty());
        assert!(evaluate_classifier_mismatch("Cây cầu bắc qua sông.").is_empty());
    }

    #[test]
    fn test_list_structure_ignores_parallel_lists() {
        for text in [
            "Cán bộ, công chức và nhân dân cùng tham gia.",
            "Phong cách nhanh, gọn và chính xác.",
            "Chúng tôi ăn, ngủ và làm việc tại đây.",
        ] {
            assert!(
                evaluate_list_structure_mismatch(text).is_empty(),
                "Bắt oan liệt kê: {:?}",
                text
            );
        }
    }
}
