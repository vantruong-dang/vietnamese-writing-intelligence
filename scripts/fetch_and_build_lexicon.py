"""Kịch bản khai phá dữ liệu từ vựng tiếng Việt (Vietnamese Lexicon Pipeline).

Thu thập, làm sạch và chuẩn hóa danh mục từ ghép tiếng Việt từ các nguồn uy tín:
- Viet39K & Viet74K (Hồ Ngọc Đức corpus via duyet/vietnamese-wordlist)
- Universal Dependencies Vietnamese Treebank (UD_Vietnamese-VTB)
- Bộ từ vựng nội tại của Vietnamese Writing Intelligence
"""

from __future__ import annotations

import re
import shutil
import sys
import unicodedata
import urllib.request
from pathlib import Path

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")

ROOT_DIR = Path(__file__).resolve().parent.parent
RAW_OUTPUT = ROOT_DIR / "data_raw" / "lexicon_dictionary.txt"
CRATE_DATA_DIR = ROOT_DIR / "crates" / "vwi-core" / "data"
CRATE_OUTPUT = CRATE_DATA_DIR / "lexicon_dictionary.txt"

# Các nguồn dữ liệu mã nguồn mở uy tín
URLS = {
    "viet39k": "https://raw.githubusercontent.com/duyet/vietnamese-wordlist/master/Viet39K.txt",
    "viet74k": "https://raw.githubusercontent.com/duyet/vietnamese-wordlist/master/Viet74K.txt",
    "vtb_train": "https://raw.githubusercontent.com/UniversalDependencies/UD_Vietnamese-VTB/master/vi_vtb-ud-train.conllu",
    "vtb_dev": "https://raw.githubusercontent.com/UniversalDependencies/UD_Vietnamese-VTB/master/vi_vtb-ud-dev.conllu",
    "vtb_test": "https://raw.githubusercontent.com/UniversalDependencies/UD_Vietnamese-VTB/master/vi_vtb-ud-test.conllu",
}

# Regex kiểm tra ký tự tiếng Việt hợp lệ (chữ cái có dấu, khoảng trắng, gạch nối)
VIETNAMESE_WORD_RE = re.compile(
    r"^[a-zA-Z\s\-àáảãạăằắẳẵặâầấẩẫậèéẻẽẹêềếểễệđìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵ]+$",
    re.IGNORECASE,
)


def fetch_url(url: str, timeout: int = 15) -> str:
    """Tải nội dung từ URL với User-Agent chuẩn."""
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64)"})
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return resp.read().decode("utf-8", errors="replace")


def clean_word(word: str) -> str | None:
    """Làm sạch, chuẩn hóa NFC và kiểm tra tính hợp lệ của một từ/từ ghép."""
    word = unicodedata.normalize("NFC", word).strip().lower()
    word = re.sub(r"\s+", " ", word)

    if not word or len(word) < 2:
        return None

    # Chỉ giữ từ ghép có từ 2 đến 5 âm tiết (phục vụ tách từ tối ưu)
    syllables = word.split()
    if len(syllables) < 2 or len(syllables) > 5:
        return None

    # Loại bỏ từ chứa ký tự số hoặc biểu tượng đặc biệt
    if not VIETNAMESE_WORD_RE.match(word):
        return None

    return word


def extract_from_conllu(content: str) -> set[str]:
    """Trích xuất từ ghép tiếng Việt từ file CoNLL-U của UD-VTB."""
    words: set[str] = set()
    for line in content.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) > 1:
            raw_word = parts[1]
            if "_" in raw_word or " " in raw_word:
                cleaned = clean_word(raw_word.replace("_", " "))
                if cleaned:
                    words.add(cleaned)
    return words


def build_lexicon() -> None:
    print("=" * 70)
    print("🚀 BẮT ĐẦU PIPELINE KHAI PHÁ DỮ LIỆU TỪ ĐIỂN TIẾNG VIỆT (LEXICON MINING)")
    print("=" * 70)

    all_compounds: set[str] = set()

    # 1. Nạp từ vựng cơ sở nội tại của vn_writing_intel
    try:
        from vn_writing_intel.linguistics.segmentation import DEFAULT_COMPOUND_VOCAB

        for w in DEFAULT_COMPOUND_VOCAB:
            c = clean_word(w)
            if c:
                all_compounds.add(c)
        print(f"✅ Đã nạp {len(all_compounds)} từ gốc từ DEFAULT_COMPOUND_VOCAB")
    except Exception as e:
        print(f"⚠️ Không nạp được DEFAULT_COMPOUND_VOCAB: {e}")

    # 2. Thu thập từ Viet39K & Viet74K
    for key in ["viet39k", "viet74k"]:
        url = URLS[key]
        print(f"📥 Đang tải {key.upper()} từ: {url} ...")
        try:
            content = fetch_url(url)
            count = 0
            for line in content.splitlines():
                c = clean_word(line)
                if c:
                    all_compounds.add(c)
                    count += 1
            print(f"   -> Thu được {count:,} từ ghép hợp lệ từ {key}")
        except Exception as e:
            print(f"   ❌ Lỗi khi tải {key}: {e}")

    # 3. Thu thập từ Universal Dependencies (UD_Vietnamese-VTB)
    for key in ["vtb_train", "vtb_dev", "vtb_test"]:
        url = URLS[key]
        print(f"📥 Đang tải CoNLL-U ({key}) từ: {url} ...")
        try:
            content = fetch_url(url)
            vtb_words = extract_from_conllu(content)
            all_compounds.update(vtb_words)
            print(f"   -> Thu được {len(vtb_words):,} từ ghép ngữ pháp từ {key}")
        except Exception as e:
            print(f"   ❌ Lỗi khi tải {key}: {e}")

    # 4. Sắp xếp danh mục theo thứ tự Unicode
    sorted_words = sorted(all_compounds)
    print("-" * 70)
    print(f"🎉 TỔNG SỐ TỪ GHÉP ĐÃ KHAI PHÁ & LÀM SẠCH: {len(sorted_words):,} từ")

    # 5. Lưu ra các đường dẫn mục tiêu
    RAW_OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    with open(RAW_OUTPUT, "w", encoding="utf-8") as f:
        for w in sorted_words:
            f.write(f"{w}\n")
    print(f"💾 Đã ghi file từ điển: {RAW_OUTPUT}")

    CRATE_DATA_DIR.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(RAW_OUTPUT, CRATE_OUTPUT)
    print(f"🚀 Đã copy trực tiếp vào Rust Crate: {CRATE_OUTPUT}")
    print("=" * 70)


if __name__ == "__main__":
    build_lexicon()
