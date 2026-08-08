//! TASK-0032: 主要フロー統合テスト — items基本フロー（IT-001, IT-002, IT-010, IT-011）
//!
//! 手動追加→一覧取得→詳細取得→PATCH部分更新→DELETE削除→関連テーブルカスケード削除確認の
//! 一連シナリオを、実DB + `tower::ServiceExt::oneshot`によるルーター経由E2Eで検証する。
//! 🔵 信頼性レベル: main-flow-integration-test-testcases.md IT-001/IT-002/IT-010/IT-011に直接対応

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use common::{build_full_router, test_app_state};

/// 【テスト用ヘルパー】: ApiOkレスポンスボディをJSON Valueへ変換する
async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("レスポンスボディの読み取りに失敗しました");
    serde_json::from_slice(&bytes).expect("レスポンスボディのJSONパースに失敗しました")
}

/// IT-001: 手動追加→一覧取得→詳細取得（source=manual確認）
/// 【テスト目的】: POST /itemsで作成したアイテムが、GET /items一覧およびGET /items/:id詳細の
/// 両方に現れ、source="manual"・external_id=nullであることを確認する
/// 【テスト内容】: {"media_type":"anime","title":"テストアニメ1"}（必須項目のみ）でPOST /itemsを実行し、
/// 続けてGET /itemsとGET /items/:idを実行する
/// 【期待される動作】: POST→201、一覧に対象IDが含まれる、詳細がsource="manual"/external_id=null
/// 🔵 信頼性レベル: main-flow-integration-test-testcases.md IT-001（acceptance-criteria.md TC-001-01ベース）
#[tokio::test]
#[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
async fn it_001_manual_create_then_list_and_get_detail_shows_manual_source() {
    // 【テストデータ準備】: 必須項目のみの最小構成リクエスト（TC-001-01相当） 🔵
    let state = test_app_state().await;
    let app = build_full_router(state);

    // 【実際の処理実行】: POST /items で手動作成する
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/items")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"media_type":"anime","title":"テストアニメ1"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED); // 【確認内容】: 手動作成が201で成功することを確認 🔵
    let created = body_json(create_response).await;
    let item_id = created["data"]["id"].as_str().unwrap().to_string();

    // 【実際の処理実行】: GET /items 一覧取得を実行する
    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/items")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = body_json(list_response).await;
    let list_data = list_json["data"].as_array().unwrap();
    assert!(
        list_data.iter().any(|v| v["id"] == item_id),
        "作成したitemが一覧に含まれること"
    ); // 【確認内容】: 作成済みアイテムが一覧に含まれることを確認 🔵

    // 【実際の処理実行】: GET /items/:id 詳細取得を実行する
    let detail_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/items/{item_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail_response.status(), StatusCode::OK);
    let detail_json = body_json(detail_response).await;

    // 【結果検証】: source="manual"・external_id=nullであることを確認する 🔵
    assert_eq!(detail_json["data"]["source"], "manual"); // 【確認内容】: 手動作成のsourceがmanualであることを確認 🔵
    assert!(detail_json["data"]["external_id"].is_null()); // 【確認内容】: 手動作成のexternal_idがnullであることを確認 🔵
}

/// IT-002: PATCH部分更新→DELETE削除→関連テーブルカスケード削除確認
/// 【テスト目的】: 作成したアイテムをPATCHで更新後、DELETEで削除し、item_tags等の関連レコードが
/// 連動して削除されることを確認する
/// 【テスト内容】: タグを1件アタッチ済みのitemに対しPATCH {"rating":4.5,"is_favorite":true}を実行後、
/// DELETEを実行し、item_tagsへの直接SELECTで0件であることを確認する
/// 【期待される動作】: PATCH→200で更新後値、DELETE→204、削除後GET→404、item_tags SELECTが0件
/// 🔵 信頼性レベル: main-flow-integration-test-testcases.md IT-002（acceptance-criteria.md TC-001-02/03ベース）
#[tokio::test]
#[ignore]
async fn it_002_patch_then_delete_cascades_related_tables() {
    let state = test_app_state().await;
    let app = build_full_router(state.clone());

    // 【テストデータ準備】: 削除対象アイテムを作成する
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/items")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"media_type":"anime","title":"カスケード削除テスト"})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let created = body_json(create_response).await;
    let item_id = created["data"]["id"].as_str().unwrap().to_string();

    // 【前提条件設定】: タグを作成し、アイテムへアタッチする（カスケード削除確認のための関連データ） 🔵
    let tag_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tags")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"name": format!("IT002タグ-{item_id}")}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tag_response.status(), StatusCode::CREATED);
    let tag_created = body_json(tag_response).await;
    let tag_id = tag_created["data"]["id"].as_str().unwrap().to_string();

    let attach_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/items/{item_id}/tags/{tag_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(attach_response.status(), StatusCode::CREATED); // 【確認内容】: タグのアタッチが成功することを確認 🔵

    // 【実際の処理実行】: PATCH /items/:id で部分更新する
    let patch_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/items/{item_id}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"rating": 4.5, "is_favorite": true}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(patch_response.status(), StatusCode::OK); // 【確認内容】: PATCHが200で成功することを確認 🔵
    let patched = body_json(patch_response).await;
    assert_eq!(patched["data"]["rating"], 4.5); // 【確認内容】: ratingが更新値になることを確認 🔵
    assert_eq!(patched["data"]["is_favorite"], true); // 【確認内容】: is_favoriteが更新値になることを確認 🔵

    // 【実際の処理実行】: DELETE /items/:id で削除する
    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/items/{item_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT); // 【確認内容】: DELETEが204で成功することを確認 🔵

    // 【結果検証】: 削除後のGETが404になることを確認する 🔵
    let item_uuid: uuid::Uuid = item_id.parse().unwrap();
    let get_after_delete = app
        .oneshot(
            Request::builder()
                .uri(format!("/items/{item_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_after_delete.status(), StatusCode::NOT_FOUND); // 【確認内容】: 削除後のGETが404になることを確認 🔵

    // 【結果検証】: item_tagsへの直接SELECTで関連レコードが0件であることを確認する（カスケード削除確認・最重要） 🔵
    let remaining_tags: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM item_tags WHERE item_id = $1")
            .bind(item_uuid)
            .fetch_one(&state.db)
            .await
            .expect("item_tags件数取得に失敗しました");
    assert_eq!(remaining_tags, 0); // 【確認内容】: アイテム削除に伴いitem_tagsもカスケード削除され0件であることを確認 🔵
}

/// IT-010: 全フィールドNoneでのPATCH（変更なし確認）
/// 【テスト目的】: PATCH更新で更新対象フィールドを一切指定しない場合、何も更新せず現在の状態を
/// 返す既存仕様（note.md TASK-0012）が統合経路でも保たれることを確認する
/// 【テスト内容】: 既存itemに対しPATCH {}（空オブジェクト）を実行する
/// 【期待される動作】: 200、更新前と同じ内容のitemが返る
/// 🟡 信頼性レベル: main-flow-integration-test-testcases.md IT-010（note.md記載の既存実装仕様からの妥当な推測）
#[tokio::test]
#[ignore]
async fn it_010_patch_with_all_fields_none_returns_unchanged_item() {
    let state = test_app_state().await;
    let app = build_full_router(state);

    // 【テストデータ準備】: 更新確認対象のアイテムを作成する
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/items")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"media_type":"anime","title":"no-op PATCH確認用"})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::CREATED);
    let created = body_json(create_response).await;
    let item_id = created["data"]["id"].as_str().unwrap().to_string();
    let title_before = created["data"]["title"].clone();
    let updated_at_before = created["data"]["updated_at"].clone();

    // 【実際の処理実行】: 空オブジェクトでPATCHを実行する（更新対象フィールドなし） 🟡
    let patch_response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/items/{item_id}"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // 【結果検証】: 200が返り、内容が変化していないことを確認する 🟡
    assert_eq!(patch_response.status(), StatusCode::OK); // 【確認内容】: 全フィールドNoneのPATCHが200で成功することを確認 🟡
    let patched = body_json(patch_response).await;
    assert_eq!(patched["data"]["title"], title_before); // 【確認内容】: titleが変化していないことを確認 🟡
    assert_eq!(patched["data"]["updated_at"], updated_at_before); // 【確認内容】: updated_atが変化していない（UPDATE未実行）ことを確認 🟡
}

/// IT-011: 削除後の再GET（境界: 存在しないID）
/// 【テスト目的】: DELETE直後のIDという「ちょうど消えた直後」の境界状態でのエラーコードの
/// 一貫性を確認する
/// 【テスト内容】: アイテムを作成・削除した直後の同IDに対しGET /items/:idを実行する
/// 【期待される動作】: 404 ITEM_NOT_FOUND
/// 🔵 信頼性レベル: main-flow-integration-test-testcases.md IT-011（acceptance-criteria.md TC-001-E02ベース）
#[tokio::test]
#[ignore]
async fn it_011_get_immediately_after_delete_returns_404_item_not_found() {
    let state = test_app_state().await;
    let app = build_full_router(state);

    // 【テストデータ準備】: 削除対象アイテムを作成する
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/items")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"media_type":"anime","title":"削除直後境界確認用"})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let created = body_json(create_response).await;
    let item_id = created["data"]["id"].as_str().unwrap().to_string();

    // 【前提条件確認】: 直前にDELETEを実行する
    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/items/{item_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    // 【実際の処理実行】: 削除直後の同IDへGETを実行する
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/items/{item_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // 【結果検証】: 404・ITEM_NOT_FOUNDであることを確認する 🔵
    assert_eq!(response.status(), StatusCode::NOT_FOUND); // 【確認内容】: 削除直後のGETが404になることを確認 🔵
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "ITEM_NOT_FOUND"); // 【確認内容】: エラーコードがITEM_NOT_FOUNDであることを確認 🔵
}

/// TASK-0003 TC1/TC2/TC6: include_total指定時のみpagination.totalが返る
/// 【テスト目的】: `include_total=true`指定時のみtotalが返り、未指定・falseでは省略されることを確認する
/// 🔵 信頼性レベル: TASK-0003完了条件・単体テストTC1/TC2/TC6に直接対応
#[tokio::test]
#[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
async fn task_0003_include_total_returns_total_only_when_requested() {
    let state = test_app_state().await;
    let app = build_full_router(state);

    // 【テストデータ準備】: is_favorite=trueのアイテムを2件作成する
    for _ in 0..2 {
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/items")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "media_type": "anime",
                            "title": "include_totalテスト",
                            "is_favorite": true
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create_response.status(), StatusCode::CREATED);
    }

    // 【実際の処理実行】: include_total未指定でGET /itemsを実行する
    let without_total = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/items?is_favorite=true")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(without_total.status(), StatusCode::OK);
    let without_total_json = body_json(without_total).await;
    assert!(
        without_total_json["pagination"].get("total").is_none(),
        "include_total未指定時はtotalフィールドが省略されること"
    ); // 【確認内容】: TC2 未指定時はtotalが返らないことを確認 🔵

    // 【実際の処理実行】: include_total=falseを明示指定してGET /itemsを実行する
    let explicit_false = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/items?is_favorite=true&include_total=false")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let explicit_false_json = body_json(explicit_false).await;
    assert!(
        explicit_false_json["pagination"].get("total").is_none(),
        "include_total=false明示時もtotalフィールドが省略されること"
    ); // 【確認内容】: TC6 include_total=falseは未指定時と同じ挙動であることを確認 🟡

    // 【実際の処理実行】: include_total=trueでGET /itemsを実行する
    let with_total = app
        .oneshot(
            Request::builder()
                .uri("/items?is_favorite=true&include_total=true")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(with_total.status(), StatusCode::OK);
    let with_total_json = body_json(with_total).await;

    // 【結果検証】: is_favorite=trueに該当する件数（2件以上、少なくとも作成した2件を含む）がtotalに反映されることを確認する
    let total = with_total_json["pagination"]["total"]
        .as_i64()
        .expect("include_total=true時はtotalが数値で返ること"); // 【確認内容】: TC1 totalが返ることを確認 🔵
    assert!(total >= 2, "作成した2件を含む件数がtotalに反映されること");
}
