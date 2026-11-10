// content.js - Vietnamese Writing Intelligence (VWI)
// Trích xuất văn bản đang bôi đen hoặc đang soạn thảo trên trang web

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === "getSelectedText") {
    let selectedText = "";

    // 1. Kiểm tra văn bản đang chọn qua window.getSelection()
    const selection = window.getSelection();
    if (selection && selection.rangeCount > 0) {
      selectedText = selection.toString();
    }

    // 2. Nếu không có, kiểm tra ô input/textarea đang focus
    if (!selectedText) {
      const activeEl = document.activeElement;
      if (
        activeEl &&
        (activeEl.tagName === "TEXTAREA" ||
          (activeEl.tagName === "INPUT" && activeEl.type === "text"))
      ) {
        const start = activeEl.selectionStart;
        const end = activeEl.selectionEnd;
        if (typeof start === "number" && typeof end === "number" && start !== end) {
          selectedText = activeEl.value.substring(start, end);
        } else if (activeEl.value) {
          selectedText = activeEl.value;
        }
      }
    }

    sendResponse({ text: selectedText.trim() });
  }
  return true;
});
