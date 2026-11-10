mod mcp;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use serde_json::json;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use vwi_core::auditor::{Auditor, AuditorConfig};
use vwi_core::report::AuditReport;
use vwi_core::types::Severity;

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
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum OutputFormat {
    Terminal,
    Json,
    Markdown,
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
        } => {
            let raw_text = read_input_text(pos_file, file, text)?;

            let mut config = AuditorConfig::default();
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
        } => {
            let raw_text = read_input_text(pos_file, file, text)?;

            let mut config = AuditorConfig::default();
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
            println!();
        }
    }
}
