# 🇻🇳 Vietnamese Writing Intelligence (v0.2.1)

> **An agentic framework for native Vietnamese writing, Topic–Comment discourse planning, voice profiling, and fact-preserving text refinement.**

[![PyPI version](https://badge.fury.io/py/vietnamese-writing-intelligence.svg)](https://pypi.org/project/vietnamese-writing-intelligence/)
[![Downloads](https://pepy.tech/badge/vietnamese-writing-intelligence)](https://pepy.tech/project/vietnamese-writing-intelligence)
[![CI Status](https://github.com/vantruong-dang/vietnamese-writing-intelligence/actions/workflows/ci.yml/badge.svg)](https://github.com/vantruong-dang/vietnamese-writing-intelligence/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Python Version](https://img.shields.io/badge/Python-3.11%20to%203.14%2B-green.svg)](pyproject.toml)

---

## 📌 Triết lý Kiến trúc: "LLM Brain + Deterministic Sensors"

Dự án không tìm cách biến mọi đặc trưng ngôn ngữ học thành các luật cứng (hard rules) cực đoan. Chúng tôi áp dụng mô hình lai (**Neuro-symbolic Architecture**):
1. **LLM làm bộ não (Brain):** Xử lý ngữ cảnh, lập dàn ý, tư duy cấu trúc **Đề – Thuyết (Topic–Comment)** và sinh bản thảo tự nhiên qua `NativeDrafter`.
2. **NLP & Ngữ pháp chức năng làm cảm biến (Deterministic Sensors):** Đo lường bề mặt, phân tích ranh giới từ, phát hiện sáo ngữ AI, câu khuyết tật Khung đề, bị động nhân tạo và kiểm toán ngữ nghĩa dựa trên 11 Tiên đề Ngôn ngữ học tiếng Việt (GS. Cao Xuân Hạo, học giả Nguyễn Hiến Lê).
3. **JSON Schema & Anchor Guardrails:** Rào chắn dữ liệu (`immutable_anchors`) ngăn chặn tuyệt đối việc làm rơi rớt số liệu, dịch bừa thuật ngữ chuyên ngành hoặc làm sai lệch chuỗi nhân quả kỹ thuật khi biên tập.

---

## 🚀 Tính năng cốt lõi trong v0.2.1

- **Multi-Pass Pipeline Engine (`pipeline.py`):** Kiến trúc 6 tầng (`NormalizationPass` $\rightarrow$ `RewritePass` $\rightarrow$ `SensorAuditPass` $\rightarrow$ `ProfileFilterPass` $\rightarrow$ `SemanticAuditPass` $\rightarrow$ `DecisionStateMachinePass`).
- **20+ Context-Aware Linguistic Rules (`linguistics/`):**
  - Phân tầng danh từ hóa 3 cấp độ (bắt danh từ hóa sáo rỗng `việc thực hiện`, miễn trừ danh từ gốc & chủ ngữ trừu tượng hợp lệ).
  - Bắt câu khuyết tật biến Chủ đề thành Khung đề (*"Trong báo cáo này đòi hỏi..."*) và xung đột giới từ – liên từ (*"Với... nhưng..."*).
  - Phân định thể nhượng bộ Hiện thực (`Tuy/Mặc dù`) vs. Giả định (`Dù`) và cấu trúc khung thời gian đầu câu.
  - Phát hiện trùng ngữ Hán – Việt (*"tối ưu nhất"*, *"hoàn toàn tuyệt đối"*) có danh sách miễn trừ từ ghép chính danh (*"anh trai"*, *"chị gái"*).
- **Native Drafter & Pluggable LLM Providers (`drafting/`):** Lập dàn ý Đề – Thuyết (`DraftPlanner`) và kết nối trực tiếp với `AnthropicProvider`, `OpenAIProvider`, `OllamaProvider`, hoặc `MockLLMProvider` (zero external SDK bloat).
- **Stateless & Token-Efficient MCP Server (`mcp_server.py`):** Tích hợp trực tiếp với **Claude Desktop** và **Cursor** qua giao thức `stdio`, sử dụng cơ chế nén *Compact Stateless Payload* giúp **tiết kiệm ~63.6% lượng token** tiêu thụ.
- **200-Case Linguistic Gold Benchmark:** Bộ dữ liệu kiểm chuẩn định lượng 200 câu chia đều 4 nhóm ($4 \times 50$ câu) đạt **100.00% Precision, 100.00% Recall, 0.00% False Positive Rate**.

---

## 💻 Cài đặt & Sử dụng Nhanh

### 1. Cài đặt từ PyPI (Khuyên dùng)
```bash
# Cài đặt thư viện cốt lõi và công cụ dòng lệnh vn-intel
pip install vietnamese-writing-intelligence

# Cài đặt kèm tính năng MCP Server cho Claude Desktop / Cursor
pip install "vietnamese-writing-intelligence[mcp]"
```

### 2. Sử dụng qua Dòng lệnh (`vn-intel` CLI)

```bash
# 1. Quét nhanh (Lint) một file văn bản
vn-intel lint path/to/document.md

# 2. Kiểm toán chuyên sâu với Voice Profile & Draft Manifest
vn-intel audit path/to/document.md --profile profiles/systems_architect.json --manifest manifest.json

# 3. Phác thảo văn bản bản địa từ DraftManifest (hỗ trợ mock, ollama, anthropic, openai)
vn-intel draft --manifest manifest.json --provider ollama --model qwen2.5:7b --output draft.md

# 4. Chạy báo cáo kiểm chuẩn 200 ca ngôn ngữ học (Gold Standard Benchmark)
vn-intel benchmark
```

### 3. Tích hợp MCP Server cho Claude Desktop / Cursor

Thêm cấu hình sau vào `claude_desktop_config.json` hoặc `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "vietnamese-writing-intelligence": {
      "command": "vn-intel",
      "args": ["mcp"]
    }
  }
}
```
*(Hoặc dùng `uvx --from "vietnamese-writing-intelligence[mcp]" vn-intel mcp` nếu không muốn cài cố định vào môi trường).*

---

## 📊 Bộ Kiểm thử Hồi quy Luật Ngôn ngữ (`vn-intel benchmark`)

> [!NOTE]
> Bộ 200 ca kiểm thử đơn vị (`linguistic_gold_standard.jsonl`) đóng vai trò là **Deterministic Regression Suite** nhằm bảo đảm 20+ luật ngữ pháp chức năng và cảm biến bề mặt vận hành đúng quy cách, không bị hồi quy logic khi refactor code. Dự án đang tiếp tục xây dựng bộ ngữ liệu độc lập gán nhãn thủ công từ thực tế (Real-world Human-Annotated Corpus).

| Hạng mục / Nhóm Kiểm thử Hồi quy | Số lượng ca | Kết quả |
| :--- | :---: | :---: |
| **Nhóm 1:** Danh từ hóa 3 tầng (GS. Cao Xuân Hạo) | `50 / 50` | **100.0% PASS** |
| **Nhóm 2:** Cấu trúc Đề – Thuyết & Cú pháp bản địa (GS. Cao Xuân Hạo) | `50 / 50` | **100.0% PASS** |
| **Nhóm 3:** Hiện thực/Giả định (`Tuy` – `Dù`) & Trùng ngữ (Humanizer) | `50 / 50` | **100.0% PASS** |
| **Nhóm 4:** Bảo toàn Dữ kiện, Code Block & Đối ngẫu Cổ văn (Nguyễn Hiến Lê) | `50 / 50` | **100.0% PASS** |
| **Tổng cộng (Precision / Recall / F1-Score / FPR)** | **`200 / 200`** | **`100% / 100% / 100% / 0%`** |

Chi tiết ánh xạ kiến trúc xem tại: **[`docs/KNOWLEDGE_ARCHITECTURE_MAP.md`](docs/KNOWLEDGE_ARCHITECTURE_MAP.md)**.

---

## 🗺 Lộ trình Phát triển 2 Trục Song hành (Roadmap to v1.0)

```
        Trục A: Năng lực Ngôn ngữ                       Trục B: Năng lực Tạo sinh
 (Deterministic NLP & Linguistic Sensors)           (Agentic Drafting & Voice Profiling)
                   │                                                  │
v0.1               ├─ Schema Validation & Regex ACL Adapter           ├─ Basic Orchestrator & CLI
                   │                                                  │
v0.2 (Current)     ├─ Segmentation, Normalization & 20+ Context Rules ├─ Native Drafter, Providers & MCP Server
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

## 📚 Tham khảo, Nguồn cảm hứng & Ghi công (Acknowledgments)

Dự án **Vietnamese Writing Intelligence** được xây dựng trên sự kế thừa, chắt lọc triết lý và kiến trúc từ các công trình ngôn ngữ học và dự án nguồn mở tiên phong:

### 1. Nền tảng Ngôn ngữ học Tiếng Việt Bản địa
* **GS. Cao Xuân Hạo:** *Tiếng Việt — Sơ thảo ngữ pháp chức năng* & *Tiếng Việt, Văn Việt, Người Việt* (Lý thuyết cấu trúc Đề – Thuyết, phân định Khung đề vs. Chủ đề, thể nhượng bộ hiện thực/giả định, phân tích trùng ngữ Hán – Việt và ngoại lệ danh từ thân tộc).
* **Học giả Nguyễn Hiến Lê:** *Cổ văn Trung Quốc* & *Luyện văn* (Nhịp điệu câu văn biền ngẫu, phép tiểu đối và tiết tấu tiếng Việt).


### 2. Dự án Thượng nguồn Trực tiếp (Upstream & Linguistic Baseline)
* **[longhang2004/vietnamese-humanizer](https://github.com/longhang2004/vietnamese-humanizer)** (PyPI: `vietnamese-writing-skills`) của tác giả Hàng Nhựt Long (@longhang2004) và Lê Ngọc Phương Thư:
  - *Kế thừa & tích hợp:* State Machine 4 chế độ đầu ra (`clean_rewrite`, `no_change`, `review_comment`, `needs_author_decision`), triết lý bảo toàn dữ kiện ngữ nghĩa (Semantic Preservation), và taxonomy phân loại lỗi bề mặt tiếng Việt.

### 3. Chuẩn mực Agent Skills & Khử văn phong AI (Agentic & De-AI Paradigms)
* **[anthropics/skills](https://github.com/anthropics/skills):** Cấu trúc đặc tả Agent Skill chuẩn hóa (`SKILL.md`) với Metadata YAML frontmatter và cơ chế phân tách Mode vận hành.
* **[blader/humanizer](https://github.com/blader/humanizer):** Hệ thống 25+ negative patterns nhận diện dấu vân tay máy tính của RLHF và quy trình tự phê bình (Self-Critique Loop).
* **[danielmiessler/fabric](https://github.com/danielmiessler/fabric):** Triết lý module hóa Unix cho prompt/pattern (`humanize`, `improve_writing`).
* **[JuliusBrussee/caveman](https://github.com/JuliusBrussee/caveman):** Kỹ thuật nén mật độ thông tin cao, cắt bỏ từ đệm và triệt tiêu đại từ nhân xưng thừa thãi.
* **[obra/superpowers](https://github.com/obra/superpowers):** Cơ chế kiểm định ngầm nội bộ (Internal Verification Gate) trước khi trả kết quả.
* **[ComposioHQ/awesome-claude-skills](https://github.com/ComposioHQ/awesome-claude-skills):** Tuyển tập kiến trúc kỹ năng và phương pháp phân lập ngữ vực (Persona Engineering).

### 4. Hệ sinh thái Xử lý Ngôn ngữ Tự nhiên Tiếng Việt (Vietnamese NLP Ecosystem)
* **[Underthesea](https://github.com/undertheseanlp/underthesea)**, **[VnCoreNLP](https://github.com/vncorenlp/VnCoreNLP)**, **[PyVi](https://github.com/trungngv/pyvi)**.

---

## 📄 Bản quyền (License)

Dự án được phân phối dưới giấy phép **[MIT License](LICENSE)**.
