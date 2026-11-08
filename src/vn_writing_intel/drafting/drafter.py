"""Điều phối viên Native Drafter kết nối kế hoạch dàn bài, sinh văn bản và kiểm toán tức thì."""

from __future__ import annotations

from typing import Any

from ..interfaces import BaseLLMProvider
from ..orchestrator import V3Orchestrator
from .planner import DraftPlanner


class NativeDrafter:
    """Tác tử sinh bản thảo bản địa (Native Drafter) v0.2.0.

    Quy trình hoạt động:
    1. DraftPlanner chuyển đổi DraftManifest thành System Prompt & User Prompt cấu trúc Đề - Thuyết.
    2. LLM Provider (BaseLLMProvider) thực thi sinh văn bản tự nhiên theo phong cách tiếng Việt bản địa.
    3. Bộ điều phối (V3Orchestrator) lập tức kiểm toán tính toàn vẹn ngữ nghĩa và các luật ngữ cảnh.
    """

    def __init__(
        self,
        provider: BaseLLMProvider,
        orchestrator: V3Orchestrator | None = None,
        planner: DraftPlanner | None = None,
    ) -> None:
        self.provider = provider
        self.orchestrator = orchestrator
        self.planner = planner or DraftPlanner()

    def draft(
        self,
        manifest: dict[str, Any],
        voice_profile: dict[str, Any],
        instructions: str = "",
        auto_verify: bool = True,
        temperature: float = 0.5,
    ) -> dict[str, Any]:
        """Thực hiện một chu trình sinh bản thảo hoàn chỉnh có kiểm soát."""
        # 1. Xây dựng prompt có cấu trúc
        sys_prompt = self.planner.build_system_prompt(manifest, voice_profile)
        user_prompt = self.planner.build_user_prompt(manifest, user_instructions=instructions)

        # 2. Sinh văn bản từ LLM Provider
        generation = self.provider.generate(
            system_prompt=sys_prompt,
            user_prompt=user_prompt,
            temperature=temperature,
        )

        draft_text = generation.text
        verification = None
        status = "unverified"

        # 3. Tự động kiểm toán ngữ nghĩa và phong cách nếu được yêu cầu
        if auto_verify:
            orch = self.orchestrator or V3Orchestrator(voice_profile, validate=False)
            verification = orch.process(draft_text, manifest, validate=False)
            status = verification["action_mode"]

        return {
            "draft_text": draft_text,
            "status": status,
            "generation": generation,
            "verification": verification,
            "prompts": {
                "system_prompt": sys_prompt,
                "user_prompt": user_prompt,
            },
        }
