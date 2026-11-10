use vwi_core::auditor::{Auditor, AuditorConfig};
use vwi_core::types::Severity;

#[test]
fn test_crippled_topic_thorough_matrix() {
    let auditor = Auditor::default();

    // 1. Các trường hợp vi phạm KHUNG ĐỀ QUÈ kinh điển
    let violations = vec![
        "Trong bối cảnh nền kinh tế hiện nay đòi hỏi các doanh nghiệp phải chuyển mình mạnh mẽ.",
        "Qua kết quả khảo sát thực tế cho thấy sự hài lòng của khách hàng đã tăng 15%.",
        "Bằng các chính sách tiền tệ linh hoạt đã giúp kiềm chế lạm phát hiệu quả.",
        "Với sự nỗ lực không ngừng nghỉ của toàn thể cán bộ nhân viên đã mang lại doanh thu kỷ lục.",
        "Ở khu vực vùng sâu vùng xa đòi hỏi ngân sách đầu tư hạ tầng giao thông lớn hơn.",
        "Theo nhận định của các chuyên gia hàng đầu cảnh báo rủi ro biến động tỷ giá.",
    ];

    for text in violations {
        let report = auditor.audit_document(text);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.pattern_id == "CRIPPLED_PREPOSITIONAL_TOPIC"),
            "Phải bắt được lỗi CRIPPLED_PREPOSITIONAL_TOPIC trong câu: \"{}\"",
            text
        );
        assert!(report.score < 100.0);
    }

    // 2. Các trường hợp HỢP LỆ (False Positive Guards)
    let valid_sentences = vec![
        // Động từ tồn tại / sở hữu
        "Trong phòng họp có mười thành viên đang chăm chú lắng nghe.",
        "Trên bàn còn lại ba bản tài liệu chưa ký.",
        "Ngoài sân xuất hiện nhiều dấu vết lạ.",
        "Ở Việt Nam tồn tại nhiều di sản văn hóa phi vật thể quý báu.",
        // Có dấu phẩy ngăn cách Đề ngữ / Trạng ngữ với Thuyết ngữ có Chủ ngữ riêng
        "Trong bối cảnh khó khăn, tập thể chúng ta đã xuất sắc vượt qua chỉ tiêu.",
        "Qua khảo sát này, ban giám đốc nhận thấy nhiều cơ hội mở rộng thị trường.",
        "Với quyết tâm cao, toàn đội tự tin bước vào trận chung kết.",
        "Bằng công nghệ mới, nhà máy đã giảm 30% lượng khí thải độc hại.",
        // Có trợ từ nhấn mạnh "thì" làm dấu hiệu Đề ngữ
        "Trong trường hợp này thì phương án A là lựa chọn tối ưu.",
    ];

    for text in valid_sentences {
        let report = auditor.audit_document(text);
        let has_crippled_topic = report
            .findings
            .iter()
            .any(|f| f.pattern_id == "CRIPPLED_PREPOSITIONAL_TOPIC");
        assert!(
            !has_crippled_topic,
            "KHÔNG ĐƯỢC báo lỗi nhầm CRIPPLED_PREPOSITIONAL_TOPIC trong câu hợp lệ: \"{}\"",
            text
        );
    }
}

#[test]
fn test_artificial_passive_matrix() {
    let auditor = Auditor::default();

    // 1. Vi phạm câu bị động nhân tạo "được/bị ... bởi"
    let violations = vec![
        "Đề án này đã được phê duyệt bởi Thủ tướng Chính phủ vào sáng nay.",
        "Bản hợp đồng thương mại vừa được ký kết bởi đại diện hai tập đoàn.",
        "Hệ thống bị tấn công bởi một nhóm hacker ẩn danh.",
        "Chính sách nhân sự mới được công bố bởi phòng hành chính.",
    ];

    for text in violations {
        let report = auditor.audit_document(text);
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.pattern_id == "ARTIFICIAL_PASSIVE_BE_BY"),
            "Phải bắt được lỗi ARTIFICIAL_PASSIVE_BE_BY trong câu: \"{}\"",
            text
        );
    }

    // 2. Hợp lệ: "bởi vì", "bởi do", hoặc "được/bị" tự nhiên
    let valids = vec![
        "Dự án chậm tiến độ bởi vì thời tiết mưa bão kéo dài nhiều tuần.",
        "Doanh số sụt giảm bởi do thiếu hụt nguồn cung chip toàn cầu.",
        "Chúng tôi rất vui khi được phục vụ quý khách hàng mỗi ngày.",
        "Bị nhắc nhở trong buổi họp là bài học quý giá cho nhân viên mới.",
    ];

    for text in valids {
        let report = auditor.audit_document(text);
        let has_passive = report
            .findings
            .iter()
            .any(|f| f.pattern_id == "ARTIFICIAL_PASSIVE_BE_BY");
        assert!(
            !has_passive,
            "KHÔNG ĐƯỢC báo lỗi nhầm ARTIFICIAL_PASSIVE_BE_BY trong câu hợp lệ: \"{}\"",
            text
        );
    }
}

#[test]
fn test_bureaucratic_cliches_and_nominalization() {
    let auditor = Auditor::default();

    let text = "Để tiến hành thực hiện việc tối ưu hóa quy trình, các phòng ban cần triển khai thực hiện việc rà soát nội bộ một cách vô cùng hết sức khẩn trương.";
    let report = auditor.audit_document(text);

    assert!(report.score < 80.0, "Điểm văn bản sáo ngữ hành chính phải dưới 80");
    assert!(
        report.findings.len() >= 2,
        "Phải bắt được ít nhất 2 lỗi sáo ngữ/danh hóa nặng"
    );

    let pattern_ids: Vec<&str> = report.findings.iter().map(|f| f.pattern_id.as_str()).collect();
    println!("Phát hiện các lỗi hành chính: {:?}", pattern_ids);
}

#[test]
fn test_pleonasm_word_redundancy() {
    let auditor = Auditor::default();

    let tests = vec![
        "Phương án này là tối ưu nhất cho quý công ty.",
        "Nguyên nhân là bởi vì chúng ta chưa chuẩn bị kỹ.",
        "Ánh nắng mặt trời buổi sớm rọi sáng căn phòng.",
    ];

    for sentence in tests {
        let report = auditor.audit_document(sentence);
        assert!(
            report.findings.iter().any(|f| f.pattern_id == "LOGICAL_PLEONASM"),
            "Phải bắt được lỗi LOGICAL_PLEONASM trong câu: \"{}\"",
            sentence
        );
    }
}

#[test]
fn test_ai_cliches_and_formulaic_metaphors() {
    let auditor = Auditor::default();

    let text = "Trong thế giới số hóa nhanh chóng ngày nay, chuyển đổi số là ngọn hải đăng soi sáng con đường cho doanh nghiệp. Tóm lại, bài viết này đã làm sáng tỏ bức tranh toàn cảnh về công nghệ.";
    let report = auditor.audit_document(text);

    assert!(
        report.findings.iter().any(|f| f.category == "humanizer"),
        "Phải gắn cờ lỗi văn phong AI sáo rỗng (humanizer)"
    );
    assert!(report.score < 85.0);
}

#[test]
fn test_auditor_concurrency_batch_rayon() {
    let auditor = Auditor::default();

    let base_docs = vec![
        "Trong bối cảnh hiện nay đòi hỏi chúng ta phải cải cách tiền lương.",
        "Báo cáo tài chính đã được công bố bởi hội đồng quản trị.",
        "Trong phòng thí nghiệm có nhiều trang thiết bị hiện đại.",
        "Phương án này là tối ưu nhất hiện nay.",
        "Tiến hành thực hiện việc kiểm toán độc lập.",
    ];

    // Nhân bản lên 500 văn bản để thử thách Rayon đa luồng
    let mut large_batch = Vec::new();
    for _ in 0..100 {
        large_batch.extend(base_docs.clone());
    }

    assert_eq!(large_batch.len(), 500);

    let start = std::time::Instant::now();
    let batch_reports = auditor.audit_document_batch(&large_batch);
    let duration = start.elapsed();

    assert_eq!(batch_reports.len(), 500);
    println!(
        "Thời gian xử lý batch 500 văn bản song song qua Rayon: {:.2?} ({:.1} docs/sec)",
        duration,
        500.0 / duration.as_secs_f64()
    );

    // Kiểm tra tính nhất quán: văn bản số 0 (có lỗi đề què) phải luôn có điểm < 90
    assert!(batch_reports[0].score < 90.0);
    // Văn bản số 2 (hợp lệ với động từ tồn tại "có") phải đạt 100 điểm
    assert_eq!(batch_reports[2].score, 100.0);
}

#[test]
fn test_edge_cases_punctuation_emojis_numbers() {
    let auditor = Auditor::default();

    // 1. Văn bản rỗng và khoảng trắng
    assert_eq!(auditor.audit_document("").score, 100.0);
    assert_eq!(auditor.audit_document("   \n\t  ").score, 100.0);

    // 2. Chứa Emojis, số liệu, tiền tệ, phần trăm, URL
    let complex_text = "🎉 Dự án đạt doanh thu 150.000.000 VNĐ (tăng 25,5%) tại https://example.com/report vào ngày 10/11/2026. Trong phòng có 3 chuyên gia tư vấn.";
    let report = auditor.audit_document(complex_text);
    // Không được crash và không được bắt nhầm lỗi đề què
    assert_eq!(report.score, 100.0);
    assert!(report.findings.is_empty());

    // 3. Chứa dấu ngoặc kép, ngoặc đơn
    let quoted_text = "Ông Nam nhấn mạnh: \"Trong tình hình khó khăn, tập thể phải đoàn kết\". Điều này được đánh giá cao.";
    let q_report = auditor.audit_document(quoted_text);
    assert_eq!(q_report.score, 100.0);
}

#[test]
fn test_disabled_rules_and_severity_filters() {
    let mut config = AuditorConfig::default();
    config.disable_rule("ARTIFICIAL_PASSIVE_BOI");
    config.set_min_severity(Severity::Critical);

    let auditor = Auditor::with_config(config);
    let text = "Dự án đã được ký kết bởi giám đốc.";
    let report = auditor.audit_document(text);

    // Rule ARTIFICIAL_PASSIVE_BOI đã bị tắt, nên không được xuất hiện
    assert!(report.findings.is_empty());
    assert_eq!(report.score, 100.0);
}
