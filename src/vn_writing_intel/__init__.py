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
    BasePOSTagger,
    BasePhraseAnalyzer,
    BaseSegmenter,
    Finding,
    GenerationResult,
    TokenSpan,
)
from .migrations import is_v1_manifest, migrate_manifest_v1_to_v2
from .orchestrator import V3Orchestrator
from .pipeline import WritingIntelligencePipeline
from .upstream_adapter import UpstreamAdapter
from .validators import SchemaRegistry, SchemaValidationError, validate_inputs

__version__ = "0.2.0"
__all__ = [
    "V3Orchestrator",
    "WritingIntelligencePipeline",
    "UpstreamAdapter",
    "SchemaRegistry",
    "SchemaValidationError",
    "validate_inputs",
    "TokenSpan",
    "Finding",
    "GenerationResult",
    "BaseSegmenter",
    "BasePOSTagger",
    "BasePhraseAnalyzer",
    "BaseContextRule",
    "BaseLLMProvider",
    "DraftPlanner",
    "MockLLMProvider",
    "OllamaProvider",
    "AnthropicProvider",
    "OpenAIProvider",
    "NativeDrafter",
    "is_v1_manifest",
    "migrate_manifest_v1_to_v2",
]
