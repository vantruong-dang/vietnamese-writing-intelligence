mod mcp;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use serde_json::json;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use tiny_http::{Header, Response, Server};
use vwi_core::auditor::{Auditor, AuditorConfig};
use vwi_core::report::AuditReport;
use vwi_core::types::{Register, Severity};

#[derive(Parser)]
#[command(name = "vwi")]
#[command(
    about = "Vietnamese Writing Intelligence - Động cơ kiểm toán văn phong & MCP Server thuần Rust",
    long_about = None
)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Kiểm toán nhanh lỗi cú pháp, sáo ngữ AI, văn dịch tiếng Anh và ngữ pháp Đề - Thuyết
    Check {
        /// Đường dẫn tới file văn bản truyền trực tiếp (hoặc dùng --file / --text)
        #[arg(value_name = "FILE")]
        pos_file: Option<PathBuf>,

        /// Đường dẫn tới file văn bản (hoặc dùng '-' để đọc từ STDIN)
        #[arg(short = 'i', long)]
        file: Option<PathBuf>,

        /// Văn bản truyền trực tiếp qua tham số dòng lệnh
        #[arg(short, long)]
        text: Option<String>,

        /// Định dạng xuất báo cáo (terminal, json, markdown)
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Terminal)]
        format: OutputFormat,

        /// Ngưỡng điểm tối thiểu (nếu điểm < fail-under, trả về exit code 1 cho CI/CD)
        #[arg(long)]
        fail_under: Option<f32>,

        /// Bỏ qua các cảnh báo Minor/Suggestion, chỉ bắt lỗi Major và Critical
        #[arg(long, default_value_t = false)]
        major_only: bool,

        /// Ngữ vực của văn bản (general, administrative, technical, literary, journalism)
        #[arg(short = 'r', long, value_enum, default_value_t = CliRegister::General)]
        register: CliRegister,
    },

    /// Kiểm toán chuyên sâu kèm đối soát bảo toàn số liệu/mỏ neo bất biến (Semantic Anchor Audit)
    Audit {
        /// Đường dẫn tới file văn bản truyền trực tiếp
        #[arg(value_name = "FILE")]
        pos_file: Option<PathBuf>,

        /// Đường dẫn tới file văn bản (hoặc dùng '-' để đọc từ STDIN)
        #[arg(short = 'i', long)]
        file: Option<PathBuf>,

        /// Văn bản truyền trực tiếp qua tham số dòng lệnh
        #[arg(short, long)]
        text: Option<String>,

        /// Danh sách các số liệu hoặc thực thể mỏ neo bất biến bắt buộc bảo toàn (có thể truyền nhiều lần)
        #[arg(short, long = "anchor")]
        anchors: Vec<String>,

        /// Danh sách các pattern_id muốn vô hiệu hóa (ví dụ: --disable-rule NOMINALIZATION_HEAVY)
        #[arg(short, long = "disable-rule")]
        disabled_rules: Vec<String>,

        /// Định dạng xuất báo cáo (terminal, json, markdown)
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Terminal)]
        format: OutputFormat,

        /// Ngưỡng điểm tối thiểu (nếu điểm < fail-under hoặc mất anchor, trả về exit code 1)
        #[arg(long)]
        fail_under: Option<f32>,

        /// Ngữ vực của văn bản (general, administrative, technical, literary, journalism)
        #[arg(short = 'r', long, value_enum, default_value_t = CliRegister::General)]
        register: CliRegister,
    },

    /// Tách từ tiếng Việt bằng thuật toán Viterbi Trellis (72.5k từ vựng nhúng) kèm nhãn POS
    Tokenize {
        /// Văn bản cần tách từ
        text: String,

        /// Hiển thị kèm nhãn từ loại Universal Dependencies (UPOS)
        #[arg(short, long, default_value_t = false)]
        pos: bool,

        /// Xuất kết quả dạng JSON
        #[arg(short, long, default_value_t = false)]
        json: bool,
    },

    /// Chạy bài test hiệu năng kiểm toán (Benchmark throughput)
    Benchmark {
        /// File văn bản dùng để benchmark
        file: PathBuf,
        /// Số vòng lặp chạy thử
        #[arg(short, long, default_value_t = 1000)]
        iterations: usize,
    },

    /// Khởi động Model Context Protocol (MCP) Server qua giao thức stdio cho Claude Desktop / Cursor
    Mcp,

    /// Khởi chạy Web Studio nội bộ trên trình duyệt (Dành cho người dùng văn phòng)
    Serve {
        /// Cổng lắng nghe HTTP
        #[arg(short, long, default_value_t = 4000)]
        port: u16,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum OutputFormat {
    Terminal,
    Json,
    Markdown,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug, Default)]
enum CliRegister {
    #[default]
    General,
    Administrative,
    Technical,
    Literary,
    Journalism,
}

impl From<CliRegister> for Register {
    fn from(r: CliRegister) -> Self {
        match r {
            CliRegister::General => Register::General,
            CliRegister::Administrative => Register::Administrative,
            CliRegister::Technical => Register::Technical,
            CliRegister::Literary => Register::Literary,
            CliRegister::Journalism => Register::Journalism,
        }
    }
}

fn read_input_text(
    pos_file: Option<PathBuf>,
    file: Option<PathBuf>,
    text: Option<String>,
) -> Result<String> {
    let target_file = file.or(pos_file);
    if let Some(path) = target_file {
        if path.to_str() == Some("-") {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        } else {
            fs::read_to_string(&path).with_context(|| format!("Không thể đọc file: {:?}", path))
        }
    } else if let Some(t) = text {
        Ok(t)
    } else {
        eprintln!(
            "{}",
            "Lỗi: Vui lòng cung cấp đường dẫn file hoặc chuỗi văn bản (--text)."
                .red()
                .bold()
        );
        std::process::exit(2);
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check {
            pos_file,
            file,
            text,
            format,
            fail_under,
            major_only,
            register,
        } => {
            let raw_text = read_input_text(pos_file, file, text)?;

            let mut config = AuditorConfig::default();
            config.set_register(register.into());
            if major_only {
                config.set_min_severity(Severity::Major);
            }
            let auditor = Auditor::with_config(config);
            let report = auditor.audit_document(&raw_text);

            match format {
                OutputFormat::Json => {
                    println!("{}", report.to_json());
                }
                OutputFormat::Markdown => {
                    println!("{}", report.to_markdown());
                }
                OutputFormat::Terminal => {
                    print_terminal_report(&report, &raw_text, &[]);
                }
            }

            if let Some(threshold) = fail_under {
                if report.score < threshold {
                    eprintln!(
                        "\n{} Điểm bài viết ({}) thấp hơn ngưỡng cho phép ({})!",
                        "CI FAILED:".red().bold(),
                        report.score.to_string().yellow().bold(),
                        threshold.to_string().cyan().bold()
                    );
                    std::process::exit(1);
                }
            }
        }

        Commands::Audit {
            pos_file,
            file,
            text,
            anchors,
            disabled_rules,
            format,
            fail_under,
            register,
        } => {
            let raw_text = read_input_text(pos_file, file, text)?;

            let mut config = AuditorConfig::default();
            config.set_register(register.into());
            for rule_id in disabled_rules {
                config.disable_rule(rule_id);
            }
            let auditor = Auditor::with_config(config);
            let report = auditor.audit_document(&raw_text);

            let missing_anchors: Vec<String> = anchors
                .iter()
                .filter(|a| !raw_text.contains(a.as_str()))
                .cloned()
                .collect();
            let facts_preserved = missing_anchors.is_empty();

            match format {
                OutputFormat::Json => {
                    let action_mode = if !facts_preserved {
                        "needs_author_decision"
                    } else if report.findings.is_empty() {
                        "no_change"
                    } else {
                        "review_comment"
                    };

                    let payload = json!({
                        "action_mode": action_mode,
                        "score": report.score,
                        "word_count": report.word_count,
                        "execution_time_ms": report.execution_time_ms,
                        "semantic_audit": {
                            "facts_preserved": facts_preserved,
                            "missing_anchors": missing_anchors,
                        },
                        "findings": report.findings,
                    });
                    println!("{}", serde_json::to_string_pretty(&payload)?);
                }
                OutputFormat::Markdown => {
                    println!("{}", report.to_markdown());
                }
                OutputFormat::Terminal => {
                    print_terminal_report(&report, &raw_text, &missing_anchors);
                }
            }

            if !facts_preserved {
                eprintln!(
                    "\n{} Phát hiện thất thoát dữ kiện mỏ neo (Immutable Anchors): {:?}",
                    "AUDIT FAILED:".red().bold(),
                    missing_anchors
                );
                std::process::exit(1);
            }

            if let Some(threshold) = fail_under {
                if report.score < threshold {
                    eprintln!(
                        "\n{} Điểm bài viết ({}) thấp hơn ngưỡng cho phép ({})!",
                        "AUDIT FAILED:".red().bold(),
                        report.score.to_string().yellow().bold(),
                        threshold.to_string().cyan().bold()
                    );
                    std::process::exit(1);
                }
            }
        }

        Commands::Tokenize { text, pos, json } => {
            let tokens = vwi_core::tokenize_viterbi(&text);
            if json {
                println!("{}", serde_json::to_string_pretty(&tokens)?);
            } else if pos {
                let words: Vec<String> = tokens
                    .into_iter()
                    .map(|t| {
                        let tag = t.pos.map(|p| p.as_str()).unwrap_or("OTHER");
                        format!("[{}/{}]", t.text, tag)
                    })
                    .collect();
                println!("{}", words.join(" "));
            } else {
                let words: Vec<String> =
                    tokens.into_iter().map(|t| format!("[{}]", t.text)).collect();
                println!("{}", words.join(" "));
            }
        }

        Commands::Benchmark { file, iterations } => {
            let text = fs::read_to_string(&file)
                .with_context(|| format!("Không thể đọc file: {:?}", file))?;
            println!("Đang benchmark với {} vòng lặp...", iterations);

            let auditor = Auditor::default();
            let start = std::time::Instant::now();
            for _ in 0..iterations {
                let _ = auditor.audit(&text);
            }
            let total_time = start.elapsed();
            let ops_per_sec = (iterations as f64) / total_time.as_secs_f64();

            println!("{}", "=== KẾT QUẢ BENCHMARK RUST CORE ===".green().bold());
            println!("Thời gian chạy : {:.4} giây", total_time.as_secs_f64());
            println!(
                "Tốc độ xử lý   : {} ops/giây",
                format!("{:.0}", ops_per_sec).yellow().bold()
            );
        }

        Commands::Mcp => {
            mcp::run_stdio_server()?;
        }

        Commands::Serve { port } => {
            start_local_web_server(port)?;
        }
    }

    Ok(())
}

fn print_terminal_report(report: &AuditReport, text: &str, missing_anchors: &[String]) {
    println!(
        "\n{}",
        "=======================================================".bright_blue()
    );
    println!("        VIETNAMESE WRITING INTELLIGENCE (VWI)         ");
    println!(
        "{}\n",
        "=======================================================".bright_blue()
    );

    let score_str = format!("{:.1}/100", report.score);
    let score_colored = if report.score >= 85.0 {
        score_str.green().bold()
    } else if report.score >= 65.0 {
        score_str.yellow().bold()
    } else {
        score_str.red().bold()
    };

    println!("  Điểm chất lượng : {}", score_colored);
    println!("  Số lượng từ     : {}", report.word_count.to_string().cyan());
    println!(
        "  Lỗi phát hiện   : {}",
        report.findings.len().to_string().magenta()
    );
    if !missing_anchors.is_empty() {
        println!(
            "  Bảo toàn dữ kiện: {} (Mất: {:?})",
            "THẤT BẠI".red().bold(),
            missing_anchors
        );
    }
    println!("  Thời gian chạy  : {:.2} ms\n", report.execution_time_ms);

    if report.findings.is_empty() && missing_anchors.is_empty() {
        println!(
            "{}",
            "✅ Văn bản xuất sắc! Không phát hiện lỗi cấu trúc hay sáo ngữ."
                .green()
                .bold()
        );
        return;
    }

    if !report.findings.is_empty() {
        println!("{}", "DANH SÁCH CHI TIẾT CÁC LỖI PHÁT HIỆN:".bold());
        let chars: Vec<char> = text.chars().collect();

        for (i, f) in report.findings.iter().enumerate() {
            let sev = Severity::parse(&f.severity);
            let badge = match sev {
                Severity::Critical => "[CRITICAL]".red().bold(),
                Severity::Major => "[MAJOR]".red().bold(),
                Severity::Minor => "[MINOR]".yellow().bold(),
                Severity::Suggestion => "[SUGGEST]".cyan().bold(),
                Severity::Advisory => "[ADVISORY]".bright_blue().bold(),
            };

            let snippet = if !f.matched_span.is_empty() {
                f.matched_span.clone()
            } else {
                chars[f.start..f.end.min(chars.len())].iter().collect()
            };

            println!(
                "  {}. {} {} tại ({}..{})",
                i + 1,
                badge,
                f.pattern_id.cyan(),
                f.start,
                f.end
            );
            println!("     Đoạn văn vi phạm : \"{}\"", snippet.underline());
            println!("     Chi tiết         : {}", f.rationale);
            if let Some(sugg) = &f.suggestion {
                println!("     Gợi ý khắc phục  : {}", sugg.green().bold());
            }
            println!();
        }
    }
}

fn is_allowed_host(host_header: Option<&str>, port: u16) -> bool {
    let Some(host) = host_header else {
        return false;
    };
    let host = host.trim().to_lowercase();
    let port_str = port.to_string();

    host == "localhost"
        || host == format!("localhost:{}", port_str)
        || host == "127.0.0.1"
        || host == format!("127.0.0.1:{}", port_str)
        || host == "[::1]"
        || host == format!("[::1]:{}", port_str)
}

fn is_allowed_origin(origin_header: Option<&str>, port: u16) -> bool {
    let Some(origin) = origin_header else {
        // Trình duyệt gửi request cùng origin (hoặc client cục bộ) không bắt buộc gửi header Origin
        return true;
    };
    let origin = origin.trim().to_lowercase();
    let port_str = port.to_string();

    origin == format!("http://localhost:{}", port_str)
        || origin == format!("http://127.0.0.1:{}", port_str)
        || origin == format!("http://[::1]:{}", port_str)
        || origin == "http://localhost"
        || origin == "http://127.0.0.1"
}

fn with_security_headers(
    mut response: Response<std::io::Cursor<Vec<u8>>>,
) -> Response<std::io::Cursor<Vec<u8>>> {
    response.add_header(Header::from_bytes(&b"X-Content-Type-Options"[..], &b"nosniff"[..]).unwrap());
    response.add_header(Header::from_bytes(&b"X-Frame-Options"[..], &b"DENY"[..]).unwrap());
    response.add_header(
        Header::from_bytes(
            &b"Content-Security-Policy"[..],
            &b"default-src 'self' 'unsafe-inline'; frame-ancestors 'none'"[..],
        )
        .unwrap(),
    );
    response
}

fn start_local_web_server(port: u16) -> Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let server = Server::http(&addr)
        .map_err(|e| anyhow::anyhow!("Không thể khởi động HTTP Server trên {}: {}", addr, e))?;

    let url = format!("http://{}", addr);
    println!(
        "\n{}",
        "=======================================================".bright_blue()
    );
    println!("      VIETNAMESE WRITING INTELLIGENCE - WEB STUDIO     ");
    println!(
        "{}\n",
        "=======================================================".bright_blue()
    );
    println!("  {}     {}", "Trạng thái :".bold(), "Đang hoạt động".green().bold());
    println!("  {}     {}", "Địa chỉ    :".bold(), url.cyan().underline().bold());
    println!("  {}     {}", "Bảo mật    :".bold(), "Bật chống DNS Rebinding & CSRF Guard".green());
    println!("  {}     Dán văn bản vào trình duyệt để kiểm toán tức thì", "Hướng dẫn  :".bold());
    println!("\n  {}", "Nhấn Ctrl+C để dừng máy chủ...\n".yellow());

    // Tự động mở trình duyệt mặc định
    if let Err(e) = opener::open(&url) {
        eprintln!("  {} Không thể tự động mở trình duyệt: {}", "Lưu ý:".yellow(), e);
        eprintln!("  Vui lòng mở thủ công đường dẫn: {}", url.cyan());
    }

    let auditor = Auditor::default();

    for mut request in server.incoming_requests() {
        // 1. Kiểm tra Host header (Phòng chống DNS Rebinding)
        let host_val = request
            .headers()
            .iter()
            .find(|h| h.field.equiv("host"))
            .map(|h| h.value.as_str());

        if !is_allowed_host(host_val, port) {
            let res = with_security_headers(
                Response::from_string("403 Forbidden: Invalid Host Header (DNS Rebinding Guard)")
                    .with_status_code(403),
            );
            let _ = request.respond(res);
            continue;
        }

        // 2. Kiểm tra Origin header (Phòng chống CSRF / Cross-Site Scripting)
        let origin_val = request
            .headers()
            .iter()
            .find(|h| h.field.equiv("origin"))
            .map(|h| h.value.as_str());

        if !is_allowed_origin(origin_val, port) {
            let res = with_security_headers(
                Response::from_string("403 Forbidden: Untrusted Origin (CSRF Guard)")
                    .with_status_code(403),
            );
            let _ = request.respond(res);
            continue;
        }

        // 3. Kiểm tra Sec-Fetch-Site (Nếu trình duyệt gửi cross-site, từ chối ngay)
        let sec_fetch_site = request
            .headers()
            .iter()
            .find(|h| h.field.equiv("sec-fetch-site"))
            .map(|h| h.value.as_str());

        if let Some(site) = sec_fetch_site {
            if site.eq_ignore_ascii_case("cross-site") {
                let res = with_security_headers(
                    Response::from_string("403 Forbidden: Cross-site request rejected")
                        .with_status_code(403),
                );
                let _ = request.respond(res);
                continue;
            }
        }

        let method = request.method().as_str().to_string();
        let path = request.url().to_string();

        if (method == "GET" || method == "HEAD") && (path == "/" || path == "/index.html") {
            let html = include_str!("ui.html");
            let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap();
            let response = with_security_headers(Response::from_string(html).with_header(header));
            let _ = request.respond(response);
        } else if method == "POST" && path == "/api/audit" {
            // Giới hạn kích thước payload tối đa 2 MB để chống cạn kiệt tài nguyên (DoS)
            let mut body = String::new();
            let mut reader = request.as_reader().take(2 * 1024 * 1024);
            if let Err(e) = reader.read_to_string(&mut body) {
                let err_res = json!({ "error": format!("Không thể đọc nội dung: {}", e) }).to_string();
                let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
                let response = with_security_headers(
                    Response::from_string(err_res).with_status_code(400).with_header(header),
                );
                let _ = request.respond(response);
                continue;
            }

            #[derive(serde::Deserialize)]
            struct AuditRequest {
                text: String,
            }

            match serde_json::from_str::<AuditRequest>(&body) {
                Ok(req) => {
                    let report = auditor.audit_document(&req.text);
                    let json_str = report.to_json();
                    let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
                    let response = with_security_headers(
                        Response::from_string(json_str).with_header(header),
                    );
                    let _ = request.respond(response);
                }
                Err(e) => {
                    let err_res = json!({ "error": format!("Dữ liệu JSON không hợp lệ: {}", e) }).to_string();
                    let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
                    let response = with_security_headers(
                        Response::from_string(err_res).with_status_code(400).with_header(header),
                    );
                    let _ = request.respond(response);
                }
            }
        } else {
            let response = with_security_headers(
                Response::from_string("404 Not Found").with_status_code(404),
            );
            let _ = request.respond(response);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_allowed_host_dns_rebinding_guard() {
        let port = 4000;
        // Hợp lệ: localhost và IP loopback
        assert!(is_allowed_host(Some("localhost:4000"), port));
        assert!(is_allowed_host(Some("localhost"), port));
        assert!(is_allowed_host(Some("127.0.0.1:4000"), port));
        assert!(is_allowed_host(Some("127.0.0.1"), port));
        assert!(is_allowed_host(Some("[::1]:4000"), port));
        assert!(is_allowed_host(Some("[::1]"), port));

        // Không hợp lệ: Tấn công DNS Rebinding
        assert!(!is_allowed_host(Some("evil.com:4000"), port));
        assert!(!is_allowed_host(Some("attacker.org"), port));
        assert!(!is_allowed_host(Some("192.168.1.100:4000"), port));
        assert!(!is_allowed_host(None, port));
    }

    #[test]
    fn test_is_allowed_origin_csrf_guard() {
        let port = 4000;
        // Hợp lệ: Không có origin (cùng nguồn nội bộ hoặc direct navigation)
        assert!(is_allowed_origin(None, port));
        // Hợp lệ: Origin từ localhost và loopback
        assert!(is_allowed_origin(Some("http://localhost:4000"), port));
        assert!(is_allowed_origin(Some("http://127.0.0.1:4000"), port));
        assert!(is_allowed_origin(Some("http://[::1]:4000"), port));

        // Không hợp lệ: Tấn công CSRF từ trang web ngoài
        assert!(!is_allowed_origin(Some("http://evil.com"), port));
        assert!(!is_allowed_origin(Some("https://attacker.site"), port));
        assert!(!is_allowed_origin(Some("null"), port));
        assert!(!is_allowed_origin(Some("http://localhost:8080"), port));
    }
}
