//! Steamライブラリインポートハンドラ
//!
//! TASK-0031: Steamライブラリインポート実装
//!
//! 【機能概要】: `POST /import/steam`ハンドラ。`{ "steam_id": "..." }`をJSONで受け取り、
//! `import::steam_import::import_steam_library`へ委譲し、`ImportSummary`を200で返す。
//! 【実装方針】: TASK-0030 `import_booklog_handler`と同様の構成（検証→usecase呼び出し→ImportSummary返却）。
//! steam_id形式不正は400 VALIDATION_ERROR、APIキー無効/未設定は401 STEAM_API_KEY_INVALID、
//! 外部APIタイムアウトは502 EXTERNAL_API_TIMEOUTへそれぞれマッピングする。
//! 【テスト対応】: TC-017-01, TC-017-E01, TC-017-E03, TC-017-02
//! 🔵🟡 信頼性レベル: steam-import-requirements.md・TASK-0031.mdに基づく
//!
//! 【Red フェーズ注記】: `import_steam_handler`は未実装（`todo!()`）。Greenフェーズで実装する。

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::AppState;
use crate::import::steam_import::{SteamCredentialLookup, import_steam_library};
use crate::models::response::{ApiError, ApiOk};

/// `POST /import/steam` リクエストボディDTO
///
/// 🔵 信頼性レベル: 要件定義書2章「SteamImportRequest { steam_id: String }」に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct SteamImportRequest {
    pub steam_id: String,
}

/// 【機能概要】: `POST /import/steam`ハンドラ本体
/// 【実装方針】:
///   1. リクエストボディを`SteamImportRequest`としてデシリアライズする
///   2. `import::steam_import::import_steam_library`へ`steam_id`を渡して委譲する
///   3. 戻り値の`ImportSummary`を200で返す（エラーは`?`演算子で`ApiError`へ自動変換）
///
/// 🔵 信頼性レベル: api-endpoints.md「POST /import/steam」・要件定義書2章データフローに直接対応
pub async fn import_steam_handler(
    State(state): State<AppState>,
    Json(request): Json<SteamImportRequest>,
) -> Result<axum::response::Response, ApiError> {
    tracing::info!("Steamライブラリのインポートを開始しました");
    // 【usecase呼び出し】: 本番経路はSteamCredentialLookup::Pool(実DB)・ベースURLはNone（本番URL固定）を使う 🔵
    let credentials = SteamCredentialLookup::Pool(state.db.clone());
    let summary = import_steam_library(&state.db, &credentials, None, &request.steam_id).await?;
    tracing::info!(
        success_count = summary.success_count,
        failure_count = summary.failure_count,
        "Steamライブラリのインポートが完了しました"
    );

    // 【成功レスポンス】: ImportSummaryを200で返す 🔵
    Ok(Json(ApiOk::new(summary)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// テスト用ヘルパー: DATABASE_URL接続のAppStateを構築する
    /// 🟡 信頼性レベル: import_booklog.rs既存test_app_stateパターンと同様
    async fn test_app_state() -> AppState {
        let database_url = std::env::var("DATABASE_URL")
            .expect("TASK-0031統合テストにはDATABASE_URL環境変数が必要です");
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
            .route("/import/steam", axum::routing::post(import_steam_handler))
            .with_state(state)
    }

    /// TC-017-E03: steam_id形式不正（空文字）時に400 VALIDATION_ERRORを返す（ハンドラ統合、実DB必要）
    /// 【テスト目的】: ハンドラ経由でも不正なsteam_idが400で拒否されることを確認する
    /// 【テスト内容】: POST /import/steamに{ "steam_id": "" }を送信する
    /// 【期待される動作】: HTTPステータス400、error.code=="VALIDATION_ERROR"
    /// 🟡 信頼性レベル: TASK-0031.md「テストケース4」・要件定義書4章に対応
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn import_steam_with_empty_steam_id_returns_400() {
        // 【テスト前準備】: 実DBプールとルーターを構築する
        let state = test_app_state().await;
        let app = build_test_router(state);

        // 【テストデータ準備】: 空文字のsteam_idを含むリクエストボディを用意する
        let body = serde_json::json!({ "steam_id": "" }).to_string();

        // 【実際の処理実行】: POST /import/steamを実行する
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/steam")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: 不正なsteam_idが400で拒否されることを確認
        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 空文字steam_idが400 VALIDATION_ERRORで拒否されることを確認 🟡
    }

    /// TC-017-E01: Steam APIキー未設定時に401 STEAM_API_KEY_INVALIDを返す（ハンドラ統合、実DB必要）
    /// 【テスト目的】: ハンドラ経由でAPIキー未設定時に401が返ることを確認する
    /// 【テスト内容】: api_credentialsにSteamキーが未登録の状態でPOST /import/steamを実行する
    /// 【期待される動作】: HTTPステータス401、error.code=="STEAM_API_KEY_INVALID"
    /// 🔵 信頼性レベル: TASK-0031.md「テストケース2（TC-017-E01）」に直接対応
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn import_steam_without_configured_api_key_returns_401() {
        // 【テスト前準備】: 実DBプールとルーターを構築し、Steamキーを未登録（削除済み）にする
        let state = test_app_state().await;
        sqlx::query("DELETE FROM api_credentials WHERE provider = 'steam'::api_provider")
            .execute(&state.db)
            .await
            .expect("テスト前クリーンアップに失敗しました");
        let app = build_test_router(state);

        // 【テストデータ準備】: 正規形式のsteam_idを含むリクエストボディを用意する
        let body = serde_json::json!({ "steam_id": "76561198000000000" }).to_string();

        // 【実際の処理実行】: POST /import/steamを実行する
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/steam")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: APIキー未設定が401で拒否されることを確認
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // 【確認内容】: APIキー未設定時に401 STEAM_API_KEY_INVALIDが返ることを確認 🔵
    }

    /// TC-017-01: 正常な所持ゲーム一覧の一括登録が200で成功する（ハンドラ統合、実DB+wiremock必要）
    /// 【テスト目的】: ハンドラ経由でのエンドツーエンドフローが200で成功することを確認する
    /// 【テスト内容】: Steam APIキーをDBへ事前投入し、wiremockが複数ゲームを返す状態でPOST /import/steamを実行する
    /// 【期待される動作】: HTTPステータス200、data.success_countが取得件数と一致する
    /// 🔵 信頼性レベル: TASK-0031.md「単体テスト要件 テストケース1（TC-017-01）」に直接対応
    ///
    /// 【注記】: 本ハンドラ統合テストはSteam APIベースURLの差し替え手段（テスト専用DI）が
    /// Greenフェーズで確定するまで、wiremockサーバーへの到達を直接検証できない。
    /// 現状は実装方針確認のためのプレースホルダとして400系にならないことのみ確認する設計とし、
    /// Greenフェーズでテスト専用ベースURL注入経路が確定した時点でアサーションを具体化する。
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）+ Steam APIベースURL差し替え手段確定後に有効化
    async fn import_steam_with_valid_key_and_games_returns_200() {
        // 【テスト前準備】: 実DBプールとルーターを構築し、Steamキーを投入する
        let state = test_app_state().await;
        sqlx::query("DELETE FROM api_credentials WHERE provider = 'steam'::api_provider")
            .execute(&state.db)
            .await
            .expect("テスト前クリーンアップに失敗しました");
        sqlx::query(
            "INSERT INTO api_credentials (provider, api_key) VALUES ('steam'::api_provider, $1)",
        )
        .bind("valid-steam-key")
        .execute(&state.db)
        .await
        .expect("テスト用キー投入に失敗しました");

        let steam_mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "response": {
                    "game_count": 2,
                    "games": [
                        {"appid": 440, "name": "Team Fortress 2", "playtime_forever": 1200},
                        {"appid": 570, "name": "Dota 2", "playtime_forever": 600}
                    ]
                }
            })))
            .mount(&steam_mock)
            .await;

        let app = build_test_router(state);

        // 【テストデータ準備】: 正規形式のsteam_idを含むリクエストボディを用意する
        let body = serde_json::json!({ "steam_id": "76561198000000000" }).to_string();

        // 【実際の処理実行】: POST /import/steamを実行する
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import/steam")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: 正常系が200で成功することを確認
        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: 有効なAPIキー・複数ゲーム返却時にHTTP200が返ることを確認 🔵
    }
}
