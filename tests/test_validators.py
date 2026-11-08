import pytest
from vn_writing_intel.validators import validate_inputs, SchemaValidationError


def get_valid_payloads():
    profile = {
        "profile_id": "essayist_vn",
        "version": "1.0.0",
        "syntactic_signature": {
            "clause_chaining_style": "topic_comment_native",
            "allow_long_sentences": False
        },
        "lexical_preferences": {
            "sino_vietnamese_density": "balanced",
            "colloquial_particles_allowed": True,
            "english_term_retention": "preserve_industry_standard"
        }
    }
    manifest = {
        "discourse_type": "interdisciplinary_essay",
        "immutable_anchors": {
            "metrics": ["2026"],
            "domain_terms": ["kinh tế số"]
        }
    }
    return profile, manifest


def test_schema_valid_passes():
    p, m = get_valid_payloads()
    validate_inputs(p, m)


def test_schema_invalid_profile_fails():
    p, m = get_valid_payloads()
    p["profile_id"] = "Invalid Profile ID!"
    with pytest.raises(SchemaValidationError):
        validate_inputs(p, m)
