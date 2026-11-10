use vwi_core::auditor::Auditor;

// Hàm tiện ích lấy danh sách rule_id từ văn bản
fn get_rule_ids(text: &str) -> Vec<String> {
    let auditor = Auditor::default();
    let report = auditor.audit_document(text);
    report.findings.into_iter().map(|f| f.rule_id).collect()
}

#[cfg(test)]
mod table_a_false_positive_traps {
    use super::*;

    #[test]
    fn test_ca_01_abbreviations_and_titles() {
        let text = "Theo GS.TS. Nguyễn Văn Nam, tại TP.HCM và các tỉnh lân cận, v.v. đều ghi nhận mật độ giao thông tăng cao tại Q.1 và P. Bến Nghé.";
        let findings = get_rule_ids(text);
        assert!(findings.is_empty(), "Bắt oan viết tắt/địa danh: {:?}", findings);
    }

    #[test]
    fn test_ca_02_time_ratio_and_decimal() {
        let text = "Trận đấu bắt đầu lúc 19:30 với tỷ số 2:1 trên sân Mỹ Đình. Dự án quy hoạch tỷ lệ 1:5000 có tổng mức đầu tư 1,250,000,000 đồng, dung sai 3.14%.";
        let findings = get_rule_ids(text);
        assert!(findings.is_empty(), "Bắt oan số liệu/giờ giấc: {:?}", findings);
    }

    #[test]
    fn test_ca_03_legal_list_lowercase() {
        let text = "Điều 1. Phạm vi điều chỉnh và đối tượng áp dụng:\na) Cán bộ, công chức tại các cơ quan hành chính nhà nước;\nb) Viên chức trong các đơn vị sự nghiệp công lập.";
        let findings = get_rule_ids(text);
        assert!(findings.is_empty(), "Bắt oan cấu trúc liệt kê a), b): {:?}", findings);
    }

    #[test]
    fn test_ca_04_brackets_and_quotes_spacing() {
        let text = "Nguyễn Du dùng chữ \"thì\" (trong Truyện Kiều) rất tài tình... Đó chính là tác tử phân giới Đề - Thuyết của câu.";
        let findings = get_rule_ids(text);
        assert!(findings.is_empty(), "Bắt oan ngoặc kép/ba chấm: {:?}", findings);
    }

    #[test]
    fn test_ca_05_url_and_code_masking() {
        let text = "Tham khảo tài liệu tại https://chinhphu.vn/nghi-dinh-30. Khi lập trình hãy gọi hàm `process_data(x, y)` để kiểm tra.";
        let findings = get_rule_ids(text);
        assert!(findings.is_empty(), "Bắt oan URL/Code block: {:?}", findings);
    }
}

#[cfg(test)]
mod table_b_true_positives {
    use super::*;

    #[test]
    fn test_ca_06_punctuation_spacing() {
        let text = "Hội nghị đã kết thúc tốt đẹp ,mở ra nhiều cơ hội hợp tác .tuy nhiên vẫn còn nhiều thách thức .";
        let findings = get_rule_ids(text);
        
        assert!(findings.contains(&"PUNCTUATION_SPACE_BEFORE".to_string()), "Bỏ sót lỗi thừa dấu cách trước dấu câu");
        assert!(findings.contains(&"PUNCTUATION_NO_SPACE_AFTER".to_string()), "Bỏ sót lỗi thiếu dấu cách sau dấu câu");
        assert!(findings.contains(&"SENTENCE_CAPITALIZATION".to_string()), "Bỏ sót lỗi không viết hoa sau dấu chấm");
    }

    #[test]
    fn test_ca_07_bracket_spacing() {
        let text = "Dự án đường vành đai 4 ( khởi công năm 2023 ) có ý nghĩa chiến lược đối với \" vùng thủ đô \" .";
        let findings = get_rule_ids(text);
        
        assert!(findings.contains(&"BRACKET_SPACE_INSIDE".to_string()), "Bỏ sót lỗi thừa dấu cách trong ngoặc");
    }

    #[test]
    fn test_ca_08_decree30_capitalization() {
        let text = "Hôm qua, Bộ Giáo Dục Và Đào Tạo đã có công văn gửi Ủy Ban Nhân Dân Tỉnh Quảng Ninh về việc tuyển sinh.";
        let findings = get_rule_ids(text);
        
        assert!(findings.contains(&"DECREE30_AGENCY_CAPITALIZATION".to_string()), "Bỏ sót lỗi viết hoa cơ quan");
        assert!(findings.contains(&"DECREE30_ADMIN_UNIT_LOWERCASE".to_string()), "Bỏ sót lỗi viết hoa đơn vị hành chính");
    }

    #[test]
    fn test_ca_09_phonotactic_telex_typos() {
        let text = "Doanh nghiệp dduowjc yêu cầu cung cấp đầỳ đủ hồ sơ tài chính trwowngf hợp có sai sót.";
        let findings = get_rule_ids(text);
        
        // Đếm số lỗi TELEX_TYPO_ERROR (Kỳ vọng bắt được ít nhất dduowjc và trwowngf)
        let telex_errors = findings.iter().filter(|id| *id == "TELEX_TYPO_ERROR").count();
        assert!(telex_errors >= 2, "Bỏ sót lỗi gõ Telex (dduowjc, trwowngf). Tìm thấy: {}", telex_errors);
    }

    #[test]
    fn test_ca_10_tone_inconsistency() {
        let text = "Chính phủ chủ trương xây dựng nền kinh tế hòa nhập quốc tế, đồng thời giữ vững môi trường hoà bình và ổn định lâu dài.";
        let findings = get_rule_ids(text);
        
        assert!(findings.contains(&"ORTHOGRAPHY_TONE_INCONSISTENCY".to_string()), "Bỏ sót lỗi bất nhất quán dấu thanh (hòa vs hoà)");
    }
}
