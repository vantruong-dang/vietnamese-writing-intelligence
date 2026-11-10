# 🇻🇳 Vietnamese Writing Intelligence (v0.3.0)

> **High-performance Vietnamese NLP Engine & Agentic Writing Framework — Powered by Pure Rust Muscle & Functional Grammar.**

[![Crates.io](https://img.shields.io/crates/v/vwi-cli.svg)](https://crates.io/crates/vwi-cli)
[![PyPI version](https://badge.fury.io/py/vietnamese-writing-intelligence.svg)](https://pypi.org/project/vietnamese-writing-intelligence/)
[![CI Status](https://github.com/vantruong-dang/vietnamese-writing-intelligence/actions/workflows/ci.yml/badge.svg)](https://github.com/vantruong-dang/vietnamese-writing-intelligence/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust: 1.80+](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](Cargo.toml)
[![Python Version](https://img.shields.io/badge/Python-3.11%20to%203.14%2B-green.svg)](pyproject.toml)

---

## ⚡ Blazing Fast Standalone CLI (`vwi` — Zero Dependencies)

Bạn muốn kiểm toán văn phong tiếng Việt trong CI/CD, Git Hook hoặc Server mà **không muốn cài đặt Python hay bất kỳ runtime phụ thuộc nào**? Hãy tải binary thực thi độc lập duy nhất (`vwi`), nặng **chỉ ~5MB**:

```bash
# 🐧 Linux (x86_64)
curl -L https://github.com/vantruong-dang/vietnamese-writing-intelligence/releases/latest/download/vwi-linux-amd64 -o vwi && chmod +x vwi
sudo mv vwi /usr/local/bin/

# 🍎 macOS (Apple Silicon M1/M2/M3/M4)
curl -L https://github.com/vantruong-dang/vietnamese-writing-intelligence/releases/latest/download/vwi-macos-arm64 -o vwi && chmod +x vwi
sudo mv vwi /usr/local/bin/

# 🪟 Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/vantruong-dang/vietnamese-writing-intelligence/releases/latest/download/vwi-windows-amd64.exe" -OutFile "vwi.exe"

# 🦀 Rust Developers (qua Cargo)
cargo install vwi-cli

# 🐍 Python Developers (qua Pip)
pip install vietnamese-writing-intelligence
```

### Sử dụng nhanh CLI `vwi`:

```bash
# 1. Kiểm toán văn bản trực tiếp từ dòng lệnh (Terminal ANSI highlight)
vwi check -t "Trong bối cảnh hiện nay đòi hỏi giải pháp tối ưu nhất được phê duyệt bởi hội đồng."

# 2. Xuất báo cáo Markdown cho Pull Request hoặc xuất JSON cho Dashboard
vwi check -i bai_viet.md -f markdown
vwi check -i bai_viet.md -f json

# 3. Sử dụng làm Quality Gate trong CI/CD (Thoát mã exit code 1 nếu điểm dưới 80)
vwi check -i README.md --fail-under 80

# 4. Tách từ tiếng Việt bằng thuật toán Viterbi Trellis (72.5k từ vựng nhúng phẳng)
vwi tokenize "Mẹ mua một chiếc bàn là mới để là quần áo cho học sinh học bài."
# Kết quả: [Mẹ] [mua] [một] [chiếc] [bàn là] [mới] [để] [là] [quần áo] [cho] [học sinh] [học] [bài] [.]
```

---

## 🏛 Kiến trúc Ba Trụ Cột (Tri-Platform Distribution)

```
                     ┌─────────────────────────────────────────────────────────┐
                     │              TRI-PLATFORM DISTRIBUTION                  │
                     └────────────────────────────┬────────────────────────────┘
                                                  │
                 ┌────────────────────────────────┼────────────────────────────────┐
                 │                                │                                │
                 ▼                                ▼                                ▼
       crates/vwi-core (Rust)            crates/vwi-py (PyO3)             crates/vwi-cli (Binary)
   ┌───────────────────────────┐    ┌───────────────────────────┐    ┌───────────────────────────┐
   │ • Viterbi Trellis Engine  │    │ • Python C-Extension      │    │ • Standalone Native CLI   │
   │ • Embedded 72.5k Lexicon  │    │ • Release Python GIL      │    │ • ANSI Terminal / JSON / MD│
   │ • 25+ Functional Rules    │    │ • Zero-overhead bridge    │    │ • CI/CD Quality Gate      │
   │ • AuditReport Scoring     │    │ • Works with FastAPI/MCP  │    │ • Zero Dependencies (<6MB)│
   └───────────────────────────┘    └───────────────────────────┘    └───────────────────────────┘
```

1. **`vwi-core` (Rust Thuần):** Thư viện lõi độc lập 100%, không cấp phát heap thừa, nhúng sẵn từ điển chuẩn 72.564 từ vào cây Double-Array Trie (`daachorse`) và ma trận xác suất chuyển tiếp Bigram.
2. **`vwi-py` (Python Extension):** Cầu nối PyO3 hiện đại được đóng gói qua `maturin` (`abi3-py311`), giải phóng hoàn toàn Python GIL thông qua `py.allow_threads` giúp xử lý đồng thời đa luồng trong web server và agent.
3. **`vwi-cli` (Standalone Executable):** Ứng dụng dòng lệnh độc lập phân phối qua GitHub Releases và `crates.io`, khởi động và trả kết quả trong **< 150ms**.

---

## 🚀 Đo kiểm Hiệu năng (Performance Benchmarks)

Kết quả đo kiểm trên máy tính tiêu chuẩn với 10.000 lượt lặp:

| Thành phần kiểm thử | Công nghệ / Thuật toán | Tốc độ xử lý | Tăng tốc |
| :--- | :--- | :---: | :---: |
| **Tách từ (Tokenization)** | Python Regex (64 từ ghép thủ công) | $15,029\text{ ops/s}$ | $1.00\times$ *(Baseline)* |
| | **Rust Daachorse** (72,564 từ nhúng, Greedy) | **$43,920\text{ ops/s}$** | **$2.92\times$** *(Nhanh gấp 3 dù từ điển lớn gấp 1,133 lần)* |
| | **Rust Viterbi Trellis** (72,564 từ + Bigram LM) | **$17,335\text{ ops/s}$** | **$1.15\times$** *(Giải quyết triệt để mọi ca nhập nhằng)* |
| **Kiểm toán Ngôn ngữ (25 Rules)** | Python thuần (Vòng lặp 25 Class Rules) | $4,774\text{ ops/s}$ | $1.00\times$ *(Baseline)* |
| | **Rust Auditor (Native C-ABI đơn luồng)** | $5,578\text{ ops/s}$ | **$1.17\times$** |
| | **Rust Auditor Batch (Rayon đa nhân CPU)** | **$21,239\text{ ops/s}$** | **$4.45\times$** |

---

## 📌 Triết lý Kiến trúc: "LLM Brain + Deterministic Sensors"

Dự án không tìm cách biến mọi đặc trưng ngôn ngữ học thành các luật cứng (hard rules) cực đoan. Chúng tôi áp dụng mô hình lai (**Neuro-symbolic Architecture**):
1. **LLM làm bộ não (Brain):** Xử lý ngữ cảnh, lập dàn ý, tư duy cấu trúc **Đề – Thuyết (Topic–Comment)** và sinh bản thảo tự nhiên qua `NativeDrafter`.
2. **NLP & Ngữ pháp chức năng làm cảm biến (Deterministic Sensors):** Đo lường bề mặt, phân tích ranh giới từ, phát hiện sáo ngữ AI, câu khuyết tật Khung đề, bị động nhân tạo và kiểm toán ngữ nghĩa dựa trên các công trình kinh điển của **GS. Cao Xuân Hạo** và học giả **Nguyễn Hiến Lê**.
3. **JSON Schema & Anchor Guardrails:** Rào chắn dữ liệu (`immutable_anchors`) ngăn chặn tuyệt đối việc làm rơi rớt số liệu, dịch bừa thuật ngữ chuyên ngành hoặc làm sai lệch chuỗi nhân quả kỹ thuật khi biên tập.

---

## 🔍 Hệ thống 25+ Luật Ngôn ngữ học Bản địa

Hệ thống cảm biến được phân chia thành 9 nhóm chuyên biệt:

1. **Nhóm Cấu trúc Đề - Thuyết (Topic - Comment):** Bắt câu khuyết tật biến Khung đề thành Chủ ngữ (*"Trong báo cáo này đòi hỏi..."*) và xung đột giới từ - liên từ (*"Với... nhưng..."*).
2. **Nhóm Bị động Nhân tạo & Giả thụ hưởng:** Phát hiện cấu trúc sao phỏng Anh ngữ *"được/bị ... bởi"* (Passive `be + V3 + by`) và lạm dụng từ *"nhận được"* cho hành động chủ động.
3. **Nhóm Dịch ngữ (Translationese) & Chủ ngữ giả:** Bắt chủ ngữ giả Tây phương (*"Nó là rất quan trọng để..."*), tính từ hóa nhân tạo (*"mang tính chất"*), và đảo ngữ trạng từ thời gian không có dấu phẩy ngắt nhịp.
4. **Nhóm Danh từ hóa Nặng nề (Nominalization):** Phân tầng danh từ hóa 3 cấp độ (bắt danh từ hóa sáo rỗng *"việc thực hiện"*, *"quá trình tối ưu hóa"*, nhưng miễn trừ danh từ gốc & khái niệm trừu tượng hợp lệ).
5. **Nhóm Liên từ Nhượng bộ & Xung đột kép:** Phân định thể nhượng bộ Hiện thực (*"Tuy/Mặc dù"*) vs. Giả định (*"Dù"*), và bắt xung đột nhượng bộ kép (*"Mặc dù... tuy nhiên..."*).
6. **Nhóm Trùng ngữ Logic (Pleonasm):** Bắt lỗi lặp thừa ý tố biểu đạt Hán - Việt (*"tối ưu nhất"*, *"nguyên nhân là bởi vì"*, *"ánh nắng mặt trời"*).
7. **Nhóm Sáo ngữ Hành chính (Bureaucracy):** Động cơ Aho-Corasick đa mẫu bắt đệm động từ rỗng (*"tiến hành thực hiện"*, *"triển khai công tác"*), chêm liên từ thừa (*"và đồng thời"*, *"cũng như là"*), và trợ từ cực cấp sáo rỗng (*"vô cùng hết sức"*).
8. **Nhóm Sáo ngữ AI (Humanizer):** Bắt mở đầu rập khuôn kiểu RLHF (*"Không thể phủ nhận rằng"*, *"Có thể nói rằng"*), đuôi tóm tắt máy móc (*"Tóm lại"*, *"Nhìn chung"*), và ẩn dụ công thức (*"chiếc chìa khóa vạn năng"*).
9. **Nhóm Cú pháp Trần thuật & Thời thể:** Bắt câu đảo ngữ trần thuật dịch thô (*"nói anh ấy"*), lạm dụng cặp liên từ đẳng lập, và ghép thời thể bất hợp thức (*"đã và đang"* trước động từ tức thời).

---

## 🐍 Cài đặt & Sử dụng Python

```bash
# Cài đặt thư viện cốt lõi và công cụ CLI vn-intel
pip install vietnamese-writing-intelligence

# Cài đặt kèm tính năng MCP Server cho Claude Desktop / Cursor
pip install "vietnamese-writing-intelligence[mcp]"
```

### Sử dụng Python API:

```python
import vn_writing_intel as vwi

# 1. Tách từ Viterbi Trellis từ Rust lõi
segmenter = vwi.RustViterbiSegmenter()
tokens = segmenter.tokenize("Mẹ mua một chiếc bàn là mới.")
print([(t.text, t.is_compound, t.pos) for t in tokens])

# 2. Kiểm toán ngôn ngữ học với Auditor cấu hình cao
auditor = vwi.Auditor()
findings = auditor.audit("Trong bối cảnh hiện nay đòi hỏi giải pháp tối ưu nhất.")
for f in findings:
    print(f"[{f.severity}] {f.pattern_id}: {f.matched_span} -> {f.rationale}")

# 3. Phân tích song song tài liệu lớn qua Rayon
batch_results = auditor.audit_batch(["Đoạn 1...", "Đoạn 2...", "Đoạn 3..."])
```

### Tích hợp MCP Server cho Claude Desktop / Cursor:

Thêm cấu hình vào `claude_desktop_config.json` hoặc `.cursor/mcp.json`:

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

---

## 📊 Kết quả Kiểm chuẩn Thực tế (`vn-intel benchmark`)

Hệ thống duy trì hai bộ kiểm chuẩn độc lập:

1. **Bộ Kiểm thử Hồi quy Tất định (Deterministic Regression Suite — 200 ca):** Đạt **100.00% Precision, 100.00% Recall, 0.00% FPR**.
2. **Bộ Kiểm chuẩn Thực tế Độc lập (Real-World Gold Benchmark — 300 ca):** Đánh giá trên 150 câu lỗi thật từ LLM/bản dịch máy thô và 150 câu chuẩn mực từ báo chí (*VnExpress, Tuổi Trẻ, Thanh Niên*) và tác phẩm của GS. Cao Xuân Hạo, học giả Nguyễn Hiến Lê:

```text
================================================================================
      REAL-WORLD BENCHMARK REPORT (SPAN-LEVEL & ANCHOR EVALUATION)
================================================================================
Tổng số mẫu thực tế đánh giá     : 300
  • Mẫu lỗi thật (MT / Raw LLM)  : 150
  • Mẫu chuẩn mực (Báo chí/Cổ văn): 150
--------------------------------------------------------------------------------
Span True Positives (Bắt trúng)  : 150
Span False Positives (Bắt sai/lệch): 0
Span False Negatives (Bỏ lọt)    : 0
================================================================================
🎯 KẾT QUẢ ĐO LƯỜNG ĐỊNH LƯỢNG:
   Span Precision (Độ chính xác) : 100.00%
   Span Recall (Độ bao phủ)      : 100.00%
   Span F1-Score                 : 100.00%
   Human Text False Alarm (FPR)  : 0.00% (Mục tiêu: < 5%)
   Anchor Retention (Bảo toàn)   : 100.00% (Mục tiêu: 100%)
================================================================================
```

---

## 📌 Scope & Current Limitations (Phạm vi & Giới hạn hiện tại)

Để đảm bảo tính minh bạch về mặt kỹ thuật, người dùng cần lưu ý các giới hạn hiện tại của phiên bản `v0.3.0`:

### Những gì VWI làm RẤT TỐT:
- **Tách từ xác suất tốc độ cao:** Thuật toán Viterbi Trellis kết hợp `daachorse` (72.500+ từ vựng) giải quyết tốt các ca nhập nhằng phổ biến trong văn chính luận, báo chí và công việc (vd: *"chiếc bàn là"* vs *"cái bàn là..."*).
- **Kiểm toán văn phong & Chống văn dịch (Translationese):** Bắt chính xác câu bị động nhân tạo sao phỏng tiếng Anh (*"được thực hiện bởi"*), chủ ngữ giả (*"nó là quan trọng để"*), và sáo ngữ hành chính/AI rập khuôn.
- **Phân tích Khung Đề - Thuyết (Functional Topic-Comment):** Nhận diện lỗi ngữ pháp chức năng phổ biến nhất: Giới ngữ đứng đầu câu bị ép làm chủ thể cho vị từ nòng cốt (*"Trong bối cảnh... đòi hỏi"*).
- **Hệ thống nhúng & CI/CD:** File binary độc lập ~5MB, độ trễ sub-millisecond, hoàn hảo để làm Quality Gate trong Git Hook hoặc GitHub Actions.

### Những gì VWI CHƯA LÀM (Nằm ngoài phạm vi v0.3.0):
- **Không phải là Deep Learning Transformer:** VWI không thay thế PhoBERT hay LLM trong các tác vụ hiểu ngữ nghĩa sâu toàn văn (Deep Semantic Understanding), tóm tắt văn bản, hay hỏi đáp (Q&A).
- **Giới hạn của POS Tagger:** Nhãn từ loại hiện tại sử dụng mô hình Bigram HMM + tra cứu phân phối Universal Dependencies kết hợp heuristic hình thái học. Nó tối ưu cho tốc độ và bộ nhớ, nhưng không thể đạt độ chính xác tương đương các mô hình Neural CRF trên các văn bản văn học cổ, thơ ca, hoặc biệt ngữ quá phức tạp.
- **Chưa xử lý Teencode / Tiếng lóng mạng xã hội:** Từ điển hiện tại tập trung vào tiếng Việt chuẩn mực (báo chí, hành chính, văn học, khoa học). Văn bản chứa lỗi chính tả nặng, teencode hoặc từ ngữ Gen-Z viết tắt sẽ được xử lý bằng cơ chế OOV Fallback (tách thành các âm tiết đơn) chứ chưa có bộ chuẩn hóa từ lóng tự động.
- **Shallow Parser chứ không phải Full Dependency Treebank:** Hệ thống chỉ phân tích các thành phần cú pháp phục vụ việc kiểm toán văn phong (Khung đề, Vị từ, Liên từ), chưa xây dựng toàn bộ cây phụ thuộc cú pháp (Full Syntactic Dependency Tree) cho toàn bộ câu.

---

## 🗺️ Development Roadmap

Chúng tôi phát triển VWI theo hướng thực dụng và từng bước vững chắc:

- [x] **v0.1.x - v0.2.x:** Bản thử nghiệm nguyên lý trên Python (Rule-based Regex).
- [x] **v0.3.0 (Hiện tại):** 
  - [x] Viết lại lõi bằng Pure Rust (`vwi-core`).
  - [x] Tách từ Viterbi Trellis + 72.5k từ vựng nhúng tĩnh.
  - [x] Bổ sung POS Tagging HMM theo chuẩn Universal Dependencies.
  - [x] Shallow Parser Đề - Thuyết theo GS. Cao Xuân Hạo.
  - [x] CLI nhị phân độc lập (`vwi-cli`) với bộ tính điểm chất lượng 0-100.
  - [x] Tích hợp PyO3 giải phóng GIL cho Python/FastAPI/MCP server.
- [ ] **v0.4.0 (Kế hoạch tiếp theo):**
  - [ ] Bổ sung cơ chế phát hiện lỗi mạch lạc liên câu (Discourse Cohesion).
  - [ ] Mở rộng bảng quy tắc hình thái học cho từ mượn và thuật ngữ công nghệ mới.
  - [ ] Tinh chỉnh ma trận chuyển tiếp POS trên tập ngữ liệu mở rộng.
- [ ] **v1.0.0:**
  - [ ] Cố định API công khai (API Freeze).
  - [ ] Hỗ trợ cấu hình tùy biến luật qua file `vwi.toml`.

---

## 📄 Bản quyền (License)

Dự án được phân phối dưới giấy phép **[MIT License](LICENSE)**.
