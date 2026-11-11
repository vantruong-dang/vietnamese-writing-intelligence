//! # GUARDRAIL CHỐNG 1 SAO
//!
//! Bộ test này mô phỏng bước "blind test" trước khi publish:
//! - Văn bản báo chí/hành chính sạch: **không được có Finding Major/Critical**.
//! - Comment mạng xã hội có teencode: chỉ được **Advisory**, điểm vẫn 100.
//!
//! Đây là synthetic corpus (không dùng nguyên văn báo) để CI chạy ổn định,
//! không phụ thuộc mạng và không vướng bản quyền.

use vwi_core::auditor::{Auditor, AuditorConfig};
use vwi_core::types::{Finding, Register};

fn damaging(findings: &[Finding]) -> Vec<(String, String)> {
    findings
        .iter()
        .filter(|f| matches!(f.severity.as_str(), "major" | "critical"))
        .map(|f| (f.pattern_id.clone(), f.matched_span.clone()))
        .collect()
}

fn assert_no_damaging(label: &str, text: &str) {
    let report = Auditor::default().audit_document(text);
    let bad = damaging(&report.findings);
    assert!(
        bad.is_empty(),
        "❌ [{}] Có lỗi Major/Critical trên văn bản sạch: {:?}\nText: {:?}",
        label,
        bad,
        text
    );
}

#[test]
fn test_journalism_paragraphs_no_damaging_findings() {
    let paragraphs = [
        // Thời sự
        "Chính phủ vừa ban hành nghị quyết về phát triển kinh tế - xã hội. Theo đó, các địa phương cần tập trung giải ngân vốn đầu tư công, tháo gỡ khó khăn cho doanh nghiệp và bảo đảm an sinh xã hội.",
        // Kinh doanh
        "Thị trường chứng khoán mở cửa phiên giao dịch đầu tuần trong sắc xanh. Nhóm cổ phiếu ngân hàng dẫn dắt đà tăng, thanh khoản cải thiện rõ rệt so với phiên trước.",
        // Thể thao
        "Đội tuyển Việt Nam giành chiến thắng 2-1 trước đối thủ. Huấn luyện viên trưởng đánh giá các cầu thủ đã thực hiện tốt đấu pháp đề ra.",
        // Văn học
        "Trời chiều buông xuống, con sông như một dải lụa mềm mại. Ông lão ngồi bên bến, lặng nhìn những con thuyền về muộn.",
    ];

    for (i, text) in paragraphs.iter().enumerate() {
        assert_no_damaging(&format!("bao-chi-{}", i + 1), text);
    }
}

#[test]
fn test_administrative_paragraph_no_damaging_findings() {
    let text = "Thực hiện Nghị định số 30/2020/NĐ-CP, Văn phòng Ủy ban nhân dân tỉnh đề nghị các đơn vị rà soát thể thức văn bản. Báo cáo gửi về Sở Nội vụ trước ngày 15 tháng 9 năm 2026.";
    let report = Auditor::with_config(AuditorConfig {
        register: Register::Administrative,
        ..Default::default()
    })
    .audit_document(text);
    let bad = damaging(&report.findings);
    assert!(
        bad.is_empty(),
        "❌ [hanh-chinh] Có lỗi Major/Critical: {:?}\nText: {:?}",
        bad,
        text
    );
}

#[test]
fn test_general_prose_does_not_run_admin_rules() {
    // Cùng câu, nhưng ở ngữ vực phổ thông: không bị áp quy chuẩn hành chính.
    let text = "Chính sách này do nhà nước ban hành.";
    let general = Auditor::default().audit_document(text);
    assert!(
        !general
            .findings
            .iter()
            .any(|f| f.category == "admin_format"),
        "❌ Admin rules chạy nhầm trong văn xuôi phổ thông: {:?}",
        general.findings
    );

    let admin = Auditor::with_config(AuditorConfig {
        register: Register::Administrative,
        ..Default::default()
    })
    .audit_document(text);
    assert!(
        admin
            .findings
            .iter()
            .any(|f| f.pattern_id == "R4_SPECIAL_NOUN_CAPITALIZATION"),
        "❌ Admin rules không chạy trong ngữ vực hành chính"
    );
}

#[test]
fn test_teencode_comments_are_advisory_only_and_score_100() {
    let comments = [
        "Mk ko bt j ca",
        "Hôm nay mk vs ny đi chơi, vui lắm",
        "Dc rồi nha",
        "Tôi ko dc đi",
        "Ship nhanh ko shop",
    ];

    for text in comments {
        let report = Auditor::default().audit_document(text);
        for f in &report.findings {
            assert_eq!(
                f.severity, "advisory",
                "❌ Teencode bị gắn mức '{}' (phải advisory): {} | {:?}",
                f.severity, f.pattern_id, text
            );
        }
        assert_eq!(
            report.score, 100.0,
            "❌ Điểm bị trừ oan trên comment teencode: {:?}",
            text
        );
        assert!(
            report.findings.iter().any(|f| f.category == "teencode"),
            "❌ Không nhận diện được teencode trong: {:?}",
            text
        );
    }
}

#[test]
fn test_clean_acronyms_and_proper_nouns_not_flagged() {
    // Viết hoa toàn bộ chữ viết tắt KHÔNG bị coi là teencode.
    assert_no_damaging(
        "viet-tat",
        "VS BT K R là các ký hiệu viết tắt trong tài liệu.",
    );
    // Tên riêng nước ngoài không bị Telex/real-word bắt oan.
    assert_no_damaging(
        "ten-rieng",
        "Google Docs và Microsoft Teams đang được dùng tại công ty.",
    );
}
