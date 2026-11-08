"""Giao diện dòng lệnh CLI cho vn-intel."""
import argparse
import json
import sys
from .orchestrator import V3Orchestrator


def main() -> None:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(encoding="utf-8")

    parser = argparse.ArgumentParser(
        prog="vn-intel",
        description="Vietnamese Writing Intelligence CLI (v0.1.0)"
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    audit_parser = subparsers.add_parser("audit", help="Kiểm toán văn bản dựa trên Manifest và Profile")
    audit_parser.add_argument("text_file", help="File văn bản đầu vào (.txt / .md)")
    audit_parser.add_argument("--profile", required=True, help="Đường dẫn file voice-profile.json")
    audit_parser.add_argument("--manifest", required=True, help="Đường dẫn file draft-manifest.json")

    args = parser.parse_args()

    if args.command == "audit":
        try:
            with open(args.text_file, "r", encoding="utf-8") as f:
                text = f.read()
            with open(args.profile, "r", encoding="utf-8") as f:
                profile = json.load(f)
            with open(args.manifest, "r", encoding="utf-8") as f:
                manifest = json.load(f)

            orchestrator = V3Orchestrator(profile)
            result = orchestrator.process(text, manifest)
            print(json.dumps(result, ensure_ascii=False, indent=2))
        except Exception as e:
            sys.stderr.write(f"Lỗi: {str(e)}\n")
            sys.exit(1)


if __name__ == "__main__":
    main()
