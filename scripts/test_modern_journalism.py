import json
import subprocess

sentences = [
    (
        "Câu 1 (Bị động dịch máy)",
        "Tổng thống Mỹ đã có chuyến thăm tới Việt Nam và ông ấy đã được đón tiếp bởi các quan chức cấp cao.",
        "ARTIFICIAL_PASSIVE_BE_BY"
    ),
    (
        "Câu 2 (Chủ ngữ giả Tây phương)",
        "Nó là rất quan trọng để chúng ta hiểu rõ bối cảnh của cuộc khủng hoảng.",
        "DUMMY_IT_TRANSLATIONESE"
    ),
    (
        "Câu 3 (Dịch thô mốc thời gian)",
        "Thỏa thuận này sẽ có hiệu lực trong vòng ba tháng tới.",
        "TEMPORAL_IN_TRANSLATIONESE"
    ),
    (
        "Câu 4 (Xung đột Giới từ - Liên từ)",
        "Với phong độ hiện tại nhưng câu lạc bộ này vẫn chưa thể tìm lại chiến thắng.",
        "PREPOSITION_CONJUNCTION_CONFLICT"
    ),
    (
        "Câu 5 (Trạng ngữ lơ lửng / Đề què)",
        "Bằng việc sở hữu hàng công mạnh nhất giải đấu đã cho thấy sức mạnh của đội bóng.",
        "CRIPPLED_PREPOSITIONAL_TOPIC"
    ),
    (
        "Câu 6 (Dư từ sau động từ phát ngôn)",
        "Huấn luyện viên cho biết là đội bóng sẽ thay đổi chiến thuật.",
        "CXH_REDUNDANT_REPORTING_LA"
    ),
    (
        "Câu 7 (Trùng ngữ logic / Pleonasm)",
        "Đây là phương án kỹ thuật tối ưu nhất cho dự án này.",
        "LOGICAL_PLEONASM"
    ),
    (
        "Câu 8 (Sáo ngữ AI + Danh từ hóa)",
        "Không thể phủ nhận rằng việc chuyển đổi số đóng vai trò vô cùng quan trọng.",
        "AI_OPENING_CLICHE"
    ),
    (
        "Câu 9 (Dính phím Telex)",
        "Hệ thống của chúng tôi dduowjc cập nhật lên phiên bản mới nhất.",
        "TELEX_TYPO_ERROR"
    ),
    (
        "Câu 10 (Chính tả chữ đầu câu & dấu kết thúc)",
        "vụ việc đang được cơ quan chức năng khẩn trương làm rõ",
        "SENTENCE_CAPITALIZATION"
    )
]

print("=" * 70)
print("     ĐỐI SOÁT THỰC NGHIỆM VWI TRÊN 10 CA 'SẠN' BÁO CHÍ HIỆN ĐẠI")
print("=" * 70)

for label, s, expected_rule in sentences:
    cmd = ["target/debug/vwi.exe", "check", "-t", s, "-f", "json"]
    res = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8")
    if res.returncode == 0:
        data = json.loads(res.stdout)
        findings = [f["rule_id"] for f in data["findings"]]
        has_expected = expected_rule in findings
        status = "✅ BẮT TRÚNG" if has_expected else "⚠️ CHƯA BẮT"
        print(f"\n{label}")
        print(f"  Văn bản : \"{s}\"")
        print(f"  Kết quả : {status} (Điểm: {data['score']:.1f}/100)")
        print(f"  Mã lỗi  : {findings}")
    else:
        print(f"\n{label}: Lỗi thực thi -> {res.stderr}")
