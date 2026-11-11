#!/usr/bin/env python3
"""Sinh bộ test chính tả / typography tiếng Việt từ văn bản sách.

Nguyên tắc
  * Mỗi câu lỗi có ĐÚNG MỘT lỗi (kiểm tra bất biến trước khi ghi file).
  * Câu sạch giữ nguyên để đo báo nhầm (FPR).
  * Chia dev/test/blind theo TÀI LIỆU (hash của đường dẫn + salt), không theo câu.
    Mặc định cân bằng văn phong giữa 3 tập (--stratify); dùng --no-stratify để
    chia thuần theo ngưỡng hash như bản gốc.
  * Không dùng LLM: chỉ là các toán tử gây lỗi theo luật.

Dùng
  python gen_benchmark.py --corpus ./corpus --out ./bench --salt <chuỗi-bí-mật>
  (thư mục con có chứa 'admin' hoặc 'hanhchinh' trong đường dẫn => văn bản hành chính)

Ghi chú thiết kế
  * Câu được khử trùng lặp toàn cục trước khi chia tập, tránh rò rỉ câu giữa
    dev/test/blind khi cùng một câu xuất hiện trong nhiều tài liệu.
  * Toán tử hành chính chỉ sinh lỗi ở vị trí mà rule tương ứng thực sự báo
    (ví dụ 'khoản' phải ở giữa câu, 'thành phố' phải đi sau 'tại', ...).
  * --lexicon (tùy chọn) dùng Viet74K hoặc từ điển âm tiết để gắn nhãn
    real-word chính xác hơn là đếm tần suất trong corpus.

Mỗi dòng JSONL gồm: id, split, doc, register, text, expected_flag, original,
operator, group, rule, label_confidence, edit_span [start, end), orig_frag,
new_frag, is_real_word. Với câu lỗi, áp dụng nghịch đảo tại edit_span:
  text[:start] + orig_frag + text[end:] == original
đó là bất biến "đúng một lỗi" được kiểm tra trước khi ghi file.

MANIFEST.json chỉ lưu salt_fingerprint (16 ký tự SHA-256), không lưu salt gốc.
"""
from __future__ import annotations

import argparse
import hashlib
import itertools
import json
import random
import re
import sys
import unicodedata
from collections import Counter, defaultdict
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

# --------------------------------------------------------------------------
# Âm tiết tiếng Việt
# --------------------------------------------------------------------------
VOW = set("aăâeêioôơuưy")
LETTERS = VOW | set("bcdđghklmnpqrstvx")
DIAC = set("ăâêôơư")
TONES = [None, "\u0300", "\u0301", "\u0309", "\u0303", "\u0323"]
COMB_TONES = {t for t in TONES if t}
COMB_DIAC = {"\u0306", "\u0302", "\u031b"}
INI = ["ngh", "ng", "nh", "ch", "tr", "th", "kh", "ph", "gh", "gi", "qu",
       "b", "c", "d", "đ", "g", "h", "k", "l", "m", "n", "p", "r", "s", "t", "v", "x"]


def parse(tok):
    """Tách âm tiết thành [[chữ_cái_NFC, dấu_thanh_hoặc_None], ...]."""
    units = []
    for ch in unicodedata.normalize("NFD", tok):
        if ch in COMB_TONES:
            if not units or units[-1][1] is not None:
                return None
            units[-1][1] = ch
        elif ch in COMB_DIAC:
            if not units:
                return None
            units[-1][0] = unicodedata.normalize("NFC", units[-1][0] + ch)
        else:
            units.append([ch, None])
    if not units or any(len(c) != 1 or c not in LETTERS for c, _ in units):
        return None
    return units


def build(units):
    return "".join(unicodedata.normalize("NFC", c + (t or "")) for c, t in units)


def split_syl(t):
    """Tách âm tiết không dấu thanh.

    Trả về (độ_dài_phụ_âm_đầu, bắt_đầu_nguyên_âm, hết_nguyên_âm,
    độ_dài_phụ_âm_cuối), hoặc None nếu không phải âm tiết hợp lệ.
    """
    ini = ""
    for i in INI:
        if t.startswith(i):
            ini = i
            break
    if ini == "gi":
        rest = t[2:]
        if not rest or rest[0] not in VOW:
            ini = "g"
    start = len(ini)
    coda = ""
    for c in ("ng", "nh", "ch", "c", "m", "n", "p", "t"):
        if t.endswith(c) and len(t) - len(c) > start and t[len(t) - len(c) - 1] in VOW:
            coda = c
            break
    end = len(t) - len(coda)
    if end <= start or any(ch not in VOW for ch in t[start:end]):
        return None
    return len(ini), start, end, len(coda)


def tone_target(t, a, b, coda_len):
    """Vị trí đặt dấu thanh (kiểu mới: hoà, thuỷ).

    Quy ước hiện hành:
      * Có nguyên âm mang dấu cấu tạo (ă â ê ô ơ ư): đặt ở nguyên âm đó.
      * Nhóm oa/oe/uy: đặt ở nguyên âm thứ hai (hoà, thuỷ).
      * Các nhóm hai nguyên âm khác: đặt ở nguyên âm đầu nếu không có phụ âm cuối.
      * Nhóm ba nguyên âm: đặt ở nguyên âm giữa.
    """
    nuc = t[a:b]
    idx = [i for i, ch in enumerate(nuc) if ch in DIAC]
    if idx:
        return a + idx[-1]
    n = len(nuc)
    if n == 1:
        return a
    if n == 2:
        if nuc in {"oa", "oe", "uy"}:
            return a + 1
        return a + 1 if coda_len else a
    return a + 1


# --------------------------------------------------------------------------
# Token và toán tử
# --------------------------------------------------------------------------
def cands(text):
    out = []
    for m in re.finditer(r"\w+", text):
        tok = m.group()
        s, e = m.span()
        if len(tok) < 2 or not tok.isalpha() or tok != tok.lower():
            continue
        if (s > 0 and text[s - 1] in "-/_@#") or (e < len(text) and text[e] in "-/_@#"):
            continue
        out.append((s, e, tok))
    return out


def _syl_edit(text, rng, fn):
    cs = cands(text)
    rng.shuffle(cs)
    for s, e, tok in cs:
        u = parse(tok)
        if not u:
            continue
        t = "".join(c for c, _ in u)
        sp = split_syl(t)
        if not sp:
            continue
        res = fn(u, t, sp, rng)
        if res is None:
            continue
        new = build(res)
        if new == tok:
            continue
        return text[:s] + new + text[e:], s, s + len(new), tok, new
    return None


def f_tone(u, t, sp, rng):
    _, a, b, cl = sp
    ti = [i for i, (_, tn) in enumerate(u) if tn]
    if len(ti) > 1:
        return None
    cur = u[ti[0]][1] if ti else None
    new = rng.choice([x for x in TONES if x != cur])
    pos = ti[0] if ti else tone_target(t, a, b, cl)
    out = [list(x) for x in u]
    out[pos][1] = new
    return out


INIT_PAIRS = {"s": ["x"], "x": ["s"], "ch": ["tr"], "tr": ["ch"],
              "d": ["gi", "r"], "gi": ["d", "r"], "r": ["d", "gi"],
              "l": ["n"], "n": ["l"]}


def f_initial(u, t, sp, rng):
    il, a, _, _ = sp
    choices = INIT_PAIRS.get(t[:il])
    if not choices:
        return None
    new = rng.choice(choices)
    if new == "gi" and t[a] == "i":
        return None
    return [[c, None] for c in new] + [list(x) for x in u[il:]]


FINAL_PAIRS = {"n": ["ng"], "ng": ["n"], "t": ["c"], "c": ["t"]}


def f_final(u, t, sp, rng):
    _, _, _, cl = sp
    if cl == 0:
        return None
    choices = FINAL_PAIRS.get(t[len(t) - cl:])
    if not choices:
        return None
    new = rng.choice(choices)
    return [list(x) for x in u[:len(u) - cl]] + [[c, None] for c in new]


def f_conv(u, t, sp, rng):
    il, a, _, _ = sp
    ini, nxt = t[:il], t[a]
    new = None
    if ini == "gh" and nxt in "eêi":
        new = "g"
    elif ini == "ngh" and nxt in "eêi":
        new = "ng"
    elif ini == "k" and nxt in "eêiy":
        new = "c"
    elif ini == "c" and nxt in "aăâoôơuư":
        new = "k"
    elif ini == "g" and nxt in "aăâoôơuư":
        new = "gh"
    elif ini == "ng" and nxt not in "eêiy":
        new = "ngh"
    if new is None:
        return None
    return [[c, None] for c in new] + [list(x) for x in u[il:]]


DROP = {"đ": "d", "ă": "a", "â": "a", "ê": "e", "ô": "o", "ơ": "o", "ư": "u"}


def f_drop(u, t, sp, rng):
    idx = [i for i, (c, _) in enumerate(u) if c in DROP]
    if not idx:
        return None
    i = rng.choice(idx)
    out = [list(x) for x in u]
    out[i][0] = DROP[out[i][0]]
    return out


def op_merge(text, rng):
    cs = cands(text)
    pairs = [(a, b) for a, b in itertools.pairwise(cs) if text[a[1]:b[0]] == " "]
    if not pairs:
        return None
    (s1, _, t1), (_, e2, t2) = rng.choice(pairs)
    new = t1 + t2
    return text[:s1] + new + text[e2:], s1, s1 + len(new), t1 + " " + t2, new


FUNC = {"đã", "được", "của", "và", "là", "cho", "để", "các", "những",
        "với", "trong", "về", "thì", "mà", "sẽ", "đang", "bị"}


def op_dup(text, rng):
    cs = [
        c for c in cands(text)
        if c[2] in FUNC
        and not text[c[1]:].startswith(" " + c[2])
        and not text[:c[0]].endswith(" " + c[2])
    ]
    if not cs:
        return None
    s, e, tok = rng.choice(cs)
    new = tok + " " + tok
    return text[:s] + new + text[e:], s, s + len(new), tok, new


def op_space_before_comma(text, rng):
    idx = [m.start() for m in re.finditer(r"(?<=[^\W\d_]), (?=[^\W\d_])", text)]
    if not idx:
        return None
    i = rng.choice(idx)
    return text[:i] + " ," + text[i + 1:], i, i + 2, ",", " ,"


def op_no_space_after_comma(text, rng):
    idx = [m.start() for m in re.finditer(r"(?<=[^\W\d_]), (?=[^\W\d_])", text)]
    if not idx:
        return None
    i = rng.choice(idx)
    return text[:i + 1] + text[i + 2:], i, i + 1, ", ", ","


def op_initial_lower(text, rng):
    if not text or not text[0].isalpha() or not text[0].isupper() or text[:2].isupper():
        return None
    return text[0].lower() + text[1:], 0, 1, text[0], text[0].lower()


# --------------------------------------------------------------------------
# Toán tử hành chính (chỉ áp dụng cho văn bản hành chính).
# rule: mã quy tắc trong cuộc trò chuyện (R1, R3, R4, R5, R7, R11);
# conf: độ tin cậy của nhãn.
# --------------------------------------------------------------------------
SENT_END = set(".!?:\n")


def is_sentence_start(text, pos):
    """Giống is_sentence_start trong crates/vwi-core/src/rules/admin_format.rs."""
    i = pos - 1
    while i >= 0 and text[i].isspace():
        i -= 1
    return i < 0 or text[i] in SENT_END


def make_regex_op(pattern, repl, mid_sentence=False):
    rx = re.compile(pattern)

    def fn(text, rng):
        ms = [
            m for m in rx.finditer(text)
            if not mid_sentence or not is_sentence_start(text, m.start())
        ]
        if not ms:
            return None
        m = rng.choice(ms)
        new = m.expand(repl)
        if new == m.group():
            return None
        return text[:m.start()] + new + text[m.end():], m.start(), m.start() + len(new), m.group(), new
    return fn


def thousand_comma(text, rng):
    ms = list(re.finditer(r"\b\d{1,3}(?:\.\d{3})+\b", text))
    if not ms:
        return None
    m = rng.choice(ms)
    new = m.group().replace(".", ",")
    return text[:m.start()] + new + text[m.end():], m.start(), m.start() + len(new), m.group(), new


def nha_nuoc_low(text, rng):
    """'Nhà nước' -> 'nhà nước', bỏ qua ngữ cảnh định ngữ đã đúng quy tắc."""
    ms = [
        m for m in re.finditer(r"\bNhà\s+nước\b", text)
        if not is_sentence_start(text, m.start())
        and not any(k in text[max(0, m.start() - 120):m.start()].lower()
                    for k in ("cơ quan", "tổ chức", "chính quyền"))
    ]
    if not ms:
        return None
    m = rng.choice(ms)
    new = "nhà" + m.group(0)[len("Nhà"):]
    return text[:m.start()] + new + text[m.end():], m.start(), m.start() + len(new), m.group(), new


def motto_low(text, rng):
    rx = re.compile(r"(?i)độc\s+lập\s*[-–]\s*tự\s+do\s*[-–]\s*hạnh\s+phúc")
    ms = list(rx.finditer(text))
    if not ms:
        return None
    m = rng.choice(ms)
    new = m.group().lower()
    if new == m.group():
        return None
    return text[:m.start()] + new + text[m.end():], m.start(), m.start() + len(new), m.group(), new


ADMIN = {
    "date_day_zero":   (r"ngày(\s+)0([1-9])(?=\s+tháng)", r"ngày\1\2", "R1", "high", False),
    "date_month_zero": (r"tháng(\s+)0([12])(?!\d)", r"tháng\1\2", "R1", "high", False),
    "legal_khoan_cap": (r"khoản(\s+)(\d+)(?=\s+Điều)", r"Khoản\1\2", "R3", "high", True),
    "legal_diem_cap":  (r"điểm(\s+)([a-zđ])(?=\s+khoản)", r"Điểm\1\2", "R3", "high", True),
    "legal_dieu_low":  (r"Điều(\s+)(\d+|[IVX]+)", r"điều\1\2", "R3", "high", True),
    "nhan_dan_low":    (r"Nhân(\s+)dân(?!\s+tệ)", r"nhân\1dân", "R4", "high", True),
    # R5 chỉ báo khi danh từ địa danh đi sau "tại" và trước một tên riêng.
    "tp_hcm_low":      (r"([Tt]ại\s+)Thành phố(?= Hồ Chí Minh)", r"\1thành phố", "R5", "high", False),
    "thu_do_low":      (r"([Tt]ại\s+)Thủ đô(?= Hà Nội)", r"\1thủ đô", "R5", "high", False),
    # R11 của plugin hiện chỉ nhận số thập phân 1 chữ số kèm dấu %.
    "decimal_dot":     (r"(?<=\d),(?=\d\s*%)", r".", "R11", "medium", False),
}


def make_syl(fn):
    return lambda text, rng: _syl_edit(text, rng, fn)


OPS = {}
for _n, _f, _g in [("tone", f_tone, "spelling"), ("initial", f_initial, "spelling"),
                   ("final", f_final, "spelling"), ("spell_conv", f_conv, "spelling"),
                   ("diac_drop", f_drop, "spelling")]:
    OPS[_n] = {"fn": make_syl(_f), "group": _g, "rule": "R12", "conf": "high",
               "admin": False, "token": True}
for _n, _f, _g in [("merge_words", op_merge, "spacing"), ("dup_function", op_dup, "grammar_lite"),
                   ("space_before_comma", op_space_before_comma, "punct"),
                   ("no_space_after_comma", op_no_space_after_comma, "punct"),
                   ("initial_lower", op_initial_lower, "capitalization")]:
    OPS[_n] = {"fn": _f, "group": _g, "rule": "R12", "conf": "high",
               "admin": False, "token": False}
for _n, (_p, _r, _rule, _conf, _mid) in ADMIN.items():
    OPS[_n] = {"fn": make_regex_op(_p, _r, _mid), "group": "admin_" + _rule, "rule": _rule,
               "conf": _conf, "admin": True, "token": False}
OPS["nha_nuoc_low"] = {"fn": nha_nuoc_low, "group": "admin_R4", "rule": "R4",
                       "conf": "medium", "admin": True, "token": False}
OPS["motto_low"] = {"fn": motto_low, "group": "admin_R7", "rule": "R7",
                    "conf": "high", "admin": True, "token": False}
OPS["thousand_comma"] = {"fn": thousand_comma, "group": "admin_R11", "rule": "R11",
                         "conf": "low", "admin": True, "token": False}


# --------------------------------------------------------------------------
# Tách câu và lọc
# --------------------------------------------------------------------------
ABBR = {"TS.", "PGS.", "ThS.", "GS.", "TP.", "TT.", "TX.", "BS.", "KS.", "Q.", "P.", "H.", "ĐT.", "Mr.", "Ms."}
PUNCT_OK = set(" .,;:!?()-–—\"'%/“”‘’«»…‑")


def split_sentences(par):
    pieces = re.split(r"(?<=[.!?])\s+", par.strip())
    out = []
    for p in pieces:
        if out:
            last = out[-1].split()[-1] if out[-1].split() else ""
            if last in ABBR or (len(last) == 2 and last[0].isupper() and last[1] == "."):
                out[-1] += " " + p
                continue
        out.append(p)
    return out


def is_viet_char(ch):
    if ch.isdigit() or ch in PUNCT_OK:
        return True
    if ch.isalpha():
        base = unicodedata.normalize("NFD", ch.lower())[0]
        return base in LETTERS
    return False


OPENERS = set("\"'“‘«([")


def good_sentence(s, lo, hi):
    if not s:
        return False
    n = len(s.split())
    if n < lo or n > hi or "http" in s or "@" in s:
        return False
    i = 0
    while i < len(s) and (s[i].isspace() or s[i] in OPENERS):
        i += 1
    if i >= len(s) or not (s[i].isupper() or s[i].isdigit()):
        return False
    return all(is_viet_char(c) for c in s)


def load_docs(root, admin_kw, lo, hi, skip_dir=None):
    docs = []
    kws = [k.lower() for k in admin_kw]
    base = Path(root).resolve()
    skip = Path(skip_dir).resolve() if skip_dir else None
    for p in sorted(Path(root).rglob("*.txt")):
        if p.name == "stats.txt":
            continue
        if skip and skip != base and p.resolve().is_relative_to(skip):
            continue  # tránh đọc lại stats.txt nếu --out nằm trong --corpus
        rel = p.relative_to(root).as_posix()
        raw = unicodedata.normalize("NFC", p.read_text(encoding="utf-8", errors="replace"))
        reg = "administrative" if any(k in rel.lower() for k in kws) else "general"
        sents = []
        for line in raw.splitlines():
            line = re.sub(r"^[\-\u2013\u2014\u2022\*\s]+", "", line)
            line = re.sub(r"[ \t\u00a0]+", " ", line).strip()
            if not line:
                continue
            for s in split_sentences(line):
                if good_sentence(s, lo, hi):
                    sents.append(s)
        docs.append((rel, reg, sents, raw))
    return docs


def dedupe_docs(docs):
    """Khử câu trùng lặp giữa các tài liệu để chống rò rỉ dev/test/blind."""
    seen = set()
    n_dup = 0
    out = []
    for rel, reg, sents, raw in docs:
        keep = []
        for s in sents:
            key = re.sub(r"\s+", " ", s).strip().casefold()
            if key in seen:
                n_dup += 1
                continue
            seen.add(key)
            keep.append(s)
        if keep:
            out.append((rel, reg, keep, raw))
    return out, n_dup


def bucket(rel, salt, ratio):
    """Chia thuần theo hash (giữ cho tương thích và --no-stratify)."""
    h = int(hashlib.sha256((salt + "|" + rel).encode("utf-8")).hexdigest()[:8], 16) % 100
    acc = 0
    for name, w in zip(("dev", "test", "blind"), ratio, strict=True):
        acc += w
        if h < acc:
            return name
    return "blind"


def _split_counts(n, ratio):
    """Số tài liệu mỗi tập, bám sát tỉ lệ và giữ đủ mặt bằng khi có >= 3 tài liệu."""
    target = [n * w / 100 for w in ratio]
    counts = [1 if n >= 3 and w > 0 else 0 for w in ratio]
    remaining = n - sum(counts)
    while remaining > 0:
        i = max(range(3), key=lambda j: target[j] - counts[j])
        counts[i] += 1
        remaining -= 1
    return counts


def assign_splits(docs, salt, ratio, stratify=True):
    """Gán tài liệu vào tập, tùy chọn cân bằng theo văn phong.

    Mỗi văn phong được sắp xếp theo hash(path + salt); thứ tự này quyết định
    tài liệu nào vào tập nào. Nhờ vậy vẫn chia theo TÀI LIỆU và vẫn tất định,
    nhưng dev/test/blind đều có mặt các văn phong khi số tài liệu cho phép.
    """
    if not stratify:
        return {rel: bucket(rel, salt, ratio) for rel, _, _, _ in docs}
    by_reg = defaultdict(list)
    for d in docs:
        by_reg[d[1]].append(d)
    result = {}
    for _, group in sorted(by_reg.items()):
        ordered = sorted(
            group,
            key=lambda d: hashlib.sha256((salt + "|" + d[0]).encode("utf-8")).hexdigest(),
        )
        pos = 0
        for sp, count in zip(("dev", "test", "blind"), _split_counts(len(ordered), ratio),
                             strict=True):
            for d in ordered[pos:pos + count]:
                result[d[0]] = sp
            pos += count
    return result


# --------------------------------------------------------------------------
# Sinh dữ liệu
# --------------------------------------------------------------------------
def ok_edit(orig, new_text, s, e, ofrag, nfrag):
    return (new_text != orig and new_text[s:e] == nfrag
            and new_text[:s] + ofrag + new_text[e:] == orig)


def make_split(items, rng, per_op, n_clean, vocab, min_count, split,
               lexicon=None):
    rng.shuffle(items)
    recs = []
    quota = dict.fromkeys(OPS, per_op)
    clean, rest = items[:n_clean], items[n_clean:]

    def is_real_word(frag):
        if lexicon is not None:
            return frag.lower() in lexicon
        return vocab.get(frag.lower(), 0) >= min_count

    for doc, reg, sent in clean:
        recs.append({"split": split, "doc": doc, "register": reg, "text": sent,
                     "expected_flag": False, "original": sent, "operator": None, "group": "clean",
                     "rule": None, "label_confidence": "high", "edit_span": None,
                     "orig_frag": None, "new_frag": None, "is_real_word": None})
    for doc, reg, sent in rest:
        names = [n for n in OPS if quota[n] > 0 and (reg == "administrative" or not OPS[n]["admin"])]
        rng.shuffle(names)
        if reg == "administrative":
            names.sort(key=lambda n: not OPS[n]["admin"])
        for n in names:
            res = OPS[n]["fn"](sent, rng)
            if not res:
                continue
            new_text, s, e, ofrag, nfrag = res
            if not ok_edit(sent, new_text, s, e, ofrag, nfrag):
                continue
            rw = is_real_word(nfrag) if OPS[n]["token"] else None
            quota[n] -= 1
            recs.append({"split": split, "doc": doc, "register": reg, "text": new_text,
                         "expected_flag": True, "original": sent, "operator": n,
                         "group": OPS[n]["group"], "rule": OPS[n]["rule"],
                         "label_confidence": OPS[n]["conf"], "edit_span": [s, e],
                         "orig_frag": ofrag, "new_frag": nfrag, "is_real_word": rw})
            break
        if all(v <= 0 for v in quota.values()):
            break
    rng.shuffle(recs)
    for i, r in enumerate(recs):
        r["id"] = f"{split}-{i:05d}"
    validate_records(recs)
    return recs


def validate_records(recs):
    """Bất biến: câu sạch không đổi; câu lỗi đổi đúng một đoạn và khôi phục được."""
    ids = set()
    for r in recs:
        if r["id"] in ids:
            raise AssertionError("id trùng: " + r["id"])
        ids.add(r["id"])
        if not (unicodedata.is_normalized("NFC", r["text"])
                and unicodedata.is_normalized("NFC", r["original"])):
            raise AssertionError("văn bản chưa ở dạng NFC: " + r["id"])
        if r["expected_flag"]:
            s, e = r["edit_span"]
            if not (r["text"] != r["original"]
                    and r["text"][s:e] == r["new_frag"]
                    and r["text"][:s] + r["orig_frag"] + r["text"][e:] == r["original"]):
                raise AssertionError("vi phạm bất biến một lỗi: " + r["id"])
            if r["operator"] not in OPS:
                raise AssertionError("toán tử lạ: " + str(r["operator"]))
        else:
            if (r["text"] != r["original"] or r["operator"] is not None
                    or r["edit_span"] is not None):
                raise AssertionError("câu sạch bị sửa: " + r["id"])
    return True


def load_lexicon(path):
    words = set()
    for line in Path(path).read_text(encoding="utf-8", errors="replace").splitlines():
        for w in re.split(r"[\s\-]+", unicodedata.normalize("NFC", line.strip().lower())):
            if w:
                words.add(w)
    return words


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--corpus", required=True,
                    help="thư mục chứa các file .txt UTF-8 (1 file = 1 tài liệu)")
    ap.add_argument("--out", required=True)
    ap.add_argument("--salt", default="vwi-default-salt",
                    help="giữ bí mật để không ai tái tạo lại cách chia")
    ap.add_argument("--seed", type=int, default=20261111)
    ap.add_argument("--ratio", default="60,30,10", help="dev,test,blind (%%)")
    ap.add_argument("--per-op", type=int, default=50, help="số ca tối đa mỗi toán tử mỗi tập")
    ap.add_argument("--clean", type=int, default=300, help="số câu sạch mỗi tập")
    ap.add_argument("--min-tokens", type=int, default=6)
    ap.add_argument("--max-tokens", type=int, default=45)
    ap.add_argument("--min-vocab-count", type=int, default=3,
                    help="âm tiết xuất hiện >= n lần trong corpus mới coi là 'từ có thật'")
    ap.add_argument("--lexicon", default=None,
                    help="tùy chọn: file âm tiết (ví dụ data_raw/Viet74K.txt) để gắn nhãn real-word")
    ap.add_argument("--admin-keyword", action="append",
                    default=["admin", "hanhchinh", "hanh_chinh"])
    ap.add_argument("--stratify", action=argparse.BooleanOptionalAction, default=True,
                    help="cân bằng văn phong giữa dev/test/blind (mặc định: bật)")
    a = ap.parse_args()

    try:
        ratio = [int(x) for x in a.ratio.split(",")]
    except ValueError:
        ap.error("--ratio phải gồm 3 số nguyên, ví dụ 60,30,10")
    if len(ratio) != 3 or sum(ratio) != 100 or any(w < 0 for w in ratio):
        ap.error("--ratio phải có 3 số không âm cộng lại 100")
    if a.per_op < 0 or a.clean < 0 or a.min_vocab_count < 0:
        ap.error("--per-op, --clean và --min-vocab-count phải >= 0")
    if a.min_tokens < 1 or a.max_tokens < a.min_tokens:
        ap.error("cần 1 <= --min-tokens <= --max-tokens")

    docs = load_docs(a.corpus, a.admin_keyword, a.min_tokens, a.max_tokens, a.out)
    if not docs:
        sys.exit("Không tìm thấy file .txt trong " + a.corpus)
    if a.lexicon and not Path(a.lexicon).exists():
        sys.exit("Không tìm thấy file lexicon: " + a.lexicon)
    docs, n_dup = dedupe_docs(docs)
    if not docs:
        sys.exit("Không có câu hợp lệ sau khi lọc và khử trùng lặp.")

    vocab = Counter()
    for _, _, _, raw in docs:
        vocab.update(w.lower() for w in re.findall(r"[^\W\d_]+", raw))
    lexicon = load_lexicon(a.lexicon) if a.lexicon else None

    split_of = assign_splits(docs, a.salt, ratio, a.stratify)
    by_split = defaultdict(list)
    n_docs = Counter({"dev": 0, "test": 0, "blind": 0})
    n_register = defaultdict(Counter)
    for rel, reg, sents, _ in docs:
        sp = split_of[rel]
        n_docs[sp] += 1
        n_register[sp][reg] += 1
        by_split[sp].extend((rel, reg, s) for s in sents)

    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    params = dict(vars(a))
    salt = params.pop("salt", "")
    params["salt_fingerprint"] = hashlib.sha256(salt.encode("utf-8")).hexdigest()[:16]
    manifest = {
        "schema_version": 1,
        "params": params,
        "n_docs": dict(n_docs),
        "register_docs": {sp: dict(c) for sp, c in n_register.items()},
        "dropped_duplicate_sentences": n_dup,
        "operators": {
            n: {"group": m["group"], "rule": m["rule"], "confidence": m["conf"],
                "admin": m["admin"], "token": m["token"]}
            for n, m in OPS.items()
        },
        "files": {},
        "counts": {},
    }
    lines = []
    for sp in ("dev", "test", "blind"):
        rng = random.Random(f"{a.seed}|{sp}")
        recs = make_split(by_split[sp], rng, a.per_op, a.clean, vocab,
                          a.min_vocab_count, sp, lexicon)
        path = out / (sp + ".jsonl")
        with path.open("w", encoding="utf-8", newline="\n") as f:
            for r in recs:
                f.write(json.dumps(r, ensure_ascii=False) + "\n")
        manifest["files"][sp] = hashlib.sha256(path.read_bytes()).hexdigest()
        ops = Counter(r["operator"] for r in recs if r["expected_flag"])
        n_clean = sum(1 for r in recs if not r["expected_flag"])
        n_err = sum(1 for r in recs if r["expected_flag"])
        rw = sum(1 for r in recs if r["is_real_word"])
        manifest["counts"][sp] = {"total": len(recs), "clean": n_clean,
                                  "errors": n_err, "real_word": rw}
        regs_txt = ", ".join(f"{r}={v}" for r, v in sorted(n_register[sp].items())) or "không có"
        lines.append(
            f"== {sp}: {n_docs[sp]} tài liệu ({regs_txt}), {len(recs)} câu "
            f"(sạch={n_clean}, lỗi={n_err}, real-word={rw})")
        for k, v in sorted(ops.items()):
            lines.append(f"   {k:<22} {v:>4}")
        under = [n for n in OPS if ops[n] < a.per_op]
        if under:
            lines.append(f"   (chưa đủ quota {a.per_op}: {', '.join(under)})")
        if n_clean < a.clean:
            lines.append(f"   (chưa đủ {a.clean} câu sạch: chỉ có {n_clean})")
    all_regs = sorted({r for c in n_register.values() for r in c})
    for sp in ("dev", "test", "blind"):
        missing_regs = [r for r in all_regs if n_register[sp].get(r, 0) == 0]
        if missing_regs:
            lines.append(
                f"   (cảnh báo: {sp} không có tài liệu {', '.join(missing_regs)}; "
                "cần thêm tài liệu cho văn phong này)")
    if n_dup:
        lines.append(f"Đã bỏ {n_dup} câu trùng lặp giữa các tài liệu.")
    (out / "MANIFEST.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    (out / "stats.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print("\n".join(lines))


if __name__ == "__main__":
    main()
