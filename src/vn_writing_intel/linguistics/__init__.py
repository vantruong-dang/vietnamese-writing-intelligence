"""Gói phân tích ngôn ngữ học tiếng Việt bản địa (GS. Cao Xuân Hạo, Nguyễn Hiến Lê)."""

from .context_rules import (
    AIOpeningClicheRule,
    AISummaryTailRule,
    ArtificialPassiveRule,
    AspectMechanicsRule,
    CrippledPrepositionalTopicRule,
    FactualConcessiveMisuseRule,
    FormulaicMetaphorRule,
    HypotheticalConcessiveMisuseRule,
    LinguisticSensorEngine,
    LogicalPleonasmRule,
    NominalizationHeavyRule,
    PrepositionConjunctionConflictRule,
    RedundantBeneficiaryPassiveRule,
    SuperlativeMisuseRule,
    SyntaxClicheRule,
    SyntaxParallelRule,
    SyntaxReportingRule,
    TemporalFramingRule,
)
from .findings import create_finding
from .normalization import normalize_nfc, normalize_punctuation, normalize_whitespace
from .phrase_analysis import PhraseAnalyzer
from .pos import RuleBasedPOSTagger
from .segmentation import RuleBasedSegmenter

__all__ = [
    "AIOpeningClicheRule",
    "AISummaryTailRule",
    "ArtificialPassiveRule",
    "AspectMechanicsRule",
    "CrippledPrepositionalTopicRule",
    "FactualConcessiveMisuseRule",
    "FormulaicMetaphorRule",
    "HypotheticalConcessiveMisuseRule",
    "LinguisticSensorEngine",
    "LogicalPleonasmRule",
    "NominalizationHeavyRule",
    "PhraseAnalyzer",
    "PrepositionConjunctionConflictRule",
    "RedundantBeneficiaryPassiveRule",
    "RuleBasedPOSTagger",
    "RuleBasedSegmenter",
    "SuperlativeMisuseRule",
    "SyntaxClicheRule",
    "SyntaxParallelRule",
    "SyntaxReportingRule",
    "TemporalFramingRule",
    "create_finding",
    "normalize_nfc",
    "normalize_punctuation",
    "normalize_whitespace",
]
