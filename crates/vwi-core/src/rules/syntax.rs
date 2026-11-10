use crate::types::Finding;
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};
use regex::RegexBuilder;
use std::sync::LazyLock;

static SYNTAX_REPORTING_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"(?:^|[.?!;\n]\s*)((?:phải nói là|cho là|cho rằng)\s+(?:một\s+)?(?:cú|sự|cuộc|việc|quyết định)\s+[^,.?!;\n]+(?:[.?!]|$))")
        .case_insensitive(true)
        .multi_line(true)
        .build()
        .expect("Regex CXH_REPORTING_VERB_INCOMPLETE_CLAUSE không hợp lệ")
});

static SYNTAX_PARALLEL_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"(?:^|[.?!;\n]\s*)(một\s+[^,.?!;\n]+?,?\s+nhưng\s+(?:không|chưa|thiếu)\b)")
        .case_insensitive(true)
        .multi_line(true)
        .build()
        .expect("Regex CXH_PARALLEL_CONJUNCTION_ASYMMETRY không hợp lệ")
});

static SYNTAX_CLICHE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\bnhư\s+[^,]+?\s+đã\s+biết\s+(?:là|rằng)\b")
        .case_insensitive(true)
        .build()
        .expect("Regex CXH_HYBRID_COMPLEX_CONJUNCTION không hợp lệ")
});

static REDUNDANT_REPORTING_LA_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\b(cho\s+biết|nói\s+rằng|tuyên\s+bố|nhấn\s+mạnh|khẳng\s+định|thừa\s+nhận|tiết\s+lộ)\s+là\b")
        .case_insensitive(true)
        .build()
        .expect("Regex CXH_REDUNDANT_REPORTING_LA không hợp lệ")
});

static REDUNDANT_CONJ_PAIRS_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\b(tuy\s+nhiên,\s*nhưng(?:\s+mà)?|bên\s+cạnh\s+đó,\s*đồng\s+thời|vì\s+vậy,\s*cho\s+nên|do\s+đó,\s*nên)\b")
        .case_insensitive(true)
        .build()
        .expect("Regex REDUNDANT_CONJUNCTION_PAIRS không hợp lệ")
});

static WRONG_HE_NEN_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\b(hễ)\b[^.,!?;\n]{1,50}?\b(nên)\b")
        .case_insensitive(true)
        .build()
        .expect("Regex LOGIC_HE_NEN_MISUSE không hợp lệ")
});

static NEU_CLAUSE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"(?:^|[.?!;\n]\s*)(nếu\b[^,.?!;\n]{3,100}?,[^.?!;\n]+[.?!])")
        .case_insensitive(true)
        .multi_line(true)
        .build()
        .expect("Regex NEU_CLAUSE không hợp lệ")
});

static BAO_NHIEU_CLAUSE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\b(bao\s+nhiêu\b[^.?!;\n]+[.?!])")
        .case_insensitive(true)
        .build()
        .expect("Regex BAO_NHIEU_CLAUSE không hợp lệ")
});

static REDUNDANT_CAUSE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\b(chính\s+vì\b[^,.?!;\n]{3,100}?\bnên)\b")
        .case_insensitive(true)
        .build()
        .expect("Regex REDUNDANT_CAUSE không hợp lệ")
});

pub const SYNTAX_REPORTING_RULE_ID: &str = "CXH_REPORTING_VERB_INCOMPLETE_CLAUSE";
pub const SYNTAX_PARALLEL_RULE_ID: &str = "CXH_PARALLEL_CONJUNCTION_ASYMMETRY";
pub const SYNTAX_CLICHE_RULE_ID: &str = "CXH_HYBRID_COMPLEX_CONJUNCTION";
pub const SYNTAX_REDUNDANT_REPORTING_LA_RULE_ID: &str = "CXH_REDUNDANT_REPORTING_LA";
pub const SYNTAX_REDUNDANT_CONJ_PAIRS_RULE_ID: &str = "REDUNDANT_CONJUNCTION_PAIRS";
pub const SYNTAX_HE_NEN_MISUSE_RULE_ID: &str = "LOGIC_HE_NEN_MISUSE";
pub const SYNTAX_MISSING_THI_AFTER_NEU_RULE_ID: &str = "MISSING_THI_AFTER_NEU";
pub const SYNTAX_MISSING_BAY_NHIEU_RULE_ID: &str = "MISSING_BAY_NHIEU";
pub const SYNTAX_REDUNDANT_CAUSE_RULE_ID: &str = "REDUNDANT_CAUSE";

/// Bắt lỗi sau vị từ trần thuật dùng một danh ngữ đơn độc thay vì mệnh đề trọn vẹn.
pub fn evaluate_syntax_reporting(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for cap in SYNTAX_REPORTING_RE.captures_iter(&norm) {
        if let Some(m) = cap.get(1) {
            let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
            findings.push(
                Finding::new(
                    SYNTAX_REPORTING_RULE_ID,
                    "syntax_reporting",
                    "major",
                    start_char,
                    end_char,
                    0.95,
                    "Sau 'phải nói là' bắt buộc phải là một mệnh đề trọn vẹn, không dùng một danh ngữ đơn độc.",
                    "linguistic",
                    norm[m.start()..m.end()].to_string(),
                )
                .with_suggestion("Bổ sung vị ngữ để tạo mệnh đề hoàn chỉnh sau vị từ trần thuật"),
            );
        }
    }

    findings
}

/// Bắt lỗi liên từ 'nhưng' nối hai vế bất đối xứng (danh ngữ đối lập vị từ cụt).
pub fn evaluate_syntax_parallel(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for cap in SYNTAX_PARALLEL_RE.captures_iter(&norm) {
        if let Some(m) = cap.get(1) {
            let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
            findings.push(
                Finding::new(
                    SYNTAX_PARALLEL_RULE_ID,
                    "syntax_parallel",
                    "major",
                    start_char,
                    end_char,
                    0.95,
                    "Liên từ 'nhưng' nối hai vế bất đối xứng: Danh ngữ đứng trước đối lập với cụm vị từ cụt phía sau.",
                    "linguistic",
                    norm[m.start()..m.end()].to_string(),
                )
                .with_suggestion("Cân bằng hai vế liên từ 'nhưng' cùng là danh ngữ hoặc cùng là vị ngữ trọn vẹn"),
            );
        }
    }

    findings
}

/// Bắt lỗi ghép nhầm hai cấu trúc diễn ngôn 'Như... đã biết' và '... đã biết rằng'.
pub fn evaluate_syntax_cliche(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in SYNTAX_CLICHE_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        findings.push(
            Finding::new(
                SYNTAX_CLICHE_RULE_ID,
                "syntax_cliche",
                "minor",
                start_char,
                end_char,
                0.95,
                "Ghép nhầm hai kiểu câu: 'Như... đã biết' và '... đã biết rằng'.",
                "linguistic",
                norm[m.start()..m.end()].to_string(),
            )
            .with_suggestion("Chỉ dùng 'Như... đã biết,' hoặc 'Chúng ta đều biết rằng...'"),
        );
    }

    findings
}

/// Bắt lỗi thừa hư từ 'là' sau động từ phát ngôn ('cho biết là', 'tuyên bố là'...).
pub fn evaluate_redundant_reporting_la(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in REDUNDANT_REPORTING_LA_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        let span = norm[m.start()..m.end()].to_string();
        // Rút gọn: bỏ " là" ở cuối cụm
        let repl = if span.ends_with(" là") {
            span[..span.len() - 3].to_string()
        } else {
            span.replace(" là", "")
        };

        findings.push(
            Finding::new(
                SYNTAX_REDUNDANT_REPORTING_LA_RULE_ID,
                "syntax",
                "minor",
                start_char,
                end_char,
                0.95,
                "Thừa hư từ 'là' sau động từ phát ngôn. Hãy dẫn trực tiếp mệnh đề để câu văn gãy gọn hơn.",
                "linguistic",
                span,
            )
            .with_note_and_fix(
                "Lược bỏ hư từ 'là' thừa sau động từ phát ngôn.",
                start_char,
                end_char,
                repl,
            ),
        );
    }

    findings
}

/// Bắt lỗi dùng cặp từ nối trùng lặp ở đầu câu ('Tuy nhiên, nhưng', 'Bên cạnh đó, đồng thời'...).
pub fn evaluate_redundant_conjunction_pairs(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in REDUNDANT_CONJ_PAIRS_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        let span = norm[m.start()..m.end()].to_string();
        let lower = span.to_lowercase();
        let repl = if lower.contains("tuy nhiên") {
            "Tuy nhiên,"
        } else if lower.contains("bên cạnh đó") {
            "Bên cạnh đó,"
        } else if lower.contains("vì vậy") {
            "Vì vậy,"
        } else {
            "Do đó,"
        };

        findings.push(
            Finding::new(
                SYNTAX_REDUNDANT_CONJ_PAIRS_RULE_ID,
                "syntax",
                "minor",
                start_char,
                end_char,
                0.95,
                "Xếp chồng cặp liên từ trùng lặp ở đầu câu. Chỉ nên dùng một liên từ duy nhất.",
                "linguistic",
                span,
            )
            .with_note_and_fix(
                "Chỉ dùng một liên từ duy nhất để câu văn mạch lạc.",
                start_char,
                end_char,
                repl,
            ),
        );
    }

    findings
}

/// Bắt lỗi sai logic liên từ 'Hễ... nên' (chuẩn phải là 'Hễ... thì').
pub fn evaluate_he_nen(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in WRONG_HE_NEN_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        let span = norm[m.start()..m.end()].to_string();
        let repl = span.replace("nên", "thì").replace("Nên", "Thì");

        findings.push(
            Finding::new(
                SYNTAX_HE_NEN_MISUSE_RULE_ID,
                "syntax",
                "major",
                start_char,
                end_char,
                0.95,
                "Sai logic liên từ: Cấu trúc điều kiện 'Hễ' phải đi với hệ quả 'thì', không kết hợp với 'nên'.",
                "linguistic",
                span,
            )
            .with_note_and_fix(
                "Đổi 'nên' thành 'thì' theo quy chuẩn cặp liên từ tiếng Việt.",
                start_char,
                end_char,
                repl,
            ),
        );
    }

    findings
}

/// Bắt lỗi cấu trúc 'Nếu...' thiếu liên từ 'thì' ở vế kết quả.
pub fn evaluate_missing_thi_after_neu(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for cap in NEU_CLAUSE_RE.captures_iter(&norm) {
        if let Some(m) = cap.get(1) {
            let matched_str = m.as_str();
            // Tách theo dấu phẩy để kiểm tra vế sau có chứa từ 'thì' không
            if let Some((_, after_comma)) = matched_str.split_once(',') {
                let lower_after = after_comma.to_lowercase();
                let has_thi = lower_after.split_whitespace().any(|w| w == "thì" || w.starts_with("thì"));
                if !has_thi {
                    let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
                    findings.push(
                        Finding::new(
                            SYNTAX_MISSING_THI_AFTER_NEU_RULE_ID,
                            "syntax",
                            "major",
                            start_char,
                            end_char,
                            0.9,
                            "Cấu trúc hô ứng 'Nếu...' bắt buộc phải có 'thì' ở vế kết quả để đảm bảo tính liên tục của câu.",
                            "linguistic",
                            matched_str.to_string(),
                        )
                        .with_suggestion("Thêm 'thì' vào đầu mệnh đề kết quả sau dấu phẩy"),
                    );
                }
            }
        }
    }

    findings
}

/// Bắt lỗi cấu trúc hô ứng 'Bao nhiêu...' thiếu 'bấy nhiêu'.
pub fn evaluate_missing_bay_nhieu(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for cap in BAO_NHIEU_CLAUSE_RE.captures_iter(&norm) {
        if let Some(m) = cap.get(1) {
            let matched_str = m.as_str();
            let lower = matched_str.to_lowercase();
            if !lower.contains("bấy nhiêu") {
                let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
                findings.push(
                    Finding::new(
                        SYNTAX_MISSING_BAY_NHIEU_RULE_ID,
                        "syntax",
                        "major",
                        start_char,
                        end_char,
                        0.9,
                        "Cấu trúc hô ứng so sánh 'Bao nhiêu...' cần có 'bấy nhiêu' ở vế đối ứng.",
                        "linguistic",
                        matched_str.to_string(),
                    )
                    .with_suggestion("Bổ sung 'bấy nhiêu' vào vế sau để cân đối câu văn"),
                );
            }
        }
    }

    findings
}

/// Bắt lỗi thừa hư từ nhân quả 'Chính vì... nên' (lặp ý nghĩa).
pub fn evaluate_redundant_cause(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for cap in REDUNDANT_CAUSE_RE.captures_iter(&norm) {
        if let Some(m) = cap.get(1) {
            let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
            let span = norm[m.start()..m.end()].to_string();
            let repl = span.replace("Chính vì", "Vì").replace("chính vì", "vì");

            findings.push(
                Finding::new(
                    SYNTAX_REDUNDANT_CAUSE_RULE_ID,
                    "syntax",
                    "minor",
                    start_char,
                    end_char,
                    0.9,
                    "Thừa hư từ: 'Chính vì... nên' là lặp lại ý nghĩa nhân quả. Nên rút gọn thành 'Vì... nên' hoặc bỏ 'nên'.",
                    "linguistic",
                    span,
                )
                .with_note_and_fix(
                    "Đổi 'Chính vì... nên' thành 'Vì... nên'.",
                    start_char,
                    end_char,
                    repl,
                ),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_reporting() {
        let text = "Phải nói là một quyết định sáng suốt.";
        let findings = evaluate_syntax_reporting(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, SYNTAX_REPORTING_RULE_ID);
    }

    #[test]
    fn test_syntax_parallel() {
        let text = "Một kế hoạch hoàn hảo nhưng không khả thi.";
        let findings = evaluate_syntax_parallel(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, SYNTAX_PARALLEL_RULE_ID);
    }

    #[test]
    fn test_syntax_cliche() {
        let text = "Như chúng ta đã biết rằng việc này rất khó.";
        let findings = evaluate_syntax_cliche(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, SYNTAX_CLICHE_RULE_ID);
    }
}
