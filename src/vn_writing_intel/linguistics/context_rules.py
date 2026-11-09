"""Các luật ngữ cảnh tiếng Việt theo ngữ pháp chức năng (Cao Xuân Hạo, Nguyễn Hiến Lê)."""

from __future__ import annotations

import re
from typing import Any

from ..interfaces import BaseContextRule, Finding, TokenSpan
from .findings import create_finding
from .normalization import normalize_nfc

try:
    from .. import _core

    _HAS_RUST_CORE = True
except (ImportError, ModuleNotFoundError):
    _HAS_RUST_CORE = False


class CrippledPrepositionalTopicRule(BaseContextRule):
    """Bắt lỗi giới ngữ chỉ phạm vi/phương tiện bị ép làm chủ thể cho vị từ."""

    rule_id = "CRIPPLED_PREPOSITIONAL_TOPIC"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"^(trong|qua|bằng|với)\s+[^,]+?\s+(đòi hỏi|yêu cầu|cho thấy|chứng minh|đã chứng minh|giúp|đã giúp|khiến|buộc|đã buộc|đã phát hiện|phát hiện|chứng tỏ)\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE | re.MULTILINE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="topic_comment",
                    severity="major",
                    start=m.start(),
                    end=m.end(),
                    rationale="Lẫn lộn Khung đề và Chủ đề: Giới từ biến cụm từ thành trạng ngữ/phương tiện, không thể làm chủ thể cho vị từ.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class PrepositionConjunctionConflictRule(BaseContextRule):
    """Bắt lỗi xung đột cú pháp giữa giới từ mở đầu và liên từ phản đề 'nhưng'."""

    rule_id = "PREPOSITION_CONJUNCTION_CONFLICT"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"^(với những|với|qua|bằng)\s+[^,]+?\s+nhưng\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE | re.MULTILINE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="topic_comment",
                    severity="major",
                    start=m.start(),
                    end=m.end(),
                    rationale="Xung đột cú pháp: Giới từ mở đầu trạng ngữ không thể kết hợp trực tiếp với liên từ phản đề 'nhưng'.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class FactualConcessiveMisuseRule(BaseContextRule):
    """Bắt lỗi dùng liên từ nhượng bộ hiện thực (tuy/mặc dù) cho sự việc tương lai giả định."""

    rule_id = "FACTUAL_CONCESSIVE_MISUSE"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = (
            r"(?:\b(ngày mai|tuần tới|năm sau|sắp tới|sau này)[,\s]+(tuy|mặc dù|mặc dầu)\b|"
            r"\b(tuy|mặc dù|mặc dầu)\b[^,.]+?\b(đi chăng nữa|chăng nữa|đến mấy|thế nào chăng nữa)\b)"
        )
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="concessive_aspect",
                    severity="major",
                    start=m.start(),
                    end=m.end(),
                    rationale="Sự việc tương lai giả định (phi hiện thực). 'Tuy' chỉ dùng cho sự việc đã/đang diễn ra; giả định bắt buộc dùng 'Dù'.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class HypotheticalConcessiveMisuseRule(BaseContextRule):
    """Bắt lỗi dùng liên từ nhượng bộ giả định (dù) cho sự việc quá khứ xác định (hiện thực)."""

    rule_id = "HYPOTHETICAL_CONCESSIVE_MISUSE"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"\b(hôm qua|năm ngoái|tuần trước|vừa qua|thực tế là|rõ ràng là)[,\s]+dù\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="concessive_aspect",
                    severity="major",
                    start=m.start(),
                    end=m.end(),
                    rationale="Sự việc quá khứ là hiện thực xác định. Bắt buộc dùng 'Tuy/Mặc dù', không dùng liên từ giả định 'Dù'.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class TemporalFramingRule(BaseContextRule):
    """Bắt lỗi dịch ngữ đưa khung thời gian tương lai ra cuối câu ('sẽ... trong X ngày')."""

    rule_id = "TEMPORAL_IN_TRANSLATIONESE"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"\bsẽ\s+[^,.]+?\s+(trong\s+(?:một|hai|ba|bốn|năm|sáu|bảy|tám|chín|mười|\d+)\s+(?:ngày|giờ|phút|tuần|tháng|năm)|sau\s+(?:một|hai|ba|bốn|năm|sáu|bảy|tám|chín|mười|\d+)\s+(?:ngày|giờ|phút|tuần|tháng|năm)\s+nữa)\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="temporal_framing",
                    severity="minor",
                    start=m.start(),
                    end=m.end(),
                    rationale="Sao phỏng 'in X days': Khoảng cách thời gian tương lai trong tiếng Việt nên đặt làm Khung đề ở đầu câu thay vì cuối câu.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class ArtificialPassiveRule(BaseContextRule):
    """Bắt lỗi câu bị động nhân tạo sao phỏng 'be + V + by' của tiếng Anh."""

    rule_id = "ARTIFICIAL_PASSIVE_BE_BY"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"\b(được|bị)\s+[^,.]+?\s+bởi(?!\s+(?:vì|thế|do|tại|vậy))\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="pseudo_passive",
                    severity="major",
                    start=m.start(),
                    end=m.end(),
                    rationale="Câu bị động nhân tạo sao phỏng công thức 'be + V3 + by' của tiếng Anh.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class RedundantBeneficiaryPassiveRule(BaseContextRule):
    """Bắt lỗi dùng 'được' thụ hưởng dư thừa cho danh ngữ chỉ vật vô tri."""

    rule_id = "REDUNDANT_BENEFICIARY_PASSIVE"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"^(căn phòng|báo cáo|hệ thống|dữ liệu|bàn ghế|nhà cửa|cái bàn|tóc nó)\s+(?:đã|sẽ|đang)?\s*(?:được|bị)\s+(lau dọn|xem xét|khởi động|xử lý|lau|cắt)\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="pseudo_passive",
                    severity="minor",
                    start=m.start(),
                    end=m.end(),
                    rationale="Danh ngữ chỉ vật vô tri đứng làm Đề không cần vị từ chỉ sự thụ hưởng 'được'.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class LogicalPleonasmRule(BaseContextRule):
    """Bắt lỗi trùng ngữ Hán - Việt hoặc lặp nghĩa logic ('tối ưu nhất', 'ánh nắng mặt trời')."""

    rule_id = "LOGICAL_PLEONASM"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"\b(tối ưu nhất|ánh nắng mặt trời|hoàn toàn tuyệt đối|nguyên nhân\s+[^,.]+?\s+là bởi vì)\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="pleonasm",
                    severity="minor",
                    start=m.start(),
                    end=m.end(),
                    rationale="Trùng ngữ ngữ nghĩa (pleonasm): Lặp thừa ý tố biểu đạt (ví dụ: 'tối' đã là nhất, 'nắng' đã là của mặt trời).",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class AspectMechanicsRule(BaseContextRule):
    """Bắt lỗi cụm 'đã và đang' dịch thô từ thì hoàn thành tiếp diễn tiếng Anh."""

    rule_id = "CXH_VERBAL_SERIES_TENSE_FALSE_MARKER"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"\b(đã và đang)\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="aspect_mechanics",
                    severity="minor",
                    start=m.start(),
                    end=m.end(),
                    rationale="Cụm 'đã và đang' dịch thô từ thì hoàn thành tiếp diễn tiếng Anh, tạo cảm giác lai căng và làm nặng câu văn.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class SyntaxReportingRule(BaseContextRule):
    """Bắt lỗi sau vị từ trần thuật dùng một danh ngữ đơn độc thay vì mệnh đề trọn vẹn."""

    rule_id = "CXH_REPORTING_VERB_INCOMPLETE_CLAUSE"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"^(phải nói là|cho là|cho rằng)\s+(?:một\s+)?(?:cú|sự|cuộc|việc|quyết định)\s+[^,.]+[.?!]?$"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="syntax_reporting",
                    severity="major",
                    start=m.start(),
                    end=m.end(),
                    rationale="Sau 'phải nói là' bắt buộc phải là một mệnh đề trọn vẹn, không dùng một danh ngữ đơn độc.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class SyntaxParallelRule(BaseContextRule):
    """Bắt lỗi liên từ 'nhưng' nối hai vế bất đối xứng (danh ngữ đối lập vị từ cụt)."""

    rule_id = "CXH_PARALLEL_CONJUNCTION_ASYMMETRY"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"^(một\s+[^,]+?),?\s+nhưng\s+(không|chưa|thiếu)\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="syntax_parallel",
                    severity="major",
                    start=m.start(),
                    end=m.end(),
                    rationale="Liên từ 'nhưng' nối hai vế bất đối xứng: Danh ngữ đứng trước đối lập với cụm vị từ cụt phía sau.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class SyntaxClicheRule(BaseContextRule):
    """Bắt lỗi ghép nhầm hai cấu trúc diễn ngôn 'Như... đã biết' và '... đã biết rằng'."""

    rule_id = "CXH_HYBRID_COMPLEX_CONJUNCTION"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"\bnhư\s+[^,]+?\s+đã\s+biết\s+(?:là|rằng)\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="syntax_cliche",
                    severity="minor",
                    start=m.start(),
                    end=m.end(),
                    rationale="Ghép nhầm hai kiểu câu: 'Như... đã biết' và '... đã biết rằng'.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class SuperlativeMisuseRule(BaseContextRule):
    """Bắt lỗi dùng 'chí ít' thay vì từ thuần Việt 'ít nhất' hoặc 'tối thiểu'."""

    rule_id = "CXH_SUPERLATIVE_CHI_IT_MISUSE"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"\bchí ít\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="lexical_misuse",
                    severity="minor",
                    start=m.start(),
                    end=m.end(),
                    rationale="'Chí' chỉ dùng cho cực cấp tuyệt đối (chí lý, chí công); 'chí ít' là lai căng, nên dùng 'ít nhất' hoặc 'tối thiểu'.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class AIOpeningClicheRule(BaseContextRule):
    """Bắt mở đầu sáo rỗng rập khuôn của AI."""

    rule_id = "AI_OPENING_CLICHE"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"^(không thể phủ nhận rằng|có thể nói rằng|nhìn chung(?!\s+lại)|trong bối cảnh .* hiện nay)[\s,]*"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE | re.MULTILINE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="humanizer",
                    severity="critical",
                    start=m.start(),
                    end=m.end(),
                    rationale="Cụm rào đón mở đầu rập khuôn kiểu RLHF, làm giảm độ tin cậy của luận điểm.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class AISummaryTailRule(BaseContextRule):
    """Bắt từ nối tổng kết máy móc."""

    rule_id = "AI_SUMMARY_TAIL"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"\b(tóm lại|nói tóm lại|nhìn chung lại|xét cho cùng)[,\s]*"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="humanizer",
                    severity="major",
                    start=m.start(),
                    end=m.end(),
                    rationale="Từ nối tổng kết máy móc làm giảm sức nặng của nhận định.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class FormulaicMetaphorRule(BaseContextRule):
    """Bắt các ẩn dụ công thức sáo rỗng."""

    rule_id = "FORMULAIC_METAPHOR"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = (
            r"\b(bức tranh toàn cảnh|chiếc chìa khóa vạn năng|con dao hai lưỡi|ngọn hải đăng)\b"
        )
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="humanizer",
                    severity="critical",
                    start=m.start(),
                    end=m.end(),
                    rationale="Sử dụng ẩn dụ sáo rỗng đặc trưng của mô hình ngôn ngữ lớn.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class NominalizationHeavyRule(BaseContextRule):
    """Bắt hiện tượng danh từ hóa nặng nề dịch từ tiếng Anh."""

    rule_id = "NOMINALIZATION_HEAVY"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"\b(việc thực hiện|sự gia tăng của|quá trình tối ưu hóa)\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="translationese",
                    severity="minor",
                    start=m.start(),
                    end=m.end(),
                    rationale="Danh từ hóa chồng chéo dịch thô từ danh động từ tiếng Anh.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class SuytNuaThiRule(BaseContextRule):
    """Bắt lỗi lai ghép 'suýt nữa thì' từ 'suýt nữa' và 'chút nữa thì'."""

    rule_id = "CXH_SUYT_NUA_THI_CONTAMINATION"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"\bsuýt nữa thì\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="lexical_misuse",
                    severity="minor",
                    start=m.start(),
                    end=m.end(),
                    rationale="Lai ghép 'suýt nữa' với 'chút nữa thì' (Cao Xuân Hạo, tr. 163).",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class TranslationeseSoleilRule(BaseContextRule):
    """Bắt lỗi dịch thô 'mặt trời' thay cho 'ánh nắng' trong không gian kín."""

    rule_id = "TRANSLATIONESE_SOLEIL_SUNSHINE"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"^không có mặt trời\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="translationese",
                    severity="minor",
                    start=m.start(),
                    end=m.end(),
                    rationale="Dịch thô 'soleil/sun' là 'mặt trời' thay vì 'ánh nắng' (Cao Xuân Hạo, tr. 164).",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class BureaucraticVerbPaddingRule(BaseContextRule):
    """Bắt lỗi đệm động từ rỗng hành chính ('tiến hành thực hiện', 'triển khai công tác')."""

    rule_id = "BUREAUCRATIC_VERB_PADDING"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"\b(tiến hành thực hiện|triển khai công tác|đưa vào áp dụng)\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="bureaucracy",
                    severity="minor",
                    start=m.start(),
                    end=m.end(),
                    rationale="Lạm dụng đệm động từ rỗng hành chính. Hãy dùng trực tiếp động từ chính.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class RedundantConnectorsRule(BaseContextRule):
    """Bắt lỗi chêm liên từ thừa ('và đồng thời', 'cũng như là')."""

    rule_id = "REDUNDANT_CONNECTORS"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"\b(và đồng thời|cũng như là)\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="bureaucracy",
                    severity="minor",
                    start=m.start(),
                    end=m.end(),
                    rationale="Chêm liên từ trùng lặp / thừa hư từ.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class VagueIntensifierRule(BaseContextRule):
    """Bắt lỗi trợ từ tăng cường sáo rỗng ('vô cùng hết sức', 'thực sự rất là')."""

    rule_id = "VAGUE_INTENSIFIER"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"\b(vô cùng hết sức|thực sự rất là)\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="bureaucracy",
                    severity="minor",
                    start=m.start(),
                    end=m.end(),
                    rationale="Xếp chồng trợ từ cực cấp sáo rỗng hoặc khẩu ngữ rườm rà.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class DummyItTranslationeseRule(BaseContextRule):
    """Bắt cấu trúc chủ ngữ giả sao phỏng 'It is...' của tiếng Anh."""

    rule_id = "DUMMY_IT_TRANSLATIONESE"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"\b(nó\s+là\s+(?:rất\s+)?(?:quan\s+trọng|cần\s+thiết|rõ\s+ràng|hiển\s+nhiên|không\s+thể\s+phủ\s+nhận))\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="translationese",
                    severity="major",
                    start=m.start(),
                    end=m.end(),
                    rationale="Cấu trúc chủ ngữ giả sao phỏng Anh ngữ ('Nó là...'). Tiếng Việt dùng Đề-Thuyết nên lược bỏ chủ ngữ giả.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class AdjectivalSuffixFakeRule(BaseContextRule):
    """Bắt tính từ hóa nhân tạo ('mang tính chất', 'có tính cách')."""

    rule_id = "ADJECTIVAL_SUFFIX_FAKE"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"\b(mang\s+tính\s+chất|có\s+tính\s+cách)\s+([a-zà-ỹ]+)\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="translationese",
                    severity="minor",
                    start=m.start(),
                    end=m.end(),
                    rationale="Tính từ hóa nhân tạo ('mang tính chất / có tính cách'). Trong tiếng Việt, hãy dùng trực tiếp tính từ.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class ConcessiveInversionConflictRule(BaseContextRule):
    """Bắt xung đột nhượng bộ kép ('Mặc dù... tuy nhiên...')."""

    rule_id = "CONCESSIVE_INVERSION_CONFLICT"

    def evaluate(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any]
    ) -> list[Finding]:
        norm = normalize_nfc(text)
        pattern = r"\b(mặc\s+dù|dẫu\s+rằng|cho\s+dù)\b([^,.!?]+?)\b(tuy\s+nhiên|nhưng\s+mà)\b"
        findings: list[Finding] = []
        for m in re.finditer(pattern, norm, flags=re.IGNORECASE):
            findings.append(
                create_finding(
                    pattern_id=self.rule_id,
                    category="concessive",
                    severity="major",
                    start=m.start(),
                    end=m.end(),
                    rationale="Xung đột nhượng bộ kép ('Mặc dù... tuy nhiên'). Đã có 'mặc dù' thì không dùng 'tuy nhiên' ở vế đối lập.",
                    metadata={"matched_span": norm[m.start() : m.end()]},
                )
            )
        return findings


class LinguisticSensorEngine:
    """Động cơ điều phối toàn bộ các cảm biến và luật ngữ cảnh tiếng Việt.

    Khi sử dụng bộ luật mặc định và có sẵn extension lõi Rust (_core), động cơ sẽ tự động
    ủy quyền cho `_core.audit_linguistics` để đạt hiệu năng xử lý cực đại; đồng thời giữ
    khả năng tương thích 100% bằng cách fallback về bộ duyệt luật Python khi cần.
    """

    def __init__(
        self,
        rules: list[BaseContextRule] | None = None,
        auditor: Any | None = None,
    ) -> None:
        self._is_default_rules = rules is None
        self._rust_auditor = auditor
        if self._rust_auditor is None and self._is_default_rules and _HAS_RUST_CORE:
            try:
                self._rust_auditor = _core.Auditor()
            except Exception:
                self._rust_auditor = None

        self.rules: list[BaseContextRule] = (
            rules
            if rules is not None
            else [
                CrippledPrepositionalTopicRule(),
                PrepositionConjunctionConflictRule(),
                FactualConcessiveMisuseRule(),
                HypotheticalConcessiveMisuseRule(),
                TemporalFramingRule(),
                ArtificialPassiveRule(),
                RedundantBeneficiaryPassiveRule(),
                LogicalPleonasmRule(),
                AspectMechanicsRule(),
                SyntaxReportingRule(),
                SyntaxParallelRule(),
                SyntaxClicheRule(),
                SuperlativeMisuseRule(),
                AIOpeningClicheRule(),
                AISummaryTailRule(),
                FormulaicMetaphorRule(),
                NominalizationHeavyRule(),
                SuytNuaThiRule(),
                TranslationeseSoleilRule(),
                BureaucraticVerbPaddingRule(),
                RedundantConnectorsRule(),
                VagueIntensifierRule(),
                DummyItTranslationeseRule(),
                AdjectivalSuffixFakeRule(),
                ConcessiveInversionConflictRule(),
            ]
        )

    def analyze(
        self, text: str, tokens: list[TokenSpan], context: dict[str, Any] | None = None
    ) -> list[Finding]:
        if self._is_default_rules and self._rust_auditor is not None:
            try:
                rust_findings = self._rust_auditor.audit(text)
                return [
                    Finding(
                        pattern_id=f.pattern_id,
                        category=f.category,
                        severity=f.severity,
                        start=f.start,
                        end=f.end,
                        confidence=f.confidence,
                        rationale=f.rationale,
                        source=f.source,
                        metadata={"matched_span": f.matched_span},
                    )
                    for f in rust_findings
                ]
            except Exception:
                pass

        ctx = context or {}
        all_findings: list[Finding] = []
        for rule in self.rules:
            findings = rule.evaluate(text, tokens, ctx)
            all_findings.extend(findings)
        return all_findings
