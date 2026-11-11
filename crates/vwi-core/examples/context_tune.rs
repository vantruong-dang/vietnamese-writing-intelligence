#[cfg(feature = "context-lm")]
use std::fs;
#[cfg(feature = "context-lm")]
use vwi_core::rules::context_spellcheck::evaluate_context_spellcheck_with;

#[cfg(feature = "context-lm")]
fn main() {
    let text = fs::read_to_string("data_processed/dev_hj.txt").expect("dev_hj.txt");
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    println!("dev_lines={}", lines.len());

    for min_abs in [-3.0f32, -4.0, -5.0, -6.0, -7.0, -8.0] {
        for margin in [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0] {
            let mut fp_lines = 0usize;
            let mut fp_findings = 0usize;
            for line in &lines {
                let f = evaluate_context_spellcheck_with(line, min_abs, margin);
                if !f.is_empty() {
                    fp_lines += 1;
                    fp_findings += f.len();
                }
            }
            println!(
                "min_abs={:<6} margin={:<4} fp_lines={:<3} fp_findings={}",
                min_abs, margin, fp_lines, fp_findings
            );
        }
    }

    let chosen = (-4.0f32, 10.0f32);
    println!("\nFP detail at {:?}", chosen);
    for line in &lines {
        let f = evaluate_context_spellcheck_with(line, chosen.0, chosen.1);
        if !f.is_empty() {
            println!(
                "  {:?} -> {:?}",
                line,
                f.iter()
                    .map(|x| (x.matched_span.clone(), x.suggestion.clone()))
                    .collect::<Vec<_>>()
            );
        }
    }

    println!("\nH real-word at {:?}", chosen);
    let h = [
        ("H29", "Cơn bảo đang tiến vào miền Trung.", "bão"),
        ("H30", "Mùa hè, nhiều gia đình đưa con đi tám biển.", "tắm"),
        ("H31", "Bác sĩ dặn tôi uống thuốc sao bữa ăn.", "sau"),
        ("H32", "Tôi thường tập thể dục vào buổi sang.", "sáng"),
        ("H33", "Hàng xóm vừa chuyển đến căn nhà bên canh.", "cạnh"),
        ("H34", "Cô ấy mặc một chiếc vay màu đỏ đi dự tiệc.", "váy"),
        ("H35", "Chị ấy dậy sớm đẻ nấu cơm cho cả nhà.", "để"),
        ("H36", "Trời hôm nay rất nạnh, nhớ mặc áo ấm.", "lạnh"),
        ("H37", "Bạn nhớ mang theo dấy tờ tùy thân.", "giấy"),
        ("H38", "Anh ấy mượn một cuống sách ở thư viện.", "cuốn"),
    ];
    let mut hit = 0;
    for (id, text, expected) in h {
        let f = evaluate_context_spellcheck_with(text, chosen.0, chosen.1);
        let ok = f.iter().any(|x| x.suggestion.as_deref() == Some(expected));
        if ok {
            hit += 1;
        }
        println!(
            "  {} hit={} {:?}",
            id,
            ok,
            f.iter().map(|x| x.suggestion.clone()).collect::<Vec<_>>()
        );
    }
    println!("H real-word hit={}/10", hit);
}

#[cfg(not(feature = "context-lm"))]
fn main() {
    eprintln!("Bật feature `context-lm` và copy data_processed/bigram_lm.tsv vào crates/vwi-core/data/ để chạy.");
}
