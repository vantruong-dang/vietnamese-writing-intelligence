#!/usr/bin/env python3
"""
TẠO BỂ NGỮ LIỆU TỔNG HỢP AI ĐỐI ỨNG (MATCHED-TOPIC SYNTHETIC AI CORPUS)
Tạo ra hàng trăm đoạn văn AI viết về cùng chủ đề với Cao Xuân Hạo & Nguyễn Hiến Lê:
- Triết học cổ đại (Trang Tử, Tuân Tử, Hàn Phi, Nho gia, Đạo gia)
- Phương pháp tự học, đọc sách, học thuật
- Ngữ pháp tiếng Việt, phong cách hành văn, tiếng mẹ đẻ
- Giáo dục và phát triển tư duy
Toàn bộ văn bản đều mang đặc trưng "dấu vân tay" RLHF của LLM.
"""

import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")

ROOT_DIR = Path(__file__).resolve().parent.parent
OUTPUT_FILE = ROOT_DIR / "data_raw" / "ai_synthetic.txt"

TOPICS = [
    # Nhóm 1: Triết học & Tư tưởng cổ văn
    (
        "tư tưởng triết học của Trang Tử",
        "triết lý vô vi và tự do tinh thần",
        "phép trị quốc của Hàn Phi Tử",
        "quan niệm nhân nghĩa của Nho gia",
        "học thuyết tính ác của Tuân Tử",
        "sự tiếp biến văn hóa Đông Á qua cổ văn",
    ),
    # Nhóm 2: Phương pháp tự học & Đọc sách
    (
        "phương pháp tự học trong kỷ nguyên số",
        "thói quen đọc sách và trau dồi tri thức",
        "rèn luyện năng lực tư duy độc lập",
        "vai trò của việc tự giáo dục đối với thanh niên",
        "nghệ thuật viết văn và diễn đạt ý tưởng",
        "tự học suốt đời như một phẩm chất cần thiết",
    ),
    # Nhóm 3: Ngữ pháp & Văn phong tiếng Việt
    (
        "sự trong sáng và giàu đẹp của tiếng Việt",
        "cấu trúc ngữ pháp đề thuyết trong tiếng Việt",
        "ảnh hưởng của dịch thuật phương Tây đối với quốc văn",
        "vai trò của ngữ pháp chức năng trong giảng dạy",
        "chuẩn mực hóa chính tả và thuật ngữ khoa học",
        "phong cách biểu đạt giản dị mà sâu sắc của người Việt",
    ),
    # Nhóm 4: Giáo dục & Đào tạo thế hệ trẻ
    (
        "đổi mới căn bản và toàn diện nền giáo dục",
        "xây dựng môi trường học đường tích cực và nhân văn",
        "tích hợp công nghệ hiện đại vào phương pháp giảng dạy",
        "đào tạo nguồn nhân lực chất lượng cao trong tương lai",
        "phát triển toàn diện đức trí thể mỹ cho học sinh",
        "trách nhiệm của người làm công tác sư phạm",
    ),
]

AI_TEMPLATES = [
    "Không thể phủ nhận rằng {topic} đang đóng vai trò vô cùng quan trọng trong sự phát triển của tư duy hiện đại.",
    "Trong bối cảnh bùng nổ thông tin hiện nay, việc nghiên cứu {topic} được xem là một chiếc chìa khóa vàng mở ra cánh cửa tri thức.",
    "Bức tranh toàn cảnh về {topic} cho thấy sự kết hợp hài hòa giữa các giá trị truyền thống và xu hướng đương đại.",
    "Chúng ta cần tiến hành thực hiện các giải pháp một cách đồng bộ và quyết liệt nhằm làm sáng tỏ {topic}.",
    "Đây là một minh chứng rõ nét cho thấy những nỗ lực không ngừng nghỉ trong việc khám phá và bảo tồn {topic}.",
    "Không chỉ dừng lại ở đó, {topic} còn đóng góp một phần không nhỏ vào việc định hình nhân cách và bản lĩnh của con người.",
    "Nói tóm lại, dưới góc độ nhìn nhận từ nhiều khía cạnh khác nhau, {topic} chính là một bước ngoặt lịch sử mở ra kỷ nguyên mới.",
    "Sự kết hợp hài hòa giữa lý luận và thực tiễn trong {topic} đã tạo nên một bản giao hưởng tuyệt vời của sự đổi mới sáng tạo.",
    "Đứng trước bối cảnh hội nhập quốc tế sâu rộng, {topic} đóng vai trò then chốt trong việc tạo dựng nền móng vững chắc.",
    "Có thể nói rằng, việc tiếp cận {topic} một cách khoa học sẽ góp phần không nhỏ vào thành công chung của xã hội.",
    "Tóm lại, bài viết nhằm mục đích cung cấp một bức tranh toàn cảnh đa chiều và sâu sắc về {topic}.",
    "Không thể phủ nhận rằng {topic} đã và đang tạo tiền đề quan trọng cho những bước tiến mang tính đột phá.",
    "Để nâng cao hiệu quả ứng dụng, chúng ta cần tiến hành thực hiện việc đổi mới phương pháp tiếp cận {topic}.",
    "Đây chính là một minh chứng rõ nét khẳng định tính ưu việt và chiều sâu triết lý của {topic}.",
    "Không chỉ dừng lại ở góc độ học thuật, {topic} còn mở ra một chương mới cho sự phát triển toàn diện của thế hệ trẻ.",
    "Nhìn chung lại, {topic} là một viên ngọc quý cần được gìn giữ, phát huy và lan tỏa rộng rãi trong cộng đồng.",
    "Việc nhận thức đúng đắn về {topic} đóng vai trò vô cùng quan trọng đối với sự nghiệp xây dựng tương lai bền vững.",
    "Dưới góc độ phân tích chuyên sâu, {topic} chính là chìa khóa vạn năng giúp giải quyết những bài toán nan giải.",
    "Chúng ta cần đặt nền móng vững chắc cho việc nghiên cứu {topic} nhằm tối ưu hóa mọi tiềm năng sẵn có.",
    "Sự tương tác đa chiều xung quanh {topic} đã phác họa nên một bức tranh toàn cảnh vô cùng sống động và phong phú.",
    "Không thể phủ nhận rằng, việc rèn luyện và thấu hiểu {topic} đòi hỏi sự kiên trì bền bỉ và nỗ lực không ngừng nghỉ.",
    "Đóng vai trò quan trọng trong việc thúc đẩy tiến trình đổi mới, {topic} tiếp tục là chủ đề thu hút sự quan tâm lớn.",
    "Nói tóm lại, sự hòa quyện tuyệt vời của {topic} chính là nguồn cảm hứng bất tận cho các thế hệ tương lai.",
    "Chúng ta cần chung tay tiến hành thực hiện những định hướng chiến lược để khai phóng trọn vẹn giá trị của {topic}.",
]

def generate_corpus():
    lines = []
    for topic_group in TOPICS:
        for topic in topic_group:
            for tmpl in AI_TEMPLATES:
                sent = tmpl.format(topic=topic)
                lines.append(sent)

    OUTPUT_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(OUTPUT_FILE, "w", encoding="utf-8") as f:
        for line in lines:
            f.write(line + "\n")

    total_words = sum(len(line.split()) for line in lines)
    print(f"[+] Đã tạo {len(lines):,} câu AI synthetic ({total_words:,} từ) tại {OUTPUT_FILE}")

if __name__ == "__main__":
    generate_corpus()
