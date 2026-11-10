# 🇻🇳 Vietnamese Writing Intelligence (v0.3.0)

> **High-performance Vietnamese NLP Engine & Agentic Writing Framework — Powered by Pure Rust Muscle & Functional Grammar.**

[![crates.io](https://img.shields.io/crates/v/vwi-core.svg?style=flat-square&logo=rust)](https://crates.io/crates/vwi-core)
[![crates.io](https://img.shields.io/crates/v/vwi-cli.svg?style=flat-square&logo=rust&label=vwi-cli)](https://crates.io/crates/vwi-cli)
[![PyPI](https://img.shields.io/pypi/v/vietnamese-writing-intelligence.svg?style=flat-square&logo=python)](https://pypi.org/project/vietnamese-writing-intelligence/)
[![CI Status](https://img.shields.io/github/actions/workflow/status/vantruong-dang/vietnamese-writing-intelligence/ci.yml?branch=main&style=flat-square&logo=github)](https://github.com/vantruong-dang/vietnamese-writing-intelligence/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![Rust: 1.80+](https://img.shields.io/badge/rust-1.80%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Python: 3.11+](https://img.shields.io/badge/python-3.11%2B-blue.svg?style=flat-square&logo=python)](https://www.python.org)

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

## 🤖 Trục Tạo sinh: Agentic Drafting & Voice Profiling

Bên cạnh động cơ cảm biến Rust siêu tốc, VWI duy trì tầng điều phối tạo sinh (**Orchestration Layer**) mạnh mẽ cho AI:

- **State Machine 4 trạng thái:** Phân định minh bạch giữa `clean_rewrite`, `no_change`, `review_comment`, và `needs_author_decision`.
- **Rào chắn Dữ kiện Bất biến (Immutable Anchors):** Cơ chế bảo toàn 100% số liệu kỹ thuật, tên riêng, mã sản phẩm khi LLM biên tập lại văn bản.
- **Fail-Fast Schema Validation:** Kiểm định cấu trúc dàn ý bài viết (`draft-manifest.schema.json`) và hồ sơ văn phong (`voice-profile.schema.json`) theo chuẩn JSON Schema Draft 2020-12.
- **NativeDrafter & V3Orchestrator:** Tự động nạp Voice Profile chuẩn ngữ vực, kết hợp dàn ý Đề - Thuyết để sinh bản thảo tiếng Việt tự nhiên qua LLM Provider (Google Gemini, Anthropic Claude, OpenAI).

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

## 📚 Ghi công & Tri ân (Acknowledgments)

Dự án **Vietnamese Writing Intelligence** là một nỗ lực nghiêm túc nhằm số hóa tri thức ngôn ngữ học bản địa Việt Nam thành một động cơ phần mềm hiệu năng cao. Chúng tôi ghi nhận trực tiếp những nền tảng đã tạo nên lõi Rust và hệ thống luật của dự án:

### 1. Nền tảng Ngôn ngữ học Tiếng Việt Bản địa (Cội nguồn tri thức)
* **Cố GS. Cao Xuân Hạo** (*Tiếng Việt — Sơ thảo ngữ pháp chức năng* & *Tiếng Việt, Văn Việt, Người Việt*):
  * Cung cấp toàn bộ lý thuyết nền tảng cho Shallow Parser: Cấu trúc **Đề – Thuyết (Topic – Comment)**, phân định **Khung đề vs. Chủ đề** (dùng để bắt lỗi "Khung đề què"), và thể nhượng bộ hiện thực/giả định (`Tuy` vs. `Dù`).
* **Học giả Nguyễn Hiến Lê** (*Cổ văn Trung Quốc* & *Luyện văn*):
  * Cung cấp nền tảng về nhịp điệu câu văn biền ngẫu, phép tiểu đối và tiết tấu tiếng Việt tự nhiên (dùng cho các luật phát hiện sáo ngữ AI).

### 2. Dự án Thượng nguồn Trực tiếp (Upstream Dependency)
* **[longhang2004/vietnamese-humanizer](https://github.com/longhang2004/vietnamese-humanizer)** (PyPI: `vietnamese-writing-skills`):
  * Tác giả **Hàng Nhựt Long** và **Lê Ngọc Phương Thư**.
  * Kế thừa & tích hợp: State Machine 4 trạng thái (`clean_rewrite`, `no_change`, `review_comment`, `needs_author_decision`), triết lý bảo toàn dữ kiện ngữ nghĩa (`immutable_anchors`), và taxonomy phân loại lỗi bề mặt tiếng Việt.

### 3. Hạ tầng Ngôn ngữ & Dữ liệu (Technical Infrastructure)
* Crate **[daachorse](https://github.com/daac-tools/daachorse)**: Cung cấp cấu trúc dữ liệu Double-Array Aho-Corasick siêu tốc cho phép quét đa mẫu (multi-pattern scanning) trong độ phức tạp $O(N)$.
* **Universal Dependencies Vietnamese VTB (UD_Vietnamese-VTB)**: Cung cấp ngữ liệu chuẩn vàng (Gold Standard) để huấn luyện ma trận xác suất POS Tagging (Bigram HMM) và trích xuất từ vựng cho lõi Rust.

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
