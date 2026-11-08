# Knowledge-to-Architecture Map (v0.2.0)
*Cơ chế chuyển hóa tri thức từ 7 Repository nguồn mở và 8 Tiên đề Ngôn ngữ học tiếng Việt thành các thành phần kiến trúc có thể đo lường.*

---

## 📌 Tổng quan Chiến lược

Dự án **Vietnamese Writing Intelligence** không được định vị như một bản nâng cấp linter hay một lớp wrapper đơn giản quanh LLM. Mục tiêu của dự án là xây dựng một **nền tảng hiểu, tạo sinh, phân tích và biên tập tiếng Việt có nhận thức sâu sắc về ngôn ngữ, ngữ cảnh, thể loại và giọng văn**.

Mọi thiết kế kỹ thuật đều tuân thủ nguyên tắc phân định 3 cấp độ:
1. **Đã tham khảo (Referenced):** Tiếp thu ý tưởng, triết lý thiết kế.
2. **Đã chuyển hóa thành thiết kế (Architected):** Cụ thể hóa thành Data Contracts, Schema, Interfaces và State Machine.
3. **Đã triển khai & kiểm chứng (Implemented & Validated):** Có mã nguồn thực thi, có unit tests và benchmark định lượng độc lập.

---

## 🗺 Phần I: Ma trận Chuyển hóa từ 7 Repository Mã nguồn Mở

| Repository Nguồn | Nguyên lý / Tri thức cốt lõi | Module tiếp nhận trong V3 | Cơ chế Kỹ thuật (Technical Mechanism) | Chỉ số Kiểm chứng (Evaluation Metric) |
| :--- | :--- | :--- | :--- | :--- |
| **[longhang2004/vietnamese-humanizer](https://github.com/longhang2004/vietnamese-humanizer)** | **State Machine & Preservation Sheet** | `orchestrator.py` & `validators.py` | 4 trạng thái quyết định (`clean_rewrite`, `no_change`, `review_comment`, `needs_author_decision`); cơ chế che chắn code block/URL/email (`mask_protected`). | Tỷ lệ can thiệp không cần thiết (Unnecessary Rewrite Rate $\le 5\%$). |
| **[anthropics/skills](https://github.com/anthropics/skills)** | **Declarative Agent Specification** | `skills/*/SKILL.md` | Định dạng chuẩn YAML Frontmatter, phân lập ranh giới `Trigger When` vs `Do Not Use When`, chuẩn hóa Input/Output context. | Khả năng tương thích parser với Claude Code CLI / Cursor rules (100% Valid YAML & Specs). |
| **[blader/humanizer](https://github.com/blader/humanizer)** | **Negative Patterns & Critique Loop** | `linguistics/context_rules.py` & `drafting/` | Quét dấu vân tay AI (Perplexity thấp, triadic balance) + quy trình 2-Pass: Sinh bản thảo tự do $\rightarrow$ Tự phản biện độc lập. | Giảm tỷ lệ sáo ngữ AI (AI Cliche Reduction Rate $\ge 90\%$). |
| **[danielmiessler/fabric](https://github.com/danielmiessler/fabric)** | **Unix Philosophy for AI Patterns** | `drafting/planner.py` | Chia nhỏ tác vụ viết thành các pattern đơn nhiệm (Single-purpose patterns): trích xuất luận điểm, dàn ý Đề-Thuyết, biên tập vi mô. | Tỷ lệ hoàn thành tác vụ chuyên biệt (Task Precision) không bị trôi ngữ cảnh. |
| **[JuliusBrussee/caveman](https://github.com/JuliusBrussee/caveman)** | **Information Density & Pruning** | `linguistics/phrase_analysis.py` | Cắt tỉa hư từ rỗng, triệt tiêu đại từ thừa (*nó, điều này*), tối đa hóa tỷ lệ Động từ/Danh từ thực (Verb-to-Noun signal). | Mật độ thông tin (Token-to-Fact Ratio: giảm số từ thừa mà giữ nguyên số dữ kiện). |
| **[obra/superpowers](https://github.com/obra/superpowers)** | **Test-First & Verification Gates** | `tests/` & `validation/semantic_audit.py` | Kiểm toán độc lập trước khi trả về kết quả; không tin tưởng đầu ra LLM nếu chưa vượt qua bài test dữ kiện bất biến. | Tỷ lệ rò rỉ dữ kiện sai (Hallucination Escape Rate $= 0\%$). |
| **[ComposioHQ/awesome-claude-skills](https://github.com/ComposioHQ/awesome-claude-skills)** | **Persona & Register Engineering** | `schemas/voice-profile.schema.json` | Phân lập rõ ràng giữa Ngữ vực (Register), Ngữ điệu (Tone) và Vai vế giao tiếp (Relationship) thay vì gộp chung vào một prompt mơ hồ. | Độ nhất quán văn phong giữa các phân đoạn (Cross-paragraph Tone Consistency $\ge 95\%$). |

---

## 🇻🇳 Phần II: Ma trận Chuyển hóa từ 8 Tiên đề Ngôn ngữ học Tiếng Việt

```
                                 HỆ THỐNG NGÔN NGỮ HỌC TIẾNG VIỆT
                                                │
         ┌────────────────────────┬─────────────┴────────────┬────────────────────────┐
         ▼                        ▼                          ▼                        ▼
[1. Ngữ dụng & Cấu trúc]   [2. Cú pháp & Từ loại]     [3. Ngữ âm & Nhịp điệu]   [4. Ngữ vực & Xã hội]
 - Cấu trúc Đề – Thuyết     - Danh hóa có điều kiện    - Phối thanh Bằng - Trắc  - Ma trận Xưng hô Thân tộc
 - Tỉnh lược Chủ ngữ        - Động từ hóa hành động    - Nhịp ngắt Tiểu đối      - Phân lập Ngữ vực & Phương ngữ
```

| Tiên đề Ngôn ngữ học | Bản chất Ngôn ngữ bản địa | Sai lầm của AI/Linter thông thường | Chuyển hóa Kỹ thuật vào Kiến trúc V3 | Phương pháp Kiểm chứng (Validation) |
| :--- | :--- | :--- | :--- | :--- |
| **1. Cấu trúc Đề – Thuyết (Topic – Comment)** | Tiếng Việt là ngôn ngữ thiên về Chủ đề (Topic-prominent). Đưa khung cảnh/đối tượng làm tiền đề (Đề), phán đoán theo sau (Thuyết). | Ép câu văn vào cấu trúc Chủ ngữ – Động từ (SVO) ngô nghê dịch từ tiếng Anh. | `drafting/planner.py` tạo `TopicCommentPlan`: bắt buộc xác định rõ "Khung cảnh/Chủ đề" trước khi gán "Vị ngữ/Hành động". | Cặp đối chứng: Câu SVO gượng ép vs Câu Đề-Thuyết tự nhiên (Human Rating). |
| **2. Tỉnh lược Chủ ngữ (Zero Anaphora)** | Tiếng Việt định vị chủ thể qua ngữ cảnh diễn ngôn. Khi chủ đề đã rõ, câu hoàn toàn không cần đại từ nhân xưng. | Liên tục chèn *"nó", "chúng ta", "họ", "điều này"* vào đầu mỗi câu văn. | `linguistics/phrase_analysis.py` phát hiện đại từ thừa sau dấu chấm nếu ngữ cảnh không đổi chủ thể; tự động gợi ý lược bỏ. | Tỷ lệ loại bỏ đại từ rác mà không gây mơ hồ ngữ nghĩa (Ambiguity Check). |
| **3. Cơ chế Danh hóa có điều kiện (Nominalization)** | Hư từ *sự/việc* đứng trước Động từ chỉ là một dạng danh hóa; nhiều trường hợp là danh từ thực (*việc này*) hoặc thành ngữ cố định (*sự tích*). | Phạt mù quáng mọi sự xuất hiện của `việc/sự + V`. | `linguistics/context_rules.py` phân tích 3 tầng: (1) Cụm cố định $\rightarrow$ Bỏ qua; (2) Chủ ngữ mệnh đề $\rightarrow$ Chấp nhận; (3) Đi sau động từ hành động $\rightarrow$ Cảnh báo. | Precision / Recall trên tập dữ liệu benchmark danh từ hóa. |
| **4. Hư từ & Tình thái từ (Discourse Particles)** | Hệ thống tình thái từ cuối câu (*nhỉ, cơ, chứ, đa, nghen*) và liên từ khẩu ngữ (*khổ nỗi, hóa ra*) định hình quan hệ và cảm xúc. | Nhồi nhét bừa bãi để giả làm "văn người" (gây cảm giác blogger gồng), hoặc cấm sạch khiến câu văn chết cứng. | `voice-profile.schema.json` quản lý cờ `colloquial_particles_allowed` gắn chặt với `register`. Phân định rõ văn bản học thuật (cấm) vs đàm thoại (cho phép). | Kiểm tra tính phù hợp ngữ vực (Register Compliance Rate). |
| **5. Cân bằng Bằng – Trắc & Nhịp điệu (Prosody)** | Văn xuôi tự nhiên của người Việt có tính nhạc ngầm: nhịp chẵn (2/2, 4/4) và phân bổ thanh điệu (Bằng: ngang/huyền; Trắc: sắc/hỏi/ngã/nặng). | Đánh giá văn phong chỉ qua số đếm từ (Word count) hoặc ép câu ngắn cụt lủn. | `linguistics/prosody.py` (chỉ đóng vai trò Sensor phân tích, không phạt cứng): cảnh báo chuỗi âm trắc gắt gỏng hoặc nhịp câu mất cân đối ở đuôi đoạn. | So sánh phân phối thanh điệu với Pre-LLM Gold Corpus (Distribution Drift). |
| **6. Cấu trúc Hán-Việt vs Thuần Việt** | Từ Hán-Việt mang tính trừu tượng, trang trọng, cô đọng; từ thuần Việt mang tính gợi hình, đời thường, cụ thể. | Trộn lẫn từ Hán-Việt cổ với khẩu ngữ hiện đại trong cùng một đoạn văn. | `linguistics/pos.py` & `lexical_audit.py` đo mật độ Hán-Việt (`sino_vietnamese_density`) theo 3 mức: *bình dân, cân bằng, học thuật cao*. | Tỷ lệ lệch tông từ vựng (Lexical Tone Drift $\le 2\%$). |
| **7. Phân tầng Ngữ vực (Registers) độc lập với Phương ngữ** | Một nhà khoa học miền Nam viết báo cáo khoa học vẫn dùng từ toàn dân trang trọng; người miền Bắc viết kịch vẫn dùng phương ngữ Nam Bộ. | Đồng nhất "phương ngữ" với "từ sai chuẩn" hoặc đánh đồng ngữ vực thân mật với phương ngữ. | Tách rời 2 trục trong schema: `register` (học thuật, hành chính, thân mật...) và `dialect_bias` (Bắc, Trung, Nam, Toàn dân). | Ma trận kiểm thử chéo 4 Ngữ vực $\times$ 3 Vùng miền. |
| **8. Quan hệ Xưng hô Thân tộc hóa (Kinship Honorifics)** | Tiếng Việt không có đại từ trung tính tuyệt đối như *You/I*; đại từ phản ánh vị thế xã hội, tuổi tác, sự tôn trọng và khoảng cách quyền lực. | AI dùng lộn xộn *"bạn - tôi"*, *"chúng tôi - chúng ta"* (bao gồm hay loại trừ người nghe). | `validation/register_audit.py` kiểm tra tính nhất quán của ngôi xưng: cấm lẫn lộn giữa *chúng tôi* (exclusive) và *chúng ta* (inclusive). | Consistency Check trên toàn văn bản (Pronoun Drift $= 0$). |

---

## 🎯 Phần III: Tiêu chuẩn Nghiệm thu Định lượng cho Sprint v0.2.0

### 1. Cổng Kiểm định Ngôn ngữ (Linguistic Quality Gate - Trục A)
* **Tập dữ liệu kiểm chuẩn (Benchmark Corpus):** Tạo `tests/data/linguistic_gold_standard.jsonl` gồm **150 câu tiếng Việt** có gán nhãn thủ công (Ground Truth) chia đều cho 3 nhóm:
  - Nhóm 1: 50 câu danh từ hóa (*hợp lệ, không hợp lệ, từ ghép cố định*).
  - Nhóm 2: 50 câu về ranh giới từ và cấu trúc Đề – Thuyết.
  - Nhóm 3: 50 câu chứa code block, URL, tên riêng, thuật ngữ kỹ thuật.
* **Chỉ số đo lường (Metrics):**
  - **Context-Rule Precision:** $\ge 85\%$ (không được báo lỗi oan cho câu danh hóa hợp lệ).
  - **Context-Rule Recall:** $\ge 80\%$ (bắt được các cấu trúc dịch thô sáo rỗng thực sự).
  - **False Positive trên Code/Thuật ngữ:** $0\%$ (nhờ tầng che chắn).

### 2. Cổng Kiểm định Tạo sinh (Generative Quality Gate - Trục B)
* **Bảo toàn dữ kiện (Anchor Preservation):**
  - Mọi bản nháp sinh ra từ `DraftPlanner` + `BaseLLMProvider` khi chạy qua `SemanticAuditor` phải đạt:
    $$\text{Metrics Retention} = 100\% \quad \text{và} \quad \text{Domain Terms Retention} = 100\%$$
* **Không phụ thuộc Vendor (Vendor Agnostic):**
  - Có thể thay thế liền mạch giữa `MockLLMProvider` (dùng cho CI/CD), `OllamaProvider` (chạy local offline) và `Anthropic/OpenAIProvider` mà không phải sửa code trong `DraftPlanner`.
