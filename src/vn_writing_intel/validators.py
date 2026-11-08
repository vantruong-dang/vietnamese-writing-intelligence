"""Kiểm định JSON Schema cho Voice Profile và Draft Manifest."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator

from .migrations.manifest_v1_to_v2 import is_v1_manifest, migrate_manifest_v1_to_v2
from .resources import get_schema_dir


class SchemaValidationError(ValueError):
    """Ném ra khi dữ liệu cấu hình vi phạm JSON Schema."""

    pass


class SchemaRegistry:
    def __init__(self, schema_dir: Path | None = None) -> None:
        if schema_dir is None:
            self.schema_dir = get_schema_dir()
        else:
            self.schema_dir = Path(schema_dir)

        self._validators: dict[str, Draft202012Validator] = {}

    def _get_validator(self, schema_filename: str) -> Draft202012Validator:
        if schema_filename not in self._validators:
            schema_path = self.schema_dir / schema_filename
            if not schema_path.exists():
                raise FileNotFoundError(f"Không tìm thấy file schema tại: {schema_path}")

            with open(schema_path, encoding="utf-8") as f:
                schema_data = json.load(f)

            Draft202012Validator.check_schema(schema_data)
            self._validators[schema_filename] = Draft202012Validator(schema_data)

        return self._validators[schema_filename]

    def validate_voice_profile(self, profile: dict[str, Any]) -> None:
        validator = self._get_validator("voice-profile.schema.json")
        errors = sorted(validator.iter_errors(profile), key=lambda e: list(e.path))
        if errors:
            err = errors[0]
            loc = " -> ".join(str(p) for p in err.path) or "root"
            raise SchemaValidationError(f"[VoiceProfile Invalid] Lỗi tại '{loc}': {err.message}")

    def validate_draft_manifest(
        self, manifest: dict[str, Any], auto_migrate: bool = True
    ) -> dict[str, Any]:
        """Kiểm định DraftManifest.

        Hỗ trợ cả v0.1.0 và v0.2.0 thông qua cơ chế tự động migration an toàn.
        Trả về manifest sau khi đã chuẩn hóa và kiểm định.
        """
        target = manifest
        if auto_migrate and is_v1_manifest(manifest):
            target, _ = migrate_manifest_v1_to_v2(manifest)

        validator = self._get_validator("draft-manifest.schema.json")
        errors = sorted(validator.iter_errors(target), key=lambda e: list(e.path))
        if errors:
            err = errors[0]
            loc = " -> ".join(str(p) for p in err.path) or "root"
            raise SchemaValidationError(f"[DraftManifest Invalid] Lỗi tại '{loc}': {err.message}")
        return target


default_registry = SchemaRegistry()


def validate_inputs(profile: dict[str, Any], manifest: dict[str, Any]) -> dict[str, Any]:
    default_registry.validate_voice_profile(profile)
    return default_registry.validate_draft_manifest(manifest)
