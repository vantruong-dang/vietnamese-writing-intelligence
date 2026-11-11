// ============================================================================
// GOLDEN TEST SET V2 - Bộ 106 ca kiểm thử độc lập
// Nguyên tắc: ĐO TÁCH BIỆT - không gộp thành % tổng
//
// Lưu ý kỹ thuật: hàm đo "sửa đúng" phải ÁP DỤNG các AutoFix rồi so sánh cả
// câu, không thể so một suggestion cục bộ với cả câu đã sửa.
// ============================================================================

use std::collections::HashMap;
use vwi_core::auditor::{Auditor, AuditorConfig};
use vwi_core::types::{Finding, Register};

/// Định nghĩa ★ (real-word) dùng xuyên suốt bộ test:
/// âm tiết gây lỗi là **một từ/âm tiết có thật, đứng độc lập được trong tiếng
/// Việt** (năng, nghĩ, ngũ, sinh, trăm, chao, rành, no, hoàng, cự, đặc, chì,
/// chữ, sét, găng, xử...). Nhờ vậy Metric 5 đo đúng nhóm "từ đúng chính tả
/// nhưng sai ngữ cảnh", không phụ thuộc vào việc plugin sửa bằng tầng nào.
#[derive(Debug)]
#[allow(dead_code)] // `rule`, `confidence`, `source` là metadata đo lường, giữ nguyên để đối chiếu.
struct GoldenTestCase {
    id: &'static str,
    input: &'static str,
    expected_flag: bool,
    expected_fix: Option<&'static str>,
    category: &'static str,
    rule: Option<&'static str>,
    is_real_word_error: bool,
    confidence: &'static str,
    source: &'static str,
}

// ============================================================================
// DỮ LIỆU TEST CASES
// ============================================================================

fn all_test_cases() -> Vec<GoldenTestCase> {
    use GoldenTestCase as G;
    vec![
        // ====================================================================
        // BỘ 1A: 25 CÂU ĐÚNG (KHÔNG ĐƯỢC BÁO NHẦM)
        // ====================================================================
        G {
            id: "1",
            input: "Hôm nay trời nắng đẹp, chúng ta đi dạo công viên nhé.",
            expected_flag: false,
            expected_fix: None,
            category: "correct",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "2",
            input: "Tôi đã nộp báo cáo cho giám đốc vào sáng thứ Hai.",
            expected_flag: false,
            expected_fix: None,
            category: "correct",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "3",
            input: "Cô ấy đang học tiếng Anh ở trung tâm gần nhà.",
            expected_flag: false,
            expected_fix: None,
            category: "correct",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "4",
            input: "Quyết định này có hiệu lực kể từ ngày ký.",
            expected_flag: false,
            expected_fix: None,
            category: "correct",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "5",
            input: "Con đường này rất đông xe vào giờ cao điểm.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_pos_tagging",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "6",
            input: "Mẹ cho thêm một thìa đường vào cốc cà phê.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_pos_tagging",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "7",
            input: "Chúng tôi sẽ gửi email xác nhận trong vòng 24 giờ.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_loan_word",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "8",
            input: "Nguyễn Văn An sinh năm 1990 tại Đà Nẵng.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_proper_noun",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "9",
            input: "Bạn đã xem phim hôm qua chưa?",
            expected_flag: false,
            expected_fix: None,
            category: "correct_question",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "10",
            input: "Công ty TNHH Minh Phát thông báo tuyển dụng nhân viên kinh doanh.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_abbreviation",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "11",
            input: "Em ăn cơm chưa? Anh đợi em nãy giờ.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_colloquial",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "12",
            input: "Mưa suốt đêm nên sáng nay đường ngập nặng.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_journalism",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "13",
            input: "Tôi thích uống cà phê sữa đá vào buổi sáng.",
            expected_flag: false,
            expected_fix: None,
            category: "correct",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "14",
            input: "Giá xăng tăng khiến chi phí vận chuyển cũng tăng theo.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_journalism",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "15",
            input: "Sinh viên cần nộp học phí trước ngày 15 tháng 9.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_admin",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "16",
            input: "Cái điện thoại này pin trâu thật sự.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_slang",
            rule: None,
            is_real_word_error: false,
            confidence: "?",
            source: "bo1",
        },
        G {
            id: "17",
            input: "Trẻ em cần được bảo vệ và chăm sóc đầy đủ.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_natural_passive",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "18",
            input: "Cuối tuần này gia đình tôi về quê thăm ông bà.",
            expected_flag: false,
            expected_fix: None,
            category: "correct",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "19",
            input: "Hợp đồng được ký kết giữa hai bên vào ngày mùng 3.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_natural_passive",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "20",
            input: "Tôi đang dùng Google Docs để soạn thảo văn bản.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_proper_noun",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "21",
            input: "Bàn học của em bé làm bằng gỗ sồi.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_pos_tagging",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "22",
            input: "Họ đã bàn bạc kỹ trước khi đưa ra quyết định.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_pos_tagging",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "23",
            input: "Chị ấy nói rất chân thành nên ai cũng xúc động.",
            expected_flag: false,
            expected_fix: None,
            category: "correct",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "24",
            input: "Dự án sẽ được hoàn thành trước thời hạn nếu mọi người cố gắng.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_complex",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "25",
            input: "Tối qua tụi mình ăn bún chả ngon quá trời.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_colloquial",
            rule: None,
            is_real_word_error: false,
            confidence: "?",
            source: "bo1",
        },
        // ====================================================================
        // BỘ 1B: 25 CÂU SAI (PHẢI BÁO LỖI + SỬA ĐÚNG)
        // ====================================================================
        G {
            id: "26",
            input: "Hôm nay trời năng đẹp, chúng ta đi dạo nhé.",
            expected_flag: true,
            expected_fix: Some("Hôm nay trời nắng đẹp, chúng ta đi dạo nhé."),
            category: "dau_thanh",
            rule: None,
            is_real_word_error: true,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "27",
            input: "Cảm ơn bạn đã giúp đở tôi.",
            expected_flag: true,
            expected_fix: Some("Cảm ơn bạn đã giúp đỡ tôi."),
            category: "dau_thanh",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "28",
            input: "Anh ấy cần nghĩ ngơi sau giờ làm việc.",
            expected_flag: true,
            expected_fix: Some("Anh ấy cần nghỉ ngơi sau giờ làm việc."),
            category: "dau_thanh",
            rule: None,
            is_real_word_error: true,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "29",
            input: "Em bé đang ngũ trong phòng.",
            expected_flag: true,
            expected_fix: Some("Em bé đang ngủ trong phòng."),
            category: "dau_thanh",
            rule: None,
            is_real_word_error: true,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "30",
            input: "Chúng ta cần cố găng hơn nữa.",
            expected_flag: true,
            expected_fix: Some("Chúng ta cần cố gắng hơn nữa."),
            category: "dau_thanh",
            rule: None,
            is_real_word_error: true,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "31",
            input: "Tôi sẽ gữi tài liệu cho bạn vào chiều nay.",
            expected_flag: true,
            expected_fix: Some("Tôi sẽ gửi tài liệu cho bạn vào chiều nay."),
            category: "dau_thanh",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "32",
            input: "Chị ấy mua một chiếc áo mởi ở chợ.",
            expected_flag: true,
            expected_fix: Some("Chị ấy mua một chiếc áo mới ở chợ."),
            category: "dau_thanh",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "33",
            input: "Cô ấy rất sinh đẹp và hiền lành.",
            expected_flag: true,
            expected_fix: Some("Cô ấy rất xinh đẹp và hiền lành."),
            category: "phu_am_dau",
            rule: None,
            is_real_word_error: true,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "34",
            input: "Hãy xắp xếp tài liệu theo thứ tự thời gian.",
            expected_flag: true,
            expected_fix: Some("Hãy sắp xếp tài liệu theo thứ tự thời gian."),
            category: "phu_am_dau",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "35",
            input: "Trẻ em cần được trăm sóc đầy đủ.",
            expected_flag: true,
            expected_fix: Some("Trẻ em cần được chăm sóc đầy đủ."),
            category: "phu_am_dau",
            rule: None,
            is_real_word_error: true,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "36",
            input: "Tôi muốn chao đổi ý kiến với anh.",
            expected_flag: true,
            expected_fix: Some("Tôi muốn trao đổi ý kiến với anh."),
            category: "phu_am_dau",
            rule: None,
            is_real_word_error: true,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "37",
            input: "Cô ấy rành cả buổi tối để đọc sách.",
            expected_flag: true,
            expected_fix: Some("Cô ấy dành cả buổi tối để đọc sách."),
            category: "phu_am_dau",
            rule: None,
            is_real_word_error: true,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "38",
            input: "Tôi rất no lắng về kết quả kỳ thi.",
            expected_flag: true,
            expected_fix: Some("Tôi rất lo lắng về kết quả kỳ thi."),
            category: "phu_am_dau",
            rule: None,
            is_real_word_error: true,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "39",
            input: "Em gái tôi rất ngoang ngoãn.",
            expected_flag: true,
            expected_fix: Some("Em gái tôi rất ngoan ngoãn."),
            category: "phu_am_cuoi",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "40",
            input: "Anh ấy đã hoàng thành công việc đúng hạn.",
            expected_flag: true,
            expected_fix: Some("Anh ấy đã hoàn thành công việc đúng hạn."),
            category: "phu_am_cuoi",
            rule: None,
            is_real_word_error: true,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "41",
            input: "Anh ấy làm việc rất tích cự.",
            expected_flag: true,
            expected_fix: Some("Anh ấy làm việc rất tích cực."),
            category: "phu_am_cuoi",
            rule: None,
            is_real_word_error: true,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "42",
            input: "Tôi đã đặc vé máy bay cho chuyến đi tuần sau.",
            expected_flag: true,
            expected_fix: Some("Tôi đã đặt vé máy bay cho chuyến đi tuần sau."),
            category: "phu_am_cuoi",
            rule: None,
            is_real_word_error: true,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "43",
            input: "Tôi đã ăn rồi cơm.",
            expected_flag: true,
            expected_fix: Some("Tôi đã ăn cơm rồi."),
            category: "ngu_phap",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "44",
            input: "Cô ấy đẹp rất.",
            expected_flag: true,
            expected_fix: Some("Cô ấy rất đẹp."),
            category: "ngu_phap",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "45",
            input: "Tôi mua đã một cuốn sách hôm qua.",
            expected_flag: true,
            expected_fix: Some("Tôi đã mua một cuốn sách hôm qua."),
            category: "ngu_phap",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "46",
            input: "Trên bàn có ba quyển bút.",
            expected_flag: true,
            expected_fix: Some("Trên bàn có ba cây bút."),
            category: "ngu_phap",
            rule: None,
            is_real_word_error: false,
            confidence: "?",
            source: "bo1",
        },
        G {
            id: "47",
            input: "Hôm qua tôi sẽ đi chợ.",
            expected_flag: true,
            expected_fix: Some("Hôm qua tôi đã đi chợ."),
            category: "ngu_phap",
            rule: None,
            is_real_word_error: false,
            confidence: "?",
            source: "bo1",
        },
        G {
            id: "48",
            input: "Chúng tôi đang thảo luận về về kế hoạch mới.",
            expected_flag: true,
            expected_fix: Some("Chúng tôi đang thảo luận về kế hoạch mới."),
            category: "ngu_phap",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "49",
            input: "Cô ấy là rất thông minh.",
            expected_flag: true,
            expected_fix: Some("Cô ấy rất thông minh."),
            category: "ngu_phap",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        G {
            id: "50",
            input: "Tôi không đã gặp anh ấy bao giờ.",
            expected_flag: true,
            expected_fix: Some("Tôi chưa gặp anh ấy bao giờ."),
            category: "ngu_phap",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "bo1",
        },
        // ====================================================================
        // BỘ 2: 50 CÂU SAI (HÀNH CHÍNH / THỂ THỨC)
        // ====================================================================
        G {
            id: "51",
            input: "Hà Nội, ngày 5 tháng 3 năm 2026",
            expected_flag: true,
            expected_fix: Some("Hà Nội, ngày 05 tháng 3 năm 2026"),
            category: "R1_date",
            rule: Some("R1"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "52",
            input: "Hà Nội, ngày 12 tháng 2 năm 2026",
            expected_flag: true,
            expected_fix: Some("Hà Nội, ngày 12 tháng 02 năm 2026"),
            category: "R1_date",
            rule: Some("R1"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "53",
            input: "Đà Nẵng, ngày 10 tháng 1 năm 2026",
            expected_flag: true,
            expected_fix: Some("Đà Nẵng, ngày 10 tháng 01 năm 2026"),
            category: "R1_date",
            rule: Some("R1"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "54",
            input: "Hà Nội ngày 20 tháng 6 năm 2026",
            expected_flag: true,
            expected_fix: Some("Hà Nội, ngày 20 tháng 6 năm 2026"),
            category: "R9_place_date",
            rule: Some("R9"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "55",
            input: "Hà Nội, Ngày 20 tháng 6 năm 2026",
            expected_flag: true,
            expected_fix: Some("Hà Nội, ngày 20 tháng 6 năm 2026"),
            category: "R9_place_date",
            rule: Some("R9"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "56",
            input: "Hà Nội, ngày 15/6/2026",
            expected_flag: true,
            expected_fix: Some("Hà Nội, ngày 15 tháng 6 năm 2026"),
            category: "R9_place_date",
            rule: Some("R9"),
            is_real_word_error: false,
            confidence: "?",
            source: "bo2",
        },
        G {
            id: "57",
            input: "Tháng đầu tiên của năm là tháng một.",
            expected_flag: true,
            expected_fix: Some("Tháng đầu tiên của năm là tháng Một."),
            category: "R2_month_cap",
            rule: Some("R2"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "58",
            input: "Hội nghị khai mạc vào thứ ba, ngày 10 tháng 11 năm 2026.",
            expected_flag: true,
            expected_fix: Some("Hội nghị khai mạc vào thứ Ba, ngày 10 tháng 11 năm 2026."),
            category: "R2_month_cap",
            rule: Some("R2"),
            is_real_word_error: false,
            confidence: "?",
            source: "bo2",
        },
        G {
            id: "59",
            input: "Họp giao ban vào sáng thứ sáu hằng tuần.",
            expected_flag: true,
            expected_fix: Some("Họp giao ban vào sáng thứ Sáu hằng tuần."),
            category: "R2_month_cap",
            rule: Some("R2"),
            is_real_word_error: false,
            confidence: "?",
            source: "bo2",
        },
        G {
            id: "60",
            input: "Ông Nguyễn văn An được bổ nhiệm làm Giám đốc.",
            expected_flag: true,
            expected_fix: Some("Ông Nguyễn Văn An được bổ nhiệm làm Giám đốc."),
            category: "R14_proper_noun",
            rule: Some("R14"),
            is_real_word_error: false,
            confidence: "?",
            source: "bo2",
        },
        G {
            id: "61",
            input: "Thực hiện theo Khoản 2 Điều 5 Nghị định này.",
            expected_flag: true,
            expected_fix: Some("Thực hiện theo khoản 2 Điều 5 Nghị định này."),
            category: "R3_legal_ref",
            rule: Some("R3"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "62",
            input: "Quy định tại Điểm a khoản 1 Điều 9.",
            expected_flag: true,
            expected_fix: Some("Quy định tại điểm a khoản 1 Điều 9."),
            category: "R3_legal_ref",
            rule: Some("R3"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "63",
            input: "Căn cứ điều 15 Luật Lưu trữ năm 2024.",
            expected_flag: true,
            expected_fix: Some("Căn cứ Điều 15 Luật Lưu trữ năm 2024."),
            category: "R3_legal_ref",
            rule: Some("R3"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "64",
            input: "Nội dung này quy định tại chương II của Nghị định.",
            expected_flag: true,
            expected_fix: Some("Nội dung này quy định tại Chương II của Nghị định."),
            category: "R3_legal_ref",
            rule: Some("R3"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "65",
            input: "Xem tại mục II Phần I Phụ lục I.",
            expected_flag: true,
            expected_fix: Some("Xem tại Mục II Phần I Phụ lục I."),
            category: "R3_legal_ref",
            rule: Some("R3"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "66",
            input: "Cán bộ, công chức phải tận tụy phục vụ nhân dân.",
            expected_flag: true,
            expected_fix: Some("Cán bộ, công chức phải tận tụy phục vụ Nhân dân."),
            category: "R4_special_noun",
            rule: Some("R4"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "67",
            input: "Chính sách này do nhà nước ban hành.",
            expected_flag: true,
            expected_fix: Some("Chính sách này do Nhà nước ban hành."),
            category: "R4_special_noun",
            rule: Some("R4"),
            is_real_word_error: false,
            confidence: "?",
            source: "bo2",
        },
        G {
            id: "68",
            input: "Trụ sở đặt tại thủ đô Hà Nội.",
            expected_flag: true,
            expected_fix: Some("Trụ sở đặt tại Thủ đô Hà Nội."),
            category: "R5_special_place",
            rule: Some("R5"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "69",
            input: "Hội nghị tổ chức tại thành phố Hồ Chí Minh.",
            expected_flag: true,
            expected_fix: Some("Hội nghị tổ chức tại Thành phố Hồ Chí Minh."),
            category: "R5_special_place",
            rule: Some("R5"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "70",
            input: "Công văn gửi Bộ giáo dục và đào tạo.",
            expected_flag: true,
            expected_fix: Some("Công văn gửi Bộ Giáo dục và Đào tạo."),
            category: "R10_agency",
            rule: Some("R10"),
            is_real_word_error: false,
            confidence: "?",
            source: "bo2",
        },
        G {
            id: "71",
            input: "Gửi Văn phòng ủy ban nhân dân tỉnh Đồng Nai.",
            expected_flag: true,
            expected_fix: Some("Gửi Văn phòng Ủy ban nhân dân tỉnh Đồng Nai."),
            category: "R10_agency",
            rule: Some("R10"),
            is_real_word_error: false,
            confidence: "?",
            source: "bo2",
        },
        G {
            id: "72",
            input: "Bộ trưởng bộ Nội vụ ký ban hành quyết định.",
            expected_flag: true,
            expected_fix: Some("Bộ trưởng Bộ Nội vụ ký ban hành quyết định."),
            category: "R10_agency",
            rule: Some("R10"),
            is_real_word_error: false,
            confidence: "?",
            source: "bo2",
        },
        G {
            id: "73",
            input: "Chủ Tịch nước ký lệnh công bố luật.",
            expected_flag: true,
            expected_fix: Some("Chủ tịch nước ký lệnh công bố luật."),
            category: "R10_agency",
            rule: Some("R10"),
            is_real_word_error: false,
            confidence: "?",
            source: "bo2",
        },
        G {
            id: "74",
            input: "Công ty được tặng huân chương lao động hạng Ba.",
            expected_flag: true,
            expected_fix: Some("Công ty được tặng Huân chương Lao động hạng Ba."),
            category: "R10_agency",
            rule: Some("R10"),
            is_real_word_error: false,
            confidence: "?",
            source: "bo2",
        },
        G {
            id: "75",
            input: "Độc lập - tự do - hạnh phúc",
            expected_flag: true,
            expected_fix: Some("Độc lập - Tự do - Hạnh phúc"),
            category: "R7_motto",
            rule: Some("R7"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "76",
            input: "CỘNG HÒA XÃ HÔI CHỦ NGHĨA VIỆT NAM",
            expected_flag: true,
            expected_fix: Some("CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM"),
            category: "R12_spelling",
            rule: Some("R12"),
            // "hôi" là âm tiết có thật (hôi thối) → đúng định nghĩa ★.
            is_real_word_error: true,
            confidence: "—",
            source: "bo2",
        },
        G {
            id: "77",
            input: "Số 15/QĐ-UBND",
            expected_flag: true,
            expected_fix: Some("Số: 15/QĐ-UBND"),
            category: "R8_doc_number",
            rule: Some("R8"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "78",
            input: "Số: 15/QĐ UBND",
            expected_flag: true,
            expected_fix: Some("Số: 15/QĐ-UBND"),
            category: "R8_doc_number",
            rule: Some("R8"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "79",
            input: "Số: 15/QD-UBND",
            expected_flag: true,
            expected_fix: Some("Số: 15/QĐ-UBND"),
            category: "R8_doc_number",
            rule: Some("R8"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "80",
            input: "THÔNG BÁO – Số: 03/QĐ-UBND",
            expected_flag: true,
            expected_fix: Some("THÔNG BÁO – Số: 03/TB-UBND"),
            category: "R8_doc_number",
            rule: Some("R8"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "81",
            input: "Số: 12/Kh-UBND",
            expected_flag: true,
            expected_fix: Some("Số: 12/KH-UBND"),
            category: "R8_doc_number",
            rule: Some("R8"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "82",
            input: "Số: 04/CTR-ĐHV",
            expected_flag: true,
            expected_fix: Some("Số: 04/CTr-ĐHV"),
            category: "R8_doc_number",
            rule: Some("R8"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "83",
            input: "Số: 15 /QĐ-UBND",
            expected_flag: true,
            expected_fix: Some("Số: 15/QĐ-UBND"),
            category: "R8_doc_number",
            rule: Some("R8"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "84",
            input: "Hội nghị đã kết thúc. kết quả sẽ báo cáo sau.",
            expected_flag: true,
            expected_fix: Some("Hội nghị đã kết thúc. Kết quả sẽ báo cáo sau."),
            category: "R6_capitalize_after_dot",
            rule: Some("R6"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "85",
            input: "Ai chịu trách nhiệm? chúng ta cần làm rõ.",
            expected_flag: true,
            expected_fix: Some("Ai chịu trách nhiệm? Chúng ta cần làm rõ."),
            category: "R6_capitalize_after_dot",
            rule: Some("R6"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "86",
            input: "Đề nghị thực hiện ngay! các đơn vị báo cáo trước ngày 30.",
            expected_flag: true,
            expected_fix: Some("Đề nghị thực hiện ngay! Các đơn vị báo cáo trước ngày 30."),
            category: "R6_capitalize_after_dot",
            rule: Some("R6"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "87",
            input: "Kính gửi : Sở Nội vụ",
            expected_flag: true,
            expected_fix: Some("Kính gửi: Sở Nội vụ"),
            category: "R12_punctuation",
            rule: Some("R12"),
            is_real_word_error: false,
            confidence: "—",
            source: "bo2",
        },
        G {
            id: "88",
            input: "Gửi các sở ,ngành và địa phương.",
            expected_flag: true,
            expected_fix: Some("Gửi các sở, ngành và địa phương."),
            category: "R12_punctuation",
            rule: Some("R12"),
            is_real_word_error: false,
            confidence: "—",
            source: "bo2",
        },
        G {
            id: "89",
            input: "Tăng trưởng đạt 6.5% so với cùng kỳ.",
            expected_flag: true,
            expected_fix: Some("Tăng trưởng đạt 6,5% so với cùng kỳ."),
            category: "R11_decimal",
            rule: Some("R11"),
            is_real_word_error: false,
            confidence: "✓",
            source: "bo2",
        },
        G {
            id: "90",
            input: "Tổng kinh phí là 1,500,000 đồng.",
            expected_flag: true,
            expected_fix: Some("Tổng kinh phí là 1.500.000 đồng."),
            category: "R11_decimal",
            rule: Some("R11"),
            is_real_word_error: false,
            confidence: "?",
            source: "bo2",
        },
        G {
            id: "91",
            input: "Các đơn vị phải xử dụng kinh phí đúng mục đích.",
            expected_flag: true,
            expected_fix: Some("Các đơn vị phải sử dụng kinh phí đúng mục đích."),
            category: "R12_spelling",
            rule: Some("R12"),
            is_real_word_error: true,
            confidence: "—",
            source: "bo2",
        },
        G {
            id: "92",
            input: "Đề nghị các sở chiển khai kế hoạch.",
            expected_flag: true,
            expected_fix: Some("Đề nghị các sở triển khai kế hoạch."),
            category: "R12_spelling",
            rule: Some("R12"),
            is_real_word_error: false,
            confidence: "—",
            source: "bo2",
        },
        G {
            id: "93",
            input: "Phòng Tổ chức chủ chì phối hợp thực hiện.",
            expected_flag: true,
            expected_fix: Some("Phòng Tổ chức chủ trì phối hợp thực hiện."),
            category: "R12_spelling",
            rule: Some("R12"),
            is_real_word_error: true,
            confidence: "—",
            source: "bo2",
        },
        G {
            id: "94",
            input: "Cần dải quyết dứt điểm khiếu nại.",
            expected_flag: true,
            expected_fix: Some("Cần giải quyết dứt điểm khiếu nại."),
            category: "R12_spelling",
            rule: Some("R12"),
            is_real_word_error: false,
            confidence: "—",
            source: "bo2",
        },
        G {
            id: "95",
            input: "Hồ sơ được lưu chữ tại Văn phòng.",
            expected_flag: true,
            expected_fix: Some("Hồ sơ được lưu trữ tại Văn phòng."),
            category: "R12_spelling",
            rule: Some("R12"),
            is_real_word_error: true,
            confidence: "—",
            source: "bo2",
        },
        G {
            id: "96",
            input: "Phòng Kế hoạch sét duyệt hồ sơ.",
            expected_flag: true,
            expected_fix: Some("Phòng Kế hoạch xét duyệt hồ sơ."),
            category: "R12_spelling",
            rule: Some("R12"),
            is_real_word_error: true,
            confidence: "—",
            source: "bo2",
        },
        G {
            id: "97",
            input: "Quyết định này được có hiệu lực kể từ ngày ký.",
            expected_flag: true,
            expected_fix: Some("Quyết định này có hiệu lực kể từ ngày ký."),
            category: "R13_grammar",
            rule: Some("R13"),
            is_real_word_error: false,
            confidence: "—",
            source: "bo2",
        },
        G {
            id: "98",
            input: "Đề nghị các đơn vị để báo cáo trước ngày 30.",
            expected_flag: true,
            expected_fix: Some("Đề nghị các đơn vị báo cáo trước ngày 30."),
            category: "R13_grammar",
            rule: Some("R13"),
            is_real_word_error: false,
            confidence: "—",
            source: "bo2",
        },
        G {
            id: "99",
            input: "Công ty đã sẽ gửi hồ sơ vào tuần trước.",
            expected_flag: true,
            expected_fix: Some("Công ty đã gửi hồ sơ vào tuần trước."),
            category: "R13_grammar",
            rule: Some("R13"),
            is_real_word_error: false,
            confidence: "—",
            source: "bo2",
        },
        G {
            id: "100",
            input: "V/v về đề nghị bổ sung hồ sơ.",
            expected_flag: true,
            expected_fix: Some("V/v đề nghị bổ sung hồ sơ."),
            category: "R13_grammar",
            rule: Some("R13"),
            is_real_word_error: false,
            confidence: "?",
            source: "bo2",
        },
        // ====================================================================
        // 6 CA BẪY (KHÔNG ĐƯỢC BÁO NHẦM)
        // ====================================================================
        G {
            id: "G1",
            input: "CỘNG HOÀ XÃ HỘI CHỦ NGHĨA VIỆT NAM",
            expected_flag: false,
            expected_fix: None,
            category: "trap",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "trap",
        },
        G {
            id: "G2",
            input: "Kính gửi: các đơn vị trực thuộc",
            expected_flag: false,
            expected_fix: None,
            category: "trap",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "trap",
        },
        G {
            id: "G3",
            input: "các cơ quan, tổ chức nhà nước",
            expected_flag: false,
            expected_fix: None,
            category: "trap",
            rule: None,
            is_real_word_error: false,
            confidence: "?",
            source: "trap",
        },
        G {
            id: "G4",
            input: "tỉ lệ và tỷ lệ",
            expected_flag: false,
            expected_fix: None,
            category: "trap",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "trap",
        },
        G {
            id: "G5",
            input: "ký và kí, kỳ và kì",
            expected_flag: false,
            expected_fix: None,
            category: "trap",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "trap",
        },
        G {
            id: "G6",
            input: "ngày 05 tháng 3 năm 2020",
            expected_flag: false,
            expected_fix: None,
            category: "trap",
            rule: None,
            is_real_word_error: false,
            confidence: "—",
            source: "trap",
        },
    ]
}

// ============================================================================
// HÀM HELPER
// ============================================================================

/// Bộ 2 và trap là văn bản hành chính → audit với Register::Administrative.
/// Bộ 1 là văn xuôi phổ thông → Register::General (admin rules không chạy).
fn register_for(case: &GoldenTestCase) -> Register {
    if case.source == "bo2" || case.source == "trap" {
        Register::Administrative
    } else {
        Register::General
    }
}

fn run_audit_with(sentence: &str, register: Register) -> Vec<Finding> {
    Auditor::with_config(AuditorConfig {
        register,
        ..Default::default()
    })
    .audit(sentence)
}

fn run_audit(case: &GoldenTestCase) -> Vec<Finding> {
    run_audit_with(case.input, register_for(case))
}

fn is_flagged(findings: &[Finding]) -> bool {
    !findings.is_empty()
}

fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Áp dụng lặp các AutoFix (đã sắp xếp, bỏ overlap) cho đến khi hội tụ.
fn apply_all_fixes(input: &str, register: Register) -> String {
    let mut current = input.to_string();

    for _ in 0..5 {
        let findings = run_audit_with(&current, register);
        let mut fixes: Vec<(usize, usize, String)> = findings
            .into_iter()
            .filter_map(|f| f.fix.map(|fx| (fx.start, fx.end, fx.replacement)))
            .filter(|(s, e, _)| s < e)
            .collect();

        fixes.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));
        let mut chosen: Vec<(usize, usize, String)> = Vec::new();
        let mut last_end = 0;
        for fix in fixes {
            if fix.0 >= last_end {
                last_end = fix.1;
                chosen.push(fix);
            }
        }
        if chosen.is_empty() {
            break;
        }

        let byte_offsets: Vec<usize> = current
            .char_indices()
            .map(|(b, _)| b)
            .chain(std::iter::once(current.len()))
            .collect();
        let mut next = current.clone();
        for (start, end, replacement) in chosen.iter().rev() {
            if *end >= byte_offsets.len() {
                continue;
            }
            next.replace_range(byte_offsets[*start]..byte_offsets[*end], replacement);
        }
        if next == current {
            break;
        }
        current = next;
    }

    current
}

fn fix_is_correct(case: &GoldenTestCase, expected: &str) -> bool {
    normalize_ws(&apply_all_fixes(case.input, register_for(case))) == normalize_ws(expected)
}

/// Vùng văn bản thực sự bị thay đổi (char range trong input), tính bằng
/// tiền tố/hậu tố chung. Dùng để kiểm tra finding có "đúng chỗ" hay không,
/// thay vì chỉ kiểm tra "có kêu hay không".
fn changed_region(input: &str, expected: &str) -> (usize, usize) {
    let a: Vec<char> = input.chars().collect();
    let b: Vec<char> = expected.chars().collect();
    let mut prefix = 0;
    while prefix < a.len() && prefix < b.len() && a[prefix] == b[prefix] {
        prefix += 1;
    }
    let mut suffix = 0;
    while suffix < a.len().saturating_sub(prefix)
        && suffix < b.len().saturating_sub(prefix)
        && a[a.len() - 1 - suffix] == b[b.len() - 1 - suffix]
    {
        suffix += 1;
    }
    (prefix, a.len() - suffix)
}

fn finding_hits_region(findings: &[Finding], start: usize, end: usize) -> bool {
    findings.iter().any(|f| {
        if start == end {
            f.start <= start && f.end >= end
        } else {
            f.start < end && f.end > start
        }
    })
}

// ============================================================================
// METRIC 1: TỶ LỆ BÁO NHẦM (FALSE POSITIVE RATE)
// ============================================================================

#[test]
fn metric_1_false_positive_rate() {
    let cases = all_test_cases();
    let correct_cases: Vec<_> = cases.iter().filter(|c| !c.expected_flag).collect();

    let mut false_positives = 0;
    let mut details = Vec::new();

    for case in &correct_cases {
        let findings = run_audit(case);
        if is_flagged(&findings) {
            false_positives += 1;
            details.push(format!(
                "  ❌ [{}] BÁO NHẦM: {:?}\n     Findings: {:?}",
                case.id,
                case.input,
                findings
                    .iter()
                    .map(|f| (f.pattern_id.as_str(), f.matched_span.as_str()))
                    .collect::<Vec<_>>()
            ));
        }
    }

    let total = correct_cases.len();
    let fp_rate = false_positives as f64 / total as f64 * 100.0;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  METRIC 1: TỶ LỆ BÁO NHẦM (yêu cầu = 0%)                ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!(
        "║  Tổng câu đúng test: {} (gồm 25 bo1 + 6 trap)           ║",
        total
    );
    println!(
        "║  Báo nhầm: {} / {}                                       ║",
        false_positives, total
    );
    println!(
        "║  Tỷ lệ báo nhầm: {:.2}%                                  ║",
        fp_rate
    );
    println!(
        "║  Cận trên 95% (quy tắc 3): {:.1}% với n={}              ║",
        3.0 / total as f64 * 100.0,
        total
    );
    println!("╚══════════════════════════════════════════════════════════╝");

    if !details.is_empty() {
        println!("\nChi tiết các ca báo nhầm:");
        for d in details {
            println!("{}", d);
        }
    }

    assert_eq!(
        false_positives, 0,
        "Có {} ca báo nhầm - phải sửa!",
        false_positives
    );
}

// ============================================================================
// METRIC 2: TỶ LỆ PHÁT HIỆN (DETECTION RATE)
// ============================================================================

#[test]
fn metric_2_detection_rate() {
    let cases = all_test_cases();
    let wrong_cases: Vec<_> = cases.iter().filter(|c| c.expected_flag).collect();

    let mut detected = 0;
    let mut located = 0;
    let mut missed = Vec::new();
    let mut wrong_place = Vec::new();

    for case in &wrong_cases {
        let findings = run_audit(case);
        if is_flagged(&findings) {
            detected += 1;
            // "Phát hiện đúng chỗ": finding phải trùm lên vùng văn bản bị sửa.
            if let Some(expected) = case.expected_fix {
                let (start, end) = changed_region(case.input, expected);
                if finding_hits_region(&findings, start, end) {
                    located += 1;
                } else {
                    wrong_place.push(format!(
                        "  ⚠️ [{}] KÊU SAI CHỖ: {:?} (vùng đúng: {}..{})",
                        case.id, case.input, start, end
                    ));
                }
            }
        } else {
            missed.push(format!("  ❌ [{}] BỎ SÓT: {:?}", case.id, case.input));
        }
    }

    let total = wrong_cases.len();
    let rate = detected as f64 / total as f64 * 100.0;
    let located_rate = located as f64 / total as f64 * 100.0;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  METRIC 2: TỶ LỆ PHÁT HIỆN                               ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!(
        "║  Tổng câu sai: {} (25 bo1 + 50 bo2)                     ║",
        total
    );
    println!(
        "║  Phát hiện: {} / {}                                       ║",
        detected, total
    );
    println!(
        "║  Tỷ lệ phát hiện: {:.2}%                                 ║",
        rate
    );
    println!(
        "║  Đúng vị trí lỗi: {} / {} ({:.2}%)                       ║",
        located, total, located_rate
    );
    println!("╚══════════════════════════════════════════════════════════╝");

    if !wrong_place.is_empty() {
        println!("\nChi tiết các ca kêu nhưng lệch vị trí:");
        for w in &wrong_place {
            println!("{}", w);
        }
    }
    if !missed.is_empty() {
        println!("\nChi tiết các ca bỏ sót:");
        for m in missed.iter().take(20) {
            println!("{}", m);
        }
        if missed.len() > 20 {
            println!("  ... và {} ca khác", missed.len() - 20);
        }
    }
}

// ============================================================================
// METRIC 3: TỶ LỆ SỬA ĐÚNG (FIX ACCURACY)
// ============================================================================

#[test]
fn metric_3_fix_accuracy() {
    let cases = all_test_cases();
    let wrong_cases: Vec<_> = cases.iter().filter(|c| c.expected_flag).collect();

    let mut correct_fix = 0;
    let mut wrong_fix = Vec::new();

    for case in &wrong_cases {
        if let Some(expected) = case.expected_fix {
            if fix_is_correct(case, expected) {
                correct_fix += 1;
            } else {
                wrong_fix.push(format!(
                    "  ⚠️ [{}] SỬA SAI/CHƯA SỬA\n     Input:    {:?}\n     Expected: {:?}\n     Got:      {:?}",
                    case.id,
                    case.input,
                    expected,
                    apply_all_fixes(case.input, register_for(case))
                ));
            }
        }
    }

    let total_with_fix = wrong_cases
        .iter()
        .filter(|c| c.expected_fix.is_some())
        .count();
    let rate = correct_fix as f64 / total_with_fix as f64 * 100.0;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  METRIC 3: TỶ LỆ SỬA ĐÚNG                                ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!(
        "║  Tổng câu có expected_fix: {}                            ║",
        total_with_fix
    );
    println!(
        "║  Sửa đúng: {} / {}                                       ║",
        correct_fix, total_with_fix
    );
    println!(
        "║  Tỷ lệ sửa đúng: {:.2}%                                  ║",
        rate
    );
    println!("╚══════════════════════════════════════════════════════════╝");

    if !wrong_fix.is_empty() {
        println!("\nChi tiết các ca sửa sai (có ích để debug):");
        for w in wrong_fix.iter().take(40) {
            println!("{}", w);
        }
    }
}

// ============================================================================
// METRIC 4: BREAKDOWN THEO NHÓM
// ============================================================================

#[test]
fn metric_4_breakdown_by_category() {
    let cases = all_test_cases();
    let mut stats: HashMap<&str, (usize, usize, usize)> = HashMap::new();

    for case in &cases {
        if !case.expected_flag {
            continue;
        }
        let entry = stats.entry(case.category).or_insert((0, 0, 0));
        entry.0 += 1;
        let findings = run_audit(case);
        if is_flagged(&findings) {
            entry.1 += 1;
            if let Some(expected) = case.expected_fix {
                if fix_is_correct(case, expected) {
                    entry.2 += 1;
                }
            }
        }
    }

    println!("\n╔═══════════════════════════════════════════════════════════════════════════════╗");
    println!("║              METRIC 4: BREAKDOWN THEO NHÓM LỖI                                 ║");
    println!("╠═══════════════════════════════════════════════════════════════════════════════╣");
    println!(
        "║ {:<24} │ {:>4} │ {:>10} │ {:>10} ║",
        "Category", "Tổng", "Phát hiện", "Sửa đúng"
    );
    println!("╠═══════════════════════════════════════════════════════════════════════════════╣");

    let mut sorted: Vec<_> = stats.into_iter().collect();
    sorted.sort_by_key(|(k, _)| *k);

    for (cat, (total, detected, fixed)) in sorted {
        println!(
            "║ {:<24} │ {:>4} │ {:>4}/{:<5} │ {:>4}/{:<5} ║",
            cat, total, detected, total, fixed, total
        );
    }
    println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
}

// ============================================================================
// METRIC 5: LỖI "REAL-WORD" (★)
// ============================================================================

#[test]
fn metric_5_real_word_errors() {
    let cases = all_test_cases();
    let real_word_cases: Vec<_> = cases
        .iter()
        .filter(|c| c.expected_flag && c.is_real_word_error)
        .collect();

    let mut detected = 0;
    let mut missed = Vec::new();

    for case in &real_word_cases {
        let findings = run_audit(case);
        if is_flagged(&findings) {
            detected += 1;
        } else {
            missed.push(format!("  ★ [{}] BỎ SÓT: {:?}", case.id, case.input));
        }
    }

    let total = real_word_cases.len();
    let rate = if total > 0 {
        detected as f64 / total as f64 * 100.0
    } else {
        0.0
    };

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  METRIC 5: LỖI REAL-WORD (★)                             ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!(
        "║  Tổng ca ★: {}                                            ║",
        total
    );
    println!(
        "║  Phát hiện: {} / {}                                       ║",
        detected, total
    );
    println!(
        "║  Tỷ lệ: {:.2}%                                            ║",
        rate
    );
    println!("╚══════════════════════════════════════════════════════════╝");

    if !missed.is_empty() {
        println!("\nCác ca ★ bị bỏ sót:");
        for m in missed {
            println!("{}", m);
        }
    }
}

// ============================================================================
// METRIC 7: PHÁT HIỆN THEO TẦNG (engine layer)
// ============================================================================

#[test]
fn metric_7_detection_by_engine_layer() {
    let cases = all_test_cases();
    let mut layer_hits: HashMap<&'static str, usize> = HashMap::new();
    let mut pos_cases: Vec<&str> = Vec::new();

    for case in cases.iter().filter(|c| c.expected_flag) {
        let findings = run_audit(case);
        let mut sources: Vec<&'static str> = findings
            .iter()
            .filter(|f| {
                if let Some(expected) = case.expected_fix {
                    let (start, end) = changed_region(case.input, expected);
                    finding_hits_region(std::slice::from_ref(f), start, end)
                } else {
                    true
                }
            })
            .map(|f| match f.stage.as_deref().unwrap_or(f.source.as_str()) {
                "real_word" | "aho-corasick" => "aho-corasick",
                "admin_format" | "regex" => "regex",
                "pos_structure" | "pos" => "pos",
                "spellcheck" => "spellcheck",
                "bigram" => "bigram (context)",
                "orthography" => "orthography",
                "register" | "teencode" => "teencode",
                _ => "linguistic / khác",
            })
            .collect();
        sources.sort_unstable();
        sources.dedup();
        for source in &sources {
            *layer_hits.entry(source).or_insert(0) += 1;
        }
        if sources.contains(&"pos") {
            pos_cases.push(case.id);
        }
    }

    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║  METRIC 7: TẦNG NÀO PHÁT HIỆN (trên 75 ca sai)                ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    let mut rows: Vec<_> = layer_hits.into_iter().collect();
    rows.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    for (layer, count) in rows {
        println!("║  {:<55} {:>3} ca ║", layer, count);
    }
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Ca do POS cấu trúc bắt: {:<35} ║",
        if pos_cases.is_empty() {
            "0 (không ca nào trong bộ 106)".to_string()
        } else {
            pos_cases.join(", ")
        }
    );
    println!("╚═══════════════════════════════════════════════════════════════╝");
}

// ============================================================================
// METRIC 6: AN TOÀN ĐIỂM SỐ (KHÔNG CÓ LỖI MAJOR/CRITICAL TRÊN VĂN BẢN SẠCH,
// TEENCODE CHỈ ADVISORY) — "cổng chống 1 sao"
// ============================================================================

#[test]
fn metric_6_score_safety_and_advisory_floor() {
    let cases = all_test_cases();
    let mut violations = Vec::new();

    // 6.1. Câu đúng + trap: tuyệt đối không có Major/Critical.
    for case in cases.iter().filter(|c| !c.expected_flag) {
        for f in run_audit(case) {
            if matches!(f.severity.as_str(), "major" | "critical") {
                violations.push(format!(
                    "  ❌ [{}] {} ({}) trên câu sạch: {:?}",
                    case.id, f.pattern_id, f.severity, case.input
                ));
            }
        }
    }

    // 6.2. Comment mạng xã hội: chỉ Advisory, điểm vẫn 100, và phải nhận diện
    // được teencode (không im lặng bỏ qua).
    for text in [
        "Mk ko bt j ca",
        "Hôm nay mk vs ny đi chơi",
        "Dc rồi nha",
        "Tôi ko dc đi",
    ] {
        let report = Auditor::default().audit_document(text);
        for f in &report.findings {
            if f.severity != "advisory" {
                violations.push(format!(
                    "  ❌ Teencode bị gắn '{}' (phải advisory): {} | {:?}",
                    f.severity, f.pattern_id, text
                ));
            }
        }
        if !report.findings.iter().any(|f| f.category == "teencode") {
            violations.push(format!("  ❌ Bỏ sót teencode: {:?}", text));
        }
        if report.score != 100.0 {
            violations.push(format!(
                "  ❌ Điểm bị trừ oan trên teencode: {} | {:?}",
                report.score, text
            ));
        }
    }

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  METRIC 6: AN TOÀN ĐIỂM SỐ (chống 1 sao)                ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!(
        "║  Vi phạm: {}                                              ║",
        violations.len()
    );
    println!("╚══════════════════════════════════════════════════════════╝");

    for v in &violations {
        println!("{}", v);
    }

    assert!(
        violations.is_empty(),
        "Có {} vi phạm an toàn điểm số",
        violations.len()
    );
}
