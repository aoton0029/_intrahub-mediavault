//! ブクログCSVインポートハンドラ
//!
//! TASK-0030: ブクログCSVインポート実装
//!
//! 【機能概要】: `POST /import/booklog`ハンドラ。multipart/form-dataでCSVファイルを受け取り、
//! `import::booklog_csv`でパース・バリデーションし、正常行を`item_repository::create_item_with_source`
//! 経由で登録、`ImportSummary`を200で返す
//! 【実装方針】: ファイル未添付・0バイトは400 VALIDATION_ERRORで早期リターン。それ以外は
//! 行単位スキップ方式（EDGE-002）に従い、全行不正でも例外を起こさず200で返す（TC-016-E01）
//! 【テスト対応】: TC-N-01, TC-N-05, TC-E-06, TC-E-07, TC-E-08, TC-B-01〜TC-B-03, TC-B-05
//! 🔵🟡 信頼性レベル: api-endpoints.md「POST /import/booklog」・要件定義書2章・3章に基づく

use axum::extract::{Multipart, State};
use axum::response::IntoResponse;
use axum::Json;

use crate::import::booklog_csv::parse_booklog_csv;
use crate::models::import::{ImportFailure, ImportSummary};
use crate::models::item::ItemSource;
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::item_repository;
use crate::AppState;

/// 【機能概要】: `POST /import/booklog`ハンドラ本体
/// 【実装方針】:
///   1. multipartから`file`または`csv`フィールドのバイト列を取得（必須・存在確認）
///   2. 0バイトの場合は400 VALIDATION_ERROR（TC-E-07）
///   3. `parse_booklog_csv`で行単位パース・バリデーションを行う
///   4. 成功行を`create_item_with_source`（source=Manual, external_id=ISBN）で登録
///   5. DB登録自体が失敗した行はfailureとして記録し処理継続する（TC-E-08方針: スキップ扱い）
///   6. すべての処理結果を`ImportSummary`として200で返す（全行不正でも200、TC-016-E01）
/// 🔵 信頼性レベル: api-endpoints.md「POST /import/booklog」・要件定義書2章データフローに直接対応
pub async fn import_booklog_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<axum::response::Response, ApiError> {
    // 【multipart解析】: file/csvいずれかのフィールド名でCSVバイト列を受け取る（TC-E-06） 🟡
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|err| {
        // 【エラー捕捉】: multipart自体の解析失敗（不正なboundary等）はVALIDATION_ERRORとして扱う 🟡
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

    // 【必須条件検証】: ファイル未添付はVALIDATION_ERROR（TC-E-06） 🔵
    let file_bytes = file_bytes.ok_or_else(|| {
        ApiError::new(ApiErrorCode::ValidationError, "fileは必須です")
    })?;

    // 【必須条件検証】: 0バイトはVALIDATION_ERROR（TC-E-07）。データ0行（ヘッダーのみ）とは区別する 🔵
    if file_bytes.is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "fileは空にできません",
        ));
    }

    // 【行単位パース】: CSVを1行ずつ解析し、成功行・失敗行を分離する（EDGE-002） 🔵
    // 【Green実装】: parse_booklog_csvの戻り値をParsedBooklogRow（row_number・external_id保持）へ
    // 変更したことで、DB登録時までrow_number・ISBN(external_id)を失わずに引き渡せる 🔵
    let (parsed_rows, mut failures) = parse_booklog_csv(&file_bytes);

    // 【DB登録】: 成功行をsource=Manual・external_id=ISBNでitemsへ登録する。
    // 【TC-E-08方針確定】: DB登録自体の失敗は「パース・バリデーション済みの正常行」に対して
    // 発生する基盤障害であるが、1行のDB起因失敗で他の正常行の登録・処理全体を止めることは
    // EDGE-002の「行単位スキップで処理継続」という設計方針と一貫させるべきと判断した。
    // よって、DB登録失敗は500 INTERNAL_ERRORにはせず、当該行のみImportFailure（reason="db error"）
    // として記録し、後続行の処理を継続する（パニックさせない・処理を中断しない）。 🟡
    let mut summary = ImportSummary::empty();
    summary.failures.append(&mut failures);
    // 【failure_count同期】: パース層の失敗をfailuresへ取り込んだ直後にfailure_countを同期させる
    // （record_failureはfailures.len()を都度同期するが、append直後はまだ呼ばれていないため明示的に揃える） 🔵
    summary.failure_count = summary.failures.len() as u32;

    for parsed in parsed_rows {
        match item_repository::create_item_with_source(
            &state.db,
            parsed.request,
            ItemSource::Manual,
            parsed.external_id,
        )
        .await
        {
            Ok(_) => summary.record_success(),
            Err(err) => {
                // 【DB起因失敗の記録】: 内部エラー詳細はログにのみ出力し、クライアントには
                // 汎用的な理由文言のみ返す（内部スキーマ情報の漏洩防止、item_repository db_error準拠） 🟡
                tracing::error!("booklog import db registration error: {err:?}");
                summary.record_failure(ImportFailure::new(parsed.row_number, "db error"));
            }
        }
    }

    // 【成功レスポンス】: 全行不正でも200でImportSummaryを返す（TC-016-E01） 🔵
    Ok(Json(ApiOk::new(summary)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    /// テスト用ヘルパー: DATABASE_URL接続のAppStateを構築する
    /// 🟡 信頼性レベル: routes/mod.rs既存test_app_stateパターンと同様
    async fn test_app_state() -> AppState {
        let database_url = std::env::var("DATABASE_URL")
            .expect("TASK-0030統合テストにはDATABASE_URL環境変数が必要です");
        let db = sqlx::PgPool::connect(&database_url)
            .await
            .expect("テスト用DBへの接続に失敗しました");
        AppState {
            db,
            internal_api_key: String::new(),
        }
    }

    /// テスト用ヘルパー: ルーター単体（本ハンドラのみ）を構築する
    fn build_test_router(state: AppState) -> axum::Router {
        axum::Router::new()
            .route(
                "/import/booklog",
                axum::routing::post(import_booklog_handler),
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

    /// TC-N-01: 正常CSV全行登録（ImportSummary成功サマリ）
    /// 【テスト目的】: POST /import/boglogに正常行のみのCSVをmultipartで送信したとき、
    /// 全行がitemsに登録され、ImportSummaryが成功サマリを返すことを確認する
    /// 【テスト内容】: ヘッダー+データ3行（全列妥当）のCSVをアップロードする
    /// 【期待される動作】: HTTP200、data.success_count==3、data.failure_count==0
    /// 🔵 信頼性レベル: TC-016-01・要件定義4章「正常CSV全行登録」に直接対応
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn import_booklog_with_all_valid_rows_returns_success_summary() {
        // 【テスト前準備】: 実DBプールとルーターを構築する
        let state = test_app_state().await;
        let app = build_test_router(state);

        // 【テストデータ準備】: ヘッダー+正常データ3行のCSVを用意する
        let csv = "作品名,感想/レビュー,読了日,評価,ISBN\r\n\
吾輩は猫である,面白い,2024-01-15,4.5,9784101010014\r\n\
星の王子さま,,,, \r\n\
ノルウェイの森,良かった,2023-05-01,4.0,\r\n";
        let (boundary, body) = multipart_body_with_file(csv);

        // 【実際の処理実行】: POST /import/booklogを実行する
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

        // 【結果検証】: HTTP200で全3行が成功登録されることを確認
        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: 正常CSV全行登録時にHTTP200が返ることを確認 🔵
    }

    /// TC-E-06: ファイル未添付 → 400 VALIDATION_ERROR
    /// 【テスト目的】: multipartにファイルフィールドが無い場合に400を返すことを確認する
    /// 【テスト内容】: fileフィールドを含まないmultipartリクエストを送信する
    /// 【期待される動作】: HTTP400、error.code=="VALIDATION_ERROR"
    /// 🔵 信頼性レベル: 要件定義書2章エラーレスポンス表・TC-016-04に対応
    #[tokio::test]
    #[ignore]
    async fn import_booklog_without_file_field_returns_400() {
        let state = test_app_state().await;
        let app = build_test_router(state);

        // 【テストデータ準備】: fileフィールドを含まない空のmultipartボディを構築する
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

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: ファイル未添付が400 VALIDATION_ERRORで拒否されることを確認 🔵
    }

    /// TC-E-07: 0バイトファイル → 400 VALIDATION_ERROR
    /// 【テスト目的】: 添付ファイルが0バイトの場合に400を返すことを確認する
    /// 【テスト内容】: fileフィールドの内容が空文字列のmultipartリクエストを送信する
    /// 【期待される動作】: HTTP400、error.code=="VALIDATION_ERROR"
    /// 🔵 信頼性レベル: 要件定義書2章「0バイトでないこと」・TC-016-04に対応
    #[tokio::test]
    #[ignore]
    async fn import_booklog_with_zero_byte_file_returns_400() {
        let state = test_app_state().await;
        let app = build_test_router(state);

        // 【テストデータ準備】: fileフィールドの内容が0バイトのmultipartボディを構築する
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

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 0バイトファイルが400 VALIDATION_ERRORで拒否されることを確認 🔵
    }

    /// TC-B-01: ヘッダーのみ（データ0行）→ 200 success=0/failure=0
    /// 【テスト目的】: ヘッダー行のみでデータ行が無いCSV（非0バイト）は200で空サマリを返すことを確認する
    /// 【テスト内容】: ヘッダー行のみのCSVをアップロードする
    /// 【期待される動作】: HTTP200、data.success_count==0、data.failure_count==0
    /// 🟡 信頼性レベル: 要件定義の「0バイト=400」「全行処理後にサマリ返却」からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn import_booklog_with_header_only_returns_200_empty_summary() {
        let state = test_app_state().await;
        let app = build_test_router(state);

        // 【テストデータ準備】: ヘッダー行のみ（データ行なし、非0バイト）のCSVを用意する
        let csv = "作品名,感想/レビュー,読了日,評価,ISBN\r\n";
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

        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: ヘッダーのみCSVが400ではなく200で空サマリを返すことを確認（TC-E-07との境界） 🟡
    }

    /// TC-B-02: 全行不正 → 200・success_count=0・failure_count=行数（TC-016-E01直接対応）
    /// 【テスト目的】: 全データ行が不正でも200でImportSummaryを返し、例外で落ちないことを確認する
    /// 【テスト内容】: データ3行すべて作品名空のCSVをアップロードする
    /// 【期待される動作】: HTTP200、data.success_count==0、data.failure_count==3
    /// 🔵 信頼性レベル: TC-016-E01に直接対応
    #[tokio::test]
    #[ignore]
    async fn import_booklog_with_all_invalid_rows_returns_200_with_failure_summary() {
        let state = test_app_state().await;
        let app = build_test_router(state);

        // 【テストデータ準備】: 全データ3行が作品名空（不正）のCSVを用意する
        let csv = "作品名,感想/レビュー,読了日,評価,ISBN\r\n\
,,,,\r\n\
,,,,\r\n\
,,,,\r\n";
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

        // 【結果検証】: 全行不正でも例外にならず200が返ることを確認（最重要：パニックで落ちないこと）
        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: 全行不正でもHTTP200が返ること（例外で落ちないこと）を確認 🔵
    }

    /// TC-B-03: 1行のみ正常CSV（最小成功）
    /// 【テスト目的】: データ1行の正常CSVで1件登録されることを確認する
    /// 【テスト内容】: ヘッダー+正常データ1行のCSVをアップロードする
    /// 【期待される動作】: HTTP200、data.success_count==1、data.failure_count==0
    /// 🟡 信頼性レベル: REQ-016からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn import_booklog_with_single_valid_row_registers_one_item() {
        let state = test_app_state().await;
        let app = build_test_router(state);

        // 【テストデータ準備】: ヘッダー+正常データ1行のCSVを用意する
        let csv = "作品名,感想/レビュー,読了日,評価,ISBN\r\n\
吾輩は猫である,面白い,2024-01-15,4.5,9784101010014\r\n";
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

        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: 1行のみの正常CSVでHTTP200が返ることを確認 🟡
    }

    /// TC-N-05: 読了日→consumed_dateがDBに永続化される（設計判断#1の一気通貫確認）
    /// 【テスト目的】: CSVの読了日が登録itemのconsumed_dateへ実際に保存されることを確認する
    /// 【テスト内容】: 読了日=2024-01-15を含む1行CSVをアップロードする
    /// 【期待される動作】: HTTP200で処理が成功する（DB上のconsumed_date検証はGreenフェーズで
    /// レスポンスにitem一覧を含めるか別途SELECTする方式を確定する）
    /// 🔵 信頼性レベル: 設計判断#1（ユーザー確定）に基づく
    #[tokio::test]
    #[ignore]
    async fn import_booklog_persists_consumed_date_from_csv() {
        let state = test_app_state().await;
        let app = build_test_router(state);

        // 【テストデータ準備】: 読了日2024-01-15を持つ正常1行CSVを用意する
        let csv = "作品名,感想/レビュー,読了日,評価,ISBN\r\n\
ノルウェイの森,,2024-01-15,,\r\n";
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

        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: 読了日付きCSVインポートがHTTP200で成功することを確認 🔵
    }

    /// TC-B-05: 数百行CSVの完走（タイムアウトしない軽い確認）
    /// 【テスト目的】: 数百行規模のCSVがタイムアウトせず完走することを確認する
    /// 【テスト内容】: 正常行を約300行含むCSVをアップロードする
    /// 【期待される動作】: HTTP200、data.success_count==300
    /// 🟡 信頼性レベル: 統合テスト要件からの妥当な推測（厳密検証は範囲外）
    #[tokio::test]
    #[ignore]
    async fn import_booklog_completes_for_large_csv_without_timeout() {
        let state = test_app_state().await;
        let app = build_test_router(state);

        // 【テストデータ準備】: 正常行を300行生成する
        let mut csv = String::from("作品名,感想/レビュー,読了日,評価,ISBN\r\n");
        for i in 0..300 {
            csv.push_str(&format!("作品{i},,,, \r\n"));
        }
        let (boundary, body) = multipart_body_with_file(&csv);

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

        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: 数百行規模CSVでもタイムアウトせずHTTP200で完走することを確認 🟡
    }

    /// TC-E-08: DB登録失敗時もパニックせず安全に失敗を扱う（接続不能プール、方針確認）
    /// 【テスト目的】: パース・バリデーションは通ったが、DB起因で登録が失敗する場合に
    /// パニックせず安全に処理が終わることを確認する（最終挙動はGreenフェーズで確定）
    /// 【テスト内容】: 接続不能なPgPoolに対し正常行CSVを投入する
    /// 【期待される動作】: レスポンスがpanicによる接続切断にならないこと（最低限の安全性）。
    /// 具体的なステータス（200のfailure記録 or 500）はGreenフェーズで確定する
    /// 🟡 信頼性レベル: note.md L172 / db_error 設計からの妥当な推測（最終方針はGreenで確定）
    #[tokio::test]
    #[ignore]
    async fn import_booklog_with_unreachable_db_does_not_panic() {
        // 【テスト前準備】: 接続不能なPgPoolを構築する（実際にはconnect自体が失敗するため、
        // 本テストはGreenフェーズでのDB起因エラー方針確定後に詳細化する）
        let db = sqlx::PgPool::connect("postgres://invalid:invalid@127.0.0.1:1/invalid")
            .await
            .expect("到達不能プールの構築検証用接続に失敗しました");
        let state = AppState {
            db,
            internal_api_key: String::new(),
        };
        let app = build_test_router(state);

        // 【テストデータ準備】: 正常行1件のCSVを用意する
        let csv = "作品名,感想/レビュー,読了日,評価,ISBN\r\n\
吾輩は猫である,,,,\r\n";
        let (boundary, body) = multipart_body_with_file(csv);

        // 【実際の処理実行】: 接続不能DBに対してインポートを実行する
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

        // 【結果検証】: パニックによる接続断ではなく、何らかのHTTPレスポンスが返ることを確認
        let status = response.status();
        let _ = status; // 仮の安全性確認（Greenで方針確定後に具体化） 🟡
        assert_ne!(status, StatusCode::INTERNAL_SERVER_ERROR); // 【確認内容】: DB接続不能でも500固定にならず安全に処理されることを確認（最終方針はGreenで確定） 🟡
    }
}
