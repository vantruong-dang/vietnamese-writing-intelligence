import init, { WasmAuditor } from './wasm/vwi_wasm.js';

let auditor = null;

const txtInput = document.getElementById('txtInput');
const btnAudit = document.getElementById('btnAudit');
const btnGrabSelection = document.getElementById('btnGrabSelection');
const btnClear = document.getElementById('btnClear');
const resultBox = document.getElementById('resultBox');
const scoreDisplay = document.getElementById('scoreDisplay');
const statsText = document.getElementById('statsText');
const findingsList = document.getElementById('findingsList');
const loadingIndicator = document.getElementById('loadingIndicator');

// 1. Khởi tạo WASM Engine khi popup mở ra
async function loadWasm() {
  try {
    loadingIndicator.style.display = 'block';
    await init();
    auditor = new WasmAuditor();
    loadingIndicator.style.display = 'none';

    // Tự động kiểm tra văn bản bôi đen trên tab hiện tại
    const selected = await getSelectionFromCurrentTab();
    if (selected) {
      txtInput.value = selected;
      runAudit();
    }
  } catch (err) {
    console.error('Lỗi nạp WASM:', err);
    loadingIndicator.innerText = 'Lỗi nạp WASM: ' + err.message;
  }
}

// 2. Trích xuất văn bản đang bôi đen hoặc đang soạn thảo qua activeTab
async function getSelectionFromCurrentTab() {
  try {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (!tab || !tab.id || tab.url?.startsWith('chrome://')) return '';

    const injectionResults = await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      func: () => {
        let text = window.getSelection()?.toString() || '';
        if (!text) {
          const el = document.activeElement;
          if (
            el &&
            (el.tagName === 'TEXTAREA' ||
              (el.tagName === 'INPUT' && el.type === 'text'))
          ) {
            const s = el.selectionStart;
            const e = el.selectionEnd;
            if (typeof s === 'number' && typeof e === 'number' && s !== e) {
              text = el.value.substring(s, e);
            } else if (el.value) {
              text = el.value;
            }
          }
        }
        return text;
      },
    });

    return injectionResults?.[0]?.result?.trim() || '';
  } catch (err) {
    console.debug('Không thể trích xuất văn bản từ tab:', err);
    return '';
  }
}

// 3. Thực thi kiểm toán văn bản qua Rust WebAssembly (100% Offline, < 2ms)
function runAudit() {
  const text = txtInput.value.trim();
  if (!text) {
    resultBox.style.display = 'none';
    return;
  }

  if (!auditor) {
    alert('Lõi WebAssembly đang được nạp, vui lòng bấm lại sau 1 giây.');
    return;
  }

  const startTime = performance.now();
  const reportJson = auditor.audit(text);
  const elapsed = (performance.now() - startTime).toFixed(1);
  const report = JSON.parse(reportJson);

  // Hiển thị khung kết quả
  resultBox.style.display = 'block';
  const score = Math.round(report.score);
  scoreDisplay.innerText = `${score}/100`;

  // Đổi màu điểm số theo chất lượng
  scoreDisplay.className =
    'score-val ' + (score >= 90 ? 'perfect' : score >= 70 ? 'warning' : 'danger');
  statsText.innerText = `${report.word_count} từ | ${report.findings.length} vi phạm (${elapsed} ms)`;

  if (report.findings.length === 0) {
    findingsList.innerHTML =
      '<div class="clean-state">✅ Văn bản thuần Việt chuẩn mực! Không phát hiện lỗi cấu trúc hay sáo ngữ.</div>';
    return;
  }

  const chars = Array.from(text);
  let html = '';
  report.findings.forEach((f) => {
    const sev = (f.severity || 'minor').toLowerCase();
    const snippet = chars.slice(f.start, f.end).join('');
    const rationale = f.rationale || f.message || '';
    const suggestion = f.suggestion || '';

    html += `
      <div class="item ${sev}">
        <div class="item-header">
          <span class="item-title">${escapeHtml(f.pattern_id || f.rule_id || 'VIOLATION')}</span>
          <span class="item-sev ${sev}">${escapeHtml(sev)}</span>
        </div>
        <div class="item-viol">Đoạn vi phạm: <u>"${escapeHtml(snippet || f.matched_span || '')}"</u></div>
        <div class="item-msg">${escapeHtml(rationale)}</div>
        ${suggestion ? `<div class="sugg">💡 Gợi ý: ${escapeHtml(suggestion)}</div>` : ''}
      </div>
    `;
  });

  findingsList.innerHTML = html;
}

function escapeHtml(str) {
  if (!str) return '';
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

// 4. Lắng nghe các sự kiện
btnAudit.addEventListener('click', runAudit);

btnGrabSelection.addEventListener('click', async () => {
  const selected = await getSelectionFromCurrentTab();
  if (!selected) {
    alert('Không tìm thấy đoạn văn bản nào đang bôi đen trên trang web!');
    return;
  }
  txtInput.value = selected;
  runAudit();
});

btnClear.addEventListener('click', () => {
  txtInput.value = '';
  resultBox.style.display = 'none';
  findingsList.innerHTML = '';
  txtInput.focus();
});

// Khởi chạy khi popup mở
loadWasm();
