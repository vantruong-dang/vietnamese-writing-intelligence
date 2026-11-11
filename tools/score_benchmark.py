#!/usr/bin/env python3
"""Chấm điểm kết quả plugin trên file benchmark (dev/test/blind.jsonl).

Định dạng file dự đoán (JSONL, một dòng một câu, khớp theo "id"):
  {"id": "test-00012", "flagged": true, "fixed_text": "câu đã sửa toàn bộ",
   "spans": [[10, 14]]}
  - fixed_text: kết quả áp dụng TẤT CẢ gợi ý lên câu gốc (không phải từng gợi ý lẻ).
  - spans: (tùy chọn) vị trí các cảnh báo. Mặc định là chỉ số KÝ TỰ (codepoint).
           Nếu plugin Rust trả byte offset, chạy thêm: --span-unit byte

Các chỉ số detected/fixed/located/precision đều ở mức CÂU (record-level):
một câu lỗi được tính là "dung vi tri" nếu có ít nhất một span dự đoán giao
với edit_span (sai số --tol ký tự).

Dùng:
  python score_benchmark.py --bench bench/test.jsonl --pred out/test_pred.jsonl
  python score_benchmark.py ... --json out/score.json --dump-errors out/errors.jsonl
  python score_benchmark.py ... --by-doc

BLIND BASELINE (2026-11, product thật `_core`, 800 ca: 300 sạch/500 lỗi):
  FPR 7/300 = 2,3%; detection 182/500 = 36,4%; sửa đúng 148/500 = 29,6%;
  đúng vị trí 157/500 = 31,4%; precision 96,3%; real-word 6/132.
  Toàn bộ 7 FP là artifact khoảng trắng của corpus legal (" .", "“ "),
  không phải rule logic. Đây là baseline sản phẩm đầu tiên trên blind set.
"""
from __future__ import annotations

import argparse
import json
import math
import sys
import unicodedata
from collections import defaultdict
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

CONF_ORDER = {"low": 0, "medium": 1, "high": 2}


def wilson(k, n, z=1.96):
    if n == 0:
        return (0.0, 0.0)
    p = k / n
    d = 1 + z * z / n
    c = p + z * z / (2 * n)
    a = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n))
    return ((c - a) / d, (c + a) / d)


def load(path):
    with open(path, encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


def load_preds(path):
    preds = {}
    duplicates = []
    invalid = 0
    for p in load(path):
        pid = p.get("id")
        if not isinstance(pid, str) or not pid:
            invalid += 1
            continue
        if pid in preds:
            duplicates.append(pid)
        preds[pid] = p
    return preds, duplicates, invalid


def nfc(s):
    return unicodedata.normalize("NFC", s or "")


def to_char_spans(text, spans, unit):
    if unit == "char" or not spans:
        return spans
    raw = text.encode("utf-8")
    return [[len(raw[:b].decode("utf-8", errors="ignore")) for b in sp] for sp in spans]


def overlap(a, b, tol):
    return a[0] < b[1] + tol and b[0] < a[1] + tol


def fmt(k, n):
    if n == 0:
        return "   -"
    lo, hi = wilson(k, n)
    return f"{k:3d}/{n:<4d} {100 * k / n:5.1f}% [{100 * lo:4.1f}-{100 * hi:4.1f}]"


def stat(rs, preds, have_spans, span_unit, tol):
    det = fixed = loc = 0
    for r in rs:
        p = preds.get(r["id"], {})
        flagged = bool(p.get("flagged"))
        det += flagged
        if flagged:
            fixed += nfc(p.get("fixed_text")).strip() == nfc(r["original"]).strip()
            if have_spans and p.get("spans"):
                spans = to_char_spans(r["text"], p["spans"], span_unit)
                loc += any(overlap(s, r["edit_span"], tol) for s in spans)
    return det, fixed, loc, len(rs)


def cell(pair):
    return {"n": pair[1], "k": pair[0], "pct": (100.0 * pair[0] / pair[1]) if pair[1] else None}


def dump_errors(path, bench, preds, have_spans, span_unit, tol):
    rows = []
    for r in bench:
        p = preds.get(r["id"], {})
        flagged = bool(p.get("flagged"))
        if not r["expected_flag"]:
            if flagged:
                rows.append({"id": r["id"], "kind": "false_positive", "doc": r["doc"],
                             "register": r["register"], "text": r["text"],
                             "original": r["original"], "pred": p})
            continue
        if not flagged:
            rows.append({"id": r["id"], "kind": "missed", "doc": r["doc"],
                         "operator": r["operator"], "register": r["register"],
                         "text": r["text"], "original": r["original"], "pred": p})
            continue
        if nfc(p.get("fixed_text")).strip() != nfc(r["original"]).strip():
            rows.append({"id": r["id"], "kind": "not_fixed", "doc": r["doc"],
                         "operator": r["operator"], "register": r["register"],
                         "text": r["text"], "original": r["original"], "pred": p})
        if have_spans and p.get("spans"):
            spans = to_char_spans(r["text"], p["spans"], span_unit)
            if not any(overlap(s, r["edit_span"], tol) for s in spans):
                rows.append({"id": r["id"], "kind": "misplaced", "doc": r["doc"],
                             "operator": r["operator"], "register": r["register"],
                             "text": r["text"], "original": r["original"], "pred": p})
    if path:
        Path(path).parent.mkdir(parents=True, exist_ok=True)
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            for row in rows:
                f.write(json.dumps(row, ensure_ascii=False) + "\n")
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bench", required=True)
    ap.add_argument("--pred", required=True)
    ap.add_argument("--min-conf", default="medium", choices=list(CONF_ORDER),
                    help="bỏ các ca có nhãn độ tin cậy thấp hơn mức này (mặc định: bỏ 'low')")
    ap.add_argument("--span-unit", default="char", choices=["char", "byte"])
    ap.add_argument("--tol", type=int, default=2,
                    help="sai số vị trí (ký tự) khi đối chiếu cảnh báo với lỗi")
    ap.add_argument("--json", help="ghi kết quả ra file JSON")
    ap.add_argument("--dump-errors", help="ghi các ca sai ra JSONL để phân tích")
    ap.add_argument("--by-doc", action="store_true", help="in thêm thống kê theo tài liệu")
    a = ap.parse_args()
    if a.tol < 0:
        ap.error("--tol phải >= 0")
    for path in (a.bench, a.pred):
        if not Path(path).exists():
            sys.exit(f"Không tìm thấy file: {path}")

    bench_all = load(a.bench)
    bench = [r for r in bench_all if CONF_ORDER[r["label_confidence"]] >= CONF_ORDER[a.min_conf]]
    preds, duplicate_ids, invalid_preds = load_preds(a.pred)
    bench_ids = {r["id"] for r in bench_all}
    unknown_ids = sorted(set(preds) - bench_ids)
    missing = [r["id"] for r in bench if r["id"] not in preds]
    have_spans = any(preds.get(r["id"], {}).get("spans") for r in bench)

    clean = [r for r in bench if not r["expected_flag"]]
    bad = [r for r in bench if r["expected_flag"]]
    missing_clean = [r["id"] for r in clean if r["id"] not in preds]
    missing_bad = [r["id"] for r in bad if r["id"] not in preds]

    fp = sum(bool(preds.get(r["id"], {}).get("flagged")) for r in clean)
    d, f, lm, n = stat(bad, preds, have_spans, a.span_unit, a.tol)
    precision = (d, d + fp)

    res = {
        "bench": str(a.bench),
        "pred": str(a.pred),
        "config": {
            "min_conf": a.min_conf,
            "span_unit": a.span_unit,
            "tol": a.tol,
        },
        "n_bench": len(bench),
        "n_clean": len(clean),
        "n_errors": len(bad),
        "missing_predictions": len(missing),
        "missing_clean": len(missing_clean),
        "missing_bad": len(missing_bad),
        "invalid_predictions": invalid_preds,
        "duplicate_predictions": duplicate_ids,
        "unknown_predictions": unknown_ids,
        "fpr": cell((fp, len(clean))),
        "fpr_covered": cell((fp, len(clean) - len(missing_clean))),
        "detection": cell((d, n)),
        "fixed": cell((f, n)),
        "located": cell((lm, n)) if have_spans else None,
        "precision": cell(precision) if precision[1] else None,
        "operators": {},
        "registers": {},
        "documents": {},
    }

    print(f"File: {a.bench} | ca dung de cham: {len(bench)} | thieu du doan: {len(missing)}")
    print(f"Bo cac nhan do tin cay < '{a.min_conf}'. Khoang tin cay = Wilson 95%.\n")
    print("BAO NHAM (cau sach bi canh bao): " + fmt(fp, len(clean)))
    if missing_clean:
        print(f"   (FPR tren {len(clean) - len(missing_clean)} cau sach co du doan: "
              f"{fmt(fp, len(clean) - len(missing_clean)).strip()})")
    print("PHAT HIEN (tong)               : " + fmt(d, n))
    print("SUA DUNG  (tong, khop ca cau)  : " + fmt(f, n))
    if have_spans:
        print("DUNG VI TRI (tong)             : " + fmt(lm, n))
    if precision[1]:
        print("DO CHINH XAC (precision, theo ca): " + fmt(*precision))

    rw = [r for r in bad if r["is_real_word"]]
    nrw = [r for r in bad if r["is_real_word"] is False]
    for title, rs in (("loi tao ra TU CO THAT (real-word, xap xi)", rw),
                      ("loi tao ra am tiet KHONG co that", nrw)):
        if rs:
            d2, f2, _, n2 = stat(rs, preds, have_spans, a.span_unit, a.tol)
            print(f"  - {title:<42} phat hien {fmt(d2, n2).strip()} | sua dung {fmt(f2, n2).strip()}")

    print("\nTHEO NHOM / TOAN TU (phat hien | sua dung):")
    groups = defaultdict(list)
    for r in bad:
        groups[(r["group"], r["operator"])].append(r)
    for (g, op), rs in sorted(groups.items()):
        d2, f2, l2, n2 = stat(rs, preds, have_spans, a.span_unit, a.tol)
        print(f"  {g:<16} {op:<22} {fmt(d2, n2)} | {fmt(f2, n2)}")
        res["operators"][op] = {"group": g, "n": n2, "detected": d2, "fixed": f2,
                                "located": l2, "detection": cell((d2, n2)),
                                "fixed_rate": cell((f2, n2))}

    regs = defaultdict(list)
    for r in bad:
        regs[r["register"]].append(r)
    print("\nTHEO VAN PHONG:")
    for reg, rs in sorted(regs.items()):
        d2, f2, l2, n2 = stat(rs, preds, have_spans, a.span_unit, a.tol)
        print(f"  {reg:<16} phat hien {fmt(d2, n2).strip()} | sua dung {fmt(f2, n2).strip()}")
        res["registers"][reg] = {"n": n2, "detected": d2, "fixed": f2, "located": l2,
                                 "detection": cell((d2, n2)), "fixed_rate": cell((f2, n2))}

    if a.by_doc:
        docs = defaultdict(list)
        for r in bad:
            docs[r["doc"]].append(r)
        print("\nTHEO TAI LIEU:")
        for doc, rs in sorted(docs.items()):
            d2, f2, _, n2 = stat(rs, preds, have_spans, a.span_unit, a.tol)
            print(f"  {doc[-45:]:<45} {fmt(d2, n2).strip()} | {fmt(f2, n2).strip()}")
            res["documents"][doc] = {"n": n2, "detected": d2, "fixed": f2,
                                     "detection": cell((d2, n2)),
                                     "fixed_rate": cell((f2, n2))}

    if invalid_preds:
        print(f"\nCANH BAO: {invalid_preds} du doan thieu id hop le (da bo qua).")
    if duplicate_ids:
        print(f"CANH BAO: {len(duplicate_ids)} id du doan bi trung (giu ban cuoi): "
              f"{', '.join(duplicate_ids[:8])}")
    if unknown_ids:
        print(f"CANH BAO: {len(unknown_ids)} du doan khong co trong benchmark: "
              f"{', '.join(unknown_ids[:8])}")
    if missing:
        print(f"CANH BAO: {len(missing)} ca khong co du doan (tinh la khong bao loi).")

    if a.dump_errors:
        rows = dump_errors(a.dump_errors, bench, preds, have_spans, a.span_unit, a.tol)
        print(f"Da ghi {len(rows)} ca sai -> {a.dump_errors}")
        res["dumped_errors"] = len(rows)

    if a.json:
        Path(a.json).parent.mkdir(parents=True, exist_ok=True)
        with open(a.json, "w", encoding="utf-8") as f_:
            json.dump(res, f_, ensure_ascii=False, indent=2)
        print(f"Da ghi ket qua -> {a.json}")


if __name__ == "__main__":
    main()
