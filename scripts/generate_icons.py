"""KỊCH BẢN TỰ ĐỘNG SINH BỘ ICON CHROME EXTENSION CHO VWI
- Vẽ biểu tượng Ngòi bút vàng cách điệu chữ V trên nền đỏ bo góc
- Xuất tự động 3 kích thước: 16x16, 48x48, 128x128 pixel
- Sử dụng thuật toán Supersampling 4x chống răng cưa tuyệt đối
"""

from pathlib import Path

from PIL import Image, ImageDraw

ROOT_DIR = Path(__file__).resolve().parent.parent
ICONS_DIR = ROOT_DIR / "extensions" / "chrome" / "icons"
ICONS_DIR.mkdir(parents=True, exist_ok=True)


def draw_vwi_icon(size: int) -> Image.Image:
    # Vẽ ở độ phân giải cao 512x512 để khử răng cưa mượt mà
    canvas_size = 512
    img = Image.new("RGBA", (canvas_size, canvas_size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # 1. Vẽ nền bo góc (Squircle) màu đỏ thẫm Crimson
    bg_box = [20, 20, canvas_size - 20, canvas_size - 20]
    corner_radius = 110
    draw.rounded_rectangle(bg_box, radius=corner_radius, fill="#991B1B", outline="#7F1D1D", width=6)

    # 2. Vẽ ngòi bút vàng cách điệu hình chữ "V" (Golden Pen Nib)
    tip = (256, 435)             # Mũi ngòi bút hướng xuống
    shoulder_left = (120, 210)    # Vai trái
    shoulder_right = (392, 210)   # Vai phải
    top_left = (150, 110)         # Đỉnh trái
    top_right = (362, 110)        # Đỉnh phải

    # Nửa trái ngòi bút (vàng sáng phản quang #FBBF24)
    left_polygon = [
        top_left,
        shoulder_left,
        tip,
        (256, 260),               # Đáy lỗ thở
        (235, 240),               # Cạnh trái lỗ thở
        (235, 200),
        (256, 175),
        (256, 110),
    ]
    draw.polygon(left_polygon, fill="#FBBF24")

    # Nửa phải ngòi bút (vàng hổ phách đổ bóng #D97706)
    right_polygon = [
        top_right,
        shoulder_right,
        tip,
        (256, 260),
        (277, 240),
        (277, 200),
        (256, 175),
        (256, 110),
    ]
    draw.polygon(right_polygon, fill="#D97706")

    # 3. Rãnh xẻ mực trung tâm (Ink Slit)
    draw.line([(256, 435), (256, 250)], fill="#7F1D1D", width=10)

    # 4. Lỗ thở trung tâm (Breather Hole) hình thoi / kim cương sang trọng
    diamond = [
        (256, 205),  # đỉnh trên
        (275, 235),  # đỉnh phải
        (256, 265),  # đỉnh dưới
        (237, 235),  # đỉnh trái
    ]
    draw.polygon(diamond, fill="#991B1B")

    # Thu nhỏ về kích thước thực tế bằng bộ lọc LANCZOS chất lượng cao nhất
    resized = img.resize((size, size), Image.Resampling.LANCZOS)
    return resized


def main():
    print("=" * 60)
    print(">>> ĐANG TỰ ĐỘNG SINH BỘ ICON VWI CHROME EXTENSION")
    print("=" * 60)

    sizes = [16, 48, 128]
    for s in sizes:
        icon_path = ICONS_DIR / f"icon{s}.png"
        icon_img = draw_vwi_icon(s)
        icon_img.save(icon_path, "PNG")
        print(f"[+] Đã tạo icon {s}x{s}: {icon_path.relative_to(ROOT_DIR)}")

    print("\n✅ Hoàn tất 100%! Thư mục icons đã sẵn sàng đóng gói.")


if __name__ == "__main__":
    main()
