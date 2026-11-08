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
    "normalize_nfc",
    "normalize_whitespace",
    "normalize_punctuation",
    "RuleBasedSegmenter",
    "RuleBasedPOSTagger",
    "PhraseAnalyzer",
    "create_finding",
    "LinguisticSensorEngine",
    "CrippledPrepositionalTopicRule",
    "PrepositionConjunctionConflictRule",
    "FactualConcessiveMisuseRule",
    "HypotheticalConcessiveMisuseRule",
    "TemporalFramingRule",
    "ArtificialPassiveRule",
    "RedundantBeneficiaryPassiveRule",
    "LogicalPleonasmRule",
    "AspectMechanicsRule",
    "SyntaxReportingRule",
    "SyntaxParallelRule",
    "SyntaxClicheRule",
    "SuperlativeMisuseRule",
    "AIOpeningClicheRule",
    "AISummaryTailRule",
    "FormulaicMetaphorRule",
    "NominalizationHeavyRule",
]
