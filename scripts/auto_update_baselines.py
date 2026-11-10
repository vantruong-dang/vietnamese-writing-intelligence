#!/usr/bin/env python3
"""
auto_update_baselines.py
========================
Nhạc trưởng tự động hóa toàn diện (Zero-Touch Prosody Baseline Pipeline):
1. Cào dữ liệu mới (RSS Báo chí đa nguồn + Wikisource Văn học thuần Việt)
2. Gộp tự động vào các file corpus tổng với cơ chế khử trùng lặp (Deduplication)
3. Tính toán thống kê Baseline (mu, sigma, P95, P99) và sinh mã nguồn Rust `generated_prosody_baselines.rs`
4. [Tùy chọn] Chạy bộ kiểm thử Rust (`cargo test --workspace`) để đảm bảo chất lượng
5. [Tùy chọn] Tự động biên dịch phiên bản phát hành (`cargo build --release -p vwi-cli`)
"""

import argparse
import hashlib
import math
import subprocess
import sys
import time
from pathlib import Path

# Thêm thư mục scripts vào sys.path để import mô-đun anh em
SCRIPTS_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPTS_DIR.parent
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

try:
    from harvest_multi_source import (
        ADMINISTRATIVE_SAMPLE_CORPUS,
        DATA_DIR,
        LIT_DIR,
        NEWS_DIR,
        RUST_OUTPUT_FILE,
        TECHNICAL_SAMPLE_CORPUS,
        calculate_stats,
        harvest_journalism,
        harvest_wikisource,
    )
except ImportError as e:
    print(f"[!] Lỗi nhập mô-đun harvest_multi_source: {e}")
    sys.exit(1)

JOURNALISM_CORPUS = DATA_DIR / "journalism_corpus.txt"
HUMAN_GOLD_CORPUS = DATA_DIR / "human_gold.txt"


def smart_merge_directory(source_dir: Path, target_file: Path) -> tuple[int, int]:
    """
    Gộp tất cả file .txt trong source_dir vào target_file với cơ chế khử trùng lặp (Idempotent).
    Trả về (số file mới được thêm, tổng số file trong thư mục).
    """
    if not source_dir.exists():
        print(f"[-] Thư mục nguồn {source_dir.name} chưa tồn tại.")
        return 0, 0

    txt_files = sorted(source_dir.glob("*.txt"))
    if not txt_files:
        print(f"[-] Không có file .txt nào trong {source_dir.name}.")
        return 0, 0

    existing_content = ""
    if target_file.exists():
        existing_content = target_file.read_text(encoding="utf-8")

    # Tạo chỉ mục hash các đoạn văn đã có trong target_file để nhận diện trùng lặp
    existing_hashes = set()
    for block in existing_content.split("\n\n"):
        clean_block = block.strip()
        if len(clean_block) >= 30:
            existing_hashes.add(hashlib.md5(clean_block.encode("utf-8")).hexdigest())

    new_appended = 0
    newly_added_text = []

    for f in txt_files:
        content = f.read_text(encoding="utf-8").strip()
        if not content:
            continue

        file_hash = hashlib.md5(content.encode("utf-8")).hexdigest()
        # Kiểm tra nếu file đã có trong target_file
        if file_hash in existing_hashes or content in existing_content:
            continue

        newly_added_text.append(content)
        existing_hashes.add(file_hash)
        new_appended += 1

    if newly_added_text:
        with open(target_file, "a", encoding="utf-8") as out:
            if existing_content and not existing_content.endswith("\n\n"):
                out.write("\n\n")
            out.write("\n\n".join(newly_added_text) + "\n\n")

    print(
        f"[✓] Gộp {source_dir.name} -> {target_file.name}: "
        f"+{new_appended} file mới (Tổng {len(txt_files)} file nguồn)"
    )
    return new_appended, len(txt_files)


def generate_rust_baselines(lit_stats: dict, news_stats: dict, admin_stats: dict, tech_stats: dict) -> bool:
    """Sinh mã nguồn Rust chứa ma trận Baseline vào generated_prosody_baselines.rs."""
    print(f"\n[*] Đang sinh mã nguồn Rust: {RUST_OUTPUT_FILE.relative_to(ROOT_DIR)}...")

    RUST_OUTPUT_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(RUST_OUTPUT_FILE, "w", encoding="utf-8") as f:
        f.write("// FILE TỰ ĐỘNG SINH BỞI scripts/auto_update_baselines.py\n")
        f.write("// MA TRẬN BASELINE ĐA NGỮ VỰC (MULTI-REGISTER PROSODY BASELINES)\n\n")

        f.write("#[derive(Debug, Clone, Copy, PartialEq)]\n")
        f.write("pub struct ProsodyBaseline {\n")
        f.write("    pub mean_sentence_len: f32,\n")
        f.write("    pub std_dev_sentence_len: f32,\n")
        f.write("    pub mean_trac_ratio: f32,\n")
        f.write("    pub std_dev_trac_ratio: f32,\n")
        f.write("    pub p99_consecutive_trac_threshold: usize,\n")
        f.write("}\n\n")

        baselines = [
            ("LITERARY_ESSAY_BASELINE", lit_stats),
            ("JOURNALISM_BASELINE", news_stats),
            ("ADMINISTRATIVE_BASELINE", admin_stats),
            ("TECHNICAL_BASELINE", tech_stats),
        ]

        for var_name, s in baselines:
            f.write(f"pub static {var_name}: ProsodyBaseline = ProsodyBaseline {{\n")
            f.write(f"    mean_sentence_len: {s['mean_len']:.2f}f32,\n")
            f.write(f"    std_dev_sentence_len: {s['std_len']:.2f}f32,\n")
            f.write(f"    mean_trac_ratio: {s['mean_trac']:.4f}f32,\n")
            f.write(f"    std_dev_trac_ratio: {s['std_trac']:.4f}f32,\n")
            f.write(f"    p99_consecutive_trac_threshold: {math.ceil(s['p99_trac_run'])},\n")
            f.write("};\n\n")

        f.write("pub fn get_baseline_for_register(register: crate::types::Register) -> Option<&'static ProsodyBaseline> {\n")
        f.write("    match register {\n")
        f.write("        crate::types::Register::Literary => Some(&LITERARY_ESSAY_BASELINE),\n")
        f.write("        crate::types::Register::Journalism => Some(&JOURNALISM_BASELINE),\n")
        f.write("        crate::types::Register::Administrative => Some(&ADMINISTRATIVE_BASELINE),\n")
        f.write("        crate::types::Register::Technical => Some(&TECHNICAL_BASELINE),\n")
        f.write("        crate::types::Register::General => Some(&JOURNALISM_BASELINE),\n")
        f.write("    }\n")
        f.write("}\n")

    print("[✓] Xuất file mã nguồn Rust thành công!")
    return True


def run_command(cmd: list[str], desc: str) -> bool:
    """Chạy một lệnh hệ thống với hiển thị tiến trình đẹp mắt."""
    print(f"\n[*] {desc}...")
    print(f"    $ {' '.join(cmd)}")
    start = time.time()
    try:
        proc = subprocess.run(cmd, cwd=ROOT_DIR, check=True)
        elapsed = time.time() - start
        print(f"[✓] {desc} thành công trong {elapsed:.2f}s!")
        return proc.returncode == 0
    except subprocess.CalledProcessError as e:
        print(f"[!] {desc} thất bại với mã lỗi {e.returncode}")
        return False
    except FileNotFoundError:
        print(f"[!] Lệnh không tìm thấy: {cmd[0]}")
        return False


def main():
    parser = argparse.ArgumentParser(
        description="VWI Zero-Touch Prosody Baseline Pipeline (Orchestrator)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--skip-harvest",
        action="store_true",
        help="Bỏ qua bước cào mạng; sử dụng dữ liệu văn bản đã có sẵn trong data_raw",
    )
    parser.add_argument(
        "--test",
        action="store_true",
        help="Tự động chạy bộ kiểm thử Rust (cargo test --workspace) sau khi tạo baseline",
    )
    parser.add_argument(
        "--build",
        action="store_true",
        help="Tự động biên dịch phiên bản phát hành (cargo build --release -p vwi-cli)",
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Chạy toàn bộ pipeline từ đầu đến cuối (Cào + Gộp + Tính baseline + Test + Build Release)",
    )

    args = parser.parse_args()

    should_harvest = not args.skip_harvest
    should_test = args.test or args.all
    should_build = args.build or args.all

    print("=" * 65)
    print("      🚀 VWI ZERO-TOUCH EMPIRICAL BASELINE PIPELINE")
    print("=" * 65)
    print(f" Cấu hình: Cào mạng: {'BẬT' if should_harvest else 'BỎ QUA'} | "
          f"Kiểm thử: {'BẬT' if should_test else 'TẮT'} | "
          f"Biên dịch: {'BẬT' if should_build else 'TẮT'}")
    print("=" * 65)

    # 1. BƯỚC 1: THU HOẠCH DỮ LIỆU
    if should_harvest:
        print("\n>>> BƯỚC 1: THU HOẠCH DỮ LIỆU ĐA NGUỒN (RSS + WIKISOURCE)")
        harvest_journalism()
        harvest_wikisource()
    else:
        print("\n>>> BƯỚC 1: [BỎ QUA CÀO MẠNG] SỬ DỤNG DỮ LIỆU HIỆN CÓ")

    # 2. BƯỚC 2: GỘP & KHỬ TRÙNG LẶP CORPUS
    print("\n>>> BƯỚC 2: GỘP VÀ KHỬ TRÙNG LẶP CORPUS (IDEMPOTENT MERGE)")
    smart_merge_directory(NEWS_DIR, JOURNALISM_CORPUS)
    smart_merge_directory(LIT_DIR, HUMAN_GOLD_CORPUS)

    # 3. BƯỚC 3: TÍNH TOÁN BASELINE VÀ SINH FILE RUST
    print("\n>>> BƯỚC 3: TÍNH TOÁN THỐNG KÊ TOÀN DIỆN & SINH MÃ RUST")
    if not JOURNALISM_CORPUS.exists() or not HUMAN_GOLD_CORPUS.exists():
        print("[!] Lỗi: Thiếu file corpus cơ sở.")
        sys.exit(1)

    news_text = JOURNALISM_CORPUS.read_text(encoding="utf-8")
    gold_text = HUMAN_GOLD_CORPUS.read_text(encoding="utf-8")

    news_stats = calculate_stats(news_text, "Journalism (Báo chí thời sự)")
    lit_stats = calculate_stats(gold_text, "Literary (Văn học thuần Việt)")
    admin_stats = calculate_stats(ADMINISTRATIVE_SAMPLE_CORPUS, "Administrative (Hành chính / Công vụ)")
    tech_stats = calculate_stats(TECHNICAL_SAMPLE_CORPUS, "Technical (Kỹ thuật / Khoa học)")

    generate_rust_baselines(lit_stats, news_stats, admin_stats, tech_stats)

    # 4. BƯỚC 4: KIỂM THỬ RUST (TÙY CHỌN HOẶC --ALL)
    if should_test:
        print("\n>>> BƯỚC 4: KIỂM THỬ RUST REGRESSION SUITE")
        test_ok = run_command(["cargo", "test", "--workspace"], "Kiểm thử Cargo Workspace")
        if not test_ok:
            print("[!] CẢNH BÁO: Bộ kiểm thử Rust phát hiện lỗi!")
            sys.exit(1)

    # 5. BƯỚC 5: BIÊN DỊCH BẢN PHÁT HÀNH (TÙY CHỌN HOẶC --ALL)
    if should_build:
        print("\n>>> BƯỚC 5: BIÊN DỊCH RELEASE BINARY")
        build_ok = run_command(["cargo", "build", "--release", "-p", "vwi-cli"], "Biên dịch Release Binary (vwi.exe)")
        if not build_ok:
            print("[!] CẢNH BÁO: Biên dịch thất bại!")
            sys.exit(1)

    print("\n" + "=" * 65)
    print(" 🎉 TOÀN BỘ ZERO-TOUCH PIPELINE ĐÃ HOÀN TẤT THÀNH CÔNG!")
    print(f" - Rust Baseline: {RUST_OUTPUT_FILE.relative_to(ROOT_DIR)}")
    print(f" - Journalism Corpus: {len(news_text.split()):,} từ")
    print(f" - Literary Corpus:   {len(gold_text.split()):,} từ")
    print("=" * 65)


if __name__ == "__main__":
    main()
