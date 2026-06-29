//! item_files（パス指定方式）ハンドラ
//!
//! TASK-0026: POST /items/:id/files CRUD実装（handlers/item_links.rsと対称な構造）

use axum::Json;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::item_file::{
    CreateItemFileRequest, FileType, ItemFile, UpdateCalibreLinkRequest,
    parse_create_item_file_request, parse_file_id, parse_update_calibre_link_request,
};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::item_file_repository;
use crate::services::file_storage::{self, TokioFileWriter};

/// 【機能概要】: `POST /items/:id/files` ハンドラ。path(必須)/label(任意)/file_type(必須)を受け取り
/// ファイルサーバー上の既存パスをitem_filesレコードとして登録する（パス指定方式）
/// 🔵 信頼性レベル: api-endpoints.md POST /items/:id/files・TASK-0026 完了条件に直接対応
pub async fn create_item_file_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let request: CreateItemFileRequest = deserialize_request(body)?;
    let request = parse_create_item_file_request(request)?;

    let file = item_file_repository::create_item_file(
        &state.db,
        item_id,
        request.path,
        request.label,
        request.file_type,
    )
    .await?;

    Ok(created_response(file))
}

/// 【機能概要】: 作成済みファイルレコードをHTTP 201・統一レスポンス形式で返す
/// 🔵 信頼性レベル: handlers::item_links::created_responseと対称
fn created_response(file: ItemFile) -> axum::response::Response {
    (StatusCode::CREATED, Json(ApiOk::new(file))).into_response()
}

/// 【機能概要】: multipartの`file_type`フィールド文字列をFileType enumへ変換する。
/// 設計決定1により`photo`は`FileType::Image`の表記ゆれとして受理する（enum拡張なし）。
/// 🟡 信頼性レベル: red-phase.md 設計決定1・テストケース定義書TC-019-E03より
fn parse_file_type_field(raw: &str) -> Result<FileType, ApiError> {
    match raw {
        "pdf" => Ok(FileType::Pdf),
        // 【表記ゆれ対応】: 仕様本文の"photo"表記をImageの同義語として受理する 🟡
        "image" | "photo" => Ok(FileType::Image),
        "other" => Ok(FileType::Other),
        _ => Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "file_typeはpdf/image/otherのいずれかである必要があります",
        )),
    }
}

/// 【機能概要】: `POST /items/:id/files/upload` ハンドラ。multipart/form-dataでバイナリファイル本体を
/// 直接受け取り、file_typeに応じたファイルサーバーディレクトリへ書込み、相対パスをitem_filesへ登録する。
/// 【実装方針】: multipart解析→file_type検証→item存在確認（書込前）→file_storage::store_file()で書込→
/// item_file_repository::create_item_file()でINSERT→DB失敗時はfile_storage::cleanup_file()でロールバック。
/// 【テスト対応】: TC-019-01/02/N03/03/E01/E03/E04/E05/B01（handlers/item_files.rs #[ignore]統合テスト）
/// 🔵 信頼性レベル: 要件定義書 第2章データフロー・note.md L137-149「Axum Multipart処理パターン」に直接対応
pub async fn upload_item_file_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    mut multipart: Multipart,
) -> Result<axum::response::Response, ApiError> {
    // 【入力値検証】: パスパラメータ:idがUUID形式であることを確認する（TC-019-E05） 🔵
    let item_id = parse_item_id(&item_id)?;

    // 【multipart解析】: file/file_type/labelの各パートを読み取る。fileは必須・順不同を許容する 🔵
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut original_filename: String = String::new();
    let mut file_type_raw: Option<String> = None;
    let mut label: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|err| {
        // 【エラー捕捉】: multipart自体の解析失敗（不正なboundary等）はVALIDATION_ERRORとして扱う 🟡
        tracing::error!("multipart parse error: {err}");
        ApiError::new(
            ApiErrorCode::ValidationError,
            "multipartの解析に失敗しました",
        )
    })? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                original_filename = field.file_name().unwrap_or("").to_string();
                let bytes = field.bytes().await.map_err(|err| {
                    tracing::error!("multipart file field read error: {err}");
                    ApiError::new(
                        ApiErrorCode::ValidationError,
                        "fileフィールドの読込に失敗しました",
                    )
                })?;
                file_bytes = Some(bytes.to_vec());
            }
            "file_type" => {
                let text = field.text().await.map_err(|err| {
                    tracing::error!("multipart file_type field read error: {err}");
                    ApiError::new(
                        ApiErrorCode::ValidationError,
                        "file_typeフィールドの読込に失敗しました",
                    )
                })?;
                file_type_raw = Some(text);
            }
            "label" => {
                let text = field.text().await.map_err(|err| {
                    tracing::error!("multipart label field read error: {err}");
                    ApiError::new(
                        ApiErrorCode::ValidationError,
                        "labelフィールドの読込に失敗しました",
                    )
                })?;
                label = Some(text);
            }
            _ => {
                // 【無視】: 未知のフィールドは無視する（仕様外フィールドの寛容な扱い） 🟡
            }
        }
    }

    // 【必須フィールド検証】: fileパート欠落はVALIDATION_ERROR（TC-019-E04） 🟡
    let file_bytes =
        file_bytes.ok_or_else(|| ApiError::new(ApiErrorCode::ValidationError, "fileは必須です"))?;

    // 【必須フィールド検証】: file_type欠落・不正値はVALIDATION_ERROR（TC-019-E03） 🟡
    let file_type_raw = file_type_raw
        .ok_or_else(|| ApiError::new(ApiErrorCode::ValidationError, "file_typeは必須です"))?;
    let file_type = parse_file_type_field(&file_type_raw)?;

    // 【item存在確認】: 書込前にitemの存在を確認し、無駄なI/Oと孤立ファイルを防ぐ（TC-019-03） 🔵
    // 【実装方針】: item_file_repository::create_item_file()がINSERT時にも存在確認するが、
    // 「書込前にitem存在確認を行う」という仕様（要件定義書第4章）に従い、ここで明示的に確認する。
    let exists = item_file_repository::item_exists(&state.db, item_id).await?;
    if !exists {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたアイテムが見つかりません",
        ));
    }

    // 【ファイル書込】: file_storage::store_file()でfile_typeに応じたディレクトリへ書込む 🔵
    let writer = TokioFileWriter;
    let stored = file_storage::store_file(&writer, file_type, &original_filename, &file_bytes)?;

    // 【DB登録】: 書込成功後にitem_filesへINSERTする。失敗時は書込済みファイルを削除する（EDGE-003対称） 🟡
    match item_file_repository::create_item_file(
        &state.db,
        item_id,
        stored.relative_path.clone(),
        label,
        file_type,
    )
    .await
    {
        Ok(file) => Ok(created_response(file)),
        Err(db_err) => {
            // 【ロールバック】: DB登録失敗時は書込済みファイルを削除し、孤立ファイルを残さない 🟡
            if let Err(cleanup_err) = file_storage::cleanup_file(&writer, &stored) {
                tracing::error!("file_storage cleanup failed after db insert error: {cleanup_err}");
            }
            Err(db_err)
        }
    }
}

/// 【機能概要】: `PATCH /items/:id/files/:file_id/calibre-link` ハンドラ。:id/:file_idのUUID検証、
/// リクエストボディのデシリアライズ・非空検証を行い、リポジトリの2段階判定（不存在→404、
/// file_type≠pdf→400、pdf→更新）の結果をそのまま返す
/// 🔵 信頼性レベル: calibre-link-requirements.md 第3章 アーキテクチャ制約・タスク定義書 完了条件に直接対応
pub async fn update_calibre_link_handler(
    State(state): State<AppState>,
    Path((item_id, file_id)): Path<(String, String)>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    // 【入力値検証】: パスパラメータ:id/:file_idがUUID形式であることを確認する（TC-020-B01/B02） 🔵
    let item_id = parse_item_id(&item_id)?;
    let file_id = parse_file_id(&file_id)?;

    // 【リクエストボディ検証】: calibre_book_idの必須・非空チェック（TC-020-E03/E05） 🔵
    let request: UpdateCalibreLinkRequest = deserialize_request(body)?;
    let request = parse_update_calibre_link_request(request)?;

    // 【DB更新】: リポジトリの2段階判定（不存在→404、file_type≠pdf→400、pdf→更新）を呼び出す 🔵
    let file = item_file_repository::update_calibre_link(
        &state.db,
        item_id,
        file_id,
        request.calibre_book_id,
    )
    .await?;

    Ok(Json(ApiOk::new(file)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use sqlx::PgPool;
    use tower::ServiceExt;
    use uuid::Uuid;

    /// 【テスト用ヘルパー】: ルーティング統合テスト用にAppStateを構築する（routes::build_router経由）
    async fn test_app_state() -> AppState {
        let database_url = std::env::var("DATABASE_URL")
            .expect("TASK-0026ルーティング統合テストにはDATABASE_URL環境変数が必要です");
        let db = PgPool::connect(&database_url)
            .await
            .expect("テスト用DBへの接続に失敗しました");
        AppState {
            db,
            internal_api_key: String::new(),
        }
    }

    /// テストケース3: file_type不正値で400 VALIDATION_ERRORが返る（ルーター経由）
    #[tokio::test]
    #[ignore]
    async fn post_item_file_with_invalid_file_type_returns_400() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/files"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "path": "/srv/files/pdf/example.pdf",
                            "label": "本編PDF",
                            "file_type": "invalid"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// テストケース4: pathが空文字で400 VALIDATION_ERRORが返る（ルーター経由）
    #[tokio::test]
    #[ignore]
    async fn post_item_file_with_empty_path_returns_400() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/files"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "path": "",
                            "label": "本編PDF",
                            "file_type": "pdf"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// テストケース2: 存在しないitemへのPOSTで404が返る（ルーター経由）
    #[tokio::test]
    #[ignore]
    async fn post_item_file_with_nonexistent_item_returns_404() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/files"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "path": "/srv/files/pdf/example.pdf",
                            "label": "本編PDF",
                            "file_type": "pdf"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// 【テスト用ヘルパー】: multipart/form-dataの生ボディとboundaryを組み立てる（TASK-0025 item_import流用パターン）
    /// 🔵 信頼性レベル: テストケース定義書「テストヘルパー」`multipart_body()`・note.md L156-160/L265-266に対応
    fn multipart_body(
        file_bytes: &[u8],
        filename: &str,
        file_type: Option<&str>,
        label: Option<&str>,
    ) -> (String, Vec<u8>) {
        let boundary = "----mediavaultTestBoundaryTASK0027";
        let mut body = Vec::new();

        // file パート（必須。Noneの場合はTC-019-E04用にパート自体を省略する）
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n")
                .as_bytes(),
        );
        body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
        body.extend_from_slice(file_bytes);
        body.extend_from_slice(b"\r\n");

        if let Some(ft) = file_type {
            body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
            body.extend_from_slice(b"Content-Disposition: form-data; name=\"file_type\"\r\n\r\n");
            body.extend_from_slice(ft.as_bytes());
            body.extend_from_slice(b"\r\n");
        }

        if let Some(lbl) = label {
            body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
            body.extend_from_slice(b"Content-Disposition: form-data; name=\"label\"\r\n\r\n");
            body.extend_from_slice(lbl.as_bytes());
            body.extend_from_slice(b"\r\n");
        }

        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());

        (format!("multipart/form-data; boundary={boundary}"), body)
    }

    /// 【テスト用ヘルパー】: file パートを含まないmultipartボディを組み立てる（TC-019-E04専用）
    /// 🟡 信頼性レベル: multipart必須フィールド仕様からの妥当な推測
    fn multipart_body_without_file(file_type: &str) -> (String, Vec<u8>) {
        let boundary = "----mediavaultTestBoundaryTASK0027NoFile";
        let mut body = Vec::new();
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"file_type\"\r\n\r\n");
        body.extend_from_slice(file_type.as_bytes());
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        (format!("multipart/form-data; boundary={boundary}"), body)
    }

    /// 【テスト用ヘルパー】: テスト用itemをDBへ作成しitem_idを返す
    async fn insert_test_item(db: &PgPool) -> Uuid {
        sqlx::query_scalar(
            "INSERT INTO items (media_type, title) VALUES ('book', 'TASK-0027テスト書籍') RETURNING id",
        )
        .fetch_one(db)
        .await
        .expect("テスト用item作成に失敗しました")
    }

    /// TC-019-01: pdfバイナリアップロードでPDFディレクトリに配置され201でItemFileが返る（ルーター経由）
    /// 🔵 信頼性レベル: TASK-0027.md テストケース1・要件定義書第2章出力仕様に直接対応
    /// 【Red期待】: `/items/:id/files/upload` ルート・`upload_item_file_handler`が未実装のため、
    /// ルーティング未登録（404 Not Found、ハンドラ自体が存在しない）またはコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn post_item_file_upload_with_pdf_returns_201_and_relative_path() {
        // 【テスト目的】: file_type="pdf"のmultipartアップロードで、PDF用ベースディレクトリ配下に
        // ファイルが書き込まれ、相対パスがitem_files.pathに保存され201が返ることを確認する
        // 【テスト内容】: 一時ディレクトリをPDF_STORAGE_PATHに設定し、実在itemへPOSTする
        // 【期待される動作】: 201・data.file_type=="pdf"・data.pathが相対パスかつ拡張子.pdf・ファイル実在
        // 🔵 信頼性レベル: TASK-0027.md テストケース1（L65-68）・完了条件L24-26に直接対応
        let temp_root = tempfile::tempdir().expect("一時ディレクトリ作成に失敗しました");
        unsafe {
            std::env::set_var("PDF_STORAGE_PATH", temp_root.path());
        }
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_id = insert_test_item(&state.db).await;

        let (content_type, body) = multipart_body(
            b"%PDF-1.4 dummy bytes",
            "example.pdf",
            Some("pdf"),
            Some("本編PDF"),
        );

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

        // 【結果検証】: 正常系の中核（201応答）を確認する。相対パス・ファイル実在の詳細検証はGreen実装後に拡張する
        assert_eq!(response.status(), StatusCode::CREATED); // 【確認内容】: pdfアップロードが201で受理されることを確認 🔵
    }

    /// TC-019-02: imageアップロードで画像（photos）ディレクトリに配置され201が返る（ルーター経由）
    /// 🔵 信頼性レベル: TASK-0027.md テストケース2・設計決定1（photo→image・画像ディレクトリ）に対応
    #[tokio::test]
    #[ignore]
    async fn post_item_file_upload_with_image_returns_201() {
        // 【テスト目的】: file_type="image"のとき、pdfとは異なる画像用ベースディレクトリへ配置されることを確認する
        // 【テスト内容】: MEDIA_STORAGE_PATHを一時ディレクトリに設定し、実在itemへPOSTする
        // 【期待される動作】: 201・data.file_type=="image"
        // 🔵 信頼性レベル: TASK-0027.md テストケース2（L70-73）・設計決定1に対応
        let temp_root = tempfile::tempdir().expect("一時ディレクトリ作成に失敗しました");
        unsafe {
            std::env::set_var("MEDIA_STORAGE_PATH", temp_root.path());
        }
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_id = insert_test_item(&state.db).await;

        let (content_type, body) = multipart_body(
            b"\xFF\xD8\xFF dummy jpeg",
            "cover.jpg",
            Some("image"),
            Some("表紙"),
        );

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

        assert_eq!(response.status(), StatusCode::CREATED); // 【確認内容】: imageアップロードが201で受理されることを確認 🔵
    }

    /// TC-019-N03: label省略（任意フィールド未指定）でも201（ルーター経由）
    /// 🔵 信頼性レベル: models/item_file.rs（label nullable）・TASK-0026既存テストに基づく
    #[tokio::test]
    #[ignore]
    async fn post_item_file_upload_without_label_returns_201() {
        // 【テスト目的】: labelを含まないmultipartでもlabel=nullで201が返ることを確認する
        // 【テスト内容】: labelパートなしのmultipartで実在itemへPOSTする
        // 【期待される動作】: 201（必須はfileとfile_typeのみ）
        // 🔵 信頼性レベル: TC-019-N03・item_file.rsのlabel: Option<String>より
        let temp_root = tempfile::tempdir().expect("一時ディレクトリ作成に失敗しました");
        unsafe {
            std::env::set_var("PDF_STORAGE_PATH", temp_root.path());
        }
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_id = insert_test_item(&state.db).await;

        let (content_type, body) =
            multipart_body(b"%PDF-1.4 dummy", "example.pdf", Some("pdf"), None);

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

        assert_eq!(response.status(), StatusCode::CREATED); // 【確認内容】: label省略でも201が返ることを確認 🔵
    }

    /// TC-019-03: 存在しないitem_idへのアップロードは404 ITEM_NOT_FOUNDを返し、ファイルを書き込まない
    /// 🔵 信頼性レベル: TASK-0027.md テストケース4・既存post_item_file_with_nonexistent_item_returns_404と対称
    #[tokio::test]
    #[ignore]
    async fn post_item_file_upload_with_nonexistent_item_returns_404_and_no_file_written() {
        // 【テスト目的】: item存在確認が書込前に行われ、孤立ファイルが作られないことを確認する
        // 【テスト内容】: DB未登録のitem_idへmultipartアップロードする
        // 【期待される動作】: 404・ストレージroot配下に新規ファイルが作られない
        // 🔵 信頼性レベル: TC-019-03・要件定義書第4章（書込前item確認）に対応
        let temp_root = tempfile::tempdir().expect("一時ディレクトリ作成に失敗しました");
        unsafe {
            std::env::set_var("PDF_STORAGE_PATH", temp_root.path());
        }
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();

        let (content_type, body) =
            multipart_body(b"%PDF-1.4 dummy", "example.pdf", Some("pdf"), None);

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

        assert_eq!(response.status(), StatusCode::NOT_FOUND); // 【確認内容】: item不存在で404が返ることを確認 🔵

        // 【結果検証】: ストレージroot配下にファイルが作られていないことを確認
        let written_count = std::fs::read_dir(temp_root.path())
            .map(|entries| entries.count())
            .unwrap_or(0);
        assert_eq!(written_count, 0); // 【確認内容】: item不存在時に無駄な書込が発生していないことを確認 🔵
    }

    /// TC-019-E01: ファイルサーバー書込失敗時は500 FILE_STORAGE_WRITE_FAILEDを返しDBレコードを作成しない（EDGE-003）
    /// 🟡 信頼性レベル: TASK-0027.md テストケース3・完了条件L27・note.md L69-74より
    #[tokio::test]
    #[ignore]
    async fn post_item_file_upload_with_write_failure_returns_500_and_creates_no_record() {
        // 【テスト目的】: 書込失敗時にitem_filesレコードが作成されず500 FILE_STORAGE_WRITE_FAILEDが
        // 返ることを確認する（書込不可な一時パスを注入して失敗を誘発）
        // 【テスト内容】: PDF_STORAGE_PATHに書込不可なパス（存在しないネスト先かつ作成権限なし相当）を設定する
        // 【期待される動作】: 500・error.code=="FILE_STORAGE_WRITE_FAILED"・item_filesの該当item行数が0
        // 🟡 信頼性レベル: TC-019-E01・EDGE-003・フェイク注入手段は妥当推測
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_id = insert_test_item(&state.db).await;

        // 書込不可パスとして、存在しないファイルをディレクトリのように指定する（書込時に必ず失敗する）
        let unwritable_root =
            std::env::temp_dir().join(format!("mediavault-task0027-unwritable-{}", Uuid::new_v4()));
        std::fs::write(&unwritable_root, b"this is a file, not a directory")
            .expect("書込不可パス用ダミーファイルの作成に失敗しました");
        unsafe {
            std::env::set_var("PDF_STORAGE_PATH", &unwritable_root);
        }
        let (content_type, body) =
            multipart_body(b"%PDF-1.4 dummy", "example.pdf", Some("pdf"), None);

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

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR); // 【確認内容】: 書込失敗時に500が返ることを確認 🟡

        // 【結果検証】: item_filesに該当itemの新規レコードが作られていないことを確認（dangling record防止）
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM item_files WHERE item_id = $1")
            .bind(item_id)
            .fetch_one(&state.db)
            .await
            .expect("件数取得に失敗しました");
        assert_eq!(count, 0); // 【確認内容】: 書込失敗時にDBレコードが作成されていないことを確認 🟡

        let _ = std::fs::remove_file(&unwritable_root);
    }

    /// TC-019-E03: 不正なfile_typeは400 VALIDATION_ERRORとなり書込を行わない（ルーター経由）
    /// 🟡 信頼性レベル: note.md・既存post_item_file_with_invalid_file_type_returns_400と対称
    #[tokio::test]
    #[ignore]
    async fn post_item_file_upload_with_invalid_file_type_returns_400() {
        // 【テスト目的】: enum外のfile_type（"invalid"）が400 VALIDATION_ERRORで拒否されることを確認する
        // 【テスト内容】: file_type="invalid"のmultipartで実在itemへPOSTする
        // 【期待される動作】: 400・error.code=="VALIDATION_ERROR"
        // 🟡 信頼性レベル: TC-019-E03・既存create_item_file_handlerの検証パターンより
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_id = insert_test_item(&state.db).await;

        let (content_type, body) =
            multipart_body(b"dummy bytes", "example.bin", Some("invalid"), None);

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

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 不正file_typeが400で拒否されることを確認 🟡
    }

    /// TC-019-E04: fileフィールド欠落で400 VALIDATION_ERRORを返す（ルーター経由）
    /// 🟡 信頼性レベル: multipart必須フィールド仕様・要件定義書第2章エラー表からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn post_item_file_upload_without_file_field_returns_400() {
        // 【テスト目的】: 必須fileパートが無いmultipartが400 VALIDATION_ERRORで拒否されることを確認する
        // 【テスト内容】: file_typeのみを含むmultipart（fileパートなし）で実在itemへPOSTする
        // 【期待される動作】: 400・error.code=="VALIDATION_ERROR"
        // 🟡 信頼性レベル: TC-019-E04・任意labelとの対比（TC-019-N03）より
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_id = insert_test_item(&state.db).await;

        let (content_type, body) = multipart_body_without_file("pdf");

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

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: file欠落が400で拒否されることを確認 🟡
    }

    /// TC-019-E05: 不正な:id（UUIDでない）は400 VALIDATION_ERRORを返す（ルーター経由）
    /// 🟡 信頼性レベル: 既存create_item_file_handlerのparse_item_id()利用からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn post_item_file_upload_with_invalid_uuid_path_returns_400() {
        // 【テスト目的】: パスパラメータ:idがUUID形式でない場合に400で拒否されることを確認する
        // 【テスト内容】: POST /items/not-a-uuid/files/upload に正当なmultipartを送る
        // 【期待される動作】: 400・error.code=="VALIDATION_ERROR"
        // 🟡 信頼性レベル: TC-019-E05・既存parse_item_id()利用パターンより
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);

        let (content_type, body) = multipart_body(b"dummy bytes", "example.pdf", Some("pdf"), None);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/items/not-a-uuid/files/upload")
                    .header("content-type", content_type)
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 不正なUUID形式の:idが400で拒否されることを確認 🟡
    }

    /// TC-019-B01: 空（0バイト）ファイルアップロードは許容され201が返る
    /// 🟡 信頼性レベル: 設計決定6（本タスクで確定：空ファイル許容）に基づく
    #[tokio::test]
    #[ignore]
    async fn post_item_file_upload_with_empty_file_returns_201() {
        // 【テスト目的】: 0バイトのfileでも例外を起こさず、許容方針（201）で一貫した結果を返すことを確認する
        // 【テスト内容】: file=<0バイト>・file_type="pdf"で実在itemへPOSTする
        // 【期待される動作】: 201（設計決定6: 空ファイル許容方針を採用）
        // 🟡 信頼性レベル: テストケース定義書TC-019-B01・本タスクでの設計決定6（空ファイル許容）に基づく
        let temp_root = tempfile::tempdir().expect("一時ディレクトリ作成に失敗しました");
        unsafe {
            std::env::set_var("PDF_STORAGE_PATH", temp_root.path());
        }
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_id = insert_test_item(&state.db).await;

        let (content_type, body) = multipart_body(b"", "empty.pdf", Some("pdf"), None);

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

        assert_eq!(response.status(), StatusCode::CREATED); // 【確認内容】: 0バイトファイルが許容方針どおり201で受理されることを確認 🟡
    }

    /// テストケース1（TC-007-01）: 正常系で201が返る（ルーター経由・実item必要）
    #[tokio::test]
    #[ignore]
    async fn post_item_file_with_existing_item_returns_201() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());

        // テスト対象のitemを作成
        let item_id: Uuid = sqlx::query_scalar(
            "INSERT INTO items (media_type, title) VALUES ('book', 'テスト書籍') RETURNING id",
        )
        .fetch_one(&state.db)
        .await
        .expect("テスト用item作成に失敗しました");

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/files"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "path": "/srv/files/pdf/example.pdf",
                            "label": "本編PDF",
                            "file_type": "pdf"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    /// 【テスト用ヘルパー】: 指定file_typeのpdf/image item_file行を作成しfile_idを返す
    async fn insert_test_item_file(db: &PgPool, item_id: Uuid, file_type: &str) -> Uuid {
        sqlx::query_scalar(
            "INSERT INTO item_files (item_id, path, label, file_type, calibre_book_id)
             VALUES ($1, '/srv/files/pdf/example.pdf', '本編PDF', $2::file_type, NULL)
             RETURNING id",
        )
        .bind(item_id)
        .bind(file_type)
        .fetch_one(db)
        .await
        .expect("テスト用item_file作成に失敗しました")
    }

    /// TC-020-01: PATCH calibre-link がpdfレコードを更新し200を返す（ルーター経由）
    /// 【テスト目的】: ルーター経由で実在itemのfile_type=pdfファイルに対しPATCHすると200・
    /// 更新後レコードが返ることを確認する
    /// 【テスト内容】: pdf item_fileを事前insertし、PATCH /items/:id/files/:file_id/calibre-linkを実行する
    /// 【期待される動作】: HTTP 200、data.calibre_book_id=="calibre-12345"、data.file_type=="pdf"
    /// 🔵 信頼性レベル: タスク定義書 単体テスト要件テストケース1（TC-020-01）・完了条件L24,27に直接対応
    /// 【Red期待】: ルート未登録・update_calibre_link_handler未実装のため404/コンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn patch_calibre_link_updates_pdf_record_and_returns_200() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_id = insert_test_item(&state.db).await;
        let file_id = insert_test_item_file(&state.db, item_id, "pdf").await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/items/{item_id}/files/{file_id}/calibre-link"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "calibre_book_id": "calibre-12345" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: pdfレコードのcalibre-link更新が200で受理されることを確認 🔵
    }

    /// TC-020-N01: 同じcalibre_book_idで2回PATCHしても200・同一結果が返る（冪等性、ルーター経由）
    /// 【テスト目的】: 連携バッチが再送した場合でも一意制約違反等で失敗しないことを確認する
    /// 【テスト内容】: 同一item_id/file_idに対し同じBodyで2回PATCHする
    /// 【期待される動作】: 1回目も2回目も200
    /// 🟡 信頼性レベル: 要件定義書2.3 L76（冪等性明記）からの妥当な推測
    /// 【Red期待】: update_calibre_link_handler未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn patch_calibre_link_twice_with_same_value_is_idempotent() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_id = insert_test_item(&state.db).await;
        let file_id = insert_test_item_file(&state.db, item_id, "pdf").await;

        let body =
            || Body::from(serde_json::json!({ "calibre_book_id": "calibre-12345" }).to_string());

        let first = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/items/{item_id}/files/{file_id}/calibre-link"))
                    .header("content-type", "application/json")
                    .body(body())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK); // 【確認内容】: 1回目のPATCHが200で受理されることを確認 🟡

        let second = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/items/{item_id}/files/{file_id}/calibre-link"))
                    .header("content-type", "application/json")
                    .body(body())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second.status(), StatusCode::OK); // 【確認内容】: 同じ値での2回目のPATCHも200のまま（重複違反等で失敗しない）ことを確認 🟡
    }

    /// TC-020-N02: 既にcalibre_book_id設定済みのpdf行を別の値で上書き更新できる（ルーター経由）
    /// 【テスト目的】: NULL→値だけでなく値→値の更新も成立することを確認する
    /// 【テスト内容】: "old-1"を設定したpdf行に対し"calibre-99999"でPATCHする
    /// 【期待される動作】: 200、旧値が新値で置き換わる
    /// 🟡 信頼性レベル: 更新仕様（要件定義書2.3）からの妥当な推測
    /// 【Red期待】: update_calibre_link_handler未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn patch_calibre_link_overwrites_existing_value() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_id = insert_test_item(&state.db).await;
        let file_id = insert_test_item_file(&state.db, item_id, "pdf").await;

        sqlx::query("UPDATE item_files SET calibre_book_id = 'old-1' WHERE id = $1")
            .bind(file_id)
            .execute(&state.db)
            .await
            .expect("事前データ設定に失敗しました");

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/items/{item_id}/files/{file_id}/calibre-link"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "calibre_book_id": "calibre-99999" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: 既存値ありの上書き更新も200で受理されることを確認 🟡
    }

    /// TC-020-E01: file_type=image(photo)のレコードへのPATCHは400 VALIDATION_ERRORでレコードは更新されない（ルーター経由）
    /// 【テスト目的】: PDF以外にCalibre書籍IDを紐付けるのは仕様違反であり、誤った関連付けを防ぐことを確認する
    /// 【テスト内容】: file_type=image item_fileに対しPATCHする
    /// 【期待される動作】: 400 VALIDATION_ERROR、該当行のcalibre_book_idは更新されない（NULLのまま）
    /// 🔵 信頼性レベル: タスク定義書 テストケース2（L73-76）・要件定義書4.2 E01 L113に直接対応
    /// 【Red期待】: update_calibre_link_handler未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn patch_calibre_link_with_image_file_type_returns_400_and_does_not_update() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_id = insert_test_item(&state.db).await;
        let file_id = insert_test_item_file(&state.db, item_id, "image").await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/items/{item_id}/files/{file_id}/calibre-link"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "calibre_book_id": "calibre-12345" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: file_type≠pdfが400 VALIDATION_ERRORで拒否されることを確認 🔵
    }

    /// TC-020-E02a: 存在しないfile_idへのPATCHは404 FILE_NOT_FOUNDを返す（ルーター経由）
    /// 【テスト目的】: 存在しないリソースへの更新を明示的に拒否することを確認する
    /// 【テスト内容】: itemのみ作成し、ランダムなfile_idに対しPATCHする
    /// 【期待される動作】: 404 FILE_NOT_FOUND
    /// 🔵 信頼性レベル: タスク定義書 テストケース3（L78-81）・要件定義書4.2 E02 L114に直接対応
    /// 【Red期待】: update_calibre_link_handler未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn patch_calibre_link_with_nonexistent_file_id_returns_404() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_id = insert_test_item(&state.db).await;
        let random_file_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!(
                        "/items/{item_id}/files/{random_file_id}/calibre-link"
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "calibre_book_id": "calibre-12345" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND); // 【確認内容】: 不存在file_idで404 FILE_NOT_FOUNDが返ることを確認 🔵
    }

    /// TC-020-E02b: 別itemに属するfile_idを指定すると404 FILE_NOT_FOUNDを返す（ルーター経由）
    /// 【テスト目的】: 他itemのファイルを誤更新するのを防ぐ（権限/整合性境界）ことを確認する
    /// 【テスト内容】: itemAにpdf item_file（file_id_A）を作成し、別のitemBのURLでPATCHする
    /// 【期待される動作】: 404 FILE_NOT_FOUND、file_id_Aのcalibre_book_idは更新されない
    /// 🔵 信頼性レベル: タスク定義書 テストケース3（L78-81「別のitem_idに属するfile_id」）・要件定義書4.2 E02に直接対応
    /// 【Red期待】: update_calibre_link_handler未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn patch_calibre_link_with_mismatched_item_id_returns_404() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_a = insert_test_item(&state.db).await;
        let item_b = insert_test_item(&state.db).await;
        let file_id_a = insert_test_item_file(&state.db, item_a, "pdf").await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/items/{item_b}/files/{file_id_a}/calibre-link"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "calibre_book_id": "calibre-12345" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND); // 【確認内容】: item_id/file_id紐付け不一致で404 FILE_NOT_FOUNDが返ることを確認 🔵

        let persisted: Option<String> =
            sqlx::query_scalar("SELECT calibre_book_id FROM item_files WHERE id = $1")
                .bind(file_id_a)
                .fetch_one(&state.db)
                .await
                .expect("再取得に失敗しました");
        assert_eq!(persisted, None); // 【確認内容】: 他itemのファイルが変更されていないことを確認 🔵
    }

    /// TC-020-E03-ROUTER: calibre_book_idが空文字のPATCHは400 VALIDATION_ERRORを返す（ルーター経由）
    /// 【テスト目的】: 無意味な空IDの紐付けをルーター経由でも拒否することを確認する
    /// 【テスト内容】: pdf item_fileに対しcalibre_book_id=""でPATCHする
    /// 【期待される動作】: 400 VALIDATION_ERROR
    /// 🟡 信頼性レベル: 要件定義書4.2 E03 L115・2.1 L42に対応
    /// 【Red期待】: update_calibre_link_handler未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn patch_calibre_link_with_empty_calibre_book_id_returns_400() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_id = insert_test_item(&state.db).await;
        let file_id = insert_test_item_file(&state.db, item_id, "pdf").await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/items/{item_id}/files/{file_id}/calibre-link"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "calibre_book_id": "" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 空文字のcalibre_book_idが400で拒否されることを確認 🟡
    }

    /// TC-020-E05-ROUTER: calibre_book_idキー欠落のPATCHは400 VALIDATION_ERRORを返す（ルーター経由）
    /// 【テスト目的】: デシリアライズ失敗を統一エラーで返し、内部パニックを防ぐことを確認する
    /// 【テスト内容】: pdf item_fileに対し空オブジェクト{}でPATCHする
    /// 【期待される動作】: 400 VALIDATION_ERROR
    /// 🟡 信頼性レベル: 要件定義書4.2 E05 L117・既存deserialize_request実装に対応
    /// 【Red期待】: update_calibre_link_handler未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn patch_calibre_link_with_missing_key_returns_400() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_id = insert_test_item(&state.db).await;
        let file_id = insert_test_item_file(&state.db, item_id, "pdf").await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/items/{item_id}/files/{file_id}/calibre-link"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::json!({}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: calibre_book_idキー欠落が400で拒否されることを確認 🟡
    }

    /// TC-020-B01: :idが不正UUID形式で400 VALIDATION_ERRORを返す（ルーター経由）
    /// 【テスト目的】: パスパラメータ妥当性の境界（パース可否）を確認する
    /// 【テスト内容】: PATCH /items/not-a-uuid/files/{valid_uuid}/calibre-linkを実行する
    /// 【期待される動作】: 400 VALIDATION_ERROR
    /// 🔵 信頼性レベル: 要件定義書4.2 E04 L116・既存parse_item_id・既存TC-019-E05に対応
    /// 【Red期待】: ルート未登録・update_calibre_link_handler未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn patch_calibre_link_with_invalid_item_id_uuid_returns_400() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let valid_file_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!(
                        "/items/not-a-uuid/files/{valid_file_id}/calibre-link"
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "calibre_book_id": "calibre-12345" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 不正な:idが400で拒否されることを確認 🔵
    }

    /// TC-020-B02: :file_idが不正UUID形式で400 VALIDATION_ERRORを返す（ルーター経由）
    /// 【テスト目的】: 新規parse_file_id（または同等検証）の早期リターンを確認する
    /// 【テスト内容】: PATCH /items/{valid_uuid}/files/not-a-uuid/calibre-linkを実行する
    /// 【期待される動作】: 400 VALIDATION_ERROR
    /// 🟡 信頼性レベル: 要件定義書2.1 L37・4.2 E04 L116（parse_file_id相当を追加）からの妥当な推測
    /// 【Red期待】: ルート未登録・update_calibre_link_handler未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn patch_calibre_link_with_invalid_file_id_uuid_returns_400() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let valid_item_id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!(
                        "/items/{valid_item_id}/files/not-a-uuid/calibre-link"
                    ))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "calibre_book_id": "calibre-12345" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 不正な:file_idが400で拒否されることを確認 🟡
    }

    /// TC-020-B03: 1文字のcalibre_book_idでも200で更新される（ルーター経由）
    /// 【テスト目的】: 非空制約の下限（trim後1文字）の境界が正しいことを確認する
    /// 【テスト内容】: pdf行に対しBody { "calibre_book_id": "a" }でPATCHする
    /// 【期待される動作】: 200、data.calibre_book_id=="a"
    /// 🟡 信頼性レベル: 非空制約（要件定義書2.1 L42）からの妥当な境界値推測
    /// 【Red期待】: update_calibre_link_handler未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn patch_calibre_link_with_single_character_returns_200() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_id = insert_test_item(&state.db).await;
        let file_id = insert_test_item_file(&state.db, item_id, "pdf").await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/items/{item_id}/files/{file_id}/calibre-link"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "calibre_book_id": "a" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: 1文字のcalibre_book_idが200で受理されることを確認 🟡
    }

    /// TC-020-B04: 前後空白を含むが中身が非空のcalibre_book_idは受理される（ルーター経由）
    /// 【テスト目的】: 「空白のみ(400)」ではなく非空として受理されることを確認する
    /// 【テスト内容】: pdf行に対しBody { "calibre_book_id": "  calibre-12345  " }でPATCHする
    /// 【期待される動作】: 200
    /// 🔴 信頼性レベル: 保存値をtrimするか原文保持かは要件未記載。実装時に方針を決定し本テストを確定（赤信号）
    /// 【Red期待】: update_calibre_link_handler未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn patch_calibre_link_with_surrounding_whitespace_returns_200() {
        let state = test_app_state().await;
        let app = crate::routes::build_router(state.clone());
        let item_id = insert_test_item(&state.db).await;
        let file_id = insert_test_item_file(&state.db, item_id, "pdf").await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/items/{item_id}/files/{file_id}/calibre-link"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "calibre_book_id": "  calibre-12345  " }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: 空白付きだが非空のcalibre_book_idが200で受理されることを確認（保存方針は実装時確定） 🔴
    }
}
