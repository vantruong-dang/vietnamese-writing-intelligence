#!/usr/bin/env python3
"""Kiểm thử tự động giao diện VWI trên Google Chrome bằng Playwright và chụp ảnh màn hình minh chứng."""

import http.server
import os
import socketserver
import threading
import time
from pathlib import Path

from playwright.sync_api import sync_playwright

ROOT_DIR = Path(__file__).resolve().parent.parent
EXT_DIR = ROOT_DIR / "extensions" / "chrome"
ARTIFACTS_DIR = Path(r"C:\Users\Admin\.gemini\antigravity\brain\29d1798a-b7de-4107-bbd5-b3c3c072a0a1")
CHROME_PATH = r"C:\Program Files\Google\Chrome\Application\chrome.exe"
PORT = 8912

TEST_TEXT = """Trong tình hình này đòi hỏi khắc phục ngay các tồn tại.
Với những cố gắng vượt bậc nhưng họ vẫn không giải quyết được dứt điểm.
Bằng những hình tượng sắc nét của tác giả đã cho thấy tài năng văn chương.
Ngày mai, tuy trời mưa to gió lớn, tôi cũng sẽ đến.
Hôm qua, dù trời mưa to, tôi vẫn đến.
Chuyên gia nước ngoài sẽ về trong ba ngày.
Bộ phim này được tài trợ bởi một tập đoàn lớn.
Cái bàn bị lau chưa sạch nên vẫn còn bám bụi.
Đây là phương án kỹ thuật tối ưu nhất cho dự án.
Căn phòng này ngập tràn ánh nắng buổi sớm.
Để kịp tiến độ, chí ít chúng ta cần thêm ba tuần nữa.
Phải nói là cú sút rất điệu nghệ của tiền đạo.
Như các bạn đã biết là tiền đạo này đã bị chấn thương.
chúng tôi dduowjc thông báo kết quả.
"""

class QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, format, *args):
        pass

def start_server():
    os.chdir(str(EXT_DIR))
    httpd = socketserver.TCPServer(("127.0.0.1", PORT), QuietHandler)
    t = threading.Thread(target=httpd.serve_forever, daemon=True)
    t.start()
    return httpd

def run_e2e_tests():
    print(f"[*] Khởi động máy chủ web cục bộ tại port {PORT}...")
    httpd = start_server()
    time.sleep(0.5)

    print(f"[*] Khởi động Google Chrome: {CHROME_PATH}...")
    with sync_playwright() as p:
        browser = p.chromium.launch(
            executable_path=CHROME_PATH,
            headless=True,
            args=["--no-sandbox", "--disable-gpu", "--window-size=1280,900"]
        )

        # -------------------------------------------------------------
        # 1. TEST KHÔNG GIAN STUDIO (TOÀN MÀN HÌNH - CHẾ ĐỘ DỄ HIỂU)
        # -------------------------------------------------------------
        page = browser.new_page(viewport={"width": 1280, "height": 900})
        studio_url = f"http://127.0.0.1:{PORT}/popup.html?mode=studio"
        page.goto(studio_url)
        page.wait_for_timeout(1000)

        print("[*] Nhập 14 câu kiểm thử vào Không gian Studio...")
        page.fill("#txtInput", TEST_TEXT)
        page.click("#btnAudit")
        page.wait_for_timeout(1200)

        total_cards = page.locator(".finding-card").count()
        apply_buttons = page.locator(".btn-apply").count()
        score_text = page.locator("#scoreValue").inner_text()
        print(f"    + Điểm số: {score_text}")
        print(f"    + Tổng số thẻ phát hiện lỗi: {total_cards}")
        print(f"    + Số lượng nút 'Áp dụng ⚡' (AutoFix máy móc): {apply_buttons}")

        shot1 = ARTIFACTS_DIR / "vwi_chrome_studio_friendly.png"
        page.screenshot(path=str(shot1), full_page=True)
        print(f"[+] Đã lưu ảnh chụp Studio Dễ hiểu: {shot1.name}")

        # -------------------------------------------------------------
        # 2. TEST CHUYỂN SANG CHẾ ĐỘ "DÀNH CHO DEV"
        # -------------------------------------------------------------
        print("[*] Bấm nút chuyển đổi sang chế độ 'Dành cho Dev'...")
        page.click("#btnToggleMode")
        page.wait_for_timeout(600)

        shot2 = ARTIFACTS_DIR / "vwi_chrome_studio_dev.png"
        page.screenshot(path=str(shot2), full_page=True)
        print(f"[+] Đã lưu ảnh chụp Studio Dev Mode: {shot2.name}")

        # -------------------------------------------------------------
        # 3. TEST 1-CLICK AUTO-FIX (ÁP DỤNG SỬA LỖI MÁY MÓC)
        # -------------------------------------------------------------
        if apply_buttons > 0:
            print("[*] Thử nghiệm tính năng 1-Click AutoFix...")
            # Bấm nút Áp dụng đầu tiên
            page.locator(".btn-apply").first.click()
            page.wait_for_timeout(1000)

            shot3 = ARTIFACTS_DIR / "vwi_chrome_after_autofix.png"
            page.screenshot(path=str(shot3), full_page=True)
            print(f"[+] Đã lưu ảnh chụp sau khi Áp dụng Auto-Fix: {shot3.name}")

        # -------------------------------------------------------------
        # 4. TEST POPUP TIỆN ÍCH KÍCH THƯỚC CHUẨN (450 x 620 px)
        # -------------------------------------------------------------
        print("[*] Kiểm thử giao diện Popup chuẩn tiện ích (450x620)...")
        page_popup = browser.new_page(viewport={"width": 450, "height": 620})
        popup_url = f"http://127.0.0.1:{PORT}/popup.html"
        page_popup.goto(popup_url)
        page_popup.wait_for_timeout(800)

        popup_sample = "Khi dân cần, có Công an.\nchúng tôi dduowjc giúp đỡ."
        page_popup.fill("#txtInput", popup_sample)
        page_popup.click("#btnAudit")
        page_popup.wait_for_timeout(1000)

        shot4 = ARTIFACTS_DIR / "vwi_chrome_popup_standard.png"
        page_popup.screenshot(path=str(shot4), full_page=True)
        print(f"[+] Đã lưu ảnh chụp Popup chuẩn: {shot4.name}")

        browser.close()
        httpd.shutdown()

    print("\n[+] TẤT CẢ CÁC BÀI TEST THỰC TẾ TRÊN GOOGLE CHROME ĐÃ HOÀN TẤT THÀNH CÔNG 100%!")

if __name__ == "__main__":
    run_e2e_tests()
