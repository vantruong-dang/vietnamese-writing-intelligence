# 🇻🇳 Vietnamese Writing Intelligence (v0.3.1)

> **High-performance Vietnamese Functional Grammar Linter & Offline WebAssembly Checker — Powered by Pure Rust.**

[![crates.io](https://img.shields.io/crates/v/vwi-core.svg?style=flat-square&logo=rust)](https://crates.io/crates/vwi-core)
[![crates.io](https://img.shields.io/crates/v/vwi-cli.svg?style=flat-square&logo=rust&label=vwi-cli)](https://crates.io/crates/vwi-cli)
[![PyPI](https://img.shields.io/pypi/v/vietnamese-writing-intelligence.svg?style=flat-square&logo=python)](https://pypi.org/project/vietnamese-writing-intelligence/)
[![CI Status](https://img.shields.io/github/actions/workflow/status/vantruong-dang/vietnamese-writing-intelligence/ci.yml?branch=main&style=flat-square&logo=github)](https://github.com/vantruong-dang/vietnamese-writing-intelligence/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![Rust: 1.80+](https://img.shields.io/badge/rust-1.80%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Python: 3.11+](https://img.shields.io/badge/python-3.11%2B-blue.svg?style=flat-square&logo=python)](https://www.python.org)

---

## 🎯 Tại sao nên dùng VWI?

Trong kỷ nguyên AI tạo sinh, người viết và biên tập viên không thiếu công cụ để sinh chữ (ChatGPT, Claude, Gemini đã làm việc đó rất tốt). Điều họ thực sự thiếu là một **Trọng tài công tâm, chính xác và bảo mật tuyệt đối** để kiểm tra văn phong trước khi xuất bản hoặc gửi cho khách hàng.

VWI được thiết kế để giải quyết bài toán đó dựa trên **Bộ Ba Trụ Cột Thực Chất**:

1. **Tất định & Không đoán mò (Deterministic):** VWI không dùng AI để đoán lỗi hay sinh ảo giác (hallucination). Hệ thống áp dụng chính xác **Ngữ pháp chức năng của Cố GS. Cao Xuân Hạo**, chuẩn tiết tấu của học giả **Nguyễn Hiến Lê**, và quy chuẩn chính tả thể thức văn bản theo **Nghị định 30/2020/NĐ-CP**.
2. **Bảo mật tuyệt đối 100% Offline (Zero Network Telemetry):** Lõi WebAssembly chạy trực tiếp trong trình duyệt hoặc binary máy trạm. **Không một ký tự nào** của bạn bị gửi qua Internet. Hoàn toàn an toàn cho hợp đồng pháp lý, tài liệu nội bộ và văn bản mật.
3. **Tốc độ ánh sáng (< 2 mili-giây):** Xử lý văn bản bằng Rust và cấu trúc dữ liệu Double-Array Trie (`daachorse`), đạt tốc độ sub-millisecond (nhanh gấp 20–50 lần các công cụ Python thông thường).

---

## 🏛️ Hệ Sinh Thái 4 Nền Tảng (Quad-Platform Architecture)

```
                     ┌─────────────────────────────────────────────────────────┐
                     │               VWI MULTI-PLATFORM ECOSYSTEM              │
                     └────────────────────────────┬────────────────────────────┘
                                                  │
         ┌────────────────────────┬───────────────┴───────────────┬────────────────────────┐
         │                        │                               │                        │
         ▼                        ▼                               ▼                        ▼
  crates/vwi-core           crates/vwi-wasm                 crates/vwi-cli           crates/vwi-py
  ┌───────────────┐        ┌───────────────┐               ┌───────────────┐        ┌───────────────┐
  │ • Rust Thuần  │        │ • WebAssembly │               │ • Standalone  │        │ • PyO3 Engine │
  │ • Viterbi 72k │        │ • Chrome Ext  │               │ • Zero-dep CLI│        │ • Release GIL │
  │ • 25+ Rules   │        │ • 100% Local  │               │ • CI/CD Gate  │        │ • FastAPI/MCP │
  │ • Sub-ms core │        │ • < 2ms Audit │               │ • ANSI Output │        │ • Python API  │
  └───────────────┘        └───────────────┘               └───────────────┘        └───────────────┘
```

1. **`vwi-core` (Rust Thuần):** Thư viện lõi độc lập, không cấp phát heap thừa, nhúng sẵn 72.564 từ vựng và ma trận xác suất Bigram HMM.
2. **`vwi-wasm` (Chrome Extension):** Đóng gói WebAssembly chạy trực tiếp trên trình duyệt Chrome, Cốc Cốc, Edge với giao diện thân thiện, hỗ trợ chọn đa ngữ vực và phím tắt soát lỗi tức thì.
3. **`vwi-cli` (Native Binary):** Ứng dụng nhị phân độc lập (~5MB) cho Linux, macOS, Windows, tích hợp hoàn hảo làm Quality Gate trong Git Hook hoặc CI/CD pipeline.
4. **`vwi-py` (Python PyO3 C-Extension):** Cầu nối PyO3 giải phóng Python GIL (`py.allow_threads`), cung cấp API kiểm toán và giao thức Model Context Protocol (MCP) cho Claude Desktop / Cursor.

---

## 🌐 1. Chrome Extension (WASM 100% Offline)

Tiện ích mở rộng chính thức của VWI giúp soát lỗi văn bản trực tiếp trên trình duyệt mà không cần cài đặt môi trường lập trình.

### Tính năng chính trên Extension:
* **Tự động bắt văn bản đang chọn:** Bôi đen đoạn văn trên bất kỳ trang web nào (Gmail, Google Docs, Facebook, báo điện tử) và bấm **"Lấy chữ bôi đen"**.
* **Đa ngữ vực (Multi-Register Selection):**
  - 📰 **Báo chí / Phổ thông:** Kiểm tra câu cụt, chủ ngữ giả, bị động dịch máy, sáo ngữ.
  - 📑 **Hành chính / Pháp lý:** Kiểm tra thể thức theo Nghị định 30/2020/NĐ-CP (viết hoa cơ quan, đơn vị hành chính), miễn trừ kiểm tra âm luật thơ ca.
  - ✒️ **Văn học / Tùy bút:** Kiểm soát chặt chẽ nhịp điệu Bằng – Trắc (phát hiện chuỗi thanh Trắc liên tiếp vượt chuẩn P99).
  - 💻 **Kỹ thuật / Công nghệ:** Cho phép thuật ngữ tiếng Anh đan xen, tập trung vào tính logic và tường minh.
* **Đề xuất sửa lỗi 1-Click (Actionable Suggestions):** Hiển thị trực quan đoạn văn bản cần lưu ý kèm lý do và câu sửa gợi ý.

> Cài đặt nhanh: Tải file nén tại [Releases](https://github.com/vantruong-dang/vietnamese-writing-intelligence/releases/latest) (`vwi-chrome-extension-v0.3.1.zip`), vào `chrome://extensions` bật Developer Mode và chọn *Load Unpacked*.

---

## ⚡ 2. Standalone Binary CLI (`vwi` — Không cần Python)

Tải binary độc lập duy nhất (~5MB), không cần cài đặt bất kỳ runtime phụ thuộc nào:

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
```

### Sử dụng nhanh CLI `vwi`:

```bash
# 1. Kiểm tra văn bản trực tiếp từ dòng lệnh
vwi check -t "Trong bối cảnh hiện nay đòi hỏi giải pháp tối ưu nhất được phê duyệt bởi hội đồng."

# 2. Kiểm tra văn bản theo ngữ vực hành chính (Nghị định 30)
vwi check -i van_ban.txt --register administrative

# 3. Sử dụng làm Quality Gate trong CI/CD (Thoát mã exit code 1 nếu điểm dưới 80)
vwi check -i README.md --fail-under 80

# 4. Tách từ tiếng Việt bằng Viterbi Trellis (72.5k từ vựng)
vwi tokenize "Mẹ mua một chiếc bàn là mới để là quần áo cho học sinh học bài."
# Kết quả: [Mẹ] [mua] [một] [chiếc] [bàn là] [mới] [để] [là] [quần áo] [cho] [học sinh] [học] [bài] [.]
```

---

## 🐍 3. Cài đặt & Sử dụng Python

```bash
# Cài đặt thư viện cốt lõi siêu nhẹ (Lõi Rust + CLI vn-intel)
pip install vietnamese-writing-intelligence

# Cài đặt kèm tính năng MCP Server cho Claude Desktop / Cursor
pip install "vietnamese-writing-intelligence[mcp]"

# (Tùy chọn) Cài đặt kèm các công cụ sinh bản thảo LLM
pip install "vietnamese-writing-intelligence[drafting]"
```

### Sử dụng Python API:

```python
import vn_writing_intel as vwi

# 1. Tách từ Viterbi Trellis
segmenter = vwi.RustViterbiSegmenter()
tokens = segmenter.tokenize("Mẹ mua một chiếc bàn là mới.")
print([(t.text, t.is_compound, t.pos) for t in tokens])

# 2. Kiểm tra văn phong tiếng Việt
auditor = vwi.Auditor()
findings = auditor.audit("Trong bối cảnh hiện nay đòi hỏi giải pháp tối ưu nhất.")
for f in findings:
    print(f"[{f.severity}] {f.pattern_id}: {f.matched_span}")
    print(f"  -> Lý do: {f.rationale}")
    if f.suggestion:
        print(f"  -> Gợi ý sửa: {f.suggestion}")
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

## 🔍 Hệ thống 25+ Luật Ngôn ngữ học Bản địa

Hệ thống cảm biến được phân chia thành các nhóm cốt lõi:

1. **Cấu trúc Khung đề - Thuyết (Cao Xuân Hạo):** Bắt câu què biến Khung đề thành Chủ ngữ (*"Trong báo cáo này đòi hỏi..."*), vế câu cụt sau động từ phát ngôn, và xung đột giới từ - liên từ (*"Với... nhưng..."*).
2. **Bị động Nhân tạo & Giả thụ hưởng:** Phát hiện cấu trúc sao phỏng Anh ngữ *"được/bị ... bởi"* (Passive `be + V3 + by`) và lạm dụng từ *"nhận được"* cho hành động chủ động.
3. **Dịch ngữ (Translationese) & Chủ ngữ giả:** Bắt chủ ngữ giả Tây phương (*"Nó là rất quan trọng để..."*), tính từ hóa khiên cưỡng (*"mang tính chất"*), và đảo ngữ trạng từ thời gian không có dấu phẩy ngắt nhịp.
4. **Sáo ngữ Hành chính & Đệm từ rỗng:** Động cơ Aho-Corasick bắt đệm động từ rỗng (*"tiến hành thực hiện"*, *"triển khai công tác"*), chêm liên từ thừa (*"và đồng thời"*, *"cũng như là"*), và trợ từ cực cấp sáo rỗng (*"vô cùng hết sức"*).
5. **Trùng ngữ Logic (Pleonasm):** Bắt lỗi lặp thừa ý tố Hán - Việt (*"tối ưu nhất"*, *"nguyên nhân là bởi vì"*, *"ánh nắng mặt trời"*).
6. **Thể nhượng bộ & Xung đột liên từ:** Phân định thể nhượng bộ Hiện thực (*"Tuy/Mặc dù"*) vs. Giả định (*"Dù"*), và bắt xung đột nhượng bộ kép (*"Mặc dù... tuy nhiên..."*).
7. **Quy chuẩn Chính tả & Nghị định 30:** Bắt lỗi dính phím gõ Telex, khoảng cách trước sau dấu câu, lỗi viết hoa tên cơ quan nhà nước và đơn vị hành chính.
8. **Âm luật Tiết tấu (Prosody):** Phân tích chuỗi thanh Trắc liên tiếp vượt ngưỡng phân vị 99% ($P_{99}$) của văn xuôi thuần Việt.

---

## 📊 Kết quả Đo Kiểm Thực Tế (`vn-intel benchmark`)

Hệ thống duy trì hai bộ kiểm chuẩn độc lập với kết quả đo lường định lượng nghiêm ngặt:

1. **Bộ Kiểm thử Hồi quy Tất định (Deterministic Gold Suite — 200 ca):** Đạt **100.00% Precision, 100.00% Recall, 0.00% FPR**.
2. **Bộ Kiểm chuẩn Thực tế Độc lập (Real-World Benchmark — 300 ca):** Đánh giá trên 150 câu lỗi thật từ LLM/bản dịch máy và 150 câu chuẩn mực từ báo chí (*VnExpress, Tuổi Trẻ, Thanh Niên*) và tác phẩm của GS. Cao Xuân Hạo, học giả Nguyễn Hiến Lê:

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

## 🚀 Đo kiểm Hiệu năng (Performance Benchmarks)

Kết quả đo kiểm trên máy tính tiêu chuẩn (10.000 lượt lặp):

| Thành phần kiểm thử | Công nghệ / Thuật toán | Tốc độ xử lý | Tăng tốc |
| :--- | :--- | :---: | :---: |
| **Tách từ (Tokenization)** | Python Regex (64 từ ghép thủ công) | $15,029\text{ ops/s}$ | $1.00\times$ *(Baseline)* |
| | **Rust Daachorse** (72,564 từ nhúng, Greedy) | **$43,920\text{ ops/s}$** | **$2.92\times$** *(Từ điển lớn gấp 1,133 lần)* |
| | **Rust Viterbi Trellis** (72,564 từ + Bigram LM) | **$17,335\text{ ops/s}$** | **$1.15\times$** *(Giải quyết triệt để nhập nhằng)* |
| **Soát lỗi Ngôn ngữ (25 Rules)** | Python thuần (Vòng lặp 25 Class Rules) | $4,774\text{ ops/s}$ | $1.00\times$ *(Baseline)* |
| | **Rust Auditor (Native C-ABI đơn luồng)** | $5,578\text{ ops/s}$ | **$1.17\times$** |
| | **Rust Auditor Batch (Rayon đa nhân CPU)** | **$21,239\text{ ops/s}$** | **$4.45\times$** |
| | **WebAssembly in Browser (< 500 từ)** | **$0.6 - 2.0\text{ ms}$** | **Tức thì (Instant)** |

---

## 🧪 Tính năng Phụ trợ: Lập dàn ý & Rào chắn Dữ kiện (Optional Drafting Modules)

Dành cho các nhà phát triển muốn tích hợp quy trình sinh bản thảo tự động với LLM nhưng cần giữ trọn vẹn sự thật dữ liệu:

* **Rào chắn Dữ kiện Bất biến (Immutable Anchors):** Cơ chế bảo toàn 100% số liệu kỹ thuật, tên riêng, mã sản phẩm khi LLM biên tập lại văn bản.
* **Fail-Fast Schema Validation:** Kiểm định cấu trúc dàn ý bài viết (`draft-manifest.schema.json`) và hồ sơ văn phong (`voice-profile.schema.json`) theo chuẩn JSON Schema Draft 2020-12.
* **State Machine 4 trạng thái:** Phân định rõ giữa `clean_rewrite`, `no_change`, `review_comment`, và `needs_author_decision`.

Để sử dụng, cài đặt thêm phụ thuộc tùy chọn: `pip install "vietnamese-writing-intelligence[drafting]"`.

---

## 📌 Phạm vi & Giới hạn Kỹ thuật Thực tế

Để minh bạch tuyệt đối về mặt kỹ thuật, người dùng cần lưu ý phạm vi của VWI:

### Những gì VWI làm XUẤT SẮC:
* **Tách từ xác suất độ chính xác cao:** Thuật toán Viterbi Trellis kết hợp `daachorse` (72.500+ từ vựng) giải quyết triệt để các ca nhập nhằng phổ biến (*"chiếc bàn là"* vs *"cái bàn là..."*).
* **Soát lỗi văn phong & Chống văn dịch (Translationese):** Bắt chính xác câu bị động nhân tạo sao phỏng tiếng Anh, chủ ngữ giả, và sáo ngữ hành chính rập khuôn.
* **Phân tích Khung Đề - Thuyết (Functional Topic-Comment):** Nhận diện lỗi ngữ pháp chức năng phổ biến nhất: Giới ngữ đứng đầu câu bị ép làm chủ thể cho vị từ nòng cốt (*"Trong bối cảnh... đòi hỏi"*).
* **Bảo mật tuyệt đối 100% Offline:** Chạy hoàn toàn cục bộ, không gửi dữ liệu ra bên ngoài.

### Những gì VWI KHÔNG LÀM (Nằm ngoài phạm vi):
* **Không phải Deep Learning Transformer:** VWI không thay thế PhoBERT hay LLM trong các tác vụ đọc hiểu ngữ nghĩa toàn văn, tóm tắt tự động, hay hỏi đáp (Q&A).
* **Không phải AI Text Classifier đoán mò:** VWI không cố gắng phán đoán một đoạn văn là do người hay AI viết dựa trên perplexity vu vơ. Chúng tôi chỉ bắt các lỗi ngữ pháp và sáo ngữ có căn cứ ngôn ngữ học.
* **Không phải Full Dependency Treebank:** VWI sử dụng Shallow Parser tập trung vào các thành phần phục vụ soát lỗi văn phong (Khung đề, Vị từ, Liên từ), không dựng toàn bộ cây phụ thuộc 5 tầng bậc.

---

## 📚 Ghi công & Tri ân (Acknowledgments)

* **Cố GS. Cao Xuân Hạo** (*Tiếng Việt — Sơ thảo ngữ pháp chức năng* & *Tiếng Việt, Văn Việt, Người Việt*): Nền tảng lý thuyết cho cấu trúc Đề – Thuyết và phân định Khung đề vs. Chủ đề.
* **Học giả Nguyễn Hiến Lê** (*Cổ văn Trung Quốc* & *Luyện văn*): Nền tảng về nhịp điệu câu văn biền ngẫu, phép tiểu đối và tiết tấu thuần Việt.
* **Dự án Thượng nguồn [longhang2004/vietnamese-humanizer](https://github.com/longhang2004/vietnamese-humanizer)** (Tác giả Hàng Nhựt Long và Lê Ngọc Phương Thư): Kế thừa State Machine 4 trạng thái và triết lý bảo toàn dữ kiện ngữ nghĩa.
* **Crate [daachorse](https://github.com/daac-tools/daachorse)**: Cung cấp cấu trúc dữ liệu Double-Array Aho-Corasick siêu tốc.
* **Universal Dependencies Vietnamese VTB**: Cung cấp ngữ liệu chuẩn vàng để trích xuất từ vựng và ma trận xác suất chuyển tiếp Bigram.

---

## 📄 Bản quyền (License)

Dự án được phân phối dưới giấy phép **[MIT License](LICENSE)**.
