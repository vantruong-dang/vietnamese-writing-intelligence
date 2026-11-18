//! # BỘ GIẢI MÃ ÂM VỊ HỌC & LỖI GÕ PHÍM TELEX/VNI (PHONOTACTICS ENGINE)
//!
//! Áp dụng mô hình cấu trúc âm tiết tiếng Việt C1 + (w) + V + C2 + T.
//! Tự động nhận diện và dịch ngược các lỗi dính phím Unikey/Telex (`dduowjc`,
//! `trwowngf`) và VNI (`d9u7o75c`, `tru7o72ng`) mà không cần từ điển tĩnh.
//! Đặt dấu thanh chuẩn mực theo Thông tư 20/2018/TT-BGDĐT của Bộ Giáo dục và Đào tạo.
//!
//! Chốt chặn precision: chỉ nhận khi từ sau giải mã có thật trong từ điển âm
//! tiết; bỏ qua tên riêng viết hoa, từ mượn công nghệ và các từ tiếng Anh ngắn
//! mà Telex/VNI tình cờ giải mã được (`was`, `if`, `keeps`...).

use std::collections::HashSet;
use std::sync::LazyLock;

use crate::segmentation::daachorse_segmenter::EMBEDDED_LEXICON_DATA;
use crate::types::Finding;
use crate::unicode_utils::normalize_nfc;

pub const TELEX_TYPO_ERROR_RULE_ID: &str = "TELEX_TYPO_ERROR";

/// Alias migration cho downstream: cùng một rule xử lý cả lỗi gõ Telex và VNI.
///
/// **Alias này hiện KHÔNG được emit.** Mọi `Finding` của bộ giải mã IME đều dùng
/// `TELEX_TYPO_ERROR_RULE_ID`; hằng số này chỉ để các bộ lọc/consumer chuyển dần
/// sang tên mới mà không phải sửa ngay. Đừng dùng nó để so khớp `pattern_id`.
pub const TELEX_VNI_INPUT_ERROR_RULE_ID: &str = TELEX_TYPO_ERROR_RULE_ID;

/// Tập hợp toàn bộ 10.760+ âm tiết tiếng Việt hợp lệ được chiết xuất từ từ điển chuẩn 72.500+ từ.
pub static VALID_VIETNAMESE_SYLLABLES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut set = HashSet::with_capacity(12000);
    for line in EMBEDDED_LEXICON_DATA.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            for word in trimmed.split_whitespace() {
                set.insert(word);
            }
        }
    }
    set
});

/// Kiểm tra xem một từ/âm tiết có tồn tại hợp lệ trong hệ thống âm tiết tiếng Việt chuẩn hay không
pub fn is_valid_vietnamese_word(word: &str) -> bool {
    let lower = word.to_lowercase();
    VALID_VIETNAMESE_SYLLABLES.contains(lower.as_str())
}

/// Danh sách từ mượn công nghệ & văn phòng phổ biến tuyệt đối không bắt lỗi Telex
pub static COMMON_TECH_LOANWORDS: &[&str] = &[
    "server", "client", "update", "upgrade", "deadline", "feedback", "marketing",
    "file", "code", "data", "link", "app", "web", "fix", "bug", "chat", "post",
    "user", "admin", "dev", "test", "check", "team", "task", "online", "offline",
    "air", "lori", "pro", "max", "mini", "plus", "ultra",
    "project", "sale", "sales", "lead", "kpi", "okr", "meeting", "mail", "email", "inbox",
    "log", "review", "report", "dashboard", "api", "database", "ui", "ux", "mcp",
    "cpu", "ram", "gpu", "ssd", "hdd", "linux", "unix", "windows", "mac", "macos",
    "ios", "android", "docker", "git", "github", "gitlab", "cloud", "ai", "llm",
    "frontend", "backend", "fullstack", "devops", "qa", "qc", "pm", "po", "scrum",
    "sprint", "backlog", "ticket", "pull", "push", "commit", "branch", "merge",
    "hotfix", "release", "build", "deploy", "setup", "config", "tool", "tools",
    "tech", "video", "audio", "format", "version", "domain", "hosting", "network",
    "wifi", "bluetooth", "security", "firewall", "router", "switch", "hub", "port",
    "ip", "dns", "url", "http", "https", "ssl", "tls", "token", "auth", "login",
    "logout", "signup", "signin", "password", "otp", "session", "cookie", "cache",
    "proxy", "gateway", "service", "system", "cluster", "node", "pod", "load",
    "traffic", "query", "index", "schema", "table", "column", "row", "record",
    "field", "form", "input", "output", "button", "menu", "popup", "dialog",
    "modal", "tab", "card", "list", "grid", "layout", "font", "icon", "theme",
    "dark", "light", "style", "css", "html", "js", "ts", "py", "rust", "wasm",
    "sdk", "cli", "gui", "ide", "vscode", "cursor", "chrome", "edge", "firefox",
    "safari", "google", "microsoft", "apple", "amazon", "meta", "facebook",
    "tiktok", "zalo", "telegram", "slack", "notion", "jira", "confluence", "trello",
    "figma", "canva", "zoom", "meet", "teams", "youtube", "drive", "docs", "sheets",
    "slides", "gmail", "calendar", "keep", "photos", "maps", "earth", "gemini",
    "openai", "chatgpt", "claude", "anthropic", "copilot", "deepmind", "antigravity",
    // Thuật ngữ công nghệ, định dạng tệp, đơn vị đo & công cụ lập trình
    "ops", "uri", "ms", "ns", "us", "fps", "qps", "tps", "rps",
    "kb", "mb", "gb", "tb", "hz", "khz", "mhz", "ghz", "km", "cm", "mm", "kg",
    "raw", "view", "window", "draw", "show", "new",
    "zip", "tar", "gz", "rar", "7z", "toml", "yaml", "yml", "json", "csv", "xml",
    "cargo", "pip", "npm", "pnpm", "yarn", "brew", "gem",
    "bash", "zsh", "sh", "powershell", "cmd", "bench", "benchmark", "perf",
    "sync", "async", "await", "spawn", "tokio", "rayon", "serde", "req", "res",
    "uuid", "regex", "linter", "lint", "env", "repo", "dep", "deps"
];

pub fn is_exempt_loanword(word: &str) -> bool {
    let lower = word.trim().to_lowercase();
    COMMON_TECH_LOANWORDS.iter().any(|&w| w == lower)
        || is_common_foreign_word(&lower)
        || is_multisyllabic_foreign_word(&lower)
}

/// Từ tiếng Anh ngắn, thông dụng mà nếu đem giải mã Telex/VNI sẽ trùng với một
/// âm tiết Việt hợp lệ (`was` → `ứa`, `if` → `ì`, `keeps` → `kếp`...).
///
/// Danh sách sinh từ 10.000 từ tiếng Anh phổ biến nhất
/// (`google-10000-english`), lọc bằng chính bộ giải mã này: chỉ giữ những từ
/// thực sự tạo ra âm tiết Việt. Đây là chốt chặn precision cho văn bản song
/// ngữ, không phải từ điển tiếng Anh đầy đủ.
static AMBIGUOUS_ENGLISH_WORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        // w + nguyên âm ở đầu từ (thêm guard riêng, giữ ở đây cho tường minh)
        "wa", "was", "we", "web", "were", "west", "what", "when", "where", "which",
        "while", "who", "why", "will", "win", "window", "windows", "winter", "wire",
        "wise", "wish", "with", "woman", "wonder", "wood", "word", "work", "world",
        "would", "write", "wrong",
        // w ở cuối/giữa từ
        "bow", "brown", "cow", "crowd", "dawn", "dow", "down", "draw", "flower",
        "follow", "how", "know", "knowledge", "law", "lawn", "low", "lows", "now",
        "own", "owns", "power", "raw", "review", "row", "rows", "saw", "show",
        "slow", "snow", "throw", "town", "towns", "toward", "yellow",
        // đuôi -s (phím sắc Telex) trùng âm tiết Việt
        "bits", "boost", "boots", "caps", "cats", "chips", "cons", "cups", "cuts",
        "dans", "gaps", "hans", "hats", "hits", "ins", "its", "keeps", "kits",
        "lens", "lets", "lips", "loans", "loops", "lots", "mats", "meets", "mens",
        "nuts", "oops", "owns", "queens", "rats", "roots", "runs", "sans", "sets",
        "sons", "songs", "thats", "tips", "tits", "tops", "trans", "ups",
        // đuôi -f / -r / -x (phím huyền/hỏi/ngã Telex) trùng âm tiết Việt
        "arm", "barn", "beef", "born", "buf", "burn", "chair", "char", "charm",
        "chef", "choir", "conf", "corn", "deer", "def", "der", "dir", "gary",
        "gif", "hair", "harm", "her", "hero", "horn", "inf", "ira", "mar", "mary",
        "norm", "par", "reef", "ref", "roof", "ser", "sex", "sofa", "sur", "taxi",
        "term", "tex", "turn", "var", "vary", "ver",
        // ee/oo/aa (phím mũ Telex) trùng âm tiết Việt
        "bee", "been", "boom", "dee", "ee", "keen", "lee", "moon", "oo", "pee",
        "queen", "room", "seen", "tee", "teen", "thee", "too",
    ]
    .into_iter()
    .collect()
});

/// Chuỗi ASCII bắt đầu bằng `w` + nguyên âm gần như luôn là tiếng Anh
/// (`was`, `web`, `window`...). Telex dùng `w` như phím biến đổi sau nguyên âm
/// hoặc đứng một mình, không dùng làm phụ âm đầu + nguyên âm.
fn starts_with_w_vowel(lower: &str) -> bool {
    let mut chars = lower.chars();
    matches!(chars.next(), Some('w')) && chars.next().is_some_and(is_vietnamese_vowel)
}

/// CHÍNH SÁCH C (precision-first cho extension): VNI chỉ được giải mã khi token
/// có ít nhất một phím biến đổi `6..9` (a8n, kho6ng, d9i) hoặc có từ hai chữ số
/// trở lên (vie65t, tru7o72ng, d9u7o75c).
///
/// Token chỉ có đúng một phím thanh `1..5` và không có phím biến đổi (`ban1`,
/// `hoc5`, `me5`, `co2`) bị coi là mã/viết tắt, không phải lỗi gõ VNI. Đây là
/// đánh đổi có chủ đích: giảm recall một số âm tiết VNI thuần ASCII để giữ
/// FPR = 0 trên mã tài sản/phòng ban.
fn passes_vni_precision_policy(token: &str) -> bool {
    let mut digit_count = 0usize;
    let mut has_diacritic_digit = false;
    let mut has_structural_letter = false;
    let mut tone_digit_pos: Option<usize> = None;
    let char_count = token.chars().count();

    for (pos, c) in token.chars().enumerate() {
        if c.is_ascii_digit() {
            digit_count += 1;
            if matches!(c, '6' | '7' | '8' | '9') {
                has_diacritic_digit = true;
            }
            if matches!(c, '1'..='5') {
                tone_digit_pos = Some(pos);
            }
        } else if matches!(c, 'đ' | 'ă' | 'â' | 'ê' | 'ô' | 'ơ' | 'ư') {
            has_structural_letter = true;
        }
    }

    if has_diacritic_digit || digit_count >= 2 {
        return true;
    }

    // Token "vỡ bộ gõ nửa chừng": phím 6..9 đã bị ăn thành chữ có dấu cấu tạo
    // (viê5t, trươ2ng, đươ5c), chỉ còn đúng một phím thanh 1..5 nằm giữa từ.
    // Không áp dụng khi token đã có dấu thanh hoặc phím số nằm cuối từ
    // (tầng5, chương2) — đó là số chú thích, không phải lỗi gõ.
    digit_count == 1
        && has_structural_letter
        && count_tone_accents(token) == 0
        && tone_digit_pos.is_some_and(|pos| pos + 1 < char_count)
}

fn is_ambiguous_english_word(lower: &str) -> bool {
    AMBIGUOUS_ENGLISH_WORDS.contains(lower)
}

/// Mã/viết tắt alphanumeric (`B1`, `B12`, `H2O`, `CO2`, `SO2`, `MP3`, `A4`...)
/// không phải lỗi gõ VNI.
///
/// Token viết hoa đã bị chặn từ trước, nhưng dạng viết thường vẫn còn rủi ro vì
/// VNI dùng chính các chữ số `1..5` làm phím thanh. Hai quy tắc:
/// - một phím thanh `1..5` nằm giữa từ là mã (`h2o`, `n2o`, `b2b`); VNI thật đặt
///   phím thanh ở cuối hoặc ngay sau nguyên âm mang phím biến đổi (`vie65t`,
///   `tru7o72ng`);
/// - một số mã hóa học phổ biến có phím thanh ở cuối nhưng vẫn trùng âm tiết
///   Việt hợp lệ (`co2` → `cò`, `no2` → `nò`, `so2` → `sò`).
fn looks_like_alphanumeric_code(lower: &str) -> bool {
    const COMMON_CODES: &[&str] = &["co2", "no2", "so2", "so3"];
    if COMMON_CODES.contains(&lower) {
        return true;
    }

    let mut tone_digit: Option<char> = None;
    let mut digit_count = 0usize;
    for c in lower.chars().filter(|c| c.is_ascii_digit()) {
        digit_count += 1;
        if matches!(c, '1'..='5') {
            tone_digit = Some(c);
        }
    }

    digit_count == 1
        && tone_digit.is_some_and(|d| !lower.ends_with(d))
}

pub struct PhonotacticEngine;

impl PhonotacticEngine {
    /// Kiểm tra xem một từ có phải là lỗi dính phím Telex hay không.
    /// Nếu đúng, trả về từ tiếng Việt đã được dịch ngược chuẩn xác.
    pub fn detect_and_decode_telex(word: &str) -> Option<String> {
        let clean = word.trim();
        let lower = clean.to_lowercase();
        // Chuỗi 2 ký tự ASCII quá nhập nhằng (`of`, `if`, `ow`...) và gần như
        // không phải lỗi gõ trong văn xuôi.
        if lower.chars().count() < 3 {
            return None;
        }

        // 0. Bỏ qua từ viết tắt viết hoa toàn bộ (VNĐ, UBND, HĐND, VTV, VOV...)
        if clean.chars().all(|c| c.is_uppercase()) && clean.chars().count() <= 6 {
            return None;
        }

        // 🛡️ BẢN VÁ: KHÔNG BAO GIỜ KIỂM TRA TELEX CHO TÊN RIÊNG / TỪ VIẾT HOA (Lori, Air, Jensen, Schmidt...)
        // Tránh tình trạng dịch ngược Telex oan cho tên riêng (ví dụ: Lori -> Lỏi, Air -> Ải)
        if clean.chars().next().is_some_and(|c| c.is_uppercase()) {
            return None;
        }

        // Bỏ qua chuỗi mặt nạ / placeholder kiểm toán: x, xx, xxx, xxxxxxxx...
        if lower.chars().all(|c| c == 'x') {
            return None;
        }

        // Bỏ qua từ mượn tiếng Anh / công nghệ / đa âm tiết nước ngoài (Tầng 1)
        // và các từ tiếng Anh ngắn mà giải mã Telex sẽ trùng âm tiết Việt.
        if is_exempt_loanword(&lower)
            || is_ambiguous_english_word(&lower)
            || starts_with_w_vowel(&lower)
        {
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
                    // Tầng 2: Dictionary Anchor: Từ sau khi giải mã PHẢI là từ/âm tiết tiếng Việt hợp thức
                    // và phải có nguyên âm (loại "ddr"→"đ", "dts"→"dt").
                    if is_valid_vietnamese_word(&decoded)
                        && decoded.chars().any(is_vietnamese_vowel)
                    {
                        return Some(decoded);
                    }
                }
            } else if has_double_tone {
                // Tháo dấu thanh thừa trên âm lướt
                let fixed = lower.replace("ầỳ", "ầy").replace("ấý", "ấy").replace("ẩỷ", "ẩy").replace("ẫỹ", "ẫy").replace("ậỵ", "ậy");
                if fixed != lower && is_valid_vietnamese_word(&fixed) {
                    return Some(fixed);
                }
            }
        }

        None
    }

    /// Kiểm tra lỗi gõ VNI (d9→đ, a6→â, a8→ă, e6→ê, o6→ô, o7→ơ, u7→ư,
    /// 1..5 là phím thanh). Trả về âm tiết tiếng Việt đã giải mã nếu hợp lệ.
    pub fn detect_and_decode_vni(word: &str) -> Option<String> {
        let clean = word.trim();
        let lower = clean.to_lowercase();
        // VNI luôn có ít nhất một chữ số; chuỗi quá ngắn dễ trùng nhãn/viết tắt.
        if lower.chars().count() < 3 || !lower.chars().any(|c| c.is_ascii_digit()) {
            return None;
        }
        if !lower
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || is_vietnamese_letter(c))
        {
            return None;
        }
        if clean.chars().next().is_some_and(|c| c.is_uppercase()) {
            return None;
        }
        if is_exempt_loanword(&lower) || is_ambiguous_english_word(&lower) {
            return None;
        }
        if !passes_vni_precision_policy(&lower) {
            return None;
        }
        // Chốt mã alphanumeric chỉ áp cho token ASCII thuần (h2o, co2, b2b).
        // Token mixed có chữ có dấu cấu tạo đã được Policy C kiểm soát riêng.
        if lower.is_ascii() && looks_like_alphanumeric_code(&lower) {
            return None;
        }

        let decoded = reverse_vni_decode(&lower)?;
        if decoded != lower && is_valid_vietnamese_word(&decoded) {
            Some(decoded)
        } else {
            None
        }
    }

    /// Giải mã lỗi gõ của cả hai bộ gõ phổ biến: Telex và VNI.
    pub fn detect_and_decode_ime(word: &str) -> Option<String> {
        Self::detect_and_decode_telex(word).or_else(|| Self::detect_and_decode_vni(word))
    }

    /// Quét toàn bộ văn bản để tìm lỗi gõ Telex/VNI theo âm vị học.
    pub fn check_ime_errors(raw_text: &str, mask: &[bool]) -> Vec<Finding> {
        let norm = normalize_nfc(raw_text);
        let chars: Vec<char> = norm.chars().collect();
        let len = chars.len();
        let mut findings = Vec::new();

        let mut i = 0;
        while i < len {
            // VNI dùng chữ số (d9u7o75c) nên token phải quét cả alphanumeric.
            if (i < mask.len() && mask[i]) || !chars[i].is_alphanumeric() {
                i += 1;
                continue;
            }

            let word_start = i;
            while i < len
                && (i >= mask.len() || !mask[i])
                && chars[i].is_alphanumeric()
            {
                i += 1;
            }
            let word_end = i;

            // Bỏ qua token thuần số (năm, ngày, số hiệu...).
            if !chars[word_start..word_end].iter().any(|c| c.is_alphabetic()) {
                continue;
            }

            // Bỏ qua nếu từ theo sau bởi dấu chấm và là từ viết tắt hợp lệ: tr., TP., GS.
            if word_end < len && chars[word_end] == '.' && is_valid_abbrev(&chars, word_end) {
                continue;
            }

            // 🛡️ BẢN VÁ: Bỏ qua từ viết hoa chữ cái đầu (Tên riêng, địa danh, tên người nước ngoài: Lori, Air...)
            if chars[word_start].is_uppercase() {
                continue;
            }

            let word: String = chars[word_start..word_end].iter().collect();

            if let Some(correct_word) = Self::detect_and_decode_ime(&word) {
                findings.push(
                    Finding::new(
                        TELEX_TYPO_ERROR_RULE_ID,
                        "orthography",
                        "major",
                        word_start,
                        word_end,
                        0.98,
                        format!(
                            "Từ '{}' có cấu trúc âm tiết bất thường do dính phím bộ gõ Telex/VNI.",
                            word
                        ),
                        "orthography",
                        word,
                    )
                    .with_fix(word_start, word_end, correct_word),
                );
            }
        }

        findings
    }

    /// Tương thích ngược: tên cũ chỉ xử lý Telex, nay bao phủ cả VNI.
    pub fn check_telex_errors(raw_text: &str, mask: &[bool]) -> Vec<Finding> {
        Self::check_ime_errors(raw_text, mask)
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

        // Cụm 'uow' -> 'ươ'. Không áp dụng khi 'u' là phụ âm đầu 'qu'
        // (quowr = quở, không phải qưở).
        if i + 2 < len
            && chars[i] == 'u'
            && chars[i + 1] == 'o'
            && chars[i + 2] == 'w'
            && !(i > 0 && chars[i - 1] == 'q')
        {
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

/// Bộ giải mã VNI ngược: `d9`→đ, `a6`→â, `a8`→ă, `e6`→ê, `o6`→ô, `o7`→ơ,
/// `u7`→ư; các phím `1..5` là dấu thanh (sắc, huyền, hỏi, ngã, nặng) và có
/// thể nằm cuối từ hoặc ngay sau nguyên âm chính (`vie65t`, `tru7o72ng`).
pub fn reverse_vni_decode(input: &str) -> Option<String> {
    let mut base = String::with_capacity(input.len());
    let mut tone: Option<u8> = None;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            'd' if chars.peek() == Some(&'9') => {
                chars.next();
                base.push('đ');
            }
            'a' if chars.peek() == Some(&'6') => {
                chars.next();
                base.push('â');
            }
            'a' if chars.peek() == Some(&'8') => {
                chars.next();
                base.push('ă');
            }
            'e' if chars.peek() == Some(&'6') => {
                chars.next();
                base.push('ê');
            }
            'o' if chars.peek() == Some(&'6') => {
                chars.next();
                base.push('ô');
            }
            'o' if chars.peek() == Some(&'7') => {
                chars.next();
                base.push('ơ');
            }
            'u' if chars.peek() == Some(&'7') => {
                chars.next();
                base.push('ư');
            }
            '1'..='5' => {
                if tone.is_some() {
                    return None;
                }
                tone = c.to_digit(10).map(|d| d as u8);
            }
            _ => {
                // Chữ số còn lại không thuộc ngữ pháp VNI (0, 6, 7, 8, 9...).
                if c.is_ascii_digit() {
                    return None;
                }
                base.push(c);
            }
        }
    }

    if !base.chars().any(is_vietnamese_vowel) {
        return None;
    }
    // Gõ nhanh thường bỏ `u7`: "uo7" vẫn phải cho ra "ươ" chứ không phải "uơ".
    // Không áp dụng khi `u` thuộc phụ âm đầu `qu` (quowr = quở, không phải qưở).
    let base = if base.starts_with("quơ") {
        base
    } else {
        base.replace("uơ", "ươ")
    };

    Some(match tone {
        Some(t) => place_vietnamese_tone_decree20(&base, t),
        None => base,
    })
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

    // 'gi' và 'qu' là phụ âm đầu khi theo sau là nguyên âm, nên 'i'/'u' không
    // phải hạt nhân: "gia" → giá/giải, "qua" → quá/quả. Nếu không tách, bộ giải
    // mã đặt dấu lên 'i'/'u' và sinh ra "gía"/"qùa" (không có trong từ điển).
    let onset_end = if len >= 3
        && ((chars[0] == 'g' && chars[1] == 'i') || (chars[0] == 'q' && chars[1] == 'u'))
        && is_vietnamese_vowel(chars[2])
    {
        2
    } else {
        0
    };
    let nucleus: Vec<usize> = vowel_indices
        .iter()
        .copied()
        .filter(|&i| i >= onset_end)
        .collect();
    let vowel_indices: &[usize] = if nucleus.is_empty() { vowel_indices } else { &nucleus };

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

        // Vần "ao", "au", "eo", "iu" (âm tiết mở): dấu đặt trên nguyên âm đầu
        // (báo, bảo, bão, tàu, kẹo, xíu). Thiếu rule này sẽ sinh "baỏ"/"baõ".
        if (first_v == 'a' && (second_v == 'o' || second_v == 'u'))
            || (first_v == 'e' && second_v == 'o')
            || (first_v == 'i' && second_v == 'u')
        {
            return vowel_indices[0];
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

fn is_vietnamese_letter(c: char) -> bool {
    c == 'đ' || is_vietnamese_vowel(c)
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
    let chars: Vec<char> = w.chars().collect();
    for i in 1..chars.len() {
        if matches!(chars[i], 's' | 'f' | 'r' | 'x' | 'j')
            && is_vietnamese_vowel(chars[i - 1])
            && i + 1 < chars.len()
        {
            let loanwords = [
                "card", "part", "farm", "park", "form", "dark", "hard", "market", "server",
            ];
            if !loanwords.contains(&w) {
                return true;
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

    #[test]
    fn test_tone_placement_open_ao_au_eo_iu() {
        // Vần mở: dấu phải nằm trên nguyên âm đầu.
        assert_eq!(place_vietnamese_tone_decree20("bao", 1), "báo");
        assert_eq!(place_vietnamese_tone_decree20("bao", 3), "bảo");
        assert_eq!(place_vietnamese_tone_decree20("bao", 4), "bão");
        assert_eq!(place_vietnamese_tone_decree20("tau", 2), "tàu");
        assert_eq!(place_vietnamese_tone_decree20("keo", 5), "kẹo");
        assert_eq!(place_vietnamese_tone_decree20("xiu", 1), "xíu");
    }

    #[test]
    fn test_capitalized_proper_nouns_bypass_telex() {
        // Tên riêng viết hoa tiếng Anh / nước ngoài tuyệt đối không decode Telex oan
        assert_eq!(PhonotacticEngine::detect_and_decode_telex("Lori"), None);
        assert_eq!(PhonotacticEngine::detect_and_decode_telex("Air"), None);
        assert_eq!(PhonotacticEngine::detect_and_decode_telex("Jensen"), None);
        assert_eq!(PhonotacticEngine::detect_and_decode_telex("Schmidt"), None);
        assert_eq!(PhonotacticEngine::detect_and_decode_telex("Taylor"), None);

        // Lowercase tech loanwords được miễn trừ
        assert_eq!(PhonotacticEngine::detect_and_decode_telex("air"), None);
        assert_eq!(PhonotacticEngine::detect_and_decode_telex("lori"), None);

        // check_telex_errors trên câu chứa tên riêng
        let text = "MacBook Air của anh Lori và ông Jensen chạy rất mượt.";
        let findings = PhonotacticEngine::check_telex_errors(text, &[]);
        assert!(findings.is_empty(), "Không được bắt nhầm tên riêng viết hoa: {:?}", findings);

        // Các lỗi Telex thật (chữ thường) vẫn bắt chuẩn 100%
        assert_eq!(PhonotacticEngine::detect_and_decode_telex("dduowjc"), Some("được".to_string()));
        assert_eq!(PhonotacticEngine::detect_and_decode_telex("trwowngf"), Some("trường".to_string()));
    }

    #[test]
    fn test_reverse_vni_decode() {
        for (raw, expected) in [
            ("d9u7o75c", "được"),
            ("tru7o72ng", "trường"),
            ("d9u7o72ng", "đường"),
            ("d9u7o7ng2", "đường"),
            ("kho6ng", "không"),
            ("ngu7o72i", "người"),
            ("vie65t", "việt"),
            ("hoc5", "học"),
            ("ban1", "bán"),
            ("d9a6u", "đâu"),
            ("tuo6i3", "tuổi"),
            ("thuye6n2", "thuyền"),
            ("a8n", "ăn"),
            ("gia1", "giá"),
            ("giai3", "giải"),
            ("qua2", "quà"),
            ("qui1", "quí"),
        ] {
            assert_eq!(
                reverse_vni_decode(raw),
                Some(expected.to_string()),
                "VNI decode sai: {raw}"
            );
        }

        // Chữ số không thuộc ngữ pháp VNI hoặc quá nhiều dấu thanh.
        assert_eq!(reverse_vni_decode("ban10"), None);
        assert_eq!(reverse_vni_decode("ban12"), None);
        assert_eq!(reverse_vni_decode("x9y7z"), None);
    }

    #[test]
    fn test_reverse_telex_gi_qu_onset() {
        assert_eq!(reverse_telex_decode("gias"), Some("giá".to_string()));
        assert_eq!(reverse_telex_decode("giar"), Some("giả".to_string()));
        assert_eq!(reverse_telex_decode("quas"), Some("quá".to_string()));
        assert_eq!(reverse_telex_decode("quar"), Some("quả".to_string()));
    }

    #[test]
    fn test_detect_and_decode_ime_handles_telex_and_vni() {
        assert_eq!(PhonotacticEngine::detect_and_decode_ime("dduowjc"), Some("được".to_string()));
        assert_eq!(PhonotacticEngine::detect_and_decode_ime("d9u7o75c"), Some("được".to_string()));
        assert_eq!(PhonotacticEngine::detect_and_decode_ime("trwowngf"), Some("trường".to_string()));
        assert_eq!(PhonotacticEngine::detect_and_decode_ime("tru7o72ng"), Some("trường".to_string()));

        // Phụ âm đầu 'gi'/'qu': 'i'/'u' không được coi là hạt nhân.
        // Dùng dạng có phím biến đổi 6..9 để thỏa Chính sách C.
        assert_eq!(PhonotacticEngine::detect_and_decode_ime("giuwx"), Some("giữ".to_string()));
        assert_eq!(
            PhonotacticEngine::detect_and_decode_ime("giu7o72ng"),
            Some("giường".to_string())
        );
        assert_eq!(PhonotacticEngine::detect_and_decode_ime("quo61c"), Some("quốc".to_string()));
        assert_eq!(PhonotacticEngine::detect_and_decode_ime("quowr"), Some("quở".to_string()));
    }

    #[test]
    fn test_mixed_diacritic_ime_tokens() {
        // Token "vỡ bộ gõ nửa chừng": phần đầu đã ăn dấu cấu tạo, phần sau còn
        // phím thô. Cả Telex lẫn VNI đều phải khôi phục đúng.
        for (raw, expected) in [
            ("đươcj", "được"),
            ("ngươif", "người"),
            ("viêjt", "việt"),
            ("trươfng", "trường"),
            ("đươ5c", "được"),
            ("viê5t", "việt"),
            ("trươ2ng", "trường"),
        ] {
            assert_eq!(
                PhonotacticEngine::detect_and_decode_ime(raw),
                Some(expected.to_string()),
                "Bỏ sót mixed-token {raw}"
            );
        }

        // Số chú thích dính sau âm tiết đã hoàn chỉnh không phải lỗi gõ.
        for raw in ["tầng5", "chương2", "việt5", "người2"] {
            assert_eq!(
                PhonotacticEngine::detect_and_decode_ime(raw),
                None,
                "Không được decode số chú thích {raw}"
            );
        }

        // Chính sách C: một phím thanh giữa từ chỉ hợp lệ khi có dấu cấu tạo
        // và chưa có dấu thanh.
        for raw in ["viê5t", "trươ2ng", "đươ5c"] {
            assert!(passes_vni_precision_policy(raw), "{raw}");
        }
        for raw in ["tầng5", "chương2", "việt5", "người2"] {
            assert!(!passes_vni_precision_policy(raw), "{raw}");
        }
    }

    #[test]
    fn test_ime_scan_reports_char_offsets_and_fixes() {
        let text = "Tôi đã d9u7o75c bài tập.";
        let findings = PhonotacticEngine::check_ime_errors(text, &[]);
        assert_eq!(findings.len(), 1, "{findings:?}");
        let f = &findings[0];
        assert_eq!(f.pattern_id, TELEX_VNI_INPUT_ERROR_RULE_ID);
        assert_eq!(f.matched_span, "d9u7o75c");
        // "Tôi đã " = 7 ký tự, token dài 8 ký tự.
        assert_eq!((f.start, f.end), (7, 15));
        assert_eq!(f.fix.as_ref().map(|x| x.replacement.as_str()), Some("được"));

        let mixed = PhonotacticEngine::check_ime_errors("dduowjc và trwowngf", &[]);
        let spans: Vec<(&str, &str)> = mixed
            .iter()
            .map(|f| {
                (
                    f.matched_span.as_str(),
                    f.fix.as_ref().map(|x| x.replacement.as_str()).unwrap_or(""),
                )
            })
            .collect();
        assert_eq!(spans, vec![("dduowjc", "được"), ("trwowngf", "trường")]);
    }

    #[test]
    fn test_ime_scan_skips_english_loanwords_and_masked_tokens() {
        for text in [
            "The server was down; if you see this, keep it simple.",
            "Anh check lại file log, fix bug rồi update lên server giùm em trước deadline nhé.",
            "now how low own keeps lets runs sets its was if",
            "Google Docs và Microsoft Teams.",
        ] {
            let findings = PhonotacticEngine::check_ime_errors(text, &[]);
            assert!(findings.is_empty(), "Bắt oan tiếng Anh {text:?}: {findings:?}");
        }

        // Mask che token lỗi → không báo.
        let text = "dduowjc";
        let mask = vec![true; text.chars().count()];
        assert!(PhonotacticEngine::check_ime_errors(text, &mask).is_empty());
    }

    #[test]
    fn test_ime_scan_skips_uppercase_short_and_non_vietnamese() {
        for token in ["D9u7o75c", "Dduowjc", "a1", "mp3", "win10", "abc123", "x9y7z"] {
            assert_eq!(
                PhonotacticEngine::detect_and_decode_ime(token),
                None,
                "Không được decode {token:?}"
            );
        }

        assert!(PhonotacticEngine::check_ime_errors("", &[]).is_empty());
        assert!(PhonotacticEngine::check_ime_errors("12345 !!! ???", &[]).is_empty());
    }

    #[test]
    fn test_ime_scan_skips_alphanumeric_codes() {
        // Token viết hoa và token thuần số đã bị chặn; kiểm tra cả câu để chắc
        // rằng scanner không tách sai `vitamin B12` hay `COVID-19`.
        for text in [
            "B1 B12 vitamin B12 COVID-19 MP3 A4",
            "Xe hạng B1 và B12, vitamin B12, COVID-19, MP3, A4.",
            "h2o n2o k2o co2 no2 so2 so3 b2b b2c",
        ] {
            let findings = PhonotacticEngine::check_ime_errors(text, &[]);
            assert!(findings.is_empty(), "Bắt oan mã alphanumeric {text:?}: {findings:?}");
        }

        // Guard không được chặn nhầm VNI hợp lệ theo Chính sách C: phải có
        // phím biến đổi 6..9 hoặc từ hai chữ số trở lên.
        for (raw, expected) in [
            ("a8n", "ăn"),
            ("kho6ng", "không"),
            ("vie65t", "việt"),
            ("tru7o72ng", "trường"),
            ("d9u7o75c", "được"),
        ] {
            assert_eq!(
                PhonotacticEngine::detect_and_decode_ime(raw),
                Some(expected.to_string()),
                "Guard chặn nhầm VNI hợp lệ: {raw}"
            );
        }

        // Chính sách C: token chỉ có một phím thanh 1..5 (không có 6..9) bị coi
        // là mã/viết tắt, kể cả khi bộ giải mã thuần có thể đọc được.
        for raw in ["ban1", "hoc5", "me5", "ca3", "lo2", "co2"] {
            assert_eq!(
                PhonotacticEngine::detect_and_decode_ime(raw),
                None,
                "Chính sách C phải bỏ qua {raw}"
            );
        }
        // Bộ giải mã thuần vẫn xử lý đúng các token đó (tách lớp rõ ràng).
        assert_eq!(reverse_vni_decode("ban1"), Some("bán".to_string()));
        assert_eq!(reverse_vni_decode("hoc5"), Some("học".to_string()));
    }

    #[test]
    fn test_ime_scan_skips_english_trailing_w() {
        for w in [
            "new", "view", "few", "know", "show", "how", "now", "low", "saw", "draw",
        ] {
            assert_eq!(
                PhonotacticEngine::detect_and_decode_ime(w),
                None,
                "Bắt oan từ tiếng Anh có w: {w}"
            );
            assert!(PhonotacticEngine::check_ime_errors(w, &[]).is_empty());
        }
    }
}
