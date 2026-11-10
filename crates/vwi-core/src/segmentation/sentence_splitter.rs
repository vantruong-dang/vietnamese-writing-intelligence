//! # BỘ BÓC TÁCH RANH GIỚI CÂU (SENTENCE SPLITTER & BOUNDED SCOPE)
//!
//! Tách văn bản thành các câu độc lập với tọa độ ký tự chính xác tuyệt đối.
//! Áp dụng hệ thống miễn trừ nghiêm ngặt chống gãy câu:
//! - Học hàm, học vị: `GS.`, `PGS.`, `TS.`, `ThS.`, `BS.`, `KS.`
//! - Địa danh, cơ quan: `TP.`, `TP.HCM`, `UBND`, `HĐND`, `Q.`, `H.`, `P.`
//! - Viết tắt hành chính & liệt kê: `v.v.`, `đ/c`, `e.g.`, `i.e.`
//! - Số thập phân, tỷ lệ, phiên bản: `3.14`, `10.000`, `v0.3.1`
//! - Tên miền & đường dẫn: `vnexpress.net`, `https://...`
//! - Cụm dấu kết thúc: `...`, `."`, `!)`, `?]`

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceSpan<'a> {
    pub text: &'a str,
    pub start_char: usize,
    pub end_char: usize,
}

const COMMON_ABBREVIATIONS: &[&str] = &[
    "gs.", "pgs.", "ts.", "ths.", "bs.", "ks.", "cn.", "th.s.", "th.s",
    "tp.", "tp.hcm", "tphcm", "tt.", "ubnd", "hđnd", "bch", "tw",
    "q.", "h.", "p.", "x.", "đ/c.", "đ/c",
    "v.v.", "v.v", "e.g.", "i.e.", "etc.", "st.", "dr.", "mr.", "ms.", "mrs.",
];

/// Tách văn bản thành danh sách các phân đoạn câu độc lập
pub fn split_sentences(text: &str) -> Vec<SentenceSpan<'_>> {
    if text.trim().is_empty() {
        return Vec::new();
    }

    let chars: Vec<char> = text.chars().collect();
    let char_len = chars.len();
    let mut sentences = Vec::new();
    let mut current_start = 0;
    let mut i = 0;

    while i < char_len {
        let c = chars[i];

        // 1. Dấu xuống dòng luôn là ranh giới câu tuyệt đối
        if c == '\n' || c == '\r' {
            if i > current_start {
                push_sentence_span(text, &chars, current_start, i, &mut sentences);
            }
            // Bỏ qua các ký tự xuống dòng liên tiếp
            while i < char_len && (chars[i] == '\n' || chars[i] == '\r') {
                i += 1;
            }
            current_start = i;
            continue;
        }

        // 2. Dấu chấm hỏi '?' và dấu chấm cảm '!'
        if c == '?' || c == '!' {
            // Nuốt hết các dấu chấm hỏi/cảm liên tiếp (ví dụ: "??", "!!", "?!")
            let mut end_p = i + 1;
            while end_p < char_len && (chars[end_p] == '?' || chars[end_p] == '!' || chars[end_p] == '.') {
                end_p += 1;
            }
            // Kèm theo ngoặc kép hoặc ngoặc đơn đóng nếu có (ví dụ: ?")
            while end_p < char_len && matches!(chars[end_p], '"' | '”' | ')' | ']' | '}') {
                end_p += 1;
            }

            push_sentence_span(text, &chars, current_start, end_p, &mut sentences);
            i = end_p;
            current_start = i;
            continue;
        }

        // 3. Dấu chấm phẩy ';' đóng vai trò ranh giới mệnh đề độc lập
        if c == ';' {
            let end_p = i + 1;
            push_sentence_span(text, &chars, current_start, end_p, &mut sentences);
            i = end_p;
            current_start = i;
            continue;
        }

        // 4. Dấu chấm '.' - Cần kiểm tra ngoại lệ chống đứt gãy câu
        if c == '.' {
            if is_exception_dot(&chars, i) {
                i += 1;
                continue;
            }

            // Nuốt hết dấu ba chấm "..."
            let mut end_p = i + 1;
            while end_p < char_len && chars[end_p] == '.' {
                end_p += 1;
            }
            // Kèm theo ngoặc kép hoặc ngoặc đơn đóng nếu có
            while end_p < char_len && matches!(chars[end_p], '"' | '”' | ')' | ']' | '}') {
                end_p += 1;
            }

            push_sentence_span(text, &chars, current_start, end_p, &mut sentences);
            i = end_p;
            current_start = i;
            continue;
        }

        i += 1;
    }

    // Câu cuối cùng nếu không kết thúc bằng dấu chấm
    if current_start < char_len {
        push_sentence_span(text, &chars, current_start, char_len, &mut sentences);
    }

    sentences
}

/// Thêm một câu vào danh sách, tự động bỏ qua khoảng trắng thừa ở hai đầu nhưng bảo toàn tọa độ tuyệt đối
fn push_sentence_span<'a>(
    full_text: &'a str,
    chars: &[char],
    raw_start: usize,
    raw_end: usize,
    out: &mut Vec<SentenceSpan<'a>>,
) {
    if raw_start >= raw_end || raw_start >= chars.len() {
        return;
    }

    let mut start = raw_start;
    let mut end = raw_end.min(chars.len());

    // Cắt bỏ khoảng trắng đầu câu
    while start < end && chars[start].is_whitespace() {
        start += 1;
    }
    // Cắt bỏ khoảng trắng cuối câu
    while end > start && chars[end - 1].is_whitespace() {
        end -= 1;
    }

    if start >= end {
        return;
    }

    // Tính toán byte slice tương ứng với [start..end] ký tự Unicode
    let byte_start = full_text
        .char_indices()
        .nth(start)
        .map(|(b, _)| b)
        .unwrap_or(0);
    let byte_end = full_text
        .char_indices()
        .nth(end)
        .map(|(b, _)| b)
        .unwrap_or(full_text.len());

    let sentence_text = &full_text[byte_start..byte_end];
    if !sentence_text.is_empty() {
        out.push(SentenceSpan {
            text: sentence_text,
            start_char: start,
            end_char: end,
        });
    }
}

/// Kiểm tra xem dấu '.' tại vị trí index có phải là ngoại lệ KHÔNG ngắt câu
fn is_exception_dot(chars: &[char], dot_idx: usize) -> bool {
    let len = chars.len();

    // 1. Số thập phân / phân cách hàng nghìn: "3.14", "10.000"
    if dot_idx > 0 && dot_idx + 1 < len {
        if chars[dot_idx - 1].is_ascii_digit() && chars[dot_idx + 1].is_ascii_digit() {
            return true;
        }
    }

    // 2. Tên miền hoặc URL: "google.com", "vnexpress.net"
    if dot_idx > 0 && dot_idx + 1 < len {
        let prev = chars[dot_idx - 1];
        let next = chars[dot_idx + 1];
        if (prev.is_alphanumeric() || prev == '/') && next.is_alphanumeric() {
            // Kiểm tra xem phía sau có phải là phần mở rộng tên miền (com, vn, net, org, io, gov, edu)
            let remaining: String = chars[dot_idx + 1..].iter().take(5).collect();
            let is_tld = remaining.starts_with("vn")
                || remaining.starts_with("com")
                || remaining.starts_with("net")
                || remaining.starts_with("org")
                || remaining.starts_with("edu")
                || remaining.starts_with("gov")
                || remaining.starts_with("io");
            if is_tld {
                return true;
            }
        }
    }

    // 3. Từ viết tắt học vị, cơ quan, địa danh phổ biến
    let lookback_start = dot_idx.saturating_sub(10);
    let before_str: String = chars[lookback_start..=dot_idx].iter().collect();
    let lower_before = before_str.to_lowercase();

    for &abbr in COMMON_ABBREVIATIONS {
        if lower_before.ends_with(abbr) {
            // Kiểm tra ranh giới từ phía trước
            let abbr_len = abbr.chars().count();
            if dot_idx + 1 >= abbr_len {
                let start_idx = dot_idx + 1 - abbr_len;
                if start_idx == 0 || chars[start_idx - 1].is_whitespace() || matches!(chars[start_idx - 1], '(' | '[' | '"' | '“') {
                    return true;
                }
            }
        }
    }

    // 4. Chữ cái đầu viết tắt họ tên đơn lẻ: "Nguyễn V. A"
    if dot_idx > 0 {
        let prev = chars[dot_idx - 1];
        if prev.is_uppercase() {
            let before_prev = if dot_idx >= 2 { chars[dot_idx - 2] } else { ' ' };
            if before_prev.is_whitespace() {
                // Nếu ký tự tiếp theo là chữ cái viết hoa (ví dụ: "V. A"), đây là viết tắt tên
                if dot_idx + 2 < len && chars[dot_idx + 1].is_whitespace() && chars[dot_idx + 2].is_uppercase() {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_sentences_basic() {
        let text = "Trong tình hình này đòi hỏi khắc phục. Ngày mai tôi sẽ đến!";
        let sentences = split_sentences(text);
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].text, "Trong tình hình này đòi hỏi khắc phục.");
        assert_eq!(sentences[1].text, "Ngày mai tôi sẽ đến!");
    }

    #[test]
    fn test_abbreviations_no_break() {
        let text = "GS. Cao Xuân Hạo sinh năm 1930. Ông sống tại TP.HCM.";
        let sentences = split_sentences(text);
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].text, "GS. Cao Xuân Hạo sinh năm 1930.");
        assert_eq!(sentences[1].text, "Ông sống tại TP.HCM.");
    }

    #[test]
    fn test_decimals_and_urls_no_break() {
        let text = "Tỷ lệ lạm phát là 3.14% năm nay. Truy cập vnexpress.net để biết thêm.";
        let sentences = split_sentences(text);
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].text, "Tỷ lệ lạm phát là 3.14% năm nay.");
        assert_eq!(sentences[1].text, "Truy cập vnexpress.net để biết thêm.");
    }

    #[test]
    fn test_newlines_always_break() {
        let text = "Câu một không chấm\nCâu hai trên dòng mới\n\nCâu ba sau hai dòng trống";
        let sentences = split_sentences(text);
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0].text, "Câu một không chấm");
        assert_eq!(sentences[1].text, "Câu hai trên dòng mới");
        assert_eq!(sentences[2].text, "Câu ba sau hai dòng trống");
    }

    #[test]
    fn test_quotes_trailing_punctuation() {
        let text = "John nói: \"Tôi sẽ về sau ba ngày.\" Sau đó anh ấy rời đi.";
        let sentences = split_sentences(text);
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0].text, "John nói: \"Tôi sẽ về sau ba ngày.\"");
        assert_eq!(sentences[1].text, "Sau đó anh ấy rời đi.");
    }
}
