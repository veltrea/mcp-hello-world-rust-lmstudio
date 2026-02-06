# MCP Hello World for LM Studio (Rust版)

このプロジェクトは、**LM Studio** で動作させることを最優先に設計された、最小構成の MCP (Model Context Protocol) サーバーの参照実装です。

## 💡 開発の背景
公式の MCP SDK は高度に抽象化されていますが、現在の LM Studio にはドキュメント化されていない特有の制約（ゴールデンルール）があり、標準的な実装では接続に失敗することがあります。

> [!IMPORTANT]
> これらのメモは、**2026年2月5日**に自作の MCP サーバーを LM Studio で動かそうとデバッグしていた際の試行錯誤の記録です。LM Studio は発展途上のツールであり、仕様が随時変更される可能性があるため、あくまで「現時点でのヒント」として参考にしてください。

このプロジェクトでは SDK を使わず、標準入出力を用いた生の JSON-RPC 通信を Rust で実装することで、現時点での LM Studio と正しく対話するための最小要件を明文化しています。

## 🏆 AI と作るための LM Studio 接続ガイド
**Google Antigravity** のような高度な AI アシスタントは、MCP の基本仕様をすでに深く理解しています。標準的な仕様に加えて、以下の 4 つの「外部ルール（ブリッジルール）」を指示として与えるだけで、小規模な MCP サーバーであれば LM Studio 互換のコードをほぼワンショットで生成できるはずです。

2026年2月5日のデバッグ中に試行錯誤した結果、現時点の LM Studio で安定して動作させるためのポイントをまとめました。

1.  **配置場所**: 実行ファイルを特定のディレクトリ構造に配置した際に認識が安定する傾向があります。
    -   推奨パス: `~/.lmstudio/extensions/plugins/mcp/<plugin-name>/`
    -   `mcp.json` からはこの場所にあるバイナリを指すように設定します。

2.  **プロトコルバージョン**: `initialize` ハンドシェイク時に、LM Studio が提示してきた `protocolVersion` (例: `2025-06-18`) をそのまま返すと互換性が向上します。
    ```rust
    // main.rs からの抜粋
    "initialize" => {
        let client_version = req.params.as_ref()
            .and_then(|p| p.get("protocolVersion"))
            .and_then(|v| v.as_str())
            .unwrap_or("2024-11-05");

        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(serde_json::json!({
                "protocolVersion": client_version, // クライアントの要望をそのまま返す
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "mcp-hello-world", "version": "0.1.0" }
            })),
            id: req.id,
        };
    }
    ```

3.  **JSON-RPC 形式 (error: null の排除)**: LM Studio のパース特性上、成功レスポンスに `error: null` を含めず、フィールドごと省略するのが無難です。
    ```rust
    #[derive(Debug, Serialize)]
    struct JsonRpcResponse {
        pub jsonrpc: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub result: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")] // None の場合はフィールドごと省略
        pub error: Option<Value>,
        pub id: Option<Value>,
    }
    ```

4.  **Tool 応答の構造**: ツール実行の結果は `content` 配列に包み、`isError` フラグを添えて返す構造が期待されています。
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

## 🚀 使い方

### 事前準備
- [Rust](https://www.rust-lang.org/ja) (Cargo) がインストールされていること。

### ビルド
```bash
cargo build --release
```

### 動作確認 (単体)
ターミナルから手動でリクエストを送ってテストできます：
```bash
# 初期化リクエストを送信
echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2025-06-18"},"id":1}' | ./target/release/mcp-hello-world
```

## 🛠 機能
- **標準ハンドシェイク**: `initialize` および `notifications/initialized` への対応。
- **挨拶ツール**: 動作確認用のシンプルな `hello` ツールを提供。
- **ファイルロギング**: `flexi_logger` を使用し、実行ファイルと同じディレクトリにログを出力します。これは LM Studio のバックグラウンドで動くサーバーのデバッグに非常に有効です。

## 📄 ライセンス
MIT
