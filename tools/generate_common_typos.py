"""Sinh bảng ánh xạ lỗi chính tả tiếng Việt phổ biến sang Rust static slice.

Triết lý thiết kế (vì sao KHÔNG sinh mù từ Viet74K):
- Thử nghiệm "Controlled Phonetic Mutation" thuần túy trên 48.465 từ ghép 2 âm tiết
  sinh ra ~51.000 cặp "dị dạng" (lhất đẳng, thậc là, nướt...). Phần lớn không phải
  lỗi người Việt thực sự mắc, lại có nguy cơ bắt oan văn bản sạch.
- Vì vậy nguồn chân lý là SEED_PAIRS: các cặp đã được kiểm duyệt thủ công theo
  Sổ tay chính tả tiếng Việt / Từ điển chính tả. Script dùng Viet74K để VALIDATE
  hai chiều:
    + từ đúng PHẢI có trong từ điển,
    + từ sai KHÔNG được là một mục từ hợp lệ (chống nhập nhằng đồng âm khác nghĩa),
    + từ sai KHÔNG được là tiền tố của một cụm dài hơn hợp lệ
      ("thăm gia" trong "thăm gia đình", "chuyền tay" trong "chuyền tay nhau").
- Chạy `python tools/generate_common_typos.py --check` để kiểm tra file Rust còn đồng bộ.

Output: crates/vwi-core/src/rules/common_typos_data.rs
"""

from __future__ import annotations

import argparse
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEXICON_PATH = ROOT / "data_raw" / "Viet74K.txt"
RUST_OUT_PATH = ROOT / "crates" / "vwi-core" / "src" / "rules" / "common_typos_data.rs"

# fmt: off
# (sai, đúng) — đã kiểm duyệt thủ công. Chỉ đưa vào các trường hợp:
#   1) từ sai không phải là một từ tiếng Việt hợp lệ, hoặc
#   2) cụm sai là collocation rõ ràng sai ngữ cảnh (dành giật -> giành giật).
# Tuyệt đối không đưa các cặp "cả hai đều đúng nhưng khác nghĩa" (bàng quan/bàng quang,
# dành dụm/giành giật, trí thức/tri thức, nhận chức/nhậm chức...).
SEED_PAIRS: list[tuple[str, str]] = [
    # ── S / X ────────────────────────────────────────────────────────────────
    ("sử lý", "xử lý"),
    ("sáng lạn", "xán lạn"),
    ("sáng lạng", "xán lạn"),
    ("xán lạng", "xán lạn"),
    ("sát nhập", "sáp nhập"),
    ("suất sắc", "xuất sắc"),
    ("suất xắc", "xuất sắc"),
    ("xuất xắc", "xuất sắc"),
    ("sơ xuất", "sơ suất"),
    ("xơ xuất", "sơ suất"),
    ("xơ suất", "sơ suất"),
    ("sản suất", "sản xuất"),
    ("xửa chữa", "sửa chữa"),
    ("xữa chữa", "sửa chữa"),
    ("xửa đổi", "sửa đổi"),
    ("xửa lỗi", "sửa lỗi"),
    ("xữa lỗi", "sửa lỗi"),
    ("sữa chữa", "sửa chữa"),
    ("sữa đổi", "sửa đổi"),
    ("sữa sai", "sửa sai"),
    ("sữa lỗi", "sửa lỗi"),
    ("xạch sẽ", "sạch sẽ"),
    ("sạch xẽ", "sạch sẽ"),
    ("sạch sẻ", "sạch sẽ"),
    ("xôi nổi", "sôi nổi"),
    ("xáng suốt", "sáng suốt"),
    ("suề sòa", "xuề xòa"),
    ("soay sở", "xoay sở"),
    ("xan xẻ", "san sẻ"),
    ("sót xa", "xót xa"),
    ("xắp sếp", "sắp xếp"),
    ("sắp sếp", "sắp xếp"),
    ("xuất sứ", "xuất xứ"),
    ("xứ mệnh", "sứ mệnh"),
    ("xung sướng", "sung sướng"),
    ("xung túc", "sung túc"),
    ("sung phong", "xung phong"),
    ("sung đột", "xung đột"),
    ("sung quanh", "xung quanh"),
    ("sung khắc", "xung khắc"),
    ("sung kích", "xung kích"),
    ("sung lực", "xung lực"),
    ("xung mãn", "sung mãn"),
    ("xung sức", "sung sức"),
    ("sử sự", "xử sự"),
    ("sử thế", "xử thế"),
    ("sử án", "xử án"),
    ("sử phạt", "xử phạt"),
    ("sử tội", "xử tội"),
    ("sử trí", "xử trí"),
    ("sét xử", "xét xử"),
    ("sét nghiệm", "xét nghiệm"),
    ("sét tuyển", "xét tuyển"),
    ("sét nét", "xét nét"),
    ("sét lại", "xét lại"),
    ("sét thấy", "xét thấy"),
    ("sét đoán", "xét đoán"),
    ("sét hỏi", "xét hỏi"),
    ("sét về", "xét về"),
    ("sao xuyến", "xao xuyến"),
    ("sôn sao", "xôn xao"),
    ("sốn sang", "xốn xang"),
    ("suyến sao", "xuyến xao"),
    ("xai lệch", "sai lệch"),
    ("xai sót", "sai sót"),
    ("xai trái", "sai trái"),
    ("xáng sủa", "sáng sủa"),
    ("xáng chói", "sáng chói"),
    ("xương sớm", "sương sớm"),
    ("xương mù", "sương mù"),
    ("sông xáo", "xông xáo"),
    ("sông pha", "xông pha"),
    ("suất trình", "xuất trình"),
    ("suất khẩu", "xuất khẩu"),
    # KHÔNG đưa "suất hiện"/"suất phát"/"suất nhập"/"đề suất" vào: chúng là
    # chuỗi từ hợp lệ trong "công suất hiện tại", "công suất phát điện",
    # "công suất nhập khẩu", "đề suất ăn" (đề + suất).
    ("suất bản", "xuất bản"),
    ("suất cảnh", "xuất cảnh"),
    ("đề bạc", "đề bạt"),
    ("đề nghỉ", "đề nghị"),
    ("đề đạc", "đề đạt"),
    ("xúc tích", "súc tích"),
    ("súc tiến", "xúc tiến"),
    ("súc tác", "xúc tác"),
    ("súc sắc", "xúc xắc"),
    ("súc phạm", "xúc phạm"),
    ("súc động", "xúc động"),
    ("sí nghiệp", "xí nghiệp"),
    ("sí xóa", "xí xóa"),
    ("sì xào", "xì xào"),
    ("sì hơi", "xì hơi"),
    ("sỉ vả", "xỉ vả"),
    ("sỉ nhục", "xỉ nhục"),
    ("sỉ than", "xỉ than"),
    ("sỉu", "xỉu"),
    ("sỉn", "xỉn"),
    ("sỉa", "xỉa"),
    ("súm xít", "xúm xít"),
    ("súm lại", "xúm lại"),

    # ── TR / CH ──────────────────────────────────────────────────────────────
    ("trân thành", "chân thành"),
    ("chân trọng", "trân trọng"),
    ("chính sác", "chính xác"),
    ("trung thủy", "chung thủy"),
    ("chặc chẽ", "chặt chẽ"),
    ("trân thật", "chân thật"),
    ("chọn vẹn", "trọn vẹn"),
    ("tre chở", "che chở"),
    ("chở ngại", "trở ngại"),
    ("truyển biến", "chuyển biến"),
    ("chuyền thống", "truyền thống"),
    ("chăn trở", "trăn trở"),
    ("trăn nuôi", "chăn nuôi"),
    ("trấp nhận", "chấp nhận"),
    ("trứng minh", "chứng minh"),
    # KHÔNG đưa "trứng từ" vào: "trứng từ ..." là chuỗi hợp lệ (trứng từ trại).
    ("trế độ", "chế độ"),
    ("trủ yếu", "chủ yếu"),
    ("chách nhiệm", "trách nhiệm"),
    ("chung thành", "trung thành"),
    ("chung tâm", "trung tâm"),
    ("tập chung", "tập trung"),
    ("trung quy", "chung quy"),
    ("trung sống", "chung sống"),
    ("trung tay", "chung tay"),
    ("trung vui", "chung vui"),
    ("truyện trò", "chuyện trò"),
    ("kể truyện", "kể chuyện"),
    ("chuyền đạt", "truyền đạt"),
    ("chuyền thông", "truyền thông"),
    ("chuyền hình", "truyền hình"),
    ("chuyền nhiễm", "truyền nhiễm"),
    ("truyển đổi", "chuyển đổi"),
    ("truyển nhượng", "chuyển nhượng"),
    ("truyển giao", "chuyển giao"),
    ("truyển khoản", "chuyển khoản"),
    ("truyển phát", "chuyển phát"),
    ("truyển động", "chuyển động"),
    ("truyển hóa", "chuyển hóa"),
    ("truyển mình", "chuyển mình"),
    ("truyển sang", "chuyển sang"),
    ("truyển tiếp", "chuyển tiếp"),
    ("truyển vùng", "chuyển vùng"),
    ("truyển dịch", "chuyển dịch"),
    ("truyển nhà", "chuyển nhà"),
    ("truyển tiền", "chuyển tiền"),
    ("truyển hàng", "chuyển hàng"),
    ("truyển hướng", "chuyển hướng"),
    ("chuyền bệnh", "truyền bệnh"),
    ("chuyền lời", "truyền lời"),
    ("chuyền nghề", "truyền nghề"),
    ("chuyền tay", "truyền tay"),
    ("chuyền miệng", "truyền miệng"),
    ("chuyền tai", "truyền tai"),
    ("chuyền cảm hứng", "truyền cảm hứng"),
    ("chuyền lửa", "truyền lửa"),

    # ── D / GI / R / V ───────────────────────────────────────────────────────
    ("dữ gìn", "giữ gìn"),
    ("dản dị", "giản dị"),
    ("dản đơn", "giản đơn"),
    ("giũng cảm", "dũng cảm"),
    ("giư thừa", "dư thừa"),
    ("giư luận", "dư luận"),
    ("giư nợ", "dư nợ"),
    ("giuy trì", "duy trì"),
    ("giuy nhất", "duy nhất"),
    ("giao động", "dao động"),
    ("dao tiếp", "giao tiếp"),
    ("giồi giàu", "dồi dào"),
    ("dẽo dai", "dẻo dai"),
    ("giành dụm", "dành dụm"),
    ("giành cho", "dành cho"),
    ("giành thời gian", "dành thời gian"),
    ("giành tặng", "dành tặng"),
    ("giành riêng", "dành riêng"),
    ("dành giật", "giành giật"),
    ("dành chiến thắng", "giành chiến thắng"),
    ("dành giải", "giành giải"),
    ("dành quyền", "giành quyền"),
    ("dành lấy", "giành lấy"),
    ("dành về mình", "giành về mình"),
    ("tranh dành", "tranh giành"),
    ("dành thắng lợi", "giành thắng lợi"),
    ("dành độc lập", "giành độc lập"),
    ("dành chính quyền", "giành chính quyền"),
    ("dành huy chương", "giành huy chương"),
    ("dành cúp", "giành cúp"),
    ("dành vị trí", "giành vị trí"),
    ("dành thị trường", "giành thị trường"),
    ("dành khách hàng", "giành khách hàng"),
    ("dành ưu thế", "giành ưu thế"),
    ("dành phần thắng", "giành phần thắng"),
    ("dành giải thưởng", "giành giải thưởng"),
    ("dành danh hiệu", "giành danh hiệu"),
    ("dành chiến công", "giành chiến công"),
    ("dành thắng cuộc", "giành thắng cuộc"),
    ("vô hình chung", "vô hình trung"),
    ("tựu chung", "tựu trung"),
    ("tựu chung lại", "tựu trung lại"),
    ("xúi dục", "xúi giục"),
    ("dục dã", "giục giã"),

    # ── HỎI / NGÃ ────────────────────────────────────────────────────────────
    ("suy nghỉ", "suy nghĩ"),
    ("vửng vàng", "vững vàng"),
    ("vửng bền", "vững bền"),
    ("bền vửng", "bền vững"),
    ("sẳn sàng", "sẵn sàng"),
    ("sẳn lòng", "sẵn lòng"),
    ("thễ hiện", "thể hiện"),
    ("nổ lực", "nỗ lực"),
    ("nỗi lực", "nỗ lực"),
    ("mảnh liệt", "mãnh liệt"),
    ("dũng mảnh", "dũng mãnh"),
    ("vất vã", "vất vả"),
    ("vất vưỡng", "vất vưởng"),
    ("rõ ràn", "rõ ràng"),
    ("rộng rải", "rộng rãi"),
    ("rực rở", "rực rỡ"),
    ("rạng rở", "rạng rỡ"),
    ("vui vẽ", "vui vẻ"),
    ("vẽ đẹp", "vẻ đẹp"),
    ("vẽ vang", "vẻ vang"),
    ("vẻ tranh", "vẽ tranh"),
    ("vẻ hình", "vẽ hình"),
    ("nghỉa vụ", "nghĩa vụ"),
    ("nghỉa tình", "nghĩa tình"),
    ("nghỉa trang", "nghĩa trang"),
    ("nghỉ ngợi", "nghĩ ngợi"),
    ("chí nhớ", "trí nhớ"),
    ("chí óc", "trí óc"),
    ("chí tưởng", "trí tưởng"),
    ("ý trí", "ý chí"),
    ("kiên chì", "kiên trì"),
    ("băng khoăn", "băn khoăn"),
    ("trăng trở", "trăn trở"),
    ("lạng mạn", "lãng mạn"),
    ("lạng phí", "lãng phí"),
    ("nong lanh", "long lanh"),
    ("nung linh", "lung linh"),
    ("nạnh lùng", "lạnh lùng"),
    ("lũng lịu", "nũng nịu"),
    ("ngở ngàng", "ngỡ ngàng"),
    ("sững sốt", "sửng sốt"),
    ("sửng sờ", "sững sờ"),
    ("bàn hoàng", "bàng hoàng"),
    ("hoan mang", "hoang mang"),
    ("hoản hốt", "hoảng hốt"),
    ("hốt hoản", "hốt hoảng"),
    ("bối rồi", "bối rối"),
    ("rối rắng", "rối rắm"),
    ("rối loạng", "rối loạn"),
    ("lộn xộng", "lộn xộn"),
    ("hổn độn", "hỗn độn"),
    ("hổn hợp", "hỗn hợp"),
    ("hổn loạn", "hỗn loạn"),
    ("bứt xúc", "bức xúc"),
    ("bức súc", "bức xúc"),
    ("rộng ràng", "rộn ràng"),
    ("thừng thờ", "thững thờ"),
    ("hờ hửng", "hờ hững"),
    ("hửng hờ", "hững hờ"),
    ("vô nghỉa", "vô nghĩa"),

    # ── ÂM CUỐI N/NG, C/T ───────────────────────────────────────────────────
    ("thậc là", "thật là"),
    ("thật nà", "thật là"),
    ("khỗ công", "khổ công"),
    ("khổ côn", "khổ công"),
    ("lưới sắc", "lưới sắt"),
    ("lưới xắt", "lưới sắt"),
    # KHÔNG đưa "bột ngọc" vào: "bột ngọc trai" là cụm hợp lệ.
    ("bột ngọng", "bột ngọt"),
    ("đột lập", "độc lập"),
    ("bác thảo", "bách thảo"),
    ("chùm chăn", "trùm chăn"),
    ("trùm trăn", "trùm chăn"),
    ("trùm chăng", "trùm chăn"),
    ("chầm cảm", "trầm cảm"),
    ("chình tự", "trình tự"),
    ("trè khô", "chè khô"),
    ("con lưng", "cong lưng"),
    ("cong nưng", "cong lưng"),
    ("lòn máng", "lòng máng"),
    ("nòng máng", "lòng máng"),
    ("lòng mán", "lòng máng"),
    ("hò khoang", "hò khoan"),
    ("nhỡ bửa", "nhỡ bữa"),
    ("đơn chiết", "đơn chiếc"),
    ("bi đôn", "bi đông"),
    ("cam tâng", "cam tân"),
    ("yêng nghiệp", "yên nghiệp"),
    ("quỳnh yêng", "quỳnh yên"),

    # ── KINH ĐIỂN / HÁN-VIỆT ────────────────────────────────────────────────
    ("bổ xung", "bổ sung"),
    ("chỉnh chu", "chỉn chu"),
    ("chuẩn đoán", "chẩn đoán"),
    ("thăm quan", "tham quan"),
    ("chau chuốt", "trau chuốt"),
    ("trau truốt", "trau chuốt"),
    ("cọ sát", "cọ xát"),
    ("khuyến khính", "khuyến khích"),
    ("khuyến khít", "khuyến khích"),
    ("khuyến khịch", "khuyến khích"),
    ("chín mùi", "chín muồi"),
    ("xơ xài", "sơ sài"),
    ("sơ xài", "sơ sài"),
    ("mẫu thuẫn", "mâu thuẫn"),
    ("mâu thuẩn", "mâu thuẫn"),
    ("tiêu chẩn", "tiêu chuẩn"),
    # KHÔNG đưa "năng xuất" vào: "khả năng xuất hiện" là cụm hợp lệ.
    ("xuất ăn", "suất ăn"),
    ("tham dò", "thăm dò"),
    ("tham hỏi", "thăm hỏi"),
    ("tham nom", "thăm nom"),
    ("tham thú", "thăm thú"),
    ("tham viếng", "thăm viếng"),
    # KHÔNG đưa "thăm dự", "thăm gia", "thăm khảo", "thăm lam" vào: chúng là
    # tiền tố hợp lệ của "thăm dự án", "thăm gia đình", "thăm khảo cổ",
    # "thăm Lam" và sẽ bắt oan văn bản sạch.
    ("thăm nhũng", "tham nhũng"),
    ("thăm vọng", "tham vọng"),
    ("thăm ô", "tham ô"),
]
# fmt: on


def nfc(s: str) -> str:
    return unicodedata.normalize("NFC", s.strip().lower())


def load_lexicon() -> set[str]:
    if not LEXICON_PATH.exists():
        raise FileNotFoundError(f"Không tìm thấy từ điển gốc: {LEXICON_PATH}")
    valid: set[str] = set()
    with open(LEXICON_PATH, encoding="utf-8") as f:
        for line in f:
            word = nfc(line)
            if word:
                valid.add(word)
    return valid


def load_valid_prefixes(valid: set[str]) -> dict[int, set[str]]:
    """Tập tiền tố n từ của các cụm dài hơn trong từ điển.

    Dùng để chặn false positive khi cụm sai chỉ là phần đầu của một cụm hợp lệ:
    "thăm gia" (trong "thăm gia đình"), "chuyền tay" (trong "chuyền tay nhau"),
    "giành cho" (trong "giành cho mình")...
    """
    prefixes: dict[int, set[str]] = {}
    for entry in valid:
        parts = entry.split()
        for n in (1, 2, 3):
            if len(parts) > n:
                prefixes.setdefault(n, set()).add(" ".join(parts[:n]))
    return prefixes


def build_pairs(
    valid: set[str], prefixes: dict[int, set[str]]
) -> tuple[dict[str, str], list[str]]:
    """Trả về (map sai->đúng, danh sách cảnh báo).

    Kiểm định hai chiều bắt buộc:
      1. Cụm đúng PHẢI có trong Viet74K.
      2. Cụm sai KHÔNG được có trong Viet74K (chống đồng âm khác nghĩa).
      3. Cụm sai không được là tiền tố của một cụm dài hơn hợp lệ.
    """
    mapping: dict[str, str] = {}
    warnings: list[str] = []
    conflicts: dict[str, set[str]] = {}

    for raw_wrong, raw_right in SEED_PAIRS:
        wrong, right = nfc(raw_wrong), nfc(raw_right)
        if not wrong or not right or wrong == right:
            continue
        if wrong in valid:
            warnings.append(f"BỎ (từ sai có trong Viet74K, dễ nhập nhằng): {wrong!r}")
            continue
        if right not in valid:
            warnings.append(f"BỎ (từ đúng không có trong Viet74K): {wrong!r} -> {right!r}")
            continue
        n = len(wrong.split())
        if wrong in prefixes.get(n, set()):
            warnings.append(f"BỎ (từ sai là tiền tố của cụm hợp lệ dài hơn): {wrong!r}")
            continue
        conflicts.setdefault(wrong, set()).add(right)

    for wrong, targets in conflicts.items():
        if len(targets) > 1:
            warnings.append(f"BỎ (ánh xạ nhập nhằng {sorted(targets)}): {wrong!r}")
            continue
        mapping[wrong] = next(iter(targets))
    return mapping, warnings


def render_rust(mapping: dict[str, str]) -> str:
    pairs = sorted(mapping.items())
    lines = [
        "// @generated by tools/generate_common_typos.py — DO NOT EDIT MANUALLY.",
        "// Nguồn: danh sách hiệu đính thủ công, đã đối chiếu Viet74K để loại từ nhập nhằng.",
        f"// Tổng số cặp: {len(pairs)}",
        "",
        "/// (cụm sai, cụm đúng) — sắp xếp tăng dần theo cụm sai để binary_search.",
        "pub const COMMON_TYPO_PAIRS: &[(&str, &str)] = &[",
    ]
    for wrong, right in pairs:
        lines.append(f'    ("{wrong}", "{right}"),')
    lines.extend(["];", ""])
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Chỉ kiểm tra, không ghi file")
    args = parser.parse_args()

    valid = load_lexicon()
    prefixes = load_valid_prefixes(valid)
    mapping, warnings = build_pairs(valid, prefixes)
    for warning in warnings:
        print(f"[!] {warning}")

    if len(mapping) < 200:
        raise SystemExit(
            f"Chỉ còn {len(mapping)} cặp hợp lệ (< 200). Kiểm tra lại bộ lọc/nhập liệu."
        )

    content = render_rust(mapping)
    if args.check:
        current = RUST_OUT_PATH.read_text(encoding="utf-8") if RUST_OUT_PATH.exists() else ""
        if current != content:
            raise SystemExit("common_typos_data.rs đã lỗi thời; chạy lại không có --check.")
        print(f"[✓] common_typos_data.rs đồng bộ ({len(mapping)} cặp).")
        return

    RUST_OUT_PATH.write_text(content, encoding="utf-8")
    print(f"[+] Đã sinh {len(mapping)} cặp -> {RUST_OUT_PATH}")


if __name__ == "__main__":
    main()
