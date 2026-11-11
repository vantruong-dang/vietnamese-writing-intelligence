//! # QUY CHUẨN THỂ THỨC VĂN BẢN HÀNH CHÍNH
//!
//! Căn cứ Nghị định 30/2020/NĐ-CP và Quyết định 1989/QĐ-TTg.
//! Bổ sung các rule còn thiếu: R1 (ngày), R3 (dẫn chiếu), R4 (danh từ đặc biệt),
//! R5 (địa danh), R7 (khẩu hiệu), R8 (số ký hiệu), R9 (địa danh + ngày),
//! R10 (tên cơ quan), R11 (số thập phân), R12 (quốc hiệu), R13 (thừa từ).

use crate::rules::trap_whitelist::mask_trap_phrases;
use crate::types::Finding;
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

pub const R1_DATE_PADDING_RULE_ID: &str = "R1_DATE_PADDING";
pub const R3_LEGAL_REF_RULE_ID: &str = "R3_LEGAL_REF_CASE";
pub const R4_SPECIAL_NOUN_RULE_ID: &str = "R4_SPECIAL_NOUN_CAPITALIZATION";
pub const R5_SPECIAL_PLACE_RULE_ID: &str = "R5_SPECIAL_PLACE_CAPITALIZATION";
pub const R7_MOTTO_RULE_ID: &str = "R7_MOTTO_CAPITALIZATION";
pub const R8_DOC_NUMBER_RULE_ID: &str = "R8_DOC_NUMBER_FORMAT";
pub const R9_PLACE_DATE_RULE_ID: &str = "R9_PLACE_DATE";
pub const R10_AGENCY_NAME_RULE_ID: &str = "R10_AGENCY_NAME_CAPITALIZATION";
pub const R11_DECIMAL_RULE_ID: &str = "R11_DECIMAL_COMMA";
pub const R12_QUOC_HIEU_RULE_ID: &str = "R12_QUOC_HIEU_SPELLING";
pub const R13_REDUNDANT_WORD_RULE_ID: &str = "R13_REDUNDANT_WORD";

// ── R1: ngày < 10 và tháng 1, 2 phải đệm 0 ──────────────────────────────
// Phụ lục I Nghị định 30/2020/NĐ-CP: "đối với những số thể hiện ngày nhỏ hơn
// 10 và tháng 1, 2 phải ghi thêm số 0 phía trước". Tháng 3 trở đi giữ nguyên
// (không có tháng 13, 14... nên không cần đệm).
static R1_DAY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bngày\s+(\d{1,2})\s+tháng\b").unwrap());
static R1_MONTH_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bngày\s+\d{1,2}\s+tháng\s+([12])\b").unwrap());

// ── R9: "Hà Nội ngày 20" -> "Hà Nội, ngày 20" ────────────────────────────
static R9_PLACE_DATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([\p{Lu}][\p{Ll}]*(?:\s+[\p{Lu}][\p{Ll}]*)*)\s+(ngày\s+\d{1,2}\s+tháng)").unwrap()
});
// "Hà Nội, Ngày 20" -> "Hà Nội, ngày 20"
static R9_CAPITALIZED_NGAY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(,\s*)Ngày\b").unwrap());
// Dòng địa danh + ngày: "Hà Nội, ngày 15/6/2026" -> "... ngày 15 tháng 6 năm 2026"
// Chỉ áp dụng khi có địa danh + dấu phẩy (đầu văn bản hành chính), không áp
// dụng cho "vào ngày 10/11/2026" trong câu văn thường.
static R9_SLASH_DATE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(,\s*ngày\s+)(\d{1,2})/(\d{1,2})/(\d{4})").unwrap());

// ── R2: thứ trong tuần & tên tháng khi đã xác định ngữ cảnh thời gian ────
// "vào thứ ba", "sáng thứ sáu" -> viết hoa tên thứ. Không bắt "đứng thứ ba"
// (số thứ tự) vì không có giới từ/trạng ngữ thời gian đứng trước.
static R2_WEEKDAY_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(vào|sáng|chiều|tối|trưa)\s+thứ\s+(hai|ba|tư|năm|sáu|bảy)\b").unwrap()
});
// "là tháng một" -> "là tháng Một" (định nghĩa tên tháng, không phải "một tháng").
static R2_MONTH_NAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\blà\s+tháng\s+(một|hai|ba|tư|năm|sáu|bảy|tám|chín|mười)\b").unwrap()
});

// ── R3: dẫn chiếu pháp lý ────────────────────────────────────────────────
static R3_KHOAN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bKhoản\b").unwrap());
static R3_DIEM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bĐiểm\b").unwrap());
static R3_DIEU_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bđiều\s+(?:\d+|[IVX]+)\b").unwrap());
static R3_CHUONG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bchương\s+(?:\d+|[IVX]+)\b").unwrap());
static R3_MUC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bmục\s+(?:\d+|[IVX]+)\b").unwrap());

// ── R4: danh từ đặc biệt ─────────────────────────────────────────────────
static R4_NHAN_DAN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bnhân\s+dân\b").unwrap());
static R4_NHA_NUOC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bnhà\s+nước\b").unwrap());

// ── R5: địa danh đặc biệt ────────────────────────────────────────────────
static R5_PLACE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\btại\s+(thủ đô|thành phố)\s+[\p{Lu}]").unwrap());

// ── R7: khẩu hiệu "Độc lập - Tự do - Hạnh phúc" ──────────────────────────
static R7_MOTTO_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)độc\s+lập\s*[-–]\s*tự\s+do\s*[-–]\s*hạnh\s+phúc").unwrap());

// ── R8: số ký hiệu văn bản ───────────────────────────────────────────────
static R8_SO_COLON_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\bSố\s+\d+/").unwrap());
static R8_CODE_HYPHEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Chấp nhận cả khoảng trắng lẫn gạch ngang en/em (lỗi gõ) giữa mã loại
    // văn bản và mã cơ quan: "31/KH–UBND", "15/QĐ UBND".
    Regex::new(r"(?i)(/[A-ZĐ]+)(\s*[–—]\s*|\s+)(UBND|HĐND)\b").unwrap()
});
// Mã loại văn bản sau dấu "/" phải đúng chính tả: KH, TB, QĐ, CTr, TT, NQ...
static R8_DOC_CODE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)/(kh|tb|qđ|qd|ctr|tt|nq)-").expect("Regex mã loại văn bản không hợp lệ")
});
static R8_SPACE_BEFORE_SLASH_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\d)\s+/([A-ZĐ])").unwrap());
static R8_DOC_TYPE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)thông\s+báo[^\n]*?số:\s*\d+/([A-Za-zĐ]+)\s*-").unwrap());

// ── R10: tên cơ quan, chức danh, danh hiệu ───────────────────────────────
static R10_PHRASES: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (
            Regex::new(r"(?i)\bbộ\s+giáo\s+dục\s+và\s+đào\s+tạo\b").unwrap(),
            "Bộ Giáo dục và Đào tạo",
        ),
        (
            Regex::new(r"(?i)\bbộ\s+trưởng\s+bộ\s+nội\s+vụ\b").unwrap(),
            "Bộ trưởng Bộ Nội vụ",
        ),
        (
            Regex::new(r"(?i)\bvăn\s+phòng\s+ủy\s+ban\s+nhân\s+dân\b").unwrap(),
            "Văn phòng Ủy ban nhân dân",
        ),
        (
            Regex::new(r"(?i)\bchủ\s+tịch\s+nước\b").unwrap(),
            "Chủ tịch nước",
        ),
        (
            Regex::new(r"(?i)\bhuân\s+chương\s+lao\s+động\b").unwrap(),
            "Huân chương Lao động",
        ),
    ]
});
// "Nguyễn văn An" -> "Nguyễn Văn An"
static R10_MIDDLE_NAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Phải có tên riêng viết hoa theo sau; tránh bắt oan "Công văn gửi...".
    Regex::new(r"\b([\p{Lu}][\p{Ll}]+)\s+(văn|thị)\s+([\p{Lu}][\p{Ll}]+)").unwrap()
});

// ── R11: số thập phân dùng dấu phẩy, phân cách nghìn dùng dấu chấm ──────
// Ghi chú: chỉ nhận diện 1 chữ số thập phân (6.5%). Số 3.14% hiện đang được
// bộ regression cũ chấp nhận; cần quyết định chính sách trước khi mở rộng.
static R11_DECIMAL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d)\.(\d)\s*%").unwrap());
// 1,500,000 -> 1.500.000 (nhóm 3 chữ số, không nhầm với 6,5%).
static R11_THOUSANDS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\d{1,3}(?:,\d{3})+").unwrap());

// ── R12: quốc hiệu ───────────────────────────────────────────────────────
static R12_QUOC_HIEU_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)cộng\s+h(?:oà|òa)\s+xã\s+hôi\s+chủ\s+nghĩa\s+việt\s+nam").unwrap()
});

// ── R13: "V/v về" -> "V/v" ───────────────────────────────────────────────
static R13_VV_VE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\bv/v\s+về\b").unwrap());

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn is_sentence_start(text: &str, byte_pos: usize) -> bool {
    let before = &text[..byte_pos];
    for c in before.chars().rev() {
        if c.is_whitespace() {
            continue;
        }
        return matches!(c, '.' | '!' | '?' | ':' | '\n');
    }
    true
}

#[allow(clippy::too_many_arguments)]
fn add_finding(
    findings: &mut Vec<Finding>,
    seen: &mut HashSet<(String, usize, usize)>,
    rule_id: &str,
    map: &CharOffsetMap,
    start_byte: usize,
    end_byte: usize,
    replacement: impl Into<String>,
    confidence: f64,
    rationale: impl Into<String>,
    matched_span: &str,
) {
    let (start, end) = map.byte_range_to_char_range(start_byte, end_byte);
    if start >= end {
        return;
    }
    let key = (rule_id.to_string(), start, end);
    if !seen.insert(key) {
        return;
    }
    findings.push(
        Finding::new(
            rule_id,
            "admin_format",
            "minor",
            start,
            end,
            confidence,
            rationale,
            "admin_format",
            matched_span,
        )
        .with_fix(start, end, replacement)
        .with_stage("regex"),
    );
}

pub fn evaluate_admin_format(raw_text: &str) -> Vec<Finding> {
    let normalized = normalize_nfc(raw_text);
    let text = mask_trap_phrases(&normalized);
    let map = CharOffsetMap::new(&text);
    let mut findings = Vec::new();
    let mut seen: HashSet<(String, usize, usize)> = HashSet::new();

    // ═══ R1: Đệm 0 cho ngày ═════════════════════════════════════════════
    for cap in R1_DAY_RE.captures_iter(&text) {
        let day = cap.get(1).unwrap();
        if day.as_str().len() == 1 {
            add_finding(
                &mut findings,
                &mut seen,
                R1_DATE_PADDING_RULE_ID,
                &map,
                day.start(),
                day.end(),
                format!("0{}", day.as_str()),
                0.97,
                "Ngày phải đủ 2 chữ số theo Nghị định 30/2020/NĐ-CP (ví dụ: ngày 05).",
                day.as_str(),
            );
        }
    }
    for cap in R1_MONTH_RE.captures_iter(&text) {
        let month = cap.get(1).unwrap();
        add_finding(
            &mut findings,
            &mut seen,
            R1_DATE_PADDING_RULE_ID,
            &map,
            month.start(),
            month.end(),
            format!("0{}", month.as_str()),
            0.97,
            "Tháng 1 và tháng 2 phải đệm số 0 theo Phụ lục I Nghị định 30/2020/NĐ-CP.",
            month.as_str(),
        );
    }

    // ═══ R9: Địa danh + ngày ════════════════════════════════════════════
    for cap in R9_PLACE_DATE_RE.captures_iter(&text) {
        let place = cap.get(1).unwrap();
        let ngay = cap.get(2).unwrap();
        // Không bắt nếu địa danh nằm giữa một từ dài hơn.
        if place.start() > 0 {
            if let Some(prev) = text[..place.start()].chars().next_back() {
                if prev.is_alphanumeric() {
                    continue;
                }
            }
        }
        // Chỉ chèn dấu phẩy vào khoảng trắng giữa địa danh và "ngày".
        add_finding(
            &mut findings,
            &mut seen,
            R9_PLACE_DATE_RULE_ID,
            &map,
            place.end(),
            ngay.start(),
            ", ",
            0.94,
            "Thiếu dấu phẩy ngăn cách giữa địa danh và ngày ban hành.",
            &text[place.end()..ngay.start()],
        );
    }
    for cap in R9_CAPITALIZED_NGAY_RE.captures_iter(&text) {
        let ngay = cap.get(0).unwrap();
        // Vị trí của chữ "Ngày" trong match.
        let offset = ngay.as_str().to_lowercase().find("ngày").unwrap_or(0);
        let start = ngay.start() + offset;
        // Chỉ báo khi thực sự viết hoa sai ("Ngày"); "ngày" đúng thì bỏ qua.
        if !text[start..].starts_with('N') {
            continue;
        }
        add_finding(
            &mut findings,
            &mut seen,
            R9_PLACE_DATE_RULE_ID,
            &map,
            start,
            start + "Ngày".len(),
            "ngày",
            0.95,
            "Sau dấu phẩy, 'ngày' viết thường theo mẫu thể thức.",
            "Ngày",
        );
    }

    // "Hà Nội, ngày 15/6/2026" -> "Hà Nội, ngày 15 tháng 6 năm 2026"
    for cap in R9_SLASH_DATE_RE.captures_iter(&text) {
        let day = cap.get(2).unwrap();
        let month = cap.get(3).unwrap();
        let year = cap.get(4).unwrap();
        let day_num = day.as_str().parse::<u32>().unwrap_or(0);
        let month_num = month.as_str().parse::<u32>().unwrap_or(0);
        let day_text = if day_num < 10 {
            format!("{:02}", day_num)
        } else {
            day_num.to_string()
        };
        let month_text = if month_num < 3 {
            format!("{:02}", month_num)
        } else {
            month_num.to_string()
        };
        add_finding(
            &mut findings,
            &mut seen,
            R9_PLACE_DATE_RULE_ID,
            &map,
            day.start(),
            year.end(),
            format!("{} tháng {} năm {}", day_text, month_text, year.as_str()),
            0.92,
            "Dòng địa danh - ngày ban hành phải viết đầy đủ 'ngày ... tháng ... năm ...'.",
            &text[day.start()..year.end()],
        );
    }

    // ═══ R2: Thứ trong tuần & tên tháng (ngữ cảnh thời gian) ═════════════
    for cap in R2_WEEKDAY_RE.captures_iter(&text) {
        let day = cap.get(2).unwrap();
        add_finding(
            &mut findings,
            &mut seen,
            R1_DATE_PADDING_RULE_ID,
            &map,
            day.start(),
            day.end(),
            capitalize_first(day.as_str()),
            0.9,
            "Tên thứ trong tuần viết hoa khi đứng sau trạng ngữ thời gian (vào/sáng/chiều...).",
            day.as_str(),
        );
    }
    for cap in R2_MONTH_NAME_RE.captures_iter(&text) {
        let month = cap.get(1).unwrap();
        add_finding(
            &mut findings,
            &mut seen,
            R1_DATE_PADDING_RULE_ID,
            &map,
            month.start(),
            month.end(),
            capitalize_first(month.as_str()),
            0.88,
            "Tên tháng viết hoa khi dùng như danh từ riêng (tháng Một, tháng Hai...).",
            month.as_str(),
        );
    }

    // ═══ R3: Dẫn chiếu pháp lý ══════════════════════════════════════════
    for m in R3_KHOAN_RE.find_iter(&text) {
        if !is_sentence_start(&text, m.start()) {
            add_finding(
                &mut findings,
                &mut seen,
                R3_LEGAL_REF_RULE_ID,
                &map,
                m.start(),
                m.end(),
                "khoản",
                0.92,
                "Trong câu, 'khoản' viết thường; chỉ viết hoa khi mở đầu câu/điểm.",
                m.as_str(),
            );
        }
    }
    for m in R3_DIEM_RE.find_iter(&text) {
        if !is_sentence_start(&text, m.start()) {
            add_finding(
                &mut findings,
                &mut seen,
                R3_LEGAL_REF_RULE_ID,
                &map,
                m.start(),
                m.end(),
                "điểm",
                0.92,
                "Trong câu, 'điểm' viết thường; chỉ viết hoa khi mở đầu câu/điểm.",
                m.as_str(),
            );
        }
    }
    for m in R3_DIEU_RE.find_iter(&text) {
        add_finding(
            &mut findings,
            &mut seen,
            R3_LEGAL_REF_RULE_ID,
            &map,
            m.start(),
            m.start() + "điều".len(),
            "Điều",
            0.92,
            "'Điều' viết hoa khi dẫn chiếu điều luật.",
            "điều",
        );
    }
    for m in R3_CHUONG_RE.find_iter(&text) {
        add_finding(
            &mut findings,
            &mut seen,
            R3_LEGAL_REF_RULE_ID,
            &map,
            m.start(),
            m.start() + "chương".len(),
            "Chương",
            0.92,
            "'Chương' viết hoa khi dẫn chiếu chương.",
            "chương",
        );
    }
    for m in R3_MUC_RE.find_iter(&text) {
        add_finding(
            &mut findings,
            &mut seen,
            R3_LEGAL_REF_RULE_ID,
            &map,
            m.start(),
            m.start() + "mục".len(),
            "Mục",
            0.92,
            "'Mục' viết hoa khi dẫn chiếu mục.",
            "mục",
        );
    }

    // ═══ R4: Danh từ đặc biệt ═══════════════════════════════════════════
    for m in R4_NHAN_DAN_RE.find_iter(&text) {
        if m.as_str().chars().next().is_some_and(|c| c.is_lowercase()) {
            add_finding(
                &mut findings,
                &mut seen,
                R4_SPECIAL_NOUN_RULE_ID,
                &map,
                m.start(),
                m.end(),
                capitalize_first(m.as_str()),
                0.9,
                "'Nhân dân' là danh từ chính trị - xã hội, viết hoa trong văn bản hành chính.",
                m.as_str(),
            );
        }
    }
    for m in R4_NHA_NUOC_RE.find_iter(&text) {
        if m.as_str().chars().next().is_some_and(|c| c.is_lowercase()) {
            // "cơ quan nhà nước", "tổ chức nhà nước": định ngữ -> viết thường.
            let mut lookback_start = m.start().saturating_sub(120);
            while lookback_start < m.start() && !text.is_char_boundary(lookback_start) {
                lookback_start += 1;
            }
            let window = text[lookback_start..m.start()].to_lowercase();
            if window.contains("cơ quan")
                || window.contains("tổ chức")
                || window.contains("chính quyền")
            {
                continue;
            }
            add_finding(
                &mut findings,
                &mut seen,
                R4_SPECIAL_NOUN_RULE_ID,
                &map,
                m.start(),
                m.end(),
                "Nhà nước",
                0.9,
                "'Nhà nước' viết hoa khi là chủ thể; viết thường khi làm định ngữ (cơ quan nhà nước).",
                m.as_str(),
            );
        }
    }

    // ═══ R5: Địa danh đặc biệt ══════════════════════════════════════════
    for cap in R5_PLACE_RE.captures_iter(&text) {
        let place = cap.get(1).unwrap();
        if place
            .as_str()
            .chars()
            .next()
            .is_some_and(|c| c.is_lowercase())
        {
            add_finding(
                &mut findings,
                &mut seen,
                R5_SPECIAL_PLACE_RULE_ID,
                &map,
                place.start(),
                place.end(),
                capitalize_first(place.as_str()),
                0.9,
                "Danh từ chung chỉ địa danh đặc biệt viết hoa khi đi với tên riêng.",
                place.as_str(),
            );
        }
    }

    // ═══ R7: Khẩu hiệu ══════════════════════════════════════════════════
    const MOTTO: &str = "Độc lập - Tự do - Hạnh phúc";
    for m in R7_MOTTO_RE.find_iter(&text) {
        let matched = m.as_str();
        if matched == MOTTO {
            continue;
        }
        let alpha = matched.chars().filter(|c| c.is_alphabetic()).count();
        let upper = matched.chars().filter(|c| c.is_uppercase()).count();
        if alpha > 0 && upper == alpha {
            continue; // Khẩu hiệu in hoa toàn bộ trên tiêu đề là hợp lệ.
        }
        add_finding(
            &mut findings,
            &mut seen,
            R7_MOTTO_RULE_ID,
            &map,
            m.start(),
            m.end(),
            MOTTO,
            0.96,
            "Khẩu hiệu quốc gia phải viết hoa đúng mẫu: Độc lập - Tự do - Hạnh phúc.",
            matched,
        );
    }

    // ═══ R8: Số ký hiệu văn bản ═════════════════════════════════════════
    for m in R8_SO_COLON_RE.find_iter(&text) {
        add_finding(
            &mut findings,
            &mut seen,
            R8_DOC_NUMBER_RULE_ID,
            &map,
            m.start(),
            m.start() + "Số".len(),
            "Số:",
            0.96,
            "Số ký hiệu văn bản phải viết 'Số:' (có dấu hai chấm).",
            "Số",
        );
    }
    for cap in R8_CODE_HYPHEN_RE.captures_iter(&text) {
        let code = cap.get(1).unwrap();
        let separator = cap.get(2).unwrap();
        let tail = cap.get(3).unwrap();
        if separator.as_str() == "-" {
            continue;
        }
        add_finding(
            &mut findings,
            &mut seen,
            R8_DOC_NUMBER_RULE_ID,
            &map,
            code.end(),
            tail.start(),
            "-",
            0.95,
            "Mã loại văn bản và mã cơ quan ban hành phải nối bằng dấu gạch nối thường (-).",
            separator.as_str(),
        );
    }
    for m in R8_DOC_CODE_RE.find_iter(&text) {
        // Bỏ dấu "/" ở đầu và "-" ở cuối, chỉ lấy phần mã.
        let code_start = m.start() + 1;
        let code_end = m.end() - 1;
        let code = &text[code_start..code_end];
        let canonical = match code.to_lowercase().as_str() {
            "kh" => "KH",
            "tb" => "TB",
            "qđ" | "qd" => "QĐ",
            "ctr" => "CTr",
            "tt" => "TT",
            "nq" => "NQ",
            _ => continue,
        };
        if code != canonical {
            add_finding(
                &mut findings,
                &mut seen,
                R8_DOC_NUMBER_RULE_ID,
                &map,
                code_start,
                code_end,
                canonical,
                0.95,
                "Mã loại văn bản phải viết đúng quy ước (KH, TB, QĐ, CTr...).",
                code,
            );
        }
    }
    for cap in R8_SPACE_BEFORE_SLASH_RE.captures_iter(&text) {
        let num = cap.get(1).unwrap();
        let following = cap.get(2).unwrap();
        let slash_pos = following.start() - '/'.len_utf8();
        add_finding(
            &mut findings,
            &mut seen,
            R8_DOC_NUMBER_RULE_ID,
            &map,
            num.end(),
            slash_pos,
            "",
            0.95,
            "Không có khoảng trắng trước dấu gạch chéo trong số ký hiệu.",
            &text[num.end()..slash_pos],
        );
    }
    for cap in R8_DOC_TYPE_RE.captures_iter(&text) {
        let code = cap.get(1).unwrap();
        if !code.as_str().eq_ignore_ascii_case("TB") {
            add_finding(
                &mut findings,
                &mut seen,
                R8_DOC_NUMBER_RULE_ID,
                &map,
                code.start(),
                code.end(),
                "TB",
                0.94,
                "Thông báo phải dùng mã loại văn bản 'TB'.",
                code.as_str(),
            );
        }
    }

    // ═══ R10: Tên cơ quan, chức danh, danh hiệu ═════════════════════════
    for (re, canonical) in R10_PHRASES.iter() {
        for m in re.find_iter(&text) {
            if m.as_str() == *canonical {
                continue;
            }
            add_finding(
                &mut findings,
                &mut seen,
                R10_AGENCY_NAME_RULE_ID,
                &map,
                m.start(),
                m.end(),
                *canonical,
                0.94,
                format!(
                    "Tên cơ quan/chức danh/danh hiệu viết hoa chưa chuẩn: '{}'.",
                    canonical
                ),
                m.as_str(),
            );
        }
    }
    for cap in R10_MIDDLE_NAME_RE.captures_iter(&text) {
        let middle = cap.get(2).unwrap();
        add_finding(
            &mut findings,
            &mut seen,
            R10_AGENCY_NAME_RULE_ID,
            &map,
            middle.start(),
            middle.end(),
            capitalize_first(middle.as_str()),
            0.88,
            "Tên riêng tiếng Việt phải viết hoa cả các âm tiết (ví dụ: Nguyễn Văn An).",
            middle.as_str(),
        );
    }

    // ═══ R11: Số thập phân ══════════════════════════════════════════════
    for cap in R11_DECIMAL_RE.captures_iter(&text) {
        let int_part = cap.get(1).unwrap();
        let frac = cap.get(2).unwrap();
        add_finding(
            &mut findings,
            &mut seen,
            R11_DECIMAL_RULE_ID,
            &map,
            int_part.end(),
            frac.start(),
            ",",
            0.93,
            "Trong văn bản hành chính Việt Nam, số thập phân dùng dấu phẩy (6,5%).",
            ".",
        );
    }
    // 1,500,000 -> 1.500.000 (dấu chấm phân cách hàng nghìn)
    for m in R11_THOUSANDS_RE.find_iter(&text) {
        let replacement = m.as_str().replace(',', ".");
        add_finding(
            &mut findings,
            &mut seen,
            R11_DECIMAL_RULE_ID,
            &map,
            m.start(),
            m.end(),
            replacement,
            0.92,
            "Phân cách hàng nghìn dùng dấu chấm theo quy ước văn bản Việt Nam (1.500.000).",
            m.as_str(),
        );
    }

    // ═══ R12: Quốc hiệu ═════════════════════════════════════════════════
    for m in R12_QUOC_HIEU_RE.find_iter(&text) {
        let lower = m.as_str().to_lowercase();
        if let Some(rel) = lower.find("hôi") {
            let start = m.start() + rel;
            let end = start + "hôi".len();
            let original = &text[start..end];
            let replacement = if original.chars().next().is_some_and(|c| c.is_uppercase()) {
                "HỘI"
            } else {
                "hội"
            };
            add_finding(
                &mut findings,
                &mut seen,
                R12_QUOC_HIEU_RULE_ID,
                &map,
                start,
                end,
                replacement,
                0.99,
                "Quốc hiệu viết đúng là 'CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM'.",
                original,
            );
        }
    }

    // ═══ R13: Thừa từ ═══════════════════════════════════════════════════
    for m in R13_VV_VE_RE.find_iter(&text) {
        add_finding(
            &mut findings,
            &mut seen,
            R13_REDUNDANT_WORD_RULE_ID,
            &map,
            m.start(),
            m.end(),
            "V/v",
            0.92,
            "Không dùng 'V/v về' (thừa từ); chỉ viết 'V/v'.",
            m.as_str(),
        );
    }

    findings.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| a.end.cmp(&b.end)));
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixes(text: &str) -> Vec<(String, String)> {
        evaluate_admin_format(text)
            .into_iter()
            .map(|f| {
                (
                    f.matched_span,
                    f.fix.map(|x| x.replacement).unwrap_or_default(),
                )
            })
            .collect()
    }

    #[test]
    fn test_r1_day_padding() {
        assert!(
            fixes("Hà Nội, ngày 5 tháng 3 năm 2026").contains(&("5".to_string(), "05".to_string()))
        );
        // Ngày đã đủ 2 chữ số: không báo.
        assert!(fixes("ngày 05 tháng 3 năm 2020").is_empty());
    }

    #[test]
    fn test_r1_month_jan_feb_padding_only() {
        // Tháng 1 và 2 phải đệm 0 (Phụ lục I Nghị định 30).
        assert!(fixes("Hà Nội, ngày 12 tháng 2 năm 2026")
            .contains(&("2".to_string(), "02".to_string())));
        assert!(fixes("Đà Nẵng, ngày 10 tháng 1 năm 2026")
            .contains(&("1".to_string(), "01".to_string())));
        // Tháng 3 trở đi KHÔNG đệm (tránh bắt oan).
        assert!(fixes("Hà Nội, ngày 05 tháng 3 năm 2020").is_empty());
    }

    #[test]
    fn test_r9_slash_date_in_header() {
        let f = fixes("Hà Nội, ngày 15/6/2026");
        assert!(f.iter().any(|(_, r)| r == "15 tháng 6 năm 2026"));
        // Trong câu văn thường (không có địa danh + dấu phẩy) thì không đổi.
        assert!(fixes("Dự án hoàn thành vào ngày 10/11/2026.").is_empty());
    }

    #[test]
    fn test_r2_weekday_and_month_name_context() {
        assert!(fixes("Hội nghị khai mạc vào thứ ba.")
            .iter()
            .any(|(s, r)| s == "ba" && r == "Ba"));
        assert!(fixes("Họp giao ban vào sáng thứ sáu.")
            .iter()
            .any(|(s, r)| s == "sáu" && r == "Sáu"));
        // "đứng thứ ba" (số thứ tự) không bị bắt.
        assert!(fixes("Đội của anh ấy đứng thứ ba.").is_empty());

        assert!(fixes("Tháng đầu tiên của năm là tháng một.")
            .iter()
            .any(|(s, r)| s == "một" && r == "Một"));
        // "một tháng" (số lượng) không bị bắt.
        assert!(fixes("Tôi ở đó một tháng.").is_empty());
    }

    #[test]
    fn test_r11_thousands_separator() {
        let f = fixes("Tổng kinh phí là 1,500,000 đồng.");
        assert!(f.iter().any(|(_, r)| r == "1.500.000"));
        // Số thập phân 6,5% không bị đổi thành phân cách nghìn.
        assert!(fixes("Tỷ lệ đạt 6,5%.").iter().all(|(_, r)| r != "6.5%"));
    }

    #[test]
    fn test_r9_place_date_comma() {
        assert!(fixes("Hà Nội ngày 20 tháng 6 năm 2026")
            .iter()
            .any(|(_, r)| r == ", "));
        assert!(fixes("Hà Nội, Ngày 20 tháng 6 năm 2026")
            .iter()
            .any(|(span, r)| span == "Ngày" && r == "ngày"));
    }

    #[test]
    fn test_r3_legal_refs() {
        let f = fixes("Thực hiện theo Khoản 2 Điều 5.");
        assert!(f.iter().any(|(s, r)| s == "Khoản" && r == "khoản"));
        let f = fixes("Căn cứ điều 15 Luật Lưu trữ.");
        assert!(f.iter().any(|(s, r)| s == "điều" && r == "Điều"));
    }

    #[test]
    fn test_r4_exempts_nha_nuoc_as_modifier() {
        assert!(fixes("các cơ quan, tổ chức nhà nước").is_empty());
        assert!(fixes("Chính sách này do nhà nước ban hành.")
            .iter()
            .any(|(s, r)| s == "nhà nước" && r == "Nhà nước"));
    }

    #[test]
    fn test_r8_doc_numbers() {
        let f = fixes("Số 15/QĐ-UBND");
        assert!(f.iter().any(|(s, r)| s == "Số" && r == "Số:"));
        let f = fixes("Số: 15/QĐ UBND");
        assert!(f.iter().any(|(s, r)| s == " " && r == "-"));
        let f = fixes("Số: 15/QD-UBND");
        assert!(f.iter().any(|(s, r)| s == "QD" && r == "QĐ"));
        let f = fixes("Số: 12/Kh-UBND");
        assert!(f.iter().any(|(s, r)| s == "Kh" && r == "KH"));
        let f = fixes("Số: 04/CTR-ĐHV");
        assert!(f.iter().any(|(s, r)| s == "CTR" && r == "CTr"));
    }

    #[test]
    fn test_r11_decimal_percent() {
        assert!(fixes("Tăng trưởng đạt 6.5% so với cùng kỳ.")
            .iter()
            .any(|(_, r)| r == ","));
    }

    #[test]
    fn test_r12_quoc_hieu_misspelling_only() {
        assert!(fixes("CỘNG HÒA XÃ HÔI CHỦ NGHĨA VIỆT NAM")
            .iter()
            .any(|(_, r)| r == "HỘI"));
        // Biến thể "HOÀ" được chấp nhận (trap case).
        assert!(fixes("CỘNG HOÀ XÃ HỘI CHỦ NGHĨA VIỆT NAM").is_empty());
    }
}
