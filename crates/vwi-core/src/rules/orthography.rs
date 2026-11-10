//! Module kiểm tra lỗi chính tả cơ bản, dấu câu và quy chuẩn văn thư tiếng Việt
//! Căn cứ: Nghị định 30/2020/NĐ-CP (Công tác văn thư) & Thông tư 20/2018/TT-BGDĐT

use crate::rules::phonotactics::PhonotacticEngine;
use crate::types::Finding;
use crate::unicode_utils::normalize_nfc;

pub const PUNCTUATION_SPACE_BEFORE_RULE_ID: &str = "PUNCTUATION_SPACE_BEFORE";
pub const PUNCTUATION_NO_SPACE_AFTER_RULE_ID: &str = "PUNCTUATION_NO_SPACE_AFTER";
pub const BRACKET_SPACE_INSIDE_RULE_ID: &str = "BRACKET_SPACE_INSIDE";
pub const SENTENCE_CAPITALIZATION_RULE_ID: &str = "SENTENCE_CAPITALIZATION";
pub const DECREE30_AGENCY_CAPITALIZATION_RULE_ID: &str = "DECREE30_AGENCY_CAPITALIZATION";
pub const DECREE30_ADMIN_UNIT_LOWERCASE_RULE_ID: &str = "DECREE30_ADMIN_UNIT_LOWERCASE";
pub const ORTHOGRAPHY_TONE_INCONSISTENCY_RULE_ID: &str = "ORTHOGRAPHY_TONE_INCONSISTENCY";

pub struct OrthographyChecker;

impl OrthographyChecker {
    /// 1. TẦNG MẶT NẠ BẢO VỆ (TRUE MASKING SHIELD)
    /// Che chắn triệt để URLs, Emails, Markdown, Fenced Code, Inline Code
    pub fn compute_masking_shield(chars: &[char]) -> Vec<bool> {
        let len = chars.len();
        let mut mask = vec![false; len];
        let s: String = chars.iter().collect();

        // 1.1. Che chắn Fenced Code Blocks (``` ... ```)
        let mut in_fenced_code = false;
        let mut fence_start = 0;
        let mut i = 0;
        while i < len {
            if i + 2 < len && chars[i] == '`' && chars[i + 1] == '`' && chars[i + 2] == '`' {
                if !in_fenced_code {
                    in_fenced_code = true;
                    fence_start = i;
                    i += 3;
                    continue;
                } else {
                    in_fenced_code = false;
                    for k in fence_start..=i + 2 {
                        if k < len {
                            mask[k] = true;
                        }
                    }
                    i += 3;
                    continue;
                }
            }
            i += 1;
        }
        if in_fenced_code {
            for k in fence_start..len {
                mask[k] = true;
            }
        }

        // 1.2. Che chắn Inline Code (`...`)
        let mut in_inline_code = false;
        let mut inline_start = 0;
        let mut j = 0;
        while j < len {
            if !mask[j] && chars[j] == '`' {
                if !in_inline_code {
                    in_inline_code = true;
                    inline_start = j;
                } else {
                    in_inline_code = false;
                    for k in inline_start..=j {
                        mask[k] = true;
                    }
                }
            }
            j += 1;
        }

        // 1.3. Che chắn URLs và Web Links (http://, https://, ftp://, www.)
        let url_prefixes = ["http://", "https://", "ftp://", "www."];
        let mut byte_idx = 0;
        for (char_idx, &c) in chars.iter().enumerate() {
            if !mask[char_idx] {
                let rem_str = &s[byte_idx..];
                for prefix in &url_prefixes {
                    if rem_str.starts_with(prefix) {
                        let mut k = char_idx;
                        while k < len && !chars[k].is_whitespace() && !matches!(chars[k], '>' | '"' | '\'' | ')' | ']') {
                            mask[k] = true;
                            k += 1;
                        }
                        // Nếu URL kết thúc bằng dấu chấm và theo sau là khoảng trắng hoặc hết chuỗi:
                        // Dấu chấm đó đóng vai trò là dấu kết thúc câu của tác giả, không thuộc URL
                        if k > char_idx && chars[k - 1] == '.' {
                            mask[k - 1] = false;
                        }
                        break;
                    }
                }
            }
            byte_idx += c.len_utf8();
        }

        // 1.4. Che chắn Email Addresses (user@domain.ext)
        for (idx, &c) in chars.iter().enumerate() {
            if c == '@' && idx > 0 && idx + 1 < len && !mask[idx] {
                let mut start = idx;
                while start > 0 && (chars[start - 1].is_alphanumeric() || matches!(chars[start - 1], '.' | '_' | '-')) {
                    start -= 1;
                }
                let mut end = idx + 1;
                while end < len && (chars[end].is_alphanumeric() || matches!(chars[end], '.' | '_' | '-')) {
                    end += 1;
                }
                if start < idx && end > idx + 1 {
                    for k in start..end {
                        mask[k] = true;
                    }
                }
            }
        }

        // 1.5. Che chắn Markdown Link Targets [text](url)
        for idx in 0..len {
            if chars[idx] == ']' && idx + 1 < len && chars[idx + 1] == '(' && !mask[idx] {
                let mut k = idx + 2;
                while k < len && chars[k] != ')' && chars[k] != '\n' {
                    mask[k] = true;
                    k += 1;
                }
                if k < len && chars[k] == ')' {
                    mask[k] = true;
                }
            }
        }

        mask
    }

    /// Kiểm tra xem một vị trí dấu chấm có thuộc từ viết tắt chuẩn mực hay không:
    /// TP., TP.HCM, GS., PGS., TS., ThS., BS., KS., CN., Q., H., P., X., TX., TT., v.v., tr., Mr., Mrs., Dr.
    fn is_valid_abbreviation_dot(chars: &[char], dot_idx: usize) -> bool {
        let len = chars.len();
        if dot_idx >= len || chars[dot_idx] != '.' {
            return false;
        }

        let mut start = dot_idx;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '.') {
            start -= 1;
        }

        let mut end = dot_idx + 1;
        while end < len && (chars[end].is_alphanumeric() || chars[end] == '.') {
            end += 1;
        }

        let token: String = chars[start..end].iter().collect();
        let token_lower = token.to_lowercase();

        let valid_abbrevs = [
            "tp.", "tp.hcm", "gs.", "ts.", "gs.ts.", "pgs.", "pgs.ts.", "ths.", "bs.", "ks.", "cn.",
            "q.", "h.", "p.", "x.", "tx.", "tt.", "v.v.", "tr.", "đ/c",
            "mr.", "mrs.", "ms.", "dr.", "prof.", "ubnd", "hđnd", "tbt.", "ctn."
        ];

        for &abbrev in &valid_abbrevs {
            if token_lower == abbrev || token_lower.starts_with(abbrev) {
                return true;
            }
        }

        if token.chars().any(|c| c.is_ascii_uppercase()) {
            if end > dot_idx + 1 && (chars[dot_idx + 1].is_ascii_uppercase() || chars[dot_idx + 1].is_ascii_digit()) {
                return true;
            }
        }

        false
    }

    /// 2. KIỂM TRA LỖI DẤU CÂU VÀ KHOẢNG TRẮNG (PUNCTUATION SPACING)
    pub fn check_punctuation_spacing(raw_text: &str) -> Vec<Finding> {
        let norm = normalize_nfc(raw_text);
        let chars: Vec<char> = norm.chars().collect();
        let len = chars.len();
        let mask = Self::compute_masking_shield(&chars);
        let mut findings = Vec::new();

        for i in 0..len {
            if mask[i] {
                continue;
            }

            let c = chars[i];

            // 2.1. Lỗi thừa khoảng trắng TRƯỚC dấu câu: "chào , bạn" hoặc "sao ?"
            if matches!(c, ',' | '.' | ';' | ':' | '!' | '?') {
                if i > 0 && chars[i - 1] == ' ' {
                    // Ngoại lệ: Dấu ba chấm "..."
                    let is_ellipsis = c == '.'
                        && ((i + 1 < len && chars[i + 1] == '.')
                            || (i > 1 && chars[i - 2] == '.'));
                    if !is_ellipsis {
                        let matched_span: String = chars[i - 1..=i].iter().collect();
                        findings.push(
                            Finding::new(
                                PUNCTUATION_SPACE_BEFORE_RULE_ID,
                                "orthography",
                                "major",
                                i - 1,
                                i + 1,
                                0.98,
                                format!(
                                    "Thừa khoảng trắng trước dấu '{}'. Đúng chuẩn văn thư: viết sát vào chữ phía trước.",
                                    c
                                ),
                                "orthography",
                                matched_span,
                            )
                            .with_fix(i - 1, i + 1, c.to_string()),
                        );
                    }
                }

                // 2.2. Lỗi THIẾU khoảng trắng SAU dấu câu: "chào bạn.tôi là"
                if i + 1 < len {
                    let next_c = chars[i + 1];

                    if !next_c.is_whitespace()
                        && !matches!(
                            next_c,
                            ')' | ']' | '}' | '"' | '”' | '»' | '\'' | '’' | ',' | '.' | '!' | '?' | ';' | ':' | '/' | '-' | '…'
                        )
                    {
                        // Ngoại lệ 1: Số thập phân hoặc phân cách hàng nghìn (3.14 hoặc 10,5 hoặc 1,250,000)
                        let is_numeric = (c == '.' || c == ',')
                            && i > 0
                            && chars[i - 1].is_ascii_digit()
                            && next_c.is_ascii_digit();

                        // Ngoại lệ 2: Giờ giấc hoặc tỷ lệ / tỷ số (10:30, 2:1, 1:50000)
                        let is_time_or_ratio = c == ':'
                            && i > 0
                            && chars[i - 1].is_ascii_digit()
                            && next_c.is_ascii_digit();

                        // Ngoại lệ 3: Tên viết tắt chuẩn mực (TP.HCM, Q.1, GS., ThS., v.v.)
                        let is_abbrev = c == '.' && Self::is_valid_abbreviation_dot(&chars, i);

                        // Ngoại lệ 4: Dấu ba chấm "..."
                        let is_ellipsis = c == '.'
                            && (next_c == '.' || (i > 0 && chars[i - 1] == '.'));

                        if !is_numeric && !is_time_or_ratio && !is_abbrev && !is_ellipsis {
                            let matched_span: String = chars[i..=i + 1].iter().collect();
                            findings.push(
                                Finding::new(
                                    PUNCTUATION_NO_SPACE_AFTER_RULE_ID,
                                    "orthography",
                                    "major",
                                    i,
                                    i + 2,
                                    0.96,
                                    format!(
                                        "Thiếu khoảng trắng sau dấu '{}'. Cần cách một dấu cách trước từ tiếp theo.",
                                        c
                                    ),
                                    "orthography",
                                    matched_span,
                                )
                                .with_fix(i, i + 2, format!("{} {}", c, next_c)),
                            );
                        }
                    }
                }
            }

            // 2.3. Lỗi khoảng trắng thừa bên trong ngoặc đơn hoặc ngoặc kép: "( nội dung )" hoặc '" vùng thủ đô "'
            // Ngoặc đơn ( và ngoặc kép mở “:
            if (c == '(' || c == '“') && i + 1 < len && chars[i + 1] == ' ' {
                let matched_span: String = chars[i..=i + 1].iter().collect();
                findings.push(
                    Finding::new(
                        BRACKET_SPACE_INSIDE_RULE_ID,
                        "orthography",
                        "minor",
                        i,
                        i + 2,
                        0.95,
                        format!("Thừa khoảng trắng sau dấu mở '{}'. Ký tự đầu phải viết sát dấu.", c),
                        "orthography",
                        matched_span,
                    )
                    .with_fix(i, i + 2, c.to_string()),
                );
            }
            // Ngoặc đơn ) và ngoặc kép đóng ”:
            if (c == ')' || c == '”') && i > 0 && chars[i - 1] == ' ' {
                let matched_span: String = chars[i - 1..=i].iter().collect();
                findings.push(
                    Finding::new(
                        BRACKET_SPACE_INSIDE_RULE_ID,
                        "orthography",
                        "minor",
                        i - 1,
                        i + 1,
                        0.95,
                        format!("Thừa khoảng trắng trước dấu đóng '{}'. Chữ cuối phải viết sát dấu.", c),
                        "orthography",
                        matched_span,
                    )
                    .with_fix(i - 1, i + 1, c.to_string()),
                );
            }
            // Dấu ngoặc kép thẳng ": phân biệt mở hay đóng dựa trên ngữ cảnh trước/sau
            if c == '"' {
                let is_open_quote = i == 0 || chars[i - 1].is_whitespace() || matches!(chars[i - 1], '(' | '[');
                let is_close_quote = i + 1 == len || chars[i + 1].is_whitespace() || matches!(chars[i + 1], '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']');

                if is_open_quote && i + 1 < len && chars[i + 1] == ' ' {
                    let matched_span: String = chars[i..=i + 1].iter().collect();
                    findings.push(
                        Finding::new(
                            BRACKET_SPACE_INSIDE_RULE_ID,
                            "orthography",
                            "minor",
                            i,
                            i + 2,
                            0.95,
                            "Thừa khoảng trắng sau dấu mở ngoặc kép '\"'. Ký tự đầu phải viết sát dấu.",
                            "orthography",
                            matched_span,
                        )
                        .with_fix(i, i + 2, "\"".to_string()),
                    );
                }
                if is_close_quote && i > 0 && chars[i - 1] == ' ' {
                    let matched_span: String = chars[i - 1..=i].iter().collect();
                    findings.push(
                        Finding::new(
                            BRACKET_SPACE_INSIDE_RULE_ID,
                            "orthography",
                            "minor",
                            i - 1,
                            i + 1,
                            0.95,
                            "Thừa khoảng trắng trước dấu đóng ngoặc kép '\"'. Chữ cuối phải viết sát dấu.",
                            "orthography",
                            matched_span,
                        )
                        .with_fix(i - 1, i + 1, "\"".to_string()),
                    );
                }
            }
        }

        findings
    }

    /// 3. KIỂM TRA VIẾT HOA ĐẦU CÂU (SENTENCE CAPITALIZATION)
    pub fn check_sentence_capitalization(raw_text: &str) -> Vec<Finding> {
        let norm = normalize_nfc(raw_text);
        let chars: Vec<char> = norm.chars().collect();
        let len = chars.len();
        let mask = Self::compute_masking_shield(&chars);
        let mut findings = Vec::new();

        let mut require_cap = true;
        let mut idx = 0;

        while idx < len {
            if mask[idx] {
                idx += 1;
                continue;
            }

            let c = chars[idx];

            if matches!(c, '.' | '!' | '?') {
                if c == '.' && idx > 0 && chars[idx - 1].is_ascii_digit() && (idx + 1 < len && chars[idx + 1].is_ascii_digit()) {
                    idx += 1;
                    continue;
                }

                if c == '.' && Self::is_valid_abbreviation_dot(&chars, idx) {
                    idx += 1;
                    continue;
                }

                let mut next_idx = idx + 1;
                while next_idx < len && chars[next_idx] == '.' {
                    next_idx += 1;
                }

                require_cap = true;
                idx = next_idx;
                continue;
            }

            if c == ';' {
                require_cap = false;
                idx += 1;
                continue;
            }

            if c.is_whitespace() || matches!(c, '(' | '[' | '{' | '"' | '”' | '“' | '«' | '»' | '\'' | '’' | '`') {
                idx += 1;
                continue;
            }

            if require_cap && (c.is_ascii_lowercase() || c == 'đ') && idx + 1 < len && chars[idx + 1] == ')' {
                require_cap = false;
                idx += 2;
                continue;
            }

            // Bỏ qua chuỗi mask / placeholder kiểm toán: x, xx, xxx, xxxxxxxx...
            if (c == 'x' || c == 'X') && require_cap {
                let mut end_x = idx;
                while end_x < len && (chars[end_x] == 'x' || chars[end_x] == 'X') {
                    end_x += 1;
                }
                if end_x > idx && (end_x == len || !chars[end_x].is_alphabetic()) {
                    require_cap = false;
                    idx = end_x;
                    continue;
                }
            }

            if c.is_alphabetic() {
                if require_cap && c.is_lowercase() {
                    findings.push(
                        Finding::new(
                            SENTENCE_CAPITALIZATION_RULE_ID,
                            "orthography",
                            "major",
                            idx,
                            idx + 1,
                            0.95,
                            format!("Chữ cái đầu câu '{}' chưa được viết hoa.", c),
                            "orthography",
                            c.to_string(),
                        )
                        .with_fix(idx, idx + 1, c.to_uppercase().to_string()),
                    );
                }
                require_cap = false;
            } else if c.is_ascii_digit() {
                require_cap = false;
            }

            idx += 1;
        }

        findings
    }

    /// 4. QUY CHUẨN VIẾT HOA THEO NGHỊ ĐỊNH 30/2020/NĐ-CP (PHỤ LỤC II)
    /// Kiểm tra token-based cho tên cơ quan, tổ chức & địa danh hành chính kết hợp danh từ chung
    pub fn check_decree30_capitalization(raw_text: &str) -> Vec<Finding> {
        let norm = normalize_nfc(raw_text);
        let chars: Vec<char> = norm.chars().collect();
        let len = chars.len();
        let mask = Self::compute_masking_shield(&chars);
        let mut findings = Vec::new();
        let s: String = chars.iter().collect();

        // 4.1. Viết hoa tên cơ quan, tổ chức (Token & Prefix Mapping)
        // Căn cứ Phụ lục II, Nghị định 30: Viết hoa chữ cái đầu của từ, cụm từ chỉ loại hình cơ quan,
        // và chữ cái đầu của từ, cụm từ chỉ chức năng, lĩnh vực hoạt động.
        let agency_patterns: &[(&str, &str)] = &[
            ("Bộ Giáo Dục Và Đào Tạo", "Bộ Giáo dục và Đào tạo"),
            ("Bộ Giáo Dục và Đào Tạo", "Bộ Giáo dục và Đào tạo"),
            ("Sở Giáo Dục Và Đào Tạo", "Sở Giáo dục và Đào tạo"),
            ("Sở Giáo Dục và Đào Tạo", "Sở Giáo dục và Đào tạo"),
            ("Bộ Tài Chính", "Bộ Tài chính"),
            ("Sở Tài Chính", "Sở Tài chính"),
            ("Bộ Kế Hoạch Và Đầu Tư", "Bộ Kế hoạch và Đầu tư"),
            ("Bộ Kế Hoạch và Đầu Tư", "Bộ Kế hoạch và Đầu tư"),
            ("Bộ Nông Nghiệp Và Phát Triển Nông Thôn", "Bộ Nông nghiệp và Phát triển nông thôn"),
            ("Bộ Nông Nghiệp và Phát Triển Nông Thôn", "Bộ Nông nghiệp và Phát triển nông thôn"),
            ("Bộ Y Tế", "Bộ Y tế"),
            ("Sở Y Tế", "Sở Y tế"),
            ("Bộ Xây Dựng", "Bộ Xây dựng"),
            ("Sở Xây Dựng", "Sở Xây dựng"),
            ("Bộ Tư Pháp", "Bộ Tư pháp"),
            ("Sở Tư Pháp", "Sở Tư pháp"),
            ("Bộ Ngoại Giao", "Bộ Ngoại giao"),
            ("Bộ Nội Vụ", "Bộ Nội vụ"),
            ("Bộ Giao Thông Vận Tải", "Bộ Giao thông vận tải"),
            ("Bộ Khoa Học Và Công Nghệ", "Bộ Khoa học và Công nghệ"),
            ("Bộ Tài Nguyên Và Môi Trường", "Bộ Tài nguyên và Môi trường"),
            ("Bộ Thông Tin Và Truyền Thông", "Bộ Thông tin và Truyền thông"),
            ("Ủy Ban Nhân Dân", "Ủy ban nhân dân"),
            ("Uỷ Ban Nhân Dân", "Ủy ban nhân dân"),
            ("Hội Đồng Nhân Dân", "Hội đồng nhân dân"),
            ("Mặt Trận Tổ Quốc", "Mặt trận Tổ quốc"),
            ("Bảo Hiểm Xã Hội", "Bảo hiểm xã hội"),
            ("Viện Kiểm Sát Nhân Dân", "Viện kiểm sát nhân dân"),
            ("Tòa Án Nhân Dân", "Tòa án nhân dân"),
        ];

        for &(bad, good) in agency_patterns {
            let mut search_from = 0;
            while let Some(rel_byte) = s[search_from..].find(bad) {
                let match_byte = search_from + rel_byte;
                let char_start = s[..match_byte].chars().count();
                let char_count = bad.chars().count();
                let char_end = char_start + char_count;

                if char_end <= len && !mask[char_start] {
                    let slice: String = chars[char_start..char_end].iter().collect();
                    findings.push(
                        Finding::new(
                            DECREE30_AGENCY_CAPITALIZATION_RULE_ID,
                            "orthography",
                            "minor",
                            char_start,
                            char_end,
                            0.96,
                            format!(
                                "Tên cơ quan, tổ chức '{}' viết hoa chưa chuẩn theo Phụ lục II, Nghị định 30/2020/NĐ-CP.",
                                bad
                            ),
                            "orthography",
                            slice,
                        )
                        .with_fix(char_start, char_end, good.to_string()),
                    );
                }
                search_from = match_byte + bad.len();
            }
        }

        // 4.2. Viết hoa địa danh hành chính kết hợp danh từ chung giữa câu:
        // Ví dụ: "...gửi UBND Tỉnh Tuyên Quang..." -> Tỉnh viết thường ("tỉnh Tuyên Quang")
        // NHƯNG nếu đứng đầu câu ("Tỉnh Tuyên Quang đã ban hành...") thì HỢP LỆ!
        let admin_units = [
            ("Tỉnh ", "tỉnh "),
            ("Thành Phố ", "thành phố "),
            ("Thành phố ", "thành phố "),
            ("Quận ", "quận "),
            ("Huyện ", "huyện "),
            ("Thị Xã ", "thị xã "),
            ("Phường ", "phường "),
            ("Xã ", "xã "),
        ];

        for &(bad_unit, good_unit) in &admin_units {
            let mut search_from = 0;
            while let Some(rel_byte) = s[search_from..].find(bad_unit) {
                let match_byte = search_from + rel_byte;
                let char_pos = s[..match_byte].chars().count();
                let unit_len = bad_unit.chars().count();

                if char_pos + unit_len <= len && !mask[char_pos] {
                    // Kiểm tra xem vị trí này CÓ PHẢI LÀ ĐẦU CÂU KHÔNG
                    let is_start_of_sentence = is_char_pos_start_of_sentence(&chars, char_pos);

                    if !is_start_of_sentence {
                        // Kiểm tra xem theo sau có phải là Tên riêng viết hoa không (ví dụ: Tuyên Quang, Ba Đình...)
                        let next_pos = char_pos + unit_len;
                        if next_pos < len && chars[next_pos].is_uppercase() {
                            let matched: String = chars[char_pos..char_pos + unit_len].iter().collect();
                            findings.push(
                                Finding::new(
                                    DECREE30_ADMIN_UNIT_LOWERCASE_RULE_ID,
                                    "orthography",
                                    "minor",
                                    char_pos,
                                    char_pos + unit_len,
                                    0.94,
                                    "Theo Nghị định 30/2020/NĐ-CP, từ chỉ đơn vị hành chính viết thường khi đi liền với tên riêng ở giữa câu.",
                                    "orthography",
                                    matched,
                                )
                                .with_fix(char_pos, char_pos + unit_len, good_unit.to_string()),
                            );
                        }
                    }
                }

                search_from = match_byte + bad_unit.len();
            }
        }

        findings
    }

    /// 5. CẢM BIẾN NHẤT QUÁN DẤU THANH KIỂU MỚI VS. KIỂU CŨ (TONE INCONSISTENCY SENSOR)
    /// Căn cứ: Thông tư 20/2018/TT-BGDĐT
    pub fn check_tone_consistency(raw_text: &str) -> Vec<Finding> {
        let norm = normalize_nfc(raw_text);
        let chars: Vec<char> = norm.chars().collect();
        let len = chars.len();
        let mask = Self::compute_masking_shield(&chars);
        let mut findings = Vec::new();

        let pairs: &[(&str, &str)] = &[
            ("hoà", "hòa"),
            ("toà", "tòa"),
            ("thuỷ", "thủy"),
            ("khoẻ", "khỏe"),
            ("hoè", "hòe"),
            ("hoạ", "họa"),
            ("xoá", "xóa"),
            ("ngoạ", "ngọa"),
            ("thoả", "thỏa"),
            ("thuý", "thúy"),
            ("huỷ", "hủy"),
            ("luỹ", "lũy"),
            ("khoá", "khóa"),
            ("loá", "lóa"),
            ("choá", "chóa"),
            ("toá", "tóa"),
            ("doạ", "dọa"),
            ("nguỵ", "ngụy"),
            ("tuỳ", "tùy"),
            ("thuỳ", "thùy"),
            ("uỷ", "ủy"),
        ];

        let mut old_occurrences: Vec<(usize, usize, String, String)> = Vec::new();
        let mut has_new_style = false;

        let mut i = 0;
        while i < len {
            if mask[i] || !chars[i].is_alphabetic() {
                i += 1;
                continue;
            }

            let start = i;
            while i < len && !mask[i] && chars[i].is_alphabetic() {
                i += 1;
            }
            let end = i;

            let word: String = chars[start..end].iter().collect();
            let word_lower = word.to_lowercase();

            for &(old, new) in pairs {
                if word_lower == old {
                    let sugg = if word.chars().next().map_or(false, |c| c.is_uppercase()) {
                        let mut nc = new.chars();
                        nc.next().unwrap().to_uppercase().to_string() + nc.as_str()
                    } else {
                        new.to_string()
                    };
                    old_occurrences.push((start, end, word.clone(), sugg));
                }
                if word_lower == new {
                    has_new_style = true;
                }
            }
        }

        if !old_occurrences.is_empty() && has_new_style {
            for (start, end, matched, sugg) in old_occurrences {
                findings.push(
                    Finding::new(
                        ORTHOGRAPHY_TONE_INCONSISTENCY_RULE_ID,
                        "orthography",
                        "minor",
                        start,
                        end,
                        0.93,
                        "Bất nhất quán dấu thanh: Văn bản đang dùng lẫn lộn cả dấu thanh kiểu cũ ('hoà, thuỷ') và kiểu mới ('hòa, thủy'). Khuyến nghị quy chuẩn theo Thông tư 20/2018/TT-BGDĐT.",
                        "orthography",
                        matched,
                    )
                    .with_fix(start, end, sugg),
                );
            }
        }

        findings
    }
}

/// Kiểm tra xem một vị trí ký tự có phải là bắt đầu câu mới hay không
fn is_char_pos_start_of_sentence(chars: &[char], pos: usize) -> bool {
    if pos == 0 {
        return true;
    }
    let mut i = pos;
    while i > 0 {
        i -= 1;
        let c = chars[i];
        if c == '\n' || c == '.' || c == '!' || c == '?' {
            return true;
        }
        if !c.is_whitespace() && !matches!(c, '"' | '”' | '“' | '«' | '»' | '\'' | '(' | '[') {
            return false;
        }
    }
    true
}

/// Điểm gắn kết duy nhất (Public Evaluator) được gọi từ `engine.rs` và `auditor.rs`
pub fn evaluate_orthography(text: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let norm = normalize_nfc(text);
    let chars: Vec<char> = norm.chars().collect();
    let mask = OrthographyChecker::compute_masking_shield(&chars);

    // 1. Dấu câu & khoảng trắng
    findings.extend(OrthographyChecker::check_punctuation_spacing(text));

    // 2. Viết hoa đầu câu
    findings.extend(OrthographyChecker::check_sentence_capitalization(text));

    // 3. Cấu trúc âm tiết & Telex tự động (Phonotactic Engine)
    findings.extend(PhonotacticEngine::check_telex_errors(text, &mask));

    // 4. Quy chuẩn Nghị định 30 (tên cơ quan & địa danh hành chính)
    findings.extend(OrthographyChecker::check_decree30_capitalization(text));

    // 5. Cảm biến nhất quán dấu thanh (Thông tư 20)
    findings.extend(OrthographyChecker::check_tone_consistency(text));

    // Sắp xếp tọa độ tăng dần
    findings.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| a.end.cmp(&b.end)));
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orthography_spacing_before_punct() {
        let f = OrthographyChecker::check_punctuation_spacing("Hôm nay trời đẹp quá , tôi đi dạo .");
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].pattern_id, PUNCTUATION_SPACE_BEFORE_RULE_ID);
        assert_eq!(f[0].suggestion, Some(",".to_string()));
    }

    #[test]
    fn test_orthography_spacing_after_punct() {
        let f = OrthographyChecker::check_punctuation_spacing("Hôm nay trời đẹp.tôi muốn đi chơi.");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].pattern_id, PUNCTUATION_NO_SPACE_AFTER_RULE_ID);
        assert_eq!(f[0].suggestion, Some(". t".to_string()));
    }

    #[test]
    fn test_orthography_bracket_space() {
        let f = OrthographyChecker::check_punctuation_spacing("Đi chơi ( công viên ) rất vui.");
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].pattern_id, BRACKET_SPACE_INSIDE_RULE_ID);
        assert_eq!(f[0].suggestion, Some("(".to_string()));
        assert_eq!(f[1].suggestion, Some(")".to_string()));
    }

    #[test]
    fn test_orthography_capitalization() {
        let f = OrthographyChecker::check_sentence_capitalization("hôm nay trời đẹp. ngày mai đi làm.");
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].pattern_id, SENTENCE_CAPITALIZATION_RULE_ID);
        assert_eq!(f[0].suggestion, Some("H".to_string()));
        assert_eq!(f[1].suggestion, Some("N".to_string()));
    }

    #[test]
    fn test_orthography_abbreviations_and_math_exceptions() {
        let text = "Theo GS. Cao Xuân Hạo tại TP.HCM vào lúc 10:30, tỷ số trận đấu là 2:1. Đọc tr. 45 để biết thêm 1,250,000 VNĐ và 3.14 v.v. để hiểu rõ.";
        let findings = evaluate_orthography(text);
        assert!(findings.is_empty(), "Không được bắt nhầm các từ viết tắt, số học và giờ giấc: {:?}", findings);
    }

    #[test]
    fn test_decree30_capitalization_agency_and_admin_unit() {
        let text = "Văn bản gửi Bộ Giáo Dục Và Đào Tạo tại Tỉnh Tuyên Quang. Bộ Tài Chính đã thông qua.";
        let f = OrthographyChecker::check_decree30_capitalization(text);
        assert!(f.len() >= 3);
        assert!(f.iter().any(|item| item.pattern_id == DECREE30_AGENCY_CAPITALIZATION_RULE_ID && item.suggestion == Some("Bộ Giáo dục và Đào tạo".to_string())));
        assert!(f.iter().any(|item| item.pattern_id == DECREE30_AGENCY_CAPITALIZATION_RULE_ID && item.suggestion == Some("Bộ Tài chính".to_string())));
        assert!(f.iter().any(|item| item.pattern_id == DECREE30_ADMIN_UNIT_LOWERCASE_RULE_ID && item.suggestion == Some("tỉnh ".to_string())));
    }

    #[test]
    fn test_decree30_admin_unit_start_of_sentence_is_valid() {
        let text = "Tỉnh Tuyên Quang đã ban hành công văn mới.";
        let f = OrthographyChecker::check_decree30_capitalization(text);
        assert!(f.is_empty(), "Tỉnh đứng đầu câu bắt buộc viết hoa, không được bắt lỗi!");
    }

    #[test]
    fn test_tone_consistency_sensor() {
        let mixed_text = "Chính sách hoà giải mang lại môi trường hòa bình bền vững.";
        let f = OrthographyChecker::check_tone_consistency(mixed_text);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].pattern_id, ORTHOGRAPHY_TONE_INCONSISTENCY_RULE_ID);
        assert_eq!(f[0].matched_span, "hoà");
        assert_eq!(f[0].suggestion, Some("hòa".to_string()));
    }

    #[test]
    fn test_masking_shield_code_and_markdown() {
        let text = "Đoạn mã: ```let x = 3.14; if (x > 0) { return; }``` và hàm `fn main() {}` không được báo lỗi.";
        let findings = evaluate_orthography(text);
        assert!(findings.is_empty(), "Mặt nạ bảo vệ phải che chắn mã nguồn: {:?}", findings);
    }

    #[test]
    fn test_death_sentence_matrix() {
        // Bài test tử thần của người dùng:
        let death_text = "Bộ Giáo Dục Và Đào Tạo đã có công văn gửi UBND Tỉnh Tuyên Quang. Trong cuộc họp bàn về kế hoạch hoà giải, chúng tôi đã thống nhất hòa bình với các bên. Tuy nhiên, kế hoạch này vẫn chưa được dduowjc phê duyệt.";
        let findings = evaluate_orthography(death_text);

        // 1. Phải bắt Bộ Giáo Dục Và Đào Tạo
        assert!(findings.iter().any(|f| f.pattern_id == DECREE30_AGENCY_CAPITALIZATION_RULE_ID && f.matched_span == "Bộ Giáo Dục Và Đào Tạo"));
        // 2. Phải bắt Tỉnh Tuyên Quang giữa câu
        assert!(findings.iter().any(|f| f.pattern_id == DECREE30_ADMIN_UNIT_LOWERCASE_RULE_ID && f.matched_span == "Tỉnh "));
        // 3. Phải bắt không nhất quán hoà giải và hòa bình
        assert!(findings.iter().any(|f| f.pattern_id == ORTHOGRAPHY_TONE_INCONSISTENCY_RULE_ID && f.matched_span == "hoà"));
        // 4. Phải bắt lỗi Telex dduowjc và đề xuất được
        assert!(findings.iter().any(|f| f.pattern_id == "TELEX_TYPO_ERROR" && f.matched_span == "dduowjc" && f.suggestion == Some("được".to_string())));
    }
}
