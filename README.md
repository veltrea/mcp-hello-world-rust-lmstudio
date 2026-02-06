# MCP Hello World for LM Studio

This is a minimal, standalone MCP (Model Context Protocol) server written in Rust, specifically designed as a reference implementation for **LM Studio**.

## 💡 Purpose
While the official MCP SDKs provide high-level abstractions, LM Studio currently has specific, undocumented requirements (Golden Rules) that can cause standard implementations to fail. 

> [!NOTE]
> These observations were gathered during a debug session on **2026-02-05** while trying to get an MCP server working in LM Studio. As LM Studio is rapidly evolving, these should be treated as current "hints" rather than official specifications.

This project implements the protocol from scratch using raw JSON-RPC over Standard Input/Output to demonstrate exactly how to satisfy these requirements as of today.

## 🏆 Integration Guide for AI-Assisted Development
AI assistants like **Google Antigravity** already possess a deep understanding of the standard MCP specification. By simply providing these 4 "bridge rules" as external constraints, you can often generate a fully compatible, small-scale MCP server for LM Studio in a single shot:

Based on some trial and error on **2026-02-05**, the following implementation details seem to ensure stable integration with the current version of LM Studio:

1.  **Binary Location**: LM Studio appears most stable when the binary is located in a specific directory structure.
    -   Recommended Path: `~/.lmstudio/extensions/plugins/mcp/<plugin-name>/`
    -   Configure your `mcp.json` to point to the binary in this location.

2.  **Protocol Version Verification**: During the `initialize` handshake, LM Studio sends a `protocolVersion` (e.g., `2025-06-18`). Mirroring this exact version back to the client improves compatibility.
    ```rust
    // snippet from main.rs
    "initialize" => {
        let client_version = req.params.as_ref()
            .and_then(|p| p.get("protocolVersion"))
            .and_then(|v| v.as_str())
            .unwrap_or("2024-11-05");

        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({
                "protocolVersion": client_version, // Mirror client's version
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "mcp-hello-world", "version": "0.1.0" }
            })),
            id: req.id,
        };
    }
    ```

3.  **Strict JSON-RPC Compliance (No `null` Errors)**: LM Studio's parser can be sensitive to `null` values. For successful responses, it is safer to omit the `error` field entirely rather than setting it to `null`.
    ```rust
    #[derive(Debug, Serialize)]
    struct JsonRpcResponse {
        pub jsonrpc: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub result: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")] // Field is omitted if None
        pub error: Option<Value>,
        pub id: Option<Value>,
    }
    ```

4.  **Tool Call Response Structure**: Tool outputs must be wrapped in a `content` array and include an `isError` flag.
    ```rust
    "tools/call" => {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({
                "content": [{ "type": "text", "text": "Hello, World!" }],
                "isError": false
            })),
            error: None,
            id: req.id,
        };
    }
    ```

## 🚀 Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/) (Cargo)

### Build
```bash
cargo build --release
```

### Run (Standalone)
You can test the server manually in your terminal:
```bash
# Send an initialize request
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-06-18"},"id":1}' | ./target/release/mcp-hello-world
```

## 🛠 Features
- **Standard Handshake**: Supports `initialize` and `notifications/initialized`.
- **Greeting Tool**: Provides a simple `hello` tool to verify functionality.
- **File Logging**: Uses `flexi_logger` to output logs to a file in the same directory as the executable, helping with background debugging.

## 📄 License
MIT
