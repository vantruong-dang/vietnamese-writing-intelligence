use crate::segmentation::tokenize_viterbi;
use crate::types::{Finding, PosTag, TokenSpan};
use crate::unicode_utils::normalize_nfc;

pub const CRIPPLED_TOPIC_RULE_ID: &str = "CRIPPLED_PREPOSITIONAL_TOPIC";
pub const PREP_CONJ_RULE_ID: &str = "PREPOSITION_CONJUNCTION_CONFLICT";

/// Danh sách vị từ ngoại hướng / đòi hỏi chủ thể chi phối (Governing Transitive Verbs).
/// Khi gắn trực tiếp sau khung giới từ mà không có dấu phẩy hoặc chủ ngữ thực sự,
/// câu sẽ phạm lỗi "Khung đề què" (Crippled Prepositional Topic) theo GS. Cao Xuân Hạo.
const GOVERNING_TRANSITIVE_VERBS: &[&str] = &[
    "đòi hỏi",
    "yêu cầu",
    "cho thấy",
    "chứng minh",
    "khẳng định",
    "chứng tỏ",
    "buộc",
    "khiến",
    "giúp",
    "làm cho",
    "phát hiện",
    "chỉ ra",
    "nêu rõ",
    "nhấn mạnh",
    "minh chứng",
    "đã chứng minh",
    "đã giúp",
    "đã buộc",
    "đã phát hiện",
    "đã chỉ ra",
    "đã đòi hỏi",
];

/// Shallow Parser phân tích cấu trúc Đề - Thuyết (Topic-Comment Engine) dựa trên
/// Ngữ pháp chức năng tiếng Việt của GS. Cao Xuân Hạo.
///
/// Phân tách câu thành Khung Đề (Prepositional/Circumstantial Frame) và Thuyết (Comment),
/// kiểm tra tính toàn vẹn của vị từ nòng cốt mà không bị giới hạn số lượng token.
pub fn parse_topic_comment(tokens: &[TokenSpan], norm_text: &str) -> Vec<Finding> {
    if tokens.is_empty() {
        return Vec::new();
    }

    let mut findings = Vec::new();

    // 1. Phân đoạn danh sách token thành từng câu (dựa trên dấu chấm, hỏi, than hoặc xuống dòng)
    let mut sentences: Vec<&[TokenSpan]> = Vec::new();
    let mut sent_start = 0;

    for (i, tok) in tokens.iter().enumerate() {
        let is_punct_term = tok.pos == Some(PosTag::Punct)
            && (tok.text == "."
                || tok.text == "?"
                || tok.text == "!"
                || tok.text == "..."
                || tok.text == "…");

        let is_newline_boundary = if i + 1 < tokens.len() {
            let next_tok = &tokens[i + 1];
            if next_tok.start > tok.end {
                norm_text
                    .chars()
                    .skip(tok.end)
                    .take(next_tok.start - tok.end)
                    .any(|c| c == '\n' || c == '\r')
            } else {
                false
            }
        } else {
            false
        };

        if is_punct_term || is_newline_boundary {
            if i >= sent_start {
                sentences.push(&tokens[sent_start..=i]);
            }
            sent_start = i + 1;
        }
    }

    if sent_start < tokens.len() {
        sentences.push(&tokens[sent_start..]);
    }

    // 2. Duyệt từng câu để phân tích cú pháp nông (Shallow Parsing)
    for sent in sentences {
        // Bỏ qua các dấu ngoặc kép hoặc ký tự mở đầu câu không mang ngữ nghĩa
        let lead_idx = sent.iter().position(|t| {
            t.pos != Some(PosTag::Punct)
                && !t
                    .text
                    .chars()
                    .all(|c| c.is_ascii_punctuation() || "“„”«»'\"()[]".contains(c))
        });

        let lead_idx = match lead_idx {
            Some(idx) => idx,
            None => continue,
        };

        let first_token = &sent[lead_idx];
        let first_lower = first_token.text.to_lowercase();

        // Kiểm tra xem câu có mở đầu bằng một Giới từ chỉ hoàn cảnh (Prepositional Frame Opener)
        let is_prep_opener = matches!(
            first_lower.as_str(),
            "trong" | "qua" | "bằng" | "với" | "với những" | "dưới" | "theo" | "ở" | "tại" | "từ"
        );

        if !is_prep_opener {
            continue;
        }

        let mut inter_count = 0;
        let mut prev_is_aspect = false;

        // Quét các token tiếp theo trong khung đề
        for tok in &sent[lead_idx + 1..] {
            let lower = tok.text.to_lowercase();

            // A. Ranh giới dấu câu: Dấu phẩy hoặc chấm phẩy đóng Khung Đề hợp thức
            if tok.text == "," || tok.text == ";" || tok.text == "—" {
                break;
            }

            // B. Trợ từ phân cách Đề - Thuyết ("thì", "là", "mà")
            // Theo GS. Cao Xuân Hạo, "thì" là hư từ đặc thù đánh dấu ranh giới Đề - Thuyết
            if lower == "thì" || lower == "mà" {
                break;
            }

            // C. Phát hiện xung đột giới từ - liên từ nghịch đối (PREPOSITION_CONJUNCTION_CONFLICT)
            // Cụm mở đầu bằng "Với / Qua / Bằng" nhưng lại kết nối với "nhưng / song"
            if (lower == "nhưng" || lower == "song") && inter_count >= 1 {
                let start_char = first_token.start;
                let end_char = tok.end;
                let matched_span: String = norm_text
                    .chars()
                    .skip(start_char)
                    .take(end_char - start_char)
                    .collect();

                findings.push(Finding::new(
                    PREP_CONJ_RULE_ID,
                    "topic_comment",
                    "major",
                    start_char,
                    end_char,
                    0.95,
                    "Xung đột cú pháp: Giới từ mở đầu trạng ngữ không thể kết hợp trực tiếp với liên từ phản đề 'nhưng'.",
                    "linguistic",
                    matched_span,
                ));
                break;
            }

            // D. Phát hiện vị từ ngoại hướng bị ép nhận Khung đề làm chủ thể (CRIPPLED_PREPOSITIONAL_TOPIC)
            let is_gov = GOVERNING_TRANSITIVE_VERBS
                .iter()
                .any(|&verb| lower == verb);

            if (is_gov || (prev_is_aspect && is_gov)) && inter_count >= 1 {
                let start_char = first_token.start;
                let end_char = tok.end;
                let matched_span: String = norm_text
                    .chars()
                    .skip(start_char)
                    .take(end_char - start_char)
                    .collect();

                findings.push(Finding::new(
                    CRIPPLED_TOPIC_RULE_ID,
                    "topic_comment",
                    "major",
                    start_char,
                    end_char,
                    0.95,
                    "Lẫn lộn Khung đề và Chủ đề: Giới từ biến cụm từ thành trạng ngữ/phương tiện, không thể làm chủ thể cho vị từ.",
                    "linguistic",
                    matched_span,
                ));
                break;
            }

            // Cập nhật trạng thái phó từ thời thể ("đã", "đang", "sẽ")
            if lower == "đã" || lower == "đang" || lower == "sẽ" {
                prev_is_aspect = true;
            } else {
                prev_is_aspect = false;
                inter_count += 1;
            }
        }
    }

    findings
}

/// Đánh giá lỗi Khung đề què (Crippled Prepositional Topic): Giới ngữ bị ép làm chủ thể cho vị từ.
pub fn evaluate_crippled_prepositional_topic(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let tokens = tokenize_viterbi(&norm);
    parse_topic_comment(&tokens, &norm)
        .into_iter()
        .filter(|f| f.pattern_id == CRIPPLED_TOPIC_RULE_ID)
        .collect()
}

/// Đánh giá lỗi Xung đột Cú pháp (Preposition Conjunction Conflict): Giới từ mở đầu kết hợp với 'nhưng'.
pub fn evaluate_preposition_conjunction_conflict(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let tokens = tokenize_viterbi(&norm);
    parse_topic_comment(&tokens, &norm)
        .into_iter()
        .filter(|f| f.pattern_id == PREP_CONJ_RULE_ID)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crippled_topic_detection() {
        let text = "Trong tình hình kinh tế hiện nay đòi hỏi doanh nghiệp phải tái cơ cấu toàn diện.";
        let findings = evaluate_crippled_prepositional_topic(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, CRIPPLED_TOPIC_RULE_ID);
        assert_eq!(
            findings[0].matched_span,
            "Trong tình hình kinh tế hiện nay đòi hỏi"
        );
    }

    #[test]
    fn test_prep_conj_conflict_detection() {
        let text = "Với những nỗ lực phi thường nhưng họ vẫn không thể hoàn thành dự án đúng hạn.";
        let findings = evaluate_preposition_conjunction_conflict(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, PREP_CONJ_RULE_ID);
        assert_eq!(
            findings[0].matched_span,
            "Với những nỗ lực phi thường nhưng"
        );
    }

    #[test]
    fn test_clean_topic_sentence() {
        let text = "Tình hình kinh tế hiện nay đòi hỏi doanh nghiệp phải tái cơ cấu.";
        let findings = evaluate_crippled_prepositional_topic(text);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_clean_with_comma_boundary() {
        let text = "Trong tình hình kinh tế hiện nay, các doanh nghiệp đang nỗ lực tái cơ cấu.";
        let findings = evaluate_crippled_prepositional_topic(text);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_clean_with_topic_marker_thi() {
        let text = "Trong hoàn cảnh khó khăn này thì mọi người cần đoàn kết gắn bó.";
        let findings = evaluate_crippled_prepositional_topic(text);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_clean_existential_sentence() {
        // Cao Xuân Hạo: Vị từ tồn tại "có" đi với khung đề chỉ nơi chốn là hoàn toàn chuẩn mực
        let text = "Trong nhà có năm người đang ngồi thảo luận kế hoạch.";
        let findings = evaluate_crippled_prepositional_topic(text);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_multi_sentence_detection() {
        let text = "Hôm nay thời tiết rất đẹp. Qua phân tích chuỗi cung ứng linh kiện khiến các nhà máy phải điều chỉnh ca làm.";
        let findings = evaluate_crippled_prepositional_topic(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].matched_span,
            "Qua phân tích chuỗi cung ứng linh kiện khiến"
        );
    }

    #[test]
    fn test_aspect_prefixed_verb() {
        let text = "Bằng những số liệu thực nghiệm đã chứng minh giả thuyết của nhóm là chính xác.";
        let findings = evaluate_crippled_prepositional_topic(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].matched_span,
            "Bằng những số liệu thực nghiệm đã chứng minh"
        );
    }
}
