//! # BỘ MẶT NẠ BẢO VỆ VĂN BẢN KỸ THUẬT & MARKDOWN (SPACE-PRESERVING MASKING SHIELD)
//!
//! Tự động che chắn code block, inline code, tên file, URL, CLI flag và bảng Markdown
//! bằng khoảng trắng có cùng độ dài ký tự để bảo toàn tuyệt đối tọa độ ký tự (Offsets).

use regex::Regex;
use std::sync::LazyLock;

// 1. Khối Code nhiều dòng (Code Fences: ``` ... ``` hoặc ~~~ ... ~~~)
static CODE_BLOCK_BACKTICK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)```[a-zA-Z0-9_-]*\n.*?```").unwrap()
});

static CODE_BLOCK_TILDE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)~~~[a-zA-Z0-9_-]*\n.*?~~~").unwrap()
});

// 2. Code nội dòng (Inline Code: `vwi check`, `py.allow_threads`)
static INLINE_CODE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"`[^`\n]+`").unwrap()
});

// 3. Đường dẫn Web, URIs & Emails
static URL_EMAIL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:https?|ftp|file)://[^\s)\]>]+|<(?:https?|ftp|file)://[^>]+>|\b[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}\b").unwrap()
});

// 4. Cú pháp Markdown Image & Link syntax: ![alt](url) và [alt](url)
static MARKDOWN_IMAGE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"!\[[^\]]*\]\([^)]*\)").unwrap()
});

static MARKDOWN_LINK_TARGET_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\]\([^)]+\)").unwrap()
});

// 5. Cú pháp Bảng Markdown (| Header | Col | hoặc |:---|---:|)
static MARKDOWN_TABLE_LINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*\|.*?\|\s*$").unwrap()
});

// 6. Định dạng Tên file, Package, Hàm, Version & Tham số CLI:
//    - Tên file/hàm: vwi.exe, v0.3.1.zip, py.allow_threads, draft-manifest.schema.json, target/debug/vwi
//    - Tham số dòng lệnh: -Uri, --fail-under, -f, -t, -i, -o, -v
static CLI_FLAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:\s|^)(--?[a-zA-Z0-9_-]+)").unwrap()
});

static TECH_FILENAME_EXT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b[a-zA-Z0-9_-]+\.(?:exe|zip|tar|gz|7z|rar|json|toml|yaml|yml|rs|py|js|ts|wasm|html|css|md|sh|lock|schema|svg|png|jpg|gif|pdf|bin|cfg|conf|env)\b").unwrap()
});

static TECH_METHOD_CALL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b[a-zA-Z_][a-zA-Z0-9_]*\.[a-zA-Z_][a-zA-Z0-9_]{2,}\b").unwrap()
});

static TECH_PATH_SLASH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b[a-zA-Z0-9_.-]+/[a-zA-Z0-9_./-]+\b").unwrap()
});

static TECH_VERSION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bv\d+\.\d+(?:\.\d+)*\b").unwrap()
});

// 7. Đơn vị đo lường kỹ thuật dạng slash: ops/s, req/s, km/h, bit/s, msg/s
static TECH_UNIT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b[a-zA-Z0-9]+/[a-zA-Z0-9]+\b").unwrap()
});

// 8. Thẻ HTML / XML / Markdown Comment: <div ...>, </span>, <!-- ... -->
static HTML_TAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)<!--.*?-->|<[a-zA-Z/][^>]*>").unwrap()
});

/// Tạo bản sao văn bản đã được che chắn bằng khoảng trắng có cùng độ dài ký tự Unicode.
/// Bảo toàn 100% số lượng ký tự và chỉ số tọa độ (character offsets).
pub fn create_masked_text_for_audit(raw_text: &str) -> String {
    let mut chars: Vec<char> = raw_text.chars().collect();
    let char_len = chars.len();

    // Hàm tiện ích: chuyển byte offset sang char offset
    let mask_regex_matches = |re: &Regex, char_buf: &mut Vec<char>| {
        let text_str: String = char_buf.iter().collect();
        for m in re.find_iter(&text_str) {
            let byte_start = m.start();
            let byte_end = m.end();

            // Tính toán char_start và char_end tương ứng
            let char_start = text_str[..byte_start].chars().count();
            let char_count = text_str[byte_start..byte_end].chars().count();
            let char_end = (char_start + char_count).min(char_len);

            for c in &mut char_buf[char_start..char_end] {
                if *c != '\n' && *c != '\r' {
                    if !c.is_whitespace() {
                        *c = 'x';
                    }
                }
            }
        }
    };

    // Áp dụng tuần tự các lớp mặt nạ
    mask_regex_matches(&CODE_BLOCK_BACKTICK_RE, &mut chars);
    mask_regex_matches(&CODE_BLOCK_TILDE_RE, &mut chars);
    mask_regex_matches(&HTML_TAG_RE, &mut chars);
    mask_regex_matches(&INLINE_CODE_RE, &mut chars);
    mask_regex_matches(&MARKDOWN_IMAGE_RE, &mut chars);
    mask_regex_matches(&MARKDOWN_LINK_TARGET_RE, &mut chars);
    mask_regex_matches(&URL_EMAIL_RE, &mut chars);
    mask_regex_matches(&MARKDOWN_TABLE_LINE_RE, &mut chars);
    mask_regex_matches(&CLI_FLAG_RE, &mut chars);
    mask_regex_matches(&TECH_FILENAME_EXT_RE, &mut chars);
    mask_regex_matches(&TECH_METHOD_CALL_RE, &mut chars);
    mask_regex_matches(&TECH_PATH_SLASH_RE, &mut chars);
    mask_regex_matches(&TECH_VERSION_RE, &mut chars);
    mask_regex_matches(&TECH_UNIT_RE, &mut chars);

    chars.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_masking_preserves_length() {
        let raw = "Dùng lệnh `vwi check` với file vwi.exe tại https://github.com nhé.";
        let masked = create_masked_text_for_audit(raw);
        assert_eq!(raw.chars().count(), masked.chars().count());
        assert!(!masked.contains("vwi check"));
        assert!(!masked.contains("vwi.exe"));
        assert!(!masked.contains("https://github.com"));
        assert!(masked.starts_with("Dùng lệnh "));
        assert!(masked.ends_with(" nhé."));
    }

    #[test]
    fn test_masking_code_block_and_tables() {
        let raw = "Bảng dữ liệu:\n| Cột 1 | Cột 2 |\n| :--- | :--- |\n| Giá trị 1 | 300 |\n\n```rust\nfn main() {}\n```\nKết luận ở đây.";
        let masked = create_masked_text_for_audit(raw);
        assert_eq!(raw.chars().count(), masked.chars().count());
        assert!(!masked.contains("fn main()"));
        assert!(!masked.contains("| Cột 1 |"));
        assert!(masked.contains("Kết luận ở đây."));
    }
}
