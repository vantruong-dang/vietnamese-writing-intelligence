---
name: voice-profiler
description: Kỹ năng trích xuất và áp dụng hồ sơ văn phong (Voice Profile) có kiểm soát, điều phối ma trận ghi đè quy tắc ngôn ngữ.
version: 0.1.0
---

# VOICE PROFILER DIRECTIVE

Khi mô phỏng hoặc canh chỉnh giọng văn:
1. **Nạp Voice Profile:** Kiểm tra `clause_chaining_style`, `sino_vietnamese_density` và `pattern_overrides`.
2. **Tuân thủ ngân sách cú pháp:** Nếu profile cho phép câu ghép tầng bậc (`allow_long_sentences: true`), không được tự ý chặt nhỏ câu văn kỹ thuật.
3. **Điều chỉnh hư từ:** Chỉ sử dụng tình thái từ khẩu ngữ khi `colloquial_particles_allowed: true`.
