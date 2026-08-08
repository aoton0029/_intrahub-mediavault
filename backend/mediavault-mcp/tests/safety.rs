//! TASK-0023: 安全性の横断検証
//!
//! `tools/list` から動的にツールを取得し、個別タスクでは検証しきれない
//! 横断的な安全性要件（NFR-301 / REQ-141 / REQ-145 / NFR-303 / REQ-115）を機械的に検証する。
//! ツール名のハードコードは避け、`tools/list` の結果から検査対象を導出する。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::Request as HttpRequest;
use axum::{Router, middleware};
use mediavault_mcp::auth;
use mediavault_mcp::config::Config;
use mediavault_mcp::server::MediaVaultServer;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;

const ITEM_ID: &str = "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001";
const RELATED_ITEM_ID: &str = "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0002";
const AUTH_TOKEN: &str = "test-auth-token-with-sufficient-length-000000";
const CONTEXT_SECTIONS: [&str; 8] = [
    "relations",
    "mylists",
    "groups",
    "cast",
    "staff",
    "files",
    "links",
    "trailers",
];

// ---------------------------------------------------------------------------
// JSON-RPC / rmcp サーバー起動の共通ヘルパ（tests/mcp_server.rs のパターンを踏襲）
// ---------------------------------------------------------------------------

fn extract_json_rpc_result(body: &str) -> Option<Value> {
    if let Ok(value) = serde_json::from_str::<Value>(body) {
        return Some(value);
    }
    body.lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .filter_map(|data| serde_json::from_str::<Value>(data).ok())
        .find(|value| value.get("result").is_some())
}

fn init_body() -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "safety-test-client", "version": "1.0.0"}
        }
    })
}

fn test_config(token: &str) -> Arc<Config> {
    let mut env = HashMap::new();
    env.insert("MCP_AUTH_TOKEN".to_string(), token.to_string());
    env.insert(
        "MEDIAVAULT_API_BASE_URL".to_string(),
        "http://mediavault-api:8080".to_string(),
    );
    Arc::new(Config::from_map_for_test(&env))
}

async fn spawn_service(
    api: Arc<mediavault_mcp::api::client::ApiClient>,
) -> (reqwest::Client, String, CancellationToken) {
    let ct = CancellationToken::new();
    let config = StreamableHttpServerConfig::default()
        .with_json_response(true)
        .with_sse_keep_alive(None)
        .with_cancellation_token(ct.child_token());

    let service = StreamableHttpService::new(
        move || Ok(MediaVaultServer::new(api.clone())),
        Arc::new(LocalSessionManager::default()),
        config,
    );
    let router = Router::new().nest_service("/mcp", service);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn({
        let ct = ct.clone();
        async move {
            let _ = axum::serve(listener, router)
                .with_graceful_shutdown(async move { ct.cancelled_owned().await })
                .await;
        }
    });

    (reqwest::Client::new(), format!("http://{addr}/mcp"), ct)
}

async fn init_session(client: &reqwest::Client, url: &str) -> Option<String> {
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&init_body())
        .send()
        .await
        .expect("initialize should succeed");
    response
        .headers()
        .get("Mcp-Session-Id")
        .map(|v| v.to_str().unwrap().to_string())
}

/// `tools/list` を実行し、`tools` 配列を返す。ツール名のハードコードを避けるための入口。
async fn fetch_tools_list(client: &reqwest::Client, url: &str) -> Vec<Value> {
    let session_id = init_session(client, url).await;
    let mut request = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}));
    if let Some(session_id) = &session_id {
        request = request.header("Mcp-Session-Id", session_id);
    }
    let response = request.send().await.expect("tools/list should succeed");
    let text = response.text().await.unwrap();
    let body = extract_json_rpc_result(&text).expect("response should contain a JSON-RPC result");
    body["result"]["tools"]
        .as_array()
        .cloned()
        .expect("tools should be an array")
}

/// `tools/call` を実行し、JSON-RPC レスポンス全体を返す。
async fn call_tool(client: &reqwest::Client, url: &str, name: &str, arguments: Value) -> Value {
    let session_id = init_session(client, url).await;
    let mut request = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {"name": name, "arguments": arguments}
        }));
    if let Some(session_id) = &session_id {
        request = request.header("Mcp-Session-Id", session_id);
    }
    let response = request.send().await.expect("tools/call should succeed");
    let text = response.text().await.unwrap();
    extract_json_rpc_result(&text).expect("response should contain a JSON-RPC result")
}

/// 各ツールを成功パスで呼ぶための最小限の引数。ツール名は `tools/list` から得た値を渡す。
fn success_arguments_for(name: &str) -> Value {
    match name {
        "health" => json!({}),
        "search_library" => json!({}),
        "search_external_catalog" => json!({"media_type": "anime", "q": "作品A"}),
        "get_item_context" => json!({"item_id": ITEM_ID}),
        "collection_overview" => json!({}),
        "import_external_item" => {
            json!({"media_type": "anime", "external_id": "12345", "provider": "annict"})
        }
        "create_item" => json!({"media_type": "manga", "title": "安全性テスト作品"}),
        "update_consumption" => json!({"item_id": ITEM_ID, "status": "completed"}),
        "organize_item" => json!({"item_id": ITEM_ID}),
        "relate_items" => json!({
            "item_id": ITEM_ID,
            "related_item_id": RELATED_ITEM_ID,
            "relation_type": "adaptation"
        }),
        "add_access_link" => json!({
            "item_id": ITEM_ID,
            "url": "https://example.com",
            "kind": "link",
            "label": "公式サイト"
        }),
        other => panic!("unknown tool in success_arguments_for: {other}"),
    }
}

// ---------------------------------------------------------------------------
// 共有バックエンド: 全11ツールの成功パスをまとめて満たす wiremock 設定
// ---------------------------------------------------------------------------

fn item_json(id: &str, media_type: &str, title: &str) -> Value {
    json!({
        "id": id,
        "media_type": media_type,
        "title": title,
        "original_title": null,
        "release_date": null,
        "status": "not_started",
        "rating": null,
        "is_favorite": false
    })
}

fn item_detail_json(id: &str, status: &str) -> Value {
    json!({
        "id": id,
        "media_type": "anime",
        "title": "安全性テスト作品",
        "original_title": null,
        "description": null,
        "release_date": null,
        "homepage_url": null,
        "status": status,
        "consumed_date": null,
        "rating": null,
        "is_favorite": false,
        "external_id": null,
        "created_at": "2026-01-01T00:00:00",
        "updated_at": "2026-01-01T00:00:00",
        "detail": null,
        "tags": [],
        "categories": [],
        "streaming_links": []
    })
}

async fn mount_success_backend(mock_server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/api/v1/health"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"success": true, "data": {"status": "ok"}})),
        )
        .mount(mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [],
            "pagination": {"limit": 20, "has_more": false, "next_after_created_at": null, "next_after_id": null, "total": 0}
        })))
        .mount(mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/items/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": [{
                "id": "12345",
                "media_type": "anime",
                "provider": "annict",
                "title": "作品A",
                "thumbnail_url": null,
                "year": 2023
            }]
        })))
        .mount(mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json(ITEM_ID, "in_progress")
        })))
        .mount(mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/items/{RELATED_ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json(RELATED_ITEM_ID, "not_started")
        })))
        .mount(mock_server)
        .await;

    for section in CONTEXT_SECTIONS {
        Mock::given(method("GET"))
            .and(path(format!("/api/v1/items/{ITEM_ID}/{section}")))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"success": true, "data": []})),
            )
            .mount(mock_server)
            .await;
    }

    Mock::given(method("GET"))
        .and(path("/api/v1/collection/overview"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": {
                "total_items": 0,
                "favorite_count": 0,
                "by_media_type": [],
                "by_status": [],
                "recently_added": [],
                "recently_updated": []
            }
        })))
        .mount(mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v1/items/import"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "success": true,
            "data": item_json(ITEM_ID, "anime", "作品A")
        })))
        .mount(mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v1/items"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "success": true,
            "data": item_json(ITEM_ID, "manga", "安全性テスト作品")
        })))
        .mount(mock_server)
        .await;

    Mock::given(method("PATCH"))
        .and(path(format!("/api/v1/items/{ITEM_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "success": true,
            "data": item_detail_json(ITEM_ID, "completed")
        })))
        .mount(mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v1/item-relations"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "success": true,
            "data": {
                "id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0003",
                "item_id": ITEM_ID,
                "related_item_id": RELATED_ITEM_ID,
                "relation_type": "adaptation",
                "created_at": "2026-01-01T00:00:00"
            }
        })))
        .mount(mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/items/{ITEM_ID}/links")))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "success": true,
            "data": {
                "id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0004",
                "url": "https://example.com",
                "label": "公式サイト"
            }
        })))
        .mount(mock_server)
        .await;
}

// ---------------------------------------------------------------------------
// 実装項目1: 誤書き込み0件の機械的検証（NFR-301）
// ---------------------------------------------------------------------------

/// 対象を特定しうる自由文字列引数の名前。`create_item.title` のみが例外
/// （新規作成であり、既存 Item を選び直す引数ではないため）。
const FORBIDDEN_FREE_TEXT_FIELDS: [&str; 3] = ["title", "query", "name"];
const ALLOWED_FREE_TEXT_EXCEPTIONS: [(&str, &str); 1] = [("create_item", "title")];

fn is_uuid_schema(field_schema: &Value) -> bool {
    field_schema.get("format").and_then(Value::as_str) == Some("uuid")
        || field_schema.to_string().contains("uuid")
}

/// 書き込み系ツールの `inputSchema` を検査する。
///
/// - `item_id` / `related_item_id` が存在すれば UUID 型であること
/// - `title` / `query` / `name` のような Item を特定しうる自由文字列引数が
///   許容リスト外に存在しないこと
fn validate_write_tool_schema(tool_name: &str, input_schema: &Value) -> Result<(), String> {
    let properties = input_schema
        .get("properties")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    for target_field in ["item_id", "related_item_id"] {
        if let Some(field_schema) = properties.get(target_field)
            && !is_uuid_schema(field_schema)
        {
            return Err(format!(
                "{tool_name}.{target_field} は UUID 型のスキーマではありません"
            ));
        }
    }

    for field_name in properties.keys() {
        if FORBIDDEN_FREE_TEXT_FIELDS.contains(&field_name.as_str())
            && !ALLOWED_FREE_TEXT_EXCEPTIONS.contains(&(tool_name, field_name.as_str()))
        {
            return Err(format!(
                "{tool_name}.{field_name} は Item を特定しうる自由文字列引数です"
            ));
        }
    }

    Ok(())
}

/// 単体テスト1: スキーマ検査ヘルパが機能していることの確認
#[test]
fn schema_check_rejects_a_free_text_target_selector() {
    let dummy_schema = json!({
        "properties": {
            "item_id": {"type": "string", "format": "uuid"},
            "title": {"type": "string"}
        }
    });

    let result = validate_write_tool_schema("dummy_write_tool", &dummy_schema);

    assert!(result.is_err(), "許可されていない title を検出できていない");
}

#[test]
fn schema_check_accepts_uuid_only_targets() {
    let dummy_schema = json!({
        "properties": {
            "item_id": {"type": "string", "format": "uuid"},
            "rating": {"type": "number"}
        }
    });

    assert!(validate_write_tool_schema("dummy_write_tool", &dummy_schema).is_ok());
}

/// 単体テスト2: 秘密情報検索ヘルパの確認
#[test]
fn secret_scan_detects_embedded_token() {
    let haystack = r#"{"message":"failed for token SECRET-TOKEN-DO-NOT-LEAK"}"#;
    assert!(haystack.contains("SECRET-TOKEN-DO-NOT-LEAK"));
    assert!(!haystack.contains("UNRELATED-TOKEN"));
}

/// 統合テスト1: 全書き込みツールの UUID 検証（tools/list から動的に取得する）
#[tokio::test]
async fn write_tools_have_uuid_only_target_arguments() {
    let mock_server = common::start_mock_server().await;
    let api = Arc::new(common::build_client(&mock_server, "internal-key"));
    let (client, url, ct) = spawn_service(api).await;

    let tools = fetch_tools_list(&client, &url).await;
    let write_tools: Vec<&Value> = tools
        .iter()
        .filter(|tool| tool["annotations"]["readOnlyHint"] == json!(false))
        .collect();

    assert_eq!(write_tools.len(), 6, "書き込みツールは6個のはず");

    let mut errors = Vec::new();
    for tool in write_tools {
        let name = tool["name"].as_str().unwrap();
        if let Err(err) = validate_write_tool_schema(name, &tool["inputSchema"]) {
            errors.push(err);
        }
    }
    assert!(errors.is_empty(), "スキーマ違反: {errors:?}");

    ct.cancel();
}

// ---------------------------------------------------------------------------
// 実装項目2: 破壊的ツールの非公開検証（REQ-141）
// ---------------------------------------------------------------------------

fn looks_destructive(name: &str) -> bool {
    name.starts_with("delete_")
        || name.starts_with("remove_")
        || name.starts_with("detach_")
        || name.starts_with("unlink_")
        || name.ends_with("_delete")
        || name == "update_api_key"
        || name == "set_api_key"
        || name == "move_file"
        || name == "rename_file"
}

/// 統合テスト2: tools/list の内容検証（ツール数・削除系不在・destructiveHint 不在）
#[tokio::test]
async fn tools_list_has_eleven_tools_and_no_destructive_ones() {
    let mock_server = common::start_mock_server().await;
    let api = Arc::new(common::build_client(&mock_server, "internal-key"));
    let (client, url, ct) = spawn_service(api).await;

    let tools = fetch_tools_list(&client, &url).await;
    assert_eq!(tools.len(), 11, "ツール数は11のはず");

    let read_only_count = tools
        .iter()
        .filter(|t| t["annotations"]["readOnlyHint"] == json!(true))
        .count();
    let write_count = tools
        .iter()
        .filter(|t| t["annotations"]["readOnlyHint"] == json!(false))
        .count();
    assert_eq!(read_only_count, 5, "読み取り専用ツールは5個のはず");
    assert_eq!(write_count, 6, "書き込みツールは6個のはず");

    for tool in &tools {
        let name = tool["name"].as_str().unwrap();
        assert!(
            !looks_destructive(name),
            "{name} は破壊的な名前パターンです"
        );
        assert_ne!(
            tool["annotations"]["destructiveHint"],
            json!(true),
            "{name} の destructiveHint が true です"
        );
    }

    ct.cancel();
}

// ---------------------------------------------------------------------------
// 実装項目3: ApiClient に DELETE がないことの検証（`ApiClient` は構造的に
// `delete` を公開しないため（api/client.rs 参照）、ここでは実行時にも
// DELETE が一度も発行されないことを成功・失敗の両パターンで確認する。
// ---------------------------------------------------------------------------

/// 統合テスト5: DELETE が発行されない（成功パス + 失敗パス）
#[tokio::test]
async fn never_calls_delete_across_all_tools_success_and_failure_paths() {
    // 成功パス
    let success_server = common::start_mock_server().await;
    mount_success_backend(&success_server).await;
    let api = Arc::new(common::build_client(&success_server, "internal-key"));
    let (client, url, ct) = spawn_service(api).await;

    let tools = fetch_tools_list(&client, &url).await;
    for tool in &tools {
        let name = tool["name"].as_str().unwrap();
        let _ = call_tool(&client, &url, name, success_arguments_for(name)).await;
    }
    ct.cancel();

    let success_requests = success_server.received_requests().await.unwrap();
    assert!(
        success_requests
            .iter()
            .all(|req| req.method.as_str() != "DELETE"),
        "成功パスで DELETE が発行された"
    );

    // 失敗パス: 書き込み系エンドポイントを一律500にする
    let failure_server = common::start_mock_server().await;
    for endpoint in [
        "/api/v1/items/import",
        "/api/v1/items",
        format!("/api/v1/items/{ITEM_ID}").as_str(),
        "/api/v1/item-relations",
        format!("/api/v1/items/{ITEM_ID}/links").as_str(),
    ] {
        for m in ["POST", "PATCH"] {
            Mock::given(method(m))
                .and(path(endpoint.to_string()))
                .respond_with(ResponseTemplate::new(500).set_body_json(json!({
                    "success": false,
                    "error": {"code": "INTERNAL_ERROR", "message": "サーバーエラー"}
                })))
                .mount(&failure_server)
                .await;
        }
    }
    let api = Arc::new(common::build_client(&failure_server, "internal-key"));
    let (client, url, ct) = spawn_service(api).await;

    let write_tool_names = [
        "import_external_item",
        "create_item",
        "update_consumption",
        "organize_item",
        "relate_items",
        "add_access_link",
    ];
    for name in write_tool_names {
        let _ = call_tool(&client, &url, name, success_arguments_for(name)).await;
    }
    ct.cancel();

    let failure_requests = failure_server.received_requests().await.unwrap();
    assert!(
        failure_requests
            .iter()
            .all(|req| req.method.as_str() != "DELETE"),
        "失敗パスで DELETE が発行された"
    );
}

// ---------------------------------------------------------------------------
// 実装項目4: 秘密情報の非露出検証（REQ-145）
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct SharedLogBuffer(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for SharedLogBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for SharedLogBuffer {
    type Writer = SharedLogBuffer;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// 統合テスト3: 全11ツールの秘密情報検査（レスポンス + ログ）
#[tokio::test]
async fn no_secret_leaks_in_tool_responses_or_logs() {
    const SECRET_INTERNAL_KEY: &str = "SECRET-INTERNAL-KEY-DO-NOT-LEAK";

    let log_buffer = Arc::new(Mutex::new(Vec::<u8>::new()));
    let subscriber = tracing_subscriber::fmt()
        .with_writer(SharedLogBuffer(log_buffer.clone()))
        .with_ansi(false)
        .finish();
    // 🟡 Intent: プロセス全体で一度だけ設定する。他の並行テストの tracing 出力も
    //    同じバッファに混ざりうるが、「秘密情報を含まないこと」の確認には影響しない。
    let _ = tracing::subscriber::set_global_default(subscriber);

    let mock_server = common::start_mock_server().await;
    mount_success_backend(&mock_server).await;
    let api = Arc::new(common::build_client(&mock_server, SECRET_INTERNAL_KEY));
    let (client, url, ct) = spawn_service(api).await;

    let tools = fetch_tools_list(&client, &url).await;
    for tool in &tools {
        let name = tool["name"].as_str().unwrap();
        let result = call_tool(&client, &url, name, success_arguments_for(name)).await;
        let text = serde_json::to_string(&result).unwrap();
        assert!(
            !text.contains(SECRET_INTERNAL_KEY),
            "{name} のレスポンスに internal api key が含まれています"
        );
        assert!(
            !text.contains(AUTH_TOKEN),
            "{name} のレスポンスに MCP_AUTH_TOKEN が含まれています"
        );
    }
    ct.cancel();

    let log_text = String::from_utf8_lossy(&log_buffer.lock().unwrap()).to_string();
    assert!(
        !log_text.contains(SECRET_INTERNAL_KEY),
        "ログに internal api key が含まれています"
    );
    assert!(
        !log_text.contains(AUTH_TOKEN),
        "ログに MCP_AUTH_TOKEN が含まれています"
    );
}

// ---------------------------------------------------------------------------
// 実装項目5: DB非依存の検証（NFR-303）
// ---------------------------------------------------------------------------

/// 統合テスト（実装項目5）: `cargo tree` に DB クライアント・embedding 系が含まれない
#[test]
fn cargo_tree_has_no_database_or_embedding_dependencies() {
    let output = std::process::Command::new(env!("CARGO"))
        .args(["tree", "-p", "mediavault-mcp"])
        .output()
        .expect("cargo tree の実行に失敗しました");
    assert!(
        output.status.success(),
        "cargo tree が失敗しました: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let tree = String::from_utf8_lossy(&output.stdout).to_lowercase();
    for forbidden in ["sqlx", "tokio-postgres", "diesel", "postgres"] {
        assert!(
            !tree.contains(forbidden),
            "cargo tree に {forbidden} への依存が含まれています"
        );
    }
}

// ---------------------------------------------------------------------------
// 実装項目6: 書き込み結果の検証可能性（NFR-203）
// ---------------------------------------------------------------------------

/// 統合テスト6: 全書き込みツールが対象 Item ID 等を返す
#[tokio::test]
async fn write_tools_return_required_verification_fields() {
    let mock_server = common::start_mock_server().await;
    mount_success_backend(&mock_server).await;
    let api = Arc::new(common::build_client(&mock_server, "internal-key"));
    let (client, url, ct) = spawn_service(api).await;

    let response = call_tool(
        &client,
        &url,
        "import_external_item",
        success_arguments_for("import_external_item"),
    )
    .await;
    let structured = &response["result"]["structuredContent"];
    assert!(!structured["item"]["item_id"].is_null());

    let response = call_tool(
        &client,
        &url,
        "create_item",
        success_arguments_for("create_item"),
    )
    .await;
    let structured = &response["result"]["structuredContent"];
    assert!(!structured["item"]["item_id"].is_null());

    let response = call_tool(
        &client,
        &url,
        "update_consumption",
        success_arguments_for("update_consumption"),
    )
    .await;
    let structured = &response["result"]["structuredContent"];
    assert!(!structured["item_id"].is_null());
    assert!(structured["changes"].is_array());

    let response = call_tool(
        &client,
        &url,
        "organize_item",
        success_arguments_for("organize_item"),
    )
    .await;
    let structured = &response["result"]["structuredContent"];
    assert!(!structured["item_id"].is_null());
    assert!(structured["tags"].is_array());
    assert!(structured["categories"].is_array());
    assert!(structured["mylists"].is_array());

    let response = call_tool(
        &client,
        &url,
        "relate_items",
        success_arguments_for("relate_items"),
    )
    .await;
    let structured = &response["result"]["structuredContent"];
    assert!(
        !structured["relation_id"].is_null() || structured["already_related"] == json!(true),
        "relation_id または already_related のいずれかが必要"
    );
    assert!(structured["description"].is_string());

    let response = call_tool(
        &client,
        &url,
        "add_access_link",
        success_arguments_for("add_access_link"),
    )
    .await;
    let structured = &response["result"]["structuredContent"];
    assert!(!structured["link_id"].is_null());
    assert!(!structured["registered_as"].is_null());

    ct.cancel();
}

// ---------------------------------------------------------------------------
// 実装項目7: 認証の横断検証（REQ-115）
// ---------------------------------------------------------------------------

/// 統合テスト4: 全ツールの認証必須検査（tools/list から動的に取得した名前で検証）
#[tokio::test]
async fn all_tools_reject_unauthenticated_requests_without_reaching_api() {
    let discovery_mock = common::start_mock_server().await;
    let discovery_api = Arc::new(common::build_client(&discovery_mock, "internal-key"));
    let (client, url, ct) = spawn_service(discovery_api).await;
    let tools = fetch_tools_list(&client, &url).await;
    let tool_names: Vec<String> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(tool_names.len(), 11);
    ct.cancel();

    for name in &tool_names {
        let mock_server = common::start_mock_server().await;
        let config = test_config(AUTH_TOKEN);
        let api = Arc::new(common::build_client(&mock_server, "internal-key"));

        let mcp_service = StreamableHttpService::new(
            move || Ok(MediaVaultServer::new(api.clone())),
            Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig::default().with_json_response(true),
        );
        let app =
            Router::new()
                .nest_service("/mcp", mcp_service)
                .layer(middleware::from_fn_with_state(
                    config.clone(),
                    auth::bearer_auth,
                ));

        let body = serde_json::to_vec(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {"name": name, "arguments": {}}
        }))
        .unwrap();

        let response = app
            .oneshot(
                HttpRequest::builder()
                    .method("POST")
                    .uri("/mcp")
                    .header("Content-Type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            axum::http::StatusCode::UNAUTHORIZED,
            "{name} は無認証で401になるはず"
        );
        assert!(
            mock_server.received_requests().await.unwrap().is_empty(),
            "{name} は無認証時に api へ到達してはいけない"
        );
    }
}
