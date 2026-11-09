use crate::segmentation::daachorse_segmenter::EMBEDDED_DICTIONARY;
use crate::segmentation::generated_lm::{BIGRAM_DATA, UNIGRAM_DATA};
use crate::segmentation::pos_tagger::tag_token_sequence;
use crate::types::TokenSpan;
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};
use std::collections::HashMap;
use std::sync::LazyLock;

/// Bảng tra cứu Unigram Log-Probabilities được khởi tạo trong RAM (dưới 2ms).
static UNIGRAM_MAP: LazyLock<HashMap<&'static str, f32>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(UNIGRAM_DATA.len());
    for &(word, log_prob) in UNIGRAM_DATA {
        map.insert(word, log_prob);
    }
    map
});

/// Bảng tra cứu Bigram Transition Log-Probabilities được khởi tạo trong RAM (dưới 2ms).
static BIGRAM_MAP: LazyLock<HashMap<(&'static str, &'static str), f32>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(BIGRAM_DATA.len());
    for &(w1, w2, log_prob) in BIGRAM_DATA {
        map.insert((w1, w2), log_prob);
    }
    map
});

#[derive(Clone, Debug)]
struct PathNode {
    score: f32,
    start_byte: usize,
    end_byte: usize,
    prev_node_byte: usize,
    is_compound: bool,
}

/// Bỏ qua khoảng trắng và trả về byte index bắt đầu của token kế tiếp.
#[inline(always)]
fn skip_whitespace(text: &str, from_byte: usize) -> usize {
    if from_byte >= text.len() {
        return text.len();
    }
    let mut offset = 0;
    for c in text[from_byte..].chars() {
        if c.is_whitespace() {
            offset += c.len_utf8();
        } else {
            break;
        }
    }
    from_byte + offset
}

/// Kiểm tra ranh giới từ hợp lệ: không cho phép cắt ngang một âm tiết hoặc ký tự chữ số.
#[inline(always)]
fn is_valid_word_boundary(text: &str, start_byte: usize, end_byte: usize) -> bool {
    let bytes = text.as_bytes();

    if start_byte > 0 {
        let prev_char = text[..start_byte].chars().next_back().unwrap();
        if prev_char.is_alphanumeric() || prev_char == '_' || prev_char == '-' {
            return false;
        }
    }

    if end_byte < bytes.len() {
        let next_char = text[end_byte..].chars().next().unwrap();
        if next_char.is_alphanumeric() || next_char == '_' || next_char == '-' {
            return false;
        }
    }

    true
}

/// Thuật toán Viterbi Trellis phân đoạn từ tiếng Việt dựa trên Mô hình Ngôn ngữ Thống kê (Unigram + Bigram + Length-Bonus).
/// Giải quyết triệt để các ca nhập nhằng hóc búa (như "bàn là" vs "bàn" + "là", "học sinh học").
pub fn tokenize_viterbi(raw_text: &str) -> Vec<TokenSpan> {
    if raw_text.trim().is_empty() {
        return Vec::new();
    }

    let normalized = normalize_nfc(raw_text);
    let lower = normalized.to_lowercase();
    let char_map = CharOffsetMap::new(&normalized);
    let len = normalized.len();

    // dp[byte_index]: Node tối ưu nhất kết thúc và nhảy tới byte_index
    let mut dp: Vec<Option<PathNode>> = vec![None; len + 1];

    let start_byte = skip_whitespace(&normalized, 0);
    dp[start_byte] = Some(PathNode {
        score: 0.0,
        start_byte,
        end_byte: start_byte,
        prev_node_byte: start_byte,
        is_compound: false,
    });

    for curr_byte in 0..len {
        let current_node = match &dp[curr_byte] {
            Some(node) => node.clone(),
            None => continue,
        };

        let prev_word_opt = if current_node.end_byte > current_node.start_byte {
            Some(&lower[current_node.start_byte..current_node.end_byte])
        } else {
            None
        };

        let max_word_bytes = (curr_byte + 45).min(len);
        let mut max_end = max_word_bytes;
        while max_end < len && !lower.is_char_boundary(max_end) {
            max_end += 1;
        }
        let search_slice = &lower[curr_byte..max_end];
        let mut matched_any = false;

        // 1. Quét từ điển daachorse trong phạm vi tối đa 45 bytes (từ ghép tiếng Việt tối đa 5-6 từ)
        for m in EMBEDDED_DICTIONARY.find_overlapping_iter(search_slice) {
            if m.start() == 0 {
                let match_len = m.end();
                let end_byte = curr_byte + match_len;

                if is_valid_word_boundary(&normalized, curr_byte, end_byte) {
                    matched_any = true;
                    let next_start = skip_whitespace(&normalized, end_byte);
                    let word_str = &lower[curr_byte..end_byte];
                    let syllables = word_str.split_whitespace().count().max(1);

                    // Điểm Unigram
                    let unigram_score = UNIGRAM_MAP.get(word_str).copied().unwrap_or(-15.0);

                    // Điểm thưởng độ dài (Length-Bonus): bù đắp cho từ ghép có tần suất thấp hơn từ đơn hư từ
                    let length_bonus = if syllables >= 2 {
                        (syllables as f32).powf(1.3) * 6.0
                    } else {
                        0.0
                    };

                    // Điểm Bigram chuyển tiếp
                    let bigram_bonus = if let Some(prev_word) = prev_word_opt {
                        if let Some(&log_p) = BIGRAM_MAP.get(&(prev_word, word_str)) {
                            (12.0 + log_p).max(1.0) * 1.5
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };

                    let new_score = current_node.score + unigram_score + length_bonus + bigram_bonus;

                    if dp[next_start].as_ref().map_or(true, |old| new_score > old.score) {
                        dp[next_start] = Some(PathNode {
                            score: new_score,
                            start_byte: curr_byte,
                            end_byte,
                            prev_node_byte: curr_byte,
                            is_compound: syllables >= 2,
                        });
                    }
                }
            }
        }

        // 2. Fallback: Nếu không khớp từ điển, gom từ đơn hoặc dấu câu
        if !matched_any {
            let next_char = normalized[curr_byte..].chars().next().unwrap();
            let mut end_byte = curr_byte + next_char.len_utf8();

            if next_char.is_alphanumeric() || next_char == '_' || next_char == '-' {
                while end_byte < len {
                    let c = normalized[end_byte..].chars().next().unwrap();
                    if c.is_whitespace() || !c.is_alphanumeric() && c != '_' && c != '-' {
                        break;
                    }
                    end_byte += c.len_utf8();
                }
            }

            let next_start = skip_whitespace(&normalized, end_byte);
            let fallback_score = current_node.score - 10.0; // Phạt từ lạ

            if dp[next_start].as_ref().map_or(true, |old| fallback_score > old.score) {
                dp[next_start] = Some(PathNode {
                    score: fallback_score,
                    start_byte: curr_byte,
                    end_byte,
                    prev_node_byte: curr_byte,
                    is_compound: false,
                });
            }
        }
    }

    // 3. Backtracking: Truy vết từ cuối chuỗi về đầu chuỗi
    let mut tokens = Vec::new();
    let mut curr_idx = len;

    while curr_idx > 0 && dp[curr_idx].is_none() {
        curr_idx -= 1;
    }

    while curr_idx > 0 {
        if let Some(node) = &dp[curr_idx] {
            if node.end_byte > node.start_byte {
                let (c_start, c_end) = char_map.convert_range(node.start_byte, node.end_byte);
                let tok_text = normalized[node.start_byte..node.end_byte].to_string();
                tokens.push(TokenSpan {
                    text: tok_text,
                    start: c_start,
                    end: c_end,
                    is_compound: node.is_compound,
                    pos: None,
                });
            }
            if node.prev_node_byte == node.start_byte && node.start_byte == 0 {
                break;
            }
            curr_idx = node.prev_node_byte;
        } else {
            break;
        }
    }

    tokens.reverse();
    let words: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
    let tags = tag_token_sequence(&words);
    for (t, tag) in tokens.iter_mut().zip(tags) {
        t.pos = Some(tag);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viterbi_ban_la_compound() {
        let text = "Mẹ mua một chiếc bàn là mới.";
        let tokens = tokenize_viterbi(text);
        let texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();

        assert!(texts.contains(&"chiếc"));
        assert!(texts.contains(&"bàn là"), "Phải nhận diện 'bàn là' là một từ ghép");
        assert!(texts.contains(&"mới"));
    }

    #[test]
    fn test_viterbi_hoc_sinh_hoc() {
        let text = "Học sinh nghiên cứu môn sinh học trong phòng thí nghiệm.";
        let tokens = tokenize_viterbi(text);
        let texts: Vec<String> = tokens.iter().map(|t| t.text.to_lowercase()).collect();

        assert!(texts.contains(&"học sinh".to_string()));
        assert!(texts.contains(&"sinh học".to_string()));
    }

    #[test]
    fn test_viterbi_ban_la_disambiguation() {
        // Ca 1: Bàn là (bàn ủi)
        let text1 = "Mẹ mua một chiếc bàn là mới.";
        let tokens1: Vec<String> = tokenize_viterbi(text1).iter().map(|t| t.text.clone()).collect();
        assert!(tokens1.contains(&"bàn là".to_string()), "Phải ghép 'bàn là' (bàn ủi)");

        // Ca 2: Bàn + là (cái bàn + hệ từ)
        let text2 = "Cái bàn là một đồ vật.";
        let tokens2: Vec<String> = tokenize_viterbi(text2).iter().map(|t| t.text.clone()).collect();
        assert!(tokens2.contains(&"bàn".to_string()), "Phải tách 'bàn'");
        assert!(tokens2.contains(&"là".to_string()), "Phải tách 'là'");
        assert!(!tokens2.contains(&"bàn là".to_string()), "Không được ghép 'bàn là' trong ngữ cảnh này");
    }

    #[test]
    fn test_viterbi_fallback_english_and_punctuation() {
        let text = "Năm 2026, hệ thống AI của VNG sẽ dùng Kubernetes!";
        let tokens = tokenize_viterbi(text);
        let texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();

        // Không được băm nát "Kubernetes" hay "2026"
        assert!(texts.contains(&"2026"));
        assert!(texts.contains(&"Kubernetes"));
        assert!(texts.contains(&"AI"));
        assert!(texts.contains(&"VNG"));
        // Dấu câu phải được tách riêng
        assert!(texts.contains(&","));
        assert!(texts.contains(&"!"));
    }

    #[test]
    fn test_viterbi_word_boundary_guard() {
        // Giả sử từ điển có từ "an", "to", "học"
        let text = "Thanh toán học phí.";
        let tokens = tokenize_viterbi(text);
        let texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();

        // Không được match "an" trong "Thanh" hay "to" trong "toán"
        assert!(!texts.contains(&"an"), "Lỗi match lén 'an' trong 'Thanh'");
        assert!(!texts.contains(&"to"), "Lỗi match lén 'to' trong 'toán'");

        // Phải tách đúng "Thanh toán" (nếu có trong từ điển) + "học phí"
        assert!(texts.contains(&"học phí"), "Không nhận diện được 'học phí'");
    }

    #[test]
    fn test_viterbi_empty_and_whitespace_input() {
        assert_eq!(tokenize_viterbi("").len(), 0);
        assert_eq!(tokenize_viterbi("   \n\t  ").len(), 0);
    }

    #[test]
    fn test_viterbi_char_span_integrity() {
        let text = "Trí tuệ nhân tạo và chuyển đổi số đang tối ưu hóa hệ thống phân tán.";
        let tokens = tokenize_viterbi(text);
        let text_chars: Vec<char> = normalize_nfc(text).chars().collect();

        for tok in &tokens {
            let extracted: String = text_chars[tok.start..tok.end].iter().collect();
            assert_eq!(tok.text, extracted, "Lệch tọa độ token: {:?}", tok);
        }
    }
}
