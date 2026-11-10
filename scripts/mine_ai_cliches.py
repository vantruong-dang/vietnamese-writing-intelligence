#!/usr/bin/env python3
"""
KỊCH BẢN TỰ ĐỘNG KHAI PHÁ SÁO NGỮ AI (CONTRASTIVE N-GRAM MINING)
Sử dụng thuật toán Dunning's Log-Likelihood Ratio (G^2) & Relative Risk
để phát hiện "dấu vân tay" của AI so với văn phong người Việt bản ngữ (Cao Xuân Hạo & Nguyễn Hiến Lê).
Xuất mã nguồn Rust tĩnh nạp vào Trie DoubleArrayAhoCorasick trong O(N).
"""

from __future__ import annotations

import argparse
import math
import re
import sys
import unicodedata
from collections import Counter
from pathlib import Path

# Đảm bảo stdout hỗ trợ UTF-8 trên Windows
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

ROOT_DIR = Path(__file__).resolve().parent.parent
DATA_RAW_DIR = ROOT_DIR / "data_raw"
DEFAULT_OUTPUT_RUST_FILE = (
    ROOT_DIR / "crates" / "vwi-core" / "src" / "rules" / "generated_cliches.rs"
)
DEFAULT_HUMAN_GOLD_FILE = DATA_RAW_DIR / "human_gold.txt"
DEFAULT_AI_SYNTHETIC_FILE = DATA_RAW_DIR / "ai_synthetic.txt"

# Các đường dẫn tìm kiếm sách chuẩn mực (Google Drive hoặc local)
KNOWN_BOOK_PATHS = [
    Path(r"G:\My Drive\02. Reference\03.Book\03. Language\01.East\Tieng Viet van Viet nguoi Viet - Cao Xuan Hao.pdf"),
    Path(r"G:\My Drive\02. Reference\03.Book\06.Science\tu-hoc-mot-nhu-cau-thoi-dai-nguyen-hien-le.mobi"),
    Path(r"G:\My Drive\02. Reference\03.Book\07. East\01.Novel\Hoi Ki Nguyen Hien Le - KD - Nguyen Hien Le.mobi"),
]


def normalize_text(text: str) -> str:
    """Chuẩn hóa Unicode NFC và làm sạch khoảng trắng."""
    text = unicodedata.normalize("NFC", text.strip())
    # Giữ lại chữ cái tiếng Việt, số, khoảng trắng
    text = re.sub(r"[^\w\sà-ỹđÀ-ỸĐ]", " ", text)
    return re.sub(r"\s+", " ", text).strip().lower()


PROPER_NAME_BLACK_LIST = {
    "trang", "tử", "tuân", "hàn", "phi", "nho", "khổng", "lê", "hạo",
    "nguyễn", "cao", "tô", "thức", "thiên", "mã", "thao", "lão",
}

# Các cụm từ ngữ chuẩn mực của tiếng Việt học thuật hoặc đã thuộc về luật chuyên biệt khác
LEGITIMATE_COLLOCATIONS_WHITELIST = {
    "đóng vai trò",
    "chúng ta cần",
    "sự phát triển",
    "vai trò của",
    "quan trọng trong",
    "việc nghiên cứu",
    "dưới góc độ",
    "phát triển toàn diện",
    "chính là một",
    "tuyệt vời của",
    "nói tóm lại",
    "không thể phủ nhận rằng",
    "bức tranh toàn cảnh",
}


def build_proper_noun_entities(texts: list[str]) -> set[str]:
    """
    Trích xuất các danh từ riêng viết hoa liên tiếp ở giữa câu
    (như Trang Tử, Tuân Tử, Hàn Phi Tử, Cao Xuân Hạo, Nguyễn Hiến Lê, Khổng Tử, Tư Mã Thiên...).
    """
    entities = set()
    pn_pattern = re.compile(
        r"(?<=[a-zà-ỹđ0-9]\s)([A-ZÀ-ỸĐ][a-zà-ỹđ0-9]+(?:\s+[A-ZÀ-ỸĐ][a-zà-ỹđ0-9]+){1,2})\b"
    )
    common_stops = {"chúng ta", "việt nam"}  # Ngoại lệ nếu cần
    for line in texts:
        for m in pn_pattern.finditer(line):
            ent = m.group(1).strip().lower()
            if ent not in common_stops and len(ent.split()) >= 2:
                entities.add(ent)
    return entities


def contains_proper_noun(ngram_str: str, entities: set[str]) -> bool:
    """Kiểm tra xem cụm n-gram có chứa thực thể tên riêng hay không."""
    words = ngram_str.split()
    if any(w in PROPER_NAME_BLACK_LIST for w in words):
        return True
    for ent in entities:
        if ent in ngram_str:
            return True
    return False


def extract_ngrams(tokens: list[str], min_n: int = 3, max_n: int = 5) -> list[str]:
    """Trích xuất n-gram từ danh sách token."""
    ngrams = []
    num_tokens = len(tokens)
    for n in range(min_n, max_n + 1):
        for i in range(num_tokens - n + 1):
            ngram = " ".join(tokens[i : i + n])
            ngrams.append(ngram)
    return ngrams


def compute_log_likelihood(c12: int, c1: int, c2: int, n: int) -> float:
    """
    Dunning's Log-Likelihood Ratio (G^2):
    Chuẩn mực thống kê ngôn ngữ học để so sánh 2 corpus có quy mô khác nhau.
    Bảng tiếp liên 2x2:
      a: c12 (số lần xuất hiện n-gram trong corpus 1 - AI)
      b: c1 - c12 (các n-gram khác trong corpus 1)
      c: c2 (số lần xuất hiện n-gram trong corpus 2 - Human)
      d: n - a - b - c (các n-gram khác trong corpus 2)
    """
    a = float(c12)
    b = float(c1 - c12)
    c = float(c2)
    d = float(n - a - b - c)

    if n <= 0 or (a + b) <= 0 or (c + d) <= 0 or (a + c) <= 0 or (b + d) <= 0:
        return 0.0

    def ll_term(k: float, expected: float) -> float:
        if k <= 0.0 or expected <= 0.0:
            return 0.0
        return k * math.log(k / expected)

    e1 = (a + c) * (a + b) / n
    e2 = (b + d) * (a + b) / n
    e3 = (a + c) * (c + d) / n
    e4 = (b + d) * (c + d) / n

    g2 = 2.0 * (ll_term(a, e1) + ll_term(b, e2) + ll_term(c, e3) + ll_term(d, e4))
    return max(0.0, g2)


def mine_cliches(
    ai_texts: list[str],
    human_texts: list[str],
    min_count: int = 3,
    min_relative_risk: float = 3.0,
    min_g2: float = 10.83,  # p < 0.001
    top_k: int = 50,
    min_n: int = 3,
    max_n: int = 5,
) -> list[dict]:
    """Khai phá sáo ngữ tương phản giữa hai bể văn bản."""
    print("[*] Thu thập thực thể tên riêng để lọc nhiễu...")
    all_texts = ai_texts + human_texts
    entities = build_proper_noun_entities(all_texts)
    print(f"    -> Đã nhận diện {len(entities)} cụm tên riêng cần bảo vệ.")

    print(f"[*] Đang phân tích n-gram: {len(ai_texts)} đoạn AI vs {len(human_texts)} đoạn Người bản ngữ...")

    ai_ngrams = Counter()
    human_ngrams = Counter()

    total_ai_tokens = 0
    for text in ai_texts:
        tokens = normalize_text(text).split()
        total_ai_tokens += len(tokens)
        ai_ngrams.update(extract_ngrams(tokens, min_n=min_n, max_n=max_n))

    total_human_tokens = 0
    for text in human_texts:
        tokens = normalize_text(text).split()
        total_human_tokens += len(tokens)
        human_ngrams.update(extract_ngrams(tokens, min_n=min_n, max_n=max_n))

    total_all = total_ai_tokens + total_human_tokens
    print(f"    -> Tổng số token: AI={total_ai_tokens:,} | Human={total_human_tokens:,} (Tổng: {total_all:,})")

    candidates = []

    for ngram, count_ai in ai_ngrams.items():
        if count_ai < min_count:
            continue

        if any(white in ngram for white in LEGITIMATE_COLLOCATIONS_WHITELIST):
            continue

        # Bỏ qua nếu có chứa thực thể tên riêng
        if contains_proper_noun(ngram, entities):
            continue

        count_human = human_ngrams.get(ngram, 0)

        # Tính Relative Risk (Tần suất xuất hiện chuẩn hóa trong AI / trong Người)
        prob_ai = (count_ai + 1) / (total_ai_tokens + 1000)
        prob_human = (count_human + 1) / (total_human_tokens + 1000)
        relative_risk = prob_ai / prob_human

        if relative_risk >= min_relative_risk:
            g2 = compute_log_likelihood(count_ai, total_ai_tokens, count_human, total_all)
            if g2 >= min_g2:
                candidates.append({
                    "ngram": ngram,
                    "ai_count": count_ai,
                    "human_count": count_human,
                    "relative_risk": relative_risk,
                    "g2_score": g2,
                })

    # Sắp xếp theo G^2 (độ tin cậy thống kê) và Relative Risk
    candidates.sort(key=lambda x: (x["g2_score"], x["relative_risk"]), reverse=True)

    # Non-Maximal Suppression (NMS) cải tiến:
    pruned = []
    for cand in candidates:
        text = cand["ngram"]
        words_cand = text.split()
        is_redundant = False

        for other in candidates:
            if other == cand:
                continue
            other_text = other["ngram"]
            words_other = other_text.split()

            # 1. other ngắn hơn nằm trong cand, và other có tần suất AI vượt trội (>= 1.5x)
            # Ví dụ: other='đóng vai trò' (96 lần) vs cand='đóng vai trò quan trọng' (48 lần)
            if len(words_other) < len(words_cand) and other_text in text:
                if other["ai_count"] >= cand["ai_count"] * 1.5:
                    is_redundant = True
                    break

            # 2. other dài hơn chứa cand, và cand không có tần suất độc lập (chỉ là mảnh vụn)
            # Ví dụ: cand='không thể phủ nhận' (72) vs other='không thể phủ nhận rằng' (72)
            if len(words_other) > len(words_cand) and text in other_text:
                if cand["ai_count"] <= other["ai_count"] * 1.15:
                    is_redundant = True
                    break

        if not is_redundant:
            pruned.append(cand)

    return pruned[:top_k]


def extract_books_to_human_gold(out_file: Path) -> bool:
    """
    Trích xuất văn bản từ sách Cao Xuân Hạo (PDF) và Nguyễn Hiến Lê (MOBI/PDF)
    lưu thành tệp human_gold.txt tách biệt khỏi mã nguồn repo.
    """
    out_file.parent.mkdir(parents=True, exist_ok=True)
    all_lines: list[str] = []

    print("[*] Đang tìm kiếm các cuốn sách trong kho tham khảo...")

    # 1. Cao Xuân Hạo: Tiếng Việt văn Việt người Việt (PDF)
    cxh_path = KNOWN_BOOK_PATHS[0]
    if cxh_path.exists():
        print(f"    -> Đang trích xuất từ: {cxh_path.name}...")
        try:
            from pypdf import PdfReader
            reader = PdfReader(str(cxh_path))
            count_pages = 0
            for page in reader.pages:
                txt = page.extract_text()
                if txt:
                    for p in txt.split("\n\n"):
                        p_clean = re.sub(r"\s+", " ", p).strip()
                        if len(p_clean.split()) >= 6:
                            all_lines.append(p_clean)
                    count_pages += 1
            print(f"       + Đã đọc {count_pages} trang từ Cao Xuân Hạo.")
        except Exception as e:
            print(f"       [!] Lỗi khi đọc PDF Cao Xuân Hạo: {e}")

    # 2. Nguyễn Hiến Lê: Tự học một nhu cầu thời đại (MOBI)
    nhl_mobi = KNOWN_BOOK_PATHS[1]
    if nhl_mobi.exists():
        print(f"    -> Đang trích xuất từ: {nhl_mobi.name}...")
        try:
            import mobi
            from bs4 import BeautifulSoup
            extracted_path, _ = mobi.extract(str(nhl_mobi))
            for html_file in Path(extracted_path).rglob("*.html"):
                soup = BeautifulSoup(html_file.read_text(encoding="utf-8", errors="ignore"), "html.parser")
                text = soup.get_text(separator="\n")
                for line in text.splitlines():
                    line_clean = re.sub(r"\s+", " ", line).strip()
                    if len(line_clean.split()) >= 6:
                        all_lines.append(line_clean)
            print("       + Đã trích xuất xong từ sách Nguyễn Hiến Lê.")
        except Exception as e:
            print(f"       [!] Lỗi khi đọc MOBI Nguyễn Hiến Lê: {e}")

    # 3. Hồi ký Nguyễn Hiến Lê (MOBI) nếu có
    nhl_memoir = KNOWN_BOOK_PATHS[2]
    if nhl_memoir.exists():
        print(f"    -> Đang trích xuất từ: {nhl_memoir.name}...")
        try:
            import mobi
            from bs4 import BeautifulSoup
            extracted_path, _ = mobi.extract(str(nhl_memoir))
            for html_file in Path(extracted_path).rglob("*.html"):
                soup = BeautifulSoup(html_file.read_text(encoding="utf-8", errors="ignore"), "html.parser")
                text = soup.get_text(separator="\n")
                for line in text.splitlines():
                    line_clean = re.sub(r"\s+", " ", line).strip()
                    if len(line_clean.split()) >= 6:
                        all_lines.append(line_clean)
            print("       + Đã bổ sung văn liệu Hồi ký Nguyễn Hiến Lê.")
        except Exception as e:
            print(f"       [!] Lỗi khi đọc MOBI Hồi ký: {e}")

    if all_lines:
        with open(out_file, "w", encoding="utf-8") as f:
            for line in all_lines:
                f.write(line + "\n")
        total_words = sum(len(line.split()) for line in all_lines)
        print(f"[+] Đã ghi {len(all_lines):,} đoạn ({total_words:,} từ) vào {out_file}")
        return True
    return False


def get_default_human_corpus() -> list[str]:
    """Corpus người bản ngữ dự phòng nếu chưa có file."""
    return [
        "Mẹ tôi đi chợ từ lúc trời còn tờ mờ sáng để chọn mớ rau tươi con cá đồng.",
        "Lão Hạc thổi cái mồi rơm, rít một hơi thuốc lào sòng sọc rồi thở khói ra sân.",
        "Mùa thu Hà Nội có cái se lạnh rất riêng làm lòng người chùng xuống.",
        "Anh em trong nhà bảo ban nhau làm ăn lương thiện chứ đừng dối trên lừa dưới.",
        "Tôi ăn vội bát phở nóng bên hè phố rồi cuống cuồng chạy ra bến xe kẻo nhỡ chuyến.",
        "Cuốn sách bìa da sờn góc này là kỷ vật người bạn cũ để lại trước khi đi xa.",
        "Mưa rào mùa hạ trút xuống ào ào làm ngập cả đoạn đường ngõ trước cửa nhà.",
        "Sở dĩ như vậy là vì người xưa coi trọng cái gốc đạo đức hơn vẻ hào nhoáng bên ngoài.",
        "Ngặt nỗi lúc bấy giờ tiền nong eo hẹp, muốn sửa sang lại gian nhà tranh cũng khó.",
        "Rốt cuộc thì sự việc cũng ngã ngũ, ai nấy đều thở phào nhẹ nhõm quay về với việc đồng áng.",
        "Hóa ra cái lẽ đời chẳng qua chỉ là sự thành thật giữa người với người.",
        "Thành thử học sinh rời ghế nhà trường một là tưởng mình đã biết hết, hai là hoang mang lúng túng.",
        "Chẳng qua là anh ta nóng giận nhất thời chứ trong bụng không có ý xấu gì.",
        "Vốn dĩ văn phong tiếng Việt chuộng sự giản dị, mộc mạc mà lắng sâu.",
        "Lẽ ra chúng ta nên bình tâm lắng nghe nhau thay vì tranh cãi hơn thua.",
        "Không khỏi bùi ngùi khi nhớ lại những năm tháng hàn vi vất vả nơi xóm nghèo.",
        "Cơm nước xong xuôi thì lũ trẻ kéo nhau ra bờ ao ngắm trăng nghe dế gáy.",
        "Cho nên mới nói học vấn không phải là chiếc áo khoác để khoe mẽ với thiên hạ.",
    ] * 20


def get_default_ai_synthetic_corpus() -> list[str]:
    """Corpus văn bản AI tổng hợp (Synthetic AI) tràn ngập sáo ngữ RLHF."""
    return [
        "Không thể phủ nhận rằng công nghệ trí tuệ nhân tạo đang đóng vai trò vô cùng quan trọng trong việc thúc đẩy sự phát triển của xã hội hiện đại.",
        "Đây được xem là một chiếc chìa khóa vàng mở ra cánh cửa tri thức và đưa chúng ta tiến vào kỷ nguyên mới.",
        "Bức tranh toàn cảnh về nền kinh tế số đang mở ra những cơ hội to lớn song cũng tiềm ẩn không ít thách thức nan giải.",
        "Chúng ta cần tiến hành thực hiện các giải pháp một cách đồng bộ, quyết liệt và toàn diện nhằm nâng cao hiệu quả.",
        "Đây là một minh chứng rõ nét cho thấy những nỗ lực không ngừng nghỉ của cả tập thể trong suốt chặng đường vừa qua.",
        "Sự kết hợp hài hòa giữa yếu tố truyền thống và hiện đại đã tạo nên một bản giao hưởng tuyệt vời của sự đổi mới.",
        "Không chỉ dừng lại ở đó, việc trau dồi kỹ năng tự học còn đóng góp một phần không nhỏ vào thành công của mỗi cá nhân.",
        "Nói tóm lại, đứng trước bối cảnh toàn cầu hóa, chúng ta cần đặt nền móng vững chắc cho sự phát triển bền vững.",
        "Dưới góc độ nhìn nhận từ nhiều khía cạnh khác nhau, có thể thấy đây là một bước ngoặt lịch sử mang tính đột phá.",
        "Nó là vô cùng quan trọng để các bạn trẻ nhận thức được trách nhiệm của mình đối với tương lai đất nước.",
        "Trong bối cảnh bùng nổ thông tin hiện nay, việc định hướng tư duy phản biện đóng vai trò then chốt hơn bao giờ hết.",
        "Tóm lại, bài viết đã cung cấp một cái nhìn sâu sắc và toàn diện về vấn đề nan giải này.",
        "Không thể phủ nhận rằng sự phát triển vượt bậc của công nghệ đã làm thay đổi hoàn toàn cách chúng ta tương tác.",
        "Một trong những yếu tố đóng vai trò then chốt chính là khả năng thích ứng linh hoạt trước mọi biến động.",
        "Bức tranh toàn cảnh về thị trường lao động đòi hỏi mỗi người phải liên tục nâng cao trình độ chuyên môn.",
        "Điều này là một minh chứng rõ nét khẳng định vị thế tiên phong của tổ chức trong khu vực.",
        "Chúng ta cần chung tay tiến hành thực hiện các cam kết quốc tế một cách nghiêm túc và hiệu quả.",
        "Sự kết hợp hài hòa giữa lý thuyết và thực tiễn sẽ là chìa khóa vạn năng giúp giải quyết mọi vướng mắc.",
        "Không chỉ dừng lại ở đó, sáng kiến này còn mở ra một chương mới đầy hứa hẹn cho thế hệ tương lai.",
        "Đóng vai trò quan trọng trong việc định hình nhận thức cộng đồng và lan tỏa các giá trị nhân văn sâu sắc.",
    ] * 20


def export_to_rust(cliches: list[dict], out_path: Path):
    """Xuất danh sách sáo ngữ đã khai phá thành mã nguồn Rust tĩnh."""
    out_path.parent.mkdir(parents=True, exist_ok=True)
    print(f"[*] Xuất mã nguồn Rust sang: {out_path}...")
    with open(out_path, "w", encoding="utf-8") as f:
        f.write("// FILE TỰ ĐỘNG SINH BỞI scripts/mine_ai_cliches.py\n")
        f.write("// KHÔNG SỬA TAY. Được trích xuất bằng thuật toán Contrastive G^2 Mining & Relative Risk.\n")
        f.write("// Tách biệt hoàn toàn khỏi mã nguồn ngữ liệu gốc (Gold Standard).\n\n")
        f.write("/// Danh sách các sáo ngữ AI được khai phá tự động bằng thuật toán thống kê tương phản\n")
        f.write("pub static DISCOVERED_AI_CLICHES: &[&str] = &[\n")
        for c in cliches:
            ngram = c["ngram"]
            rr = c["relative_risk"]
            g2 = c["g2_score"]
            ai_c = c["ai_count"]
            h_c = c["human_count"]
            f.write(f'    "{ngram}", // G2: {g2:.1f}, RR: {rr:.1f}x (AI: {ai_c}, Human: {h_c})\n')
        f.write("];\n")
    print(f"[+] Đã ghi nhận {len(cliches)} cụm sáo ngữ AI vào lõi Rust!")


def main():
    parser = argparse.ArgumentParser(
        description="Tự động khai phá sáo ngữ AI bằng thuật toán Thống kê Tương phản (Contrastive G^2 Mining)."
    )
    parser.add_argument(
        "--min-count", type=int, default=3, help="Tần suất xuất hiện tối thiểu trong AI corpus (mặc định: 3)"
    )
    parser.add_argument(
        "--min-n", type=int, default=3, help="Độ dài n-gram tối thiểu (mặc định: 3)"
    )
    parser.add_argument(
        "--max-n", type=int, default=5, help="Độ dài n-gram tối đa (mặc định: 5)"
    )
    parser.add_argument(
        "--min-rr", type=float, default=3.0, help="Tỷ lệ vượt trội Relative Risk tối thiểu (mặc định: 3.0x)"
    )
    parser.add_argument(
        "--min-g2", type=float, default=10.83, help="Điểm Dunning G^2 tối thiểu (10.83 tương ứng p < 0.001)"
    )
    parser.add_argument(
        "--top-k", type=int, default=50, help="Số lượng sáo ngữ tối đa cần xuất (mặc định: 50)"
    )
    parser.add_argument(
        "--ai-corpus", type=str, default=str(DEFAULT_AI_SYNTHETIC_FILE), help="Đường dẫn file văn bản AI"
    )
    parser.add_argument(
        "--human-corpus", type=str, default=str(DEFAULT_HUMAN_GOLD_FILE), help="Đường dẫn file văn bản Human Gold"
    )
    parser.add_argument(
        "--extract-books", action="store_true", help="Trích xuất tự động từ sách PDF/MOBI Cao Xuân Hạo & Nguyễn Hiến Lê"
    )
    parser.add_argument(
        "--out-rust", type=str, default=str(DEFAULT_OUTPUT_RUST_FILE), help="Đường dẫn file Rust xuất ra"
    )

    args = parser.parse_args()

    human_file = Path(args.human_corpus)
    ai_file = Path(args.ai_corpus)
    rust_out = Path(args.out_rust)

    # 1. Trích xuất hoặc nạp Human Gold Corpus
    if args.extract_books or not human_file.exists():
        print(f"[*] Kiểm tra tệp ngữ liệu người bản ngữ: {human_file}...")
        extracted = extract_books_to_human_gold(human_file)
        if not extracted and not human_file.exists():
            print("[-] Không tìm thấy file sách ngoài ổ đĩa, sử dụng bộ mẫu Human Gold chuẩn mực sẵn có.")

    human_texts: list[str] = []
    if human_file.exists():
        with open(human_file, encoding="utf-8") as f:
            human_texts = [line.strip() for line in f if len(line.strip().split()) >= 4]
        print(f"[+] Đã nạp {len(human_texts):,} câu từ Human Gold Corpus ({human_file.name}).")
    else:
        human_texts = get_default_human_corpus()
        print(f"[*] Sử dụng {len(human_texts)} đoạn mẫu Human Gold Corpus.")

    # 2. Nạp hoặc chuẩn bị Synthetic AI Corpus
    ai_texts: list[str] = []
    if ai_file.exists():
        with open(ai_file, encoding="utf-8") as f:
            ai_texts = [line.strip() for line in f if len(line.strip().split()) >= 4]
        print(f"[+] Đã nạp {len(ai_texts):,} câu từ Synthetic AI Corpus ({ai_file.name}).")
    else:
        ai_texts = get_default_ai_synthetic_corpus()
        # Lưu lại mẫu để người dùng có thể mở rộng
        ai_file.parent.mkdir(parents=True, exist_ok=True)
        with open(ai_file, "w", encoding="utf-8") as f:
            for line in ai_texts:
                f.write(line + "\n")
        print(f"[*] Đã tạo tệp mẫu Synthetic AI Corpus tại {ai_file}.")

    # 3. Tiến hành khai phá sáo ngữ tương phản
    results = mine_cliches(
        ai_texts=ai_texts,
        human_texts=human_texts,
        min_count=args.min_count,
        min_relative_risk=args.min_rr,
        min_g2=args.min_g2,
        top_k=args.top_k,
        min_n=args.min_n,
        max_n=args.max_n,
    )

    print("\n" + "=" * 65)
    print("TOP SÁO NGỮ AI KHAI PHÁ ĐƯỢC (CONTRASTIVE G^2 & RELATIVE RISK)")
    print("=" * 65)
    for idx, r in enumerate(results[:30], 1):
        print(
            f"{idx:2d}. '{r['ngram']}' "
            f"| G2: {r['g2_score']:6.1f} "
            f"| Tỷ lệ AI/Human: {r['relative_risk']:4.1f}x "
            f"(AI: {r['ai_count']:3d}, Human: {r['human_count']:2d})"
        )
    print("=" * 65)

    # 4. Xuất sang Rust
    export_to_rust(results, rust_out)


if __name__ == "__main__":
    main()
