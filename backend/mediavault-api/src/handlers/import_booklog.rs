//! ブクログCSVインポートハンドラ
//!
//! 【機能概要】: `POST /import/booklog`ハンドラ。multipart/form-dataでCSVファイルを受け取り、
//! `import::booklog_csv`でパース・バリデーションし、OpenBDのCコードで小説/学術書・専門書を
//! 再分類したうえで、ジョブをバックグラウンド起動して`job_id`を返す。
//! 楽天ブックスAPI（ISBNごとの書誌エンリッチメント: タイトル・説明・カバー画像・発売日等、
//! media_type以外の情報）は1リクエスト/ISBN・約1秒間隔でしか呼べず、実サンプル規模（数千ISBN）
//! では完走に数十分かかるため、リクエストをブロックせずバックグラウンドジョブとして処理する。
//! クライアントは`GET /import/booklog/jobs/:job_id`をポーリングして進捗と最終`ImportSummary`を取得する。
//!
//!   1. `parse_booklog_csv`でパース（タイプ=マンガ→Manga確定、タイプ=雑誌→Novel確定、
//!      それ以外はISBNがあれば暫定NovelかつOpenBD再分類対象としてマーク）
//!   2. 再分類対象のISBNをOpenBDへチャンク単位（`OPENBD_CHUNK_SIZE`件ずつ）で問い合わせ、
//!      Cコードの内容分類（9=文芸→Novel、それ以外→AcademicBook）で`media_type`を上書きする。
//!      チャンク間は`OPENBD_REQUEST_INTERVAL`だけ間隔を空け、OpenBD呼び出し失敗時は
//!      当該チャンクをNovelのままフォールバックして処理を継続する。
//!   3. ここまでを同期的に終えたらジョブを登録し、`job_id`を返す。
//!   4. バックグラウンドで各行を順に処理する: ISBNがあれば楽天ブックスへ`RAKUTEN_REQUEST_INTERVAL`
//!      間隔で問い合わせ、タイトル/説明/カバー画像/発売日/詳細情報を上書きする
//!      （rating・consumed_dateはユーザー個人データのためCSV値を常に維持し、media_typeも
//!      OpenBD判定済みの値を維持する）。楽天キー未設定・呼び出し失敗時はCSV由来の値のまま処理継続する。
//!      `find_existing_import`で(media_type, ISBN)の重複を確認し、既存なら"already imported"としてスキップ、
//!      なければ登録する。1行処理するごとにジョブの進捗を1件進める。

use std::time::Duration;

use axum::Json;
use axum::extract::{Multipart, Path, State};
use axum::response::IntoResponse;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use api_client_lib::auth::AuthStrategy;
use api_client_lib::clients::openbd::OpenBdClient;
use api_client_lib::clients::openbd::models::content_category_digit;
use api_client_lib::clients::openbd::requests::OpenBdGetRequest;
use api_client_lib::clients::rakuten::RakutenClient;
use api_client_lib::clients::rakuten::requests::SearchBooksRequest;

use crate::AppState;
use crate::import::booklog_csv::{ParsedBooklogRow, parse_booklog_csv};
use crate::models::api_credential::ApiProvider;
use crate::models::import::{ImportFailure, ImportSummary};
use crate::models::item::{ItemSource, MediaType};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::{api_credential_repository, item_repository};
use crate::services::external_search::build_book_create_request;
use crate::services::import_job_store;

/// OpenBDへの1リクエストにまとめるISBN件数の上限
const OPENBD_CHUNK_SIZE: usize = 1000;
/// OpenBDへのチャンク間リクエスト間隔（レート制限配慮）
const OPENBD_REQUEST_INTERVAL: Duration = Duration::from_millis(500);
/// 楽天ブックスAPIへのリクエスト間隔（公式レート上限非公開のため保守的に1req/1.1秒とする）
const RAKUTEN_REQUEST_INTERVAL: Duration = Duration::from_millis(1100);

/// `POST /import/booklog`のレスポンスボディ。ジョブ起動直後に返す識別子。
#[derive(Debug, Clone, Serialize)]
struct ImportJobAccepted {
    job_id: Uuid,
}

/// `POST /import/booklog`ハンドラ本体
pub async fn import_booklog_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<axum::response::Response, ApiError> {
    // 【multipart解析】: file/csvいずれかのフィールド名でCSVバイト列を受け取る
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|err| {
        tracing::error!("booklog import multipart parse error: {err}");
        ApiError::new(
            ApiErrorCode::ValidationError,
            "multipartの解析に失敗しました",
        )
    })? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" || name == "csv" {
            let bytes = field.bytes().await.map_err(|err| {
                tracing::error!("booklog import file field read error: {err}");
                ApiError::new(
                    ApiErrorCode::ValidationError,
                    "fileフィールドの読込に失敗しました",
                )
            })?;
            file_bytes = Some(bytes.to_vec());
        }
    }

    let file_bytes =
        file_bytes.ok_or_else(|| ApiError::new(ApiErrorCode::ValidationError, "fileは必須です"))?;

    if file_bytes.is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "fileは空にできません",
        ));
    }

    // 【行単位パース】: CSVを1行ずつ解析し、成功行・失敗行を分離する
    let (mut parsed_rows, failures) = parse_booklog_csv(&file_bytes);

    // 【ジャンル再分類】: OpenBDのCコードで暫定Novel行をNovel/AcademicBookへ再分類する（同期・高速）
    classify_media_types(&mut parsed_rows).await;

    // 【ジョブ登録】: 楽天エンリッチメント＋DB登録はバックグラウンドで実行し、即座にjob_idを返す
    let job_id = import_job_store::create_job(parsed_rows.len() as u32);
    let db = state.db.clone();
    tokio::spawn(run_booklog_import_job(db, job_id, parsed_rows, failures));

    Ok(Json(ApiOk::new(ImportJobAccepted { job_id })).into_response())
}

/// `GET /import/booklog/jobs/:job_id`ハンドラ本体。進捗・完了後は最終`ImportSummary`を返す。
pub async fn get_booklog_import_job_handler(
    Path(job_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let job_id = Uuid::parse_str(&job_id)
        .map_err(|_| ApiError::new(ApiErrorCode::ValidationError, "job_idの形式が不正です"))?;

    match import_job_store::get(job_id) {
        Some(status) => Ok(Json(ApiOk::new(status)).into_response()),
        None => Err(ApiError::new(
            ApiErrorCode::ImportJobNotFound,
            "指定されたインポートジョブが見つかりません",
        )),
    }
}

/// `POST /import/booklog/jobs/:job_id/cancel`ハンドラ本体。
/// 中止リクエストを受け付け、バックグラウンドループが次の行に進む前に検知して打ち切る
/// （既に完了/中止済みのジョブへのリクエストは無害な無視として扱う）。
pub async fn cancel_booklog_import_job_handler(
    Path(job_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let job_id = Uuid::parse_str(&job_id)
        .map_err(|_| ApiError::new(ApiErrorCode::ValidationError, "job_idの形式が不正です"))?;

    if import_job_store::get(job_id).is_none() {
        return Err(ApiError::new(
            ApiErrorCode::ImportJobNotFound,
            "指定されたインポートジョブが見つかりません",
        ));
    }

    import_job_store::request_cancel(job_id);

    let status = import_job_store::get(job_id).expect("job existence checked above");
    Ok(Json(ApiOk::new(status)).into_response())
}

/// バックグラウンドジョブ本体: 楽天ブックスでのエンリッチメント → 重複チェック → DB登録を
/// 行単位で順に実行し、1行ごとに`import_job_store::advance`で進捗を進める。
/// 各行の処理を始める前に中止リクエストを確認し、リクエストがあればその時点までの
/// `ImportSummary`で`Cancelled`として打ち切る（実行中の1行は最後まで完了させる）。
async fn run_booklog_import_job(
    db: PgPool,
    job_id: Uuid,
    parsed_rows: Vec<ParsedBooklogRow>,
    initial_failures: Vec<ImportFailure>,
) {
    let mut summary = ImportSummary::empty();
    summary.failures = initial_failures;
    summary.failure_count = summary.failures.len() as u32;

    let rakuten_client = build_rakuten_client(&db).await;
    let mut is_first_rakuten_call = true;

    for mut parsed in parsed_rows {
        if import_job_store::is_cancelling(job_id) {
            import_job_store::cancel(job_id, summary);
            return;
        }

        if let (Some(client), Some(isbn)) = (rakuten_client.as_ref(), parsed.external_id.clone()) {
            if !is_first_rakuten_call {
                tokio::time::sleep(RAKUTEN_REQUEST_INTERVAL).await;
            }
            is_first_rakuten_call = false;
            enrich_from_rakuten(client, &isbn, &mut parsed).await;
        }

        // 【重複チェック】: ISBNがある行は(media_type, external_id)で既存重複を確認し、
        // 既存であれば登録をスキップして"already imported"を記録する
        if let Some(external_id) = parsed.external_id.as_deref() {
            match item_repository::find_existing_import(&db, parsed.request.media_type, external_id)
                .await
            {
                Ok(true) => {
                    summary
                        .record_failure(ImportFailure::new(parsed.row_number, "already imported"));
                    import_job_store::advance(job_id);
                    continue;
                }
                Ok(false) => {}
                Err(err) => {
                    tracing::error!("booklog import duplicate check error: {err:?}");
                    summary.record_failure(ImportFailure::new(parsed.row_number, "db error"));
                    import_job_store::advance(job_id);
                    continue;
                }
            }
        }

        match item_repository::create_item_with_source(
            &db,
            parsed.request,
            ItemSource::Manual,
            parsed.external_id,
        )
        .await
        {
            Ok(_) => summary.record_success(),
            Err(err) => {
                tracing::error!("booklog import db registration error: {err:?}");
                summary.record_failure(ImportFailure::new(parsed.row_number, "db error"));
            }
        }

        import_job_store::advance(job_id);
    }

    import_job_store::complete(job_id, summary);
}

/// providerに登録済みの楽天APIキー（"applicationId:accessKey"形式）から`RakutenClient`を構築する。
/// キー未設定・不正形式・クライアント初期化失敗の場合はNoneを返し、呼び出し側はCSV由来の値のまま処理継続する。
async fn build_rakuten_client(db: &PgPool) -> Option<RakutenClient> {
    let credential =
        match api_credential_repository::find_by_provider(db, ApiProvider::Rakuten).await {
            Ok(Some(credential)) => credential,
            Ok(None) => {
                tracing::warn!("rakuten api key not configured, skipping booklog enrichment");
                return None;
            }
            Err(err) => {
                tracing::error!("rakuten credential lookup failed: {err:?}");
                return None;
            }
        };

    let Some((application_id, access_key)) = credential.api_key.split_once(':') else {
        tracing::error!("invalid rakuten credential format (expected \"applicationId:accessKey\")");
        return None;
    };

    let auth = AuthStrategy::RakutenAppAuth {
        application_id: application_id.to_string(),
        access_key: access_key.to_string(),
    };

    match RakutenClient::new(auth) {
        Ok(client) => Some(client),
        Err(err) => {
            tracing::error!("rakuten client init failed: {err}");
            None
        }
    }
}

/// 楽天ブックスをISBN検索し、見つかった書誌でタイトル/説明/カバー画像/発売日/詳細情報を上書きする。
/// rating・consumed_date（ユーザー個人データ）とmedia_type（OpenBD判定済み）は維持する。
/// 該当書誌なし・呼び出し失敗時は何もせずCSV由来の値のまま処理を継続する。
async fn enrich_from_rakuten(client: &RakutenClient, isbn: &str, parsed: &mut ParsedBooklogRow) {
    let response = match client
        .search_books(SearchBooksRequest {
            isbn: Some(isbn.to_string()),
            ..Default::default()
        })
        .await
    {
        Ok(response) => response,
        Err(err) => {
            tracing::warn!(
                "rakuten enrichment failed for booklog row {}: {err}",
                parsed.row_number
            );
            return;
        }
    };

    let Some(book) = response.model.into_iter().next() else {
        return;
    };

    let media_type = parsed.request.media_type;
    let rating = parsed.request.rating;
    let consumed_date = parsed.request.consumed_date;

    let mut enriched = build_book_create_request(&book, media_type);
    if enriched.title.trim().is_empty() {
        enriched.title = parsed.request.title.clone();
    }
    enriched.rating = rating;
    enriched.consumed_date = consumed_date;

    parsed.request = enriched;
}

/// OpenBDのCコードを使い、`needs_classification=true`な行の`media_type`を
/// Novel/AcademicBookへ再分類する。ISBNをチャンク化してリクエスト数を抑え、
/// チャンク間は`OPENBD_REQUEST_INTERVAL`だけ間隔を空ける。
/// OpenBDクライアント初期化・通信に失敗した場合は暫定Novelのままとし、処理を止めない。
async fn classify_media_types(parsed_rows: &mut [ParsedBooklogRow]) {
    let target_indices: Vec<usize> = parsed_rows
        .iter()
        .enumerate()
        .filter(|(_, row)| row.needs_classification)
        .map(|(idx, _)| idx)
        .collect();

    if target_indices.is_empty() {
        return;
    }

    let client = match OpenBdClient::new() {
        Ok(client) => client,
        Err(err) => {
            tracing::error!("openbd client init failed, keeping Novel fallback: {err}");
            return;
        }
    };

    for (chunk_idx, chunk) in target_indices.chunks(OPENBD_CHUNK_SIZE).enumerate() {
        if chunk_idx > 0 {
            tokio::time::sleep(OPENBD_REQUEST_INTERVAL).await;
        }

        let isbns: Vec<String> = chunk
            .iter()
            .filter_map(|&idx| parsed_rows[idx].external_id.clone())
            .collect();

        let response = match client.get(OpenBdGetRequest { isbns }).await {
            Ok(response) => response,
            Err(err) => {
                tracing::warn!("openbd batch lookup failed, keeping Novel fallback: {err}");
                continue;
            }
        };

        for (&idx, item) in chunk.iter().zip(response.model) {
            if let Some(media_type) = item
                .and_then(|item| item.c_code)
                .and_then(|c_code| content_category_digit(&c_code))
                .map(|digit| {
                    if digit == 9 {
                        MediaType::Novel
                    } else {
                        MediaType::AcademicBook
                    }
                })
            {
                parsed_rows[idx].request.media_type = media_type;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::Value;
    use tower::ServiceExt;

    /// テスト用ヘルパー: DATABASE_URL接続のAppStateを構築する
    async fn test_app_state() -> AppState {
        let database_url =
            std::env::var("DATABASE_URL").expect("統合テストにはDATABASE_URL環境変数が必要です");
        let db = sqlx::PgPool::connect(&database_url)
            .await
            .expect("テスト用DBへの接続に失敗しました");
        AppState {
            db,
            internal_api_key: String::new(),
        }
    }

    /// テスト用ヘルパー: ルーター（本ハンドラ + ジョブ状態取得ハンドラ）を構築する
    fn build_test_router(state: AppState) -> axum::Router {
        axum::Router::new()
            .route(
                "/import/booklog",
                axum::routing::post(import_booklog_handler),
            )
            .route(
                "/import/booklog/jobs/{job_id}",
                axum::routing::get(get_booklog_import_job_handler),
            )
            .route(
                "/import/booklog/jobs/{job_id}/cancel",
                axum::routing::post(cancel_booklog_import_job_handler),
            )
            .with_state(state)
    }

    /// テスト用ヘルパー: CSVバイト列をmultipart/form-data（fileフィールド）のボディへ組み立てる
    fn multipart_body_with_file(csv_content: &str) -> (String, Vec<u8>) {
        let boundary = "----BooklogTestBoundary";
        let mut body = Vec::new();
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            b"Content-Disposition: form-data; name=\"file\"; filename=\"booklog.csv\"\r\n",
        );
        body.extend_from_slice(b"Content-Type: text/csv\r\n\r\n");
        body.extend_from_slice(csv_content.as_bytes());
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        (boundary.to_string(), body)
    }

    /// テスト用ヘルパー: 17列固定のブクログCSV1行（タイプ=マンガ固定、OpenBD/楽天呼び出し回避）を組み立てる
    fn booklog_manga_row(isbn: &str, title: &str) -> String {
        format!(
            "\"1\",\"0000000000\",\"{isbn}\",\"\",\"\",\"読み終わった\",\"\",\"\",\"\",\"2022-06-16 12:32:57\",\"\",\"{title}\",\"著者\",\"出版社\",\"2020\",\"マンガ\",\"300\""
        )
    }

    /// テスト用ヘルパー: POST /import/booklogを実行しjob_idを取り出す
    async fn post_booklog_csv(app: axum::Router, csv: &str) -> (StatusCode, Option<Uuid>) {
        let (boundary, body) = multipart_body_with_file(csv);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/booklog")
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        if status != StatusCode::OK {
            return (status, None);
        }
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        let job_id = Uuid::parse_str(json["data"]["job_id"].as_str().unwrap()).unwrap();
        (status, Some(job_id))
    }

    /// テスト用ヘルパー: ジョブが完了するまでポーリングし、最終`ImportSummary`のJSONを返す
    async fn wait_for_job_completion(app: axum::Router, job_id: Uuid) -> Value {
        loop {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("GET")
                        .uri(format!("/import/booklog/jobs/{job_id}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let json: Value = serde_json::from_slice(&bytes).unwrap();
            if json["data"]["status"] == "completed" {
                return json["data"]["summary"].clone();
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    /// 正常CSV全行登録（ジョブ完了後のImportSummary成功サマリ）。
    /// タイプ=マンガ固定行を使い、OpenBD/楽天への外部呼び出しを回避する。
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn import_booklog_with_all_valid_rows_returns_success_summary() {
        let state = test_app_state().await;
        let app = build_test_router(state);

        let csv = format!(
            "{}\r\n{}\r\n",
            booklog_manga_row("9784063876500", "進撃の巨人 1"),
            booklog_manga_row("9784063876517", "進撃の巨人 2"),
        );
        let (status, job_id) = post_booklog_csv(app.clone(), &csv).await;
        assert_eq!(status, StatusCode::OK);

        let summary = wait_for_job_completion(app, job_id.unwrap()).await;
        assert_eq!(summary["success_count"], 2);
        assert_eq!(summary["failure_count"], 0);
    }

    /// ファイル未添付 → 400 VALIDATION_ERROR
    #[tokio::test]
    #[ignore]
    async fn import_booklog_without_file_field_returns_400() {
        let state = test_app_state().await;
        let app = build_test_router(state);

        let boundary = "----EmptyBoundary";
        let body = format!("--{boundary}--\r\n").into_bytes();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/booklog")
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// 0バイトファイル → 400 VALIDATION_ERROR
    #[tokio::test]
    #[ignore]
    async fn import_booklog_with_zero_byte_file_returns_400() {
        let state = test_app_state().await;
        let app = build_test_router(state);

        let (boundary, body) = multipart_body_with_file("");

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/booklog")
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// 全行不正でも200・パニックせず完走する（ジョブ完了後にfailure_countが行数と一致）
    #[tokio::test]
    #[ignore]
    async fn import_booklog_with_all_invalid_rows_returns_failure_summary() {
        let state = test_app_state().await;
        let app = build_test_router(state);

        let csv = format!(
            "{}\r\n{}\r\n{}\r\n",
            booklog_manga_row("", ""),
            booklog_manga_row("", ""),
            booklog_manga_row("", ""),
        );
        let (status, job_id) = post_booklog_csv(app.clone(), &csv).await;
        assert_eq!(status, StatusCode::OK);

        let summary = wait_for_job_completion(app, job_id.unwrap()).await;
        assert_eq!(summary["success_count"], 0);
        assert_eq!(summary["failure_count"], 3);
    }

    /// 同一ISBNを2回登録すると2回目は"already imported"としてスキップされ、重複登録されない
    #[tokio::test]
    #[ignore]
    async fn import_booklog_skips_already_imported_isbn_on_second_upload() {
        let state = test_app_state().await;
        let app1 = build_test_router(state.clone());
        let app2 = build_test_router(state);

        let csv = format!("{}\r\n", booklog_manga_row("9784063876500", "進撃の巨人 1"));

        let (status1, job_id1) = post_booklog_csv(app1.clone(), &csv).await;
        assert_eq!(status1, StatusCode::OK);
        let first_summary = wait_for_job_completion(app1, job_id1.unwrap()).await;
        assert_eq!(first_summary["success_count"], 1);

        let (status2, job_id2) = post_booklog_csv(app2.clone(), &csv).await;
        assert_eq!(status2, StatusCode::OK);
        let second_summary = wait_for_job_completion(app2, job_id2.unwrap()).await;
        assert_eq!(second_summary["success_count"], 0);
        assert_eq!(second_summary["failure_count"], 1);
        assert_eq!(second_summary["failures"][0]["reason"], "already imported");
    }

    /// 存在しないjob_idへのポーリングは404 IMPORT_JOB_NOT_FOUND
    #[tokio::test]
    async fn get_booklog_import_job_with_unknown_job_id_returns_404() {
        let app = axum::Router::new().route(
            "/import/booklog/jobs/{job_id}",
            axum::routing::get(get_booklog_import_job_handler),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/import/booklog/jobs/{}", Uuid::new_v4()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// job_idがUUID形式でない場合は400 VALIDATION_ERROR
    #[tokio::test]
    async fn get_booklog_import_job_with_invalid_job_id_returns_400() {
        let app = axum::Router::new().route(
            "/import/booklog/jobs/{job_id}",
            axum::routing::get(get_booklog_import_job_handler),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/import/booklog/jobs/not-a-uuid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// 存在しないjob_idへの中止リクエストは404 IMPORT_JOB_NOT_FOUND
    #[tokio::test]
    async fn cancel_booklog_import_job_with_unknown_job_id_returns_404() {
        let app = axum::Router::new().route(
            "/import/booklog/jobs/{job_id}/cancel",
            axum::routing::post(cancel_booklog_import_job_handler),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/import/booklog/jobs/{}/cancel", Uuid::new_v4()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// job_idがUUID形式でない場合は400 VALIDATION_ERROR
    #[tokio::test]
    async fn cancel_booklog_import_job_with_invalid_job_id_returns_400() {
        let app = axum::Router::new().route(
            "/import/booklog/jobs/{job_id}/cancel",
            axum::routing::post(cancel_booklog_import_job_handler),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/booklog/jobs/not-a-uuid/cancel")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    /// 実行中ジョブへ中止をリクエストするとジョブは打ち切られ、最終ステータスがcancelledになる
    /// （多数行・楽天キー未設定でRakuten間隔スリープが挟まらないため、進捗を数件確認した直後に
    /// 中止リクエストを送ることでバックグラウンドループの中止チェックに間に合わせる）
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn cancelling_a_running_job_marks_it_cancelled() {
        let state = test_app_state().await;
        let app = build_test_router(state);

        let rows: Vec<String> = (0..500)
            .map(|i| booklog_manga_row(&format!("97841010100{i:02}"), &format!("巻{i}")))
            .collect();
        let csv = format!("{}\r\n", rows.join("\r\n"));

        let (status, job_id) = post_booklog_csv(app.clone(), &csv).await;
        assert_eq!(status, StatusCode::OK);
        let job_id = job_id.unwrap();

        let cancel_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/import/booklog/jobs/{job_id}/cancel"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(cancel_response.status(), StatusCode::OK);

        let final_status = loop {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("GET")
                        .uri(format!("/import/booklog/jobs/{job_id}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let json: Value = serde_json::from_slice(&bytes).unwrap();
            let status = json["data"]["status"].as_str().unwrap().to_string();
            if status == "cancelled" || status == "completed" {
                break status;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        };

        assert_eq!(final_status, "cancelled");
    }
}
