import pytest
import vn_writing_intel as vwi


@pytest.fixture(scope="module")
def auditor():
    return vwi.Auditor()


def get_rule_ids(auditor, text: str) -> list[str]:
    findings = auditor.audit(text)
    return [f.rule_id for f in findings]


# ==========================================
# BẢNG A: BẪY BẮT OAN (Kỳ vọng: 0 lỗi)
# ==========================================


def test_ca_01_abbreviations_and_titles(auditor):
    text = (
        "Theo GS.TS. Nguyễn Văn Nam, tại TP.HCM và các tỉnh lân cận, v.v. đều ghi nhận mật độ giao thông tăng cao"
        " tại Q.1 và P. Bến Nghé."
    )
    findings = get_rule_ids(auditor, text)
    assert len(findings) == 0, f"Bắt oan viết tắt/địa danh: {findings}"


def test_ca_02_time_ratio_and_decimal(auditor):
    text = (
        "Trận đấu bắt đầu lúc 19:30 với tỷ số 2:1 trên sân Mỹ Đình. Dự án quy hoạch tỷ lệ 1:5000 có tổng mức đầu tư"
        " 1,250,000,000 đồng, dung sai 3.14%."
    )
    findings = get_rule_ids(auditor, text)
    assert len(findings) == 0, f"Bắt oan số liệu/giờ giấc: {findings}"


def test_ca_03_legal_list_lowercase(auditor):
    text = (
        "Điều 1. Phạm vi điều chỉnh và đối tượng áp dụng:\na) Cán bộ, công chức tại các cơ quan hành chính nhà nước;\nb)"
        " Viên chức trong các đơn vị sự nghiệp công lập."
    )
    findings = get_rule_ids(auditor, text)
    assert len(findings) == 0, f"Bắt oan cấu trúc liệt kê a), b): {findings}"


def test_ca_04_brackets_and_quotes_spacing(auditor):
    text = (
        'Nguyễn Du dùng chữ "thì" (trong Truyện Kiều) rất tài tình... Đó chính là tác tử phân giới Đề - Thuyết của câu.'
    )
    findings = get_rule_ids(auditor, text)
    assert len(findings) == 0, f"Bắt oan ngoặc kép/ba chấm: {findings}"


def test_ca_05_url_and_code_masking(auditor):
    text = (
        "Tham khảo tài liệu tại https://chinhphu.vn/nghi-dinh-30. Khi lập trình hãy gọi hàm `process_data(x, y)` để"
        " kiểm tra."
    )
    findings = get_rule_ids(auditor, text)
    assert len(findings) == 0, f"Bắt oan URL/Code block: {findings}"


# ==========================================
# BẢNG B: LỖI THỰC TẾ (Kỳ vọng: Bắt đúng lỗi)
# ==========================================


def test_ca_06_punctuation_spacing(auditor):
    text = "Hội nghị đã kết thúc tốt đẹp ,mở ra nhiều cơ hội hợp tác .tuy nhiên vẫn còn nhiều thách thức ."
    findings = get_rule_ids(auditor, text)

    assert "PUNCTUATION_SPACE_BEFORE" in findings, "Bỏ sót lỗi thừa dấu cách trước dấu câu"
    assert "PUNCTUATION_NO_SPACE_AFTER" in findings, "Bỏ sót lỗi thiếu dấu cách sau dấu câu"
    assert "SENTENCE_CAPITALIZATION" in findings, "Bỏ sót lỗi không viết hoa sau dấu chấm"


def test_ca_07_bracket_spacing(auditor):
    text = (
        'Dự án đường vành đai 4 ( khởi công năm 2023 ) có ý nghĩa chiến lược đối với " vùng thủ đô " .'
    )
    findings = get_rule_ids(auditor, text)

    assert "BRACKET_SPACE_INSIDE" in findings, "Bỏ sót lỗi thừa dấu cách trong ngoặc"


def test_ca_08_decree30_capitalization(auditor):
    text = (
        "Hôm qua, Bộ Giáo Dục Và Đào Tạo đã có công văn gửi Ủy Ban Nhân Dân Tỉnh Quảng Ninh về việc tuyển sinh."
    )
    findings = get_rule_ids(auditor, text)

    assert "DECREE30_AGENCY_CAPITALIZATION" in findings, "Bỏ sót lỗi viết hoa cơ quan"
    assert "DECREE30_ADMIN_UNIT_LOWERCASE" in findings, "Bỏ sót lỗi viết hoa đơn vị hành chính"


def test_ca_09_phonotactic_telex_typos(auditor):
    text = "Doanh nghiệp dduowjc yêu cầu cung cấp đầỳ đủ hồ sơ tài chính trwowngf hợp có sai sót."
    findings = get_rule_ids(auditor, text)

    telex_errors = findings.count("TELEX_TYPO_ERROR")
    assert telex_errors >= 2, f"Bỏ sót lỗi gõ Telex (dduowjc, trwowngf). Tìm thấy: {telex_errors}"


def test_ca_10_tone_inconsistency(auditor):
    text = (
        "Chính phủ chủ trương xây dựng nền kinh tế hòa nhập quốc tế, đồng thời giữ vững môi trường hoà bình và ổn"
        " định lâu dài."
    )
    findings = get_rule_ids(auditor, text)

    assert "ORTHOGRAPHY_TONE_INCONSISTENCY" in findings, "Bỏ sót lỗi bất nhất quán dấu thanh (hòa vs hoà)"
