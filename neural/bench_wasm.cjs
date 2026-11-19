// Benchmark WASM (Node/V8) cho VSM runtime.
//
//   wasm-pack build crates/vsm-wasm --target nodejs --out-dir pkg --release
//   python -m neural.export.make_fixture --full \
//       --out neural/fixtures/full_random.vsm \
//       --expected neural/fixtures/full_random_expected.txt
//   node neural/bench_wasm.cjs
//
// Đo forward đầy đủ (error + byte head) trên 2.000 byte, đúng chunk stride 128.

const fs = require("node:fs");
const path = require("node:path");
const { performance } = require("node:perf_hooks");
const { VsmScorer } = require("../crates/vsm-wasm/pkg/vsm_wasm.js");

const repository = path.resolve(__dirname, "..");
const modelPath =
  process.argv[2] || path.join(repository, "neural", "fixtures", "full_random.vsm");
const model = new VsmScorer(fs.readFileSync(modelPath));

const sentence = "Chúng ta cần kiểm tra lỗi chính tả và ngữ pháp tiếng Việt. ";
let text = "";
while (Buffer.byteLength(text) < 2000) {
  text += sentence;
}
const input = Buffer.from(text).subarray(0, 2000);

for (let index = 0; index < 3; index += 1) {
  model.error_argmax(input);
}

const samples = [];
const runs = 15;
for (let index = 0; index < runs; index += 1) {
  const start = performance.now();
  model.error_argmax(input);
  samples.push(performance.now() - start);
}
samples.sort((left, right) => left - right);
const average = samples.reduce((sum, value) => sum + value, 0) / samples.length;
console.log(
  JSON.stringify(
    {
      bytes: input.length,
      runs,
      min_ms: samples[0],
      median_ms: samples[Math.floor(samples.length / 2)],
      average_ms: average,
      max_ms: samples[samples.length - 1],
      chunks: Math.ceil(input.length / 128),
      model: path.basename(modelPath),
    },
    null,
    2,
  ),
);
