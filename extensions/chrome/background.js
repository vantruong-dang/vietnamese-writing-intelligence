// VWI Background Service Worker - Quản lý Context Menu và Side Panel
chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: "vwi_audit_selection",
    title: "Kiểm tra văn phong với VWI",
    contexts: ["selection"]
  });
});

chrome.contextMenus.onClicked.addListener((info, tab) => {
  if (info.menuItemId !== "vwi_audit_selection" || !tab?.id) return;

  // QUAN TRỌNG: Gọi sidePanel.open() NGAY LẬP TỨC (không có await phía trước)
  // để bảo toàn quyền User Gesture của Chrome!
  if (tab.windowId) {
    chrome.sidePanel.open({ windowId: tab.windowId }).catch((err) => {
      console.warn("Không thể mở Side Panel:", err);
    });
  }

  // Lấy văn bản giữ nguyên dấu xuống dòng (\n) qua scripting, fallback về info.selectionText
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
          vwi_timestamp: Date.now() // Ép kích hoạt onChanged kể cả khi bôi đen lại cùng 1 câu
        });
      }
    })
    .catch(() => {
      // Fallback cho trang không cho phép inject script (VD: trình đọc PDF, trang chrome://)
      if (info.selectionText) {
        chrome.storage.session.set({
          vwi_selected_text: info.selectionText,
          vwi_timestamp: Date.now()
        });
      }
    });
});
