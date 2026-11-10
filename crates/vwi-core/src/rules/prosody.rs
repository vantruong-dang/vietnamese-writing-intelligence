//! # MODULE: QUANTITATIVE PROSODY SENSOR (CẢM BIẾN TIẾT TẤU & NGỮ ÂM ĐỊNH LƯỢNG)
//!
//! ## 1. NGUYÊN LÝ KHOA HỌC DỰA TRÊN DỮ LIỆU THỰC CHỨNG (DATA-DRIVEN):
//! - Không dùng các con số võ đoán hay áp đặt luật thơ Đường luật vào văn xuôi hiện đại.
//! - Tham số toán học được trích xuất thực nghiệm từ **474.885 từ** qua **21.077 câu văn** của
//!   GS. Cao Xuân Hạo và học giả Nguyễn Hiến Lê (xem `generated_prosody_baselines.rs`).
//! - Dữ liệu thực chứng chỉ ra:
//!   + Tỷ lệ thanh tiếng Việt tự nhiên: **46.7% Bằng / 53.3% Trắc** (Thanh Trắc chiếm ưu thế tự nhiên).
//!   + Phân vị 95% chuỗi Trắc liên tiếp: **7 tiếng**.
//!   + Phân vị 99% chuỗi Trắc liên tiếp: **10 tiếng** (chuỗi dài nhất bắt gặp lên tới 17 tiếng trong văn luận thuyết căng thẳng).
//!
//! ## 2. NGUYÊN TẮC VẬN HÀNH:
//! - **CHỈ TƯ VẤN (Severity::Advisory):** Tuyệt đối KHÔNG trừ điểm chất lượng bài viết (0đ phạt).
//! - **MIỄN TRỪ THEO NGỮ VỰC (Register Guard):**
//!   + Văn bản Hành chính (`Register::Administrative`) & Kỹ thuật (`Register::Technical`):
//!     MIỄN TRỪ HOÀN TOÀN, không bao giờ can thiệp vào thuật ngữ hay văn bản pháp luật.

include!("generated_prosody_baselines.rs");

use crate::types::{Finding, Register, Severity};
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};

pub const PROSODIC_EXCESSIVE_TRAC_RULE_ID: &str = "PROSODIC_EXCESSIVE_TRAC";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToneCategory {
    Bang,  // Thanh Ngang, Huyền
    Trac,  // Thanh Sắc, Hỏi, Ngã, Nặng
    Other, // Ký tự số, ngoại lai, dấu câu
}

/// Xác định thanh điệu của một âm tiết tiếng Việt chuẩn NFC
pub fn classify_tone(syllable: &str) -> ToneCategory {
    let lower = syllable.trim().to_lowercase();
    let mut has_vowel = false;

    for c in lower.chars() {
        match c {
            // Thanh Trắc: Sắc, Hỏi, Ngã, Nặng
            'á' | 'ắ' | 'ấ' | 'é' | 'ế' | 'í' | 'ó' | 'ố' | 'ớ' | 'ú' | 'ứ' | 'ý' |
            'ả' | 'ẳ' | 'ẩ' | 'ẻ' | 'ể' | 'ỉ' | 'ỏ' | 'ổ' | 'ở' | 'ủ' | 'ử' | 'ỷ' |
            'ã' | 'ẵ' | 'ẫ' | 'ẽ' | 'ễ' | 'ĩ' | 'õ' | 'ỗ' | 'ỡ' | 'ũ' | 'ữ' | 'ỹ' |
            'ạ' | 'ặ' | 'ậ' | 'ẹ' | 'ệ' | 'ị' | 'ọ' | 'ộ' | 'ợ' | 'ụ' | 'ự' | 'ỵ' => {
                return ToneCategory::Trac;
            }
            // Thanh Bằng: Huyền
            'à' | 'ằ' | 'ầ' | 'è' | 'ề' | 'ì' | 'ò' | 'ồ' | 'ờ' | 'ù' | 'ừ' | 'ỳ' => {
                return ToneCategory::Bang;
            }
            // Nguyên âm không dấu (Thanh Ngang -> Bằng)
            'a' | 'ă' | 'â' | 'e' | 'ê' | 'i' | 'o' | 'ô' | 'ơ' | 'u' | 'ư' | 'y' => {
                has_vowel = true;
            }
            _ => {}
        }
    }

    if has_vowel {
        ToneCategory::Bang
    } else {
        ToneCategory::Other
    }
}

/// Đánh giá cảm biến ngữ âm thực nghiệm (Prosodic Advisory Sensor)
pub fn evaluate_prosody(text: &str, register: Register) -> Vec<Finding> {
    // 1. Miễn trừ hoàn toàn cho văn bản Hành chính & Kỹ thuật
    if matches!(register, Register::Administrative | Register::Technical) {
        return Vec::new();
    }

    let baseline = match get_baseline_for_register(register) {
        Some(b) => b,
        None => return Vec::new(),
    };

    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    // Tách văn bản thành các âm tiết kèm byte range
    let mut syllables = Vec::new();
    let mut start_idx = None;

    for (byte_idx, c) in norm.char_indices() {
        if c.is_alphanumeric() {
            if start_idx.is_none() {
                start_idx = Some(byte_idx);
            }
        } else {
            if let Some(s) = start_idx {
                syllables.push((&norm[s..byte_idx], s, byte_idx));
                start_idx = None;
            }
            // Nếu gặp dấu ngắt câu hoặc xuống dòng -> Ngắt chuỗi nhịp
            if c == '.' || c == ',' || c == ';' || c == ':' || c == '!' || c == '?' || c == '\n' || c == '—' {
                syllables.push((&norm[byte_idx..byte_idx + c.len_utf8()], byte_idx, byte_idx + c.len_utf8()));
            }
        }
    }
    if let Some(s) = start_idx {
        syllables.push((&norm[s..], s, norm.len()));
    }

    // Quét chuỗi Trắc liên tiếp vượt ngưỡng P99
    let mut current_trac_run = 0usize;
    let mut run_start_byte = 0;
    let mut run_end_byte = 0;

    for (token_text, s_byte, e_byte) in syllables {
        let is_clause_break = token_text.chars().any(|c| ".,;:!?\n—".contains(c));
        if is_clause_break {
            // Ngắt nhịp tại ranh giới dấu câu
            if current_trac_run > baseline.p99_consecutive_trac_threshold {
                let (start_char, end_char) = mapper.convert_range(run_start_byte, run_end_byte);
                let span = norm[run_start_byte..run_end_byte].to_string();
                findings.push(
                    Finding::new(
                        PROSODIC_EXCESSIVE_TRAC_RULE_ID,
                        "prosody",
                        Severity::Advisory.as_str(),
                        start_char,
                        end_char,
                        0.99,
                        format!(
                            "Dị biệt ngữ âm thống kê (p < 0.01): Chuỗi {} âm tiết Trắc liên tiếp ('{}') vượt ngưỡng phân vị 99% của văn xuôi thuần Việt (chuẩn P99 ≤ {} tiếng).",
                            current_trac_run, span, baseline.p99_consecutive_trac_threshold
                        ),
                        "empirical_prosody",
                        span,
                    )
                    .with_suggestion("Tùy chọn phong cách: Có thể xen kẽ thanh Bằng nếu muốn tiết tấu câu văn buông lơi, êm dịu hơn."),
                );
            }
            current_trac_run = 0;
            continue;
        }

        let tone = classify_tone(token_text);
        if tone == ToneCategory::Trac {
            if current_trac_run == 0 {
                run_start_byte = s_byte;
            }
            current_trac_run += 1;
            run_end_byte = e_byte;
        } else if tone == ToneCategory::Bang {
            if current_trac_run > baseline.p99_consecutive_trac_threshold {
                let (start_char, end_char) = mapper.convert_range(run_start_byte, run_end_byte);
                let span = norm[run_start_byte..run_end_byte].to_string();
                findings.push(
                    Finding::new(
                        PROSODIC_EXCESSIVE_TRAC_RULE_ID,
                        "prosody",
                        Severity::Advisory.as_str(),
                        start_char,
                        end_char,
                        0.99,
                        format!(
                            "Dị biệt ngữ âm thống kê (p < 0.01): Chuỗi {} âm tiết Trắc liên tiếp ('{}') vượt ngưỡng phân vị 99% của văn xuôi thuần Việt (chuẩn P99 ≤ {} tiếng).",
                            current_trac_run, span, baseline.p99_consecutive_trac_threshold
                        ),
                        "empirical_prosody",
                        span,
                    )
                    .with_suggestion("Tùy chọn phong cách: Có thể xen kẽ thanh Bằng nếu muốn tiết tấu câu văn buông lơi, êm dịu hơn."),
                );
            }
            current_trac_run = 0;
        }
    }

    if current_trac_run > baseline.p99_consecutive_trac_threshold {
        let (start_char, end_char) = mapper.convert_range(run_start_byte, run_end_byte);
        let span = norm[run_start_byte..run_end_byte].to_string();
        findings.push(
            Finding::new(
                PROSODIC_EXCESSIVE_TRAC_RULE_ID,
                "prosody",
                Severity::Advisory.as_str(),
                start_char,
                end_char,
                0.99,
                format!(
                    "Dị biệt ngữ âm thống kê (p < 0.01): Chuỗi {} âm tiết Trắc liên tiếp ('{}') vượt ngưỡng phân vị 99% của văn xuôi thuần Việt (chuẩn P99 ≤ {} tiếng).",
                    current_trac_run, span, baseline.p99_consecutive_trac_threshold
                ),
                "empirical_prosody",
                span,
            )
            .with_suggestion("Tùy chọn phong cách: Có thể xen kẽ thanh Bằng nếu muốn tiết tấu câu văn buông lơi, êm dịu hơn."),
        );
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_tone() {
        assert_eq!(classify_tone("sách"), ToneCategory::Trac);
        assert_eq!(classify_tone("vở"), ToneCategory::Trac);
        assert_eq!(classify_tone("nhẹ"), ToneCategory::Trac);
        assert_eq!(classify_tone("nghĩ"), ToneCategory::Trac);
        assert_eq!(classify_tone("bàn"), ToneCategory::Bang);
        assert_eq!(classify_tone("hoa"), ToneCategory::Bang);
        assert_eq!(classify_tone("123"), ToneCategory::Other);
    }

    #[test]
    fn test_register_administrative_immunity() {
        // Chuỗi 12 từ Trắc: Bị phạt nếu ở Literary, nhưng MIỄN TRỪ ở Administrative
        let text = "Cấu trúc khối dữ liệu phát tán lỗi tiếp tục phát tác rất gấp rút.";
        let findings_admin = evaluate_prosody(text, Register::Administrative);
        assert!(findings_admin.is_empty(), "Văn bản hành chính phải được miễn trừ hoàn toàn!");

        let findings_lit = evaluate_prosody(text, Register::Literary);
        assert_eq!(findings_lit.len(), 1, "Văn học phát hiện chuỗi trắc dị biệt thống kê P99");
        assert_eq!(findings_lit[0].severity, "advisory");
    }
}
