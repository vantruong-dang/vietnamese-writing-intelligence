// FILE TỰ ĐỘNG SINH BỞI scripts/mine_ai_cliches.py
// Đã được tinh lọc: Loại bỏ các cụm từ ngữ chuẩn mực của tiếng Việt học thuật
// và các cụm sáo ngữ đã được bao phủ bởi các luật chuyên biệt (AI_OPENING, AI_SUMMARY, FORMULAIC_METAPHOR).

/// Danh sách các sáo ngữ AI được khai phá tự động bằng thuật toán thống kê tương phản
pub static DISCOVERED_AI_CLICHES: &[&str] = &[
    "tiến hành thực hiện",
    "vai trò vô cùng quan trọng",
    "sự kết hợp hài hòa giữa",
    "một minh chứng rõ nét",
    "nỗ lực không ngừng nghỉ",
    "nền móng vững chắc",
    "chiếc chìa khóa vàng mở ra",
    "chìa khóa vàng mở ra",
];
