//! # WHITELIST CÁC CỤM TỪ ĐẶC BIỆT (TRAP WHITELIST)
//!
//! Bảo vệ các cụm từ hợp lệ nhưng "trông giống lỗi" khỏi bị các rule hình thái
//! (real-word, hành chính) bắt nhầm. Cơ chế: thay cụm từ bằng khoảng trắng có
//! cùng số ký tự (space-preserving) để bảo toàn tuyệt đối offset ký tự.
//!
//! Nguyên tắc: chỉ dùng cho các scanner mới (real-word, admin-format). Không
//! thay thế mặt nạ kỹ thuật toàn cục trong `masking.rs`.

use regex::Regex;
use std::sync::LazyLock;

/// Các cụm từ được miễn trừ.
///
/// - Quốc hiệu: chấp nhận cả hai biến thể dấu "HOÀ" và "HÒA", nhưng chỉ khi
///   phần "HỘI" đã đúng chính tả (câu sai "HÔI" vẫn phải bị bắt).
/// - "các cơ quan, tổ chức nhà nước": "nhà nước" ở đây là định ngữ, viết thường.
/// - "tỉ lệ" / "tỷ lệ": chấp nhận cả hai biến thể.
/// - "ký/kí/kỳ/kì": chấp nhận cả hai biến thể chính tả.
static TRAP_PHRASES_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)(?:",
        r"CỘNG\s+H(?:OÀ|ÒA)\s+XÃ\s+HỘI\s+CHỦ\s+NGHĨA\s+VIỆT\s+NAM",
        r"|các\s+cơ\s+quan,\s*tổ\s+chức\s+nhà\s+nước",
        r"|tỉ\s+lệ",
        r"|tỷ\s+lệ",
        r"|\b(?:ký|kí|kỳ|kì)\b",
        r")"
    ))
    .expect("Không thể biên dịch Trap Whitelist regex")
});

/// Các cụm từ rời (không phải câu hoàn chỉnh) được chấp nhận nguyên trạng.
///
/// Đây là các đoạn trích hợp lệ: cụm danh từ, danh sách biến thể chính tả,
/// hoặc dòng ngày tháng. Không áp quy tắc "viết hoa đầu câu" cho chúng vì
/// chúng không phải là câu.
static ACCEPTED_FRAGMENTS: &[&str] = &[
    "các cơ quan, tổ chức nhà nước",
    "tỉ lệ và tỷ lệ",
    "ký và kí, kỳ và kì",
    "ngày 05 tháng 3 năm 2020",
];

/// Chuẩn hóa khoảng trắng và chữ thường để so khớp cụm rời.
pub fn is_accepted_fragment(text: &str) -> bool {
    let normalized = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    ACCEPTED_FRAGMENTS
        .iter()
        .any(|fragment| normalized == *fragment)
}

/// Trả về bản sao văn bản đã che các cụm từ miễn trừ bằng khoảng trắng.
///
/// Số lượng ký tự (Unicode scalar values) được giữ nguyên 1:1, do đó mọi
/// offset ký tự tính trên kết quả đều dùng trực tiếp được cho văn bản gốc.
pub fn mask_trap_phrases(text: &str) -> String {
    let mut masked = text.to_string();
    let matches: Vec<(usize, usize)> = TRAP_PHRASES_RE
        .find_iter(text)
        .map(|m| (m.start(), m.end()))
        .collect();

    // Duyệt ngược để không làm lệch byte offset của các match phía trước.
    for (start, end) in matches.into_iter().rev() {
        let blank: String = text[start..end].chars().map(|_| ' ').collect();
        masked.replace_range(start..end, &blank);
    }

    masked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_masks_quoc_hieu_variants() {
        for text in [
            "CỘNG HOÀ XÃ HỘI CHỦ NGHĨA VIỆT NAM",
            "CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM",
            "cộng hòa xã hội chủ nghĩa việt nam",
        ] {
            let masked = mask_trap_phrases(text);
            assert_eq!(masked.chars().count(), text.chars().count());
            assert!(
                !masked.to_lowercase().contains("hội"),
                "input={:?} masked={:?}",
                text,
                masked
            );
        }
    }

    #[test]
    fn test_does_not_mask_misspelled_quoc_hieu() {
        let text = "CỘNG HÒA XÃ HÔI CHỦ NGHĨA VIỆT NAM";
        assert_eq!(mask_trap_phrases(text), text);
    }

    #[test]
    fn test_masks_standalone_variants_only() {
        let masked = mask_trap_phrases("ký và kí, kỳ và kì");
        assert!(!masked.contains("ký"));
        // Không che oan từ chứa chuỗi con (ví dụ "kỳthi" không phải từ độc lập).
        assert_eq!(mask_trap_phrases("kỳthi"), "kỳthi");
    }
}
