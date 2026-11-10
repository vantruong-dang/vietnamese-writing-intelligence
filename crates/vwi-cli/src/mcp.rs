use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use vwi_core::auditor::{Auditor, AuditorConfig};
use vwi_core::tokenize_viterbi;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

/// Xử lý một thông điệp JSON-RPC 2.0 theo chuẩn Model Context Protocol (MCP).
/// Trả về `Some(JsonRpcResponse)` cho các request có `id`, hoặc `None` cho các notification.
pub fn handle_mcp_request(req: &JsonRpcRequest) -> Option<JsonRpcResponse> {
    // Notifications (như notifications/initialized) không có id và không cần phản hồi
    let id = req.id.clone()?;

    match req.method.as_str() {
        "initialize" => {
            let protocol_version = req
                .params
                .as_ref()
                .and_then(|p| p.get("protocolVersion"))
                .and_then(|v| v.as_str())
                .unwrap_or("2024-11-05");

            Some(JsonRpcResponse {
                jsonrpc: "2.0",
                id,
                result: Some(json!({
                    "protocolVersion": protocol_version,
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": "vietnamese-writing-intelligence",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                })),
                error: None,
            })
        }

        "ping" => Some(JsonRpcResponse {
            jsonrpc: "2.0",
            id,
            result: Some(json!({})),
            error: None,
        }),

        "resources/list" => Some(JsonRpcResponse {
            jsonrpc: "2.0",
            id,
            result: Some(json!({ "resources": [] })),
            error: None,
        }),

        "prompts/list" => Some(JsonRpcResponse {
            jsonrpc: "2.0",
            id,
            result: Some(json!({ "prompts": [] })),
            error: None,
        }),

        "tools/list" => {
            let audit_schema = json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Văn bản tiếng Việt cần kiểm toán ngữ pháp và phong cách"
                    },
                    "compact": {
                        "type": "boolean",
                        "description": "Xuất JSON nén gọn tiết kiệm token cho LLM (mặc định: true)",
                        "default": true
                    },
                    "anchors": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Danh sách số liệu hoặc thuật ngữ mỏ neo bất biến cần kiểm tra bảo toàn"
                    }
                },
                "required": ["text"]
            });

            Some(JsonRpcResponse {
                jsonrpc: "2.0",
                id,
                result: Some(json!({
                    "tools": [
                        {
                            "name": "audit_text",
                            "description": "Kiểm toán văn bản tiếng Việt bằng lõi Rust: bắt lỗi Khung đề - Thuyết (Cao Xuân Hạo), bị động nhân tạo, sáo ngữ AI, đệm hành chính và chấm điểm chất lượng (0-100).",
                            "inputSchema": audit_schema
                        },
                        {
                            "name": "audit_vietnamese_text",
                            "description": "Alias tương thích ngược của audit_text: Kiểm toán văn phong tiếng Việt tốc độ native.",
                            "inputSchema": audit_schema
                        },
                        {
                            "name": "tokenize_vietnamese",
                            "description": "Tách từ tiếng Việt bằng thuật toán Viterbi Trellis (72.5k từ điển) kèm gán nhãn từ loại Universal Dependencies (UPOS).",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "text": {
                                        "type": "string",
                                        "description": "Văn bản tiếng Việt cần tách từ và gán nhãn POS"
                                    }
                                },
                                "required": ["text"]
                            }
                        }
                    ]
                })),
                error: None,
            })
        }

        "tools/call" => {
            let params = match &req.params {
                Some(p) => p,
                None => {
                    return Some(JsonRpcResponse {
                        jsonrpc: "2.0",
                        id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: "Missing params in tools/call".to_string(),
                        }),
                    })
                }
            };

            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));

            match tool_name {
                "audit_text" | "audit_vietnamese_text" => {
                    let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
                    let compact = args
                        .get("compact")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);
                    let anchors: Vec<String> = args
                        .get("anchors")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();

                    let auditor = Auditor::with_config(AuditorConfig::default());
                    let report = auditor.audit_document(text);

                    let missing_anchors: Vec<String> = anchors
                        .into_iter()
                        .filter(|a| !text.contains(a.as_str()))
                        .collect();
                    let facts_preserved = missing_anchors.is_empty();

                    let payload_str = if compact {
                        let action_mode = if !facts_preserved {
                            "needs_author_decision"
                        } else if report.findings.is_empty() {
                            "no_change"
                        } else {
                            "review_comment"
                        };

                        let compact_findings: Vec<Value> = report
                            .findings
                            .iter()
                            .map(|f| {
                                json!({
                                    "rule": f.pattern_id,
                                    "severity": f.severity,
                                    "span": f.matched_span,
                                    "start": f.start,
                                    "end": f.end
                                })
                            })
                            .collect();

                        let mut obj = json!({
                            "mode": action_mode,
                            "score": report.score,
                            "facts_preserved": facts_preserved,
                            "findings": compact_findings
                        });

                        if !missing_anchors.is_empty() {
                            obj["missing_anchors"] = json!(missing_anchors);
                        }

                        serde_json::to_string(&obj).unwrap_or_default()
                    } else {
                        report.to_json()
                    };

                    Some(JsonRpcResponse {
                        jsonrpc: "2.0",
                        id,
                        result: Some(json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": payload_str
                                }
                            ],
                            "isError": false
                        })),
                        error: None,
                    })
                }

                "tokenize_vietnamese" => {
                    let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
                    let tokens = tokenize_viterbi(text);
                    let token_list: Vec<Value> = tokens
                        .into_iter()
                        .map(|t| {
                            json!({
                                "text": t.text,
                                "pos": t.pos.map(|p| p.as_str()).unwrap_or("OTHER"),
                                "start": t.start,
                                "end": t.end,
                                "is_compound": t.is_compound
                            })
                        })
                        .collect();

                    let payload_str = serde_json::to_string(&token_list).unwrap_or_default();

                    Some(JsonRpcResponse {
                        jsonrpc: "2.0",
                        id,
                        result: Some(json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": payload_str
                                }
                            ],
                            "isError": false
                        })),
                        error: None,
                    })
                }

                _ => Some(JsonRpcResponse {
                    jsonrpc: "2.0",
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: format!("Unknown tool: {}", tool_name),
                    }),
                }),
            }
        }

        _ => Some(JsonRpcResponse {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", req.method),
            }),
        }),
    }
}

/// Khởi chạy MCP Server giao thức JSON-RPC 2.0 qua luồng chuẩn (stdin / stdout).
pub fn run_stdio_server() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line_res in stdin.lock().lines() {
        let line = line_res?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Content-Length:") {
            continue;
        }

        match serde_json::from_str::<JsonRpcRequest>(trimmed) {
            Ok(req) => {
                if let Some(resp) = handle_mcp_request(&req) {
                    let out = serde_json::to_string(&resp)?;
                    writeln!(stdout, "{}", out)?;
                    stdout.flush()?;
                }
            }
            Err(e) => {
                let err_resp = JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                    }),
                };
                let out = serde_json::to_string(&err_resp)?;
                writeln!(stdout, "{}", out)?;
                stdout.flush()?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_initialize_and_tools_list() {
        let init_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: Some(json!({"protocolVersion": "2024-11-05"})),
        };
        let init_resp = handle_mcp_request(&init_req).unwrap();
        assert!(init_resp.error.is_none());
        let res = init_resp.result.unwrap();
        assert_eq!(res["serverInfo"]["name"], "vietnamese-writing-intelligence");

        let list_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(2)),
            method: "tools/list".to_string(),
            params: None,
        };
        let list_resp = handle_mcp_request(&list_req).unwrap();
        let tools = list_resp.result.unwrap()["tools"].as_array().unwrap().clone();
        assert_eq!(tools.len(), 3);
    }

    #[test]
    fn test_mcp_call_audit_and_tokenize() {
        let audit_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(3)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "audit_vietnamese_text",
                "arguments": {
                    "text": "Trong tình hình kinh tế hiện nay đòi hỏi doanh nghiệp phải tái cơ cấu.",
                    "compact": true,
                    "anchors": ["doanh nghiệp"]
                }
            })),
        };
        let resp = handle_mcp_request(&audit_req).unwrap();
        let content_text = resp.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap()
            .to_string();
        let parsed: Value = serde_json::from_str(&content_text).unwrap();
        assert_eq!(parsed["mode"], "review_comment");
        assert_eq!(parsed["facts_preserved"], true);
        assert_eq!(parsed["findings"][0]["rule"], "CRIPPLED_PREPOSITIONAL_TOPIC");

        let tok_req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(4)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "tokenize_vietnamese",
                "arguments": {
                    "text": "Học sinh học sinh học."
                }
            })),
        };
        let tok_resp = handle_mcp_request(&tok_req).unwrap();
        let tok_text = tok_resp.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap()
            .to_string();
        let tokens: Value = serde_json::from_str(&tok_text).unwrap();
        assert_eq!(tokens[0]["text"], "Học sinh");
        assert_eq!(tokens[0]["pos"], "NOUN");
    }
}
