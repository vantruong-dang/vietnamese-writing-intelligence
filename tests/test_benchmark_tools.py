"""Kiểm thử bộ sinh và bộ chấm benchmark chính tả tiếng Việt.

Các test này không cần plugin/Rust: chỉ kiểm tra bất biến của dữ liệu sinh ra,
các toán tử tạo lỗi và định dạng báo cáo của bộ chấm điểm.
"""
from __future__ import annotations

import hashlib
import importlib.util
import json
import random
import subprocess
import sys
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load_tool(name: str):
    path = ROOT / "tools" / f"{name}.py"
    spec = importlib.util.spec_from_file_location(f"vwi_tool_{name}", path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


gen = load_tool("gen_benchmark")
score = load_tool("score_benchmark")


def assert_single_edit(result, before: str) -> None:
    after, start, end, orig_frag, new_frag = result
    assert after != before
    assert after[start:end] == new_frag
    assert after[:start] + orig_frag + after[end:] == before


def test_parse_build_roundtrip_and_tone_placement() -> None:
    for word in [
        "nghiêng", "người", "thuỷ", "hoà", "quyển", "giường",
        "khuỷu", "được", "chuyện", "trường", "nghỉ", "quốc",
    ]:
        units = gen.parse(word)
        assert units is not None, word
        assert gen.build(units) == unicodedata.normalize("NFC", word)
        assert gen.split_syl("".join(c for c, _ in units)) is not None

    # Kiểu mới: hoà, thuỷ (không phải hòa, thủy).
    assert gen.tone_target("thuy", 2, 4, 0) == 3
    assert gen.tone_target("hoa", 1, 3, 0) == 2
    assert gen.tone_target("tai", 1, 3, 0) == 1
    assert gen.tone_target("toan", 1, 3, 1) == 2

    units = gen.parse("thuy")
    pos = gen.tone_target("thuy", 2, 4, 0)
    units[pos][1] = "\u0309"
    assert gen.build(units) == "thuỷ"


def test_load_docs_skips_generated_stats_file(tmp_path: Path) -> None:
    corpus = tmp_path / "corpus"
    (corpus / "book").mkdir(parents=True)
    (corpus / "book" / "a.txt").write_text(
        "Hôm nay trời rất đẹp và mọi người vui vẻ.\n", encoding="utf-8")
    out = corpus / "bench"
    out.mkdir()
    (out / "stats.txt").write_text(
        "Hôm nay trời rất đẹp và mọi người vui vẻ.\n", encoding="utf-8")
    docs = gen.load_docs(corpus, ["admin"], 4, 40, out)
    assert [d[0] for d in docs] == ["book/a.txt"]


def test_good_sentence_accepts_quoted_dialogue() -> None:
    assert gen.good_sentence('"Hắn vừa đi vừa chửi, bao giờ cũng thế."', 4, 40)
    assert not gen.good_sentence('"hắn vừa đi vừa chửi, bao giờ cũng thế."', 4, 40)


def test_all_operators_keep_single_edit_invariant() -> None:
    general = [
        "Hắn vừa đi vừa chửi, bao giờ cũng thế, cứ rượu xong là hắn chửi.",
        "Chúng tôi cùng nhau xây dựng một cộng đồng vững mạnh và đoàn kết.",
        "Cô giáo nhẹ nhàng giải thích bài học cho học sinh trong lớp.",
        "Anh ấy đã dành cả buổi chiều để đọc sách ở thư viện.",
        "Gia đình tôi thường sum họp vào mỗi dịp cuối tuần.",
        "Trẻ em rất thích nghe kể chuyện trước khi đi ngủ.",
        "Ông nội tôi năm nay đã ngoài tám mươi tuổi.",
        "Buổi sáng, con đường làng tràn ngập ánh nắng.",
        "Mùa thu đến mang theo những cơn gió se lạnh.",
        "Nghỉ hè, chúng tôi thường về quê thăm ông bà.",
    ]
    admin = [
        "Hà Nội, ngày 05 tháng 01 năm 2020, cơ quan ban hành quyết định.",
        "Thực hiện theo khoản 2 Điều 5 Nghị định này.",
        "Quy định tại điểm a khoản 1 Điều 9 của văn bản.",
        "Căn cứ Điều 7 Luật Lưu trữ năm 2024.",
        "Mọi chính sách đều vì lợi ích của Nhân dân.",
        "Chính sách của Nhà nước luôn hướng về cộng đồng.",
        "Hội nghị được tổ chức tại Thành phố Hồ Chí Minh.",
        "Đoàn đại biểu đến thăm tại Thủ đô Hà Nội.",
        "Tỷ lệ đạt 6,5% trong năm nay.",
        "Tổng kinh phí là 1.500.000 đồng.",
        "Độc lập - Tự do - Hạnh phúc",
    ]
    texts = general + admin
    produced = set()
    for name, meta in gen.OPS.items():
        for text in texts:
            for seed in range(8):
                result = meta["fn"](text, random.Random(seed))
                if result is None:
                    continue
                assert_single_edit(result, text)
                produced.add(name)
                break
    assert produced == set(gen.OPS), f"toán tử không sinh được lỗi: {set(gen.OPS) - produced}"


def test_admin_operators_only_fire_in_the_right_context() -> None:
    def run(name: str, text: str):
        for seed in range(4):
            result = gen.OPS[name]["fn"](text, random.Random(seed))
            if result is not None:
                return result
        return None

    # 'khoản'/'điểm' phải ở giữa câu mới là lỗi khi viết hoa.
    assert run("legal_khoan_cap", "Thực hiện theo khoản 2 Điều 5 Nghị định này.") is not None
    assert run("legal_khoan_cap", "khoản 2 Điều 5 quy định trách nhiệm.") is None
    assert run("legal_diem_cap", "Quy định tại điểm a khoản 1 Điều 9.") is not None
    assert run("legal_diem_cap", "điểm a khoản 1 Điều 9 quy định trách nhiệm.") is None

    # R5 chỉ báo sau 'tại', không báo 'Thành phố Hồ Chí Minh' đứng đầu câu.
    assert run("tp_hcm_low", "Hội nghị tại Thành phố Hồ Chí Minh.") is not None
    assert run("tp_hcm_low", "Thành phố Hồ Chí Minh là trung tâm kinh tế.") is None
    assert run("tp_hcm_low", "Chúng tôi đến Thành phố Hồ Chí Minh.") is None

    # 'Nhà nước' là định ngữ sau 'cơ quan' thì viết thường mới đúng.
    assert run("nha_nuoc_low", "Chính sách của Nhà nước rất rõ ràng.") is not None
    assert run("nha_nuoc_low", "Theo cơ quan Nhà nước quy định.") is None

    # R11 chỉ nhận số thập phân có dấu %; 1.500.000 là cách viết đúng.
    assert run("decimal_dot", "Tỷ lệ đạt 6,5% trong năm nay.") is not None
    assert run("decimal_dot", "Tỷ lệ đạt 6,5 điểm.") is None
    assert run("thousand_comma", "Tổng kinh phí là 1.500.000 đồng.") is not None

    # Giữ nguyên khoảng trắng gốc, chỉ đổi đúng một ký tự.
    result = run("tp_hcm_low", "Hội nghị tại  Thành phố Hồ Chí Minh.")
    assert result is not None and result[4] == "tại  thành phố"
    result = run("date_day_zero", "Hà Nội, ngày  05 tháng 01 năm 2020.")
    assert result is not None and result[4] == "ngày  5"


def test_split_counts_balance_registers() -> None:
    assert gen._split_counts(6, [60, 30, 10]) == [3, 2, 1]
    assert gen._split_counts(3, [60, 30, 10]) == [1, 1, 1]
    docs = (
        [(f"hanhchinh/{i}.txt", "administrative", [], "") for i in range(6)]
        + [(f"general/{i}.txt", "general", [], "") for i in range(9)]
    )
    split = gen.assign_splits(docs, "salty", [60, 30, 10], stratify=True)
    for sp in ("dev", "test", "blind"):
        assert any(split[f"hanhchinh/{i}.txt"] == sp for i in range(6))
    plain = gen.assign_splits(docs, "salty", [60, 30, 10], stratify=False)
    assert all(plain[rel] == gen.bucket(rel, "salty", [60, 30, 10]) for rel, _, _, _ in docs)


def _write_corpus(root: Path) -> None:
    subjects = [
        "Hôm nay trời đẹp", "Chúng tôi cùng nhau", "Cô giáo nhẹ nhàng",
        "Anh ấy đã dành thời gian", "Gia đình tôi thường", "Trẻ em rất thích",
        "Ông nội tôi năm nay", "Buổi sáng con đường", "Mùa thu đến",
        "Những người nông dân", "Cơn mưa mùa hạ", "Đàn chim én",
        "Bác sĩ trong bệnh viện", "Học sinh lớp tôi", "Người lái đò",
    ]
    verbs = ["kể về", "mang đến", "chia sẻ", "xây dựng"]
    objects = [
        "một câu chuyện rất cảm động", "những kỷ niệm của tuổi thơ",
        "niềm vui và hy vọng", "một cộng đồng vững mạnh",
        "bài học về lòng nhân ái", "những con người bình dị",
        "ánh nắng của mùa xuân", "tình cảm gia đình ấm áp",
        "ước mơ của các em nhỏ", "công việc thường ngày",
        "những chuyến đi xa", "khoảng trời yên bình",
    ]

    general = root / "general"
    general.mkdir(parents=True)
    sentences = []
    for i in range(60):
        sentences.append(
            f"{subjects[i % 15]}, {verbs[(i // 15) % 4]} {objects[i % 12]}."
        )
    for i in range(20):
        chunk = sentences[i * 3:(i + 1) * 3]
        (general / f"book_{i:02d}.txt").write_text("\n".join(chunk) + "\n", encoding="utf-8")

    # Câu trùng lặp giữa hai tài liệu để kiểm tra khử rò rỉ.
    duplicate = sentences[0]
    with (general / "book_05.txt").open("a", encoding="utf-8") as f:
        f.write(duplicate + "\n")

    admin = root / "hanhchinh"
    admin.mkdir()
    admin_sentences = [
        "Hà Nội, ngày 05 tháng 01 năm 2020, cơ quan ban hành quyết định.",
        "Thực hiện theo khoản 2 Điều 5 Nghị định này.",
        "Quy định tại điểm a khoản 1 Điều 9 của văn bản.",
        "Căn cứ Điều 7 Luật Lưu trữ năm 2024.",
        "Mọi chính sách đều vì lợi ích của Nhân dân.",
        "Chính sách của Nhà nước luôn hướng về cộng đồng.",
        "Hội nghị được tổ chức tại Thành phố Hồ Chí Minh.",
        "Đoàn đại biểu đến thăm tại Thủ đô Hà Nội.",
        "Tỷ lệ đạt 6,5% trong năm nay.",
        "Tổng kinh phí là 1.500.000 đồng.",
        "Độc lập - Tự do - Hạnh phúc",
        "Hà Nội, ngày 09 tháng 02 năm 2021, hội nghị tổng kết.",
    ]
    for i in range(6):
        chunk = admin_sentences[i * 2:(i + 1) * 2]
        (admin / f"legal_{i:02d}.txt").write_text("\n".join(chunk) + "\n", encoding="utf-8")


def test_generate_benchmark_invariants(tmp_path: Path) -> None:
    corpus = tmp_path / "corpus"
    _write_corpus(corpus)
    out = tmp_path / "bench"
    cmd = [
        sys.executable, str(ROOT / "tools" / "gen_benchmark.py"),
        "--corpus", str(corpus), "--out", str(out), "--salt", "unit-test",
        "--per-op", "1", "--clean", "1", "--min-tokens", "4", "--max-tokens", "40",
        "--ratio", "50,30,20",
    ]
    subprocess.run(cmd, check=True, capture_output=True, text=True, encoding="utf-8")

    manifest = json.loads((out / "MANIFEST.json").read_text(encoding="utf-8"))
    assert manifest["dropped_duplicate_sentences"] >= 1
    assert manifest["counts"]["dev"]["total"] > 0
    # Không được để lộ salt trong manifest.
    assert "unit-test" not in json.dumps(manifest)
    assert manifest["params"]["salt_fingerprint"]
    originals = []
    seen_admin_ops = set()
    for split in ("dev", "test", "blind"):
        path = out / f"{split}.jsonl"
        assert path.exists()
        assert manifest["files"][split] == hashlib.sha256(path.read_bytes()).hexdigest()
        rows = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]
        ids = [r["id"] for r in rows]
        assert len(ids) == len(set(ids))
        for r in rows:
            originals.append(r["original"])
            if r["expected_flag"]:
                s, e = r["edit_span"]
                assert r["text"] != r["original"]
                assert r["text"][s:e] == r["new_frag"]
                assert r["text"][:s] + r["orig_frag"] + r["text"][e:] == r["original"]
                assert r["group"] != "clean"
                meta = gen.OPS[r["operator"]]
                if meta["token"]:
                    assert isinstance(r["is_real_word"], bool)
                else:
                    assert r["is_real_word"] is None
                if meta["admin"]:
                    assert r["register"] == "administrative"
                    seen_admin_ops.add(r["operator"])
            else:
                assert r["text"] == r["original"]
                assert r["operator"] is None and r["edit_span"] is None
    assert len(originals) == len(set(originals)), "câu bị rò rỉ giữa các tập"
    assert seen_admin_ops, "không sinh được ca hành chính nào"


def test_generated_benchmark_is_perfectly_scorable(tmp_path: Path) -> None:
    corpus = tmp_path / "corpus"
    _write_corpus(corpus)
    out = tmp_path / "bench"
    gen_cmd = [
        sys.executable, str(ROOT / "tools" / "gen_benchmark.py"),
        "--corpus", str(corpus), "--out", str(out), "--salt", "perfect-score",
        "--per-op", "1", "--clean", "2", "--min-tokens", "4", "--max-tokens", "40",
        "--ratio", "50,30,20",
    ]
    subprocess.run(gen_cmd, check=True, capture_output=True, text=True, encoding="utf-8")

    bench_rows = []
    preds = []
    for split in ("dev", "test", "blind"):
        path = out / f"{split}.jsonl"
        if not path.exists():
            continue
        for line in path.read_text(encoding="utf-8").splitlines():
            r = json.loads(line)
            bench_rows.append(r)
            if r["expected_flag"]:
                preds.append({
                    "id": r["id"], "flagged": True, "fixed_text": r["original"],
                    "spans": [r["edit_span"]],
                })
            else:
                preds.append({"id": r["id"], "flagged": False})
    bench = tmp_path / "all.jsonl"
    pred = tmp_path / "perfect.jsonl"
    _write_jsonl(bench, bench_rows)
    _write_jsonl(pred, preds)
    out_json = tmp_path / "score.json"
    score_cmd = [
        sys.executable, str(ROOT / "tools" / "score_benchmark.py"),
        "--bench", str(bench), "--pred", str(pred), "--json", str(out_json),
    ]
    subprocess.run(score_cmd, check=True, capture_output=True, text=True, encoding="utf-8")
    res = json.loads(out_json.read_text(encoding="utf-8"))
    assert res["fpr"]["k"] == 0
    assert res["detection"]["k"] == res["detection"]["n"]
    assert res["fixed"]["k"] == res["fixed"]["n"]
    assert res["located"]["k"] == res["located"]["n"]
    assert res["precision"]["k"] == res["precision"]["n"]


def _bench_record(rid: str, *, bad: bool, text: str, original: str,
                  edit_span=None, operator=None, group="clean",
                  conf="high", register="general", real_word=None) -> dict:
    return {
        "id": rid, "split": "test", "doc": "doc-1", "register": register,
        "text": text, "expected_flag": bad, "original": original,
        "operator": operator, "group": group, "rule": "R12" if bad else None,
        "label_confidence": conf, "edit_span": edit_span,
        "orig_frag": None, "new_frag": None, "is_real_word": real_word,
    }


def _write_jsonl(path: Path, rows: list[dict]) -> None:
    path.write_text(
        "\n".join(json.dumps(r, ensure_ascii=False) for r in rows) + "\n",
        encoding="utf-8",
    )


def test_score_benchmark_metrics_and_error_dump(tmp_path: Path) -> None:
    clean1 = _bench_record("clean-1", bad=False, text="Hôm nay trời đẹp.",
                           original="Hôm nay trời đẹp.")
    clean2 = _bench_record("clean-2", bad=False, text="Chúng tôi đi làm.",
                           original="Chúng tôi đi làm.")
    bad1 = _bench_record("bad-1", bad=True, text="Tôi đi hoc.", original="Tôi đi học.",
                         edit_span=[7, 10], operator="tone", group="spelling")
    bad2 = _bench_record("bad-2", bad=True, text="Anh ấy rất vui .",
                         original="Anh ấy rất vui.", edit_span=[14, 15],
                         operator="space_before_comma", group="punct")
    bad3 = _bench_record("bad-3", bad=True, text="Giá 1,500 đồng.",
                         original="Giá 1.500 đồng.", edit_span=[4, 11],
                         operator="thousand_comma", group="admin_R11",
                         conf="low", register="administrative")
    bench = tmp_path / "bench.jsonl"
    _write_jsonl(bench, [clean1, clean2, bad1, bad2, bad3])
    pred = tmp_path / "pred.jsonl"
    _write_jsonl(pred, [
        {"id": "clean-1", "flagged": False},
        {"id": "clean-2", "flagged": True, "fixed_text": "Chúng tôi đi làm.",
         "spans": [[0, 1]]},
        {"id": "bad-1", "flagged": True, "fixed_text": "Tôi đi học.",
         "spans": [[7, 10]]},
        {"id": "bad-2", "flagged": True, "fixed_text": "Anh ấy rất vui!",
         "spans": [[0, 1]]},
        {"id": "bad-3", "flagged": False},
    ])
    out_json = tmp_path / "score.json"
    errors = tmp_path / "errors.jsonl"
    cmd = [
        sys.executable, str(ROOT / "tools" / "score_benchmark.py"),
        "--bench", str(bench), "--pred", str(pred),
        "--json", str(out_json), "--dump-errors", str(errors),
    ]
    subprocess.run(cmd, check=True, capture_output=True, text=True, encoding="utf-8")
    res = json.loads(out_json.read_text(encoding="utf-8"))
    assert res["n_clean"] == 2 and res["n_errors"] == 2  # ca low-conf bị loại
    assert res["fpr"]["k"] == 1 and res["fpr"]["n"] == 2
    assert res["detection"]["k"] == 2 and res["detection"]["n"] == 2
    assert res["fixed"]["k"] == 1 and res["fixed"]["n"] == 2
    assert res["located"]["k"] == 1 and res["located"]["n"] == 2
    assert res["precision"]["k"] == 2 and res["precision"]["n"] == 3
    assert res["missing_predictions"] == 0
    assert res["missing_clean"] == 0 and res["missing_bad"] == 0

    kinds = [json.loads(line)["kind"] for line in errors.read_text(encoding="utf-8").splitlines()]
    assert sorted(kinds) == ["false_positive", "misplaced", "not_fixed"]


def test_score_benchmark_byte_spans_missing_and_duplicate(tmp_path: Path) -> None:
    clean = _bench_record("clean-1", bad=False, text="Câu này sạch sẽ.",
                          original="Câu này sạch sẽ.")
    bad1 = _bench_record("bad-1", bad=True, text="Tôi đi hoc.", original="Tôi đi học.",
                         edit_span=[7, 10], operator="tone", group="spelling")
    bad2 = _bench_record("bad-2", bad=True, text="Anh ấy rất vui .",
                         original="Anh ấy rất vui.", edit_span=[14, 15],
                         operator="space_before_comma", group="punct")
    bench = tmp_path / "bench.jsonl"
    _write_jsonl(bench, [clean, bad1, bad2])

    text = bad1["text"]
    byte_span = [len(text[:7].encode("utf-8")), len(text[:10].encode("utf-8"))]
    pred = tmp_path / "pred.jsonl"
    _write_jsonl(pred, [
        {"id": "clean-1", "flagged": False},
        {"id": "bad-1", "flagged": True, "fixed_text": bad1["original"],
         "spans": [byte_span]},
        {"id": "bad-1", "flagged": True, "fixed_text": bad1["original"],
         "spans": [byte_span]},  # id trùng: giữ bản cuối
        {"id": "khong-co-trong-bench", "flagged": True},
        {"flagged": True},  # thiếu id: phải bị bỏ qua, không làm sập bộ chấm
        # bad-2 cố ý thiếu dự đoán
    ])
    out_json = tmp_path / "score.json"
    cmd = [
        sys.executable, str(ROOT / "tools" / "score_benchmark.py"),
        "--bench", str(bench), "--pred", str(pred),
        "--span-unit", "byte", "--json", str(out_json),
    ]
    subprocess.run(cmd, check=True, capture_output=True, text=True, encoding="utf-8")
    res = json.loads(out_json.read_text(encoding="utf-8"))
    assert res["missing_predictions"] == 1
    assert res["missing_clean"] == 0 and res["missing_bad"] == 1
    assert res["fpr_covered"]["n"] == 1
    assert res["detection"]["k"] == 1 and res["detection"]["n"] == 2
    assert res["located"]["k"] == 1 and res["located"]["n"] == 2
    assert res["duplicate_predictions"] == ["bad-1"]
    assert res["unknown_predictions"] == ["khong-co-trong-bench"]
    assert res["invalid_predictions"] == 1
