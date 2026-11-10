# Chính Sách Bảo Mật (Privacy Policy) - VWI

**Cập nhật lần cuối:** 10/11/2026  
**Dự án:** Vietnamese Writing Intelligence (VWI)  
**Tác giả:** Đặng Văn Trường & Cộng đồng Mã nguồn mở  
**Mã nguồn:** [https://github.com/vantruong-dang/vietnamese-writing-intelligence](https://github.com/vantruong-dang/vietnamese-writing-intelligence)

---

### 1. Cam Kết Cốt Lõi: 100% Ngoại Tuyến (Offline-First)

Vietnamese Writing Intelligence (VWI) được thiết kế và xây dựng theo triết lý **Bảo mật Tối đa theo Thiết kế (Privacy by Design)**:

* **Không Thu Thập Dữ Liệu:** Chúng tôi **KHÔNG** thu thập, không ghi lại, không lưu trữ và không truyền tải bất kỳ dữ liệu văn bản, nội dung bài viết, phím gõ hay thông tin nhận dạng cá nhân nào của bạn ra máy chủ bên ngoài hoặc bên thứ ba.
* **Thực thi 100% Cục bộ:** Toàn bộ thuật toán phân tích ngữ pháp, đối soát văn phong, kiểm tra chính tả và âm tiết học được biên dịch sang mã máy **WebAssembly (Rust WASM)** và chạy trực tiếp 100% trong môi trường trình duyệt của bạn.
* **Không Có Máy Chủ Thu Thập (Zero Backend Tracking):** VWI không duy trì bất kỳ máy chủ theo dõi, cơ sở dữ liệu từ xa, API phân tích hành vi hay hệ thống thu thập thông tin người dùng nào.
* **Không Mã Từ Xa (Zero Remote Code):** Toàn bộ mã nguồn chạy độc lập từ các tệp cục bộ bên trong tiện ích. Không tải bất kỳ thư viện, script hay CDN nào từ internet.

---

### 2. Dữ Liệu Lưu Trữ Cục Bộ (Local Storage)

Tiện ích chỉ sử dụng các API lưu trữ cục bộ tiêu chuẩn của trình duyệt (`localStorage`, `chrome.storage.local`, `chrome.storage.session`) cho các mục đích kỹ thuật phục vụ trải nghiệm người dùng:

1. **Bản nháp tạm thời (`vwi_current_draft`):** Lưu đoạn văn bản bạn đang nhập trên giao diện để tránh mất nội dung khi bạn vô tình đóng Popup hoặc chuyển đổi qua lại giữa Popup và Side Panel.
2. **Đoạn trích xuất từ chuột phải (`vwi_selected_text`):** Lưu tạm thời đoạn văn bản bạn vừa bôi đen trên trang web để chuyển vào bảng điều khiển Side Panel phân tích.
3. **Tùy chọn hiển thị (`vwi_display_mode`):** Lưu trạng thái bạn chọn xem tên luật dạng *Dễ hiểu (Tiếng Việt)* hay dạng *Mã kỹ thuật (Rule ID)*.

> [!NOTE]  
> Tất cả các dữ liệu trên chỉ nằm duy nhất trên thiết bị cá nhân của bạn và không bao giờ được gửi đi bất kỳ đâu. Bạn có thể xóa toàn bộ dữ liệu này bất kỳ lúc nào bằng cách bấm nút **🧹 Xóa** trên giao diện hoặc xóa dữ liệu duyệt web của trình duyệt.

---

### 3. Giải Trình Về Các Quyền Truy Cập (Permissions Justification)

VWI tuân thủ nghiêm ngặt nguyên tắc **Đặc quyền Tối thiểu (Principle of Least Privilege)** của Google Chrome Web Store:

| Quyền (Permission) | Lý do sử dụng cụ thể |
| :--- | :--- |
| `activeTab` | Cho phép tiện ích tạm thời trích xuất đoạn văn bản mà bạn chủ động bôi đen trên tab đang mở để đưa vào bộ kiểm tra. |
| `scripting` | Cho phép trích xuất nội dung văn bản bôi đen giữ nguyên định dạng ngắt dòng (`\n`) khi bạn nhấp chuột phải. |
| `contextMenus` | Cung cấp mục menu chuột phải *"🔍 Kiểm tra văn phong với VWI"* khi bạn bôi đen một đoạn văn bản. |
| `sidePanel` | Hiển thị bảng kết quả phân tích bên hông trình duyệt giúp bạn vừa đọc tài liệu vừa đối soát văn phong thuận tiện. |
| `storage` | Lưu trữ bản nháp tạm thời và tùy chọn hiển thị giao diện cục bộ trên máy tính. |

Tiện ích **KHÔNG** yêu cầu quyền đọc lịch sử duyệt web (`history`), không yêu cầu quyền truy cập tất cả trang web (`<all_urls>`), không đọc cookies, và không có quyền sửa đổi cài đặt mạng của bạn.

---

### 4. Tuân Thủ Chính Sách Người Dùng Google (Limited Use Policy)

VWI tuân thủ đầy đủ các yêu cầu của **Chính sách Dữ liệu Người dùng Chrome Web Store** (Chrome Web Store User Data Policy), bao gồm các nguyên tắc *Sử dụng có Giới hạn (Limited Use)*.

---

### 5. Liên Hệ & Đóng Góp

Nếu bạn có bất kỳ câu hỏi, thắc mắc hoặc phát hiện bất kỳ vấn đề bảo mật nào, vui lòng liên hệ hoặc mở Issue công khai tại kho mã nguồn của chúng tôi:

* **GitHub Repository:** [https://github.com/vantruong-dang/vietnamese-writing-intelligence](https://github.com/vantruong-dang/vietnamese-writing-intelligence)
* **Kênh Tiếp Nhận Góp Ý:** [https://github.com/vantruong-dang/vietnamese-writing-intelligence/issues](https://github.com/vantruong-dang/vietnamese-writing-intelligence/issues)
