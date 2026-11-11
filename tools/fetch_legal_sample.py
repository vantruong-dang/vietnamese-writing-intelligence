#!/usr/bin/env python3
"""Lấy mẫu văn bản pháp lý từ HF datasets-server (không cần tải 748MB parquet).

Nguồn: th1nhng0/vietnamese-legal-documents, config=content, split=data.
Chỉ lấy ~12M ký tự (~1.5-2M token) để bổ sung văn phong hành chính.
"""
from __future__ import annotations

import html
import re
import sys
import time
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "data_raw" / "legal_sample.txt"
DATASET = "th1nhng0/vietnamese-legal-documents"
MAX_CHARS = 12_000_000
BATCH = 100
MAX_ROWS = 6000


def fetch(offset: int, length: int):
    query = urllib.parse.urlencode(
        {
            "dataset": DATASET,
            "config": "content",
            "split": "data",
            "offset": offset,
            "length": length,
        }
    )
    url = "https://datasets-server.huggingface.co/rows?" + query
    req = urllib.request.Request(url, headers={"User-Agent": "VWI/1.0"})
    with urllib.request.urlopen(req, timeout=120) as r:
        import json

        return json.load(r)


def clean_html(raw: str) -> str:
    raw = re.sub(r"(?is)<(style|script)[^>]*>.*?</\1>", " ", raw)
    raw = re.sub(r"(?s)<[^>]+>", " ", raw)
    raw = html.unescape(raw)
    raw = re.sub(r"\s+", " ", raw).strip()
    return raw


def main():
    total_chars = 0
    written = 0
    OUT.parent.mkdir(exist_ok=True)
    with OUT.open("w", encoding="utf-8") as out:
        for offset in range(0, MAX_ROWS, BATCH):
            for _attempt in range(3):
                try:
                    payload = fetch(offset, BATCH)
                    break
                except Exception as exc:
                    print(f"[!] offset {offset} lỗi: {exc}; thử lại...")
                    time.sleep(2)
            else:
                print(f"[!] bỏ offset {offset}")
                continue

            for row in payload.get("rows", []):
                content = row.get("row", {}).get("content_html", "")
                text = clean_html(content)
                if len(text) < 100:
                    continue
                # tách đoạn để giữ dòng vừa phải
                for para in re.split(r"(?<=[.!?])\s+", text):
                    para = para.strip()
                    if len(para) < 40:
                        continue
                    out.write(para + "\n")
                    total_chars += len(para) + 1
                    written += 1
                if total_chars >= MAX_CHARS:
                    break
            print(f"  offset={offset} chars={total_chars/1_000_000:.1f}M lines={written}")
            if total_chars >= MAX_CHARS:
                break

    size = OUT.stat().st_size / 1024 / 1024
    print(f"[+] {OUT} | {written:,} dòng | {size:.1f} MB")


if __name__ == "__main__":
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    main()
