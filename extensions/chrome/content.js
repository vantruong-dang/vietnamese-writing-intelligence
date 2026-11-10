// VWI Content Script - Hỗ trợ trích xuất văn bản bôi đen trên trang web
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === "getSelectedText") {
    let text = window.getSelection()?.toString() || "";
    if (!text) {
      const el = document.activeElement;
      if (
        el &&
        (el.tagName === "TEXTAREA" ||
          (el.tagName === "INPUT" && el.type === "text"))
      ) {
        const s = el.selectionStart;
        const e = el.selectionEnd;
        if (typeof s === "number" && typeof e === "number" && s !== e) {
          text = el.value.substring(s, e);
        } else if (el.value) {
          text = el.value;
        }
      }
    }
    sendResponse({ text: text.trim() });
  }
  return true;
});
