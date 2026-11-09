use crate::rules::artificial_passive::{
    evaluate_artificial_passive_fast, evaluate_redundant_beneficiary_passive,
};
use crate::rules::aspect::evaluate_aspect_mechanics;
use crate::rules::bureaucracy::evaluate_bureaucratic_rules;
use crate::rules::cliche::{
    evaluate_ai_opening_cliche, evaluate_ai_summary_tail, evaluate_formulaic_metaphor,
};
use crate::rules::concessive::{
    evaluate_concessive_inversion, evaluate_factual_concessive_misuse,
    evaluate_hypothetical_concessive_misuse,
};
use crate::rules::lexical::{evaluate_superlative_misuse, evaluate_suyt_nua_thi};
use crate::rules::pleonasm::evaluate_logical_pleonasm;
use crate::rules::syntax::{
    evaluate_syntax_cliche, evaluate_syntax_parallel, evaluate_syntax_reporting,
};
use crate::rules::topic_comment::{
    evaluate_crippled_prepositional_topic, evaluate_preposition_conjunction_conflict,
};
use crate::rules::translationese::{
    evaluate_dummy_it_translationese, evaluate_nominalization_heavy, evaluate_temporal_framing,
    evaluate_translationese_soleil,
};
use crate::types::Finding;

/// Động cơ cảm biến ngôn ngữ học tổng hợp (Linguistic Sensor Engine) thuần Rust.
/// Chạy toàn bộ các luật ngữ pháp chức năng, sao phỏng Tây ngữ, đệm từ hành chính
/// và trả về danh sách các vi phạm được sắp xếp theo vị trí ký tự.
pub fn audit_linguistics(text: &str) -> Vec<Finding> {
    let mut all_findings = Vec::new();

    // 1. Nhóm bị động nhân tạo & thụ hưởng dư thừa
    all_findings.extend(evaluate_artificial_passive_fast(text));
    all_findings.extend(evaluate_redundant_beneficiary_passive(text));

    // 2. Nhóm cấu trúc Khung đề - Chủ đề
    all_findings.extend(evaluate_crippled_prepositional_topic(text));
    all_findings.extend(evaluate_preposition_conjunction_conflict(text));

    // 3. Nhóm trùng ngữ logic
    all_findings.extend(evaluate_logical_pleonasm(text));

    // 4. Nhóm dịch ngữ & danh từ hóa & chủ ngữ giả
    all_findings.extend(evaluate_nominalization_heavy(text));
    all_findings.extend(evaluate_temporal_framing(text));
    all_findings.extend(evaluate_translationese_soleil(text));
    all_findings.extend(evaluate_dummy_it_translationese(text));

    // 5. Nhóm liên từ nhượng bộ, xung đột kép & thời thể
    all_findings.extend(evaluate_factual_concessive_misuse(text));
    all_findings.extend(evaluate_hypothetical_concessive_misuse(text));
    all_findings.extend(evaluate_concessive_inversion(text));
    all_findings.extend(evaluate_aspect_mechanics(text));

    // 6. Nhóm cú pháp đối ngẫu & trần thuật
    all_findings.extend(evaluate_syntax_reporting(text));
    all_findings.extend(evaluate_syntax_parallel(text));
    all_findings.extend(evaluate_syntax_cliche(text));

    // 7. Nhóm từ vựng lai căng
    all_findings.extend(evaluate_superlative_misuse(text));
    all_findings.extend(evaluate_suyt_nua_thi(text));

    // 8. Nhóm công thức & sáo ngữ AI (Humanizer)
    all_findings.extend(evaluate_ai_opening_cliche(text));
    all_findings.extend(evaluate_ai_summary_tail(text));
    all_findings.extend(evaluate_formulaic_metaphor(text));

    // 9. Nhóm sáo ngữ hành chính & đệm từ rỗng (Aho-Corasick O(N))
    all_findings.extend(evaluate_bureaucratic_rules(text));

    // Sắp xếp các phát hiện theo thứ tự xuất hiện trong văn bản
    all_findings.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| a.end.cmp(&b.end)));
    all_findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_linguistics_multiple_findings() {
        let text = "Trong tình hình hiện nay đòi hỏi giải pháp tối ưu nhất được triển khai bởi nhóm.";
        let findings = audit_linguistics(text);

        // Phải bắt được cả 3 lỗi: Khung đề què, Tối ưu nhất, Bị động nhân tạo
        assert!(findings.len() >= 3);
        let ids: Vec<&str> = findings.iter().map(|f| f.pattern_id.as_str()).collect();
        assert!(ids.contains(&"CRIPPLED_PREPOSITIONAL_TOPIC"));
        assert!(ids.contains(&"LOGICAL_PLEONASM"));
        assert!(ids.contains(&"ARTIFICIAL_PASSIVE_BE_BY"));
    }

    #[test]
    fn test_audit_linguistics_all_categories() {
        let text = "Không thể phủ nhận rằng ngày mai, tuy trời mưa nhưng dự án đã và đang là chiếc chìa khóa vạn năng.";
        let findings = audit_linguistics(text);
        let ids: Vec<&str> = findings.iter().map(|f| f.pattern_id.as_str()).collect();

        assert!(ids.contains(&"AI_OPENING_CLICHE"));
        assert!(ids.contains(&"FACTUAL_CONCESSIVE_MISUSE"));
        assert!(ids.contains(&"CXH_VERBAL_SERIES_TENSE_FALSE_MARKER"));
        assert!(ids.contains(&"FORMULAIC_METAPHOR"));
    }

    #[test]
    fn test_audit_linguistics_bureaucracy_and_dummy_it() {
        let text = "Nó là rất quan trọng để tiến hành thực hiện phương án mang tính chất đột phá này.";
        let findings = audit_linguistics(text);
        let ids: Vec<&str> = findings.iter().map(|f| f.pattern_id.as_str()).collect();

        assert!(ids.contains(&"DUMMY_IT_TRANSLATIONESE"));
        assert!(ids.contains(&"BUREAUCRATIC_VERB_PADDING"));
        assert!(ids.contains(&"ADJECTIVAL_SUFFIX_FAKE"));
    }
}
