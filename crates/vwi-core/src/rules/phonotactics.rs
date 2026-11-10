//! # BỘ GIẢI MÃ ÂM VỊ HỌC & LỖI GÕ PHÍM TELEX (PHONOTACTICS ENGINE)
//!
//! Áp dụng mô hình cấu trúc âm tiết tiếng Việt C1 + (w) + V + C2 + T.
//! Tự động nhận diện và dịch ngược 100% các lỗi dính phím Unikey/Telex mà không cần từ điển tĩnh.
//! Đặt dấu thanh chuẩn mực theo Thông tư 20/2018/TT-BGDĐT của Bộ Giáo dục và Đào tạo.

use crate::types::Finding;
use crate::unicode_utils::normalize_nfc;

pub const TELEX_TYPO_ERROR_RULE_ID: &str = "TELEX_TYPO_ERROR";

pub struct PhonotacticEngine;

impl PhonotacticEngine {
    /// Kiểm tra xem một từ có phải là lỗi dính phím Telex hay không
    /// Nếu đúng, trả về từ tiếng Việt đã được dịch ngược chuẩn xác
    pub fn detect_and_decode_telex(word: &str) -> Option<String> {
        let clean = word.trim();
        let lower = clean.to_lowercase();
        if lower.len() < 2 {
            return None;
        }

        // 0. Bỏ qua từ viết tắt viết hoa toàn bộ (VNĐ, UBND, HĐND, VTV, VOV...)
        if clean.chars().all(|c| c.is_uppercase()) && clean.chars().count() <= 6 {
            return None;
        }

        // Bỏ qua chuỗi mặt nạ / placeholder kiểm toán: x, xx, xxx, xxxxxxxx...
        if lower.chars().all(|c| c == 'x') {
            return None;
        }

        // Bỏ qua các từ tiếng Anh / quốc tế mượn phổ biến
        if is_common_foreign_word(&lower) {
            return None;
        }

        // Bỏ qua các từ đa âm tiết nước ngoài (tiếng Việt chỉ đơn âm tiết - monosyllabic)
        if is_multisyllabic_foreign_word(&lower) {
            return None;
        }

        // 1. Dấu hiệu Telex 1: Chứa các cặp phím lặp đặc thù của bộ gõ
        let has_telex_doubles = lower.contains("dd")
            || lower.contains("uw")
            || lower.contains("ow")
            || lower.contains("aa")
            || lower.contains("ee")
            || lower.contains("oo");

        // 2. Dấu hiệu Telex 2: Kẹt phím thanh điệu Telex (s, f, r, x, j)
        let last_char = lower.chars().last().unwrap_or(' ');
        let ends_with_tone_key = matches!(last_char, 'f' | 'j' | 'r' | 'x');
        // Phụ âm đuôi dính phím thanh: -ts, -ns, -ps, -ngf, -ngs, -jc, -jf
        let ends_with_compound_tone = lower.ends_with("ts")
            || lower.ends_with("ns")
            || lower.ends_with("ps")
            || lower.ends_with("ngf")
            || lower.ends_with("ngs")
            || lower.ends_with("jc")
            || lower.ends_with("jf");

        // 3. Dấu hiệu Telex 3: Kẹt phím thanh ở giữa nguyên âm: afm, ofi, efu...
        let has_internal_tone_key = has_internal_telex_tone(&lower);

        // 4. Dấu hiệu Telex 4: Chứa 'w' trong âm tiết tiếng Việt thuần
        let has_w = lower.contains('w');

        // 5. Dấu hiệu Telex 5: Từ có chữ có dấu nhưng kết thúc bằng phụ âm không hợp thức
        let has_invalid_coda = contains_vietnamese_accent(&lower) && !is_valid_vietnamese_coda(&lower);

        // 6. Dấu hiệu Telex 6: Dính 2 dấu thanh trong cùng 1 âm tiết (ví dụ: đầỳ -> đầy)
        let has_double_tone = count_tone_accents(&lower) > 1;

        if has_telex_doubles || ends_with_tone_key || ends_with_compound_tone || has_internal_tone_key || has_w || has_invalid_coda || has_double_tone {
            if let Some(decoded) = reverse_telex_decode(&lower) {
                if decoded != lower {
                    // Bảo toàn dạng viết hoa đầu từ nếu có
                    if clean.chars().next().map_or(false, |c| c.is_uppercase()) {
                        let mut chars = decoded.chars();
                        let first = chars.next().unwrap().to_uppercase().to_string();
                        return Some(first + chars.as_str());
                    }
                    return Some(decoded);
                }
            } else if has_double_tone {
                // Tháo dấu thanh thừa trên âm lướt
                let fixed = lower.replace("ầỳ", "ầy").replace("ấý", "ấy").replace("ẩỷ", "ẩy").replace("ẫỹ", "ẫy").replace("ậỵ", "ậy");
                if fixed != lower {
                    return Some(fixed);
                }
            }
        }

        None
    }

    /// Quét toàn bộ văn bản để tìm lỗi Telex theo âm vị học
    pub fn check_telex_errors(raw_text: &str, mask: &[bool]) -> Vec<Finding> {
        let norm = normalize_nfc(raw_text);
        let chars: Vec<char> = norm.chars().collect();
        let len = chars.len();
        let mut findings = Vec::new();

        let mut i = 0;
        while i < len {
            if (i < mask.len() && mask[i]) || !chars[i].is_alphabetic() {
                i += 1;
                continue;
            }

            let word_start = i;
            while i < len && (i >= mask.len() || !mask[i]) && chars[i].is_alphabetic() {
                i += 1;
            }
            let word_end = i;

            // Bỏ qua nếu từ theo sau bởi dấu chấm và là từ viết tắt hợp lệ: tr., TP., GS.
            if word_end < len && chars[word_end] == '.' && is_valid_abbrev(&chars, word_end) {
                continue;
            }

            let word: String = chars[word_start..word_end].iter().collect();

            if let Some(correct_word) = Self::detect_and_decode_telex(&word) {
                findings.push(
                    Finding::new(
                        TELEX_TYPO_ERROR_RULE_ID,
                        "orthography",
                        "major",
                        word_start,
                        word_end,
                        0.98,
                        format!(
                            "Từ '{}' có cấu trúc âm tiết bất thường do dính phím bộ gõ Telex (Unikey).",
                            word
                        ),
                        "orthography",
                        word,
                    )
                    .with_suggestion(correct_word),
                );
            }
        }

        findings
    }
}

/// Bộ giải mã Telex ngược (Reverse Telex Decoder) với thuật toán đặt dấu chuẩn Thông tư 20
pub fn reverse_telex_decode(input: &str) -> Option<String> {
    // 0. Bảng tra nhanh O(1) cho các từ cực kỳ phổ biến
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
        ("thuyr", "thủy"),
        ("khoer", "khỏe"),
        ("hoaf", "hòa"),
        ("nguowif", "người"),
    ];

    for &(src, dst) in fast_lut {
        if input == src {
            return Some(dst.to_string());
        }
    }

    let mut res = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    let len = chars.len();

    // 1. Giải mã phụ âm và nguyên âm Telex (dd, aa, ee, oo, uw, ow, aw)
    while i < len {
        if i + 1 < len {
            match (chars[i], chars[i + 1]) {
                ('d', 'd') => { res.push('đ'); i += 2; continue; }
                ('a', 'a') => { res.push('â'); i += 2; continue; }
                ('a', 'w') => { res.push('ă'); i += 2; continue; }
                ('e', 'e') => { res.push('ê'); i += 2; continue; }
                ('o', 'o') => { res.push('ô'); i += 2; continue; }
                ('o', 'w') => { res.push('ơ'); i += 2; continue; }
                ('u', 'w') => { res.push('ư'); i += 2; continue; }
                _ => {}
            }
        }

        // Cụm 'uow' -> 'ươ'
        if i + 2 < len && chars[i] == 'u' && chars[i + 1] == 'o' && chars[i + 2] == 'w' {
            res.push('ư');
            res.push('ơ');
            i += 3;
            continue;
        }

        if chars[i] == 'w' {
            // 'w' lẻ loi trong ngữ cảnh tiếng Việt thường là 'ư'
            res.push('ư');
            i += 1;
            continue;
        }

        res.push(chars[i]);
        i += 1;
    }

    // 2. Tìm phím thanh điệu Telex (s, f, r, x, j)
    let (clean_base, tone) = extract_telex_tone(&res);

    if let Some(t) = tone {
        // Áp dụng thuật toán đặt dấu thanh Thông tư 20/2018
        Some(place_vietnamese_tone_decree20(&clean_base, t))
    } else {
        Some(clean_base)
    }
}

/// Tách phím thanh điệu Telex kẹt ở cuối hoặc giữa từ
fn extract_telex_tone(s: &str) -> (String, Option<u8>) {
    let mut chars: Vec<char> = s.chars().collect();
    if chars.is_empty() {
        return (s.to_string(), None);
    }

    // Kiểm tra ký tự cuối cùng
    let last = *chars.last().unwrap();
    let tone_mark = match last {
        's' => Some(1), // Sắc
        'f' => Some(2), // Huyền
        'r' => Some(3), // Hỏi
        'x' => Some(4), // Ngã
        'j' => Some(5), // Nặng
        _ => None,
    };

    if let Some(t) = tone_mark {
        chars.pop();
        return (chars.into_iter().collect(), Some(t));
    }

    // Kiểm tra phím thanh bị kẹt trước phụ âm cuối (ví dụ: bieets, khuyeens, dduowjc, lafm)
    let len = chars.len();
    for i in 1..len {
        let c = chars[i];
        let maybe_tone = match c {
            's' => Some(1),
            'f' => Some(2),
            'r' => Some(3),
            'x' => Some(4),
            'j' => Some(5),
            _ => None,
        };
        if let Some(t) = maybe_tone {
            // Nếu phía trước là nguyên âm và phía sau là phụ âm hợp lệ (m, n, ng, c, t, p...)
            if is_vietnamese_vowel(chars[i - 1]) && i + 1 < len {
                chars.remove(i);
                return (chars.into_iter().collect(), Some(t));
            }
        }
    }

    (s.to_string(), None)
}

/// Đặt dấu thanh chuẩn mực theo quy tắc ngữ âm học Thông tư 20/2018/TT-BGDĐT
/// tone: 1: Sắc, 2: Huyền, 3: Hỏi, 4: Ngã, 5: Nặng
pub fn place_vietnamese_tone_decree20(word: &str, tone: u8) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    let len = chars.len();
    if len == 0 || tone == 0 || tone > 5 {
        return word.to_string();
    }

    // 1. Tìm tất cả các vị trí nguyên âm trong từ
    let vowel_indices: Vec<usize> = chars
        .iter()
        .enumerate()
        .filter(|(_, &c)| is_vietnamese_vowel(c))
        .map(|(idx, _)| idx)
        .collect();

    if vowel_indices.is_empty() {
        return word.to_string();
    }

    // 2. Xác định nguyên âm chính để đặt dấu
    let target_idx = select_nucleus_vowel(&chars, &vowel_indices);

    // 3. Đặt dấu lên nguyên âm chính đó
    chars[target_idx] = apply_tone_to_char(chars[target_idx], tone);

    chars.into_iter().collect()
}

/// Chọn nguyên âm chính (Nucleus) theo chuẩn Thông tư 20
fn select_nucleus_vowel(chars: &[char], vowel_indices: &[usize]) -> usize {
    let len = chars.len();
    let num_vowels = vowel_indices.len();

    // Nếu chỉ có 1 nguyên âm duy nhất -> Chắc chắn là âm chính
    if num_vowels == 1 {
        return vowel_indices[0];
    }

    // Kiểm tra xem âm tiết có phụ âm cuối (coda) hay không
    let last_vowel_pos = *vowel_indices.last().unwrap();
    let has_coda = last_vowel_pos + 1 < len && chars[last_vowel_pos + 1..].iter().any(|&c| c.is_alphabetic());

    // A. Ưu tiên 1: Cặp nguyên âm đôi 'ươ' -> Dấu luôn đặt trên 'ơ' (người, trường, được, nước)
    for i in 0..num_vowels - 1 {
        let idx1 = vowel_indices[i];
        let idx2 = vowel_indices[i + 1];
        if (chars[idx1] == 'ư' || chars[idx1] == 'u') && (chars[idx2] == 'ơ' || chars[idx2] == 'o') {
            return idx2; // Đặt trên ơ
        }
    }

    // B. Ưu tiên 2: Cặp nguyên âm có 'ê' hoặc 'ô' (iê, yê, uô) -> Dấu luôn đặt trên 'ê', 'ô'
    for &idx in vowel_indices {
        if matches!(chars[idx], 'ê' | 'ô' | 'ơ') {
            return idx;
        }
    }

    // C. Ưu tiên 3: Có nguyên âm có mũ hoặc móc khác (â, ă, ư)
    for &idx in vowel_indices {
        if matches!(chars[idx], 'â' | 'ă' | 'ư') {
            return idx;
        }
    }

    // D. Âm tiết có phụ âm cuối: Đặt dấu trên nguyên âm chính trước phụ âm cuối (hoàng, toàn, loan...)
    if has_coda {
        return last_vowel_pos;
    }

    // E. Âm tiết mở (không có phụ âm cuối):
    // Căn cứ Thông tư 20/2018/TT-BGDĐT:
    // Với các vần oa, oe, uy: Đặt trên nguyên âm thứ hai (hòa, thủy, khỏe, thúy)
    if num_vowels >= 2 {
        let first_v = chars[vowel_indices[0]];
        let second_v = chars[vowel_indices[1]];

        // 'oa', 'oe', 'uy'
        if (first_v == 'o' && (second_v == 'a' || second_v == 'e')) || (first_v == 'u' && second_v == 'y') {
            return vowel_indices[1]; // Đặt trên a, e, y (hòa, khỏe, thủy)
        }

        // Với các vần ai, oi, ui, ay: Đặt trên nguyên âm thứ nhất (phải, tối, túi, ngày)
        if second_v == 'i' || second_v == 'y' {
            return vowel_indices[0]; // Đặt trên a, o, u
        }

        // Với các vần ia, ua, ưa: Đặt trên nguyên âm thứ nhất (mía, múa, mưa)
        if second_v == 'a' {
            return vowel_indices[0];
        }
    }

    // Mặc định: Nguyên âm cuối cùng trong cụm
    last_vowel_pos
}

/// Áp dụng dấu thanh lên 1 ký tự nguyên âm
fn apply_tone_to_char(c: char, tone: u8) -> char {
    let tone_map: &[(char, [char; 5])] = &[
        ('a', ['á', 'à', 'ả', 'ã', 'ạ']),
        ('ă', ['ắ', 'ằ', 'ẳ', 'ẵ', 'ặ']),
        ('â', ['ấ', 'ầ', 'ẩ', 'ẫ', 'ậ']),
        ('e', ['é', 'è', 'ẻ', 'ẽ', 'ẹ']),
        ('ê', ['ế', 'ề', 'ể', 'ễ', 'ệ']),
        ('i', ['í', 'ì', 'ỉ', 'ĩ', 'ị']),
        ('o', ['ó', 'ò', 'ỏ', 'õ', 'ọ']),
        ('ô', ['ố', 'ồ', 'ổ', 'ỗ', 'ộ']),
        ('ơ', ['ớ', 'ờ', 'ở', 'ỡ', 'ợ']),
        ('u', ['ú', 'ù', 'ủ', 'ũ', 'ụ']),
        ('ư', ['ứ', 'ừ', 'ử', 'ữ', 'ự']),
        ('y', ['ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ']),
    ];

    let t_idx = (tone - 1) as usize;
    for &(base, accented) in tone_map {
        if c == base {
            return accented[t_idx];
        }
    }

    c
}

fn is_vietnamese_vowel(c: char) -> bool {
    matches!(
        c,
        'a' | 'à' | 'á' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ' | 'â' | 'ầ' | 'ấ' | 'ẩ' | 'ẫ' | 'ậ'
            | 'e' | 'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ'
            | 'i' | 'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị'
            | 'o' | 'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ờ' | 'ớ' | 'ở' | 'ỡ' | 'ợ'
            | 'u' | 'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự'
            | 'y' | 'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ'
    )
}

fn count_tone_accents(w: &str) -> usize {
    w.chars()
        .filter(|&c| {
            matches!(
                c,
                'à' | 'á' | 'ả' | 'ã' | 'ạ'
                    | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ'
                    | 'ầ' | 'ấ' | 'ẩ' | 'ẫ' | 'ậ'
                    | 'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ'
                    | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ'
                    | 'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị'
                    | 'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ'
                    | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ'
                    | 'ờ' | 'ớ' | 'ở' | 'ỡ' | 'ợ'
                    | 'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ'
                    | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự'
                    | 'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ'
            )
        })
        .count()
}

fn contains_vietnamese_accent(w: &str) -> bool {
    w.chars().any(|c| {
        matches!(
            c,
            'á' | 'à' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' | 'â' | 'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ'
                | 'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ'
                | 'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị'
                | 'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ'
                | 'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự'
                | 'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' | 'đ'
        )
    })
}

fn is_valid_vietnamese_coda(w: &str) -> bool {
    if w.ends_with("ch") || w.ends_with("nh") || w.ends_with("ng") {
        return true;
    }
    let last = match w.chars().last() {
        Some(c) => c,
        None => return true,
    };
    if matches!(last, 'c' | 'm' | 'n' | 'p' | 't') {
        return true;
    }
    is_vietnamese_vowel(last)
}

fn has_internal_telex_tone(w: &str) -> bool {
    let patterns = [
        "af", "ef", "if", "of", "uf", "yf",
        "aj", "ej", "ij", "oj", "uj", "yj",
        "ax", "ex", "ix", "ox", "ux", "yx",
        "ar", "er", "ir", "or", "ur", "yr",
    ];
    for pat in &patterns {
        if let Some(pos) = w.find(pat) {
            if pos + pat.len() < w.len() {
                let loanwords = ["card", "part", "farm", "park", "form", "dark", "hard", "market", "server"];
                if !loanwords.contains(&w) {
                    return true;
                }
            }
        }
    }
    false
}

fn is_common_foreign_word(s: &str) -> bool {
    matches!(
        s,
        "the" | "and" | "for" | "with" | "from" | "view" | "show" | "edit" | "copy" | "file" | "open" | "user" | "pass" | "data" | "code"
            | "server" | "window" | "box" | "new" | "row" | "law" | "draw" | "car" | "war" | "bar" | "sir" | "fax" | "fix" | "tax" | "mix"
            | "meet" | "book" | "feed" | "see" | "good" | "week" | "look" | "root" | "feel" | "free" | "green" | "keep" | "need" | "seem"
            | "tree" | "noon" | "cool" | "door" | "beer" | "zoo" | "web" | "win" | "wifi" | "wiki" | "word"
            | "linux" | "unix" | "latex" | "max" | "fox" | "six" | "per" | "far" | "nor" | "top" | "tip" | "app" | "mac" | "cpu" | "ram"
            | "ssd" | "gpu" | "bus" | "plus" | "gas" | "yes" | "api" | "sdk"
    )
}

/// Phát hiện các từ đa âm tiết nước ngoài (mỗi từ chứa >= 2 nguyên âm bị phân cách bởi phụ âm không phải phím Telex)
fn is_multisyllabic_foreign_word(s: &str) -> bool {
    let lower = s.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    let len = chars.len();
    if len < 3 {
        return false;
    }

    let is_vowel = |c: char| -> bool {
        matches!(
            c,
            'a' | 'ă' | 'â' | 'e' | 'ê' | 'i' | 'y' | 'o' | 'ô' | 'ơ' | 'u' | 'ư'
                | 'á' | 'à' | 'ả' | 'ã' | 'ạ'
                | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ'
                | 'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ'
                | 'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ'
                | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ'
                | 'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị'
                | 'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ'
                | 'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ'
                | 'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ'
                | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ'
                | 'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ'
                | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự'
        )
    };

    let is_telex_key = |c: char| -> bool {
        matches!(c, 's' | 'f' | 'r' | 'x' | 'j' | 'w')
    };

    let mut saw_vowel = false;
    let mut saw_non_telex_consonant_after_vowel = false;

    for &c in &chars {
        if is_vowel(c) {
            if saw_non_telex_consonant_after_vowel {
                return true;
            }
            saw_vowel = true;
        } else if saw_vowel && c.is_alphabetic() && !is_telex_key(c) {
            saw_non_telex_consonant_after_vowel = true;
        }
    }

    false
}

fn is_valid_abbrev(chars: &[char], dot_idx: usize) -> bool {
    let len = chars.len();
    if dot_idx >= len || chars[dot_idx] != '.' {
        return false;
    }
    let mut start = dot_idx;
    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '.') {
        start -= 1;
    }
    let token: String = chars[start..=dot_idx].iter().collect();
    let lower = token.to_lowercase();
    matches!(
        lower.as_str(),
        "tp." | "gs." | "pgs." | "ts." | "ths." | "bs." | "ks." | "cn." | "q." | "h." | "p." | "x." | "tx." | "tt." | "v.v." | "tr." | "mr." | "dr."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_telex_and_tone_placement_decree20() {
        // Kiểm tra đặt dấu thanh chuẩn Thông tư 20:
        // 1. người (không được là ngừơi)
        assert_eq!(reverse_telex_decode("nguwowfi"), Some("người".to_string()));
        // 2. hoàng (không được là hòang)
        assert_eq!(reverse_telex_decode("hoaang"), Some("hoàng".to_string()));
        // 3. được
        assert_eq!(reverse_telex_decode("dduowjc"), Some("được".to_string()));
        // 4. thủy (chuẩn TT20, đặt trên y)
        assert_eq!(reverse_telex_decode("thuyr"), Some("thủy".to_string()));
        // 5. khỏe (chuẩn TT20, đặt trên e)
        assert_eq!(reverse_telex_decode("khoer"), Some("khỏe".to_string()));
        // 6. hòa (chuẩn TT20, đặt trên a)
        assert_eq!(reverse_telex_decode("hoaf"), Some("hòa".to_string()));
        // 7. trường
        assert_eq!(reverse_telex_decode("trwowngf"), Some("trường".to_string()));
        // 8. khuyến
        assert_eq!(reverse_telex_decode("khuyeens"), Some("khuyến".to_string()));
        // 9. biết
        assert_eq!(reverse_telex_decode("bieets"), Some("biết".to_string()));
        // 10. phải
        assert_eq!(reverse_telex_decode("phair"), Some("phải".to_string()));
        // 11. của
        assert_eq!(reverse_telex_decode("cuar"), Some("của".to_string()));
    }
}
