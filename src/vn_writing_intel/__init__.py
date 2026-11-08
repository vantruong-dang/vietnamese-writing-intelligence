"""Vietnamese Writing Intelligence framework."""
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
from .upstream_adapter import UpstreamAdapter
from .validators import SchemaRegistry, SchemaValidationError, validate_inputs

__version__ = "0.2.0.dev1"
__all__ = [
    "V3Orchestrator",
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
    "is_v1_manifest",
    "migrate_manifest_v1_to_v2",
]
