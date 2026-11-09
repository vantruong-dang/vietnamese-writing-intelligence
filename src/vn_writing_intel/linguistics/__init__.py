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
from .segmentation import (
    RuleBasedSegmenter,
    RustViterbiSegmenter,
    UndertheseaSegmenter,
    get_default_segmenter,
)

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
    "RustViterbiSegmenter",
    "SuperlativeMisuseRule",
    "SyntaxClicheRule",
    "SyntaxParallelRule",
    "SyntaxReportingRule",
    "TemporalFramingRule",
    "UndertheseaSegmenter",
    "create_finding",
    "get_default_segmenter",
    "normalize_nfc",
    "normalize_punctuation",
    "normalize_whitespace",
]
