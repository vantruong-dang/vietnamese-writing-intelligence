"""Kiểm định tính năng Migration và Schema Canonical v0.2.0."""
import pytest
from vn_writing_intel.migrations import is_v1_manifest, migrate_manifest_v1_to_v2
from vn_writing_intel.validators import default_registry, SchemaValidationError


def test_is_v1_manifest():
    v1_manifest = {
        "discourse_type": "technical_architecture",
        "immutable_anchors": {
            "metrics": ["100W"],
            "domain_terms": ["FFI"],
        },
    }
    v2_manifest = {
        "discourse_spec": {
            "genre": "technical_architecture",
            "register": "professional_sharp",
            "target_audience": "domain_engineer",
        },
        "immutable_anchors": {
            "metrics": ["100W"],
            "domain_terms": ["FFI"],
        },
    }

    assert is_v1_manifest(v1_manifest) is True
    assert is_v1_manifest(v2_manifest) is False


def test_migration_known_genre_preserves_rules():
    v1_manifest = {
        "discourse_type": "technical_architecture",
        "immutable_anchors": {
            "metrics": ["20V 7.5A"],
            "domain_terms": ["type erasure"],
        },
    }

    v2_manifest, notices = migrate_manifest_v1_to_v2(v1_manifest)

    # 1. Ánh xạ chính xác genre
    assert v2_manifest["discourse_spec"]["genre"] == "technical_architecture"
    # 2. Không tự suy diễn register và target_audience -> unspecified
    assert v2_manifest["discourse_spec"]["register"] == "unspecified"
    assert v2_manifest["discourse_spec"]["target_audience"] == "unspecified"
    assert v2_manifest["discourse_spec"]["dialect_bias"] == "all_neutral"
    # 3. Bảo toàn anchor 100%
    assert v2_manifest["immutable_anchors"]["metrics"] == ["20V 7.5A"]
    assert v2_manifest["immutable_anchors"]["domain_terms"] == ["type erasure"]
    assert v2_manifest["schema_version"] == "0.2.0"
    assert len(notices) > 0

    # 4. Manifest sau khi migrate phải validate thành công theo schema canonical v0.2.0
    default_registry.validate_draft_manifest(v2_manifest, auto_migrate=False)


def test_migration_unknown_genre_sets_unspecified_safely():
    v1_manifest = {
        "discourse_type": "custom_domain_specific_type",
        "immutable_anchors": {
            "metrics": ["2026"],
            "domain_terms": ["AI"],
        },
    }

    v2_manifest, notices = migrate_manifest_v1_to_v2(v1_manifest)

    # Không tự đoán bừa genre khi không chắc chắn
    assert v2_manifest["discourse_spec"]["genre"] == "unspecified"
    assert v2_manifest["v1_discourse_type"] == "custom_domain_specific_type"
    default_registry.validate_draft_manifest(v2_manifest, auto_migrate=False)


def test_canonical_v2_manifest_full_features():
    v2_canonical = {
        "manifest_id": "doc-arch-001",
        "schema_version": "0.2.0",
        "discourse_spec": {
            "genre": "technical_architecture",
            "register": "professional_sharp",
            "dialect_bias": "all_neutral",
            "target_audience": "c_level",
        },
        "topic_comment_plan": [
            {
                "topic": "Hạ tầng phân tán",
                "intended_comment": "Cần tối ưu băng thông và giảm độ trễ FFI",
                "key_anchor_refs": ["FFI", "100 Gbps"],
            }
        ],
        "immutable_anchors": {
            "metrics": ["100 Gbps"],
            "domain_terms": ["FFI"],
            "named_entities": ["Nguyễn Văn A"],
            "causal_invariants": [
                {
                    "cause": "Tăng kích thước bộ nhớ",
                    "effect": "Giảm tỷ lệ cache miss",
                }
            ],
        },
    }

    # Validate trực tiếp canonical v0.2.0
    validated = default_registry.validate_draft_manifest(v2_canonical, auto_migrate=False)
    assert validated["manifest_id"] == "doc-arch-001"
    assert len(validated["topic_comment_plan"]) == 1


def test_invalid_canonical_v2_fails():
    invalid_manifest = {
        "discourse_spec": {
            # Thiếu target_audience và register bắt buộc
            "genre": "technical_architecture"
        },
        "immutable_anchors": {
            "metrics": ["100W"],
            "domain_terms": ["FFI"],
        },
    }
    with pytest.raises(SchemaValidationError):
        default_registry.validate_draft_manifest(invalid_manifest, auto_migrate=False)
