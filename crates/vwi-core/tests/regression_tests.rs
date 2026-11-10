//! # BỘ KIỂM THỬ CHỐNG LỖI HỒI QUY (REGRESSION TEST SUITE) CHO VWI-CORE
//!
//! Mỗi test case ở đây đại diện cho một lỗi lịch sử, cạm bẫy nhập nhằng từ vựng,
//! hoặc ca biên (edge-case) hóc búa được phát hiện trong quá trình phát triển.
//!
//! ⚠️ NGUYÊN TẮC: Tuyệt đối không xóa bất kỳ bài test nào trong file này.
//! Nếu một thay đổi làm fail bất kỳ test nào ở đây, đó là DẤU HIỆU HỒI QUY (REGRESSION)!

use vwi_core::auditor::Auditor;
use vwi_core::segmentation::viterbi_segmenter::tokenize_viterbi;

#[test]
fn test_regression_coccoc_ambiguity_ban_la() {
    // WHY: Từ "bàn là" (danh từ cái bàn ủi) rất dễ bị hư từ "là" nuốt chửng thành "bàn" + "là".
    // Viterbi Trellis phải phân biệt chuẩn xác dựa trên ngữ cảnh và ma trận chuyển tiếp.

    // Ca 1: Từ ghép "bàn là" (vật dụng)
    let t1 = tokenize_viterbi("Mẹ mua một chiếc bàn là mới.");
    let words1: Vec<&str> = t1.iter().map(|t| t.text.as_str()).collect();
    assert!(
        words1.contains(&"bàn là"),
        "LỖI HỒI QUY: Không nhận diện được từ ghép 'bàn là' trong câu mẹ mua bàn là!"
    );

    // Ca 2: Từ đơn "bàn" + hệ từ "là" (không được ghép oan)
    let t2 = tokenize_viterbi("Cái bàn là một đồ vật trong phòng.");
    let words2: Vec<&str> = t2.iter().map(|t| t.text.as_str()).collect();
    assert!(
        words2.contains(&"bàn") && words2.contains(&"là"),
        "LỖI HỒI QUY: Bị ghép oan 'bàn là' trong câu định nghĩa 'Cái bàn là một đồ vật'!"
    );
}

#[test]
fn test_regression_artificial_passive_boi_vi_avoidance() {
    // WHY: Lỗi lịch sử bản v0.1: Regex ngây thơ `(được|bị) ... bởi` bắt oan câu chứa liên từ "bởi vì".
    // Zero-Regex Scanner phải né 100% các liên từ chỉ nguyên nhân: bởi vì, bởi do, bởi thế, bởi tại.
    let auditor = Auditor::default();

    // Ca bắt oan kinh điển:
    let text_safe = "Sản phẩm này được khách hàng yêu thích bởi vì mùi hương thơm lâu.";
    let report_safe = auditor.audit_document(text_safe);
    let has_passive = report_safe
        .findings
        .iter()
        .any(|f| f.pattern_id == "ARTIFICIAL_PASSIVE_BE_BY");
    assert!(
        !has_passive,
        "LỖI HỒI QUY: Bắt oan câu có liên từ 'bởi vì' thành bị động dịch máy!"
    );

    // Ca bị động nhân tạo thật: bắt buộc phải phát hiện
    let text_bad = "Dự án này đã được phê duyệt bởi hội đồng quản trị.";
    let report_bad = auditor.audit_document(text_bad);
    let has_passive_bad = report_bad
        .findings
        .iter()
        .any(|f| f.pattern_id == "ARTIFICIAL_PASSIVE_BE_BY");
    assert!(
        has_passive_bad,
        "LỖI HỒI QUY: Bỏ lọt câu bị động sao phỏng 'be + V3 + by'!"
    );
}

#[test]
fn test_regression_existential_predicate_not_crippled() {
    // WHY: Theo GS. Cao Xuân Hạo (*Ngữ pháp chức năng tiếng Việt*), vị từ tồn tại ("có", "còn",
    // "xuất hiện", "tồn tại") đi sau Khung đề là câu chuẩn mực tiếng Việt, tuyệt đối không được báo lỗi Khung đề què.
    let auditor = Auditor::default();

    let safe_cases = [
        "Trong nhà có năm người đang ngồi uống trà.",
        "Ở ngoài sân có nhiều cây xanh bóng mát.",
        "Dưới thời tiết này xuất hiện nhiều cơn mưa giông.",
        "Trên bàn còn lại ba quyển sách của thư viện.",
        "Ở vùng quê tồn tại nhiều phong tục tập quán độc đáo.",
    ];

    for text in safe_cases {
        let rep = auditor.audit_document(text);
        let flagged = rep
            .findings
            .iter()
            .any(|f| f.pattern_id == "CRIPPLED_PREPOSITIONAL_TOPIC");
        assert!(
            !flagged,
            "LỖI HỒI QUY: Bắt oan câu vị từ tồn tại/khách quan '{}'!",
            text
        );
    }
}

#[test]
fn test_regression_syllable_boundary_than_an() {
    // WHY: Tránh lỗi substring matching: từ 'an' không được match bên trong từ 'than',
    // từ 'cá' không được match bên trong từ 'các'.
    let tokens = tokenize_viterbi("Công nhân ngành than làm việc chăm chỉ.");
    let has_an = tokens.iter().any(|t| t.text.to_lowercase() == "an");
    assert!(
        !has_an,
        "LỖI HỒI QUY: Rách ranh giới âm tiết! Bắt nhầm chữ 'an' trong 'than'!"
    );
}

#[test]
fn test_regression_unicode_nfc_coordinate_precision() {
    // WHY: Tiếng Việt có các ký tự UTF-8 chiếm 2-3 bytes (đ, ệ, ợ, ứ...).
    // Char offset trong Finding và TokenSpan bắt buộc phải khớp 100% với text.chars().
    let auditor = Auditor::default();
    let text = "Trong tình hình kinh tế hiện nay đòi hỏi giải pháp tối ưu nhất.";
    let report = auditor.audit_document(text);

    let chars: Vec<char> = text.chars().collect();
    for f in &report.findings {
        assert!(
            f.start <= f.end && f.end <= chars.len(),
            "LỖI HỒI QUY: Tọa độ offset ({}, {}) tràn mảng ký tự {}!",
            f.start,
            f.end,
            chars.len()
        );
        let snippet: String = chars[f.start..f.end].iter().collect();
        assert!(
            !snippet.is_empty(),
            "LỖI HỒI QUY: Snippet trích xuất bị rỗng!"
        );
    }
}

#[test]
fn test_regression_prep_conj_conflict_nhung() {
    // WHY: Lỗi xung đột cấu trúc "Với/Bằng/Qua ... nhưng ...".
    // Khung giới từ chỉ phương tiện/điều kiện không thể kết hợp trực tiếp với liên từ phản đề "nhưng".
    let auditor = Auditor::default();
    let text = "Với những nỗ lực không ngừng nghỉ của cả tập thể nhưng kết quả vẫn chưa đạt kỳ vọng.";
    let report = auditor.audit_document(text);

    let flagged = report
        .findings
        .iter()
        .any(|f| f.pattern_id == "PREPOSITION_CONJUNCTION_CONFLICT");
    assert!(
        flagged,
        "LỖI HỒI QUY: Không bắt được lỗi xung đột cú pháp 'Với ... nhưng'!"
    );
}

#[test]
fn test_regression_pleonasm_nguyen_nhan_la_boi_vi() {
    // WHY: Điệp ngữ thừa "nguyên nhân là bởi vì" hoặc "lý do là bởi vì".
    // Phải bắt được cả khi có hoặc không có từ chen giữa (ví dụ "nguyên nhân chính là bởi vì" và "nguyên nhân là bởi vì").
    let auditor = Auditor::default();

    let cases = [
        "Nguyên nhân là bởi vì anh ấy đến muộn.",
        "Nguyên nhân chính yếu là bởi vì hệ thống quá tải.",
        "Lý do là bởi vì kinh phí chưa được phê duyệt.",
    ];

    for text in cases {
        let report = auditor.audit_document(text);
        let flagged = report
            .findings
            .iter()
            .any(|f| f.pattern_id == "LOGICAL_PLEONASM");
        assert!(
            flagged,
            "LỖI HỒI QUY: Không bắt được lỗi trùng ngữ logic trong câu: '{}'!",
            text
        );
    }
}

#[test]
fn test_regression_two_token_compound_governing_verbs() {
    // WHY: Viterbi tách "cho thấy" thành [cho, thấy], "chỉ ra" thành [chỉ, ra].
    // Shallow Parser bắt buộc phải kiểm tra cụm 2 token để không bỏ lọt Khung đề què.
    let auditor = Auditor::default();

    let cases = [
        "Qua kết quả khảo sát thực tế cho thấy sự hài lòng của khách hàng tăng 15%.",
        "Bằng các nghiên cứu lâm sàng chỉ ra hiệu quả vượt trội của thuốc mới.",
        "Với chính sách tiền tệ nới lỏng kéo giảm lãi suất cho vay.",
    ];

    for text in cases {
        let report = auditor.audit_document(text);
        let flagged = report
            .findings
            .iter()
            .any(|f| f.pattern_id == "CRIPPLED_PREPOSITIONAL_TOPIC");
        assert!(
            flagged,
            "LỖI HỒI QUY: Bỏ lọt lỗi Khung đề què với vị từ 2 token trong câu: '{}'!",
            text
        );
    }
}
