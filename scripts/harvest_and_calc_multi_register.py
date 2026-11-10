"""
KỊCH BẢN THỰC NGHIỆM ĐA NGỮ VỰC (MULTI-REGISTER EMPIRICAL PROSODY HARVESTER)
Tự động phân tích và tạo Baseline cho 4 thể loại:
1. Literary (Văn học / Tùy bút)
2. Journalism (Báo chí thời sự)
3. Administrative (Hành chính / Pháp lý)
4. Technical (Khoa học / Kỹ thuật)
"""

import math
import re
import unicodedata
import urllib.request
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent.parent
DATA_DIR = ROOT_DIR / "data_raw"
DATA_DIR.mkdir(parents=True, exist_ok=True)

RUST_OUTPUT_FILE = ROOT_DIR / "crates" / "vwi-core" / "src" / "rules" / "generated_prosody_baselines.rs"

VNEXPRESS_RSS = [
    "https://vnexpress.net/rss/tin-moi-nhat.rss",
    "https://vnexpress.net/rss/thoi-su.rss",
    "https://vnexpress.net/rss/kinh-doanh.rss",
    "https://vnexpress.net/rss/khoa-hoc.rss",
    "https://vnexpress.net/rss/so-hoa.rss",
]

# Dữ liệu hành chính mẫu đại diện cho văn bản quy phạm pháp luật
ADMINISTRATIVE_SAMPLE_CORPUS = """
Căn cứ Luật Tổ chức Chính phủ ngày 19 tháng 6 năm 2015 và Luật sửa đổi, bổ sung một số điều của Luật Tổ chức Chính phủ.
Căn cứ Luật Ban hành văn bản quy phạm pháp luật ngày 22 tháng 6 năm 2015.
Theo đề nghị của Bộ trưởng Bộ Kế hoạch và Đầu tư về việc ban hành quy định chi tiết thi hành một số điều của Luật Đầu tư công.
Chính phủ ban hành Nghị định quy định chi tiết thi hành một số điều của Luật Đầu tư công về quản lý và sử dụng nguồn vốn ngân sách nhà nước.
Nghị định này áp dụng đối với cơ quan, tổ chức, cá nhân tham gia hoặc có liên quan đến hoạt động đầu tư công, quản lý và sử dụng vốn đầu tư công.
Việc quản lý, sử dụng vốn đầu tư công phải tuân thủ đúng quy định của pháp luật về quản lý ngân sách nhà nước và các quy định của Nghị định này.
Bảo đảm quản lý tập trung, thống nhất về mục tiêu, cơ chế, chính sách và thực hiện phân cấp trong quản lý đầu tư công theo quy định.
Các bộ, cơ quan trung ương và Ủy ban nhân dân các cấp có trách nhiệm tổ chức triển khai thực hiện kế hoạch đầu tư công theo đúng thẩm quyền.
Thủ trưởng cơ quan, tổ chức sử dụng vốn đầu tư công chịu trách nhiệm toàn diện trước pháp luật về tính chính xác của số liệu và hiệu quả đầu tư.
Quyết định phê duyệt chủ trương đầu tư phải nêu rõ mục tiêu, quy mô, tổng mức đầu tư, cơ cấu nguồn vốn và tiến độ thực hiện dự án.
Việc phân bổ vốn kế hoạch đầu tư công trung hạn và hằng năm phải tuân thủ nguyên tắc, tiêu chí và định mức quy định tại Luật Đầu tư công.
""" * 50

TECHNICAL_SAMPLE_CORPUS = """
Hệ thống xử lý luồng dữ liệu phân tán được thiết kế dựa trên kiến trúc vi dịch vụ với khả năng chịu lỗi cao.
Các tiến trình xử lý thông điệp bất đồng bộ thông qua hàng đợi phân tán đảm bảo tính toàn vẹn dữ liệu ở mức độ cao nhất.
Cơ chế sao lưu dữ liệu phân tán cho phép tự động khôi phục trạng thái hoạt động của cụm máy chủ khi xảy ra sự cố phần cứng.
Giao thức mạng hiệu năng cao được tối ưu hóa để giảm thiểu độ trễ truyền gói tin giữa các nút mạng trong cụm tính toán.
Thuật toán phân mảnh dữ liệu phân tán sử dụng hàm băm nhất quán để phân bổ tải đồng đều trên các máy chủ lưu trữ.
Kiến trúc bộ nhớ đệm đa tầng giúp tăng tốc độ truy xuất dữ liệu đối với các truy vấn đọc lặp lại có tần suất cao.
Cơ chế kiểm soát phiên bản dữ liệu bằng khóa lạc quan ngăn chặn hiện tượng xung đột dữ liệu khi có nhiều tiến trình ghi đồng thời.
Mô hình triển khai tự động hóa thông qua container cho phép mở rộng quy mô hệ thống theo chiều ngang một cách linh hoạt.
Giao thức mã hóa đầu cuối bảo vệ an toàn cho dữ liệu nhạy cảm trong toàn bộ quá trình truyền tải qua mạng internet.
""" * 50

SAC_VOWELS = set("áắấéếíóốớúứýÁẮẤÉẾÍÓỐỚÚỨÝ")
HOI_VOWELS = set("ảẳẩẻểỉỏổởủửỷẢẲẨẺỂỈỎỔỞỦỬỶ")
NGA_VOWELS = set("ãẵẫẽễĩõỗỡũữỹÃẴẪẼỄĨÕỖỠŨỮỸ")
NANG_VOWELS = set("ạặậẹệịọộợụựỵẠẶẬẸỆỊỌỘỢỤỰỴ")
HUYEN_VOWELS = set("àằầèềìòồờùừỳÀẰẦÈỀÌÒỒỜÙỪỲ")

def get_tone(syllable: str) -> str:
    syllable = unicodedata.normalize("NFC", syllable.strip().lower())
    for char in syllable:
        if char in SAC_VOWELS or char in HOI_VOWELS or char in NGA_VOWELS or char in NANG_VOWELS:
            return "T"
        if char in HUYEN_VOWELS:
            return "B"
    if re.search(r"[aăâeêioôơuưy]", syllable):
        return "B"
    return "U"

def percentile(data: list[float], p: float) -> float:
    if not data:
        return 0.0
    k = (len(data) - 1) * p
    f = math.floor(k)
    c = math.ceil(k)
    if f == c:
        return data[int(k)]
    return data[int(f)] * (c - k) + data[int(c)] * (k - f)

def harvest_journalism_from_rss() -> str:
    print("[*] Đang thu thập dữ liệu báo chí từ VnExpress RSS...")
    articles = []
    headers = {"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"}

    for url in VNEXPRESS_RSS:
        try:
            req = urllib.request.Request(url, headers=headers)
            with urllib.request.urlopen(req, timeout=5) as response:
                xml_data = response.read()
                root = ET.fromstring(xml_data)
                for item in root.findall(".//item"):
                    desc = item.find("description")
                    title = item.find("title")
                    if title is not None and title.text:
                        articles.append(title.text)
                    if desc is not None and desc.text:
                        # Bóc tách HTML thô trong description
                        clean_desc = re.sub(r"<[^>]+>", " ", desc.text)
                        articles.append(clean_desc)
        except Exception as e:
            print(f"[-] Lỗi cào RSS {url}: {e}")

    text = "\n".join(articles)
    print(f"[+] Thu hoạch thành công: {len(text.split()):,} từ báo chí.")
    return text

def calculate_stats(corpus_text: str, name: str) -> dict:
    raw_sentences = re.split(r"[.?!;\n]+", corpus_text)
    sentences = [s.strip() for s in raw_sentences if len(s.strip().split()) >= 4]

    if not sentences:
        return {
            "mean_len": 20.0, "std_len": 10.0,
            "mean_trac": 0.5, "std_trac": 0.1,
            "p95_trac_run": 7.0, "p99_trac_run": 10.0,
            "total_words": 0,
        }

    sentence_lengths = []
    sentence_trac_ratios = []
    consecutive_trac_runs = []

    for s in sentences:
        words = [re.sub(r"[^\wà-ỹđ]", "", w) for w in s.split()]
        words = [w for w in words if w]
        if not words:
            continue

        sentence_lengths.append(len(words))
        tones = [get_tone(w) for w in words]
        bang_cnt = sum(1 for t in tones if t == "B")
        trac_cnt = sum(1 for t in tones if t == "T")

        if bang_cnt + trac_cnt > 0:
            sentence_trac_ratios.append(trac_cnt / (bang_cnt + trac_cnt))

        curr_run = 0
        max_run = 0
        for t in tones:
            if t == "T":
                curr_run += 1
                if curr_run > max_run:
                    max_run = curr_run
            else:
                curr_run = 0
        consecutive_trac_runs.append(max_run)

    n = len(sentence_lengths)
    mean_len = sum(sentence_lengths) / n
    var_len = sum((x - mean_len) ** 2 for x in sentence_lengths) / n
    std_len = math.sqrt(var_len)

    mean_trac = sum(sentence_trac_ratios) / len(sentence_trac_ratios)
    var_trac = sum((x - mean_trac) ** 2 for x in sentence_trac_ratios) / len(sentence_trac_ratios)
    std_trac = math.sqrt(var_trac)

    sorted_runs = sorted(consecutive_trac_runs)
    p95_trac_run = percentile(sorted_runs, 0.95)
    p99_trac_run = percentile(sorted_runs, 0.99)

    print(f"\n--- {name.upper()} STATS ---")
    print(f"Tổng số từ        : {sum(sentence_lengths):,}")
    print(f"Độ dài câu trung bình: {mean_len:.2f} (±{std_len:.2f})")
    print(f"Tỷ lệ Trắc        : {mean_trac * 100:.2f}% (±{std_trac * 100:.2f}%)")
    print(f"Ngưỡng P99 Trắc   : {p99_trac_run:.1f} tiếng")

    return {
        "mean_len": mean_len,
        "std_len": std_len,
        "mean_trac": mean_trac,
        "std_trac": std_trac,
        "p95_trac_run": p95_trac_run,
        "p99_trac_run": p99_trac_run,
        "total_words": sum(sentence_lengths),
    }

def main():
    print("=" * 60)
    print(">>> THỰC NGHIỆM ĐA NGỮ VỰC CHO TIẾNG VIỆT (MULTI-REGISTER)")
    print("=" * 60)

    # 1. Literary Corpus (Sách Cao Xuân Hạo & Nguyễn Hiến Lê)
    gold_file = DATA_DIR / "human_gold.txt"
    literary_text = gold_file.read_text(encoding="utf-8") if gold_file.exists() else ""
    literary_stats = calculate_stats(literary_text, "Literary / Essay")

    # 2. Journalism Corpus (RSS VnExpress)
    journalism_text = harvest_journalism_from_rss()
    journalism_stats = calculate_stats(journalism_text, "Journalism")

    # 3. Administrative Corpus (Luật / Nghị định)
    admin_stats = calculate_stats(ADMINISTRATIVE_SAMPLE_CORPUS, "Administrative")

    # 4. Technical Corpus (Kiến trúc kỹ thuật)
    tech_stats = calculate_stats(TECHNICAL_SAMPLE_CORPUS, "Technical")

    # Xuất Rust Code
    print(f"\n[*] Đang sinh mã nguồn Rust: {RUST_OUTPUT_FILE.name}...")
    with open(RUST_OUTPUT_FILE, "w", encoding="utf-8") as f:
        f.write("// FILE TỰ ĐỘNG SINH BỞI scripts/harvest_and_calc_multi_register.py\n")
        f.write("// MA TRẬN BASELINE ĐA NGỮ VỰC (MULTI-REGISTER PROSODY BASELINES)\n\n")

        f.write("#[derive(Debug, Clone, Copy, PartialEq)]\n")
        f.write("pub struct ProsodyBaseline {\n")
        f.write("    pub mean_sentence_len: f32,\n")
        f.write("    pub std_dev_sentence_len: f32,\n")
        f.write("    pub mean_trac_ratio: f32,\n")
        f.write("    pub std_dev_trac_ratio: f32,\n")
        f.write("    pub p99_consecutive_trac_threshold: usize,\n")
        f.write("}\n\n")

        for var_name, s in [
            ("LITERARY_ESSAY_BASELINE", literary_stats),
            ("JOURNALISM_BASELINE", journalism_stats),
            ("ADMINISTRATIVE_BASELINE", admin_stats),
            ("TECHNICAL_BASELINE", tech_stats),
        ]:
            f.write(f"pub static {var_name}: ProsodyBaseline = ProsodyBaseline {{\n")
            f.write(f"    mean_sentence_len: {s['mean_len']:.2f}f32,\n")
            f.write(f"    std_dev_sentence_len: {s['std_len']:.2f}f32,\n")
            f.write(f"    mean_trac_ratio: {s['mean_trac']:.4f}f32,\n")
            f.write(f"    std_dev_trac_ratio: {s['std_trac']:.4f}f32,\n")
            f.write(f"    p99_consecutive_trac_threshold: {math.ceil(s['p99_trac_run'])},\n")
            f.write("};\n\n")

        f.write("pub fn get_baseline_for_register(register: crate::types::Register) -> Option<&'static ProsodyBaseline> {\n")
        f.write("    match register {\n")
        f.write("        crate::types::Register::Literary => Some(&LITERARY_ESSAY_BASELINE),\n")
        f.write("        crate::types::Register::Journalism => Some(&JOURNALISM_BASELINE),\n")
        f.write("        crate::types::Register::Administrative => Some(&ADMINISTRATIVE_BASELINE),\n")
        f.write("        crate::types::Register::Technical => Some(&TECHNICAL_BASELINE),\n")
        f.write("        crate::types::Register::General => Some(&JOURNALISM_BASELINE),\n")
        f.write("    }\n")
        f.write("}\n")

    print("[+] Hoàn tất xuất ma trận Baseline vào Rust!")

if __name__ == "__main__":
    main()
