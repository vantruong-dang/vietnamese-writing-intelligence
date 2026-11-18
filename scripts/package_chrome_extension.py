#!/usr/bin/env python3
"""Đóng gói Chrome Extension thành tệp zip sẵn sàng phát hành trên Chrome Web Store.

Lưu ý: `wasm-pack` sinh ra `extensions/chrome/wasm/.gitignore` chứa `*`, nên mọi
công cụ tôn trọng .gitignore (`git archive`, một số script zip) sẽ **bỏ mất**
`vwi_wasm_bg.wasm`. Script này dùng danh sách tệp tường minh và tự kiểm tra lại
nội dung zip sau khi ghi.
"""

import json
import re
import zipfile
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent.parent
EXT_DIR = ROOT_DIR / "extensions" / "chrome"
DIST_DIR = ROOT_DIR / "dist"

INCLUDED_FILES = [
    "manifest.json",
    "popup.html",
    "popup.js",
    "welcome.html",
    "background.js",
    "content.js",
    "icons/icon16.png",
    "icons/icon48.png",
    "icons/icon128.png",
    "wasm/vwi_wasm.js",
    "wasm/vwi_wasm_bg.wasm",
]

# Tệp wasm luôn > 1 MB; nếu nhỏ hơn nghĩa là build lỗi hoặc file rỗng.
MIN_WASM_BYTES = 1_000_000


def _read_versions() -> dict[str, str]:
    cargo = re.search(
        r'(?m)^version\s*=\s*"([^"]+)"', (ROOT_DIR / "Cargo.toml").read_text(encoding="utf-8")
    )
    pyproject = re.search(
        r'(?m)^version\s*=\s*"([^"]+)"', (ROOT_DIR / "pyproject.toml").read_text(encoding="utf-8")
    )
    manifest = json.loads((EXT_DIR / "manifest.json").read_text(encoding="utf-8"))
    return {
        "Cargo.toml": cargo.group(1) if cargo else "<không đọc được>",
        "pyproject.toml": pyproject.group(1) if pyproject else "<không đọc được>",
        "manifest.json": manifest.get("version", "<không đọc được>"),
    }


def validate_versions() -> str:
    print("[*] Kiểm tra đồng bộ phiên bản (Release Gate)...")
    versions = _read_versions()
    unique = set(versions.values())
    if len(unique) != 1:
        detail = ", ".join(f"{name}={ver}" for name, ver in versions.items())
        raise ValueError(f"❌ Lệch phiên bản giữa các file cấu hình: {detail}")
    version = unique.pop()
    print(f"  ✓ Cargo.toml, pyproject.toml, manifest.json đều là {version}")
    return version


def validate_html_files():
    print("[*] Kiểm tra tĩnh chất lượng giao diện (HTML Linter Guard)...")
    for html_file in EXT_DIR.glob("*.html"):
        content = html_file.read_text(encoding="utf-8")
        if "$" in content:
            raise ValueError(
                f"❌ Phát hiện ký tự '$' (nghi ngờ sót cú pháp LaTeX/Markdown) trong file: {html_file.name}"
            )
    print("  ✓ Toàn bộ file HTML sạch bóng cú pháp LaTeX/Markdown rác.")


def validate_wasm_artifact():
    wasm_path = EXT_DIR / "wasm" / "vwi_wasm_bg.wasm"
    if not wasm_path.exists():
        raise FileNotFoundError(f"Thiếu tệp wasm: {wasm_path} (chạy wasm-pack build trước)")
    size = wasm_path.stat().st_size
    if size < MIN_WASM_BYTES:
        raise ValueError(f"❌ vwi_wasm_bg.wasm chỉ {size} bytes — nghi ngờ build lỗi.")
    print(f"  ✓ wasm artifact hợp lệ ({size / 1024 / 1024:.2f} MB)")


def verify_zip(zip_path: Path):
    print("[*] Kiểm tra lại nội dung zip (chống rơi file vì .gitignore)...")
    with zipfile.ZipFile(zip_path) as zf:
        names = set(zf.namelist())
        missing = {rel for rel in INCLUDED_FILES if rel not in names}
        if missing:
            raise ValueError(f"❌ Zip thiếu tệp: {sorted(missing)}")
        unexpected = names - set(INCLUDED_FILES)
        if unexpected:
            raise ValueError(f"❌ Zip có tệp lạ: {sorted(unexpected)}")
        wasm_info = zf.getinfo("wasm/vwi_wasm_bg.wasm")
        if wasm_info.file_size < MIN_WASM_BYTES:
            raise ValueError("❌ vwi_wasm_bg.wasm trong zip bị rỗng/hỏng.")
    print(f"  ✓ Đủ {len(INCLUDED_FILES)} tệp, gồm wasm {wasm_info.file_size / 1024 / 1024:.2f} MB")


def package():
    version = validate_versions()
    validate_html_files()
    validate_wasm_artifact()

    DIST_DIR.mkdir(parents=True, exist_ok=True)
    zip_path = DIST_DIR / f"vwi-chrome-extension-v{version}.zip"
    print(f"[*] Đóng gói VWI Chrome Extension vào: {zip_path}...")
    with zipfile.ZipFile(zip_path, "w", zipfile.ZIP_DEFLATED) as zf:
        for rel in INCLUDED_FILES:
            file_path = EXT_DIR / rel
            if not file_path.exists():
                raise FileNotFoundError(f"Thiếu tệp: {file_path}")
            zf.write(file_path, arcname=rel)
            size_kb = file_path.stat().st_size / 1024.0
            print(f"  + {rel} ({size_kb:.1f} KB)")

    verify_zip(zip_path)
    total_size_mb = zip_path.stat().st_size / (1024.0 * 1024.0)
    print(f"\n[+] Đóng gói THÀNH CÔNG: {zip_path} ({total_size_mb:.2f} MB)")


if __name__ == "__main__":
    package()
