use crate::segmentation::pos_tagger::tag_pos;
use crate::types::TokenSpan;
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};
use daachorse::DoubleArrayAhoCorasick;
use std::sync::LazyLock;

/// Dữ liệu từ điển 62.550+ từ ghép tiếng Việt được nhúng trực tiếp vào binary khi biên dịch.
/// Đảm bảo Zero Runtime I/O và khởi tạo cực nhanh.
const EMBEDDED_LEXICON_DATA: &str = include_str!("../../data/lexicon_dictionary.txt");

/// Double-Array Aho-Corasick Trie toàn cục cho từ điển chuẩn tiếng Việt.
pub static EMBEDDED_DICTIONARY: LazyLock<DoubleArrayAhoCorasick<usize>> = LazyLock::new(|| {
    let mut words: Vec<String> = EMBEDDED_LEXICON_DATA
        .lines()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    words.sort();
    words.dedup();
    DoubleArrayAhoCorasick::new(words).expect("Khởi tạo DoubleArrayAhoCorasick nhúng thất bại")
});

pub struct DaachorseSegmenter {
    pma: Option<DoubleArrayAhoCorasick<usize>>,
}

impl DaachorseSegmenter {
    /// Khởi tạo bộ tách từ sử dụng từ điển 62.000+ từ ghép nhúng sẵn.
    pub fn new() -> Self {
        Self { pma: None }
    }

    /// Khởi tạo bộ tách từ với tập từ điển người dùng tự định nghĩa.
    pub fn with_vocab(custom_vocab: &[&str]) -> Result<Self, String> {
        let mut patterns: Vec<String> = custom_vocab
            .iter()
            .map(|w| normalize_nfc(w).to_lowercase())
            .collect();
        patterns.sort();
        patterns.dedup();

        let pma = DoubleArrayAhoCorasick::new(patterns)
            .map_err(|e| format!("Lỗi khởi tạo Daachorse: {:?}", e))?;
        Ok(Self { pma: Some(pma) })
    }

    /// Tách từ theo giải thuật tham lam cực đại (Greedy / Longest Match First),
    /// giải quyết triệt để bài toán nhập nhằng chồng chéo (Overlap Ambiguity).
    pub fn tokenize(&self, text: &str) -> Vec<TokenSpan> {
        let norm = normalize_nfc(text);
        let lower = norm.to_lowercase();
        let mapper = CharOffsetMap::new(&norm);

        let pma = self.pma.as_ref().unwrap_or(&EMBEDDED_DICTIONARY);

        // 1. Tìm tất cả các ứng viên từ ghép bằng Daachorse
        let mut raw_matches: Vec<(usize, usize)> = pma
            .find_overlapping_iter(&lower)
            .map(|m| (m.start(), m.end()))
            .collect();

        // 2. Lọc biên từ (Word Boundary Check)
        // Đảm bảo không khớp một từ ghép nằm lưng chừng giữa các âm tiết khác
        raw_matches.retain(|&(start, end)| {
            Self::is_boundary(&lower, start, true) && Self::is_boundary(&lower, end, false)
        });

        // 3. Giải thuật Longest Match First (Greedy Disambiguation)
        // Tiêu chí sắp xếp:
        // - Vị trí start nhỏ hơn đứng trước (duyệt từ trái qua phải).
        // - Nếu cùng start, độ dài (end - start) lớn hơn đứng trước (ưu tiên từ dài nhất).
        raw_matches.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| (b.1 - b.0).cmp(&(a.1 - a.0))));

        let mut non_overlapping_compounds: Vec<(usize, usize)> = Vec::new();
        let mut occupied_until = 0;

        for (start, end) in raw_matches {
            // Nếu ứng viên bắt đầu sau hoặc tại vị trí vùng đã chiếm đóng, ta chọn nó
            if start >= occupied_until {
                non_overlapping_compounds.push((start, end));
                occupied_until = end;
            }
        }

        // 4. Lắp ráp kết quả: kết hợp từ ghép và phân rã các khoảng trống thành từ đơn / dấu câu
        let mut tokens: Vec<TokenSpan> = Vec::new();
        let mut cursor = 0;

        for (comp_start, comp_end) in non_overlapping_compounds {
            // Xử lý khoảng trống giữa cursor và comp_start
            if cursor < comp_start {
                Self::tokenize_simple_range(
                    &norm,
                    cursor,
                    comp_start,
                    &mapper,
                    &mut tokens,
                );
            }

            // Thêm token từ ghép (chỉ đánh dấu is_compound nếu chứa khoảng trắng / từ 2 âm tiết trở lên)
            let (start_char, end_char) = mapper.convert_range(comp_start, comp_end);
            let matched_text = norm[comp_start..comp_end].to_string();
            let is_compound = matched_text.contains(' ');

            let pos = Some(tag_pos(&matched_text));
            tokens.push(TokenSpan {
                text: matched_text,
                start: start_char,
                end: end_char,
                is_compound,
                pos,
            });

            cursor = comp_end;
        }

        // Xử lý phần còn lại sau compound cuối cùng
        if cursor < norm.len() {
            Self::tokenize_simple_range(
                &norm,
                cursor,
                norm.len(),
                &mapper,
                &mut tokens,
            );
        }

        tokens
    }

    /// Kiểm tra ranh giới từ (ký tự trước hoặc sau là whitespace, dấu câu hoặc ranh giới chuỗi).
    #[inline]
    fn is_boundary(text: &str, byte_idx: usize, is_start: bool) -> bool {
        if is_start {
            if byte_idx == 0 {
                return true;
            }
            let prev_char = text[..byte_idx].chars().next_back().unwrap();
            !prev_char.is_alphanumeric() && prev_char != '_' && prev_char != '-'
        } else {
            if byte_idx >= text.len() {
                return true;
            }
            let next_char = text[byte_idx..].chars().next().unwrap();
            !next_char.is_alphanumeric() && next_char != '_' && next_char != '-'
        }
    }

    /// Tách các từ đơn và dấu câu trong khoảng ký tự không phải từ ghép.
    fn tokenize_simple_range(
        norm: &str,
        start_byte: usize,
        end_byte: usize,
        mapper: &CharOffsetMap,
        tokens: &mut Vec<TokenSpan>,
    ) {
        let slice = &norm[start_byte..end_byte];
        let mut char_iter = slice.char_indices().peekable();

        while let Some((rel_start, ch)) = char_iter.next() {
            if ch.is_whitespace() {
                continue;
            }

            let tok_start_byte = start_byte + rel_start;

            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                let mut tok_end_byte = tok_start_byte + ch.len_utf8();
                while let Some(&(next_rel, next_ch)) = char_iter.peek() {
                    if next_ch.is_alphanumeric() || next_ch == '_' || next_ch == '-' {
                        tok_end_byte = start_byte + next_rel + next_ch.len_utf8();
                        char_iter.next();
                    } else {
                        break;
                    }
                }
                let (start_char, end_char) = mapper.convert_range(tok_start_byte, tok_end_byte);
                let text = norm[tok_start_byte..tok_end_byte].to_string();
                let pos = Some(tag_pos(&text));
                tokens.push(TokenSpan {
                    text,
                    start: start_char,
                    end: end_char,
                    is_compound: false,
                    pos,
                });
            } else {
                let tok_end_byte = tok_start_byte + ch.len_utf8();
                let (start_char, end_char) = mapper.convert_range(tok_start_byte, tok_end_byte);
                let text = norm[tok_start_byte..tok_end_byte].to_string();
                let pos = Some(tag_pos(&text));
                tokens.push(TokenSpan {
                    text,
                    start: start_char,
                    end: end_char,
                    is_compound: false,
                    pos,
                });
            }
        }
    }
}

impl Default for DaachorseSegmenter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_dictionary_loading() {
        let segmenter = DaachorseSegmenter::new();
        let text = "Học sinh trong căn phòng thí nghiệm.";
        let tokens = segmenter.tokenize(text);

        let compounds: Vec<String> = tokens
            .iter()
            .filter(|t| t.is_compound)
            .map(|t| t.text.to_lowercase())
            .collect();

        assert!(compounds.iter().any(|s| s == "học sinh"));
        assert!(compounds.iter().any(|s| s == "căn phòng"));
        assert!(compounds.iter().any(|s| s == "thí nghiệm"));
    }

    #[test]
    fn test_overlap_disambiguation() {
        let segmenter = DaachorseSegmenter::new();
        // Nhập nhằng kinh điển: "học sinh" vs "sinh học" trong cụm "học sinh học"
        // Thuật toán Greedy Left-to-Right ưu tiên "học sinh" (vị trí trước), sau đó còn lại "học" (từ đơn)
        let text = "học sinh học";
        let tokens = segmenter.tokenize(text);

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].text, "học sinh");
        assert!(tokens[0].is_compound);
        assert_eq!(tokens[1].text, "học");
        assert!(!tokens[1].is_compound);

        // 2. Minh chứng Longest Match First:
        // Trong "nghiên cứu sinh học": "nghiên cứu sinh" (3 âm tiết) dài hơn "nghiên cứu" (2 âm tiết)
        // nên Longest Match ưu tiên chọn "nghiên cứu sinh", còn lại "học"
        let text2 = "nghiên cứu sinh học";
        let tokens2 = segmenter.tokenize(text2);
        assert_eq!(tokens2.len(), 2);
        assert_eq!(tokens2[0].text, "nghiên cứu sinh");
        assert!(tokens2[0].is_compound);
        assert_eq!(tokens2[1].text, "học");
        assert!(!tokens2[1].is_compound);

        // 3. Khi "sinh học" đứng sau một cụm từ không chồng lấn:
        let text3 = "học phần sinh học";
        let tokens3 = segmenter.tokenize(text3);
        let compounds3: Vec<String> = tokens3
            .iter()
            .filter(|t| t.is_compound)
            .map(|t| t.text.to_lowercase())
            .collect();
        assert!(compounds3.iter().any(|s| s == "học phần"));
        assert!(compounds3.iter().any(|s| s == "sinh học"));
    }

    #[test]
    fn test_longest_match_priority() {
        let segmenter = DaachorseSegmenter::new();
        // "bàn là" vs "bàn" (nếu có)
        let text = "Mẹ mua một chiếc bàn là mới.";
        let tokens = segmenter.tokenize(text);

        let compounds: Vec<&str> = tokens
            .iter()
            .filter(|t| t.is_compound)
            .map(|t| t.text.as_str())
            .collect();
        assert!(compounds.contains(&"bàn là"));
    }

    #[test]
    fn test_char_span_integrity_with_62k_words() {
        let segmenter = DaachorseSegmenter::new();
        let text = "Trí tuệ nhân tạo và chuyển đổi số đang tối ưu hóa hệ thống phân tán.";
        let tokens = segmenter.tokenize(text);

        let text_chars: Vec<char> = normalize_nfc(text).chars().collect();
        for tok in &tokens {
            let extracted: String = text_chars[tok.start..tok.end].iter().collect();
            assert_eq!(tok.text, extracted, "Lệch tọa độ token: {:?}", tok);
        }
    }
}
