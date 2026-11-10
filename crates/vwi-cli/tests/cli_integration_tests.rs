use std::process::Command;

#[test]
fn test_cli_check_clean_text() {
    let output = Command::new(env!("CARGO_BIN_EXE_vwi"))
        .args(["check", "--text", "Đây là một câu văn hoàn toàn bình thường, diễn đạt tự nhiên và chuẩn mực."])
        .output()
        .expect("Không thể chạy lệnh vwi check");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Điểm chất lượng : 100.0/100"));
    assert!(stdout.contains("Không phát hiện lỗi cấu trúc hay sáo ngữ"));
}

#[test]
fn test_cli_check_with_violations() {
    let text = "Trong bối cảnh hiện nay đòi hỏi chúng ta phải tiến hành thực hiện việc tối ưu hóa quy trình. Đề án đã được duyệt bởi ban giám đốc.";
    let output = Command::new(env!("CARGO_BIN_EXE_vwi"))
        .args(["check", "--text", text])
        .output()
        .expect("Không thể chạy lệnh vwi check");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("CRIPPLED_PREPOSITIONAL_TOPIC") || stdout.contains("ARTIFICIAL_PASSIVE_BE_BY"));
    assert!(stdout.contains("LỖI PHÁT HIỆN"));
}

#[test]
fn test_cli_check_fail_under_ci() {
    let text = "Trong bối cảnh hiện nay đòi hỏi chúng ta phải tiến hành thực hiện việc tối ưu hóa quy trình.";
    let output = Command::new(env!("CARGO_BIN_EXE_vwi"))
        .args(["check", "--text", text, "--fail-under", "95.0"])
        .output()
        .expect("Không thể chạy lệnh vwi check");

    // Phải trả về exit code 1 vì điểm số < 95.0
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("CI FAILED"));
}

#[test]
fn test_cli_check_formats_json_and_markdown() {
    let text = "Phương án này là tối ưu nhất.";
    
    // 1. Kiểm tra JSON
    let json_output = Command::new(env!("CARGO_BIN_EXE_vwi"))
        .args(["check", "--text", text, "--format", "json"])
        .output()
        .expect("Không thể chạy vwi check format json");

    assert!(json_output.status.success());
    let json_str = String::from_utf8_lossy(&json_output.stdout);
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("Phải parse được JSON hợp lệ");
    assert!(val.get("score").is_some());
    assert!(val.get("findings").is_some());

    // 2. Kiểm tra Markdown
    let md_output = Command::new(env!("CARGO_BIN_EXE_vwi"))
        .args(["check", "--text", text, "--format", "markdown"])
        .output()
        .expect("Không thể chạy vwi check format markdown");

    assert!(md_output.status.success());
    let md_str = String::from_utf8_lossy(&md_output.stdout);
    assert!(md_str.contains("# BÁO CÁO KIỂM TOÁN VĂN BẢN"));
    assert!(md_str.contains("| Vị trí | Mức độ |"));
}

#[test]
fn test_cli_audit_semantic_anchors_preserved_and_missing() {
    // 1. Dữ kiện mỏ neo được bảo toàn
    let good_text = "Năm 2026, doanh thu công ty cán mốc 150 tỷ đồng nhờ mở rộng thị trường.";
    let good_out = Command::new(env!("CARGO_BIN_EXE_vwi"))
        .args(["audit", "--text", good_text, "--anchor", "150 tỷ đồng"])
        .output()
        .expect("Không thể chạy vwi audit");

    assert!(good_out.status.success());

    // 2. Dữ kiện mỏ neo bị thất thoát trong bản viết lại -> Lập tức báo động FAILED (exit code 1)
    let bad_text = "Năm 2026, doanh thu công ty tăng trưởng mạnh nhờ mở rộng thị trường.";
    let bad_out = Command::new(env!("CARGO_BIN_EXE_vwi"))
        .args(["audit", "--text", bad_text, "--anchor", "150 tỷ đồng"])
        .output()
        .expect("Không thể chạy vwi audit");

    assert_eq!(bad_out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&bad_out.stderr);
    assert!(stderr.contains("AUDIT FAILED") || stderr.contains("thất thoát dữ kiện mỏ neo"));
}

#[test]
fn test_cli_tokenize_and_pos() {
    let output = Command::new(env!("CARGO_BIN_EXE_vwi"))
        .args(["tokenize", "Học sinh học sinh học", "--pos"])
        .output()
        .expect("Không thể chạy vwi tokenize");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[Học sinh/NOUN]") || stdout.contains("[học/VERB]"));

    let json_output = Command::new(env!("CARGO_BIN_EXE_vwi"))
        .args(["tokenize", "Bàn là một đồ vật", "--json"])
        .output()
        .expect("Không thể chạy vwi tokenize --json");

    assert!(json_output.status.success());
    let json_str = String::from_utf8_lossy(&json_output.stdout);
    let val: serde_json::Value = serde_json::from_str(&json_str).expect("Phải parse được JSON tokens");
    assert!(val.is_array());
}
