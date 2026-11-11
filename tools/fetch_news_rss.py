#!/usr/bin/env python3
"""Thu thập tiêu đề + mô tả từ RSS VnExpress (nguồn công khai) để bổ sung LM.

Ghi ra data_raw/news_fresh.txt, mỗi dòng một đoạn. Không ghi đè dữ liệu cũ.
Chỉ dùng cho LM nội bộ; không tái phân phối nội dung.
"""
from __future__ import annotations

import re
import sys
import urllib.request
import xml.etree.ElementTree as ET
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "data_raw" / "news_fresh.txt"

FEEDS = [
    "https://vnexpress.net/rss/tin-moi-nhat.rss",
    "https://vnexpress.net/rss/khoa-hoc.rss",
    "https://vnexpress.net/rss/so-hoa.rss",
    "https://vnexpress.net/rss/kinh-doanh.rss",
    "https://vnexpress.net/rss/the-gioi.rss",
    "https://vnexpress.net/rss/giao-duc.rss",
]


def clean_html(text: str) -> str:
    text = re.sub(r"<[^>]+>", " ", text)
    text = text.replace("&nbsp;", " ")
    return re.sub(r"\s+", " ", text).strip()


def main():
    lines: list[str] = []
    for feed in FEEDS:
        try:
            req = urllib.request.Request(feed, headers={"User-Agent": "Mozilla/5.0"})
            with urllib.request.urlopen(req, timeout=15) as resp:
                tree = ET.fromstring(resp.read())
            count = 0
            for item in tree.findall(".//item"):
                title = item.find("title")
                desc = item.find("description")
                if title is not None and title.text:
                    lines.append(clean_html(title.text))
                    count += 1
                if desc is not None and desc.text:
                    cleaned = clean_html(desc.text)
                    if len(cleaned) > 20:
                        lines.append(cleaned)
            print(f"[+] {feed.split('/')[-1]}: {count} tiêu đề")
        except Exception as exc:
            print(f"[!] Lỗi {feed}: {exc}")

    # dedupe
    unique = list(dict.fromkeys(lines))
    OUT.parent.mkdir(exist_ok=True)
    OUT.write_text("\n".join(unique) + "\n", encoding="utf-8")
    print(f"[+] Đã ghi {len(unique)} dòng -> {OUT}")


if __name__ == "__main__":
    main()
