//! TASK-0032: 主要フロー統合テスト — ファイル登録フロー（IT-004, IT-005）
//!
//! パス指定方式（POST /items/:id/files）とアップロード方式（POST /items/:id/files/upload）の
//! 両方を、実DB + `tower::ServiceExt::oneshot`によるルーター経由E2Eで検証する。
//! アップロード方式は`tempfile`crateで作成した一時ディレクトリを`PDF_STORAGE_PATH`へ
//! 設定することで、実ファイルシステムを汚さずに検証する（file_storage::resolve_base_dirが
//! `PDF_STORAGE_PATH`/`MEDIA_STORAGE_PATH`環境変数を読む実装に基づく）。
//! 🔵 信頼性レベル: main-flow-integration-test-testcases.md IT-004/IT-005に直接対応

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use common::{build_full_router, test_app_state};

/// 【テスト用ヘルパー】: レスポンスボディをJSON Valueへ変換する
async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("レスポンスボディの読み取りに失敗しました");
    serde_json::from_slice(&bytes).expect("レスポンスボディのJSONパースに失敗しました")
}

/// 【テスト用ヘルパー】: テスト用アイテムを作成しitem_idを返す
async fn create_test_item(app: &axum::Router, title: &str) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/items")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"media_type":"anime","title":title}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let json = body_json(response).await;
    json["data"]["id"].as_str().unwrap().to_string()
}

/// 【テスト用ヘルパー】: multipart/form-dataの生ボディとboundaryを組み立てる
/// （handlers/item_files.rs既存テストのmultipart_body()パターンを踏襲）
fn multipart_body(file_bytes: &[u8], filename: &str, label: Option<&str>) -> (String, Vec<u8>) {
    let boundary = "----mediavaultTask0032Boundary";
    let mut body = Vec::new();

    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n")
            .as_bytes(),
    );
    body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
    body.extend_from_slice(file_bytes);
    body.extend_from_slice(b"\r\n");

    if let Some(lbl) = label {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"label\"\r\n\r\n");
        body.extend_from_slice(lbl.as_bytes());
        body.extend_from_slice(b"\r\n");
    }

    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

    (format!("multipart/form-data; boundary={boundary}"), body)
}

/// IT-004: ファイル登録（パス指定方式）
/// 【テスト目的】: `POST /items/:id/files`に既存パスを指定してitem_filesが作成されることを確認する
/// 【テスト内容】: `{"path":"/data/test/sample.pdf","file_type":"pdf"}`をPOSTする
/// 【期待される動作】: 201、レスポンスのpathが入力と一致、DB直接SELECTでも同値
/// 🔵 信頼性レベル: main-flow-integration-test-testcases.md IT-004（acceptance-criteria.md TC-007-01）
#[tokio::test]
#[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
async fn it_004_register_file_by_path_persists_exact_path() {
    let state = test_app_state().await;
    let app = build_full_router(state.clone());
    let item_id = create_test_item(&app, "IT-004パス指定テスト").await;

    // 【実際の処理実行】: POST /items/:id/files へパス指定でリクエストする 🔵
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/items/{item_id}/files"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "path": "/data/test/sample.pdf"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // 【結果検証】: 201・レスポンスのpathが入力と一致することを確認する 🔵
    assert_eq!(response.status(), StatusCode::CREATED); // 【確認内容】: パス指定方式のファイル登録が201で成功することを確認 🔵
    let json = body_json(response).await;
    assert_eq!(json["data"]["path"], "/data/test/sample.pdf"); // 【確認内容】: レスポンスのpathが入力どおりであることを確認 🔵
    let file_id: uuid::Uuid = json["data"]["id"].as_str().unwrap().parse().unwrap();

    // 【結果検証】: DB直接SELECTでも同値であることを確認する 🔵
    let persisted_path: String = sqlx::query_scalar("SELECT path FROM item_files WHERE id = $1")
        .bind(file_id)
        .fetch_one(&state.db)
        .await
        .expect("item_filesの再取得に失敗しました");
    assert_eq!(persisted_path, "/data/test/sample.pdf"); // 【確認内容】: DB上のpathが入力どおりであることを確認 🔵
}

/// IT-005: ファイル登録（バイナリアップロード方式）
/// 【テスト目的】: `POST /items/:id/files/upload`にmultipartでバイナリを送信し、配置後の相対パスが
/// DBに保存されることを確認する
/// 【テスト内容】: テスト用一時ディレクトリ（`tempfile`crate）をPDF_STORAGE_PATHに設定し、
/// ダミーPDFバイナリをmultipartでアップロードする
/// 【期待される動作】: 201、レスポンスのpathが配置後の相対パス形式、実ファイルが一時ディレクトリ上に存在する
/// 🔵 信頼性レベル: main-flow-integration-test-testcases.md IT-005（acceptance-criteria.md TC-019-01）
#[tokio::test]
#[ignore]
async fn it_005_upload_file_via_multipart_persists_relative_path_under_temp_dir() {
    // 【テストデータ準備】: tempfile crateで一時ディレクトリを作成し、PDF_STORAGE_PATHへ設定する 🔵
    let temp_root = tempfile::tempdir().expect("一時ディレクトリの作成に失敗しました");
    unsafe {
        std::env::set_var("PDF_STORAGE_PATH", temp_root.path());
    }

    let state = test_app_state().await;
    let app = build_full_router(state.clone());
    let item_id = create_test_item(&app, "IT-005アップロードテスト").await;

    let (content_type, body) = multipart_body(
        b"%PDF-1.4 IT-005 dummy bytes",
        "sample.pdf",
        Some("IT-005本編PDF"),
    );

    // 【実際の処理実行】: POST /items/:id/files/upload へmultipartでアップロードする 🔵
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/items/{item_id}/files/upload"))
                .header("content-type", content_type)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    // 【結果検証】: 201・レスポンスのpathが相対パス形式であることを確認する 🔵
    assert_eq!(response.status(), StatusCode::CREATED); // 【確認内容】: アップロードが201で成功することを確認 🔵
    let json = body_json(response).await;
    let relative_path = json["data"]["path"].as_str().unwrap().to_string();
    assert!(!relative_path.starts_with('/')); // 【確認内容】: 保存されたpathが絶対パスでなく相対パス形式であることを確認 🔵

    // 【結果検証】: 一時ディレクトリ配下に実ファイルが存在することを確認する 🔵
    let absolute_path = temp_root.path().join(&relative_path);
    assert!(absolute_path.exists()); // 【確認内容】: 配置後の実ファイルが一時ディレクトリ上に存在することを確認 🔵

    // 【結果検証】: DB上のpathもレスポンスと同値であることを確認する 🔵
    let file_id: uuid::Uuid = json["data"]["id"].as_str().unwrap().parse().unwrap();
    let persisted_path: String = sqlx::query_scalar("SELECT path FROM item_files WHERE id = $1")
        .bind(file_id)
        .fetch_one(&state.db)
        .await
        .expect("item_filesの再取得に失敗しました");
    assert_eq!(persisted_path, relative_path); // 【確認内容】: DB上のpathがレスポンスのpathと一致することを確認 🔵
}
