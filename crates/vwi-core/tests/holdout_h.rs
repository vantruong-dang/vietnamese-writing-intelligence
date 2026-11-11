//! # BỘ H — HOLD-OUT 50 CA (20 đúng / 30 sai)
//!
//! Bộ này được soạn độc lập với 106 ca dev, khác bề mặt từ vựng. Mục đích:
//! phát hiện overfitting ("học vẹt") của lexicon/collocation.
//!
//! QUAN TRỌNG: file này được chạy **trước** khi bổ sung bất kỳ collocation nào
//! cho bộ H. Kết quả baseline (5/30, ★0/10) mới là generalization thật.
//! Sau remediation, H **không còn blind**; chỉ dùng chẩn đoán, không dùng để
//! chọn threshold. Context bigram đo trên VSEC, không tune trên H.

use std::collections::HashMap;
use vwi_core::auditor::{Auditor, AuditorConfig};
use vwi_core::types::{Finding, Register};

#[derive(Debug)]
#[allow(dead_code)] // `category` giữ để đối chiếu nhóm khi cần
struct HoldoutCase {
    id: &'static str,
    input: &'static str,
    expected_flag: bool,
    expected_fix: Option<&'static str>,
    category: &'static str,
    admin_register: bool,
    real_word: bool,
}

fn cases() -> Vec<HoldoutCase> {
    use HoldoutCase as C;
    vec![
        // ── 20 câu ĐÚNG ────────────────────────────────────────────────────
        C {
            id: "H01",
            input: "Anh ấy đứng thứ ba trong cuộc thi chạy bộ.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_ordinal",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H02",
            input: "Các cơ quan nhà nước phải công khai thông tin.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_modifier",
            admin_register: true,
            real_word: false,
        },
        C {
            id: "H03",
            input: "Cuộc họp bắt đầu lúc 8 giờ 30 phút sáng nay.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_time",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H04",
            input: "Công ty vừa ký hợp đồng trị giá 2,5 tỷ đồng.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_decimal",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H05",
            input: "Hà Nội, ngày 03 tháng 12 năm 2026",
            expected_flag: false,
            expected_fix: None,
            category: "correct_date",
            admin_register: true,
            real_word: false,
        },
        C {
            id: "H06",
            input: "Căn cứ khoản 2 Điều 7 Luật Lưu trữ năm 2024.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_legal_ref",
            admin_register: true,
            real_word: false,
        },
        C {
            id: "H07",
            input: "Trận Hà Nội FC vs Hải Phòng diễn ra tối nay.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_sports_vs",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H08",
            input: "Bà ngoại kể chuyện cổ tích cho các cháu nghe.",
            expected_flag: false,
            expected_fix: None,
            category: "correct",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H09",
            input: "Thủ tướng Chính phủ chủ trì cuộc họp sáng nay.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_title",
            admin_register: true,
            real_word: false,
        },
        C {
            id: "H10",
            input: "Thành phố Hồ Chí Minh là trung tâm kinh tế lớn của cả nước.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_place",
            admin_register: true,
            real_word: false,
        },
        C {
            id: "H11",
            input: "Bà cụ khóa cửa rồi đi chợ, tay cầm một chiếc khóa cũ.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_homonym",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H12",
            input: "Hôm qua tôi mới biết mai trời sẽ mưa.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_reported_speech",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H13",
            input: "Quý khách vui lòng đợi trong giây lát.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_polite",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H14",
            input: "Tổng chi phí khoảng 1.250.000 đồng.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_thousands",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H15",
            input: "Nhân dân ta luôn đoàn kết một lòng.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_capitalized",
            admin_register: true,
            real_word: false,
        },
        C {
            id: "H16",
            input: "Cô giáo cho cả lớp nghỉ sớm vì trời mưa to.",
            expected_flag: false,
            expected_fix: None,
            category: "correct",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H17",
            input: "Anh ấy chạy rất nhanh nên về nhất.",
            expected_flag: false,
            expected_fix: None,
            category: "correct",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H18",
            input: "Kính gửi: các sở, ban, ngành thuộc tỉnh.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_salutation_lower",
            admin_register: true,
            real_word: false,
        },
        C {
            id: "H19",
            input: "Kính gửi: Các sở, ban, ngành thuộc tỉnh.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_salutation_upper",
            admin_register: true,
            real_word: false,
        },
        C {
            id: "H20",
            input: "Ông Trần Quang Minh sinh năm 1960 tại Cần Thơ.",
            expected_flag: false,
            expected_fix: None,
            category: "correct_proper_name",
            admin_register: false,
            real_word: false,
        },
        // ── 30 câu SAI ─────────────────────────────────────────────────────
        C {
            id: "H21",
            input: "Bạn hãy ngồi xuông ghế chờ nhé.",
            expected_flag: true,
            expected_fix: Some("Bạn hãy ngồi xuống ghế chờ nhé."),
            category: "dau_thanh",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H22",
            input: "Chiếc cầu bắc qua sông rất dài và vửng chắc.",
            expected_flag: true,
            expected_fix: Some("Chiếc cầu bắc qua sông rất dài và vững chắc."),
            category: "dau_thanh",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H23",
            input: "Tôi rất thích món phỡ bò ở quán này.",
            expected_flag: true,
            expected_fix: Some("Tôi rất thích món phở bò ở quán này."),
            category: "dau_thanh",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H24",
            input: "Cậu bé rất nười học bài.",
            expected_flag: true,
            expected_fix: Some("Cậu bé rất lười học bài."),
            category: "phu_am_dau",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H25",
            input: "Xin cho phép tôi chình bày ý kiến.",
            expected_flag: true,
            expected_fix: Some("Xin cho phép tôi trình bày ý kiến."),
            category: "phu_am_dau",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H26",
            input: "Bà nội tôi ngĩ rằng chiều nay trời sẽ mưa.",
            expected_flag: true,
            expected_fix: Some("Bà nội tôi nghĩ rằng chiều nay trời sẽ mưa."),
            category: "phu_am_dau",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H27",
            input: "Cô ấy mua một chiếc kặp màu xanh.",
            expected_flag: true,
            expected_fix: Some("Cô ấy mua một chiếc cặp màu xanh."),
            category: "phu_am_dau",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H28",
            input: "Em ngồi trên chiếc gế gỗ nhỏ.",
            expected_flag: true,
            expected_fix: Some("Em ngồi trên chiếc ghế gỗ nhỏ."),
            category: "phu_am_dau",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H29",
            input: "Cơn bảo đang tiến vào miền Trung.",
            expected_flag: true,
            expected_fix: Some("Cơn bão đang tiến vào miền Trung."),
            category: "real_word",
            admin_register: false,
            real_word: true,
        },
        C {
            id: "H30",
            input: "Mùa hè, nhiều gia đình đưa con đi tám biển.",
            expected_flag: true,
            expected_fix: Some("Mùa hè, nhiều gia đình đưa con đi tắm biển."),
            category: "real_word",
            admin_register: false,
            real_word: true,
        },
        C {
            id: "H31",
            input: "Bác sĩ dặn tôi uống thuốc sao bữa ăn.",
            expected_flag: true,
            expected_fix: Some("Bác sĩ dặn tôi uống thuốc sau bữa ăn."),
            category: "real_word",
            admin_register: false,
            real_word: true,
        },
        C {
            id: "H32",
            input: "Tôi thường tập thể dục vào buổi sang.",
            expected_flag: true,
            expected_fix: Some("Tôi thường tập thể dục vào buổi sáng."),
            category: "real_word",
            admin_register: false,
            real_word: true,
        },
        C {
            id: "H33",
            input: "Hàng xóm vừa chuyển đến căn nhà bên canh.",
            expected_flag: true,
            expected_fix: Some("Hàng xóm vừa chuyển đến căn nhà bên cạnh."),
            category: "real_word",
            admin_register: false,
            real_word: true,
        },
        C {
            id: "H34",
            input: "Cô ấy mặc một chiếc vay màu đỏ đi dự tiệc.",
            expected_flag: true,
            expected_fix: Some("Cô ấy mặc một chiếc váy màu đỏ đi dự tiệc."),
            category: "real_word",
            admin_register: false,
            real_word: true,
        },
        C {
            id: "H35",
            input: "Chị ấy dậy sớm đẻ nấu cơm cho cả nhà.",
            expected_flag: true,
            expected_fix: Some("Chị ấy dậy sớm để nấu cơm cho cả nhà."),
            category: "real_word",
            admin_register: false,
            real_word: true,
        },
        C {
            id: "H36",
            input: "Trời hôm nay rất nạnh, nhớ mặc áo ấm.",
            expected_flag: true,
            expected_fix: Some("Trời hôm nay rất lạnh, nhớ mặc áo ấm."),
            category: "real_word",
            admin_register: false,
            real_word: true,
        },
        C {
            id: "H37",
            input: "Bạn nhớ mang theo dấy tờ tùy thân.",
            expected_flag: true,
            expected_fix: Some("Bạn nhớ mang theo giấy tờ tùy thân."),
            category: "real_word",
            admin_register: false,
            real_word: true,
        },
        C {
            id: "H38",
            input: "Anh ấy mượn một cuống sách ở thư viện.",
            expected_flag: true,
            expected_fix: Some("Anh ấy mượn một cuốn sách ở thư viện."),
            category: "real_word",
            admin_register: false,
            real_word: true,
        },
        C {
            id: "H39",
            input: "Anh ấy hát hay rất.",
            expected_flag: true,
            expected_fix: Some("Anh ấy hát rất hay."),
            category: "ngu_phap",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H40",
            input: "Chúng ta cần cần hoàn thành bài tập.",
            expected_flag: true,
            expected_fix: Some("Chúng ta cần hoàn thành bài tập."),
            category: "ngu_phap",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H41",
            input: "Cô ấy nuôi hai con mèo và một cây chó.",
            expected_flag: true,
            expected_fix: Some("Cô ấy nuôi hai con mèo và một con chó."),
            category: "ngu_phap",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H42",
            input: "Anh ấy được phạt vì đi sai làn đường.",
            expected_flag: true,
            expected_fix: Some("Anh ấy bị phạt vì đi sai làn đường."),
            category: "ngu_phap",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H43",
            input: "Ngày mai tôi đã gặp thầy giáo.",
            expected_flag: true,
            expected_fix: Some("Ngày mai tôi sẽ gặp thầy giáo."),
            category: "ngu_phap",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H44",
            input: "Tôi đã đã gửi thư cho cô ấy.",
            expected_flag: true,
            expected_fix: Some("Tôi đã gửi thư cho cô ấy."),
            category: "ngu_phap",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H45",
            input: "Cần Thơ, ngày 8 tháng 12 năm 2026",
            expected_flag: true,
            expected_fix: Some("Cần Thơ, ngày 08 tháng 12 năm 2026"),
            category: "R1",
            admin_register: true,
            real_word: false,
        },
        C {
            id: "H46",
            input: "Huế, ngày 21 tháng 2 năm 2027",
            expected_flag: true,
            expected_fix: Some("Huế, ngày 21 tháng 02 năm 2027"),
            category: "R1",
            admin_register: true,
            real_word: false,
        },
        C {
            id: "H47",
            input: "Theo Khoản 3 Điều 12 của văn bản này.",
            expected_flag: true,
            expected_fix: Some("Theo khoản 3 Điều 12 của văn bản này."),
            category: "R3",
            admin_register: true,
            real_word: false,
        },
        C {
            id: "H48",
            input: "Mọi chính sách đều vì lợi ích của nhân dân.",
            expected_flag: true,
            expected_fix: Some("Mọi chính sách đều vì lợi ích của Nhân dân."),
            category: "R4",
            admin_register: true,
            real_word: false,
        },
        C {
            id: "H49",
            input: "Cuộc họp bắt đầu lúc 8 giờ. chủ tọa phát biểu khai mạc.",
            expected_flag: true,
            expected_fix: Some("Cuộc họp bắt đầu lúc 8 giờ. Chủ tọa phát biểu khai mạc."),
            category: "R6",
            admin_register: false,
            real_word: false,
        },
        C {
            id: "H50",
            input: "Số: 31/KH–UBND",
            expected_flag: true,
            expected_fix: Some("Số: 31/KH-UBND"),
            category: "R8",
            admin_register: true,
            real_word: false,
        },
    ]
}

fn run_audit(case: &HoldoutCase) -> Vec<Finding> {
    let register = if case.admin_register {
        Register::Administrative
    } else {
        Register::General
    };
    Auditor::with_config(AuditorConfig {
        register,
        ..Default::default()
    })
    .audit(case.input)
}

fn audit_with(text: &str, admin: bool) -> Vec<Finding> {
    Auditor::with_config(AuditorConfig {
        register: if admin {
            Register::Administrative
        } else {
            Register::General
        },
        ..Default::default()
    })
    .audit(text)
}

fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn apply_all_fixes(case: &HoldoutCase) -> String {
    let mut current = case.input.to_string();
    for _ in 0..5 {
        let findings = audit_with(&current, case.admin_register);
        let mut fixes: Vec<(usize, usize, String)> = findings
            .into_iter()
            .filter_map(|f| f.fix.map(|fx| (fx.start, fx.end, fx.replacement)))
            .filter(|(s, e, _)| s < e)
            .collect();
        fixes.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));
        let mut chosen = Vec::new();
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

#[test]
fn holdout_h_scorecard() {
    let cases = cases();
    let correct: Vec<_> = cases.iter().filter(|c| !c.expected_flag).collect();
    let wrong: Vec<_> = cases.iter().filter(|c| c.expected_flag).collect();

    let mut fp = 0;
    let mut fp_detail = Vec::new();
    for case in &correct {
        let findings = run_audit(case);
        if !findings.is_empty() {
            fp += 1;
            fp_detail.push(format!(
                "  FP [{}] {:?} -> {:?}",
                case.id,
                case.input,
                findings
                    .iter()
                    .map(|f| (f.pattern_id.as_str(), f.matched_span.as_str()))
                    .collect::<Vec<_>>()
            ));
        }
    }

    let mut detected = 0;
    let mut fixed_ok = 0;
    let mut detected_real_word = 0;
    let mut detail = Vec::new();
    let mut layers: HashMap<&'static str, usize> = HashMap::new();

    for case in &wrong {
        let findings = run_audit(case);
        if findings.is_empty() {
            detail.push(format!("  MISS [{}] {:?}", case.id, case.input));
            continue;
        }
        detected += 1;
        for layer in findings
            .iter()
            .map(|f| match f.stage.as_deref().unwrap_or(f.source.as_str()) {
                "aho-corasick" | "real_word" => "aho-corasick",
                "regex" | "admin_format" => "regex",
                "pos" | "pos_structure" => "pos",
                "spellcheck" => "spellcheck",
                "confusion" => "confusion",
                "tense" => "tense",
                "bigram" => "bigram (context)",
                "register" | "teencode" => "teencode",
                _ => "khác",
            })
            .collect::<std::collections::HashSet<_>>()
        {
            *layers.entry(layer).or_insert(0) += 1;
        }
        if case.real_word {
            detected_real_word += 1;
        }
        if let Some(expected) = case.expected_fix {
            let got = apply_all_fixes(case);
            if normalize_ws(&got) == normalize_ws(expected) {
                fixed_ok += 1;
            } else {
                detail.push(format!(
                    "  WRONGFIX [{}] {:?}\n      expected {:?}\n      got      {:?}",
                    case.id, case.input, expected, got
                ));
            }
        }
    }

    let real_word_total = wrong.iter().filter(|c| c.real_word).count();
    println!("\n===== BỘ H — HOLDOUT SCORECARD =====");
    println!(
        "Đúng (20): báo nhầm {} / {}  (FPR {:.1}%)",
        fp,
        correct.len(),
        fp as f64 / correct.len() as f64 * 100.0
    );
    println!(
        "Sai (30): phát hiện {} / {}  ({:.1}%)",
        detected,
        wrong.len(),
        detected as f64 / wrong.len() as f64 * 100.0
    );
    println!(
        "Sửa đúng: {} / {}  ({:.1}%)",
        fixed_ok,
        wrong.len(),
        fixed_ok as f64 / wrong.len() as f64 * 100.0
    );
    println!(
        "Real-word ★: {} / {}  ({:.1}%)",
        detected_real_word,
        real_word_total,
        detected_real_word as f64 / real_word_total as f64 * 100.0
    );
    let mut layer_rows: Vec<_> = layers.into_iter().collect();
    layer_rows.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    println!("--- tầng phát hiện (ca) ---");
    for (layer, count) in layer_rows {
        println!("  {:<16} {}", layer, count);
    }
    println!("--- chi tiết ---");
    for d in fp_detail.iter().chain(detail.iter()) {
        println!("{}", d);
    }

    // Không assert cứng: đây là phép đo, không phải gate.
}
