# VWI Chrome Extension (WebAssembly 100% Offline)

Tiện ích mở rộng Chrome kiểm toán văn phong tiếng Việt chạy 100% Offline trên trình duyệt thông qua WebAssembly (WASM).

## 🌟 Tính năng nổi bật

- **100% Offline & Bảo mật tuyệt đối:** Không gửi bất kỳ ký tự nào của người dùng lên server (Zero-Cloud).
- **Tốc độ cực hạn:** Kiểm toán văn bản trong < 2ms với WebAssembly biên dịch từ Rust.
- **Tích hợp sẵn bộ từ điển & tri thức:** 72.500 từ vựng tiếng Việt, thuật toán tách từ Viterbi Trellis và 25+ luật ngữ pháp chức năng & phong cách tiếng Việt.
- **Hỗ trợ bôi đen trực tiếp:** Tự động bắt văn bản đang bôi đen trên Gmail, Google Docs, Facebook, Notion, v.v.

## 🛠️ Hướng dẫn cài đặt thử nghiệm (Local Development)

### 1. Biên dịch WebAssembly
Nếu bạn chưa có thư mục `wasm/`, chạy lệnh sau từ thư mục gốc của repo:

```bash
wasm-pack build crates/vwi-wasm --target web --out-dir ../../extensions/chrome/wasm --release
```

### 2. Nạp tiện ích vào Chrome
1. Mở trình duyệt Google Chrome hoặc Microsoft Edge, truy cập:
   ```text
   chrome://extensions/
   ```
2. Bật công tắc **Developer mode** (Chế độ dành cho nhà phát triển) ở góc trên bên phải.
3. Bấm nút **Load unpacked** (Tải tiện ích đã giải nén).
4. Chọn thư mục `extensions/chrome`.
5. Biểu tượng **VWI** sẽ xuất hiện trên thanh công cụ của trình duyệt!

## 📦 Đóng gói phát hành Chrome Web Store

Để tạo tệp `.zip` tải lên Chrome Web Store Developer Dashboard:

```bash
python scripts/package_chrome_extension.py
```

Tệp zip sẽ được tạo tại `dist/vwi-chrome-extension-v0.3.1.zip` với dung lượng chỉ ~1.1 MB.
