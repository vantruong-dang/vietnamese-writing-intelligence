#!/usr/bin/env python3
"""Kiểm tra dump viwiki mới nhất: ngày, tên file, kích thước, đã complete chưa."""
from __future__ import annotations

import re
import sys
import urllib.parse
import urllib.request

BASE = "https://dumps.wikimedia.org/viwiki/"
UA = {"User-Agent": "VWI-Corpus-Check/1.0"}


def fetch(url: str) -> str:
    req = urllib.request.Request(url, headers=UA)
    with urllib.request.urlopen(req, timeout=30) as r:
        return r.read().decode("utf-8", "ignore")


def head_size(url: str) -> int:
    req = urllib.request.Request(url, headers=UA, method="HEAD")
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            return int(r.headers.get("content-length", 0))
    except Exception:
        return 0


def main():
    html = fetch(BASE)
    dates = sorted(set(re.findall(r'href="(\d{8})/"', html)), reverse=True)
    print("latest_dates:", dates[:5])
    if not dates:
        print("no dumps found")
        return
    for date in dates[:3]:
        page = BASE + date + "/"
        try:
            sub = fetch(page)
        except Exception as exc:
            print(date, "ERROR", exc)
            continue
        files = re.findall(r'href="([^"]*pages-articles[^"]*\.bz2)"', sub)
        plain = [f for f in files if "multistream" not in f]
        status = "complete" if "Dump complete" in sub or "dump complete" in sub else "unknown"
        print(f"\n{date} status={status} plain_files={len(plain)}")
        for name in plain[:6]:
            url = urllib.parse.urljoin(page, name)
            size = head_size(url)
            print(f"  {name.rsplit('/', 1)[-1]}  {size/1024/1024:.1f} MB")


if __name__ == "__main__":
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    main()
