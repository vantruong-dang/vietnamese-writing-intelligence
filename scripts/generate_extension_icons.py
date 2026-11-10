#!/usr/bin/env python3
"""Tạo các biểu tượng PNG (16x16, 48x48, 128x128) cho VWI Chrome Extension."""

import math
import struct
import zlib
from pathlib import Path

OUT_DIR = Path(__file__).resolve().parent.parent / "extensions" / "chrome" / "icons"
OUT_DIR.mkdir(parents=True, exist_ok=True)


def draw_icon(size: int) -> bytes:
    width = size
    height = size
    cx = width / 2.0
    cy = height / 2.0
    r_outer = size * 0.45

    # Màu sắc: Nền đỏ (#DA251D), Ngôi sao vàng (#FFFF00)
    bg_r, bg_g, bg_b = 218, 37, 29
    star_r, star_g, star_b = 255, 255, 0

    # Hàm kiểm tra điểm (x, y) có nằm trong ngôi sao 5 cánh không
    def in_star(px: float, py: float) -> bool:
        dx = px - cx
        dy = py - cy
        dist = math.hypot(dx, dy)
        if dist > r_outer:
            return False
        # Góc từ tâm (-pi đến pi)
        angle = math.atan2(dy, dx) - (math.pi / 2.0)
        # Chuẩn hóa về [0, 2*pi]
        angle = (angle + 2 * math.pi) % (2 * math.pi)
        # Mỗi cánh 72 độ (2*pi/5)
        segment = (2 * math.pi) / 5.0
        local_angle = (angle % segment) - (segment / 2.0)
        # Bán kính biên ngôi sao tại góc này
        r_inner = r_outer * 0.382
        # Góc nhọn
        max_r = r_inner / math.cos(math.pi / 5.0 - abs(local_angle))
        return dist <= max_r

    raw_data = bytearray()
    for y in range(height):
        raw_data.append(0)  # filter type 0 (None)
        for x in range(width):
            # Điểm trong ngôi sao vàng
            if in_star(x + 0.5, y + 0.5):
                raw_data.extend([star_r, star_g, star_b, 255])
            else:
                raw_data.extend([bg_r, bg_g, bg_b, 255])

    compressed = zlib.compress(bytes(raw_data), level=9)

    def chunk(tag: bytes, data: bytes) -> bytes:
        crc = zlib.crc32(tag + data) & 0xFFFFFFFF
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)

    header = b"\x89PNG\r\n\x1a\n"
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return header + chunk(b"IHDR", ihdr) + chunk(b"IDAT", compressed) + chunk(b"IEND", b"")


for s in [16, 48, 128]:
    png_bytes = draw_icon(s)
    out_file = OUT_DIR / f"icon{s}.png"
    out_file.write_bytes(png_bytes)
    print(f"[*] Đã sinh icon: {out_file} ({len(png_bytes)} bytes)")
