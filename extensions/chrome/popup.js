import init, { WasmAuditor } from './wasm/vwi_wasm.js';

let auditor = null;
const DRAFT_STORAGE_KEY = 'vwi_current_draft';

// DOM Elements
const txtInput = document.getElementById('txtInput');
const btnAudit = document.getElementById('btnAudit');
const btnGetSelection = document.getElementById('btnGetSelection');
const btnClear = document.getElementById('btnClear');
const btnSidePanel = document.getElementById('btnSidePanel');
const btnOpenStudio = document.getElementById('btnOpenStudio');
const resultsBox = document.getElementById('resultsBox');
const scoreValue = document.getElementById('scoreValue');
const metaInfo = document.getElementById('metaInfo');
const findingsContainer = document.getElementById('findingsContainer');
const emptyNotice = document.getElementById('emptyNotice');

// 1. ĐÓN NHẬN VĂN BẢN TỪ CONTEXT MENU (CHUỘT PHẢI)
async function consumeSelectedText() {
  if (!chrome?.storage?.session) return false;
  try {
    const data = await chrome.storage.session.get("vwi_selected_text");
    if (data?.vwi_selected_text && data.vwi_selected_text.trim()) {
      txtInput.value = data.vwi_selected_text;
      localStorage.setItem(DRAFT_STORAGE_KEY, data.vwi_selected_text);
      executeAudit(data.vwi_selected_text);
      await chrome.storage.session.remove("vwi_selected_text");
      return true;
    }
  } catch (err) {
    console.debug("Lỗi đọc session storage:", err);
  }
  return false;
}

// Lắng nghe liên tục khi Side Panel ĐANG MỞ SẴN và người dùng bôi đen đoạn khác rồi chuột phải
if (chrome?.storage?.onChanged) {
  chrome.storage.onChanged.addListener((changes, areaName) => {
    if (areaName === "session" && changes.vwi_selected_text?.newValue) {
      consumeSelectedText();
    }
  });
}

// 2. KHỞI TẠO WASM & NẠP DRAFT TỰ ĐỘNG
async function initApp() {
  try {
    await init();
    auditor = new WasmAuditor();
    console.log("VWI WASM Engine đã nạp thành công!");

    // Ưu tiên 1: Đón nhận văn bản vừa bôi đen từ chuột phải
    const hasContextText = await consumeSelectedText();

    // Ưu tiên 2: Nếu không có từ chuột phải, phục hồi bản nháp đã lưu
    if (!hasContextText) {
      const savedDraft = localStorage.getItem(DRAFT_STORAGE_KEY);
      if (savedDraft && savedDraft.trim()) {
        txtInput.value = savedDraft;
        executeAudit(savedDraft);
      } else {
        if (window.innerWidth >= 850 && emptyNotice) {
          emptyNotice.style.display = 'block';
        }
      }
    }
  } catch (err) {
    console.error("Lỗi khởi tạo WebAssembly:", err);
  }
}
initApp();

// 3. TỰ ĐỘNG LƯU BẢN NHÁP VÀO LOCALSTORAGE
txtInput.addEventListener('input', (e) => {
  localStorage.setItem(DRAFT_STORAGE_KEY, e.target.value);
});

// 4. NÚT KIỂM TRA
btnAudit.addEventListener('click', () => {
  const text = txtInput.value.trim();
  if (!text) return;
  executeAudit(text);
});

// 5. HÀM CHẠY LÕI PHÂN TÍCH RUST WASM
function executeAudit(rawText) {
  if (!auditor || !rawText.trim()) return;

  const startTime = performance.now();
  const reportJson = auditor.audit(rawText);
  const duration = (performance.now() - startTime).toFixed(1);
  const report = JSON.parse(reportJson);

  renderReport(report, rawText, duration);
}

// 6. HIỂN THỊ KẾT QUẢ RA GIAO DIỆN
function renderReport(report, text, durationMs) {
  if (emptyNotice) emptyNotice.style.display = 'none';
  resultsBox.style.display = 'flex';

  // Hiển thị Điểm
  const roundedScore = Math.round(report.score);
  scoreValue.innerText = `${roundedScore}/100`;

  if (roundedScore >= 80) {
    scoreValue.style.color = '#16a34a'; // Xanh lá
  } else if (roundedScore >= 60) {
    scoreValue.style.color = '#d97706'; // Vàng cam
  } else {
    scoreValue.style.color = '#dc2626'; // Đỏ
  }

  // Thông số thống kê
  metaInfo.innerText = `${report.word_count} từ | ${report.findings.length} góp ý (${durationMs} ms)`;

  // Danh sách lỗi
  if (report.findings.length === 0) {
    findingsContainer.innerHTML = `
      <div class="clean-alert">
        <span>✅</span>
        <span>Văn bản thuần Việt chuẩn mực! Không phát hiện lỗi cấu trúc hay sáo ngữ.</span>
      </div>
    `;
    return;
  }

  const chars = Array.from(text);
  let html = '';

  report.findings.forEach(f => {
    const isMajor = (f.severity || '').toLowerCase() === 'major';
    const snippet = chars.slice(f.start, f.end).join('') || f.matched_span || '';
    const badgeClass = isMajor ? 'major' : 'minor';
    const badgeLabel = isMajor ? 'MAJOR' : 'MINOR';
    const ruleId = f.rule_id || f.pattern_id || 'VIOLATION';
    const desc = f.message || f.rationale || '';

    html += `
      <div class="finding-card ${badgeClass}">
        <div class="finding-top">
          <span class="finding-rule">${escapeHtml(ruleId)}</span>
          <span class="severity-tag ${badgeClass}">${badgeLabel}</span>
        </div>
        <div style="font-size: 13px;">Đoạn cần lưu ý: <span class="snippet-text">"${escapeHtml(snippet)}"</span></div>
        <div class="finding-desc">${escapeHtml(desc)}</div>
        ${f.suggestion ? `<div class="suggestion-alert">💡 <b>Gợi ý:</b> ${escapeHtml(f.suggestion)}</div>` : ''}
      </div>
    `;
  });

  findingsContainer.innerHTML = html;
}

// 7. LẤY CHỮ BÔI ĐEN TRÊN TRANG WEB (SendMessage + Scripting Fallback)
btnGetSelection.addEventListener('click', async () => {
  try {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (!tab?.id || tab.url?.startsWith('chrome://')) {
      alert("Không thể trích xuất văn bản từ trang hệ thống của trình duyệt!");
      return;
    }

    chrome.tabs.sendMessage(tab.id, { action: "getSelectedText" }, async (response) => {
      let selectedText = response?.text;

      if (chrome.runtime.lastError || !selectedText) {
        try {
          const results = await chrome.scripting.executeScript({
            target: { tabId: tab.id },
            func: () => {
              let t = window.getSelection()?.toString() || "";
              if (!t) {
                const el = document.activeElement;
                if (el && (el.tagName === "TEXTAREA" || (el.tagName === "INPUT" && el.type === "text"))) {
                  const s = el.selectionStart;
                  const e = el.selectionEnd;
                  if (typeof s === "number" && typeof e === "number" && s !== e) {
                    t = el.value.substring(s, e);
                  } else if (el.value) {
                    t = el.value;
                  }
                }
              }
              return t.trim();
            }
          });
          selectedText = results?.[0]?.result || "";
        } catch (e) {
          console.debug("Lỗi fallback executeScript:", e);
        }
      }

      if (!selectedText) {
        alert("Vui lòng bôi đen đoạn văn bản trên trang web trước!");
        return;
      }

      txtInput.value = selectedText;
      localStorage.setItem(DRAFT_STORAGE_KEY, selectedText);
      executeAudit(selectedText);
    });
  } catch (err) {
    console.warn("Không thể lấy vùng chọn:", err);
  }
});

// 8. XÓA NỘI DUNG
btnClear.addEventListener('click', () => {
  txtInput.value = '';
  localStorage.removeItem(DRAFT_STORAGE_KEY);
  resultsBox.style.display = 'none';
  if (emptyNotice && window.innerWidth >= 850) {
    emptyNotice.style.display = 'block';
  }
  txtInput.focus();
});

// 9. ĐIỀU HƯỚNG MỞ SIDE PANEL
btnSidePanel.addEventListener('click', async () => {
  try {
    const win = await chrome.windows.getCurrent();
    await chrome.sidePanel.open({ windowId: win.id });
    window.close(); // Đóng popup khi Side Panel đã mở
  } catch (err) {
    console.error("Lỗi mở Side Panel:", err);
  }
});

// 10. ĐIỀU HƯỚNG MỞ TAB STUDIO TOÀN MÀN HÌNH
btnOpenStudio.addEventListener('click', () => {
  const studioUrl = chrome.runtime.getURL("popup.html?mode=studio");
  chrome.tabs.create({ url: studioUrl });
});

// Hàm escape chống XSS nội bộ
function escapeHtml(str) {
  if (!str) return '';
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}
