#!/usr/bin/env python3
"""Tải một shard viwiki + extract + gộp thành corpus sạch (bản pilot).

Ví dụ:
  python tools/fetch_viwiki_shard.py \
    --url https://dumps.wikimedia.org/viwiki/20261101/viwiki-20261101-pages-articles2.xml-p832083p1952085.bz2
"""
from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
import urllib.request
from pathlib import Path

from tqdm import tqdm

ROOT = Path(__file__).resolve().parent.parent
RAW = ROOT / "data_raw"
DUMP_DIR = RAW / "viwiki_dump"
EXTRACT_DIR = RAW / "viwiki_extracted"
OUTPUT = RAW / "viwiki_clean.txt"


def download(url: str, dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.exists() and dest.stat().st_size > 1_000_000:
        print(f"[*] Đã có {dest.name} ({dest.stat().st_size/1024/1024:.1f} MB)")
        return
    req = urllib.request.Request(url, headers={"User-Agent": "VWI-Corpus/1.0"})
    with urllib.request.urlopen(req, timeout=60) as r:
        total = int(r.headers.get("content-length", 0))
        with dest.open("wb") as f, tqdm(
            total=total, unit="B", unit_scale=True, desc=dest.name
        ) as bar:
            while True:
                chunk = r.read(1024 * 1024)
                if not chunk:
                    break
                f.write(chunk)
                bar.update(len(chunk))
    print(f"[+] Đã tải {dest}")


def extract(bz2_path: Path) -> None:
    if EXTRACT_DIR.exists():
        shutil.rmtree(EXTRACT_DIR)
    EXTRACT_DIR.mkdir(parents=True)
    cmd = [
        sys.executable,
        "-m",
        "wikiextractor.WikiExtractor",
        str(bz2_path),
        "-o",
        str(EXTRACT_DIR),
        "--no-templates",
        "--processes",
        "4",
        "-q",
    ]
    print("[*] Đang extract...")
    result = subprocess.run(cmd)
    if result.returncode != 0:
        raise SystemExit("wikiextractor thất bại")
    print("[+] Extract xong")


def merge(max_lines: int) -> None:
    print("[*] Đang gộp + làm sạch...")
    total = 0
    with OUTPUT.open("w", encoding="utf-8") as out:
        for path in sorted(EXTRACT_DIR.rglob("wiki_*")):
            with path.open(encoding="utf-8", errors="ignore") as f:
                for line in f:
                    line = line.strip()
                    if line.startswith("<doc") or line.startswith("</doc>"):
                        continue
                    if len(line) < 40:
                        continue
                    line = re.sub(r"\s+", " ", line)
                    out.write(line + "\n")
                    total += 1
                    if total >= max_lines:
                        break
            if total >= max_lines:
                break
    size = OUTPUT.stat().st_size / 1024 / 1024
    print(f"[+] {OUTPUT} | {total:,} dòng | {size:.1f} MB")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--url", required=True)
    ap.add_argument("--max-lines", type=int, default=1_000_000)
    args = ap.parse_args()

    name = args.url.rsplit("/", 1)[-1]
    bz2_path = DUMP_DIR / name
    download(args.url, bz2_path)
    extract(bz2_path)
    merge(args.max_lines)


if __name__ == "__main__":
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    main()
