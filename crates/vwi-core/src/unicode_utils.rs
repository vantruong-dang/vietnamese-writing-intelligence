use unicode_normalization::UnicodeNormalization;

/// Chuẩn hóa chuỗi văn bản sang Unicode NFC.
pub fn normalize_nfc(text: &str) -> String {
    text.nfc().collect()
}

/// Cấu trúc hỗ trợ chuyển đổi $O(1)$ giữa UTF-8 byte offset và Unicode char offset.
pub struct CharOffsetMap {
    byte_to_char: Vec<usize>,
    total_chars: usize,
}

impl CharOffsetMap {
    pub fn new(text: &str) -> Self {
        let mut byte_to_char = Vec::with_capacity(text.len() + 1);
        let mut char_count: usize = 0;

        for (byte_idx, _) in text.char_indices() {
            while byte_to_char.len() < byte_idx {
                byte_to_char.push(char_count.saturating_sub(1));
            }
            byte_to_char.push(char_count);
            char_count += 1;
        }

        while byte_to_char.len() <= text.len() {
            byte_to_char.push(char_count);
        }

        Self {
            byte_to_char,
            total_chars: char_count,
        }
    }

    /// Trả về char offset tương ứng với byte offset.
    #[inline]
    pub fn byte_to_char(&self, byte_offset: usize) -> usize {
        if byte_offset >= self.byte_to_char.len() {
            self.total_chars
        } else {
            self.byte_to_char[byte_offset]
        }
    }

    /// Chuyển đổi tuple byte range `(start_byte, end_byte)` sang char range `(start_char, end_char)`.
    #[inline]
    pub fn convert_range(&self, start_byte: usize, end_byte: usize) -> (usize, usize) {
        (self.byte_to_char(start_byte), self.byte_to_char(end_byte))
    }

    /// Alias cho convert_range để đồng nhất phong cách gọi.
    #[inline]
    pub fn byte_range_to_char_range(&self, start_byte: usize, end_byte: usize) -> (usize, usize) {
        self.convert_range(start_byte, end_byte)
    }
}

/// Kiểm tra xem một span [start_byte..end_byte] trong text có nằm ở ranh giới từ (word boundary) hay không.
#[inline]
pub fn is_word_boundary(text: &str, start_byte: usize, end_byte: usize) -> bool {
    let bytes = text.as_bytes();
    if start_byte > 0 {
        if let Some(prev_char) = text[..start_byte].chars().next_back() {
            if prev_char.is_alphanumeric() || prev_char == '_' || prev_char == '-' {
                return false;
            }
        }
    }
    if end_byte < bytes.len() {
        if let Some(next_char) = text[end_byte..].chars().next() {
            if next_char.is_alphanumeric() || next_char == '_' || next_char == '-' {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_offset_mapping_vietnamese() {
        let text = "Trí tuệ nhân tạo";
        let mapper = CharOffsetMap::new(text);

        // "Trí " -> 'í' là 2 bytes trong UTF-8
        // Byte: T(0), r(1), í(2,3), ' '(4) -> char 3 (khoảng trắng)
        assert_eq!(mapper.byte_to_char(0), 0);
        assert_eq!(mapper.byte_to_char(4), 3);
        assert_eq!(mapper.byte_to_char(5), 4); // 't'

        // Ký tự cuối cùng:
        assert_eq!(mapper.byte_to_char(text.len()), 16);
    }
}
