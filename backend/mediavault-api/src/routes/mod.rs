//! ルーター構築
//!
//! TASK-0007: Axumルーター骨格・DB接続プール設定・main.rs実装
//! TASK-0029: `/internal` 専用ルーター（internal.rs）を追加 🔵

pub mod internal;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::get;

use crate::AppState;
use crate::handlers::categories::{
    attach_category_handler, create_category_handler, delete_category_handler,
    detach_category_handler, list_categories_handler,
};
use crate::handlers::health::health_handler;
use crate::handlers::import_booklog::import_booklog_handler;
use crate::handlers::import_steam::import_steam_handler;
use crate::handlers::item_episodes::{create_item_episode_handler, list_item_episodes_handler};
use crate::handlers::item_files::{
    create_item_file_handler, update_calibre_link_handler, upload_item_file_handler,
};
use crate::handlers::item_groups::{create_item_group_handler, list_item_groups_handler};
use crate::handlers::item_links::{create_item_link_handler, delete_item_link_handler};
use crate::handlers::item_relations::{create_item_relation_handler, delete_item_relation_handler};
use crate::handlers::item_trailers::{create_item_trailer_handler, delete_item_trailer_handler};
use crate::handlers::items::{
    count_items_by_media_type_handler, create_item_handler, delete_item_handler, get_item_handler,
    import_item_handler, list_items_handler, search_items_handler, update_item_handler,
    update_item_status_handler,
};
use crate::handlers::mylists::{
    add_mylist_item_handler, create_mylist_handler, remove_mylist_item_handler,
};
use crate::handlers::settings::update_api_key_handler;
use crate::handlers::staff::{
    create_item_staff_handler, create_staff_handler, delete_item_staff_handler,
};
use crate::handlers::tags::{
    attach_tag_handler, create_tag_handler, delete_tag_handler, detach_tag_handler,
    list_tags_handler,
};

/// アプリケーション全体のRouterを構築する。
/// Phase 2以降のルートはこのRouterに `.merge()` や `.nest()` で追加していく。
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        // 【TASK-0010】: GET /items（一覧・絞り込み）を既存POST /itemsと同一パスに追加 🔵
        .route("/items", get(list_items_handler).post(create_item_handler))
        // 【TASK-0024】: GET /items/search（外部API検索）を/items/{id}より前にリテラル登録し誤マッチ防止 🔵
        .route("/items/search", get(search_items_handler))
        // 【TASK-0025】: POST /items/import（外部検索結果からのインポート）を/items/{id}より前に
        // リテラル登録し誤マッチ防止 🔵
        .route("/items/import", axum::routing::post(import_item_handler))
        // GET /items/counts-by-media-type（サイドバー用メディア種別件数）を/items/{id}より前に
        // リテラル登録し誤マッチ防止（/items/search・/items/importと同一方針）
        .route(
            "/items/counts-by-media-type",
            get(count_items_by_media_type_handler),
        )
        // 【TASK-0011】: GET /items/{id}（個別詳細取得） 🟡
        // 【TASK-0012】: PATCH /items/{id}（部分更新）を同一パスに追加 🔵
        // 【TASK-0013】: DELETE /items/{id}（カスケード削除）を同一パスに追加 🔵
        .route(
            "/items/{id}",
            get(get_item_handler)
                .patch(update_item_handler)
                .delete(delete_item_handler),
        )
        // 【TASK-0014】: PATCH /items/{id}/status（視聴・読了状況更新専用エンドポイント） 🔵
        .route(
            "/items/{id}/status",
            axum::routing::patch(update_item_status_handler),
        )
        // 【TASK-0015】: タグCRUD・アイテムへの付与・削除 🔵🟡
        // GET /tags（付与件数付き一覧、サイドバー・フィルタチップ表示用）を追加
        .route("/tags", get(list_tags_handler).post(create_tag_handler))
        .route("/tags/{id}", axum::routing::delete(delete_tag_handler))
        .route(
            "/items/{id}/tags/{tag_id}",
            axum::routing::post(attach_tag_handler).delete(detach_tag_handler),
        )
        // 【TASK-0015】: カテゴリCRUD・アイテムへの付与・削除 🔵🟡
        // GET /categories（付与件数付き一覧）を追加
        .route(
            "/categories",
            get(list_categories_handler).post(create_category_handler),
        )
        .route(
            "/categories/{id}",
            axum::routing::delete(delete_category_handler),
        )
        .route(
            "/items/{id}/categories/{category_id}",
            axum::routing::post(attach_category_handler).delete(detach_category_handler),
        )
        // 【TASK-0016】: マイリストCRUD・itemの追加・削除 🔵
        .route("/mylists", axum::routing::post(create_mylist_handler))
        .route(
            "/mylists/{id}/items",
            axum::routing::post(add_mylist_item_handler),
        )
        .route(
            "/mylists/{id}/items/{item_id}",
            axum::routing::delete(remove_mylist_item_handler),
        )
        // 【TASK-0017】: item_relations（関連付け・DLC）CRUD 🔵
        .route(
            "/item-relations",
            axum::routing::post(create_item_relation_handler),
        )
        .route(
            "/item-relations/{id}",
            axum::routing::delete(delete_item_relation_handler),
        )
        // 【TASK-0018】: item_groups（シーズン/巻/章）CRUD 🔵🟡
        .route(
            "/items/{id}/groups",
            axum::routing::post(create_item_group_handler).get(list_item_groups_handler),
        )
        // 【TASK-0019】: item_episodes（season/chapter配下の話数）CRUD + EDGE-101検証 🔵
        .route(
            "/groups/{group_id}/episodes",
            axum::routing::post(create_item_episode_handler).get(list_item_episodes_handler),
        )
        // 【TASK-0020】: スタッフ管理CRUD（staff作成・itemへの紐付け・紐付け削除） 🔵🟡
        .route("/staff", axum::routing::post(create_staff_handler))
        .route(
            "/items/{id}/staff",
            axum::routing::post(create_item_staff_handler),
        )
        .route(
            "/items/{id}/staff/{item_staff_id}",
            axum::routing::delete(delete_item_staff_handler),
        )
        // 【TASK-0026】: item_files（パス指定方式）登録 🔵
        .route(
            "/items/{id}/files",
            axum::routing::post(create_item_file_handler),
        )
        // 【TASK-0027】: POST /items/{id}/files/upload（multipart/form-dataバイナリ直接アップロード）。
        // DefaultBodyLimitのデフォルト（2MB）はPDF等の大容量ファイルには不足するため、
        // このルートに限り100MBへ緩和する 🟡
        .route(
            "/items/{id}/files/upload",
            axum::routing::post(upload_item_file_handler)
                .layer(DefaultBodyLimit::max(100 * 1024 * 1024)),
        )
        // 【TASK-0028】: PATCH /items/{id}/files/{file_id}/calibre-link
        // （Calibre-Web取込完了後のcalibre_book_id紐付け） 🔵
        .route(
            "/items/{id}/files/{file_id}/calibre-link",
            axum::routing::patch(update_calibre_link_handler),
        )
        // 【TASK-0021】: item_links（参考リンク）CRUD 🔵🟡
        .route(
            "/items/{id}/links",
            axum::routing::post(create_item_link_handler),
        )
        .route(
            "/items/{id}/links/{link_id}",
            axum::routing::delete(delete_item_link_handler),
        )
        // 【TASK-0021】: item_trailers（トレーラー動画リンク）CRUD 🔵🟡
        .route(
            "/items/{id}/trailers",
            axum::routing::post(create_item_trailer_handler),
        )
        .route(
            "/items/{id}/trailers/{trailer_id}",
            axum::routing::delete(delete_item_trailer_handler),
        )
        // 【TASK-0022】: api_credentials（外部APIキー管理）PUT /settings/api-keys/{provider} 🔵
        .route(
            "/settings/api-keys/{provider}",
            axum::routing::put(update_api_key_handler),
        )
        // 【TASK-0030】: POST /import/booklog（ブクログCSVインポート）。/items/search・/items/import等の
        // 既存リテラルパス群と同様にトップレベルの専用リテラルパスとして登録する 🔵
        .route(
            "/import/booklog",
            axum::routing::post(import_booklog_handler),
        )
        // 【TASK-0031】: POST /import/steam（Steamライブラリインポート）。既存/import/booklogと
        // 同様にトップレベルの専用リテラルパスとして登録する 🔵
        .route("/import/steam", axum::routing::post(import_steam_handler))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    /// 【テスト用ヘルパー】: GET /itemsルーティング統合テスト用にAppStateを構築する。
    /// DATABASE_URL環境変数（テスト用Postgres）への接続が必要なため、本ヘルパーを使う
    /// テストはすべて#[ignore]とし、`cargo test -- --ignored` で実行する 🟡
    async fn test_app_state() -> AppState {
        let database_url = std::env::var("DATABASE_URL")
            .expect("TASK-0010ルーティング統合テストにはDATABASE_URL環境変数が必要です");
        let db = sqlx::PgPool::connect(&database_url)
            .await
            .expect("テスト用DBへの接続に失敗しました");
        AppState {
            db,
            internal_api_key: String::new(),
        }
    }

    /// TC-0010-E01: 不正なmedia_type値 → 400（ルーター経由）
    /// 🔵 信頼性レベル: 要件 EC-1・TASK-0010 注意事項に直接対応
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn get_items_with_invalid_media_type_returns_400() {
        // 【テスト目的】: media_type=invalidのようなenum外の値がAxumのQuery抽出段階で
        // 400 Bad Requestとして拒否されることを確認する
        // 【テスト内容】: GET /items?media_type=invalid をルーター経由で実行する
        // 【期待される動作】: レスポンスステータスが400であること
        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/items?media_type=invalid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 不正なenum値が400で拒否されることを確認 🔵
    }

    /// TC-0010-E02: 不正なafter_created_at値（非日時形式）→ 400（ルーター経由）
    /// 🔵 信頼性レベル: 要件 EC-1・note.md 6章 技術的制約に直接対応
    #[tokio::test]
    #[ignore]
    async fn get_items_with_non_datetime_after_created_at_returns_400() {
        // 【テスト目的】: after_created_at=abcのようなNaiveDateTimeにパースできない値が400で拒否されることを確認する
        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/items?after_created_at=abc")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 日時形式でないafter_created_atが400で拒否されることを確認
    }

    /// TC-0010-E03: 不正なis_favorite値（bool以外）→ 400（ルーター経由）
    /// 🟡 信頼性レベル: 要件 入力仕様表（is_favorite: true/false）からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn get_items_with_non_boolean_is_favorite_returns_400() {
        // 【テスト目的】: is_favorite=yesのようなbool以外の値が400で拒否されることを確認する
        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/items?is_favorite=yes")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: bool以外のis_favoriteが400で拒否されることを確認 🟡
    }

    /// TC-0024-E04: 必須qパラメータ欠落で400（ルーター経由、実DB必要）
    /// 🟡 信頼性レベル: 要件 4.2「必須パラメータ欠落 → 400 VALIDATION_ERROR」より妥当推測
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn get_items_search_with_missing_q_returns_400() {
        // 【テスト目的】: qが無い?media_type=animeのみのリクエストで、Axum Query<ItemSearchQuery>
        // extractorがデシリアライズに失敗し400を返すことを確認する
        // 【テスト内容】: GET /items/search?media_type=anime をルーター経由で実行する
        // 【期待される動作】: レスポンスステータスが400であること
        // 🟡 信頼性レベル: 要件 4.2「必須パラメータ欠落 → 400 VALIDATION_ERROR」より妥当推測
        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/items/search?media_type=anime")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: q欠落が400で拒否されることを確認 🟡
    }

    /// TC-0024-E05: 必須media_typeパラメータ欠落で400（ルーター経由、実DB必要）
    /// 🟡 信頼性レベル: 要件 2.1（media_type 必須）・4.2 より妥当推測
    #[tokio::test]
    #[ignore]
    async fn get_items_search_with_missing_media_type_returns_400() {
        // 【テスト目的】: media_typeが無い?q=fooのみのリクエストでextractorが拒否し400を返すことを確認する
        // 【テスト内容】: GET /items/search?q=foo をルーター経由で実行する
        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/items/search?q=foo")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: media_type欠落が400で拒否されることを確認 🟡
    }

    /// TC-0024-E06: 無効なmedia_type値で400（ルーター経由、実DB必要）
    /// 🔵 信頼性レベル: 要件 4.2「media_type 不正値 → 400 VALIDATION_ERROR」・routes/mod.rs既存パターンより
    #[tokio::test]
    #[ignore]
    async fn get_items_search_with_invalid_media_type_returns_400() {
        // 【テスト目的】: enumに存在しないmedia_type=invalidでextractorがenumデシリアライズに失敗し
        // 400を返すことを確認する（既存GET /itemsのmedia_type=invalid→400と同一方針）
        // 【テスト内容】: GET /items/search?media_type=invalid&q=foo をルーター経由で実行する
        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/items/search?media_type=invalid&q=foo")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 不正なenum値が400で拒否されることを確認 🔵
    }

    /// TC-0024-B02: ルート誤マッチ防止 — /items/searchが/items/{id}に吸われない（ルーター経由、実DB必要）
    /// 🔵 信頼性レベル: 要件 3 ルーティング制約・note.md L47-50・L192 より
    #[tokio::test]
    #[ignore] // 実DB・外部API（wiremock）が必要
    async fn get_items_search_does_not_fall_through_to_item_id_route() {
        // 【テスト目的】: /items/searchがリテラルパスとして登録され、/items/{id}（UUID必須）の
        // 動的経路に誤って吸われない（"search"という文字列がUUIDとしてパースされず400 UUIDエラーにならない）
        // ことを確認する
        // 【テスト内容】: GET /items/search?media_type=anime&q=foo をルーター経由で実行する
        // 【期待される動作】: UUID形式エラーに起因する400（parse_item_id由来）にはならない。
        // search_items_handler側のロジック結果（200系、または別要因の400/422/502）に到達する
        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/items/search?media_type=anime&q=foo")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: /items/{id}経由のUUIDパースエラー（"idはUUID形式である必要があります"）には
        // ならないことを、最低限「サーバーエラーで落ちない（500ではない）」ことで確認する
        assert_ne!(response.status(), StatusCode::INTERNAL_SERVER_ERROR); // 【確認内容】: ルーティング誤マッチによるpanic/500が発生しないことを確認 🔵
    }

    /// TC-0011-E01: 存在しないitemで404（ルーター経由、実DB必要）
    /// 🔵 信頼性レベル: タスクファイル テストケース2に直接対応
    #[tokio::test]
    #[ignore]
    async fn get_item_with_nonexistent_id_returns_404() {
        let state = test_app_state().await;
        let app = build_router(state);
        let id = uuid::Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/items/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND); // 【確認内容】: 存在しないitemで404が返ることを確認 🔵
    }

    /// TC-0011-E02: 不正なUUID形式で400（ルーター経由、実DB必要）
    /// 🟡 信頼性レベル: タスクファイル テストケース3に直接対応
    #[tokio::test]
    #[ignore]
    async fn get_item_with_invalid_uuid_returns_400() {
        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/items/not-a-uuid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 不正なUUID形式が400で拒否されることを確認 🟡
    }

    /// TASK-0013: 存在しないitemへのDELETEで404（ルーター経由、実DB必要）
    /// 🔵 信頼性レベル: タスクファイル テストケース2（存在しないitemで404）に直接対応
    #[tokio::test]
    #[ignore]
    async fn delete_item_with_nonexistent_id_returns_404() {
        let state = test_app_state().await;
        let app = build_router(state);
        let id = uuid::Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/items/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND); // 【確認内容】: 存在しないitemのDELETEで404が返ることを確認 🔵
    }

    /// TASK-0013: 不正なUUID形式のDELETEで400（ルーター経由）
    /// 🟡 信頼性レベル: 既存parse_item_id再利用方針からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn delete_item_with_invalid_uuid_returns_400() {
        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/items/not-a-uuid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 不正なUUID形式が400で拒否されることを確認 🟡
    }

    /// 統合テスト共通: 指定providerの行をDBから削除しクリーンな状態にする（TASK-0022）
    async fn cleanup_provider(state: &AppState, provider: &str) {
        sqlx::query("DELETE FROM api_credentials WHERE provider = $1::api_provider")
            .bind(provider)
            .execute(&state.db)
            .await
            .expect("テスト前クリーンアップに失敗しました");
    }

    /// TC-015-01-C: `PUT /settings/api-keys/tmdb`が新規登録時に200と更新後`ApiCredential`を返す（ハンドラ統合）
    /// 🔵 信頼性レベル: 要件定義書REQ-0022-05・NFR-0022-01、完了基準L126・129より
    #[tokio::test]
    #[ignore]
    async fn put_api_key_tmdb_with_new_record_returns_200() {
        // 【テスト目的】: ルーティング→ハンドラ→リポジトリのフルパスで、新規登録時のHTTPレスポンスが規約通りかを確認する
        // 【テスト内容】: PUT /settings/api-keys/tmdb に { "api_key": "valid-tmdb-key" } を送信する
        // 【期待される動作】: HTTPステータス200、ボディがApiOk<ApiCredential>形式
        // 🔵 信頼性レベル: 要件定義書REQ-0022-05・NFR-0022-01、完了基準L126・129より
        let state = test_app_state().await;
        cleanup_provider(&state, "tmdb").await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/settings/api-keys/tmdb")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"api_key":"valid-tmdb-key"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK); // 【確認内容】: 新規登録時にHTTPステータス200が返ることを確認 🔵
    }

    /// TC-015-02-B: 不正provider指定でHTTP 400 INVALID_PROVIDERを返し、DBへ書き込まない（ハンドラ統合）
    /// 🔵 信頼性レベル: 要件定義書シナリオ2・REQ-0022-101、タスクファイルテストケース2・api-endpoints.md L388より
    #[tokio::test]
    #[ignore]
    async fn put_api_key_with_unknown_provider_returns_400_and_no_db_write() {
        // 【テスト目的】: HTTPレベルで不正providerリクエストを受け、エラー応答とDB無変更の両方を確認する
        // 【テスト内容】: PUT /settings/api-keys/unknown_provider に { "api_key": "x" } を送信する
        // 【期待される動作】: HTTPステータス400、ApiErrorCode::InvalidProvider、かつDBに行が作成されない
        // 🔵 信頼性レベル: 要件定義書シナリオ2・REQ-0022-101、api-endpoints.md L388より
        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/settings/api-keys/unknown_provider")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"api_key":"x"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 不正providerが400 INVALID_PROVIDERで拒否されることを確認 🔵
    }

    /// TC-NEW-02: `jikan`は`ApiProvider`に含まれずINVALID_PROVIDERになる（ハンドラ統合部分）
    /// 🔵 信頼性レベル: 要件定義書EDGE-0022-01・「対象外」L24、note.md L6・タスクファイル注意事項L98より
    #[tokio::test]
    #[ignore]
    async fn put_api_key_jikan_returns_400_invalid_provider() {
        // 【テスト目的】: キー不要のためenum対象外とされたjikanを明示的に検証する
        // 【テスト内容】: PUT /settings/api-keys/jikan に { "api_key": "x" } を送信する
        // 【期待される動作】: HTTPステータス400 INVALID_PROVIDER、DB無変更
        // 🔵 信頼性レベル: 要件定義書EDGE-0022-01・「対象外」L24より
        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/settings/api-keys/jikan")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"api_key":"x"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: jikanがINVALID_PROVIDERとして400で拒否されることを確認 🔵
    }

    /// TC-NEW-04: `api_key`欠落リクエストでボディパースエラー（ハンドラ統合）
    /// 🟡 信頼性レベル: 要件定義書EDGE-0022-02（DTO必須フィールドからの妥当な推測）より
    #[tokio::test]
    #[ignore]
    async fn put_api_key_with_missing_api_key_field_returns_400() {
        // 【テスト目的】: 必須フィールドapi_keyが欠落したリクエストボディの処理を確認する
        // 【テスト内容】: PUT /settings/api-keys/tmdb に {}（api_keyキーなし）を送信する
        // 【期待される動作】: axumのJsonエクストラクタ/既存deserialize_requestによるデシリアライズ失敗で400系レスポンス、DB無変更
        // 🟡 信頼性レベル: 要件定義書EDGE-0022-02からの妥当な推測。期待値は既存ボディパースエラー処理（VALIDATION_ERROR想定）に従う
        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/settings/api-keys/tmdb")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: api_key欠落時に400系（VALIDATION_ERROR想定）で拒否されることを確認 🟡
    }
}
