use crate::segmentation::tokenize_viterbi;
use crate::types::Finding;
use crate::unicode_utils::{normalize_nfc, CharOffsetMap};
use regex::RegexBuilder;
use std::sync::LazyLock;

static FACTUAL_CONCESSIVE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(
        r"(?:\b(ngày mai|ngày mốt|tuần tới|tuần sau|tháng sau|năm sau|sang năm|sắp tới|sau này|tương lai)[,\s]+(tuy|mặc dù|mặc dầu)\b|(?:\b|^)(tuy|mặc dù|mặc dầu)\s+(?:ngày mai|ngày mốt|tuần tới|tuần sau|tháng sau|năm sau|sang năm|sắp tới|mai|mai đây)\b|\b(tuy|mặc dù|mặc dầu)\b[^,.]+?\b(đi chăng nữa|chăng nữa|đến mấy|thế nào chăng nữa)\b)",
    )
    .case_insensitive(true)
    .build()
    .expect("Regex FACTUAL_CONCESSIVE_MISUSE không hợp lệ")
});

static HYPOTHETICAL_TIME_ANCHOR_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(
        r"(?i)(?:\b(hôm qua|hôm kia|hôm trước|năm ngoái|năm trước|tuần trước|tháng trước|vừa qua|trước đây|xưa nay|thực tế là|rõ ràng là)[,\s]+dù\b|(?:\b|^)dù\s+(?:hôm qua|hôm kia|hôm trước|năm ngoái|năm trước|tuần trước|tháng trước|vừa qua|trước đây|xưa nay)\b)",
    )
    .build()
    .expect("Regex HYPOTHETICAL_TIME_ANCHOR_RE không hợp lệ")
});

static HYPOTHETICAL_IDIOMATIC_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(
        r"(?i)(?:\b|^)dù\s+(?:tuổi đã cao|đã già|đã lớn tuổi|đã từng|vốn dĩ|vốn là|sẵn có|hiển nhiên)\b",
    )
    .build()
    .expect("Regex HYPOTHETICAL_IDIOMATIC_RE không hợp lệ")
});

static CONCESSIVE_INVERSION_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    RegexBuilder::new(r"\b(mặc\s+dù|dẫu\s+rằng|cho\s+dù)\b([^,.!?]+?)\b(tuy\s+nhiên|nhưng\s+mà)\b")
        .case_insensitive(true)
        .build()
        .expect("Regex CONCESSIVE_INVERSION_CONFLICT không hợp lệ")
});

pub const FACTUAL_CONCESSIVE_RULE_ID: &str = "FACTUAL_CONCESSIVE_MISUSE";
pub const HYPOTHETICAL_CONCESSIVE_RULE_ID: &str = "HYPOTHETICAL_CONCESSIVE_MISUSE";
pub const CONCESSIVE_INVERSION_RULE_ID: &str = "CONCESSIVE_INVERSION_CONFLICT";

/// Danh sách danh từ trừu tượng / xu hướng kinh tế - xã hội.
/// Trong các mệnh đề trạng thái như "...dù đà tăng trưởng đã chậm lại...",
/// cấu trúc này diễn đạt sự nhượng bộ bối cảnh/xu hướng, không phải hành động chủ thể cố ý trong quá khứ.
const ABSTRACT_TREND_NOUNS: &[&str] = &[
    "đà",
    "tốc độ",
    "áp lực",
    "lực",
    "tỷ lệ",
    "tỉ lệ",
    "mức",
    "chỉ số",
    "giá",
    "giá cả",
    "lãi suất",
    "lạm phát",
    "nguồn cung",
    "thị trường",
    "ngân sách",
    "kim ngạch",
    "tiến độ",
    "dự án",
    "kinh tế",
];

/// Kiểm tra xem trước vị trí byte `pos` trong chuỗi `text` có phải là liên từ ghép ngoại lệ
/// như "mặc dù", "cho dù", "dẫu dù" hay không.
fn is_preceded_by_concessive_particle(text: &str, pos: usize) -> bool {
    let prefix = text[..pos].trim_end();
    let lower_prefix = prefix.to_lowercase();
    lower_prefix.ends_with("mặc")
        || lower_prefix.ends_with("cho")
        || lower_prefix.ends_with("dẫu")
        || lower_prefix.ends_with("nếu")
        || lower_prefix.ends_with("mà")
}

/// Đánh giá lỗi Nhượng bộ Hiện thực dùng cho Giả định tương lai (Factual Concessive Misuse).
pub fn evaluate_factual_concessive_misuse(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();

    for m in FACTUAL_CONCESSIVE_RE.find_iter(&norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        findings.push(
            Finding::new(
                FACTUAL_CONCESSIVE_RULE_ID,
                "concessive_aspect",
                "major",
                start_char,
                end_char,
                1.0,
                "Sự việc tương lai giả định (phi hiện thực). 'Tuy' chỉ dùng cho sự việc đã/đang diễn ra; giả định bắt buộc dùng 'Dù'.",
                "linguistic",
                norm[m.start()..m.end()].to_string(),
            )
            .with_suggestion("Đổi liên từ 'Tuy/Mặc dù' thành 'Dù/Cho dù' đối với sự việc giả định hoặc tương lai"),
        );
    }

    findings
}

/// Đánh giá lỗi Nhượng bộ Giả định dùng cho Hiện thực quá khứ (Hypothetical Concessive Misuse).
/// Sử dụng phương pháp kết hợp:
/// 1. Anchor thời gian quá khứ (Past Time Anchors): bắt chính xác tọa độ theo chuẩn Benchmark ("Hôm qua, dù", v.v.)
///    nhưng bảo vệ tuyệt đối "mặc dù", "cho dù", "dẫu dù".
/// 2. Thành ngữ / Cố định ("dù đã già", "dù tuổi đã cao").
/// 3. Phân tích ngữ cảnh POS: Nhận diện "dù [Chủ thể] đã/từng [Hành động]" nhưng miễn trừ
///    các danh từ trạng thái/xu hướng trừu tượng ("đà", "tốc độ", "áp lực", "lạm phát", ...).
pub fn evaluate_hypothetical_concessive_misuse(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    let mut findings = Vec::new();
    let mut matched_ranges: Vec<(usize, usize)> = Vec::new();

    // 1. Nhóm Anchor thời gian quá khứ / thực tế xác định (Bảo toàn 100% tọa độ cho Gold Benchmark)
    for m in HYPOTHETICAL_TIME_ANCHOR_RE.find_iter(&norm) {
        // Kiểm tra bảo vệ "mặc dù" / "cho dù"
        let matched_text = m.as_str().to_lowercase();
        if matched_text.starts_with("dù") && is_preceded_by_concessive_particle(&norm, m.start()) {
            continue;
        }

        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        matched_ranges.push((start_char, end_char));
        findings.push(
            Finding::new(
                HYPOTHETICAL_CONCESSIVE_RULE_ID,
                "concessive_aspect",
                "major",
                start_char,
                end_char,
                0.95,
                "Sự việc quá khứ là hiện thực xác định. Bắt buộc dùng 'Tuy/Mặc dù', không dùng liên từ giả định 'Dù'.",
                "linguistic",
                norm[m.start()..m.end()].to_string(),
            )
            .with_suggestion("Đổi liên từ 'Dù' thành 'Tuy/Mặc dù' đối với sự việc đã diễn ra trong thực tế"),
        );
    }

    // 2. Nhóm thành ngữ / trạng thái cố định ("Dù đã già", "Dù tuổi đã cao", v.v.)
    for m in HYPOTHETICAL_IDIOMATIC_RE.find_iter(&norm) {
        if is_preceded_by_concessive_particle(&norm, m.start()) {
            continue;
        }

        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        if matched_ranges.iter().any(|(s, e)| *s <= start_char && end_char <= *e) {
            continue;
        }

        matched_ranges.push((start_char, end_char));
        findings.push(
            Finding::new(
                HYPOTHETICAL_CONCESSIVE_RULE_ID,
                "concessive_aspect",
                "major",
                start_char,
                end_char,
                0.95,
                "Sự việc quá khứ là hiện thực xác định. Bắt buộc dùng 'Tuy/Mặc dù', không dùng liên từ giả định 'Dù'.",
                "linguistic",
                norm[m.start()..m.end()].to_string(),
            )
            .with_suggestion("Đổi liên từ 'Dù' thành 'Tuy/Mặc dù' đối với sự việc đã diễn ra trong thực tế"),
        );
    }

    // 3. Phân tích ngữ cảnh POS (POS-Aware Context Scanner):
    // Quét "dù" + [Chủ thể] + "đã/từng"
    let tokens = tokenize_viterbi(&norm);
    let n = tokens.len();

    for i in 0..n {
        let t = &tokens[i];
        if t.text.eq_ignore_ascii_case("dù") {
            // Loại trừ nếu đứng sau "mặc", "cho", "dẫu"
            if i > 0 {
                let prev_text = tokens[i - 1].text.to_lowercase();
                if prev_text == "mặc" || prev_text == "cho" || prev_text == "dẫu" || prev_text == "nếu" || prev_text == "mà" {
                    continue;
                }
            } else if is_preceded_by_concessive_particle(&norm, t.start) {
                continue;
            }

            // Quét các token phía sau (trong cùng một vế câu, dừng nếu gặp dấu phẩy, chấm, chấm phẩy)
            let mut found_past_marker_idx: Option<usize> = None;
            let mut has_abstract_noun = false;

            for j in (i + 1)..n.min(i + 12) {
                let sub_token = &tokens[j];
                let sub_text = sub_token.text.to_lowercase();

                if sub_text == "," || sub_text == "." || sub_text == ";" || sub_text == "!" || sub_text == "?" {
                    break;
                }

                // Kiểm tra xem có chứa danh từ trừu tượng / xu hướng không
                if ABSTRACT_TREND_NOUNS.iter().any(|&noun| sub_text == noun) {
                    has_abstract_noun = true;
                }

                // Kiểm tra mốc thời gian thể quá khứ "đã" hoặc "từng"
                if sub_text == "đã" || sub_text == "từng" {
                    found_past_marker_idx = Some(j);
                    break;
                }
            }

            // Nếu phát hiện có "đã/từng" nhưng KHÔNG PHẢI danh từ trừu tượng/xu hướng
            if let Some(past_idx) = found_past_marker_idx {
                if !has_abstract_noun {
                    let start_char = t.start;
                    let end_char = tokens[past_idx].end;

                    let already_covered = matched_ranges
                        .iter()
                        .any(|(s, e)| (*s <= start_char && end_char <= *e) || (start_char <= *s && *e <= end_char));

                    if !already_covered {
                        matched_ranges.push((start_char, end_char));
                        // Lấy substring theo character offset
                        let matched_span: String = norm.chars().skip(start_char).take(end_char - start_char).collect();
                        findings.push(
                            Finding::new(
                                HYPOTHETICAL_CONCESSIVE_RULE_ID,
                                "concessive_aspect",
                                "major",
                                start_char,
                                end_char,
                                0.95,
                                "Sự việc quá khứ là hiện thực xác định. Bắt buộc dùng 'Tuy/Mặc dù', không dùng liên từ giả định 'Dù'.",
                                "linguistic",
                                matched_span,
                            )
                            .with_suggestion("Đổi liên từ 'Dù' thành 'Tuy/Mặc dù' đối với sự việc đã diễn ra trong thực tế"),
                        );
                    }
                }
            }
        }
    }

    findings
}

/// Đánh giá lỗi xung đột nhượng bộ kép "Mặc dù/Dẫu rằng ... tuy nhiên/nhưng mà".
pub fn evaluate_concessive_inversion(text: &str) -> Vec<Finding> {
    let norm = normalize_nfc(text);
    let mapper = CharOffsetMap::new(&norm);
    evaluate_concessive_inversion_with_map(&norm, &mapper)
}

/// Phiên bản tối ưu nhận sẵn CharOffsetMap.
pub fn evaluate_concessive_inversion_with_map(norm: &str, mapper: &CharOffsetMap) -> Vec<Finding> {
    let mut findings = Vec::new();

    for m in CONCESSIVE_INVERSION_RE.find_iter(norm) {
        let (start_char, end_char) = mapper.convert_range(m.start(), m.end());
        findings.push(
            Finding::new(
                CONCESSIVE_INVERSION_RULE_ID,
                "concessive",
                "major",
                start_char,
                end_char,
                0.95,
                "Xung đột nhượng bộ kép ('Mặc dù... tuy nhiên'). Đã có 'mặc dù' thì không dùng 'tuy nhiên' ở vế đối lập.",
                "linguistic",
                norm[m.start()..m.end()].to_string(),
            )
            .with_suggestion("Chỉ dùng 'Mặc dù... nhưng...' hoặc lược bỏ 'Mặc dù' và tách câu dùng 'Tuy nhiên'"),
        );
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factual_concessive_future() {
        let text = "Ngày mai, tuy trời mưa to, tôi cũng sẽ đến đúng giờ.";
        let findings = evaluate_factual_concessive_misuse(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, FACTUAL_CONCESSIVE_RULE_ID);
        assert_eq!(findings[0].matched_span, "Ngày mai, tuy");
        assert_eq!(findings[0].confidence, 1.0);
    }

    #[test]
    fn test_hypothetical_concessive_past() {
        let text = "Hôm qua, dù trời mưa rất to, trận đấu vẫn diễn ra sôi nổi.";
        let findings = evaluate_hypothetical_concessive_misuse(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, HYPOTHETICAL_CONCESSIVE_RULE_ID);
        assert_eq!(findings[0].matched_span, "Hôm qua, dù");
    }

    #[test]
    fn test_concessive_inversion() {
        let text = "Mặc dù anh ấy rất cố gắng tuy nhiên kết quả vẫn chưa đạt.";
        let findings = evaluate_concessive_inversion(text);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_id, CONCESSIVE_INVERSION_RULE_ID);
    }

    #[test]
    fn test_bypass_mac_du_and_abstract_trend() {
        // 1. Mặc dù không bao giờ bị bắt lỗi này
        let mac_du_text = "Mặc dù hôm qua trời mưa to, tôi vẫn đến.";
        let findings_mac_du = evaluate_hypothetical_concessive_misuse(mac_du_text);
        assert!(findings_mac_du.is_empty(), "Không được bắt lỗi 'Mặc dù'");

        // 2. Mệnh đề trạng thái xu hướng trừu tượng
        let trend_text = "...dù đà tăng trưởng đã chậm lại đôi chút...";
        let findings_trend = evaluate_hypothetical_concessive_misuse(trend_text);
        assert!(findings_trend.is_empty(), "Không được bắt lỗi câu xu hướng trừu tượng");

        // 3. True positive câu hành động con người vẫn phải bắt
        let human_text = "Dù anh ấy đã cố gắng rất nhiều, kết quả vẫn không như ý.";
        let findings_human = evaluate_hypothetical_concessive_misuse(human_text);
        println!("findings_human: {:?}", findings_human);
        assert_eq!(findings_human.len(), 1);
        assert_eq!(findings_human[0].matched_span, "Dù anh ấy đã");
    }
}
