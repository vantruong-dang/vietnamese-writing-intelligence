"""Các nhà cung cấp mô hình ngôn ngữ lớn (LLM Providers) phục vụ Native Drafter."""

from __future__ import annotations

import json
import os
import urllib.error
import urllib.request
from collections.abc import Callable
from typing import Any

from ..interfaces import BaseLLMProvider, GenerationResult


class MockLLMProvider(BaseLLMProvider):
    """Mock LLM Provider phục vụ kiểm thử đơn vị độc lập và môi trường offline."""

    def __init__(
        self,
        default_response: str | None = None,
        generator_fn: Callable[[str, str], str] | None = None,
        model_name: str = "mock-drafter-v1",
    ) -> None:
        self.default_response = (
            default_response or "Bản thảo phác thảo mẫu đảm bảo cấu trúc Đề - Thuyết bản địa."
        )
        self.generator_fn = generator_fn
        self.model_name = model_name
        self.history: list[dict[str, Any]] = []

    def generate(
        self,
        system_prompt: str,
        user_prompt: str,
        temperature: float = 0.5,
    ) -> GenerationResult:
        self.history.append(
            {
                "system_prompt": system_prompt,
                "user_prompt": user_prompt,
                "temperature": temperature,
            }
        )

        if self.generator_fn is not None:
            text = self.generator_fn(system_prompt, user_prompt)
        else:
            text = self.default_response

        prompt_tokens = len(system_prompt.split()) + len(user_prompt.split())
        comp_tokens = len(text.split())
        return GenerationResult(
            text=text,
            provider="mock",
            model=self.model_name,
            usage={
                "prompt_tokens": prompt_tokens,
                "completion_tokens": comp_tokens,
                "total_tokens": prompt_tokens + comp_tokens,
            },
            finish_reason="stop",
            raw_response={"mock_status": "success"},
        )


class OllamaProvider(BaseLLMProvider):
    """Nhà cung cấp LLM qua Ollama API cục bộ (Local Model, không tốn chi phí)."""

    def __init__(
        self,
        model: str = "llama3:latest",
        base_url: str = "http://localhost:11434",
        timeout: float = 120.0,
    ) -> None:
        self.model = model
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout

    def generate(
        self,
        system_prompt: str,
        user_prompt: str,
        temperature: float = 0.5,
    ) -> GenerationResult:
        endpoint = f"{self.base_url}/api/chat"
        payload = {
            "model": self.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt},
            ],
            "stream": False,
            "options": {"temperature": temperature},
        }

        req = urllib.request.Request(
            endpoint,
            data=json.dumps(payload).encode("utf-8"),
            headers={"Content-Type": "application/json"},
            method="POST",
        )

        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                data = json.loads(resp.read().decode("utf-8"))
        except urllib.error.URLError as e:
            raise RuntimeError(
                f"[OllamaProvider Error] Không thể kết nối tới Ollama tại {self.base_url} ({e}). "
                "Hãy đảm bảo rằng tiến trình Ollama đang chạy (ví dụ: 'ollama serve')."
            ) from e

        content = data.get("message", {}).get("content", "")
        prompt_tokens = data.get("prompt_eval_count", 0)
        completion_tokens = data.get("eval_count", 0)

        return GenerationResult(
            text=content,
            provider="ollama",
            model=self.model,
            usage={
                "prompt_tokens": prompt_tokens,
                "completion_tokens": completion_tokens,
                "total_tokens": prompt_tokens + completion_tokens,
            },
            finish_reason=data.get("done_reason", "stop"),
            raw_response=data,
        )


class AnthropicProvider(BaseLLMProvider):
    """Nhà cung cấp LLM qua Anthropic Messages API (Claude)."""

    def __init__(
        self,
        api_key: str | None = None,
        model: str = "claude-3-5-sonnet-20241022",
        base_url: str = "https://api.anthropic.com",
        timeout: float = 60.0,
    ) -> None:
        self.api_key = api_key or os.environ.get("ANTHROPIC_API_KEY", "")
        if not self.api_key:
            raise ValueError(
                "[AnthropicProvider Error] Thiếu API key. Hãy truyền tham số api_key "
                "hoặc đặt biến môi trường ANTHROPIC_API_KEY."
            )
        self.model = model
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout

    def generate(
        self,
        system_prompt: str,
        user_prompt: str,
        temperature: float = 0.5,
    ) -> GenerationResult:
        endpoint = f"{self.base_url}/v1/messages"
        payload = {
            "model": self.model,
            "max_tokens": 4096,
            "temperature": temperature,
            "system": system_prompt,
            "messages": [
                {"role": "user", "content": user_prompt},
            ],
        }

        req = urllib.request.Request(
            endpoint,
            data=json.dumps(payload).encode("utf-8"),
            headers={
                "Content-Type": "application/json",
                "x-api-key": self.api_key,
                "anthropic-version": "2023-06-01",
            },
            method="POST",
        )

        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                data = json.loads(resp.read().decode("utf-8"))
        except urllib.error.HTTPError as e:
            err_body = e.read().decode("utf-8", errors="replace")
            raise RuntimeError(f"[AnthropicProvider HTTP Error {e.code}] {err_body}") from e
        except urllib.error.URLError as e:
            raise RuntimeError(f"[AnthropicProvider Error] Kết nối thất bại: {e}") from e

        content_blocks = data.get("content", [])
        text = "".join(b.get("text", "") for b in content_blocks if b.get("type") == "text")
        usage_data = data.get("usage", {})
        prompt_tokens = usage_data.get("input_tokens", 0)
        completion_tokens = usage_data.get("output_tokens", 0)

        return GenerationResult(
            text=text,
            provider="anthropic",
            model=self.model,
            usage={
                "prompt_tokens": prompt_tokens,
                "completion_tokens": completion_tokens,
                "total_tokens": prompt_tokens + completion_tokens,
            },
            finish_reason=data.get("stop_reason", "stop"),
            raw_response=data,
        )


class OpenAIProvider(BaseLLMProvider):
    """Nhà cung cấp LLM qua OpenAI-compatible Chat Completions API."""

    def __init__(
        self,
        api_key: str | None = None,
        model: str = "gpt-4o",
        base_url: str = "https://api.openai.com/v1",
        timeout: float = 60.0,
    ) -> None:
        self.api_key = api_key or os.environ.get("OPENAI_API_KEY", "")
        if not self.api_key:
            raise ValueError(
                "[OpenAIProvider Error] Thiếu API key. Hãy truyền tham số api_key "
                "hoặc đặt biến môi trường OPENAI_API_KEY."
            )
        self.model = model
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout

    def generate(
        self,
        system_prompt: str,
        user_prompt: str,
        temperature: float = 0.5,
    ) -> GenerationResult:
        endpoint = f"{self.base_url}/chat/completions"
        payload = {
            "model": self.model,
            "temperature": temperature,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt},
            ],
        }

        req = urllib.request.Request(
            endpoint,
            data=json.dumps(payload).encode("utf-8"),
            headers={
                "Content-Type": "application/json",
                "Authorization": f"Bearer {self.api_key}",
            },
            method="POST",
        )

        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                data = json.loads(resp.read().decode("utf-8"))
        except urllib.error.HTTPError as e:
            err_body = e.read().decode("utf-8", errors="replace")
            raise RuntimeError(f"[OpenAIProvider HTTP Error {e.code}] {err_body}") from e
        except urllib.error.URLError as e:
            raise RuntimeError(f"[OpenAIProvider Error] Kết nối thất bại: {e}") from e

        choices = data.get("choices", [])
        text = choices[0].get("message", {}).get("content", "") if choices else ""
        usage_data = data.get("usage", {})

        return GenerationResult(
            text=text,
            provider="openai",
            model=self.model,
            usage={
                "prompt_tokens": usage_data.get("prompt_tokens", 0),
                "completion_tokens": usage_data.get("completion_tokens", 0),
                "total_tokens": usage_data.get("total_tokens", 0),
            },
            finish_reason=choices[0].get("finish_reason", "stop") if choices else "stop",
            raw_response=data,
        )
