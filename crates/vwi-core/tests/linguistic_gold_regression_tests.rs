use vwi_core::auditor::Auditor;

fn check_rule(text: &str, expected_rule: &str) {
    let auditor = Auditor::default();
    let report = auditor.audit_document(text);
    let findings: Vec<String> = report.findings.into_iter().map(|f| f.rule_id).collect();

    assert!(
        findings.contains(&expected_rule.to_string()),
        "❌ BỎ SÓT LỖI: Không bắt được rule '{}' trong câu: '{}'. Các lỗi tìm thấy: {:?}",
        expected_rule, text, findings
    );
}

#[cfg(test)]
mod cao_xuan_hao_classics {
    use super::*;

    #[test]
    fn test_01_crippled_topic_trong() {
        check_rule("Trong tình hình này đòi hỏi khắc phục ngay các tồn tại.", "CRIPPLED_PREPOSITIONAL_TOPIC");
    }

    #[test]
    fn test_02_prep_conj_conflict() {
        check_rule("Với những cố gắng vượt bậc nhưng họ vẫn không giải quyết được dứt điểm.", "PREPOSITION_CONJUNCTION_CONFLICT");
    }

    #[test]
    fn test_03_crippled_topic_bang() {
        check_rule("Bằng những hình tượng sắc nét của tác giả đã cho thấy tài năng miêu tả tâm lý.", "CRIPPLED_PREPOSITIONAL_TOPIC");
    }

    #[test]
    fn test_04_factual_concessive_tuy() {
        check_rule("Ngày mai, tuy trời mưa to, tôi cũng sẽ đến.", "FACTUAL_CONCESSIVE_MISUSE");
        check_rule("Sau này tuy có giàu con cũng đừng hoang phí.", "FACTUAL_CONCESSIVE_MISUSE");
    }

    #[test]
    fn test_05_hypothetical_concessive_du() {
        check_rule("Hôm qua, dù trời mưa to, tôi vẫn đến.", "HYPOTHETICAL_CONCESSIVE_MISUSE");
    }

    #[test]
    fn test_06_temporal_translationese() {
        check_rule("John sẽ về trong ba ngày.", "TEMPORAL_IN_TRANSLATIONESE");
    }

    #[test]
    fn test_07_redundant_beneficiary() {
        check_rule("Tóc nó được cắt ngắn, cái bàn bị lau chưa sạch.", "REDUNDANT_BENEFICIARY_PASSIVE");
    }

    #[test]
    fn test_08_artificial_passive_by() {
        check_rule("Bộ phim này được tài trợ bởi một tập đoàn lớn.", "ARTIFICIAL_PASSIVE_BE_BY");
        check_rule("Ất bị đánh bởi Giáp.", "ARTIFICIAL_PASSIVE_BE_BY");
    }

    #[test]
    fn test_09_logical_pleonasm() {
        check_rule("Đây là giải pháp kỹ thuật tối ưu nhất, giúp đem lại ánh nắng mặt trời cho căn nhà.", "LOGICAL_PLEONASM");
    }

    #[test]
    fn test_10_chi_it_misuse() {
        check_rule("Để hoàn thành kế hoạch, chí ít chúng ta cần thêm ba tháng nữa.", "CXH_SUPERLATIVE_CHI_IT_MISUSE");
    }

    #[test]
    fn test_11_reporting_verb_incomplete() {
        check_rule("Phải nói là cú sút rất điệu nghệ của tiền đạo.", "CXH_REPORTING_VERB_INCOMPLETE_CLAUSE");
    }

    #[test]
    fn test_12_hybrid_complex_conjunction() {
        check_rule("Như các bạn đã biết là tiền đạo này đã bị chấn thương.", "CXH_HYBRID_COMPLEX_CONJUNCTION");
    }
}
