"""Bộ lập kế hoạch phác thảo văn bản (Draft Planner) theo ngữ pháp chức năng tiếng Việt."""
from __future__ import annotations

from typing import Any, Dict, Optional


class DraftPlanner:
    """Lập kế hoạch phác thảo bản thảo dựa trên DraftManifest và VoiceProfile."""

    GENRE_PROMPT_DESCRIPTIONS = {
        "technical_architecture": "Bài viết kiến trúc kỹ thuật chuyên sâu, chặt chẽ, chính xác về mặt hệ thống.",
        "financial_analysis": "Bản tin phân tích tài chính vĩ mô, lập luận sắc bén, giữ vững thuật ngữ thị trường.",
        "policy_brief": "Bản khuyến nghị chính sách rõ ràng, văn phong công vụ hiện đại, gãy gọn.",
        "essay": "Tiểu luận học thuật hoặc văn nghệ sâu sắc, cấu trúc Đề - Thuyết uyển chuyển.",
        "administrative": "Văn bản hành chính chuẩn mực, ngắn gọn, minh bạch.",
        "social_copy": "Nội dung truyền thông xã hội tự nhiên, hấp dẫn, không sáo rỗng.",
        "unspecified": "Văn bản phân tích đa ngành mạch lạc.",
    }

    REGISTER_PROMPT_DESCRIPTIONS = {
        "academic_formal": "Học thuật trang trọng, dùng từ ngữ chuẩn xác, tránh khẩu ngữ.",
        "professional_sharp": "Chuyên nghiệp, sắc bén, trực diện, không dài dòng.",
        "neutral_journalistic": "Báo chí trung tính, khách quan, giàu thông tin thực chứng.",
        "conversational": "Tự nhiên như đàm thoại thường ngày nhưng có chiều sâu.",
        "intimate": "Thân mật, gần gũi.",
        "unspecified": "Tự nhiên, sáng rõ.",
    }

    def build_system_prompt(self, manifest: Dict[str, Any], voice_profile: Optional[Dict[str, Any]] = None) -> str:
        """Xây dựng System Prompt định hướng phong cách bản địa và các ranh giới bất biến."""
        dspec = manifest.get("discourse_spec", {})
        genre = dspec.get("genre", "unspecified")
        register = dspec.get("register", "unspecified")
        dialect = dspec.get("dialect_bias", "all_neutral")
        audience = dspec.get("target_audience", "unspecified")

        genre_desc = self.GENRE_PROMPT_DESCRIPTIONS.get(genre, genre)
        reg_desc = self.REGISTER_PROMPT_DESCRIPTIONS.get(register, register)

        anchors = manifest.get("immutable_anchors", {})
        metrics = anchors.get("metrics", [])
        domain_terms = anchors.get("domain_terms", [])
        named_entities = anchors.get("named_entities", [])
        causal_invariants = anchors.get("causal_invariants", [])

        voice_profile = voice_profile or {}
        banned_phrases = voice_profile.get("banned_phrases", [])

        lines = [
            "Bạn là Native Drafter - chuyên gia biên soạn văn bản tiếng Việt bản địa đỉnh cao.",
            f"- Thể loại (Genre): {genre} ({genre_desc})",
            f"- Ngữ vực (Register): {register} ({reg_desc})",
            f"- Đối tượng tiếp nhận (Target Audience): {audience}",
            f"- Thiên hướng phương ngữ: {dialect}",
            "",
            "NGUYÊN TẮC NGỮ PHÁP TIẾNG VIỆT (GS. Cao Xuân Hạo & Nguyễn Hiến Lê):",
            "1. Cấu trúc Đề - Thuyết: Đưa thông tin đã biết/phạm vi bàn luận lên làm Khung đề hoặc Chủ đề ở đầu câu.",
            "2. Tuyệt đối không sao phỏng ngữ pháp châu Âu: Không dịch thô 'in X days' ra cuối câu, không dùng bị động 'được/bị ... bởi' nhân tạo.",
            "3. Không lạm dụng danh từ hóa (việc, sự, quá trình...) thay cho vị từ hành động trực tiếp.",
            "4. Phân biệt rõ ràng: 'Tuy/Mặc dù' cho sự việc hiện thực đã/đang xảy ra; 'Dù' cho sự việc giả định phi hiện thực.",
            "",
            "CÁC MỎ NEO BẤT BIẾN (BẮT BUỘC GIỮ NGUYÊN VẸN 100%):",
        ]

        if metrics:
            lines.append(f"- Số liệu định lượng: {', '.join(metrics)}")
        if domain_terms:
            lines.append(f"- Thuật ngữ chuyên ngành (cấm dịch bừa): {', '.join(domain_terms)}")
        if named_entities:
            lines.append(f"- Thực thể định danh: {', '.join(named_entities)}")
        if causal_invariants:
            lines.append(f"- Mối quan hệ nhân quả cốt lõi: {', '.join(causal_invariants)}")

        if banned_phrases:
            lines.append("")
            lines.append(f"TỪ/CỤM TỪ CẤM XUẤT HIỆN: {', '.join(banned_phrases)}")

        return "\n".join(lines)

    def build_user_prompt(self, manifest: Dict[str, Any], user_instructions: str = "") -> str:
        """Xây dựng User Prompt chi tiết hóa kế hoạch triển khai từng đoạn văn (Topic-Comment Plan)."""
        plan = manifest.get("topic_comment_plan", [])
        lines = ["Hãy phác thảo nội dung hoàn chỉnh theo dàn bài Đề - Thuyết (Topic-Comment Plan) dưới đây:"]

        if plan:
            for idx, step in enumerate(plan, 1):
                sec_id = step.get("section_id", f"Phần {idx}")
                topic = step.get("topic", "")
                comment = step.get("intended_comment") or step.get("comment_guideline", "")
                chain_pref = step.get("clause_chaining_preference", "")
                pref_note = f" (Kiểu nối câu: {chain_pref})" if chain_pref else ""
                lines.append(f"{idx}. [{sec_id}] Đề (Topic): {topic}")
                lines.append(f"   Thuyết (Comment Guideline): {comment}{pref_note}")
        else:
            lines.append("- Triển khai lập luận mạch lạc, sáng rõ theo thông điệp chính.")

        if user_instructions:
            lines.append("")
            lines.append(f"Yêu cầu bổ sung từ tác giả: {user_instructions}")

        return "\n".join(lines)

    def build_prompts(
        self,
        manifest: Dict[str, Any],
        voice_profile: Optional[Dict[str, Any]] = None,
        user_instructions: str = "",
    ) -> tuple[str, str]:
        """Tạo đồng thời cặp prompt (system_prompt, user_prompt) phục vụ LLM / MCP."""
        sys_prompt = self.build_system_prompt(manifest, voice_profile)
        usr_prompt = self.build_user_prompt(manifest, user_instructions)
        return sys_prompt, usr_prompt
