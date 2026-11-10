#!/usr/bin/env python3
"""Đóng gói Chrome Extension thành tệp zip sẵn sàng phát hành trên Chrome Web Store."""

import zipfile
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent.parent
EXT_DIR = ROOT_DIR / "extensions" / "chrome"
DIST_DIR = ROOT_DIR / "dist"
DIST_DIR.mkdir(parents=True, exist_ok=True)
ZIP_PATH = DIST_DIR / "vwi-chrome-extension-v0.3.1.zip"

INCLUDED_FILES = [
    "manifest.json",
    "popup.html",
    "popup.js",
    "icons/icon16.png",
    "icons/icon48.png",
    "icons/icon128.png",
    "wasm/vwi_wasm.js",
    "wasm/vwi_wasm_bg.wasm",
]


def package():
    print(f"[*] Đóng gói VWI Chrome Extension vào: {ZIP_PATH}...")
    with zipfile.ZipFile(ZIP_PATH, "w", zipfile.ZIP_DEFLATED) as zf:
        for rel in INCLUDED_FILES:
            file_path = EXT_DIR / rel
            if not file_path.exists():
                raise FileNotFoundError(f"Thiếu tệp: {file_path}")
            zf.write(file_path, arcname=rel)
            size_kb = file_path.stat().st_size / 1024.0
            print(f"  + {rel} ({size_kb:.1f} KB)")

    total_size_mb = ZIP_PATH.stat().st_size / (1024.0 * 1024.0)
    print(f"\n[+] Đóng gói THÀNH CÔNG: {ZIP_PATH} ({total_size_mb:.2f} MB)")


if __name__ == "__main__":
    package()
