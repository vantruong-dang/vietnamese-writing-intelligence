"""Vietnamese Writing Intelligence framework."""

from .drafting import (
    AnthropicProvider,
    DraftPlanner,
    MockLLMProvider,
    NativeDrafter,
    OllamaProvider,
    OpenAIProvider,
)
from .interfaces import (
    BaseContextRule,
    BaseLLMProvider,
    BasePhraseAnalyzer,
    BasePOSTagger,
    BaseSegmenter,
    Finding,
    GenerationResult,
    TokenSpan,
)
from .linguistics import RuleBasedSegmenter, UndertheseaSegmenter, get_default_segmenter
from .migrations import is_v1_manifest, migrate_manifest_v1_to_v2
from .orchestrator import V3Orchestrator
from .pipeline import WritingIntelligencePipeline
from .upstream_adapter import UpstreamAdapter
from .validators import SchemaRegistry, SchemaValidationError, validate_inputs

__version__ = "0.2.2"
__all__ = [
    "AnthropicProvider",
    "BaseContextRule",
    "BaseLLMProvider",
    "BasePOSTagger",
    "BasePhraseAnalyzer",
    "BaseSegmenter",
    "DraftPlanner",
    "Finding",
    "GenerationResult",
    "MockLLMProvider",
    "NativeDrafter",
    "OllamaProvider",
    "OpenAIProvider",
    "RuleBasedSegmenter",
    "SchemaRegistry",
    "SchemaValidationError",
    "TokenSpan",
    "UndertheseaSegmenter",
    "UpstreamAdapter",
    "V3Orchestrator",
    "WritingIntelligencePipeline",
    "get_default_segmenter",
    "is_v1_manifest",
    "migrate_manifest_v1_to_v2",
    "validate_inputs",
]
