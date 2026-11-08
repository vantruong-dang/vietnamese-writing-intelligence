---
name: native-drafter
description: Kỹ năng tạo sinh văn bản tiếng Việt từ số 0, tư duy trực tiếp theo cấu trúc Đề – Thuyết và kiểm soát chặt chẽ điểm neo dữ kiện.
version: 0.1.0
---

# NATIVE DRAFTER WORKFLOW DIRECTIVE

## BƯỚC 1: XÁC LẬP HỢP ĐỒNG DỮ KIỆN (PRE-DRAFT MANIFEST)
Trước khi sinh đoạn văn, phải phân lập rõ các yếu tố bất biến:
- `metrics`: Các con số, mốc thời gian, đơn vị đo lường (giữ nguyên 100%).
- `domain_terms`: Thuật ngữ chuyên môn không được tự ý dịch thô (VD: *type erasure, call margin*).
- `causal_chains`: Mối quan hệ nguyên nhân – kết quả cốt lõi.

## BƯỚC 2: TƯ DUY CẤU TRÚC ĐỀ – THUYẾT (TOPIC – COMMENT)
- Thay vì bắt đầu mọi câu bằng cấu trúc Chủ ngữ – Động từ cứng nhắc, đưa phần "Đề" (chủ đề cần nói) lên trước, phần "Thuyết" (nhận định, hành động) theo sau:
  - *Chủ – Vị công thức:* "Hệ thống bộ nhớ gặp sự cố phân mảnh khi dữ liệu tăng."
  - *Đề – Thuyết bản địa:* "Dữ liệu càng phình to, bộ nhớ càng dễ phân mảnh."

## BƯỚC 3: XUẤT BẢN THẢO VÀ TỰ ĐỐI SOÁT
Đối chiếu bản thảo với `DraftManifest`. Nếu thiếu một thông số kỹ thuật, tự động bổ sung trước khi kết xuất.
