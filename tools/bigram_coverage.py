#!/usr/bin/env python3
"""Đo độ phủ của bigram hiện có (generated_lm.rs) trên bộ H.

Mục đích: trước khi mở rộng corpus, biết bigram hiện tại có chứa các cặp
đúng cần thiết cho real-word hay không. Không dùng để tinh chỉnh threshold.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

ROOT = Path(__file__).resolve().parent.parent
LM = ROOT / "crates/vwi-core/src/segmentation/generated_lm.rs"

BIGRAM_RE = re.compile(r'\("([^"]*)", "([^"]*)", (-?\d+(?:\.\d+)?)\)')
UNIGRAM_RE = re.compile(r'^\s*\("([^"]*)", (-?\d+(?:\.\d+)?)\),\s*$', re.M)

# (ca, bigram đúng kỳ vọng, bigram sai đang có trong câu)
CASES = [
    ("H29", ("cơn", "bão"), ("cơn", "bảo")),
    ("H30", ("đi", "tắm"), ("đi", "tám")),
    ("H31", ("sau", "bữa"), ("sao", "bữa")),
    ("H32", ("buổi", "sáng"), ("buổi", "sang")),
    ("H33", ("bên", "cạnh"), ("bên", "canh")),
    ("H34", ("chiếc", "váy"), ("chiếc", "vay")),
    ("H35", ("để", "nấu"), ("đẻ", "nấu")),
    ("H36", ("rất", "lạnh"), ("rất", "nạnh")),
    ("H37", ("giấy", "tờ"), ("dấy", "tờ")),
    ("H38", ("cuốn", "sách"), ("cuống", "sách")),
]

CORRECT_SENTS = [
    "Anh ấy đứng thứ ba trong cuộc thi chạy bộ.",
    "Các cơ quan nhà nước phải công khai thông tin.",
    "Cuộc họp bắt đầu lúc 8 giờ 30 phút sáng nay.",
    "Công ty vừa ký hợp đồng trị giá 2,5 tỷ đồng.",
    "Hà Nội, ngày 03 tháng 12 năm 2026",
    "Căn cứ khoản 2 Điều 7 Luật Lưu trữ năm 2024.",
    "Trận Hà Nội FC vs Hải Phòng diễn ra tối nay.",
    "Bà ngoại kể chuyện cổ tích cho các cháu nghe.",
    "Thủ tướng Chính phủ chủ trì cuộc họp sáng nay.",
    "Thành phố Hồ Chí Minh là trung tâm kinh tế lớn của cả nước.",
    "Bà cụ khóa cửa rồi đi chợ, tay cầm một chiếc khóa cũ.",
    "Hôm qua tôi mới biết mai trời sẽ mưa.",
    "Quý khách vui lòng đợi trong giây lát.",
    "Tổng chi phí khoảng 1.250.000 đồng.",
    "Nhân dân ta luôn đoàn kết một lòng.",
    "Cô giáo cho cả lớp nghỉ sớm vì trời mưa to.",
    "Anh ấy chạy rất nhanh nên về nhất.",
    "Kính gửi: các sở, ban, ngành thuộc tỉnh.",
    "Kính gửi: Các sở, ban, ngành thuộc tỉnh.",
    "Ông Trần Quang Minh sinh năm 1960 tại Cần Thơ.",
]


def main():
    text = LM.read_text(encoding="utf-8")
    bigrams = {(a, b) for a, b, _ in BIGRAM_RE.findall(text)}
    unigrams = {w for w, _ in UNIGRAM_RE.findall(text)}
    print(f"bigrams={len(bigrams)} unigrams={len(unigrams)}")

    print("\n--- H real-word: bigram đúng có trong LM? ---")
    covered = 0
    for cid, good, bad in CASES:
        has_good = good in bigrams
        has_bad = bad in bigrams
        covered += 1 if has_good else 0
        print(
            f"{cid:<4} good={good!r:28} {str(has_good):<5} "
            f"bad={bad!r:28} {has_bad}"
        )
    print(f"coverage_good={covered}/10")

    print("\n--- 20 câu đúng H: tỷ lệ bigram lạ ---")
    total = missing = 0
    for sent in CORRECT_SENTS:
        words = re.findall(r"[^\W\d_]+", sent.lower(), flags=re.UNICODE)
        for a, b in zip(words, words[1:]):
            total += 1
            if (a, b) not in bigrams:
                missing += 1
    print(f"pairs={total} missing={missing} ({missing/total*100:.1f}%)")


if __name__ == "__main__":
    main()
