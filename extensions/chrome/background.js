// VWI Background Service Worker - Quản lý Context Menu và Side Panel

function createMenu() {
  chrome.contextMenus.removeAll(() => {
    chrome.contextMenus.create({
      id: "vwi_audit_selection",
      title: "🔍 Kiểm tra văn phong với VWI",
      contexts: ["selection"]
    });
  });
}

// Đăng ký menu khi cài đặt hoặc khởi động service worker
chrome.runtime.onInstalled.addListener(createMenu);
chrome.runtime.onStartup.addListener(createMenu);

// Xử lý sự kiện click chuột phải
chrome.contextMenus.onClicked.addListener((info, tab) => {
  if (info.menuItemId !== "vwi_audit_selection" || !tab?.id) return;

  // 1. Mở Side Panel đồng bộ ngay lập tức để bảo toàn User Gesture
  // Ưu tiên truyền tabId theo chuẩn Chrome MV3
  chrome.sidePanel.open({ tabId: tab.id }).catch(() => {
    if (tab.windowId) {
      chrome.sidePanel.open({ windowId: tab.windowId }).catch((err) => {
        console.warn("Không thể mở Side Panel:", err);
      });
    }
  });

  // 2. Trích xuất văn bản bôi đen giữ nguyên xuống dòng (\n)
  chrome.scripting
    .executeScript({
      target: { tabId: tab.id },
      func: () => window.getSelection().toString()
    })
    .then((results) => {
      const rawText = results?.[0]?.result || info.selectionText || "";
      if (rawText.trim()) {
        chrome.storage.session.set({
          vwi_selected_text: rawText,
          vwi_timestamp: Date.now()
        });
        chrome.storage.local.set({
          selectedText: rawText,
          vwi_timestamp: Date.now()
        });
      }
    })
    .catch(() => {
      if (info.selectionText) {
        chrome.storage.session.set({
          vwi_selected_text: info.selectionText,
          vwi_timestamp: Date.now()
        });
        chrome.storage.local.set({
          selectedText: info.selectionText,
          vwi_timestamp: Date.now()
        });
      }
    });
});
