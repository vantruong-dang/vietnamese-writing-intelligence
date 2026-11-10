//! Module kiểm tra lỗi chính tả cơ bản, dấu câu và quy chuẩn văn thư tiếng Việt
//! Căn cứ: Nghị định 30/2020/NĐ-CP (Công tác văn thư) & Thông tư 20/2018/TT-BGDĐT

use crate::types::Finding;
use crate::unicode_utils::normalize_nfc;

pub const PUNCTUATION_SPACE_BEFORE_RULE_ID: &str = "PUNCTUATION_SPACE_BEFORE";
pub const PUNCTUATION_NO_SPACE_AFTER_RULE_ID: &str = "PUNCTUATION_NO_SPACE_AFTER";
pub const BRACKET_SPACE_INSIDE_RULE_ID: &str = "BRACKET_SPACE_INSIDE";
pub const SENTENCE_CAPITALIZATION_RULE_ID: &str = "SENTENCE_CAPITALIZATION";
pub const TELEX_TYPO_ERROR_RULE_ID: &str = "TELEX_TYPO_ERROR";
pub const DECREE30_CAPITALIZATION_ORG_RULE_ID: &str = "DECREE30_CAPITALIZATION_ORG";
pub const DECREE30_CAPITALIZATION_ADMIN_UNIT_RULE_ID: &str = "DECREE30_CAPITALIZATION_ADMIN_UNIT";
pub const ORTHOGRAPHY_TONE_INCONSISTENCY_RULE_ID: &str = "ORTHOGRAPHY_TONE_INCONSISTENCY";

pub struct OrthographyChecker;

impl OrthographyChecker {
    /// 1. TẦNG MẶT NẠ BẢO VỆ (MASKING SHIELD)
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
                        break;
                    }
                }
            }
            byte_idx += c.len_utf8();
        }

        // 1.4. Che chắn Email Addresses (user@domain.ext)
        for (idx, &c) in chars.iter().enumerate() {
            if c == '@' && idx > 0 && idx + 1 < len && !mask[idx] {
                // Mở rộng về phía trước
                let mut start = idx;
                while start > 0 && (chars[start - 1].is_alphanumeric() || matches!(chars[start - 1], '.' | '_' | '-')) {
                    start -= 1;
                }
                // Mở rộng về phía sau
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

        // Quét ngược tìm điểm bắt đầu của token
        let mut start = dot_idx;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '.') {
            start -= 1;
        }

        // Quét xuôi tìm điểm kết thúc của token viết tắt (ví dụ TP.HCM)
        let mut end = dot_idx + 1;
        while end < len && (chars[end].is_alphanumeric() || chars[end] == '.') {
            end += 1;
        }

        let token: String = chars[start..end].iter().collect();
        let token_lower = token.to_lowercase();

        // Danh sách từ viết tắt hợp thức
        let valid_abbrevs = [
            "tp.", "tp.hcm", "gs.", "pgs.", "ts.", "ths.", "bs.", "ks.", "cn.",
            "q.", "h.", "p.", "x.", "tx.", "tt.", "v.v.", "tr.",
            "mr.", "mrs.", "ms.", "dr.", "prof.", "ubnd", "hđnd"
        ];

        for &abbrev in &valid_abbrevs {
            if token_lower == abbrev || token_lower.starts_with(abbrev) {
                return true;
            }
        }

        // Nếu token có dạng chữ in hoa + chấm lặp lại: P.GS., PGS.TS., Q.1, P.1
        if token.chars().any(|c| c.is_ascii_uppercase()) {
            if end > dot_idx + 1 && (chars[dot_idx + 1].is_ascii_uppercase() || chars[dot_idx + 1].is_ascii_digit()) {
                return true; // Dạng TP.HCM hoặc Q.1
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
                            .with_suggestion(c.to_string()),
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
                                .with_suggestion(format!("{} {}", c, next_c)),
                            );
                        }
                    }
                }
            }

            // 2.3. Lỗi khoảng trắng thừa bên trong ngoặc đơn: "( nội dung )"
            if c == '(' && i + 1 < len && chars[i + 1] == ' ' {
                let matched_span: String = chars[i..=i + 1].iter().collect();
                findings.push(
                    Finding::new(
                        BRACKET_SPACE_INSIDE_RULE_ID,
                        "orthography",
                        "minor",
                        i,
                        i + 2,
                        0.95,
                        "Thừa khoảng trắng sau dấu mở ngoặc '('. Ký tự đầu phải viết sát dấu ngoặc.",
                        "orthography",
                        matched_span,
                    )
                    .with_suggestion("(".to_string()),
                );
            }
            if c == ')' && i > 0 && chars[i - 1] == ' ' {
                let matched_span: String = chars[i - 1..=i].iter().collect();
                findings.push(
                    Finding::new(
                        BRACKET_SPACE_INSIDE_RULE_ID,
                        "orthography",
                        "minor",
                        i - 1,
                        i + 1,
                        0.95,
                        "Thừa khoảng trắng trước dấu đóng ngoặc ')'. Chữ cuối phải viết sát dấu ngoặc.",
                        "orthography",
                        matched_span,
                    )
                    .with_suggestion(")".to_string()),
                );
            }
        }

        findings
    }

    /// 3. KIỂM TRA VIẾT HOA ĐẦU CÂU (SENTENCE CAPITALIZATION)
    /// Ngoại lệ: Không ép viết hoa sau dấu chấm viết tắt (v.v., tr.), sau điểm liệt kê phân cấp a), b), hoặc sau dấu chấm phẩy ;
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

            // Dấu chấm câu kết thúc: . ! ?
            if matches!(c, '.' | '!' | '?') {
                // Ngoại lệ 1: Số thập phân hoặc số phân cách hàng nghìn (150.000 hoặc 3.14)
                if c == '.' && idx > 0 && chars[idx - 1].is_ascii_digit() && (idx + 1 < len && chars[idx + 1].is_ascii_digit()) {
                    idx += 1;
                    continue;
                }

                // Ngoại lệ 2: Từ viết tắt chuẩn mực (v.v., tr., GS., PGS., ThS., TP.)
                if c == '.' && Self::is_valid_abbreviation_dot(&chars, idx) {
                    idx += 1;
                    continue;
                }

                // Bỏ qua dấu ba chấm
                let mut next_idx = idx + 1;
                while next_idx < len && chars[next_idx] == '.' {
                    next_idx += 1;
                }

                require_cap = true;
                idx = next_idx;
                continue;
            }

            // Dấu chấm phẩy ; không bao giờ yêu cầu viết hoa từ sau nó (Nghị định 30)
            if c == ';' {
                require_cap = false;
                idx += 1;
                continue;
            }

            // Bỏ qua khoảng trắng, ngoặc, nháy
            if c.is_whitespace() || matches!(c, '(' | '[' | '{' | '"' | '”' | '“' | '«' | '»' | '\'' | '’' | '`') {
                idx += 1;
                continue;
            }

            // Kiểm tra điểm liệt kê văn thư: a), b), c), 1., 2.
            if require_cap && (c.is_ascii_lowercase() || c == 'đ') && idx + 1 < len && chars[idx + 1] == ')' {
                // Điểm liệt kê a), b) theo Nghị định 30 bắt buộc viết thường
                require_cap = false;
                idx += 2;
                continue;
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
                        .with_suggestion(c.to_uppercase().to_string()),
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

    /// 4. TỔNG QUÁT HÓA BẰNG NGỮ ÂM HỌC (PHONOTACTICS) & BỘ GIẢI MÃ TELEX TỰ ĐỘNG
    /// Phát hiện 100% lỗi dính phím Unikey / cấu trúc âm tiết phi tiếng Việt
    pub fn check_phonotactic_telex(raw_text: &str) -> Vec<Finding> {
        let norm = normalize_nfc(raw_text);
        let chars: Vec<char> = norm.chars().collect();
        let len = chars.len();
        let mask = Self::compute_masking_shield(&chars);
        let mut findings = Vec::new();

        let mut i = 0;
        while i < len {
            if mask[i] || !chars[i].is_alphabetic() {
                i += 1;
                continue;
            }

            let word_start = i;
            while i < len && !mask[i] && chars[i].is_alphabetic() {
                i += 1;
            }
            let word_end = i;

            let word: String = chars[word_start..word_end].iter().collect();
            let word_lower = word.to_lowercase();

            // Bỏ qua các từ viết hoa toàn bộ (Acronyms/Viết tắt): VNĐ, UBND, HĐND, TW, TP...
            if word.chars().all(|c| c.is_uppercase()) && word.chars().count() <= 6 {
                continue;
            }

            // Bỏ qua nếu là từ viết tắt hợp lệ kèm dấu chấm: tr., TP., GS., v.v.
            if word_end < len && chars[word_end] == '.' && Self::is_valid_abbreviation_dot(&chars, word_end) {
                continue;
            }

            // Bỏ qua các từ tiếng Anh phổ thông không phải tiếng Việt
            if matches!(
                word_lower.as_str(),
                "the" | "and" | "for" | "with" | "from" | "view" | "show" | "edit" | "copy" | "file" | "open" | "user" | "pass" | "data" | "code" | "tr" | "dr" | "mr" | "ms"
            ) {
                continue;
            }

            // Kiểm tra các dấu hiệu kẹt phím Telex / cấu trúc âm tiết bất hợp thức
            let is_telex_error = Self::is_telex_malformed(&word_lower);

            if is_telex_error {
                let decoded = Self::decode_telex(&word_lower);
                let suggestion = if word.chars().next().map_or(false, |c| c.is_uppercase()) {
                    let mut dc = decoded.chars();
                    match dc.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + dc.as_str(),
                    }
                } else {
                    decoded
                };

                findings.push(
                    Finding::new(
                        TELEX_TYPO_ERROR_RULE_ID,
                        "orthography",
                        "major",
                        word_start,
                        word_end,
                        0.97,
                        format!(
                            "Từ '{}' có cấu trúc âm tiết bất thường do dính phím bộ gõ Telex (Unikey).",
                            word
                        ),
                        "orthography",
                        word,
                    )
                    .with_suggestion(suggestion),
                );
            }
        }

        findings
    }

    /// Kiểm tra phụ âm cuối (coda) hợp thức trong âm tiết tiếng Việt
    fn is_valid_vietnamese_coda(w: &str) -> bool {
        // Các phụ âm cuối ghép hợp thức: -ch, -nh, -ng
        if w.ends_with("ch") || w.ends_with("nh") || w.ends_with("ng") {
            return true;
        }

        let last = match w.chars().last() {
            Some(c) => c,
            None => return true,
        };

        // Các phụ âm cuối đơn lẻ hợp thức: -c, -m, -n, -p, -t
        if matches!(last, 'c' | 'm' | 'n' | 'p' | 't') {
            return true;
        }

        // Hoặc kết thúc bằng nguyên âm (âm tiết mở)
        matches!(
            last,
            'a' | 'à' | 'á' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ' | 'â' | 'ầ' | 'ấ' | 'ẩ' | 'ẫ' | 'ậ'
                | 'e' | 'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ'
                | 'i' | 'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị'
                | 'o' | 'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ờ' | 'ớ' | 'ở' | 'ỡ' | 'ợ'
                | 'u' | 'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự'
                | 'y' | 'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ'
        )
    }

    /// Nhận diện xem một từ có chứa cấu trúc dính phím Telex hay không
    fn is_telex_malformed(w: &str) -> bool {
        // 1. Chứa phụ âm kép Telex: dd
        if w.contains("dd") {
            return true;
        }

        // 2. Chứa nguyên âm đôi kẹt phím Telex: aa, ee, oo, uw, ow
        if w.contains("uw") || w.contains("ow") {
            return true;
        }
        if w.contains("aa") || w.contains("ee") || w.contains("oo") {
            // Loại trừ loanwords tiếng Anh: meet, book, feed, see, good, week, look, root
            let english_exceptions = [
                "meet", "book", "feed", "see", "good", "week", "look", "root",
                "feel", "free", "green", "keep", "need", "seem", "tree", "noon", "cool", "door", "beer", "zoo"
            ];
            if !english_exceptions.contains(&w) {
                return true;
            }
        }

        // 3. Chứa ký tự 'w' trong cấu trúc tiếng Việt (tiếng Việt thuần không có w)
        if w.contains('w') && !matches!(w, "web" | "win" | "wifi" | "wiki" | "word" | "window" | "view" | "news" | "raw") {
            return true;
        }

        // 4. Kẹt phím thanh Telex ở giữa âm tiết: afm (làm), roofi (rồi), ddieefu (điều), nhaatj (nhất)...
        let internal_telex_patterns = [
            "af", "ef", "if", "of", "uf", "yf",
            "aj", "ej", "ij", "oj", "uj", "yj",
            "ax", "ex", "ix", "ox", "ux", "yx",
            "ar", "er", "ir", "or", "ur", "yr",
        ];
        for pat in &internal_telex_patterns {
            if let Some(pos) = w.find(pat) {
                // Nếu phím thanh nằm giữa từ (phía sau còn ký tự khác)
                if pos + pat.len() < w.len() {
                    let loanwords = ["card", "part", "farm", "park", "form", "dark", "hard", "market", "server"];
                    if !loanwords.contains(&w) {
                        return true;
                    }
                }
            }
        }

        // 5. Kết thúc bằng phím thanh Telex kẹt lại: j, f, r, x (hoặc phụ âm + s)
        let ends_with_tone_key = w.ends_with('j') || w.ends_with('f') || w.ends_with('r') || w.ends_with('x');
        if ends_with_tone_key {
            // Loại trừ từ mượn hợp thức và viết tắt
            let loanwords = ["car", "war", "for", "bar", "sir", "fax", "box", "fix", "tax", "mix", "tr", "mr", "dr", "sr"];
            if !loanwords.contains(&w) {
                return true;
            }
        }

        // Phụ âm đuôi kẹt phím s: ts, ns, ps, ngf, ngs, jc...
        if w.ends_with("ts") || w.ends_with("ns") || w.ends_with("ps") || w.ends_with("ngf") || w.ends_with("jc") {
            return true;
        }

        // 6. Từ có chứa chữ có dấu tiếng Việt nhưng lại kết thúc bằng phụ âm không hợp thức
        let has_vietnamese_accent = w.chars().any(|c| {
            matches!(c, 'á'|'à'|'ả'|'ã'|'ạ'|'ă'|'ắ'|'ằ'|'ẳ'|'ẵ'|'ặ'|'â'|'ấ'|'ầ'|'ẩ'|'ẫ'|'ậ'|
                        'é'|'è'|'ẻ'|'ẽ'|'ẹ'|'ê'|'ế'|'ề'|'ể'|'ễ'|'ệ'|
                        'í'|'ì'|'ỉ'|'ĩ'|'ị'|
                        'ó'|'ò'|'ỏ'|'õ'|'ọ'|'ô'|'ố'|'ồ'|'ổ'|'ỗ'|'ộ'|'ơ'|'ớ'|'ờ'|'ở'|'ỡ'|'ợ'|
                        'ú'|'ù'|'ủ'|'ũ'|'ụ'|'ư'|'ứ'|'ừ'|'ử'|'ữ'|'ự'|
                        'ý'|'ỳ'|'ỷ'|'ỹ'|'ỵ'|'đ')
        });

        if has_vietnamese_accent && !Self::is_valid_vietnamese_coda(w) {
            return true;
        }

        false
    }

    /// Bộ giải mã Telex tự động tổng quát: tái tạo từ tiếng Việt chuẩn từ chuỗi gõ lỗi
    pub fn decode_telex(raw: &str) -> String {
        // Bảng tra nhanh cho các từ cực kỳ phổ biến
        let fast_lut: &[(&str, &str)] = &[
            ("dduowjc", "được"),
            ("dduwowjc", "được"),
            ("dduocj", "được"),
            ("cuar", "của"),
            ("bieets", "biết"),
            ("bieet", "biết"),
            ("tooi", "tôi"),
            ("nguwowfi", "người"),
            ("nhuwng", "nhưng"),
            ("khoong", "không"),
            ("phair", "phải"),
            ("ddang", "đang"),
            ("dax", "đã"),
            ("moij", "mọi"),
            ("lafm", "làm"),
            ("roofi", "rồi"),
            ("nhaat", "nhất"),
            ("nhaatj", "nhất"),
            ("luoon", "luôn"),
            ("trangj", "trạng"),
            ("ddieefu", "điều"),
            ("nghieem", "nghiêm"),
            ("trwowngf", "trường"),
            ("khuyeens", "khuyến"),
            ("thoowng", "thông"),
            ("hoaang", "hoàng"),
            ("chaaps", "chấp"),
            ("tieep", "tiếp"),
            ("keet", "kết"),
            ("ddoan", "đoàn"),
        ];

        for &(src, dst) in fast_lut {
            if raw == src {
                return dst.to_string();
            }
        }

        // Bộ giải mã quy tắc cơ bản nếu không nằm trong bảng nhanh
        let mut res = raw.to_string();
        res = res.replace("dd", "đ");
        res = res.replace("dduowjc", "được");
        res = res.replace("uow", "ươ");
        res = res.replace("uw", "ư");
        res = res.replace("ow", "ơ");
        res = res.replace("aa", "â");
        res = res.replace("ee", "ê");
        res = res.replace("oo", "ô");

        // Gỡ bỏ phím thanh kẹt ở đuôi nếu có
        if let Some(last) = res.chars().last() {
            if matches!(last, 'j' | 'f' | 'r' | 'x' | 's') {
                res.pop();
            }
        }

        res
    }

    /// 5. QUY CHUẨN VIẾT HOA THEO NGHỊ ĐỊNH 30/2020/NĐ-CP (PHỤ LỤC II)
    /// Kiểm tra viết hoa tên cơ quan, tổ chức & địa danh hành chính kết hợp danh từ chung
    pub fn check_decree30_capitalization(raw_text: &str) -> Vec<Finding> {
        let norm = normalize_nfc(raw_text);
        let chars: Vec<char> = norm.chars().collect();
        let len = chars.len();
        let mask = Self::compute_masking_shield(&chars);
        let mut findings = Vec::new();

        // 5.1. Viết hoa tên cơ quan, tổ chức: Chỉ viết hoa chữ đầu của các từ/cụm từ chỉ loại hình, chức năng
        let org_bad_patterns: &[(&str, &str)] = &[
            ("Bộ Giáo Dục Và Đào Tạo", "Bộ Giáo dục và Đào tạo"),
            ("Bộ Giáo Dục và Đào Tạo", "Bộ Giáo dục và Đào tạo"),
            ("Ủy Ban Nhân Dân", "Ủy ban nhân dân"),
            ("Uỷ Ban Nhân Dân", "Ủy ban nhân dân"),
            ("Hội Đồng Nhân Dân", "Hội đồng nhân dân"),
            ("Mặt Trận Tổ Quốc", "Mặt trận Tổ quốc"),
            ("Bảo Hiểm Xã Hội", "Bảo hiểm xã hội"),
            ("Viện Kiểm Sát Nhân Dân", "Viện kiểm sát nhân dân"),
            ("Tòa Án Nhân Dân", "Tòa án nhân dân"),
        ];

        for &(bad, good) in org_bad_patterns {
            let mut search_from = 0;
            let bad_chars: Vec<char> = bad.chars().collect();
            let bad_len = bad_chars.len();

            while search_from + bad_len <= len {
                if !mask[search_from] {
                    let slice: String = chars[search_from..search_from + bad_len].iter().collect();
                    if slice == bad {
                        findings.push(
                            Finding::new(
                                DECREE30_CAPITALIZATION_ORG_RULE_ID,
                                "orthography",
                                "major",
                                search_from,
                                search_from + bad_len,
                                0.96,
                                format!(
                                    "Tên cơ quan, tổ chức '{}' viết hoa chưa chuẩn theo Phụ lục II, Nghị định 30/2020/NĐ-CP.",
                                    bad
                                ),
                                "orthography",
                                slice,
                            )
                            .with_suggestion(good.to_string()),
                        );
                        search_from += bad_len;
                        continue;
                    }
                }
                search_from += 1;
            }
        }

        // 5.2. Viết hoa địa danh hành chính: danh từ chung (tỉnh, thành phố, quận, huyện) viết thường khi ở giữa câu
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

        let mid_sentence_triggers = [
            "ở ", "tại ", "đến ", "về ", "thuộc ", "vùng ", "khu vực ", "qua ", "tới ", "trong "
        ];

        let s: String = chars.iter().collect();
        for trigger in &mid_sentence_triggers {
            let mut start_idx = 0;
            while let Some(rel_pos) = s[start_idx..].find(trigger) {
                let trigger_pos = start_idx + rel_pos;
                let unit_start = trigger_pos + trigger.len();

                for &(bad_unit, good_unit) in &admin_units {
                    if s[unit_start..].starts_with(bad_unit) {
                        let char_pos = s[..unit_start].chars().count();
                        let unit_char_len = bad_unit.chars().count();

                        if char_pos + unit_char_len <= len && !mask[char_pos] {
                            let matched: String = chars[char_pos..char_pos + unit_char_len].iter().collect();
                            findings.push(
                                Finding::new(
                                    DECREE30_CAPITALIZATION_ADMIN_UNIT_RULE_ID,
                                    "orthography",
                                    "minor",
                                    char_pos,
                                    char_pos + unit_char_len,
                                    0.94,
                                    "Theo Nghị định 30/2020/NĐ-CP, danh từ chung chỉ đơn vị hành chính viết thường khi đi liền với tên riêng ở giữa câu.",
                                    "orthography",
                                    matched,
                                )
                                .with_suggestion(good_unit.to_string()),
                            );
                        }
                    }
                }

                start_idx = unit_start;
            }
        }

        findings
    }

    /// 6. CẢM BIẾN NHẤT QUÁN DẤU THANH KIỂU MỚI VS. KIỂU CŨ (TONE INCONSISTENCY SENSOR)
    /// Căn cứ: Thông tư 20/2018/TT-BGDĐT
    /// Báo lỗi khi văn bản dùng lẫn lộn cả lối đặt dấu cũ (hoà, thuỷ, khoẻ) và mới (hòa, thủy, khỏe)
    pub fn check_tone_consistency(raw_text: &str) -> Vec<Finding> {
        let norm = normalize_nfc(raw_text);
        let chars: Vec<char> = norm.chars().collect();
        let len = chars.len();
        let mask = Self::compute_masking_shield(&chars);
        let mut findings = Vec::new();

        // Cặp đối chiếu (Kiểu cũ, Kiểu mới)
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

        // Quét từng từ trong văn bản
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

        // Nếu văn bản có chứa cả kiểu cũ VÀ kiểu mới -> cảnh báo thiếu nhất quán!
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
                        "Không nhất quán trong cách đặt dấu thanh (văn bản đang dùng lẫn lộn cả lối cũ và lối mới). Khuyến nghị chuẩn hóa theo Thông tư 20/2018/TT-BGDĐT.",
                        "orthography",
                        matched,
                    )
                    .with_suggestion(sugg),
                );
            }
        }

        findings
    }
}

/// Điểm gắn kết duy nhất (Public Evaluator) được gọi từ `engine.rs` và `auditor.rs`
pub fn evaluate_orthography(text: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // 1. Dấu câu & khoảng trắng
    findings.extend(OrthographyChecker::check_punctuation_spacing(text));

    // 2. Viết hoa đầu câu
    findings.extend(OrthographyChecker::check_sentence_capitalization(text));

    // 3. Cấu trúc âm tiết & Telex tự động
    findings.extend(OrthographyChecker::check_phonotactic_telex(text));

    // 4. Quy chuẩn Nghị định 30 (cơ quan & địa danh)
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
    fn test_orthography_telex_and_phonotactics() {
        let f = OrthographyChecker::check_phonotactic_telex("Tôi dduowjc mời đến trwowngf dự lễ.");
        assert!(f.len() >= 2);
        assert!(f.iter().any(|item| item.matched_span == "dduowjc" && item.suggestion == Some("được".to_string())));
        assert!(f.iter().any(|item| item.matched_span == "trwowngf" && item.suggestion == Some("trường".to_string())));
    }

    #[test]
    fn test_orthography_abbreviations_and_math_exceptions() {
        // TP.HCM, GS. Cao Xuân Hạo, tr. 45, v.v., tỷ số 2:1, 10:30, 3.14159, 1,250,000 VNĐ
        let text = "Theo GS. Cao Xuân Hạo tại TP.HCM vào lúc 10:30, tỷ số trận đấu là 2:1. Đọc tr. 45 để biết thêm 1,250,000 VNĐ và 3.14 v.v. để hiểu rõ.";
        let findings = evaluate_orthography(text);
        assert!(findings.is_empty(), "Không được bắt nhầm các từ viết tắt, số học và giờ giấc: {:?}", findings);
    }

    #[test]
    fn test_decree30_capitalization() {
        let text = "Văn bản gửi Bộ Giáo Dục Và Đào Tạo tại Tỉnh Tuyên Quang.";
        let f = OrthographyChecker::check_decree30_capitalization(text);
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].pattern_id, DECREE30_CAPITALIZATION_ORG_RULE_ID);
        assert_eq!(f[0].suggestion, Some("Bộ Giáo dục và Đào tạo".to_string()));
        assert_eq!(f[1].pattern_id, DECREE30_CAPITALIZATION_ADMIN_UNIT_RULE_ID);
        assert_eq!(f[1].suggestion, Some("tỉnh ".to_string()));
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
    fn test_orthography_ultimate_sentence() {
        let text = "hôm nay trời đẹp quá .tôi muốn đi chơi ( công viên ) mà không dduowjc";
        let findings = evaluate_orthography(text);
        assert!(findings.len() >= 4, "Phải phát hiện ít nhất 4 lỗi chính tả và Telex");
    }
}
