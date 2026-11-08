"""Các nhà cung cấp mô hình ngôn ngữ lớn (LLM Providers) phục vụ Native Drafter."""
from __future__ import annotations

from typing import Any, Callable, Dict, Optional
from ..interfaces import BaseLLMProvider, GenerationResult


class MockLLMProvider(BaseLLMProvider):
    """Mock LLM Provider phục vụ kiểm thử đơn vị độc lập và môi trường offline."""

    def __init__(
        self,
        default_response: Optional[str] = None,
        generator_fn: Optional[Callable[[str, str], str]] = None,
        model_name: str = "mock-drafter-v1",
    ) -> None:
        self.default_response = default_response or "Bản thảo phác thảo mẫu đảm bảo cấu trúc Đề - Thuyết bản địa."
        self.generator_fn = generator_fn
        self.model_name = model_name
        self.history: list[Dict[str, Any]] = []

    def generate(
        self,
        system_prompt: str,
        user_prompt: str,
        temperature: float = 0.5,
    ) -> GenerationResult:
        self.history.append({
            "system_prompt": system_prompt,
            "user_prompt": user_prompt,
            "temperature": temperature,
        })

        if self.generator_fn is not None:
            text = self.generator_fn(system_prompt, user_prompt)
        else:
            text = self.default_response

        return GenerationResult(
            text=text,
            provider="mock",
            model=self.model_name,
            usage={
                "prompt_tokens": len(system_prompt.split()) + len(user_prompt.split()),
                "completion_tokens": len(text.split()),
                "total_tokens": len(system_prompt.split()) + len(user_prompt.split()) + len(text.split()),
            },
            finish_reason="stop",
            raw_response={"mock_status": "success"},
        )
