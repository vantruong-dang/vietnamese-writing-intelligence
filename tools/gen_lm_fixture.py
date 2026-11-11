from pathlib import Path

root = Path(__file__).resolve().parent.parent
bigram_src = root / "data_processed" / "bigram_lm.tsv"
unigram_src = root / "data_processed" / "unigram_lm.tsv"
out_bigram = root / "crates" / "vwi-core" / "data" / "bigram_lm_test.tsv"
out_unigram = root / "crates" / "vwi-core" / "data" / "unigram_lm_test.tsv"

targets = [
    ("cơn", "bão"),
    ("buổi", "sáng"),
    ("giấy", "tờ"),
    ("bên", "cạnh"),
    ("chiếc", "váy"),
]

rows = []
for line in bigram_src.read_text(encoding="utf-8").splitlines():
    if not line or line.startswith("#"):
        continue
    parts = line.split("\t")
    if len(parts) < 3:
        continue
    rows.append((parts[0], parts[1], int(parts[2])))

by_pair = {(a, b): c for a, b, c in rows}

selected = []
seen = set()
for a, b in targets:
    count = by_pair.get((a, b))
    if count is None:
        raise SystemExit(f"missing target bigram: {a} {b}")
    selected.append(f"{a}\t{b}\t{count}")
    seen.add((a, b))

rows.sort(key=lambda r: -r[2])
for a, b, c in rows:
    if len(selected) >= 120:
        break
    if (a, b) in seen:
        continue
    selected.append(f"{a}\t{b}\t{c}")
    seen.add((a, b))

out_bigram.write_text(
    "# w1\tw2\tcount\n" + "\n".join(selected) + "\n", encoding="utf-8"
)

uni = {}
for line in unigram_src.read_text(encoding="utf-8").splitlines():
    if not line or line.startswith("#"):
        continue
    parts = line.split("\t")
    if len(parts) >= 2:
        uni[parts[0]] = int(parts[1])

words = sorted({w for a, b in seen for w in (a, b)})
out_unigram.write_text(
    "# word\tcount\n" + "\n".join(f"{w}\t{uni.get(w, 1)}" for w in words) + "\n",
    encoding="utf-8",
)
print(f"bigram lines={len(selected)} unigram words={len(words)}")
