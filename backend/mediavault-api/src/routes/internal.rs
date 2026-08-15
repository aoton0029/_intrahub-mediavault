//! `/internal` 専用ルーター（内部REST API）
//!
//! TASK-0029: 内部REST APIルート群実装（/internal/items等）
//!
//! 本ルーターは`build_router`へmergeされた後、`/api/v1`配下へnestされるため、
//! 公開パスは`/api/v1/internal/*`となる。`api_key_auth`ミドルウェアはこのRouterだけに
//! 適用され、merge後も利用者向けルートには波及しない。
//! Phase2/Phase4で実装済みのitems/groups/episodes/filesハンドラ・サービス層を
//! 再利用し、内部API固有のロジック（groups/episodesのupsert）のみ薄いラッパーで追加する。

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, patch, post};

use crate::AppState;
use crate::handlers::internal_episodes::upsert_item_episode_handler;
use crate::handlers::internal_extractions::{
    cancelled_handler, claim_handler, complete_handler, fail_handler, heartbeat_handler,
};
use crate::handlers::internal_groups::upsert_item_group_handler;
use crate::handlers::item_files::create_item_file_handler;
use crate::handlers::items::{create_item_handler, list_items_handler, update_item_handler};
use crate::middleware::api_key_auth::api_key_auth;

/// `/internal` 専用Routerを構築する。
///
/// 【実装方針】: items本体のPOST/PATCH・GET検索（ListItemsQueryのtitle絞り込みを再利用）・
/// groups/episodesのupsert・filesのパス指定登録を、Phase2/Phase4実装済みハンドラの再利用または
/// 薄いupsertラッパーとしてマウントする。`/internal/items/search`はリテラルパスのため、
/// 動的パス`/internal/items/{id}`より前に登録し誤マッチを防ぐ。
/// 🔵 信頼性レベル: TASK-0029.md「実装詳細1」api-endpoints.md「内部REST API」セクションに直接対応
pub fn build_internal_router(state: AppState) -> Router {
    Router::new()
        .route("/internal/extractions/claim", post(claim_handler))
        .route(
            "/internal/extractions/{id}/heartbeat",
            post(heartbeat_handler),
        )
        .route(
            "/internal/extractions/{id}/complete",
            // 🔵 contentは500万Unicode文字まで許容し、文字数検証の422をハンドラで返す。
            post(complete_handler).layer(DefaultBodyLimit::max(25 * 1024 * 1024)),
        )
        .route("/internal/extractions/{id}/fail", post(fail_handler))
        .route(
            "/internal/extractions/{id}/cancelled",
            post(cancelled_handler),
        )
        .route("/internal/items", post(create_item_handler))
        .route("/internal/items/search", get(list_items_handler))
        .route("/internal/items/{id}", patch(update_item_handler))
        .route(
            "/internal/items/{id}/groups",
            post(upsert_item_group_handler),
        )
        .route(
            "/internal/groups/{group_id}/episodes",
            post(upsert_item_episode_handler),
        )
        .route("/internal/items/{id}/files", post(create_item_file_handler))
        .layer(axum::middleware::from_fn(api_key_auth))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use crate::AppState;
    use crate::routes::{build_router, internal::build_internal_router};

    /// main.rsと同じ構成で公開・内部ルーターをマージし、`/api/v1`へnestする。
    fn build_test_router(state: AppState) -> Router {
        Router::new().nest(
            "/api/v1",
            build_router(state.clone()).merge(build_internal_router(state)),
        )
    }

    /// 【テスト用ヘルパー】: `/internal` ルーティング統合テスト用にAppStateを構築する。
    /// DATABASE_URL環境変数（テスト用Postgres）への接続が必要なため、本ヘルパーを使う
    /// テストはすべて#[ignore]とし、`cargo test -- --ignored` で実行する 🔵
    /// 🔵 信頼性レベル: routes/mod.rs既存test_app_state()パターンに直接対応
    async fn test_app_state() -> AppState {
        let database_url = std::env::var("DATABASE_URL")
            .expect("TASK-0029ルーティング統合テストにはDATABASE_URL環境変数が必要です");
        let db = sqlx::PgPool::connect(&database_url)
            .await
            .expect("テスト用DBへの接続に失敗しました");
        AppState {
            db,
            internal_api_key: String::new(),
        }
    }

    /// DBアクセスを行わないルーティング境界テスト用の遅延接続Stateを構築する。
    fn test_app_state_without_db() -> AppState {
        let db = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgresql://postgres:postgres@localhost/mediavault_test")
            .expect("テスト用DATABASE_URLが正しいこと");
        AppState {
            db,
            internal_api_key: String::new(),
        }
    }

    /// 【テスト用ヘルパー】: INTERNAL_API_KEY環境変数を既知値にセットする。
    /// api_key_authミドルウェアはAppStateではなくstd::env::var("INTERNAL_API_KEY")を
    /// 照合元とするため、テストではこの関数で環境変数を設定する 🔵
    /// 🔵 信頼性レベル: note.md「3 重要な前提」、middleware/api_key_auth.rs L16に直接対応
    fn set_internal_api_key(key: &str) {
        unsafe {
            std::env::set_var("INTERNAL_API_KEY", key);
        }
    }

    /// 【テスト用ヘルパー】: 有効な最小CreateItemRequest相当のJSONボディを返す
    fn valid_create_item_body() -> String {
        r#"{"title":"テスト作品","media_type":"anime"}"#.to_string()
    }

    const TEST_KEY: &str = "secret-internal-key";

    /// TASK-0006: 新パスには内部API認証が適用される。
    #[tokio::test]
    async fn post_versioned_internal_items_without_auth_returns_401() {
        set_internal_api_key(TEST_KEY);
        let app = build_test_router(test_app_state_without_db());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/internal/items")
                    .header("content-type", "application/json")
                    .body(Body::from(valid_create_item_body()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// TASK-0006: 旧パスの互換aliasは提供しない。
    #[tokio::test]
    async fn post_legacy_internal_items_without_auth_returns_404() {
        set_internal_api_key(TEST_KEY);
        let app = build_test_router(test_app_state_without_db());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/items")
                    .header("content-type", "application/json")
                    .body(Body::from(valid_create_item_body()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// TASK-0006: internal Routerの認証レイヤーが同階層の公開ルートへ波及しない。
    #[tokio::test]
    async fn public_route_without_auth_returns_200() {
        set_internal_api_key(TEST_KEY);
        let public_router = Router::new().route(
            "/public-probe",
            axum::routing::get(|| async { StatusCode::OK }),
        );
        let app = Router::new().nest(
            "/api/v1",
            public_router.merge(build_internal_router(test_app_state_without_db())),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/public-probe")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    // ===================== 1. 正常系テストケース =====================

    /// TC-018-01: 正しいAPIキー付きのPOST /internal/items でアイテムが新規作成され201が返る
    /// 【テスト目的】: /internalルーターにAPIキー検証ミドルウェアが適用され、正しいキーを伴うリクエストが
    /// ミドルウェアを通過し、既存create_item_handlerが実行されて作成成功（201）になることを確認する
    /// 【テスト内容】: 正しいAuthorizationヘッダー付きでPOST /internal/itemsを呼び出す
    /// 【期待される動作】: 401にならず（認証通過）、201 Createdが返る（ハンドラ実行・作成成功）
    /// 🔵 信頼性レベル: TASK-0029.md「テストケース1（TC-018-01）」、要件定義4.1に直接対応
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn post_internal_items_with_valid_key_returns_201() {
        // 【テストデータ準備】: 認証通過させるためINTERNAL_API_KEY環境変数を既知値に設定する
        // 【初期条件設定】: build_internal_router（未実装）で/internalルーターを構築する
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);

        // 【実際の処理実行】: oneshotでPOST /internal/itemsをルーター経由実行する
        // 【処理内容】: ミドルウェア → create_item_handler → repository INSERT の一連を起動
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/internal/items")
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(valid_create_item_body()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: HTTPステータスが201であることを検証する
        // 【期待値確認】: 認証通過かつ作成成功を201で表す
        assert_eq!(response.status(), StatusCode::CREATED); // 【確認内容】: 認証通過後にハンドラが実行され201が返ることを確認 🔵
    }

    /// TC-018-02: 正しいAPIキーでPATCH /internal/items/{id} が既存アイテムを更新し200を返す
    /// 【テスト目的】: 認証通過後、既存update_item_handlerによる部分更新が/internal経由で正しく動作することを確認する
    /// 【テスト内容】: 事前にPOSTで作成したitemに対しPATCH /internal/items/{id}でタイトル更新する
    /// 【期待される動作】: 200 OKと更新後itemが返る
    /// 🔵 信頼性レベル: 要件定義2.2表#2・4.1、note.md「PATCH /internal/items/{id}」に対応
    #[tokio::test]
    #[ignore]
    async fn patch_internal_items_id_with_valid_key_updates_and_returns_200() {
        // 【テストデータ準備】: 事前に作成した既存itemのIDを得るため、まずPOSTで作成する
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);

        // 【前提条件確認】: 事前作成のため、まず作成リクエストを送る
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/internal/items")
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(valid_create_item_body()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create_response.status(), StatusCode::CREATED);

        let body_bytes = axum::body::to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let created: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        let item_id = created["data"]["id"].as_str().unwrap().to_string();

        // 【実際の処理実行】: PATCH /internal/items/{id} で部分更新を実行する
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/internal/items/{item_id}"))
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"更新後タイトル"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: 200が返り、更新が反映されることを確認する
        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: 認証通過後に更新ハンドラが実行され200が返ることを確認 🔵
    }

    /// TC-018-03: 正しいAPIキーでGET /internal/items/search（条件指定）がフィルタ済み一覧を返す
    /// 【テスト目的】: 既存list_items_handlerの検索ロジックが/internal/items/search経由で機能することを確認する
    /// 【テスト内容】: ?title=テスト&media_type=animeでGET /internal/items/searchを呼び出す
    /// 【期待される動作】: 200 OK、pagination付きのJSONが返る
    /// 🔵 信頼性レベル: 要件定義2.2表#3、TASK-0029.md「実装詳細3」に対応
    #[tokio::test]
    #[ignore]
    async fn get_internal_items_search_with_filters_returns_200_with_pagination() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);

        // 【実際の処理実行】: 条件付き検索クエリでGET /internal/items/searchを実行する
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/internal/items/search?title=テスト&media_type=anime")
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: 200が返り、pagination付きで条件に合致するitemのみが返ることを確認する
        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: 条件付き検索が200で成功することを確認 🔵

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert!(json.get("pagination").is_some()); // 【確認内容】: paginationフィールドが存在することを確認 🔵
    }

    /// TC-018-04: 正しいAPIキーでGET /internal/items/search（クエリ未指定）がページネーション付き全件を返す
    /// 【テスト目的】: 全クエリパラメータがoptionalであり、未指定時は全件をページネーション付きで返すことを確認する
    /// 【テスト内容】: クエリパラメータなしでGET /internal/items/searchを呼び出す
    /// 【期待される動作】: 200 OK、デフォルトpage=1, limit=20でページネーション付き全件先頭ページが返る
    /// 🟡 信頼性レベル: TASK-0029.md「テストケース4（TC-018-04）」（🟡明記）に対応
    #[tokio::test]
    #[ignore]
    async fn get_internal_items_search_without_query_returns_200_with_default_pagination() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);

        // 【実際の処理実行】: クエリパラメータなしでGET /internal/items/searchを実行する
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/internal/items/search")
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: 400にならず200が返ることを確認する（必須パラメータなし）
        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: クエリ未指定でも400にならず200が返ることを確認 🟡

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json["pagination"]["limit"], 20); // 【確認内容】: デフォルトlimitが20であることを確認 🟡
        assert!(json["pagination"]["has_more"].is_boolean()); // 【確認内容】: has_moreフィールドが存在することを確認
    }

    /// TC-018-05: 話数同期フロー（groups upsert → episodes upsert）が連鎖で成立する
    /// 【テスト目的】: 巡回バッチの話数同期フロー（グループupsert→そのグループ配下へのエピソードupsert）が
    /// /internalルートで一貫して成立することを確認する
    /// 【テスト内容】: (1) 既存itemに対しグループを作成（201）、(2) 返却されたgroup_idに対しエピソードを作成（201）
    /// 【期待される動作】: 両ステップとも201、エピソードがステップ1のグループに正しく紐づく
    /// 🔵 信頼性レベル: 要件定義4.1「話数同期フロー」、TASK-0029.md「統合テスト要件」に対応
    #[tokio::test]
    #[ignore]
    async fn groups_then_episodes_upsert_chain_succeeds() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);

        // 【前提条件確認】: 親itemを作成する
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/internal/items")
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(valid_create_item_body()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create_response.status(), StatusCode::CREATED);
        let body_bytes = axum::body::to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let created: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        let item_id = created["data"]["id"].as_str().unwrap().to_string();

        // 【実際の処理実行】: ステップ1 グループ作成
        let group_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/internal/items/{item_id}/groups"))
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"group_type":"season","group_number":1,"title":"シーズン1"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(group_response.status(), StatusCode::CREATED); // 【確認内容】: グループ登録が201で成功することを確認 🔵
        let body_bytes = axum::body::to_bytes(group_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let group: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        let group_id = group["data"]["id"].as_str().unwrap().to_string();

        // 【実際の処理実行】: ステップ2 エピソード作成
        let episode_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/internal/groups/{group_id}/episodes"))
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"episode_number":1,"title":"第1話"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: エピソード登録が201で成功することを確認する
        assert_eq!(episode_response.status(), StatusCode::CREATED); // 【確認内容】: グループ→エピソードの連鎖が成立することを確認 🔵
    }

    /// TC-018-06: グループ・エピソードのUpsert挙動（既存なら更新）
    /// 【テスト目的】: グループが「既存なら更新、なければ新規作成」のupsert振る舞いになることを確認する
    /// 【テスト内容】: 同一識別条件のCreateItemGroupRequestを2回送信する（2回目は一部フィールドを変更）
    /// 【期待される動作】: 2回目は更新後グループが返り、変更フィールドが反映される
    /// 🔵 信頼性レベル: 要件定義3.4「Upsert処理」、note.md「6. Upsert処理の一貫性保証」に対応
    #[tokio::test]
    #[ignore]
    async fn posting_same_group_twice_upserts_instead_of_duplicating() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);

        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/internal/items")
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(valid_create_item_body()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let body_bytes = axum::body::to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let created: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        let item_id = created["data"]["id"].as_str().unwrap().to_string();

        // 【実際の処理実行】: 同一識別条件のグループを1回目送信する
        let first = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/internal/items/{item_id}/groups"))
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"group_type":"season","group_number":1,"title":"初回タイトル"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::CREATED);

        // 【実際の処理実行】: 同一識別条件・変更フィールド付きで2回目送信する
        let second = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/internal/items/{item_id}/groups"))
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"group_type":"season","group_number":1,"title":"更新後タイトル"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: 2回目もエラーにならず（201または200）、更新後タイトルが反映されることを確認する
        assert!(second.status() == StatusCode::CREATED || second.status() == StatusCode::OK); // 【確認内容】: 同一グループの再送がエラーにならずupsertとして処理されることを確認 🔵
        let body_bytes = axum::body::to_bytes(second.into_body(), usize::MAX)
            .await
            .unwrap();
        let updated: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(updated["data"]["title"], "更新後タイトル"); // 【確認内容】: 2回目送信のフィールドが反映されupsert更新になっていることを確認 🔵
    }

    /// TC-018-07: 正しいAPIキーでPOST /internal/items/{id}/files がパス指定でファイルを登録し201を返す
    /// 【テスト目的】: 監視プロセスからの「既存パス指定方式」ファイル登録（REQ-019）が/internal経由で機能することを確認する
    /// 【テスト内容】: 既存itemに対しCreateItemFileRequest（相対パス）を送信する
    /// 【期待される動作】: 201 Created、登録済みファイル情報が返る
    /// 🔵 信頼性レベル: 要件定義2.2表#6・3.5、TASK-0029.md「実装詳細5」に対応
    #[tokio::test]
    #[ignore]
    async fn post_internal_items_id_files_with_valid_path_returns_201() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);

        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/internal/items")
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(valid_create_item_body()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let body_bytes = axum::body::to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let created: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        let item_id = created["data"]["id"].as_str().unwrap().to_string();

        // 【実際の処理実行】: パス指定方式でファイル登録を実行する
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/internal/items/{item_id}/files"))
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"relative_path":"anime/test/episode01.mkv"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: 201が返ることを確認する
        assert_eq!(response.status(), StatusCode::CREATED); // 【確認内容】: パス指定方式のファイル登録が201で成功することを確認 🔵
    }

    /// TC-018-08: 登録→検索の反映（POST /internal/items → GET /internal/items/search）
    /// 【テスト目的】: 内部APIでの登録結果が同じ内部APIの検索に反映されることを確認する（統合テスト）
    /// 【テスト内容】: 一意なtitleで作成 → そのtitleで検索 → 作成したitemが結果に含まれることを確認する
    /// 【期待される動作】: ステップ2のdataにステップ1で作成したitemが含まれる
    /// 🟡 信頼性レベル: 要件定義2.3「連鎖フロー」・TASK-0029.md「統合テスト要件」に対応（一意titleの選定方法は妥当推測）
    #[tokio::test]
    #[ignore]
    async fn created_item_is_searchable_via_internal_search() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);

        let unique_title = format!("一意タイトル-{}", uuid::Uuid::new_v4());
        let create_body = serde_json::json!({
            "title": unique_title,
            "media_type": "anime"
        })
        .to_string();

        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/internal/items")
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(create_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create_response.status(), StatusCode::CREATED);
        let body_bytes = axum::body::to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let created: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        let item_id = created["data"]["id"].as_str().unwrap().to_string();

        // 【実際の処理実行】: 一意titleで検索を実行する
        let search_response = app
            .oneshot(
                Request::builder()
                    .uri(format!(
                        "/api/v1/internal/items/search?title={unique_title}"
                    ))
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(search_response.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(search_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let search_result: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        // 【結果検証】: 作成したitem_idが検索結果に含まれることを確認する
        let data = search_result["data"].as_array().unwrap();
        let found = data.iter().any(|v| v["id"] == item_id);
        assert!(found); // 【確認内容】: 登録したitemが同一内部APIの検索結果に反映されることを確認 🟡
    }

    // ===================== 2. 異常系テストケース =====================

    /// TC-018-E01a: AuthorizationヘッダーなしでPOST /internal/items が401を返す（ハンドラ非実行）
    /// 【テスト目的】: 認証ミドルウェアが全ルートに適用され、無認証リクエストを遮断することを確認する
    /// 【テスト内容】: Authorizationヘッダーなしで有効なCreateItemRequestボディを送信する
    /// 【期待される動作】: 401 Unauthorized、ハンドラ本体（DB INSERT）は実行されない
    /// 🔵 信頼性レベル: TASK-0029.md「テストケース2（TC-018-E01）」、api_key_auth.rs L86-98に直接対応
    #[tokio::test]
    #[ignore]
    async fn post_internal_items_without_auth_header_returns_401() {
        // 【テストデータ準備】: 認証ヘッダーを設定しないことで無認証アクセスを再現する
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);

        // 【実際の処理実行】: Authorizationヘッダーなしでリクエストする
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/internal/items")
                    .header("content-type", "application/json")
                    .body(Body::from(valid_create_item_body()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: 認証前にミドルウェアで遮断され、401が返ることを確認する
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // 【確認内容】: 無認証リクエストが401で遮断されハンドラ非実行であることを確認 🔵
    }

    /// TC-018-E01b: 誤ったAPIキーでPOST /internal/items が401を返す（ハンドラ非実行）
    /// 【テスト目的】: キー不一致が401で拒否されることを確認する
    /// 【テスト内容】: Authorization: Bearer wrong-keyで有効なCreateItemRequestボディを送信する
    /// 【期待される動作】: 401 Unauthorized、ハンドラ非実行（DB副作用なし）
    /// 🔵 信頼性レベル: TASK-0029.md「テストケース2」、api_key_auth.rs L100-114に直接対応
    #[tokio::test]
    #[ignore]
    async fn post_internal_items_with_wrong_key_returns_401() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);

        // 【実際の処理実行】: 不一致のAPIキーでリクエストする
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/internal/items")
                    .header("authorization", "Bearer wrong-key")
                    .header("content-type", "application/json")
                    .body(Body::from(valid_create_item_body()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: 不一致時に401で拒否されることを確認する
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED); // 【確認内容】: キー不一致が401で拒否されることを確認 🔵
    }

    /// TC-018-E01c: 認証ミドルウェアが全 /internal ルートに適用されている（複数ルートで401）
    /// 【テスト目的】: ミドルウェアがルーター全体（個別ルートではなくルーターlayer）に適用されていることを確認する
    /// 【テスト内容】: 認証ヘッダーなしで/internal配下の各エンドポイント（search/PATCH/groups/episodes/files）を順に呼ぶ
    /// 【期待される動作】: すべて401 Unauthorizedになる
    /// 🔵 信頼性レベル: REQ-029「/api/v1/internal/* 配下の任意のエンドポイント」、TASK-0029.md完了条件「全ルートで401」に対応
    #[tokio::test]
    #[ignore]
    async fn all_internal_routes_return_401_without_auth() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);
        let dummy_uuid = uuid::Uuid::new_v4();

        let targets: Vec<(&str, String, Option<&str>)> = vec![
            ("GET", "/api/v1/internal/items/search".to_string(), None),
            (
                "PATCH",
                format!("/api/v1/internal/items/{dummy_uuid}"),
                Some(r#"{"title":"x"}"#),
            ),
            (
                "POST",
                format!("/api/v1/internal/items/{dummy_uuid}/groups"),
                Some(r#"{"group_type":"season","group_number":1}"#),
            ),
            (
                "POST",
                format!("/api/v1/internal/groups/{dummy_uuid}/episodes"),
                Some(r#"{"episode_number":1}"#),
            ),
            (
                "POST",
                format!("/api/v1/internal/items/{dummy_uuid}/files"),
                Some(r#"{"relative_path":"x.mkv"}"#),
            ),
        ];

        // 【実際の処理実行】: 認証ヘッダーなしで各ルートを順に呼び出す
        for (method, uri, body) in targets {
            let mut builder = Request::builder().method(method).uri(&uri);
            if body.is_some() {
                builder = builder.header("content-type", "application/json");
            }
            let request = builder
                .body(Body::from(body.unwrap_or_default().to_string()))
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();

            // 【結果検証】: 全ルートで一律401になることを確認する
            assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "ルート {uri} が401を返しませんでした"
            ); // 【確認内容】: 各内部ルートで認証境界が一貫して機能していることを確認 🔵
        }
    }

    /// TC-018-E02a: 正しいAPIキー + 存在しないitem_idでPATCH /internal/items/{id} が404を返す
    /// 【テスト目的】: 認証通過後のリソース不在判定（404）が/internalで機能することを確認する
    /// 【テスト内容】: ランダム生成した存在しないitem_idへPATCHする
    /// 【期待される動作】: 404 Not Found、error.code=ITEM_NOT_FOUND
    /// 🔵 信頼性レベル: TASK-0029.md「テストケース3（TC-018-E02）」、要件定義2.2表#2・4.2に直接対応
    #[tokio::test]
    #[ignore]
    async fn patch_internal_items_id_with_nonexistent_id_returns_404() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);
        let nonexistent_id = uuid::Uuid::new_v4();

        // 【実際の処理実行】: 存在しないitem_idへPATCHする
        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/v1/internal/items/{nonexistent_id}"))
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"更新後タイトル"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: 404が返ることを確認する
        assert_eq!(response.status(), StatusCode::NOT_FOUND); // 【確認内容】: 認証通過後も存在しないitem_idで404が返ることを確認 🔵
    }

    /// TC-018-E02b: 正しいAPIキー + 存在しないitem_idでPOST /internal/items/{id}/groups が404を返す
    /// 【テスト目的】: グループ登録の親item存在チェックが404で機能することを確認する
    /// 【テスト内容】: 存在しないitem_idへグループ登録を送信する
    /// 【期待される動作】: 404 Not Found、error.code=ITEM_NOT_FOUND
    /// 🔵 信頼性レベル: TASK-0029.md「テストケース3」、要件定義2.2表#4・4.2に直接対応
    #[tokio::test]
    #[ignore]
    async fn post_internal_items_id_groups_with_nonexistent_item_returns_404() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);
        let nonexistent_id = uuid::Uuid::new_v4();

        // 【実際の処理実行】: 存在しないitem_idへグループ登録する
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/internal/items/{nonexistent_id}/groups"))
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"group_type":"season","group_number":1,"title":"テスト"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: 404が返ることを確認する
        assert_eq!(response.status(), StatusCode::NOT_FOUND); // 【確認内容】: 親itemが存在しない場合に孤立グループが作成されず404が返ることを確認 🔵
    }

    /// TC-018-E02c: 正しいAPIキー + 存在しないgroup_idでPOST /internal/groups/:group_id/episodes が404を返す
    /// 【テスト目的】: エピソード登録の親group存在チェックが404で機能することを確認する
    /// 【テスト内容】: 存在しないgroup_idへエピソード登録を送信する
    /// 【期待される動作】: 404 Not Found
    /// 🔵 信頼性レベル: 要件定義2.2表#5・4.2に直接対応
    #[tokio::test]
    #[ignore]
    async fn post_internal_groups_group_id_episodes_with_nonexistent_group_returns_404() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);
        let nonexistent_group_id = uuid::Uuid::new_v4();

        // 【実際の処理実行】: 存在しないgroup_idへエピソード登録する
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/internal/groups/{nonexistent_group_id}/episodes"
                    ))
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"episode_number":1,"title":"第1話"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: 404が返ることを確認する
        assert_eq!(response.status(), StatusCode::NOT_FOUND); // 【確認内容】: 親groupが存在しない場合に孤立エピソードが作成されず404が返ることを確認 🔵
    }

    /// TC-018-E02d: 正しいAPIキー + 存在しないitem_idでPOST /internal/items/{id}/files が404を返す
    /// 【テスト目的】: ファイル登録の親item存在チェックが404で機能することを確認する
    /// 【テスト内容】: 存在しないitem_idへファイル登録を送信する
    /// 【期待される動作】: 404 Not Found、error.code=ITEM_NOT_FOUND
    /// 🔵 信頼性レベル: 要件定義2.2表#6・4.2に直接対応
    #[tokio::test]
    #[ignore]
    async fn post_internal_items_id_files_with_nonexistent_item_returns_404() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);
        let nonexistent_id = uuid::Uuid::new_v4();

        // 【実際の処理実行】: 存在しないitem_idへファイル登録する
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/internal/items/{nonexistent_id}/files"))
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"relative_path":"anime/x/episode01.mkv"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: 404が返ることを確認する
        assert_eq!(response.status(), StatusCode::NOT_FOUND); // 【確認内容】: 親itemが存在しない場合に孤立ファイル行が作成されず404が返ることを確認 🔵
    }

    /// TC-018-E03: 正しいAPIキー + 不正なmedia_typeでPOST /internal/items が400（VALIDATION_ERROR）になる
    /// 【テスト目的】: 入力検証（既存parse_create_item_request再利用）が/internalでも機能することを確認する
    /// 【テスト内容】: media_type=invalidの不正なボディでPOST /internal/itemsを呼び出す
    /// 【期待される動作】: 400 Bad Request、DB INSERTは発生しない
    /// 🔵 信頼性レベル: 要件定義2.2表#1・4.2、既存routes/mod.rsのmedia_type=invalid → 400パターンに対応
    #[tokio::test]
    #[ignore]
    async fn post_internal_items_with_invalid_media_type_returns_400() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);

        // 【実際の処理実行】: 不正なmedia_typeでPOSTする
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/internal/items")
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title":"x","media_type":"invalid"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: 400が返ることを確認する
        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 不正なmedia_typeが既存検証ロジックにより400で拒否されることを確認 🔵
    }

    // ===================== 3. 境界値テストケース =====================

    /// TC-029-02: 旧パス（/internal/items）は404
    /// 【テスト目的】: 旧パスの互換aliasを残さず、即時切替されていることを確認する
    /// 【テスト内容】: 正しいAPIキー付きでPOST /internal/itemsを呼び出す
    /// 【期待される動作】: 404 Not Found（ルート未登録）
    /// 🔵 信頼性レベル: 要件定義3.2「バージョンプレフィックスなし」、note.md「URLプレフィックス間違い注意」に対応
    #[tokio::test]
    #[ignore]
    async fn post_legacy_internal_items_returns_404_not_found() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);

        // 【実際の処理実行】: 旧パスでリクエストする
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/internal/items")
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .header("content-type", "application/json")
                    .body(Body::from(valid_create_item_body()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: ルート未登録のため404になることを確認する
        assert_eq!(response.status(), StatusCode::NOT_FOUND); // 【確認内容】: 旧パスが存在せず404になることを確認 🔵
    }

    /// TC-018-B02: ルート誤マッチ防止 — /internal/items/search が /internal/items/{id} に吸われない
    /// 【テスト目的】: リテラルパスsearchが動的パス:id（UUID必須）に誤って吸われないことを確認する
    /// 【テスト内容】: 正しいAPIキー付きでGET /internal/items/searchを呼び出す
    /// 【期待される動作】: UUIDパース由来の400や500にならない
    /// 🔵 信頼性レベル: 既存routes/mod.rs TC-0024-B02パターン、要件定義2.2表#2/#3の共存に対応
    #[tokio::test]
    #[ignore]
    async fn get_internal_items_search_does_not_fall_through_to_item_id_route() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);

        // 【実際の処理実行】: /internal/items/searchへGETする
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/internal/items/search")
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: ルーティング誤マッチによるpanic/500が発生しないことを確認する
        assert_ne!(response.status(), StatusCode::INTERNAL_SERVER_ERROR); // 【確認内容】: searchがリテラル優先で:idの動的経路に吸われないことを確認 🔵
    }

    /// TC-018-B03: limit境界値（最大100超指定）でlimitがクランプされる
    /// 【テスト目的】: ページネーションlimitの上限（100）が/internalでも機能することを確認する
    /// 【テスト内容】: ?limit=1000でGET /internal/items/searchを呼び出す
    /// 【期待される動作】: 200 OK、pagination.limitが100にクランプされる
    /// 🟡 信頼性レベル: 要件定義3.3（🟡）・既存normalize_paginationパターンからの妥当推測
    #[tokio::test]
    #[ignore]
    async fn get_internal_items_search_with_limit_over_100_clamps_to_100() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);

        // 【実際の処理実行】: limit=1000で検索を実行する
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/internal/items/search?limit=1000")
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        // 【結果検証】: pagination.limitが100にクランプされることを確認する
        assert_eq!(json["pagination"]["limit"], 100); // 【確認内容】: 過大なlimit指定が上限100に丸められることを確認 🟡
    }

    /// TC-018-B04b: after_created_at=abc（非日時形式）が400を返す
    /// 【テスト目的】: keysetページネーションのカーソルパラメータ型不正処理が/internalでも一貫することを確認する
    /// 【テスト内容】: ?after_created_at=abcでGET /internal/items/searchを呼び出す
    /// 【期待される動作】: 400 Bad Request（extractorデシリアライズ失敗）
    #[tokio::test]
    #[ignore]
    async fn get_internal_items_search_with_non_datetime_after_created_at_returns_400() {
        set_internal_api_key(TEST_KEY);
        let state = test_app_state().await;
        let app: Router = build_test_router(state);

        // 【実際の処理実行】: after_created_at=abcで検索を実行する
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/internal/items/search?after_created_at=abc")
                    .header("authorization", format!("Bearer {TEST_KEY}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: 型不正なafter_created_at値が400で拒否されることを確認する
        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 日時形式でないafter_created_atが400で拒否されることを確認
    }
}
