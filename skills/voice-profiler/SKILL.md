---
name: voice-profiler
description: Kỹ năng phân tích, trích xuất và điều phối phong cách tác giả thông qua ma trận ghi đè quy tắc cú pháp và từ vựng.
version: 0.1.0
---

# VOICE PROFILER DIRECTIVE

## 1. NẠP VÀ ÁP DỤNG PROFILE MA TRẬN
Khi nhận chỉ định `profile_id`, phải tra cứu các tham số cú pháp trong `voice-profile.schema.json`:
- `clause_chaining_style`: Phong cách nối mệnh đề (*hypotactic_causal*, *topic_comment_native*).
- `allow_long_sentences`: Nếu `true`, **tuyệt đối không chặt nhỏ** các câu ghép nhiều tầng bậc của chuyên gia.
- `colloquial_particles_allowed`: Nếu `false`, loại bỏ toàn bộ từ khẩu ngữ (*chứ, cơ, khổ nỗi, thành thử*).

## 2. MA TRẬN GHI ĐÈ LUẬT (PATTERN OVERRIDES)
Nếu profile chỉ định:
- `enforcement: "disabled"` cho một rule (ví dụ: `NOMINALIZATION_HEAVY`): Cho phép sử dụng danh từ hóa cho các định danh khái niệm kỹ thuật.
- `enforcement: "strict_ban"`: Phạt ở mức tối đa (`critical`).
