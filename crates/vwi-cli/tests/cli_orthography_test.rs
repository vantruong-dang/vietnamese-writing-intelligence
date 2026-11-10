use std::process::Command;

#[test]
fn test_cli_orthography_and_telex_rules() {
    let output = Command::new(env!("CARGO_BIN_EXE_vwi"))
        .args([
            "check",
            "--text",
            "Bộ Giáo Dục Và Đào Tạo đã gửi công văn dduowjc phê duyệt.",
        ])
        .output()
        .expect("Không thể chạy lệnh vwi check");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("DECREE30_AGENCY_CAPITALIZATION"), "Thiếu rule DECREE30_AGENCY_CAPITALIZATION: {}", stdout);
    assert!(stdout.contains("Bộ Giáo dục và Đào tạo"), "Thiếu gợi ý viết hoa cơ quan: {}", stdout);
    assert!(stdout.contains("TELEX_TYPO_ERROR"), "Thiếu rule TELEX_TYPO_ERROR: {}", stdout);
    assert!(stdout.contains("được"), "Thiếu gợi ý giải mã Telex: {}", stdout);
}
