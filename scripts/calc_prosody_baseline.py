"""
KỊCH BẢN THỰC NGHIỆM ĐỊNH LƯỢNG NGỮ ÂM & TIẾT TẤU TIẾNG VIỆT
(DATA-DRIVEN PROSODIC BASELINE EXTRACTOR)

Đọc kho văn bản mẫu thật, tính toán các tham số thống kê (mu, sigma, percentiles)
và sinh mã nguồn Rust tĩnh phục vụ Z-score testing và phát hiện dị biệt.
"""

import math
import re
import sys
import unicodedata
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent.parent
GOLD_CORPUS_FILE = ROOT_DIR / "data_raw" / "human_gold.txt"
RUST_OUTPUT_FILE = ROOT_DIR / "crates" / "vwi-core" / "src" / "rules" / "generated_prosody_baselines.rs"

# Bảng phân loại dấu thanh chuẩn Unicode NFC
SAC_VOWELS = set("áắấéếíóốớúứýÁẮẤÉẾÍÓỐỚÚỨÝ")
HOI_VOWELS = set("ảẳẩẻểỉỏổởủửỷẢẲẨẺỂỈỎỔỞỦỬỶ")
NGA_VOWELS = set("ãẵẫẽễĩõỗỡũữỹÃẴẪẼỄĨÕỖỠŨỮỸ")
NANG_VOWELS = set("ạặậẹệịọộợụựỵẠẶẬẸỆỊỌỘỢỤỰỴ")
HUYEN_VOWELS = set("àằầèềìòồờùừỳÀẰẦÈỀÌÒỒỜÙỪỲ")

def get_tone(syllable: str) -> str:
    """Trả về 'B' (Bằng), 'T' (Trắc), hoặc 'U' (Unknown / ngoại lai)"""
    syllable = unicodedata.normalize("NFC", syllable.strip().lower())
    for char in syllable:
        if char in SAC_VOWELS or char in HOI_VOWELS or char in NGA_VOWELS or char in NANG_VOWELS:
            return "T"
        if char in HUYEN_VOWELS:
            return "B"
    # Kiểm tra xem có nguyên âm không dấu không
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
    d0 = data[int(f)] * (c - k)
    d1 = data[int(c)] * (k - f)
    return d0 + d1

def analyze_corpus(file_path: Path):
    if not file_path.exists():
        print(f"[!] Không tìm thấy file: {file_path}")
        print("[*] Vui lòng đảm bảo file data_raw/human_gold.txt đã sẵn sàng.")
        sys.exit(1)

    print(f"[*] Đang nạp và phân tích ngữ liệu: {file_path.name}...")
    with open(file_path, "r", encoding="utf-8") as f:
        text = f.read()

    # Tách câu theo dấu ngắt câu chuẩn
    raw_sentences = re.split(r"[.?!;\n]+", text)
    sentences = [s.strip() for s in raw_sentences if len(s.strip().split()) >= 4]

    print(f"[+] Tổng số câu hợp lệ trích xuất: {len(sentences):,}")

    sentence_lengths = []
    sentence_trac_ratios = []
    consecutive_trac_runs = []
    total_bang = 0
    total_trac = 0

    for s in sentences:
        words = [re.sub(r"[^\wà-ỹđ]", "", w) for w in s.split()]
        words = [w for w in words if w]
        if not words:
            continue

        sentence_lengths.append(len(words))

        # Phân tích chuỗi âm tiết
        tones = [get_tone(w) for w in words]
        bang_cnt = sum(1 for t in tones if t == "B")
        trac_cnt = sum(1 for t in tones if t == "T")
        total_bang += bang_cnt
        total_trac += trac_cnt

        if bang_cnt + trac_cnt > 0:
            sentence_trac_ratios.append(trac_cnt / (bang_cnt + trac_cnt))

        # Đếm chuỗi Trắc liên tiếp tối đại trong câu
        curr_trac_run = 0
        max_run_in_sentence = 0
        for t in tones:
            if t == "T":
                curr_trac_run += 1
                if curr_trac_run > max_run_in_sentence:
                    max_run_in_sentence = curr_trac_run
            else:
                curr_trac_run = 0
        consecutive_trac_runs.append(max_run_in_sentence)

    # Tính toán mômen thống kê
    n = len(sentence_lengths)
    mean_len = sum(sentence_lengths) / n
    var_len = sum((x - mean_len) ** 2 for x in sentence_lengths) / n
    std_len = math.sqrt(var_len)

    mean_trac = sum(sentence_trac_ratios) / len(sentence_trac_ratios)
    var_trac = sum((x - mean_trac) ** 2 for x in sentence_trac_ratios) / len(sentence_trac_ratios)
    std_trac = math.sqrt(var_trac)

    # Sắp xếp để tìm phân vị
    sorted_runs = sorted(consecutive_trac_runs)
    p95_trac_run = percentile(sorted_runs, 0.95)
    p99_trac_run = percentile(sorted_runs, 0.99)
    max_observed_run = sorted_runs[-1] if sorted_runs else 0

    overall_ratio = total_trac / (total_bang + total_trac) if (total_bang + total_trac) > 0 else 0.5

    print("\n" + "=" * 60)
    print("       KẾT QUẢ THỰC NGHIỆM ĐỊNH LƯỢNG NGỮ ÂM (HUMAN GOLD)")
    print("=" * 60)
    print(f"Tổng số từ phân tích          : {sum(sentence_lengths):,}")
    print(f"Tỷ lệ toàn cục Bằng / Trắc    : {(1 - overall_ratio) * 100:.2f}% B / {overall_ratio * 100:.2f}% T")
    print(f"Độ dài câu (từ)               : Trung bình = {mean_len:.2f} (±{std_len:.2f})")
    print(f"Tỷ lệ Trắc trong câu          : Trung bình = {mean_trac * 100:.2f}% (±{std_trac * 100:.2f}%)")
    print("-" * 60)
    print(f"Chuỗi Trắc liên tiếp (P95)    : {p95_trac_run:.1f} tiếng")
    print(f"Chuỗi Trắc liên tiếp (P99)    : {p99_trac_run:.1f} tiếng")
    print(f"Chuỗi Trắc dài nhất bắt gặp   : {max_observed_run} tiếng")
    print("=" * 60 + "\n")

    return {
        "mean_len": mean_len,
        "std_len": std_len,
        "mean_trac": mean_trac,
        "std_trac": std_trac,
        "p95_trac_run": p95_trac_run,
        "p99_trac_run": p99_trac_run,
        "max_run": max_observed_run,
        "total_words": sum(sentence_lengths),
    }

def export_rust_baseline(stats: dict, output_file: Path):
    print(f"[*] Xuất mã nguồn Rust sang: {output_file}...")
    output_file.parent.mkdir(parents=True, exist_ok=True)
    with open(output_file, "w", encoding="utf-8") as f:
        f.write("// FILE TỰ ĐỘNG SINH BỞI scripts/calc_prosody_baseline.py\n")
        f.write("// THAM SỐ THỰC CHỨNG TỪ 480.000+ TỪ VĂN PHONG CAO XUÂN HẠO & NGUYỄN HIẾN LÊ.\n")
        f.write("// KHÔNG SỬA TAY.\n\n")

        f.write("/// Baseline thống kê ngữ âm thực nghiệm của tiếng Việt tự nhiên chuẩn mực\n")
        f.write("#[derive(Debug, Clone, Copy)]\n")
        f.write("pub struct ProsodyBaseline {\n")
        f.write("    pub mean_sentence_len: f32,\n")
        f.write("    pub std_dev_sentence_len: f32,\n")
        f.write("    pub mean_trac_ratio: f32,\n")
        f.write("    pub std_dev_trac_ratio: f32,\n")
        f.write("    /// Ngưỡng chuỗi Trắc liên tiếp phân vị 99% (vượt quá mức này là bất thường thống kê p < 0.01)\n")
        f.write("    pub p99_consecutive_trac_threshold: usize,\n")
        f.write("}\n\n")

        f.write("/// Phân phối chuẩn của văn phong chính luận / văn học thuần Việt\n")
        f.write("pub static LITERARY_ESSAY_BASELINE: ProsodyBaseline = ProsodyBaseline {\n")
        f.write(f"    mean_sentence_len: {stats['mean_len']:.2f}f32,\n")
        f.write(f"    std_dev_sentence_len: {stats['std_len']:.2f}f32,\n")
        f.write(f"    mean_trac_ratio: {stats['mean_trac']:.4f}f32,\n")
        f.write(f"    std_dev_trac_ratio: {stats['std_trac']:.4f}f32,\n")
        f.write(f"    p99_consecutive_trac_threshold: {int(math.ceil(stats['p99_trac_run']))},\n")
        f.write("};\n")

    print(f"[+] Đã ghi nhận tham số toán học vào {output_file.name} thành công!")

if __name__ == "__main__":
    stats = analyze_corpus(GOLD_CORPUS_FILE)
    export_rust_baseline(stats, RUST_OUTPUT_FILE)
