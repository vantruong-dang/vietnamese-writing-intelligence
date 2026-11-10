from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

ARTIFACTS_DIR = Path(r"C:\Users\Admin\.gemini\antigravity\brain\29d1798a-b7de-4107-bbd5-b3c3c072a0a1")
OUTPUT_DIR = Path("dist/screenshots")
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

TARGET_WIDTH = 1280
TARGET_HEIGHT = 800
TARGET_RATIO = TARGET_WIDTH / TARGET_HEIGHT  # 1.6 (16:10)

def process_studio_image(img_path: Path, out_name: str):
    if not img_path.exists():
        print(f"⚠️ Không tìm thấy file: {img_path}")
        return

    img = Image.open(img_path)
    orig_w, orig_h = img.size
    orig_ratio = orig_w / orig_h

    print(f"🖼️ Đang xử lý: {img_path.name} (Gốc: {orig_w}x{orig_h})")

    # Center Crop
    if orig_ratio > TARGET_RATIO:
        new_width = int(orig_h * TARGET_RATIO)
        left = (orig_w - new_width) // 2
        img = img.crop((left, 0, left + new_width, orig_h))
    elif orig_ratio < TARGET_RATIO:
        new_height = int(orig_w / TARGET_RATIO)
        top = (orig_h - new_height) // 2
        img = img.crop((0, top, orig_w, top + new_height))

    resized_img = img.resize((TARGET_WIDTH, TARGET_HEIGHT), Image.Resampling.LANCZOS)
    out_path = OUTPUT_DIR / out_name
    resized_img.save(out_path, format="PNG")
    print(f"✅ Đã lưu ảnh chuẩn Store: {out_path} ({TARGET_WIDTH}x{TARGET_HEIGHT})")

def process_popup_mockup(img_path: Path, out_name: str):
    if not img_path.exists():
        print(f"⚠️ Không tìm thấy file: {img_path}")
        return

    popup = Image.open(img_path).convert("RGBA")
    pw, ph = popup.size

    # Tạo canvas nền đẹp chuẩn 1280x800
    canvas = Image.new("RGBA", (TARGET_WIDTH, TARGET_HEIGHT), (241, 245, 249, 255))

    # Vẽ nền gradient nhẹ hoặc viền nhẹ
    draw = ImageDraw.Draw(canvas)
    for y in range(TARGET_HEIGHT):
        ratio = y / TARGET_HEIGHT
        r = int(241 * (1 - ratio) + 226 * ratio)
        g = int(245 * (1 - ratio) + 232 * ratio)
        b = int(249 * (1 - ratio) + 240 * ratio)
        draw.line([(0, y), (TARGET_WIDTH, y)], fill=(r, g, b, 255))

    # Tọa độ căn giữa
    scale = min(1.0, 720.0 / ph)
    if scale < 1.0:
        new_pw = int(pw * scale)
        new_ph = int(ph * scale)
        popup = popup.resize((new_pw, new_ph), Image.Resampling.LANCZOS)
        pw, ph = new_pw, new_ph

    pos_x = (TARGET_WIDTH - pw) // 2
    pos_y = (TARGET_HEIGHT - ph) // 2

    # Đổ bóng (Drop shadow)
    shadow = Image.new("RGBA", (TARGET_WIDTH, TARGET_HEIGHT), (0, 0, 0, 0))
    s_draw = ImageDraw.Draw(shadow)
    s_draw.rounded_rectangle(
        [pos_x - 4, pos_y - 2, pos_x + pw + 4, pos_y + ph + 8],
        radius=14,
        fill=(0, 0, 0, 70)
    )
    shadow = shadow.filter(ImageFilter.GaussianBlur(16))
    canvas.paste(shadow, (0, 0), shadow)

    # Dán popup lên canvas
    canvas.paste(popup, (pos_x, pos_y), popup)

    out_path = OUTPUT_DIR / out_name
    canvas.convert("RGB").save(out_path, format="PNG")
    print(f"✅ Đã lưu ảnh Mockup Popup chuẩn Store: {out_path} ({TARGET_WIDTH}x{TARGET_HEIGHT})")

if __name__ == "__main__":
    print("🚀 Bắt đầu chuẩn hóa bộ ảnh chất lượng cao cho Chrome Web Store...")

    # 1. Studio View (Dễ hiểu)
    process_studio_image(ARTIFACTS_DIR / "vwi_chrome_studio_friendly.png", "01_studio_friendly_mode_1280x800.png")

    # 2. Studio View (Dành cho Dev)
    process_studio_image(ARTIFACTS_DIR / "vwi_chrome_studio_dev.png", "02_studio_dev_mode_1280x800.png")

    # 3. AutoFix Interaction (Sau khi bấm Áp dụng)
    process_studio_image(ARTIFACTS_DIR / "vwi_chrome_after_autofix.png", "03_studio_after_autofix_1280x800.png")

    # 4. Standard Popup Mockup (Căn giữa nghệ thuật trên nền Store 1280x800)
    process_popup_mockup(ARTIFACTS_DIR / "vwi_chrome_popup_standard.png", "04_popup_mockup_1280x800.png")

    print(f"\n🎉 HOÀN TẤT! Toàn bộ 4 ảnh 1280x800 chuẩn tuyệt đối đã sẵn sàng tại: {OUTPUT_DIR.resolve()}")
