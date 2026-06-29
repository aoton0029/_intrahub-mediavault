//! TASK-0032: 主要フロー統合テスト — 外部API検索→インポートフロー（IT-003）
//!
//! 🟡 信頼性レベル: main-flow-integration-test-testcases.md IT-003（acceptance-criteria.md
//! TC-002-01/02/03、note.mdのwiremock/with_test_base_urlsパターンベース）

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::{build_full_router, test_app_state};

/// IT-003: 外部API検索（モック）→インポート（source=api確認）
///
/// 【テスト目的】: `wiremock`でJikan APIをスタブ化し、`GET /items/search`の結果を
/// `POST /items/import`に渡してitemが作成され、`source=api`・`external_id`一致が
/// 確認できることを検証する（テストケース定義書IT-003）。
///
/// 【実装できなかった理由・調査結果】:
/// `backend/mediavault-api/src/handlers/items.rs` の `search_items_handler` は
/// `ExternalSearchService::new(state.db.clone())` をハンドラ内部で都度構築しており
/// （L220）、`AppState`はテスト用ベースURLや固定認証情報を注入する経路を一切持たない。
/// 一方、`backend/mediavault-api/src/services/external_search.rs` が提供する
/// テスト用DI口（`with_fixed_credentials`・`with_test_base_urls`）は、いずれも
/// `#[cfg(test)]`属性付きの**非pub（プライベート）メソッド**として定義されている
/// （L176-192）。これらは`external_search.rs`自身の`#[cfg(test)] mod tests`からのみ
/// 呼び出し可能であり、別クレートのリンク単位として扱われる`tests/`配下の統合テスト
/// （本ファイル）からは、`pub`でない上に`#[cfg(test)]`が外部クレートのビルドでは
/// 有効にならないため、原理的に呼び出せない。
/// すなわち、wiremockの`MockServer`URLをハンドラ経由の`GET /items/search`へ注入する
/// テスト用シームが現在のプロダクションコードに存在しない。
///
/// 【対応方針】: タスク指示（「ハンドラの修正が必要そうなら推測で実装せず、本テストに
/// `#[ignore = "..."]`を付けてギャップを明記する」）に従い、本テストはシナリオの記述のみ
/// 残し、実行不能（恒久的ignore）とする。ハンドラまたはAppStateにテスト注入用の
/// コンストラクタ（例: `AppState`に`ExternalSearchService`生成用のフックを持たせる、
/// または`with_test_base_urls`等を`pub(crate)`化し統合テストから到達可能にする）を
/// 追加するプロダクションコード変更が必要であり、これはTASK-0032（統合テストのみ）の
/// スコープ外と判断し、本タスクでは実施しない。
///
/// 🟡 信頼性レベル: main-flow-integration-test-testcases.md IT-003、
/// services/external_search.rs L136-192（テスト専用DIが非pub）の実装確認に基づく
#[tokio::test]
#[ignore = "no test seam for base URL override in handler yet: \
    search_items_handler (handlers/items.rs L220) constructs ExternalSearchService::new(state.db.clone()) \
    directly, and ExternalSearchService::with_test_base_urls / with_fixed_credentials \
    (services/external_search.rs L176-192) are private #[cfg(test)] methods unreachable from \
    an external tests/ integration test crate. A production-code seam (e.g. exposing a pub(crate) \
    test constructor, or injecting ExternalSearchService via AppState) would be required to wire \
    a wiremock MockServer into GET /items/search from here; that is out of scope for TASK-0032."]
async fn it_003_external_search_mock_then_import_creates_item_with_source_api() {
    // 【テストデータ準備】: Jikan検索APIの固定レスポンス（mal_id=12345等）をwiremockでスタブ化する想定 🟡
    let jikan_mock = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{ "mal_id": 12345, "title": "鬼滅の刃" }]
        })))
        .mount(&jikan_mock)
        .await;

    // 【初期条件設定】: 本来はExternalSearchServiceのベースURLをjikan_mock.uri()へ差し替えたいが、
    // 現状のプロダクションコードには注入経路が存在しないため、通常のAppStateで構築する 🟡
    let state = test_app_state().await;
    let app = build_full_router(state);

    // 【実際の処理実行】: GET /items/search を実行する（モック未注入のため実Jikan APIへ到達してしまう経路） 🟡
    let search_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/items/search?media_type=anime&q=鬼滅の刃")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(search_response.status(), StatusCode::OK);

    // 【実際の処理実行】: 検索結果から得たexternal_idでPOST /items/importを実行する 🟡
    let import_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/items/import")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "media_type": "anime",
                        "external_id": "12345",
                        "title": "鬼滅の刃"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // 【結果検証】: 201・source="api"・external_id="12345"であることを確認する想定 🟡
    assert_eq!(import_response.status(), StatusCode::CREATED); // 【確認内容】: インポートが201で成功することを確認 🟡
    let bytes = axum::body::to_bytes(import_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["data"]["source"], "api"); // 【確認内容】: インポート経由のsourceがapiであることを確認 🟡
    assert_eq!(json["data"]["external_id"], "12345"); // 【確認内容】: external_idが検索結果のIDと一致することを確認 🟡
}
