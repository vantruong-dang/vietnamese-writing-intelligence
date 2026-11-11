#!/usr/bin/env python3
"""Tải Leipzig vie_news_2022 (1M câu) và xuất text sạch cho bigram LM."""
from __future__ import annotations

import re
import sys
import tarfile
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RAW = ROOT / "data_raw"
URL = "https://downloads.wortschatz-leipzig.de/corpora/vie_news_2022_1M.tar.gz"
ARCHIVE = RAW / "vie_news_2022_1M.tar.gz"
OUT = RAW / "leipzig_news.txt"


def download():
    RAW.mkdir(exist_ok=True)
    if ARCHIVE.exists() and ARCHIVE.stat().st_size > 100_000_000:
        print(f"[*] Đã có {ARCHIVE.name}")
        return
    print(f"[*] Tải {URL} ...")
    req = urllib.request.Request(URL, headers={"User-Agent": "VWI/1.0"})
    with urllib.request.urlopen(req, timeout=60) as r, ARCHIVE.open("wb") as f:
        total = int(r.headers.get("content-length", 0))
        done = 0
        while True:
            chunk = r.read(1024 * 1024)
            if not chunk:
                break
            f.write(chunk)
            done += len(chunk)
            if total:
                print(f"\r  {done/1024/1024:.1f}/{total/1024/1024:.1f} MB", end="")
    print()


def extract():
    print("[*] Giải nén...")
    with tarfile.open(ARCHIVE, "r:gz") as tar:
        members = [m for m in tar.getmembers() if m.name.endswith("-sentences.txt")]
        if not members:
            raise SystemExit("Không tìm thấy *-sentences.txt trong archive")
        member = members[0]
        print(f"  -> {member.name}")
        fh = tar.extractfile(member)
        assert fh is not None
        count = 0
        with OUT.open("w", encoding="utf-8") as out:
            for raw in fh:
                line = raw.decode("utf-8", "ignore").rstrip("\n")
                # format: <id>\t<sentence>
                if "\t" in line:
                    _, sentence = line.split("\t", 1)
                else:
                    sentence = line
                sentence = re.sub(r"\s+", " ", sentence).strip()
                if len(sentence) >= 20:
                    out.write(sentence + "\n")
                    count += 1
        print(f"[+] {OUT} | {count:,} câu | {OUT.stat().st_size/1024/1024:.1f} MB")


if __name__ == "__main__":
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    download()
    extract()
