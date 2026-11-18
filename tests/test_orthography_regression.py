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


def test_ca_09b_phonotactic_vni_typos(auditor):
    text = (
        "Doanh nghiệp d9u7o75c yêu cầu cung cấp hồ sơ tài chính tru7o72ng hợp có sai sót."
    )
    findings = [f for f in auditor.audit(text) if f.rule_id == "TELEX_TYPO_ERROR"]
    fixes = {f.matched_span: f.suggestion for f in findings}

    assert fixes.get("d9u7o75c") == "được", f"Bỏ sót lỗi gõ VNI d9u7o75c: {findings}"
    assert fixes.get("tru7o72ng") == "trường", f"Bỏ sót lỗi gõ VNI tru7o72ng: {findings}"


def test_vni_policy_c_rejects_single_tone_digit_codes(auditor):
    # Chính sách C: token chỉ có một phím thanh 1..5 và không có phím biến đổi
    # 6..9 (ban1, ca3, hoc5...) bị coi là mã/viết tắt, không phải lỗi gõ VNI.
    text = "Mã ban1, ca3, lo2, co2, hoc5, me5 đã được cập nhật vào biểu mẫu."
    findings = [f for f in auditor.audit(text) if f.rule_id == "TELEX_TYPO_ERROR"]
    assert not findings, f"LỖI BẮT OAN MÃ VNI MỘT CHỮ SỐ: {[f.rationale for f in findings]}"


def test_vni_policy_c_accepts_diacritic_digits(auditor):
    # Có phím biến đổi 6..9 (a8n, kho6ng) hoặc >= 2 chữ số (d9u7o75c) thì vẫn bắt.
    text = "Doanh nghiệp d9u7o75c yêu cầu a8n và kho6ng được chậm trễ."
    findings = [f for f in auditor.audit(text) if f.rule_id == "TELEX_TYPO_ERROR"]
    fixes = {f.matched_span: f.suggestion for f in findings}
    assert fixes.get("d9u7o75c") == "được", findings
    assert fixes.get("a8n") == "ăn", findings
    assert fixes.get("kho6ng") == "không", findings


def test_mixed_ime_tokens_are_detected(auditor):
    # "Vỡ bộ gõ nửa chừng": phần đầu đã ăn dấu cấu tạo, phần sau còn phím thô.
    text = (
        "Tôi đã đươcj, ngươif, viêjt, trươfng, đươ5c, viê5t và trươ2ng."
    )
    findings = [f for f in auditor.audit(text) if f.rule_id == "TELEX_TYPO_ERROR"]
    fixes = {f.matched_span: f.suggestion for f in findings}
    for raw, expected in [
        ("đươcj", "được"),
        ("ngươif", "người"),
        ("viêjt", "việt"),
        ("trươfng", "trường"),
        ("đươ5c", "được"),
        ("viê5t", "việt"),
        ("trươ2ng", "trường"),
    ]:
        assert fixes.get(raw) == expected, f"Bỏ sót mixed-token {raw}: {findings}"


def test_mixed_ime_tokens_preserve_case(auditor):
    # Đầu câu / tiêu đề: kiểu chữ hoa phải được bảo toàn.
    text = "Những người đồng nghiệp đã làm Viêjt và TRƯƠFNG, VIÊ5T, Đươcj."
    findings = [f for f in auditor.audit(text) if f.rule_id == "TELEX_TYPO_ERROR"]
    fixes = {f.matched_span: f.suggestion for f in findings}
    for raw, expected in [
        ("Viêjt", "Việt"),
        ("TRƯƠFNG", "TRƯỜNG"),
        ("VIÊ5T", "VIỆT"),
        ("Đươcj", "Được"),
    ]:
        assert fixes.get(raw) == expected, f"Sai kiểu chữ cho {raw}: {findings}"

    # Tên riêng / từ tiếng Anh viết hoa và âm tiết Việt hợp lệ: không báo.
    clean = "Lori, Air, Jensen, Windows, Word, Được, Trường, Người, Việt, UBND, VNĐ."
    assert not [
        f for f in auditor.audit(clean) if f.rule_id == "TELEX_TYPO_ERROR"
    ], "Bắt oan chữ viết hoa"


def test_mixed_ime_offsets_match_python_string_indices(auditor):
    # Rust trả offset theo ký tự (char). Python str index cũng là code point,
    # nên `text[start:end]` phải cắt đúng token lỗi kể cả khi có emoji trước đó.
    text = "😀 Những người đồng nghiệp đã làm viêjt rất tốt."
    findings = [f for f in auditor.audit(text) if f.rule_id == "TELEX_TYPO_ERROR"]
    assert len(findings) == 1, findings
    f = findings[0]
    assert text[f.start:f.end] == "viêjt", (f.start, f.end, text[f.start:f.end])
    assert f.suggestion == "việt"
    fixed = text[:f.start] + f.suggestion + text[f.end:]
    assert fixed == "😀 Những người đồng nghiệp đã làm việt rất tốt."


def test_footnote_digits_are_not_ime_typos(auditor):
    # Số chú thích dính sau âm tiết hoàn chỉnh không phải lỗi gõ bộ gõ.
    text = "Xem tầng5, chương2, việt5 và người2 trong tài liệu."
    findings = [f for f in auditor.audit(text) if f.rule_id == "TELEX_TYPO_ERROR"]
    assert not findings, f"LỖI BẮT OAN SỐ CHÚ THÍCH: {[f.rationale for f in findings]}"


def test_punctuation_no_false_positive_at_paragraph_break(auditor):
    # Dấu câu đứng ngay trước xuống dòng / ký tự vô hình copy từ web
    # (ZWSP, BOM, soft hyphen) không phải lỗi thiếu dấu cách.
    cases = [
        "Giá còn 47.000 đồng/cổ phiếu.\nCTCP FPT vừa công bố.",
        "Giá còn 47.000 đồng/cổ phiếu.\n\nCTCP FPT vừa công bố.",
        "Giá còn 47.000 đồng/cổ phiếu.\r\n\r\nCTCP FPT vừa công bố.",
        "Giá còn 47.000 đồng/cổ phiếu.\u200bCTCP FPT vừa công bố.",
        "Giá còn 47.000 đồng/cổ phiếu.\ufeffCTCP FPT vừa công bố.",
        "Giá còn 47.000 đồng/cổ phiếu.\u00adCTCP FPT vừa công bố.",
    ]
    for text in cases:
        findings = [
            f for f in auditor.audit(text) if f.rule_id == "PUNCTUATION_NO_SPACE_AFTER"
        ]
        assert not findings, f"Báo oan thiếu dấu cách: {text!r} -> {findings}"

    # Thiếu dấu cách thật (chữ liền chữ) vẫn phải bắt.
    findings = [
        f
        for f in auditor.audit("Hôm nay trời đẹp.tôi đi chơi.")
        if f.rule_id == "PUNCTUATION_NO_SPACE_AFTER"
    ]
    assert len(findings) == 1, findings


def test_ca_10_tone_inconsistency(auditor):
    text = (
        "Chính phủ chủ trương xây dựng nền kinh tế hòa nhập quốc tế, đồng thời giữ vững môi trường hoà bình và ổn"
        " định lâu dài."
    )
    findings = get_rule_ids(auditor, text)

    assert "ORTHOGRAPHY_TONE_INCONSISTENCY" in findings, "Bỏ sót lỗi bất nhất quán dấu thanh (hòa vs hoà)"


def test_no_false_positive_on_english_tech_loanwords(auditor):
    text = (
        "Anh check lại file log, fix bug rồi update lên server giùm em trước deadline nhé. Team marketing cần data gấp."
    )
    findings = auditor.audit(text)
    telex_errors = [f for f in findings if f.rule_id == "TELEX_TYPO_ERROR"]
    assert not telex_errors, f"LỖI BẮT OAN TỪ TIẾNG ANH: {[f.rationale for f in telex_errors]}"


def test_no_false_positive_on_ambiguous_english_words(auditor):
    # Các từ tiếng Anh ngắn nếu giải mã Telex sẽ trùng âm tiết Việt
    # (was→ứa, if→ì, keeps→kếp...). Tuyệt đối không được bắt lỗi.
    text = "The server was down; if you see this, keeps it simple and check the logs."
    findings = [f for f in auditor.audit(text) if f.rule_id == "TELEX_TYPO_ERROR"]
    assert not findings, f"LỖI BẮT OAN TỪ TIẾNG ANH NGẮN: {[f.rationale for f in findings]}"


def test_no_false_positive_on_english_trailing_w(auditor):
    # 'w' cuối từ là phím biến đổi Telex nhưng các từ tiếng Anh này phải được bỏ qua.
    text = "They saw a few new shows and know how to view a low bow now."
    findings = [f for f in auditor.audit(text) if f.rule_id == "TELEX_TYPO_ERROR"]
    assert not findings, f"LỖI BẮT OAN TỪ TIẾNG ANH CÓ W: {[f.rationale for f in findings]}"


def test_no_false_positive_on_alphanumeric_codes(auditor):
    # VNI dùng chữ số 1..5 làm phím thanh; mã/viết tắt alphanumeric không phải lỗi gõ.
    text = (
        "Xe hạng B1 và B12, vitamin B12, COVID-19, MP3, A4; "
        "công thức h2o, n2o, k2o, co2, no2, so2, so3, b2b, b2c."
    )
    findings = [f for f in auditor.audit(text) if f.rule_id == "TELEX_TYPO_ERROR"]
    assert not findings, f"LỖI BẮT OAN MÃ ALPHANUMERIC: {[f.rationale for f in findings]}"


def test_fuzz_extreme_unicode_and_punctuation_no_panic(auditor):
    fuzz_cases = [
        "!!! ??? ... ,,, ::: ;;; ((( ))) \"\"\" ''' --- === +++",
        "🔥🚀 Giá cổ phiếu tăng 15.5% lúc 10:30 (theo tỷ lệ 2:1) @user #tag $USD",
        "",
        "   \n\t\r  \n   ",
        "\u200B\u200C\u200DTiếng Việt\uFEFF",
        "." * 1000,
        "a" * 1000,
    ]
    for case in fuzz_cases:
        _ = auditor.audit(case)
