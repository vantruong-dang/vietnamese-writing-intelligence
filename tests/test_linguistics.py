"""Kiểm thử đơn vị cho toàn bộ tầng nền tảng ngôn ngữ học (Stage 2: Linguistics Foundation)."""

from __future__ import annotations

from vn_writing_intel.linguistics import (
    LinguisticSensorEngine,
    PhraseAnalyzer,
    RuleBasedPOSTagger,
    RuleBasedSegmenter,
    normalize_nfc,
    normalize_punctuation,
    normalize_whitespace,
)


def test_normalization_nfc_and_whitespace():
    # Kiểm tra chuẩn hóa NFD sang NFC
    nfd_text = "Tiê\u0301ng Vi\u00ea\u0323t"
    nfc_text = normalize_nfc(nfd_text)
    assert nfc_text == "Tiếng Việt"

    # Kiểm tra bảo toàn ranh giới đoạn văn
    multiline = "Đoạn 1 có nhiều    khoảng trắng.\r\n\r\n\r\n\r\nĐoạn 2 sau nhiều dòng trống."
    norm_para = normalize_whitespace(multiline, preserve_paragraphs=True)
    assert norm_para == "Đoạn 1 có nhiều khoảng trắng.\n\nĐoạn 2 sau nhiều dòng trống."

    # Kiểm tra gộp hoàn toàn khoảng trắng khi preserve_paragraphs=False
    single_line = normalize_whitespace(multiline, preserve_paragraphs=False)
    assert "\n" not in single_line
    assert "Đoạn 1 có nhiều khoảng trắng. Đoạn 2 sau nhiều dòng trống." == single_line


def test_normalization_punctuation():
    text = "Văn bản chứa “ngoặc kép cong” và ‘ngoặc đơn cong’ cùng dấu ba chấm…"
    norm = normalize_punctuation(text)
    assert "“" not in norm and "”" not in norm
    assert "‘" not in norm and "’" not in norm
    assert '"ngoặc kép cong"' in norm
    assert "'ngoặc đơn cong'" in norm
    assert "..." in norm


def test_rule_based_segmenter():
    segmenter = RuleBasedSegmenter()
    text = "Trí tuệ nhân tạo và chuyển đổi số đang tối ưu hóa hệ thống phân tán."
    tokens = segmenter.tokenize(text)

    token_texts = [t.text for t in tokens]
    # Phải bắt được các từ ghép chính xác
    assert "Trí tuệ nhân tạo" in token_texts
    assert "chuyển đổi số" in token_texts
    assert "tối ưu hóa" in token_texts
    assert "hệ thống phân tán" in token_texts

    # Kiểm tra tọa độ character spans
    for tok in tokens:
        assert text[tok.start : tok.end] == tok.text


def test_rule_based_pos_tagger():
    segmenter = RuleBasedSegmenter()
    tagger = RuleBasedPOSTagger()

    text = "Anh trai tôi tuy nghèo nhưng sửa máy rất giỏi."
    tokens = segmenter.tokenize(text)
    tagged = tagger.tag(tokens)

    tags = {t.text: t.pos_tag for t in tagged}
    assert tags.get("Anh trai") == "Kin"
    assert tags.get("tôi") == "P"
    assert tags.get("tuy") == "C"
    assert tags.get("nhưng") == "C"
    assert tags.get("sửa") == "V"
    assert tags.get("rất") == "R"


def test_phrase_analyzer():
    segmenter = RuleBasedSegmenter()
    tagger = RuleBasedPOSTagger()
    analyzer = PhraseAnalyzer()

    # 1. Phân tích cụm mở đầu bằng giới từ
    text_prep = "Trong tình hình kinh tế hiện nay đòi hỏi đổi mới."
    tokens_prep = tagger.tag(segmenter.tokenize(text_prep))
    phrases_prep = analyzer.analyze_phrases(text_prep, tokens_prep)
    types_prep = [p["type"] for p in phrases_prep]
    assert "prepositional_opening" in types_prep

    # 2. Phân tích chuỗi danh từ hóa
    text_nom = "Việc thực hiện quá trình tối ưu hóa giúp giảm độ trễ."
    tokens_nom = tagger.tag(segmenter.tokenize(text_nom))
    phrases_nom = analyzer.analyze_phrases(text_nom, tokens_nom)
    types_nom = [p["type"] for p in phrases_nom]
    assert "nominalization_chain" in types_nom

    # 3. Phân tích đối ngẫu cổ điển
    text_anti = "Người có tài không phải là khó, tự đem dùng được cái tài của mình mới thật là khó."
    tokens_anti = tagger.tag(segmenter.tokenize(text_anti))
    phrases_anti = analyzer.analyze_phrases(text_anti, tokens_anti)
    types_anti = [p["type"] for p in phrases_anti]
    assert "classical_antithesis" in types_anti

    # 4. Phân tích bị động chính danh
    text_pass = "Nó bị ngã gãy tay khi chạy bộ."
    tokens_pass = tagger.tag(segmenter.tokenize(text_pass))
    phrases_pass = analyzer.analyze_phrases(text_pass, tokens_pass)
    pass_matches = [p for p in phrases_pass if p["type"] == "passive_structure"]
    assert len(pass_matches) > 0
    assert pass_matches[0]["is_adversative_genuine"] is True


def test_linguistic_sensor_engine():
    engine = LinguisticSensorEngine()
    segmenter = RuleBasedSegmenter()

    text = "Ngày mai, tuy trời mưa to, tôi cũng sẽ đến đúng giờ."
    tokens = segmenter.tokenize(text)
    findings = engine.analyze(text, tokens)

    pids = [f.pattern_id for f in findings]
    assert "FACTUAL_CONCESSIVE_MISUSE" in pids
    assert findings[0].source == "linguistic"
    assert findings[0].confidence == 1.0
    assert findings[0].span(text) == "Ngày mai, tuy"
