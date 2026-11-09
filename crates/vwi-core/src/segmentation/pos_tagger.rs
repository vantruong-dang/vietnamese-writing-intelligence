use crate::types::PosTag;

include!("generated_pos.rs");

/// Suy luận nhãn từ loại dự phòng (Heuristic Fallback) cho từ ngoài từ điển (OOV).
pub fn infer_oov_pos(word: &str, prev_tag: Option<PosTag>) -> PosTag {
    let lower = word.to_lowercase();
    let trimmed = lower.trim();
    let chars: Vec<char> = trimmed.chars().collect();

    // 1. Ký tự trống
    if chars.is_empty() {
        return PosTag::Other;
    }

    // 2. Dấu câu / Ký tự đặc biệt đơn lẻ
    if chars.len() == 1 && !chars[0].is_alphanumeric() {
        return PosTag::Punct;
    }

    // 3. Số từ (Digits / Percent)
    if chars
        .iter()
        .all(|c| c.is_ascii_digit() || *c == '.' || *c == ',' || *c == '%' || *c == '/')
    {
        return PosTag::Num;
    }

    // 4. Hình thái học: Tiền tố danh từ hóa
    if trimmed.starts_with("sự ")
        || trimmed.starts_with("việc ")
        || trimmed.starts_with("cuộc ")
        || trimmed.starts_with("chiếc ")
        || trimmed.starts_with("cái ")
        || trimmed.starts_with("bức ")
        || trimmed.starts_with("tấm ")
        || trimmed.starts_with("người ")
        || trimmed.starts_with("nhà ")
        || trimmed.ends_with(" tính")
    {
        return PosTag::Noun;
    }

    // 5. Hình thái học: Hậu tố động từ hóa (-hóa)
    if trimmed.ends_with(" hóa") || trimmed.ends_with(" hoá") {
        return PosTag::Verb;
    }

    // 6. Ngữ cảnh vị trí (Syntactic Context Fallback từ Bigram)
    if let Some(prev) = prev_tag {
        match prev {
            // Sau giới từ thường là danh từ: "trong [phòng]", "bằng [phương tiện]"
            PosTag::Adp => return PosTag::Noun,
            // Sau phó từ chỉ mức độ thường là tính từ / vị từ: "rất [nhanh]", "quá [cao]"
            PosTag::Adv => return PosTag::Adj,
            _ => {}
        }
    }

    // 7. Mặc định mở cho tiếng Việt (Open class standard)
    PosTag::Noun
}

/// Gán nhãn một từ đơn lẻ (kèm tra cứu O(log N) và OOV fallback).
pub fn tag_pos(word: &str) -> PosTag {
    if let Some(tag) = lookup_pos_tag(word) {
        tag
    } else {
        infer_oov_pos(word, None)
    }
}

/// Gán nhãn toàn bộ chuỗi token có xét ngữ cảnh Bigram Transition.
pub fn tag_token_sequence(tokens: &[&str]) -> Vec<PosTag> {
    if tokens.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(tokens.len());
    let mut prev_tag = PosTag::Punct;

    for &word in tokens {
        let tag = if let Some(direct_tag) = lookup_pos_tag(word) {
            direct_tag
        } else {
            infer_oov_pos(word, Some(prev_tag))
        };

        result.push(tag);
        prev_tag = tag;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_tagging_lookup_and_heuristics() {
        // Closed-class lookups
        assert_eq!(tag_pos("trong"), PosTag::Adp);
        assert_eq!(tag_pos("với"), PosTag::Adp);
        assert_eq!(tag_pos("qua"), PosTag::Adp);
        assert_eq!(tag_pos("và"), PosTag::Cconj);
        assert_eq!(tag_pos("nhưng"), PosTag::Cconj);
        assert_eq!(tag_pos("tuy"), PosTag::Sconj);
        assert_eq!(tag_pos("nếu"), PosTag::Sconj);
        assert_eq!(tag_pos("thì"), PosTag::Part);
        assert_eq!(tag_pos("tôi"), PosTag::Pron);
        assert_eq!(tag_pos("đòi hỏi"), PosTag::Verb);

        // Heuristics
        assert_eq!(tag_pos("tối ưu hóa"), PosTag::Verb);
        assert_eq!(tag_pos("2026"), PosTag::Num);
        assert_eq!(tag_pos("."), PosTag::Punct);

        // Sequence context
        let words = vec!["trong", "phòng", "thí nghiệm"];
        let tags = tag_token_sequence(&words);
        assert_eq!(tags[0], PosTag::Adp);
        assert_eq!(tags[1], PosTag::Noun);
    }
}
