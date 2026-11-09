/// Bộ gán nhãn từ loại tiếng Việt theo ngữ pháp chức năng (GS. Cao Xuân Hạo).
/// Tra cứu tức thì O(1) không cấp phát heap.
pub fn tag_pos(word: &str) -> &'static str {
    let lower = word.to_lowercase();
    match lower.as_str() {
        "trong" | "ngoài" | "bằng" | "với" | "qua" | "tại" | "sau" | "trước" | "từ" | "đến"
        | "theo" | "vào" => "E",

        "tuy" | "dù" | "mặc dù" | "mặc dầu" | "nhưng" | "mà" | "thì" | "và" | "hoặc"
        | "bởi vì" | "rằng" | "là" | "nên" | "cho nên" | "do đó" | "nếu" | "hễ" | "giá như" => "C",

        "việc" | "sự" | "cuộc" | "cú" | "quá trình" | "chiếc" | "căn" | "ngọn" | "bức"
        | "tấm" => "Nc",

        "anh trai" | "chị gái" | "em gái" | "em trai" | "anh" | "chị" | "em" | "cha" | "mẹ"
        | "bố" | "con" | "anh em" => "Kin",

        "đã" | "đang" | "sẽ" | "đã và đang" | "vừa" | "mới" | "từng" | "sắp" => "Asp",

        "không" | "chưa" | "chẳng" | "cũng" | "đều" | "rất" | "quá" | "lắm" | "nhất" | "tất"
        | "cả" | "hãy" | "chớ" | "đừng" => "R",

        "tôi" | "ta" | "chúng ta" | "chúng tôi" | "họ" | "anh ấy" | "cô ấy" | "ông ấy"
        | "bà ta" | "nó" | "mình" | "gì" | "ai" | "đâu" | "nào" | "sao" => "P",

        "đòi hỏi" | "chứng minh" | "mua" | "bán" | "sửa" | "nỗ lực" | "tối ưu"
        | "tối ưu hóa" | "xem xét" | "lau dọn" | "khởi động" | "chối" | "nhận" | "ngã"
        | "gãy" | "thành" | "làm" | "tạo" | "tăng trưởng" | "thủng" | "quét" | "phủ nhận"
        | "nói" | "biết" | "cho là" | "cho rằng" | "phải nói là" => "V",

        _ => {
            if !word.is_empty() && word.chars().all(|c| c.is_ascii_digit()) {
                "M"
            } else if word.chars().count() == 1
                && !word.chars().next().unwrap().is_alphanumeric()
            {
                "CH"
            } else {
                "N"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_tagging_lookup() {
        assert_eq!(tag_pos("Anh trai"), "Kin");
        assert_eq!(tag_pos("tôi"), "P");
        assert_eq!(tag_pos("tuy"), "C");
        assert_eq!(tag_pos("nhưng"), "C");
        assert_eq!(tag_pos("sửa"), "V");
        assert_eq!(tag_pos("rất"), "R");
        assert_eq!(tag_pos("2026"), "M");
        assert_eq!(tag_pos("."), "CH");
    }
}
