#!/usr/bin/env python3
"""
ĐỘNG CƠ THU HOẠCH ĐA NGUỒN & BIÊN DỊCH BASELINE ĐA NGỮ VỰC CHO TIẾNG VIỆT
(UNIFIED MULTI-SOURCE HARVESTER & EMPIRICAL PROSODY BASELINE COMPILER)

Tự động:
1. Cào tin tức báo chí hiện đại (VnExpress, Tuổi Trẻ, Thanh Niên) qua RSS + Trafilatura full text.
2. Cào văn học thuần Việt kinh điển public domain từ Wikisource (Nam Cao, Thạch Lam).
3. Hợp nhất ngữ liệu và tính toán thống kê (mu, sigma, P95, P99) độc lập cho 4 ngữ vực:
   - Literary (Văn học / Tản văn)
   - Journalism (Báo chí thời sự)
   - Administrative (Văn bản pháp luật / Công vụ)
   - Technical (Kiến trúc kỹ thuật / Khoa học)
4. Tự động sinh file Rust nhúng `crates/vwi-core/src/rules/generated_prosody_baselines.rs`.
"""

import hashlib
import math
import re
import time
import unicodedata
import urllib.parse
from pathlib import Path

import feedparser
import requests
import trafilatura
from bs4 import BeautifulSoup

ROOT_DIR = Path(__file__).resolve().parent.parent
DATA_DIR = ROOT_DIR / "data_raw"
NEWS_DIR = DATA_DIR / "news_extra"
LIT_DIR = DATA_DIR / "literature_wikisource"
RUST_OUTPUT_FILE = ROOT_DIR / "crates" / "vwi-core" / "src" / "rules" / "generated_prosody_baselines.rs"

NEWS_DIR.mkdir(parents=True, exist_ok=True)
LIT_DIR.mkdir(parents=True, exist_ok=True)

HEADERS = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 (VWI-Research-Harvester/1.0)"
}

RSS_FEEDS = [
    # VnExpress
    "https://vnexpress.net/rss/tin-moi-nhat.rss",
    "https://vnexpress.net/rss/thoi-su.rss",
    "https://vnexpress.net/rss/kinh-doanh.rss",
    "https://vnexpress.net/rss/khoa-hoc.rss",
    "https://vnexpress.net/rss/so-hoa.rss",
    "https://vnexpress.net/rss/giao-duc.rss",
    # Tuổi Trẻ
    "https://tuoitre.vn/home.rss",
    "https://tuoitre.vn/rss/thoi-su.rss",
    "https://tuoitre.vn/rss/kinh-doanh.rss",
    # Thanh Niên
    "https://thanhnien.vn/rss/home.rss",
    "https://thanhnien.vn/rss/kinh-te.rss",
]

WIKISOURCE_TARGETS = [
    # Nam Cao
    "https://vi.wikisource.org/wiki/Ch%C3%AD_Ph%C3%A8o",
    "https://vi.wikisource.org/wiki/L%C3%A3o_H%E1%BA%A1c",
    "https://vi.wikisource.org/wiki/%C4%90%E1%BB%9Di_th%E1%BB%ABa",
    "https://vi.wikisource.org/wiki/N%E1%BB%A5_c%C6%B0%E1%BB%9Di_(Nam_Cao)",
    "https://vi.wikisource.org/wiki/Gi%C4%83ng_s%C3%A1ng",
    "https://vi.wikisource.org/wiki/%C4%90%C3%B4i_m%E1%BA%AFt",
    "https://vi.wikisource.org/wiki/M%E1%BB%99t_b%E1%BB%AFa_no",
    # Thạch Lam
    "https://vi.wikisource.org/wiki/Gi%C3%B3_%C4%91%E1%BA%A7u_m%C3%B9a",
    "https://vi.wikisource.org/wiki/Hai_%C4%91%E1%BB%A9a_tr%E1%BA%BB",
    "https://vi.wikisource.org/wiki/D%C6%B0%E1%BB%9Bi_b%C3%B3ng_ho%C3%A0ng_lan",
    "https://vi.wikisource.org/wiki/C%C3%B4_h%C3%A0ng_x%C3%A9n",
]

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

def harvest_journalism() -> str:
    print("\n" + "=" * 60)
    print(">>> 1. THU HOẠCH DỮ LIỆU BÁO CHÍ (VNEXPRESS, TUỔI TRẺ, THANH NIÊN)")
    print("=" * 60)

    seen = set()
    total_words = 0
    article_texts = []

    for feed_url in RSS_FEEDS:
        print(f"[*] Đang nạp feed: {feed_url}")
        try:
            feed = feedparser.parse(feed_url)
            entries = feed.entries[:20]  # Thu thập top bài mỗi feed
        except Exception as e:
            print(f"  [-] Lỗi đọc feed: {e}")
            continue

        for entry in entries:
            link = entry.get("link") or entry.get("id")
            if not link or link in seen:
                continue
            seen.add(link)

            title = entry.get("title", "").strip()
            summary = entry.get("summary", "") or entry.get("description", "")
            clean_summary = re.sub(r"<[^>]+>", " ", summary).strip()

            # Trích xuất toàn văn bằng trafilatura
            body = ""
            try:
                downloaded = trafilatura.fetch_url(link)
                if downloaded:
                    body = trafilatura.extract(downloaded, include_comments=False, include_tables=False) or ""
            except Exception:
                pass

            if len(body) < 150:
                body = clean_summary

            full_text = f"{title}\n\n{body}".strip()
            words_count = len(full_text.split())
            if words_count < 30:
                continue

            article_texts.append(full_text)
            total_words += words_count

            # Lưu file riêng biệt cho nghiên cứu
            h = hashlib.md5(link.encode()).hexdigest()[:10]
            out_file = NEWS_DIR / f"{h}.txt"
            out_file.write_text(full_text, encoding="utf-8")

        time.sleep(0.5)

    combined_news = "\n\n".join(article_texts)
    (DATA_DIR / "journalism_corpus.txt").write_text(combined_news, encoding="utf-8")
    print(f"[✓] Đã thu hoạch thành công: {len(article_texts)} bài báo ({total_words:,} từ)")
    return combined_news

def harvest_wikisource() -> str:
    print("\n" + "=" * 60)
    print(">>> 2. THU HOẠCH VĂN HỌC PUBLIC DOMAIN TỪ WIKISOURCE TIẾNG VIỆT")
    print("=" * 60)

    total_words = 0
    collected_texts = []

    for url in WIKISOURCE_TARGETS:
        work_name = urllib.parse.unquote(url.split("/")[-1])
        print(f"[*] Tác phẩm: {work_name}...")
        try:
            r = requests.get(url, headers=HEADERS, timeout=15)
            r.raise_for_status()
            soup = BeautifulSoup(r.text, "html.parser")

            # Xóa các thành phần giao diện Wikipedia
            for sel in [".mw-editsection", "sup", "table", ".reference", "#toc", ".ws-summary"]:
                for tag in soup.select(sel):
                    tag.decompose()

            content = soup.select_one("#mw-content-text") or soup.select_one(".mw-parser-output")
            if not content:
                print("  [-] Không tìm thấy thẻ nội dung chính")
                continue

            clean_text = content.get_text(separator="\n")
            lines = [ln.strip() for ln in clean_text.splitlines() if ln.strip()]
            full_text = "\n".join(lines)

            words = len(full_text.split())
            if words < 200:
                print(f"  [-] Nội dung quá ngắn ({words} từ), bỏ qua")
                continue

            slug = re.sub(r"[^\w]+", "_", work_name)[:50]
            out_path = LIT_DIR / f"{slug}.txt"
            out_path.write_text(full_text, encoding="utf-8")

            collected_texts.append(full_text)
            total_words += words
            print(f"  [+] Đã lưu {out_path.name} ({words:,} từ)")
            time.sleep(1.0)
        except Exception as e:
            print(f"  [-] Lỗi: {e}")

    # Ghép thêm vào human_gold.txt nếu có
    gold_file = DATA_DIR / "human_gold.txt"
    existing_gold = gold_file.read_text(encoding="utf-8") if gold_file.exists() else ""
    merged_gold = existing_gold + "\n\n" + "\n\n".join(collected_texts)
    print(f"[✓] Tổng từ văn học bổ sung từ Wikisource: {total_words:,} từ")
    print(f"[✓] Tổng kho văn học thuần Việt sau hợp nhất: {len(merged_gold.split()):,} từ")
    return merged_gold

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
    print(f"Tổng số từ            : {sum(sentence_lengths):,}")
    print(f"Độ dài câu trung bình : {mean_len:.2f} (±{std_len:.2f})")
    print(f"Tỷ lệ Trắc            : {mean_trac * 100:.2f}% (±{std_trac * 100:.2f}%)")
    print(f"Ngưỡng P95 Trắc       : {p95_trac_run:.1f} tiếng")
    print(f"Ngưỡng P99 Trắc       : {p99_trac_run:.1f} tiếng")

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
    print("    VWI EMPIRICAL MULTI-REGISTER COMPILER (DATA HARVESTER)")
    print("=" * 60)

    # 1. Thu hoạch Báo chí RSS
    journalism_text = harvest_journalism()
    journalism_stats = calculate_stats(journalism_text, "Journalism (VnExpress, Tuổi Trẻ, Thanh Niên)")

    # 2. Thu hoạch Văn học Wikisource & hợp nhất Human Gold
    literary_text = harvest_wikisource()
    literary_stats = calculate_stats(literary_text, "Literary (Cao Xuân Hạo, Nguyễn Hiến Lê, Nam Cao, Thạch Lam)")

    # 3. Tính toán Administrative & Technical
    admin_stats = calculate_stats(ADMINISTRATIVE_SAMPLE_CORPUS, "Administrative (Luật / Nghị định)")
    tech_stats = calculate_stats(TECHNICAL_SAMPLE_CORPUS, "Technical (Kiến trúc hệ thống)")

    # 4. Xuất mã nguồn Rust
    print(f"\n[*] Đang sinh mã nguồn Rust: {RUST_OUTPUT_FILE.name}...")
    with open(RUST_OUTPUT_FILE, "w", encoding="utf-8") as f:
        f.write("// FILE TỰ ĐỘNG SINH BỞI scripts/harvest_multi_source.py\n")
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

    print("[✓] Hoàn tất xuất ma trận Baseline vào Rust thành công!")

if __name__ == "__main__":
    main()
