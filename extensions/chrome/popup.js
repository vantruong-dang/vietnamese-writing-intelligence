import init, { WasmAuditor } from './wasm/vwi_wasm.js';

let auditor = null;
let debounceTimer = null;
const DRAFT_STORAGE_KEY = 'vwi_current_draft';
const DISPLAY_MODE_KEY = 'vwi_display_mode';
let displayMode = localStorage.getItem(DISPLAY_MODE_KEY) || 'friendly'; // 'friendly' | 'dev'

// Cache báo cáo phân tích gần nhất để chuyển chế độ hiển thị tức thì
let lastReport = null;
let lastRawText = '';
let lastDurationMs = '0.0';

// Bản đồ tên luật thân thiện tiếng Việt cho người dùng phổ thông
const RULE_LABELS_VI = {
  // Cú pháp & Khung đề (Syntax & Topic-Comment)
  'CRIPPLED_PREPOSITIONAL_TOPIC': 'Câu thiếu chủ ngữ',
  'PREPOSITION_CONJUNCTION_CONFLICT': 'Xung đột Giới từ - Liên từ',
  'CXH_REPORTING_VERB_INCOMPLETE_CLAUSE': 'Câu cụt sau động từ phát ngôn',
  'CXH_PARALLEL_CONJUNCTION_ASYMMETRY': 'Lệch vế liên từ song song',
  'CXH_HYBRID_COMPLEX_CONJUNCTION': 'Lai tạp liên từ phát ngôn',

  // Bị động & Thụ hưởng (Passives)
  'ARTIFICIAL_PASSIVE_BE_BY': "Bị động dịch máy ('được/bị... bởi')",
  'REDUNDANT_BENEFICIARY_PASSIVE': "Lạm dụng từ 'bị/được' cho đồ vật",

  // Dịch thuật & Sao phỏng ngoại ngữ (Translationese)
  'NOMINALIZATION_HEAVY': "Lạm dụng danh từ hóa ('việc/sự')",
  'TEMPORAL_IN_TRANSLATIONESE': "Dịch thô mốc thời gian ('trong 3 ngày')",
  'TRANSLATIONESE_SOLEIL_SUNSHINE': 'Sao phỏng lối diễn đạt thô (ánh nắng...)',
  'DUMMY_IT_TRANSLATIONESE': "Chủ ngữ giả Tây phương ('Nó là...')",
  'ADJECTIVAL_SUFFIX_FAKE': 'Hậu tố tính từ khiên cưỡng (- mang tính)',

  // Hành chính, Hư từ & Sáo ngữ (Bureaucracy & Cliches)
  'BUREAUCRATIC_VERB_PADDING': "Đệm động từ rỗng ('tiến hành thực hiện')",
  'REDUNDANT_CONNECTORS': "Chêm liên từ thừa ('và đồng thời')",
  'VAGUE_INTENSIFIER': 'Hư từ khoa trương / sáo rỗng',
  'AI_OPENING_CLICHE': 'Mở đầu rập khuôn kiểu AI',
  'AI_SUMMARY_TAIL': 'Kết bài đúc kết sáo rỗng kiểu AI',
  'FORMULAIC_METAPHOR': 'Ẩn dụ công thức / sáo ngữ',
  'AI_DISCOVERED_CLICHE': 'Sáo ngữ AI phổ biến',

  // Nhượng bộ & Liên từ (Concessives)
  'FACTUAL_CONCESSIVE_MISUSE': "Dùng sai 'Tuy' cho giả định",
  'HYPOTHETICAL_CONCESSIVE_MISUSE': "Dùng sai 'Dù' cho hiện thực",
  'CONCESSIVE_INVERSION_CONFLICT': 'Đảo trật tự liên từ nhượng bộ',

  // Từ vựng & Phó từ (Lexical & Aspects)
  'CXH_VERBAL_SERIES_TENSE_FALSE_MARKER': 'Chuỗi vị từ gán thì giả tạo (đã/đang/sẽ)',
  'CXH_SUPERLATIVE_CHI_IT_MISUSE': "Dùng sai từ 'chí ít'",
  'CXH_SUYT_NUA_THI_CONTAMINATION': 'Lai tạp phó từ suýt nữa / suýt nữa thì',
  'LOGICAL_PLEONASM': "Trùng ngữ / Lặp nghĩa ('tối ưu nhất')",

  // Chính tả & Trình bày (Decree 30, Phonotactics & Orthography)
  'PUNCTUATION_SPACE_BEFORE': 'Thừa dấu cách trước dấu câu',
  'PUNCTUATION_NO_SPACE_AFTER': 'Thiếu dấu cách sau dấu câu',
  'BRACKET_SPACE_INSIDE': 'Thừa dấu cách trong ngoặc / nháy',
  'SENTENCE_CAPITALIZATION': 'Chưa viết hoa đầu câu',
  'DECREE30_AGENCY_CAPITALIZATION': 'Viết hoa sai tên cơ quan (NĐ 30)',
  'DECREE30_ADMIN_UNIT_LOWERCASE': 'Viết hoa sai địa danh hành chính',
  'ORTHOGRAPHY_TONE_INCONSISTENCY': 'Lẫn lộn dấu thanh mới và cũ',
  'TELEX_TYPO_ERROR': 'Lỗi dính phím gõ Telex',

  // Âm luật (Prosody)
  'PROSODIC_EXCESSIVE_TRAC': 'Mất cân đối thanh điệu (chuỗi thanh Trắc liên tiếp)',
};

// Nhãn mức độ nghiêm trọng thân thiện tiếng Việt
const SEVERITY_LABELS_VI = {
  'major': '🔴 LỖI NẶNG',
  'minor': '🟡 GÓP Ý',
  'advisory': '🔵 TƯ VẤN',
  'critical': '🚨 NGHIÊM TRỌNG',
};

// DOM Elements
const txtInput = document.getElementById('txtInput');
const btnAudit = document.getElementById('btnAudit');
const btnGetSelection = document.getElementById('btnGetSelection');
const btnClear = document.getElementById('btnClear');
const btnSidePanel = document.getElementById('btnSidePanel');
const btnOpenStudio = document.getElementById('btnOpenStudio');
const btnToggleMode = document.getElementById('btnToggleMode');
const lblModeIcon = document.getElementById('lblModeIcon');
const lblMode = document.getElementById('lblMode') || document.getElementById('lblModeText');
const resultsBox = document.getElementById('resultsBox');
const scoreCard = document.getElementById('scoreCard');
const syncStatus = document.getElementById('syncStatus');
const scoreValue = document.getElementById('scoreValue');
const metaInfo = document.getElementById('metaInfo');
const findingsContainer = document.getElementById('findingsContainer');
const emptyNotice = document.getElementById('emptyNotice');

// Cập nhật giao diện nút chuyển đổi chế độ
function updateModeButtonUI() {
  if (displayMode === 'friendly') {
    if (lblModeIcon) lblModeIcon.innerText = '⚙️';
    if (lblMode) lblMode.innerText = 'Dễ hiểu';
    if (btnToggleMode) btnToggleMode.title = 'Đang ở chế độ Dễ hiểu. Nhấp để chuyển sang chế độ Dành cho Dev (xem mã lỗi)';
  } else {
    if (lblModeIcon) lblModeIcon.innerText = '🇻🇳';
    if (lblMode) lblMode.innerText = 'Dành cho Dev';
    if (btnToggleMode) btnToggleMode.title = 'Đang ở chế độ Dành cho Dev. Nhấp để quay về chế độ Dễ hiểu';
  }
}

// 1. ĐÓN NHẬN VĂN BẢN TỪ CONTEXT MENU (CHUỘT PHẢI)
async function consumeSelectedText() {
  let text = '';
  if (chrome?.storage?.session) {
    try {
      const data = await chrome.storage.session.get("vwi_selected_text");
      if (data?.vwi_selected_text && data.vwi_selected_text.trim()) {
        text = data.vwi_selected_text;
        await chrome.storage.session.remove("vwi_selected_text");
      }
    } catch (e) {
      console.debug("Lỗi đọc session storage:", e);
    }
  }

  // Fallback đọc từ storage.local
  if (!text && chrome?.storage?.local) {
    try {
      const data = await chrome.storage.local.get(["selectedText", "vwi_selected_text"]);
      if (data?.selectedText && data.selectedText.trim()) {
        text = data.selectedText;
        await chrome.storage.local.remove("selectedText");
      } else if (data?.vwi_selected_text && data.vwi_selected_text.trim()) {
        text = data.vwi_selected_text;
        await chrome.storage.local.remove("vwi_selected_text");
      }
    } catch (e) {
      console.debug("Lỗi đọc local storage:", e);
    }
  }

  if (text && text.trim()) {
    txtInput.value = text;
    localStorage.setItem(DRAFT_STORAGE_KEY, text);
    executeAudit(text);
    return true;
  }
  return false;
}

// Lắng nghe liên tục khi Side Panel ĐANG MỞ SẴN và người dùng chuột phải câu khác
if (chrome?.storage?.onChanged) {
  chrome.storage.onChanged.addListener((changes, areaName) => {
    if (
      (areaName === "session" && changes.vwi_selected_text?.newValue) ||
      (areaName === "local" && (changes.selectedText?.newValue || changes.vwi_selected_text?.newValue))
    ) {
      consumeSelectedText();
    }
  });
}

// 2. KHỞI TẠO WASM & NẠP DRAFT TỰ ĐỘNG
async function initApp() {
  try {
    await init();
    auditor = new WasmAuditor();
    console.log("VWI WASM Engine đã sẵn sàng!");

    // Cập nhật nút chế độ hiển thị
    updateModeButtonUI();

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

// 3. LIVE AUTO-CHECK (DEBOUNCE 350MS) & ĐỔI TRẠNG THÁI THỊ GIÁC
txtInput.addEventListener('input', (e) => {
  const text = e.target.value;
  localStorage.setItem(DRAFT_STORAGE_KEY, text);

  // Đổi trạng thái sang màu vàng (Đang nhập) + làm mờ bảng điểm
  if (syncStatus) {
    syncStatus.className = 'status-tag status-stale';
    syncStatus.innerText = '○ Đang nhập...';
  }
  if (scoreCard) {
    scoreCard.classList.add('results-stale');
  }

  // Hẹn giờ tự động chạy sau 350ms ngừng gõ
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    if (text.trim()) {
      executeAudit(text);
    } else {
      resultsBox.style.display = 'none';
      if (emptyNotice && window.innerWidth >= 850) {
        emptyNotice.style.display = 'block';
      }
      if (syncStatus) {
        syncStatus.className = 'status-tag status-fresh';
        syncStatus.innerText = '• Sẵn sàng';
      }
      if (scoreCard) {
        scoreCard.classList.remove('results-stale');
      }
    }
  }, 350);
});

// 4. NÚT KIỂM TRA THỦ CÔNG & NÚT CHUYỂN ĐỔI CHẾ ĐỘ HIỂN THỊ
btnAudit.addEventListener('click', () => {
  const text = txtInput.value.trim();
  if (!text) return;
  executeAudit(text);
});

if (btnToggleMode) {
  btnToggleMode.addEventListener('click', () => {
    displayMode = displayMode === 'friendly' ? 'dev' : 'friendly';
    localStorage.setItem(DISPLAY_MODE_KEY, displayMode);
    updateModeButtonUI();
    if (lastReport && lastRawText) {
      renderReport(lastReport, lastRawText, lastDurationMs);
    }
  });
}

// 5. HÀM CHẠY LÕI PHÂN TÍCH RUST WASM
function executeAudit(rawText) {
  if (!auditor || !rawText.trim()) return;

  const startTime = performance.now();
  const reportJson = auditor.audit(rawText);
  const durationMs = (performance.now() - startTime).toFixed(1);
  const report = JSON.parse(reportJson);

  lastReport = report;
  lastRawText = rawText;
  lastDurationMs = durationMs;

  renderReport(report, rawText, durationMs);
}

// 6. HIỂN THỊ KẾT QUẢ RA GIAO DIỆN & HIỆU ỨNG NHÁY SÁNG
function renderReport(report, text, durationMs) {
  if (emptyNotice) emptyNotice.style.display = 'none';
  resultsBox.style.display = 'flex';

  // Cập nhật trạng thái màu xanh + thời gian thực thi
  if (syncStatus) {
    syncStatus.className = 'status-tag status-fresh';
    syncStatus.innerText = `● Vừa kiểm tra (${durationMs} ms)`;
  }

  // Hiệu ứng nháy sáng Visual Feedback khi cập nhật
  if (scoreCard) {
    scoreCard.classList.remove('results-stale');
    scoreCard.classList.remove('pulse-updated');
    void scoreCard.offsetWidth; // Trigger reflow để kích hoạt lại CSS animation
    scoreCard.classList.add('pulse-updated');
  }

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
  metaInfo.innerText = `${report.word_count} từ | ${report.findings.length} góp ý`;

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
    const rawSeverity = (f.severity || '').toLowerCase();
    const isMajor = rawSeverity === 'major' || rawSeverity === 'critical';
    const badgeClass = ['major', 'minor', 'advisory', 'critical'].includes(rawSeverity)
      ? rawSeverity
      : (isMajor ? 'major' : 'minor');

    const ruleId = f.rule_id || f.pattern_id || 'VIOLATION';
    const desc = f.message || f.rationale || '';

    // Badges & Rule Titles theo chế độ Dễ hiểu hoặc Mã luật
    let badgeLabel = '';
    let ruleHeaderHtml = '';

    if (displayMode === 'friendly') {
      const friendlyName = RULE_LABELS_VI[ruleId] || ruleId;
      badgeLabel = SEVERITY_LABELS_VI[rawSeverity] || (isMajor ? '🔴 LỖI NẶNG' : '🟡 GÓP Ý');
      ruleHeaderHtml = `
        <span class="rule-name-vi">${escapeHtml(friendlyName)}</span>
        <span class="rule-code-sub">(${escapeHtml(ruleId)})</span>
      `;
    } else {
      badgeLabel = (f.severity || (isMajor ? 'MAJOR' : 'MINOR')).toUpperCase();
      ruleHeaderHtml = `<span class="rule-name-vi">${escapeHtml(ruleId)}</span>`;
    }

    // 1. TRÍCH XUẤT NGỮ CẢNH (30 ký tự trước / sau lỗi)
    const startContext = Math.max(0, f.start - 30);
    const endContext = Math.min(chars.length, f.end + 30);
    const beforeText = chars.slice(startContext, f.start).join('');
    const errorText = chars.slice(f.start, f.end).join('') || f.matched_span || '';
    const afterText = chars.slice(f.end, endContext).join('');
    const displayContext = `${startContext > 0 ? '...' : ''}${escapeHtml(beforeText)}<span class="snippet-text">${escapeHtml(errorText)}</span>${escapeHtml(afterText)}${endContext < chars.length ? '...' : ''}`;

    // 2. BỘ LỌC THÔNG MINH (CHỐNG LỖI FRANKENSTEIN):
    // Phân biệt chính xác giữa "Lời khuyên hướng dẫn" và "Từ thay thế 1:1 trực tiếp"
    const isInstruction = !f.suggestion || 
                          f.suggestion.length > 40 || 
                          f.suggestion.includes("Lược bỏ") || 
                          f.suggestion.includes("Chuyển sang") || 
                          f.suggestion.includes("thay vì") ||
                          f.suggestion.includes("Viết lại") ||
                          f.suggestion.includes("Hãy") ||
                          f.suggestion === errorText;

    let suggHtml = '';
    if (f.suggestion) {
      if (isInstruction) {
        // Lời khuyên tư duy: Hiện màu xám trung tính, KHÔNG hiện nút Áp dụng
        suggHtml = `
          <div class="sugg-box sugg-instruction">
            <div class="sugg-text">💡 <b>Gợi ý:</b> ${escapeHtml(f.suggestion)}</div>
          </div>
        `;
      } else {
        // Từ thay thế 1:1: Hiện màu xanh ngọc bích KÈM nút Áp dụng ⚡
        suggHtml = `
          <div class="sugg-box sugg-actionable">
            <div class="sugg-text">💡 <b>Sửa thành:</b> "${escapeHtml(f.suggestion)}"</div>
            <button class="btn-apply" data-start="${f.start}" data-end="${f.end}" data-sugg="${escapeAttr(f.suggestion)}" title="Thay thế ngay đoạn lỗi">
              Áp dụng ⚡
            </button>
          </div>
        `;
      }
    }

    html += `
      <div class="finding-card ${badgeClass}" data-start="${f.start}" data-end="${f.end}" title="Nhấp để chuyển đến và bôi đen vị trí lỗi trong văn bản">
        <div class="finding-top">
          <div class="finding-rule">${ruleHeaderHtml}</div>
          <span class="severity-tag ${badgeClass}">${badgeLabel}</span>
        </div>
        <div class="snippet-context">${displayContext}</div>
        <div class="finding-desc">${escapeHtml(desc)}</div>
        ${suggHtml}
      </div>
    `;
  });

  findingsContainer.innerHTML = html;
}

// 7. SỰ KIỆN CLICK VÀO THẺ LỖI ĐỂ TỰ ĐỘNG NHẢY ĐẾN VÀ BÔI ĐEN (AUTO-NAVIGATE & HIGHLIGHT)
document.addEventListener('click', (e) => {
  const card = e.target.closest('.finding-card');
  if (card && !e.target.closest('.btn-apply')) {
    const start = parseInt(card.getAttribute('data-start'), 10);
    const end = parseInt(card.getAttribute('data-end'), 10);
    if (!isNaN(start) && !isNaN(end)) {
      txtInput.focus();
      txtInput.setSelectionRange(start, end);

      // Cuộn mượt mà textarea để hiển thị ngay vị trí lỗi
      const lines = txtInput.value.substring(0, start).split('\n');
      const lineHeight = 20;
      txtInput.scrollTop = Math.max(0, (lines.length - 2) * lineHeight);
    }
  }
});

// 8. TÍNH NĂNG 1-CLICK ÁP DỤNG (AUTO-FIX)
document.addEventListener('click', (e) => {
  const btn = e.target.closest('.btn-apply');
  if (!btn) return;
  e.stopPropagation(); // Ngăn sự kiện nổi bọt lên thẻ cha gây giật bôi đen

  const start = parseInt(btn.getAttribute('data-start'), 10);
  const end = parseInt(btn.getAttribute('data-end'), 10);
  const sugg = btn.getAttribute('data-sugg');
  if (isNaN(start) || isNaN(end) || sugg === null) return;

  // Dùng mảng ký tự Unicode để bảo toàn dấu tiếng Việt tuyệt đối
  const chars = Array.from(txtInput.value);
  const newChars = chars.slice(0, start).concat(Array.from(sugg)).concat(chars.slice(end));
  const newText = newChars.join('');

  txtInput.value = newText;
  txtInput.dispatchEvent(new Event('input')); // Kích hoạt Debounce tự động kiểm tra lại
});

// 9. LẤY CHỮ BÔI ĐEN TRÊN TRANG WEB
btnGetSelection.addEventListener('click', async () => {
  try {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (!tab?.id || tab.url?.startsWith('chrome://') || tab.url?.startsWith('chrome-extension://')) {
      alert("Không thể trích xuất văn bản từ trang hệ thống của trình duyệt!");
      return;
    }

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
          return (t || "").trim();
        }
      });

      const selectedText = results?.[0]?.result || "";
      if (!selectedText) {
        alert("Vui lòng bôi đen đoạn văn bản trên trang web trước!");
        return;
      }

      txtInput.value = selectedText;
      txtInput.dispatchEvent(new Event('input'));
    } catch (e) {
      console.debug("Lỗi executeScript:", e);
      alert("Không thể lấy văn bản từ tab này: " + (e.message || e));
    }
  } catch (err) {
    console.warn("Không thể lấy vùng chọn:", err);
  }
});

// 9. XÓA NỘI DUNG
btnClear.addEventListener('click', () => {
  txtInput.value = '';
  localStorage.removeItem(DRAFT_STORAGE_KEY);
  resultsBox.style.display = 'none';
  if (emptyNotice && window.innerWidth >= 850) {
    emptyNotice.style.display = 'block';
  }
  if (syncStatus) {
    syncStatus.className = 'status-tag status-fresh';
    syncStatus.innerText = '• Sẵn sàng';
  }
  if (scoreCard) {
    scoreCard.classList.remove('results-stale');
  }
  txtInput.focus();
});

// 10. ĐIỀU HƯỚNG MỞ SIDE PANEL (TRUYỀN TABID CHUẨN XÁC)
btnSidePanel.addEventListener('click', async () => {
  try {
    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (!tab?.id || tab.url?.startsWith('chrome://') || tab.url?.startsWith('chrome-extension://')) {
      alert("Không thể mở Side Panel trên trang nội bộ này. Hãy thử trên một trang web như VnExpress, Voz hoặc Google Docs!");
      return;
    }

    // Mở Side Panel bằng tabId (Chuẩn xác 100% trên Chrome MV3)
    await chrome.sidePanel.open({ tabId: tab.id });
    window.close(); // Đóng popup nhỏ
  } catch (err) {
    try {
      const win = await chrome.windows.getCurrent();
      await chrome.sidePanel.open({ windowId: win.id });
      window.close();
    } catch (e) {
      alert("Không thể mở Side Panel trên tab này: " + (err.message || err));
    }
  }
});

// 11. ĐIỀU HƯỚNG MỞ TAB STUDIO TOÀN MÀN HÌNH
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

function escapeAttr(str) {
  if (!str) return '';
  return str.replace(/"/g, '&quot;');
}
