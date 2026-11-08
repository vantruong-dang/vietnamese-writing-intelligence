---
name: humanizer-vi
description: Kỹ năng chuyên sâu nhận diện và triệt tiêu dấu vân tay học máy (AI Footprint), sáo ngữ RLHF, và nhịp điệu phẳng lì trong văn bản tiếng Việt.
version: 0.1.0
---

# HUMANIZER-VI DIRECTIVE

## 1. VÙNG CẤM SÁO NGỮ TUYỆT ĐỐI (NEGATIVE FOOTPRINT BLACKLIST)
Tuyệt đối KHÔNG xuất hiện các mẫu câu công thức sau:
- Mở đầu giáo điều: *"Không thể phủ nhận rằng"*, *"Có thể nói rằng"*, *"Trong bối cảnh kỷ nguyên số hiện nay"*, *"Nhìn chung"*.
- Ẩn dụ sách vở rỗng: *"Bức tranh toàn cảnh"*, *"Chiếc chìa khóa vạn năng"*, *"Con dao hai lưỡi"*, *"Ngọn hải đăng"*, *"Hành trình khám phá"*.
- Tính từ phóng đại rỗng: *"Đóng vai trò vô cùng quan trọng"*, *"Mang tính đột phá"*, *"Cực kỳ then chốt"*.
- Kết bài đạo đức trung dung: *"Tóm lại, dù có khó khăn nhưng nếu cố gắng chúng ta sẽ bước vào tương lai tươi sáng..."*.

## 2. TRIỆT TIÊU BỆNH LIỆT KÊ BỘ BA (TRIADIC LIST BIAS)
AI thường có thói quen chia mọi luận điểm thành 3 vế cân đối chằn chặn: *"A, B và C"*.
- Phá vỡ: Gộp 2 ý chính hoặc tách thành các câu độc lập có độ dài lệch nhau.

## 3. TẠO ĐỘ LỆCH ĐỘ DÀI CÂU (BURSTINESS INJECTION)
- Tránh viết 3 câu liên tiếp cùng có độ dài 20–25 từ.
- Luôn đan xen một câu dứt điểm cực ngắn (3–6 từ) sau một chuỗi lập luận phức tạp.
