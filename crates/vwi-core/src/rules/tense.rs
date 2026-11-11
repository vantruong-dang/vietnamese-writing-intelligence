//! # XUNG ĐỘT THÌ QUÁ KHỨ / TƯƠNG LAI (ADVISORY)
//!
//! "Hôm qua tôi sẽ đi chợ" là câu sai thì. Luật chỉ bắt khi:
//! - có trạng ngữ quá khứ rõ ràng (`hôm qua`, `tuần trước`, ...),
//! - theo sau là `sẽ` trong **cùng một vế ngắn** (≤ 15 ký tự, không có dấu
//!   phẩy/chấm), và
//! - không có động từ tường thuật (`biết`, `nói`, `tin`, `hy vọng`, `hứa`).
//!
//! Nhờ vậy câu tường thuật "Hôm qua ông ấy nói ngày mai trời sẽ mưa" không bị
//! bắt oan. Luôn `Advisory` (0 điểm phạt).

use crate::types::Finding;
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};
use regex::Regex;
use std::sync::LazyLock;

pub const PAST_FUTURE_CONFLICT_RULE_ID: &str = "PAST_FUTURE_TENSE_CONFLICT";

static PAST_FUTURE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(hôm qua|hôm kia|tuần trước|tháng trước|năm ngoái)\b([^.!?,;]{0,15}?)\b(sẽ)\b",
    )
    .expect("Regex PAST_FUTURE_TENSE_CONFLICT không hợp lệ")
});

static FUTURE_PAST_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(ngày mai|tuần sau|tháng sau|năm sau|sắp tới)\b([^.!?,;]{0,15}?)\b(đã)\b")
        .expect("Regex FUTURE_PAST_TENSE_CONFLICT không hợp lệ")
});

static REPORTING_VERBS: &[&str] = &[
    "biết",
    "nói",
    "cho biết",
    "tin",
    "hy vọng",
    "hứa",
    "dự định",
    "kế hoạch",
];

pub fn evaluate_past_future_conflict(raw_text: &str) -> Vec<Finding> {
    let normalized = normalize_nfc(raw_text);
    let map = CharOffsetMap::new(&normalized);
    let mut findings = Vec::new();

    for cap in PAST_FUTURE_RE.captures_iter(&normalized) {
        let gap = cap.get(2).unwrap().as_str().to_lowercase();
        if REPORTING_VERBS.iter().any(|verb| gap.contains(verb)) {
            continue;
        }

        let se = cap.get(3).unwrap();
        let (start, end) = map.byte_range_to_char_range(se.start(), se.end());
        findings.push(
            Finding::new(
                PAST_FUTURE_CONFLICT_RULE_ID,
                "syntax",
                "advisory",
                start,
                end,
                0.78,
                "Trạng ngữ chỉ quá khứ đi với 'sẽ' (tương lai) — có thể sai thì.",
                "tense",
                se.as_str(),
            )
            .with_fix(start, end, "đã")
            .with_stage("regex"),
        );
    }

    for cap in FUTURE_PAST_RE.captures_iter(&normalized) {
        let gap = cap.get(2).unwrap().as_str().to_lowercase();
        if REPORTING_VERBS.iter().any(|verb| gap.contains(verb)) {
            continue;
        }
        let da = cap.get(3).unwrap();
        let (start, end) = map.byte_range_to_char_range(da.start(), da.end());
        findings.push(
            Finding::new(
                PAST_FUTURE_CONFLICT_RULE_ID,
                "syntax",
                "advisory",
                start,
                end,
                0.78,
                "Trạng ngữ chỉ tương lai đi với 'đã' (quá khứ) — có thể sai thì.",
                "tense",
                da.as_str(),
            )
            .with_fix(start, end, "sẽ")
            .with_stage("regex"),
        );
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spans(text: &str) -> Vec<String> {
        evaluate_past_future_conflict(text)
            .into_iter()
            .map(|f| f.matched_span)
            .collect()
    }

    #[test]
    fn test_flags_past_marker_with_future_marker() {
        assert_eq!(spans("Hôm qua tôi sẽ đi chợ."), vec!["sẽ"]);
        assert_eq!(spans("Tuần trước công ty sẽ gửi hồ sơ."), vec!["sẽ"]);
    }

    #[test]
    fn test_flags_future_marker_with_past_marker() {
        assert_eq!(spans("Ngày mai tôi đã gặp thầy giáo."), vec!["đã"]);
    }

    #[test]
    fn test_ignores_reported_speech_and_correct_tense() {
        assert!(spans("Hôm qua ông ấy nói ngày mai trời sẽ mưa.").is_empty());
        assert!(spans("Hôm qua tôi đã đi chợ.").is_empty());
        assert!(spans("Ngày mai tôi sẽ đi chợ.").is_empty());
    }

    #[test]
    fn test_always_advisory() {
        for f in evaluate_past_future_conflict("Hôm qua tôi sẽ đi chợ.") {
            assert_eq!(f.severity, "advisory");
        }
    }
}
