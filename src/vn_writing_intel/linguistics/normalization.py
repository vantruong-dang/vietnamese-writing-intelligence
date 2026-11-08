"""Module chuẩn hóa văn bản tiếng Việt bảo toàn ngữ cảnh và ranh giới đoạn văn."""
from __future__ import annotations

import re
import unicodedata


def normalize_nfc(text: str) -> str:
    """Chuẩn hóa chuỗi ký tự sang dạng Unicode NFC chuẩn mực."""
    return unicodedata.normalize("NFC", text)


def normalize_whitespace(text: str, preserve_paragraphs: bool = True) -> str:
    """Chuẩn hóa khoảng trắng tiếng Việt.

    Nếu preserve_paragraphs=True:
    - Bảo toàn các ngắt dòng giữa các đoạn văn (tối đa 2 dấu xuống dòng liên tiếp).
    - Chuẩn hóa khoảng trắng ngang (spaces, tabs, non-breaking spaces) thành 1 khoảng trắng duy nhất.
    - Xóa khoảng trắng thừa ở đầu và cuối mỗi dòng.
    Nếu preserve_paragraphs=False:
    - Gộp toàn bộ khoảng trắng thành 1 dấu cách duy nhất (phục vụ regex đối chiếu số liệu/thuật ngữ).
    """
    text = normalize_nfc(text)
    if not preserve_paragraphs:
        return re.sub(r"\s+", " ", text).strip()

    # Chuẩn hóa các ký tự xuống dòng kiểu Windows (CRLF) sang chuẩn LF (\n)
    text = text.replace("\r\n", "\n").replace("\r", "\n")

    # Xử lý từng dòng: gộp khoảng trắng ngang
    lines = [re.sub(r"[^\S\n]+", " ", line).strip() for line in text.split("\n")]
    result = "\n".join(lines)

    # Giới hạn tối đa 2 dấu xuống dòng liên tiếp (1 dòng trống giữa 2 đoạn)
    result = re.sub(r"\n{3,}", "\n\n", result)
    return result.strip()


def normalize_punctuation(text: str) -> str:
    """Chuẩn hóa dấu câu tiếng Việt theo chuẩn xuất bản hiện đại."""
    text = normalize_nfc(text)
    # Chuẩn hóa dấu ngoặc kép cong sang ngoặc kép chuẩn hoặc thống nhất
    text = re.sub(r"[“”]", '"', text)
    text = re.sub(r"[‘’]", "'", text)
    # Chuẩn hóa dấu ba chấm
    text = text.replace("…", "...")
    return text
