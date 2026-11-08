"""Script tự động nạp 200 ca kiểm chuẩn Linguistic Gold Standard (chuẩn Spec v0.2.0)."""
import json
import sys

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")
from pathlib import Path

DATA_FILE = Path(__file__).resolve().parents[1] / "tests" / "data" / "linguistic_gold_standard.jsonl"

CASES = []

# ==============================================================================
# NHÓM 1: DANH TỪ HÓA 3 TẦNG (50 CÂU: 20 lỗi sáo rỗng, 15 trừu tượng hợp lệ, 15 danh từ gốc)
# ==============================================================================
# 20 câu danh từ hóa sáo rỗng (cần cảnh báo / biên tập)
nom_flawed = [
    ("NOM_FLAW_001", "Cần đẩy nhanh việc thực hiện kiểm thử tự động để kịp tiến độ.", "Tối ưu hóa hành động trực tiếp"),
    ("NOM_FLAW_002", "Việc thực hiện tối ưu hóa cấu hình mang lại sự cải thiện về tốc độ.", "Danh từ hóa chồng chéo dịch từ -tion"),
    ("NOM_FLAW_003", "Doanh nghiệp chú trọng việc thực hiện chuyển đổi số trong năm nay.", "Lạm dụng 'việc thực hiện'"),
    ("NOM_FLAW_004", "Chúng tôi tiến hành quá trình tối ưu hóa bộ nhớ đệm.", "Lạm dụng 'quá trình tối ưu hóa'"),
    ("NOM_FLAW_005", "Sự gia tăng của chi phí vận hành làm giảm biên lợi nhuận.", "Lạm dụng 'sự gia tăng của'"),
    ("NOM_FLAW_006", "Ban quản lý thúc đẩy việc thực hiện thanh toán không dùng tiền mặt.", "Thừa cụm 'việc thực hiện'"),
    ("NOM_FLAW_007", "Nhóm kỹ sư giám sát quá trình tối ưu hóa cơ sở dữ liệu.", "Thừa cụm 'quá trình'"),
    ("NOM_FLAW_008", "Việc thực hiện kiểm toán nội bộ phải diễn ra hàng quý.", "Lạm dụng danh từ hóa hành động"),
    ("NOM_FLAW_009", "Cần ngăn chặn sự gia tăng của nợ xấu ngân hàng.", "Lạm dụng 'sự gia tăng của'"),
    ("NOM_FLAW_010", "Dự án yêu cầu việc thực hiện khảo sát địa chất kỹ lưỡng.", "Lạm dụng 'việc thực hiện'"),
    ("NOM_FLAW_011", "Họ đang đẩy mạnh việc thực hiện các biện pháp an toàn lao động.", "Lạm dụng 'việc thực hiện'"),
    ("NOM_FLAW_012", "Bộ phận IT phụ trách quá trình tối ưu hóa đường truyền mạng.", "Thừa cụm 'quá trình'"),
    ("NOM_FLAW_013", "Sự gia tăng của nhiệt độ toàn cầu đe dọa môi trường sống.", "Lạm dụng 'sự gia tăng của'"),
    ("NOM_FLAW_014", "Chúng ta cần theo dõi việc thực hiện cam kết quốc tế.", "Thừa cụm 'việc thực hiện'"),
    ("NOM_FLAW_015", "Kế hoạch bao gồm việc thực hiện đào tạo nhân sự mới.", "Lạm dụng 'việc thực hiện'"),
    ("NOM_FLAW_016", "Nhà máy bắt đầu quá trình tối ưu hóa dây chuyền lắp ráp.", "Thừa cụm 'quá trình'"),
    ("NOM_FLAW_017", "Chính phủ chỉ đạo việc thực hiện giải ngân vốn đầu tư công.", "Thừa cụm 'việc thực hiện'"),
    ("NOM_FLAW_018", "Cảnh báo sự gia tăng của tội phạm công nghệ cao.", "Lạm dụng 'sự gia tăng của'"),
    ("NOM_FLAW_019", "Cần bảo đảm việc thực hiện quy chế tài chính cơ quan.", "Lạm dụng 'việc thực hiện'"),
    ("NOM_FLAW_020", "Các chuyên gia điều phối quá trình tối ưu hóa chuỗi cung ứng.", "Thừa cụm 'quá trình'"),
]
for idx, (cid, text, rationale) in enumerate(nom_flawed):
    CASES.append({
        "id": cid, "category": "nominalization", "input_text": text,
        "expected_action": "review_comment", "rule_id": "NOMINALIZATION_HEAVY",
        "linguistic_rationale": f"Danh từ hóa sáo rỗng: {rationale} (Cao Xuân Hạo)."
    })

# 15 câu chủ ngữ trừu tượng hợp lệ (cấm phạt)
nom_valid_abstract = [
    ("NOM_ABS_001", "Việc triển khai dự án bị chậm do thiếu hụt nguồn nhân lực.", "Chủ ngữ trừu tượng hợp lệ"),
    ("NOM_ABS_002", "Việc học ngoại ngữ đòi hỏi sự kiên trì bền bỉ mỗi ngày.", "Chủ đề trừu tượng hợp lệ"),
    ("NOM_ABS_003", "Việc xây dựng nhà máy dự kiến hoàn tất vào cuối năm sau.", "Chủ ngữ biểu thị sự vụ"),
    ("NOM_ABS_004", "Việc tuân thủ quy trình an toàn là bắt buộc đối với mọi nhân viên.", "Chủ đề quy chuẩn"),
    ("NOM_ABS_005", "Việc bảo tồn di sản văn hóa cần sự chung tay của cộng đồng.", "Chủ đề trừu tượng"),
    ("NOM_ABS_006", "Việc mở rộng thị trường xuất khẩu mở ra nhiều cơ hội mới.", "Chủ ngữ sự vụ hợp lệ"),
    ("NOM_ABS_007", "Việc ứng dụng công nghệ giúp nâng cao năng suất cây trồng.", "Chủ đề trừu tượng"),
    ("NOM_ABS_008", "Việc cắt giảm nhân sự là quyết định khó khăn của ban giám đốc.", "Chủ đề danh hóa hợp lệ"),
    ("NOM_ABS_009", "Việc đánh giá cán bộ phải dựa trên năng lực và hiệu quả thực tế.", "Chủ ngữ trừu tượng"),
    ("NOM_ABS_010", "Việc thay đổi chính sách tiền tệ tác động trực tiếp tới tỷ giá.", "Chủ đề kinh tế hợp lệ"),
    ("NOM_ABS_011", "Việc bảo mật thông tin khách hàng là ưu tiên hàng đầu.", "Chủ đề quy định"),
    ("NOM_ABS_012", "Việc xử lý rác thải đô thị đang là bài toán cấp bách.", "Chủ ngữ sự vụ"),
    ("NOM_ABS_013", "Việc chuyển đổi cơ cấu cây trồng đem lại hiệu quả kinh tế rõ rệt.", "Chủ đề hợp lệ"),
    ("NOM_ABS_014", "Việc kiểm soát lạm phát đòi hỏi sự phối hợp đồng bộ giữa các ngành.", "Chủ ngữ trừu tượng"),
    ("NOM_ABS_015", "Việc nâng cao dân trí là nền tảng cho sự phát triển bền vững.", "Chủ đề chân phương"),
]
for cid, text, rationale in nom_valid_abstract:
    CASES.append({
        "id": cid, "category": "nominalization", "input_text": text,
        "expected_action": "no_change", "rule_id": "LEGITIMATE_ABSTRACT_NOMINALIZATION",
        "linguistic_rationale": f"{rationale} đứng làm Đề, không phải lỗi dịch máy (Cao Xuân Hạo)."
    })

# 15 câu danh từ gốc / từ ghép cố định (cấm phạt)
nom_compound = [
    ("NOM_COM_001", "Công việc ổn định là mong ước của nhiều người lao động.", "Danh từ gốc 'công việc'"),
    ("NOM_COM_002", "Sự tích quả dưa hấu được lưu truyền qua nhiều thế hệ.", "Từ ghép cố định 'sự tích'"),
    ("NOM_COM_003", "Việc này rất gấp, anh cần xử lý ngay trong buổi sáng.", "Danh từ chỉ sự vụ cụ thể 'việc này'"),
    ("NOM_COM_004", "Sự nghiệp giáo dục là trách nhiệm chung của toàn xã hội.", "Từ ghép 'sự nghiệp'"),
    ("NOM_COM_005", "Mọi sự cố kỹ thuật đều đã được khắc phục kịp thời.", "Từ ghép 'sự cố'"),
    ("NOM_COM_006", "Bác Ba là người rất có trách nhiệm với công việc chung.", "Danh từ 'công việc'"),
    ("NOM_COM_007", "Sự thật lịch sử luôn cần được tôn trọng khách quan.", "Từ ghép 'sự thật'"),
    ("NOM_COM_008", "Đây là sự việc nghiêm trọng cần cơ quan chức năng làm rõ.", "Từ ghép 'sự việc'"),
    ("NOM_COM_009", "Những việc nhỏ hàng ngày tạo nên nhân cách con người.", "Danh từ 'việc'"),
    ("NOM_COM_010", "Sự sống trên trái đất phụ thuộc vào nguồn năng lượng mặt trời.", "Từ ghép 'sự sống'"),
    ("NOM_COM_011", "Lòng tự trọng là phẩm giá cao quý của mỗi công dân.", "Từ ghép 'tự trọng'"),
    ("NOM_COM_012", "Sự kiên trì giúp anh ấy vượt qua mọi thử thách khắc nghiệt.", "Từ ghép 'sự kiên trì'"),
    ("NOM_COM_013", "Hai bên đã bàn bạc kỹ lưỡng về từng công việc cụ thể.", "Danh từ 'công việc'"),
    ("NOM_COM_014", "Sự đồng thuận của nhân dân là yếu tố quyết định thành công.", "Từ ghép 'sự đồng thuận'"),
    ("NOM_COM_015", "Việc công việc tư cần được phân định rõ ràng minh bạch.", "Danh từ chỉ việc vụ"),
]
for cid, text, rationale in nom_compound:
    CASES.append({
        "id": cid, "category": "nominalization", "input_text": text,
        "expected_action": "no_change", "rule_id": "COMPOUND_NOUN_PRESERVE",
        "linguistic_rationale": f"{rationale}, không phải danh từ hóa sáo rỗng (Cao Xuân Hạo)."
    })

# ==============================================================================
# NHÓM 2: CẤU TRÚC ĐỀ - THUYẾT & CÂU KHUYẾT TẬT (50 CÂU)
# ==============================================================================
# 20 câu lỗi Khung đề què biến thành Chủ ngữ (cần cảnh báo)
topic_flawed = [
    ("TOP_FLAW_001", "Trong tình hình kinh tế hiện nay đòi hỏi doanh nghiệp phải tái cơ cấu toàn diện.", "Trong tình hình... đòi hỏi"),
    ("TOP_FLAW_002", "Bằng những số liệu thực nghiệm đã chứng minh giả thuyết của nhóm nghiên cứu là chính xác.", "Bằng những... đã chứng minh"),
    ("TOP_FLAW_003", "Qua việc khảo sát thực tế cho thấy cơ sở hạ tầng nông thôn đã đổi mới.", "Qua việc... cho thấy"),
    ("TOP_FLAW_004", "Trong báo cáo tài chính mới nhất đòi hỏi ban điều hành phải giải trình rõ.", "Trong báo cáo... đòi hỏi"),
    ("TOP_FLAW_005", "Bằng những chính sách ưu đãi giúp thu hút mạnh mẽ dòng vốn FDI.", "Bằng những... giúp"),
    ("TOP_FLAW_006", "Qua đợt thanh tra vừa qua đã phát hiện nhiều sai phạm nghiêm trọng.", "Qua đợt... đã phát hiện"),
    ("TOP_FLAW_007", "Trong nghị quyết đại hội lần này đòi hỏi sự đồng lòng của toàn thể đảng viên.", "Trong nghị quyết... đòi hỏi"),
    ("TOP_FLAW_008", "Bằng những công nghệ hiện đại đã giúp nâng cao năng suất lao động rõ rệt.", "Bằng những... đã giúp"),
    ("TOP_FLAW_009", "Qua các buổi tiếp xúc cử tri cho thấy nguyện vọng của nhân dân rất chính đáng.", "Qua các... cho thấy"),
    ("TOP_FLAW_010", "Trong hoàn cảnh khó khăn hiện tại đòi hỏi mọi người phải thắt lưng buộc bụng.", "Trong hoàn cảnh... đòi hỏi"),
    ("TOP_FLAW_011", "Bằng những nỗ lực phi thường đã giúp đội bóng lọt vào trận chung kết.", "Bằng những... đã giúp"),
    ("TOP_FLAW_012", "Qua nghiên cứu tài liệu cổ cho thấy nguồn gốc lâu đời của ngôi làng.", "Qua nghiên cứu... cho thấy"),
    ("TOP_FLAW_013", "Trong giai đoạn chuyển giao công nghệ đòi hỏi kỹ sư phải học hỏi nhanh.", "Trong giai đoạn... đòi hỏi"),
    ("TOP_FLAW_014", "Bằng các giải pháp tài chính linh hoạt giúp doanh nghiệp duy trì thanh khoản.", "Bằng các... giúp"),
    ("TOP_FLAW_015", "Qua số liệu thống kê hàng năm cho thấy tốc độ tăng trưởng ổn định.", "Qua số liệu... cho thấy"),
    ("TOP_FLAW_016", "Trong bối cảnh cạnh tranh khốc liệt đòi hỏi chúng ta phải đổi mới sáng tạo.", "Trong bối cảnh... đòi hỏi"),
    ("TOP_FLAW_017", "Bằng những chứng cứ không thể chối cãi đã buộc hung thủ phải nhận tội.", "Bằng những... đã buộc"),
    ("TOP_FLAW_018", "Qua việc phân tích mẫu nước cho thấy mức độ ô nhiễm vượt ngưỡng an toàn.", "Qua việc... cho thấy"),
    ("TOP_FLAW_019", "Trong quy chế hoạt động mới đòi hỏi thành viên phải tuân thủ kỷ luật.", "Trong quy chế... đòi hỏi"),
    ("TOP_FLAW_020", "Bằng sự phối hợp chặt chẽ giữa các đơn vị đã giúp dập tắt đám cháy nhanh chóng.", "Bằng sự... đã giúp"),
]
for cid, text, rationale in topic_flawed:
    CASES.append({
        "id": cid, "category": "topic_comment", "input_text": text,
        "expected_action": "review_comment", "rule_id": "CRIPPLED_PREPOSITIONAL_TOPIC",
        "linguistic_rationale": f"Lẫn lộn Khung đề và Chủ đề: {rationale} (Cao Xuân Hạo, tr. 130)."
    })

# 10 câu xung đột Giới từ mở đầu trạng ngữ và liên từ 'nhưng'
prep_conj = [
    ("TOP_CONJ_001", "Với những nỗ lực phi thường nhưng họ vẫn không thể hoàn thành dự án đúng hạn.", "Với... nhưng"),
    ("TOP_CONJ_002", "Qua nhiều lần đàm phán gay gắt nhưng hai bên chưa đạt được thỏa thuận chung.", "Qua... nhưng"),
    ("TOP_CONJ_003", "Bằng các biện pháp cứu trợ kịp thời nhưng đời sống người dân vẫn gặp khó khăn.", "Bằng... nhưng"),
    ("TOP_CONJ_004", "Với trang thiết bị tối tân nhưng đội ngũ vận hành chưa làm chủ được công nghệ.", "Với... nhưng"),
    ("TOP_CONJ_005", "Qua các đợt tập huấn nâng cao nghiệp vụ nhưng hiệu quả công việc chưa cải thiện.", "Qua... nhưng"),
    ("TOP_CONJ_006", "Với số vốn đầu tư ban đầu lớn nhưng công ty vẫn rơi vào cảnh thua lỗ.", "Với... nhưng"),
    ("TOP_CONJ_007", "Bằng sự can thiệp của chính quyền nhưng xung đột đất đai vẫn chưa hạ nhiệt.", "Bằng... nhưng"),
    ("TOP_CONJ_008", "Với quyết tâm cao của tập thể nhưng mục tiêu tăng trưởng vẫn chưa về đích.", "Với... nhưng"),
    ("TOP_CONJ_009", "Qua nhiều năm du học ở nước ngoài nhưng vốn sống thực tế của anh ấy còn mỏng.", "Qua... nhưng"),
    ("TOP_CONJ_010", "Với sự hướng dẫn tận tình của thầy cô nhưng học sinh vẫn làm bài lúng túng.", "Với... nhưng"),
]
for cid, text, rationale in prep_conj:
    CASES.append({
        "id": cid, "category": "topic_comment", "input_text": text,
        "expected_action": "review_comment", "rule_id": "PREPOSITION_CONJUNCTION_CONFLICT",
        "linguistic_rationale": f"Xung đột cú pháp {rationale}, tạo ảo giác khung đề là một câu độc lập (Cao Xuân Hạo)."
    })

# 10 câu Đề - Thuyết mẫu mực có tác tử 'thì' (cấm xóa)
topic_valid_thi = [
    ("TOP_VAL_001", "Áo cũ quần cũ bà ta đều mua tất.", "Chủ đề số phức chi phối phó từ 'đều'"),
    ("TOP_VAL_002", "Tôi tên là Nam, quê ở Huế, có ba con.", "Chủ đề bao quát chuỗi vị ngữ"),
    ("TOP_VAL_003", "Cha thì đi cày, mẹ thì đi cấy, hai anh em thì ở nhà học bài.", "Tác tử 'thì' phân giới tương phản sóng đôi"),
    ("TOP_VAL_004", "Trâu thì đen, bò thì vàng, ngựa thì trắng.", "Cấu trúc tiểu đối bắt buộc dùng 'thì'"),
    ("TOP_VAL_005", "Tham thì thâm, giận thì mất khôn.", "Tục ngữ Đề - Thuyết kinh điển"),
    ("TOP_VAL_006", "Bồi ở, lở đi, đất nào lành thì chim đậu.", "Đề - Thuyết dân gian mẫu mực"),
    ("TOP_VAL_007", "Thứ nhạc này tôi không thích nghe.", "Đề đứng đầu câu làm bổ ngữ của Thuyết"),
    ("TOP_VAL_008", "Trong vườn trồng toàn cam và bưởi da xanh.", "Khung đề nơi chốn thuần Việt"),
    ("TOP_VAL_009", "Con thì đứng, con thì nằm, trông rất ngộ nghĩnh.", "Tương phản sóng đôi có 'thì'"),
    ("TOP_VAL_010", "Đến tối thì về, không được la cà hàng quán.", "Khung đề thời gian đi kèm 'thì'"),
]
for cid, text, rationale in topic_valid_thi:
    CASES.append({
        "id": cid, "category": "topic_comment", "input_text": text,
        "expected_action": "no_change", "rule_id": "VALID_TOPIC_COMMENT_AGREEMENT",
        "linguistic_rationale": f"{rationale} (Cao Xuân Hạo, tr. 18-20, 96)."
    })

# 10 câu tỉnh lược chủ ngữ tự nhiên (Zero Anaphora)
zero_anaphora = [
    ("TOP_ZERO_001", "Hôm qua mẹ về. Sáng nay mẹ lại đi chợ sớm mua thức ăn.", "Chủ đề đã xác lập từ câu trước"),
    ("TOP_ZERO_002", "Bác Ba sang chơi. Ngồi nói chuyện một lát rồi lại vội vã ra đồng.", "Tỉnh lược chủ ngữ tự nhiên"),
    ("TOP_ZERO_003", "Bông hoa hồng vừa nở. Tỏa hương thơm ngát khắp gian phòng nhỏ.", "Zero anaphora chuẩn mực"),
    ("TOP_ZERO_004", "Anh Nam vừa nhận tháng lương đầu tiên. Đem về biếu mẹ dưỡng già.", "Chủ thể rõ ràng không cần đại từ thừa"),
    ("TOP_ZERO_005", "Gió mùa đông bắc tràn về. Mang theo cái rét căm căm trên từng góc phố.", "Dòng chảy diễn ngôn liền mạch"),
    ("TOP_ZERO_006", "Đoàn tàu chuyển bánh từ từ. Rồi từ từ tăng tốc lăn bánh về phương Nam.", "Tỉnh lược tự nhiên"),
    ("TOP_ZERO_007", "Chú cún con thấy chủ về vẫy đuôi mừng rỡ. Chạy vòng quanh nhảy cẫng lên.", "Ngữ pháp Đề - Thuyết bản địa"),
    ("TOP_ZERO_008", "Cây bàng trước sân trút lá mùa đông. Trơ trụi những cành khẳng khiu đón gió.", "Zero anaphora"),
    ("TOP_ZERO_009", "Chị Lan tốt nghiệp thủ khoa đại học sư phạm. Về quê mở lớp dạy học miễn phí cho trẻ em nghèo.", "Mạch văn tự nhiên"),
    ("TOP_ZERO_010", "Chiếc thuyền nan cập bến sông. Buông neo nghỉ ngơi sau chuyến đánh cá đêm dài.", "Tỉnh lược chủ ngữ hợp ngữ cảnh"),
]
for cid, text, rationale in zero_anaphora:
    CASES.append({
        "id": cid, "category": "topic_comment", "input_text": text,
        "expected_action": "no_change", "rule_id": "VALID_ZERO_ANAPHORA",
        "linguistic_rationale": f"{rationale}, không cần chèn đại từ 'nó/họ' máy móc (Cao Xuân Hạo)."
    })

# ==============================================================================
# NHÓM 3: HIỆN THỰC VS GIẢ ĐỊNH, THỜI GIAN, TRÙNG NGỮ, SÁO NGỮ AI (50 CÂU)
# ==============================================================================
# 10 câu dùng sai Tuy trong tương lai giả định (cần sửa sang Dù)
tuy_misuse = [
    ("CONC_TUY_001", "Ngày mai, tuy trời mưa to gió lớn, tôi cũng sẽ đến đúng giờ.", "Ngày mai... tuy"),
    ("CONC_TUY_002", "Tuần tới, tuy công việc bận rộn đến mấy, chúng tôi vẫn sắp xếp tham gia.", "Tuần tới... tuy"),
    ("CONC_TUY_003", "Năm sau, tuy giá nguyên vật liệu có tăng cao, công ty vẫn giữ nguyên giá bán.", "Năm sau... tuy"),
    ("CONC_TUY_004", "Sắp tới, tuy thị trường có nhiều biến động, chúng ta vẫn kiên trì mục tiêu.", "Sắp tới... tuy"),
    ("CONC_TUY_005", "Ngày mai, mặc dù khó khăn chồng chất, tôi tin các bạn sẽ vượt qua.", "Ngày mai... mặc dù"),
    ("CONC_TUY_006", "Sau này, tuy không còn làm việc cùng nhau, tôi vẫn luôn nhớ đến anh.", "Sau này... tuy"),
    ("CONC_TUY_007", "Tuần tới, mặc dầu thời tiết không thuận lợi, chuyến bay vẫn cất cánh.", "Tuần tới... mặc dầu"),
    ("CONC_TUY_008", "Năm sau, tuy có phải vay thêm vốn, tôi vẫn quyết tâm khởi nghiệp.", "Năm sau... tuy"),
    ("CONC_TUY_009", "Sắp tới, tuy đối thủ cạnh tranh gay gắt, sản phẩm của ta vẫn có chỗ đứng.", "Sắp tới... tuy"),
    ("CONC_TUY_010", "Ngày mai, tuy có ai can ngăn đi nữa, tôi vẫn sẽ lên đường.", "Ngày mai... tuy"),
]
for cid, text, rationale in tuy_misuse:
    CASES.append({
        "id": cid, "category": "concessive_aspect", "input_text": text,
        "expected_action": "review_comment", "rule_id": "FACTUAL_CONCESSIVE_MISUSE",
        "linguistic_rationale": f"Lỗi dùng Tuy cho tương lai giả định: {rationale} (Cao Xuân Hạo, tr. 128-129, 163)."
    })

# 10 câu dùng sai Dù trong quá khứ hiện thực (cần sửa sang Tuy/Mặc dù)
du_misuse = [
    ("CONC_DU_001", "Hôm qua, dù trời mưa tầm tã, đội cứu hộ vẫn tiếp cận được hiện trường.", "Hôm qua... dù"),
    ("CONC_DU_002", "Năm ngoái, dù gặp nhiều đợt bão lũ, nhân dân miền Trung vẫn đạt mùa màng khá.", "Năm ngoái... dù"),
    ("CONC_DU_003", "Tuần trước, dù đối mặt với áp lực xả hàng lớn, chỉ số VN-Index vẫn giữ được sắc xanh.", "Tuần trước... dù"),
    ("CONC_DU_004", "Vừa qua, dù nguồn cung gián đoạn, chuỗi siêu thị vẫn bình ổn giá cả thị trường.", "Vừa qua... dù"),
    ("CONC_DU_005", "Hôm qua, dù bị sốt cao, em học sinh vẫn hoàn thành tốt bài thi môn toán.", "Hôm qua... dù"),
    ("CONC_DU_006", "Năm ngoái, dù dịch bệnh kéo dài, doanh nghiệp vẫn bảo toàn được lực lượng lao động.", "Năm ngoái... dù"),
    ("CONC_DU_007", "Tuần trước, dù bị chấn thương nhẹ, cầu thủ vẫn thi đấu trọn vẹn 90 phút.", "Tuần trước... dù"),
    ("CONC_DU_008", "Vừa qua, dù doanh thu sụt giảm, ngân hàng vẫn chi trả cổ tức đầy đủ cho cổ đông.", "Vừa qua... dù"),
    ("CONC_DU_009", "Hôm qua, dù đường sá ngập sâu, các cô giáo vùng cao vẫn bám lớp bám trường.", "Hôm qua... dù"),
    ("CONC_DU_010", "Năm ngoái, dù chi phí đầu vào tăng vọt, hợp tác xã vẫn mở rộng được quy mô sản xuất.", "Năm ngoái... dù"),
]
for cid, text, rationale in du_misuse:
    CASES.append({
        "id": cid, "category": "concessive_aspect", "input_text": text,
        "expected_action": "review_comment", "rule_id": "HYPOTHETICAL_CONCESSIVE_MISUSE",
        "linguistic_rationale": f"Lỗi dùng Dù cho sự thể quá khứ hiện thực: {rationale} (Cao Xuân Hạo, tr. 128, 163)."
    })

# 10 câu lỗi dịch thô khung thời gian "sẽ... trong X ngày"
temporal_calque = [
    ("TEMP_CALQ_001", "Chuyên gia nước ngoài sẽ có mặt tại Hà Nội trong ba ngày.", "trong ba ngày"),
    ("TEMP_CALQ_002", "Hệ thống sẽ tự động khởi động lại sau hai giờ nữa.", "sau hai giờ nữa"),
    ("TEMP_CALQ_003", "Chuyến tàu sẽ cập cảng Hải Phòng trong năm ngày.", "trong năm ngày"),
    ("TEMP_CALQ_004", "Hội nghị quốc tế sẽ khai mạc trong hai tuần.", "trong hai tuần"),
    ("TEMP_CALQ_005", "Bản cập nhật phần mềm sẽ được tung ra trong ba ngày.", "trong ba ngày"),
    ("TEMP_CALQ_006", "Bộ trưởng sẽ trả lời chất vấn trước quốc hội trong hai ngày.", "trong hai ngày"),
    ("TEMP_CALQ_007", "Công trình cầu vượt sẽ hoàn thành sau sáu tháng nữa.", "sau sáu tháng nữa"),
    ("TEMP_CALQ_008", "Đoàn kiểm tra sẽ đưa ra kết luận chính thức trong một tuần.", "trong một tuần"),
    ("TEMP_CALQ_009", "Hợp đồng kinh tế mới sẽ có hiệu lực trong mười ngày.", "trong mười ngày"),
    ("TEMP_CALQ_010", "Dây chuyền sản xuất mới sẽ đi vào hoạt động trong hai tháng.", "trong hai tháng"),
]
for cid, text, rationale in temporal_calque:
    CASES.append({
        "id": cid, "category": "temporal_framing", "input_text": text,
        "expected_action": "review_comment", "rule_id": "TEMPORAL_IN_TRANSLATIONESE",
        "linguistic_rationale": f"Sao phỏng tiếng Anh: {rationale}. Tiếng Việt ưu tiên Khung đề đầu câu (Cao Xuân Hạo, tr. 101-104)."
    })

# 10 câu trùng ngữ logic (cần sửa)
pleonasm_flawed = [
    ("PLEO_FLAW_001", "Đây là giải pháp kỹ thuật tối ưu nhất cho bài toán mở rộng hạ tầng.", "tối ưu nhất"),
    ("PLEO_FLAW_002", "Ánh nắng mặt trời buổi sớm chiếu qua khe cửa sổ rực rỡ.", "ánh nắng mặt trời"),
    ("PLEO_FLAW_003", "Nguyên nhân sự cố mạng là bởi vì máy chủ quá tải.", "nguyên nhân... là bởi vì"),
    ("PLEO_FLAW_004", "Thuật toán này đảm bảo độ chính xác hoàn toàn tuyệt đối.", "hoàn toàn tuyệt đối"),
    ("PLEO_FLAW_005", "Quy trình này đem lại hiệu quả tối ưu nhất trong điều kiện khắc nghiệt.", "tối ưu nhất"),
    ("PLEO_FLAW_006", "Lớp sơn phủ giúp bảo vệ vỏ xe khỏi ánh nắng mặt trời gay gắt.", "ánh nắng mặt trời"),
    ("PLEO_FLAW_007", "Nguyên nhân thất bại của dự án là bởi vì thiếu vốn đầu tư.", "nguyên nhân... là bởi vì"),
    ("PLEO_FLAW_008", "Văn bản này có giá trị pháp lý hoàn toàn tuyệt đối.", "hoàn toàn tuyệt đối"),
    ("PLEO_FLAW_009", "Cần lựa chọn phương án thi công tối ưu nhất để tiết kiệm chi phí.", "tối ưu nhất"),
    ("PLEO_FLAW_010", "Nguyên nhân giá vàng tăng là bởi vì lạm phát toàn cầu leo thang.", "nguyên nhân... là bởi vì"),
]
for cid, text, rationale in pleonasm_flawed:
    CASES.append({
        "id": cid, "category": "pleonasm", "input_text": text,
        "expected_action": "review_comment", "rule_id": "LOGICAL_PLEONASM",
        "linguistic_rationale": f"Trùng ngữ logic: {rationale} (Cao Xuân Hạo, tr. 105)."
    })

# 10 câu ngoại lệ thân tộc bản địa (anh trai, chị gái - CẤM PHẠT)
kinship_valid = [
    ("KIN_VAL_001", "Anh trai và chị gái tôi đều đã tốt nghiệp đại học.", "anh trai, chị gái"),
    ("KIN_VAL_002", "Bác trai và bác gái mới từ quê ra thăm gia đình tôi.", "bác trai, bác gái"),
    ("KIN_VAL_003", "Em trai tôi đang theo học ngành công nghệ thông tin tại Hà Nội.", "em trai"),
    ("KIN_VAL_004", "Cháu gái học rất giỏi và luôn đạt thành tích cao trong trường.", "cháu gái"),
    ("KIN_VAL_005", "Anh trai tôi là bác sĩ chuyên khoa ngoại tại bệnh viện Bạch Mai.", "anh trai"),
    ("KIN_VAL_006", "Chị gái tôi mở một cửa hàng bán hoa tươi trên phố cổ.", "chị gái"),
    ("KIN_VAL_007", "Hai anh em trai cùng chung tay xây dựng trang trại nông sản sạch.", "anh em trai"),
    ("KIN_VAL_008", "Cháu trai của cụ Nam vừa được nhận học bổng du học toàn phần.", "cháu trai"),
    ("KIN_VAL_009", "Em gái tôi rất khéo tay và yêu thích nghề làm bánh ngọt.", "em gái"),
    ("KIN_VAL_010", "Bác gái tôi là nhà giáo mẫu mực được bao thế hệ học trò kính trọng.", "bác gái"),
]
for cid, text, rationale in kinship_valid:
    CASES.append({
        "id": cid, "category": "pleonasm", "input_text": text,
        "expected_action": "no_change", "rule_id": "LEGITIMATE_KINSHIP_MODIFIER",
        "linguistic_rationale": f"Ngoại lệ thân tộc: {rationale} chỉ ruột thịt, cấm coi là trùng ngữ (Cao Xuân Hạo, tr. 106-108)."
    })

# ==============================================================================
# NHÓM 4: BẢO TOÀN DỮ KIỆN, KHỐI BẢO VỆ, CỔ VĂN (50 CÂU)
# ==============================================================================
# 15 câu bảo toàn mỏ neo bất biến thành công (no_change)
anchor_intact = [
    ("ANC_INT_001", "Bộ nguồn thay thế 20V 7.5A cung cấp công suất 150W ổn định cho hệ thống.", ["20V 7.5A", "150W"], []),
    ("ANC_INT_002", "Ranh giới FFI giữa Rust và C làm xuất hiện hiện tượng type erasure gây vỡ hợp đồng.", [], ["FFI", "type erasure"]),
    ("ANC_INT_003", "Chỉ số VN-Index đóng cửa tại 1250 điểm với thanh khoản đạt 25000 tỷ đồng.", ["1250 điểm", "25000 tỷ đồng"], ["VN-Index"]),
    ("ANC_INT_004", "Hệ thống hỗ trợ dòng điện tối đa 16A tại điện áp 220V xoay chiều tiêu chuẩn.", ["16A", "220V"], []),
    ("ANC_INT_005", "Áp lực force-sell và call margin quét qua cổ phiếu ngành ngân hàng trong phiên ATC.", [], ["force-sell", "call margin", "ATC"]),
    ("ANC_INT_006", "Tốc độ xử lý của CPU đạt 3.8GHz với mức tiêu thụ năng lượng 65W mát mẻ.", ["3.8GHz", "65W"], []),
    ("ANC_INT_007", "Dung lượng bộ nhớ RAM được nâng cấp lên 64GB DDR5 băng thông cực cao.", ["64GB"], ["RAM", "DDR5"]),
    ("ANC_INT_008", "Quỹ đầu tư thoái vốn 1200000 cổ phiếu FPT với giá trị giao dịch lớn.", ["1200000 cổ phiếu"], ["FPT"]),
    ("ANC_INT_009", "Cổng kết nối USB-C hỗ trợ chuẩn Thunderbolt 4 tốc độ 40Gbps siêu nhanh.", ["40Gbps"], ["USB-C", "Thunderbolt 4"]),
    ("ANC_INT_010", "Dự án cao tốc dài 120km dự kiến khánh thành vào ngày 30/4 sắp tới.", ["120km", "30/4"], []),
    ("ANC_INT_011", "Độ trễ truy vấn giảm từ 150ms xuống còn 15ms sau khi bổ sung chỉ mục.", ["150ms", "15ms"], []),
    ("ANC_INT_012", "Cơ chế Garbage Collection trong Java gây ra hiện tượng stop-the-world dài 500ms.", ["500ms"], ["Garbage Collection", "stop-the-world", "Java"]),
    ("ANC_INT_013", "Khối lượng xuất khẩu gạo năm nay đạt 8.2 triệu tấn, kim ngạch 4.5 tỷ USD.", ["8.2 triệu tấn", "4.5 tỷ USD"], []),
    ("ANC_INT_014", "Áp dụng cơ chế copy-on-write giúp tiết kiệm 45% dung lượng lưu trữ đĩa cứng.", ["45%"], ["copy-on-write"]),
    ("ANC_INT_015", "Mức lãi suất cơ bản được duy trì ở ngưỡng 4.5% trên năm theo quyết định mới.", ["4.5%"], []),
]
for cid, text, metrics, terms in anchor_intact:
    CASES.append({
        "id": cid, "category": "semantic_anchor", "input_text": text,
        "expected_action": "no_change", "rule_id": "IMMUTABLE_ANCHOR_INTACT",
        "manifest": {"discourse_spec": {"genre": "technical_architecture", "register": "professional_sharp", "target_audience": "domain_engineer"}, "immutable_anchors": {"metrics": metrics, "domain_terms": terms}},
        "linguistic_rationale": "Bảo toàn nguyên vẹn 100% mỏ neo số liệu và thuật ngữ kỹ thuật."
    })

# 15 câu báo lỗi làm mất hoặc biến dạng mỏ neo (needs_author_decision)
anchor_loss = [
    ("ANC_LOS_001", "Bộ nguồn thay thế cung cấp công suất ổn định cho toàn bộ hệ thống.", ["20V 7.5A", "150W"], []),
    ("ANC_LOS_002", "Ranh giới giao diện ngoại lai làm xuất hiện sự xóa kiểu dữ liệu phức tạp.", [], ["FFI", "type erasure"]),
    ("ANC_LOS_003", "Chỉ số chứng khoán đóng cửa phiên giao dịch với thanh khoản ở mức rất cao.", ["1250 điểm", "25000 tỷ đồng"], ["VN-Index"]),
    ("ANC_LOS_004", "Hệ thống hỗ trợ dòng điện tối đa tại điện áp xoay chiều dân dụng thông thường.", ["16A", "220V"], []),
    ("ANC_LOS_005", "Áp lực bán giải chấp và gọi ký quỹ quét qua cổ phiếu các ngân hàng lớn.", [], ["force-sell", "call margin"]),
    ("ANC_LOS_006", "Tốc độ xử lý của vi xử lý đạt mức cao với năng lượng tiêu thụ thấp.", ["3.8GHz", "65W"], []),
    ("ANC_LOS_007", "Dung lượng bộ nhớ trong được mở rộng lên mức tối đa cho máy trạm.", ["64GB"], ["RAM", "DDR5"]),
    ("ANC_LOS_008", "Quỹ đầu tư bán một lượng lớn cổ phiếu công nghệ trong phiên vừa qua.", ["1200000 cổ phiếu"], ["FPT"]),
    ("ANC_LOS_009", "Cổng kết nối mới hỗ trợ chuẩn truyền dữ liệu tốc độ cực nhanh.", ["40Gbps"], ["USB-C", "Thunderbolt 4"]),
    ("ANC_LOS_010", "Dự án đường cao tốc mới dự kiến sẽ khánh thành trong thời gian tới.", ["120km", "30/4"], []),
    ("ANC_LOS_011", "Độ trễ truy vấn của cơ sở dữ liệu đã giảm đi đáng kể sau đợt bảo trì.", ["150ms", "15ms"], []),
    ("ANC_LOS_012", "Cơ chế thu gom rác tự động làm dừng toàn bộ chương trình trong giây lát.", ["500ms"], ["Garbage Collection", "stop-the-world"]),
    ("ANC_LOS_013", "Khối lượng xuất khẩu gạo và kim ngạch thu về đều đạt con số kỷ lục trong năm.", ["8.2 triệu tấn", "4.5 tỷ USD"], []),
    ("ANC_LOS_014", "Áp dụng kỹ thuật sao chép khi ghi giúp giảm thiểu không gian đĩa bị chiếm dụng.", ["45%"], ["copy-on-write"]),
    ("ANC_LOS_015", "Mức lãi suất điều hành được điều chỉnh theo quyết định mới nhất của thống đốc.", ["4.5%"], []),
]
for cid, text, metrics, terms in anchor_loss:
    CASES.append({
        "id": cid, "category": "semantic_anchor", "input_text": text,
        "expected_action": "needs_author_decision", "rule_id": "ANCHOR_DRIFT_METRIC_LOSS",
        "manifest": {"discourse_spec": {"genre": "technical_architecture", "register": "professional_sharp", "target_audience": "domain_engineer"}, "immutable_anchors": {"metrics": metrics, "domain_terms": terms}},
        "linguistic_rationale": "Bản thảo làm rơi mất số liệu hoặc tự ý dịch thuật ngữ chuyên ngành đã khóa."
    })

# 10 câu bảo vệ Code Block, URL, Email (cấm phạt lỗi)
protected_cases = [
    ("PROT_001", "Kiểm tra hàm `execute_batch()` trong file `src/engine.py` để tối ưu.", [], []),
    ("PROT_002", "Mọi thắc mắc xin vui lòng gửi về email support@vietnamese-ai.vn để được giải đáp.", [], []),
    ("PROT_003", "Tham khảo tài liệu hướng dẫn chi tiết tại https://github.com/vantruong-dang để cài đặt.", [], []),
    ("PROT_004", "Chạy lệnh `npm install -g vn-writing-intel` để cài đặt công cụ toàn cục.", [], []),
    ("PROT_005", "Đường dẫn file cấu hình nằm tại `/etc/vn_writing/config.yaml` trên hệ điều hành Linux.", [], []),
    ("PROT_006", "Đoạn mã sau đây thực thi việc kiểm tra: ```def run(): return True```.", [], []),
    ("PROT_007", "Truy cập cổng dịch vụ tại địa chỉ http://localhost:8080 để kiểm tra kết quả.", [], []),
    ("PROT_008", "Khai báo biến môi trường `DATABASE_URL=postgres://user:pass@host/db` trước khi chạy.", [], []),
    ("PROT_009", "Gọi phương thức `client.send_message()` qua API endpoint an toàn.", [], []),
    ("PROT_010", "Tải tài nguyên mẫu từ https://huggingface.co/datasets/vietnamese-prose về máy.", [], []),
]
for cid, text, metrics, terms in protected_cases:
    CASES.append({
        "id": cid, "category": "protected_block", "input_text": text,
        "expected_action": "no_change", "rule_id": "PROTECTED_BLOCK_PRESERVE",
        "manifest": {"discourse_spec": {"genre": "technical_architecture", "register": "professional_sharp", "target_audience": "general_public"}, "immutable_anchors": {"metrics": [], "domain_terms": []}},
        "linguistic_rationale": "Khối mã code, URL, email được che chắn an toàn, cấm báo lỗi giả."
    })

# 10 câu biền ngẫu, tiểu đối cổ văn mẫu mực (Nguyễn Hiến Lê, Cổ văn Trung Quốc - cấm phạt)
classical_cases = [
    ("CLAS_001", "Người có tài không phải là khó, tự đem dùng được cái tài của mình mới thật là khó.", "Nguyễn Hiến Lê - Luận về Giả Nghị"),
    ("CLAS_002", "Núi không tại cao, có tiên thì nổi danh; nước không tại sâu, có rồng thì hóa linh.", "Nguyễn Hiến Lê - Lậu thất minh"),
    ("CLAS_003", "Trời đất là quán trọ của vạn vật; quang âm là khách qua đường của trăm đời.", "Lý Bạch - Xuân dạ yến đào lý viên tự"),
    ("CLAS_004", "Lạc hà dữ cô vụ tề phi; thu thủy cộng trường thiên nhất sắc.", "Vương Bột - Đằng Vương Các tự"),
    ("CLAS_005", "Đồng thanh tương ứng, đồng khí tương cầu, nước chảy chỗ trũng, lửa tìm nơi khô.", "Nguyễn Hiến Lê dịch Dịch Kinh"),
    ("CLAS_006", "Cơm có nát thì khen là dẻo, cơm có sống thì khen là bùi.", "Tục ngữ Đề - Thuyết tiểu đối mẫu mực"),
    ("CLAS_007", "Râu hùm, hàm én, mày ngài, vai năm tấc rộng, thân mười thước cao.", "Nguyễn Du - Truyện Kiều"),
    ("CLAS_008", "Được mùa chớ phụ ngô khoai, đến năm thất bát lấy ai bạn cùng.", "Ca dao khuyên răn Đề - Thuyết"),
    ("CLAS_009", "Bên thì bừa cạn, bên thì cày sâu, nông phu sớm tối dãi dầu nắng mưa.", "Văn xuôi nông nghiệp đối xứng"),
    ("CLAS_010", "Sách này khó kiếm lắm, tiếng Anh dễ học.", "Câu Đề - Thuyết không chủ ngữ kinh điển"),
]
for cid, text, rationale in classical_cases:
    CASES.append({
        "id": cid, "category": "classical_rhythm", "input_text": text,
        "expected_action": "no_change", "rule_id": "CLASSICAL_ANTITHESIS_PRESERVE",
        "manifest": {"discourse_spec": {"genre": "essay", "register": "academic_formal", "target_audience": "general_public"}, "immutable_anchors": {"metrics": [], "domain_terms": []}},
        "linguistic_rationale": f"Cổ văn mẫu mực, nhịp điệu tầng bậc: {rationale} (Cấm cắt xén)."
    })

def main():
    DATA_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(DATA_FILE, "w", encoding="utf-8") as f:
        for c in CASES:
            f.write(json.dumps(c, ensure_ascii=False) + "\n")

    print(f"✅ Đã nạp thành công {len(CASES)}/200 ca kiểm chuẩn vào:")
    print(f"   {DATA_FILE}")

    # Thống kê số lượng từng nhóm
    groups = {
        "Nhóm 1 (Danh từ hóa)": len([c for c in CASES if c["id"].startswith("NOM_")]),
        "Nhóm 2 (Đề - Thuyết)": len([c for c in CASES if c["id"].startswith("TOP_")]),
        "Nhóm 3 (Hiện thực/Giả định & Trùng ngữ)": len([c for c in CASES if c["id"].startswith(("CONC_", "TEMP_", "PLEO_", "KIN_"))]),
        "Nhóm 4 (Mỏ neo, Code & Cổ văn)": len([c for c in CASES if c["id"].startswith(("ANC_", "PROT_", "CLAS_"))]),
    }
    for g, count in groups.items():
        print(f"   • {g}: {count}/50 câu")

if __name__ == "__main__":
    main()
