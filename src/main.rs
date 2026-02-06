use anyhow::{Context, Result};
use flexi_logger::{FileSpec, Logger, WriteMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::io::{self, BufRead, Write};

#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
}

fn main() -> Result<()> {
    let exe_path = env::current_exe().context("Failed to get exe path")?;
    let log_dir = exe_path.parent().context("Failed to get parent dir")?;

    Logger::try_with_str("debug")?
        .log_to_file(FileSpec::default().directory(log_dir).basename("hello-mcp"))
        .duplicate_to_stderr(flexi_logger::Duplicate::All)
        .write_mode(WriteMode::Direct)
        .start()?;

    log::info!("Hello World MCP Server starting (V2 - Strict)...");

    let stdin = io::stdin();
    let reader = stdin.lock();
    let mut stdout = io::stdout();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        log::debug!("Received: {}", line);

        if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(&line) {
            match req.method.as_str() {
                "initialize" => {
                    log::info!("Initializing MCP server...");
                    let client_version = req
                        .params
                        .as_ref()
                        .and_then(|p| p.get("protocolVersion"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("2024-11-05");

                    log::info!("Client requested protocol version: {}", client_version);

                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(serde_json::json!({
                            "protocolVersion": client_version,
                            "capabilities": {
                                "tools": {},
                                "resources": {},
                                "prompts": {}
                            },
                            "serverInfo": {
                                "name": "hello-world-mcp",
                                "version": "1.0.1"
                            }
                        })),
                        error: None,
                        id: req.id,
                    };
                    send_response(&mut stdout, response)?;
                }
                "tools/list" => {
                    log::info!("Handling tools/list request...");
                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(serde_json::json!({
                            "tools": [
                                {
                                    "name": "hello",
                                    "description": "Returns a friendly greeting",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "name": {
                                                "type": "string",
                                                "description": "The name to greet"
                                            }
                                        },
                                        "required": ["name"]
                                    }
                                }
                            ]
                        })),
                        error: None,
                        id: req.id,
                    };
                    send_response(&mut stdout, response)?;
                }
                "tools/call" => {
                    log::info!("Handling tools/call request...");
                    let name = req
                        .params
                        .as_ref()
                        .and_then(|p| p.get("arguments"))
                        .and_then(|a| a.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("World");

                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(serde_json::json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": format!("Hello, {}! This is a greeting from the MCP server.", name)
                                }
                            ],
                            "isError": false
                        })),
                        error: None,
                        id: req.id,
                    };
                    send_response(&mut stdout, response)?;
                }
                "notifications/initialized" => {
                    log::info!("Received notifications/initialized. Handshake complete.");
                }
                "list_tools" => {
                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(serde_json::json!({
                            "tools": [
                                {
                                    "name": "greet",
                                    "description": "Returns a friendly greeting.",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "name": { "type": "string", "description": "Name to greet" }
                                        },
                                        "required": ["name"]
                                    }
                                }
                            ]
                        })),
                        error: None,
                        id: req.id,
                    };
                    send_response(&mut stdout, response)?;
                }
                "call_tool" => {
                    let name = req
                        .params
                        .as_ref()
                        .and_then(|p| p.get("name").cloned())
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .unwrap_or_else(|| "World".to_string());

                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(serde_json::json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": format!("Hello, {}! Strict handshake successful.", name)
                                }
                            ]
                        })),
                        error: None,
                        id: req.id,
                    };
                    send_response(&mut stdout, response)?;
                }
                _ => {
                    if req.id.is_some() {
                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: None,
                            error: Some(serde_json::json!({
                                "code": -32601,
                                "message": format!("Method not found: {}", req.method)
                            })),
                            id: req.id,
                        };
                        send_response(&mut stdout, response)?;
                    }
                }
            }
        }
    }

    log::info!("Exiting Hello World MCP Server.");
    Ok(())
}

fn send_response(stdout: &mut io::Stdout, response: JsonRpcResponse) -> Result<()> {
    let resp_str = serde_json::to_string(&response)?;
    stdout.write_all(resp_str.as_bytes())?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    log::debug!("Sent: {}", resp_str);
    Ok(())
}
