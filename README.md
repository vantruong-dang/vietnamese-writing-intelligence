# 🇻🇳 Vietnamese Writing Intelligence — Orchestration Foundation (v0.1.0)

> **An extensible framework for Vietnamese writing intelligence, currently providing orchestration, structured validation, and an evolving roadmap toward native language understanding and generation.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Python Version](https://img.shields.io/badge/Python-3.11%20to%203.14%2B-green.svg)](pyproject.toml)
[![Architecture](https://img.shields.io/badge/Architecture-LLM%20Brain%20%2B%20NLP%20Sensors-purple.svg)](#-triết-lý-kiến-trúc-llm-brain--deterministic-sensors)

---

## 📌 Triết lý Kiến trúc: "LLM Brain + Deterministic Sensors"

Dự án không tìm cách biến mọi đặc trưng ngôn ngữ học thành các luật cứng (hard rules) cực đoan. Chúng tôi áp dụng mô hình lai (Neuro-symbolic approach):
1. **LLM làm bộ não (Brain):** Xử lý ngữ cảnh, lập dàn ý, tư duy cấu trúc Đề – Thuyết và sinh bản thảo tự nhiên.
2. **NLP truyền thống làm cảm biến (Sensors):** Đo lường bề mặt, phân tích ranh giới từ, phát hiện sáo ngữ và kiểm toán ngữ âm/nhịp điệu mà không can thiệp thô bạo.
3. **JSON Schema & Anchor Guardrails:** Rào chắn dữ liệu (`immutable_anchors`) ngăn chặn việc làm biến dạng số liệu, tên riêng hoặc đảo ngược chuỗi nhân quả kỹ thuật khi biên tập.

---

## 🚀 Tính năng cốt lõi trong v0.1.0

- **State Machine 4 trạng thái:** Phân định rõ ràng giữa `clean_rewrite`, `no_change`, `review_comment`, và `needs_author_decision`.
- **Anti-Corruption Adapter (`upstream_adapter.py`):** Tách biệt và bảo vệ hệ thống khỏi sự thay đổi của upstream linter (`vietnamese-writing-skills`), tự động kích hoạt bộ quy tắc nội tại khi chạy độc lập, đồng thời che chắn code fences, URLs, emails để chống báo lỗi giả.
- **Fail-Fast Schema Validation:** Kiểm định cấu trúc `VoiceProfile` và `DraftManifest` theo chuẩn JSON Schema Draft 2020-12 ngay tại thời điểm khởi tạo.
- **Anchor-based Semantic Integrity Checks:** Đối soát các điểm neo dữ kiện bất biến (`immutable_anchors`) với cơ chế chuẩn hóa khoảng trắng, Unicode tổ hợp (NFC) và ranh giới từ, loại bỏ tình trạng báo lỗi giả và bảo toàn số liệu/thực thể.

---

## 🗺 Lộ trình Phát triển 2 Trục Song hành (Roadmap to v1.0)

```
        Trục A: Năng lực Ngôn ngữ                       Trục B: Năng lực Tạo sinh
 (Deterministic NLP & Linguistic Sensors)           (Agentic Drafting & Voice Profiling)
                   │                                                  │
v0.1 (Current)     ├─ Schema Validation & Regex ACL Adapter           ├─ Basic Orchestrator & CLI
                   │                                                  │
v0.2               ├─ Segmentation, Normalization & Context Rules     ├─ Native Drafter (Workflow Planner)
                   │                                                  │
v0.3               ├─ Register & Dialect Modeling (Bắc/Trung/Nam)     ├─ Voice Profiler & Feature Extraction
                   │                                                  │
v0.4               ├─ Prosody Analyzer (Nhịp điệu & Bằng - Trắc)      ├─ Deep Semantic & Causal Audit
                   │                                                  │
                   └─────────────────────────┬────────────────────────┘
                                             ▼
v1.0                         Integrated Vietnamese Writing Agent
```

---

## 💻 Cài đặt & Sử dụng

### 1. Cài đặt môi trường phát triển (Python 3.11 – 3.14+)
```bash
git clone https://github.com/vantruong-dang/vietnamese-writing-intelligence.git
cd vietnamese-writing-intelligence
python -m pip install -e ".[dev]"
```

### 2. Chạy bộ kiểm thử (Unit Tests)
```bash
pytest -v
```

### 3. Sử dụng qua dòng lệnh (CLI)
```bash
vn-intel audit <file_van_ban.txt> --profile profiles/systems_architect.json --manifest <file_manifest.json>
```

---

## 📚 Tham khảo, Nguồn cảm hứng & Ghi công (Acknowledgments)

Dự án **Vietnamese Writing Intelligence** được xây dựng trên sự kế thừa, chắt lọc triết lý và kiến trúc từ các dự án nguồn mở tiên phong:

### 1. Dự án Thượng nguồn Trực tiếp (Upstream & Linguistic Baseline)
* **[longhang2004/vietnamese-humanizer](https://github.com/longhang2004/vietnamese-humanizer)** (Sản phẩm: *Vietnamese Writing Skills* / PyPI: `vietnamese-writing-skills`) của tác giả Hàng Nhựt Long (@longhang2004) và Lê Ngọc Phương Thư:
  - *Kế thừa & tích hợp:* State Machine 4 chế độ đầu ra (`clean_rewrite`, `no_change`, `review_comment`, `needs_author_decision`), triết lý bảo toàn dữ kiện ngữ nghĩa (Semantic Preservation), và taxonomy phân loại lỗi bề mặt tiếng Việt.

### 2. Chuẩn mực Agent Skills & Khử văn phong AI (Agentic & De-AI Paradigms)
* **[anthropics/skills](https://github.com/anthropics/skills):** Cấu trúc đặc tả Agent Skill chuẩn hóa (`SKILL.md`) với Metadata YAML frontmatter và cơ chế phân tách Mode vận hành.
* **[blader/humanizer](https://github.com/blader/humanizer):** Hệ thống 25+ negative patterns nhận diện dấu vân tay máy tính của RLHF và quy trình tự phê bình (Self-Critique Loop).
* **[danielmiessler/fabric](https://github.com/danielmiessler/fabric):** Triết lý module hóa Unix cho prompt/pattern (`humanize`, `improve_writing`).
* **[JuliusBrussee/caveman](https://github.com/JuliusBrussee/caveman):** Kỹ thuật nén mật độ thông tin cao, cắt bỏ từ đệm và triệt tiêu đại từ nhân xưng thừa thãi.
* **[obra/superpowers](https://github.com/obra/superpowers):** Cơ chế kiểm định ngầm nội bộ (Internal Verification Gate) trước khi trả kết quả.
* **[ComposioHQ/awesome-claude-skills](https://github.com/ComposioHQ/awesome-claude-skills):** Tuyển tập kiến trúc kỹ năng và phương pháp phân lập ngữ vực (Persona Engineering).

### 3. Hệ sinh thái Xử lý Ngôn ngữ Tự nhiên Tiếng Việt (Vietnamese NLP Ecosystem)
Dự án tham chiếu các nghiên cứu và công cụ nền tảng cho Roadmap từ v0.2:
* **[Underthesea](https://github.com/undertheseanlp/underthesea):** Hệ thống NLP tiếng Việt phục vụ phân tích hình thái, tách từ (Word Segmentation) và gán nhãn từ loại (POS Tagging).
* **[VnCoreNLP](https://github.com/vncorenlp/VnCoreNLP):** Bộ công cụ xử lý ngôn ngữ tự nhiên tiếng Việt chuẩn học thuật (tách từ, gán nhãn, phân tích cú pháp phụ thuộc).
* **[PyVi](https://github.com/trungngv/pyvi):** Thư viện Python nhẹ cho tác vụ phân đoạn từ và nhận diện thực thể tiếng Việt.

---

## 📄 Bản quyền (License)

Dự án được phân phối dưới giấy phép **[MIT License](LICENSE)**.
