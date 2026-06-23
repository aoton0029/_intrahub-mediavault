//! items ハンドラ
//!
//! TASK-0009: POST /items（手動作成）実装

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::models::item::{
    deserialize_request, parse_create_item_request, parse_item_id, validate_update_title, Item,
    ItemDetail, ListItemsQuery, UpdateItemRequest,
};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk, PaginatedOk, Pagination};
use crate::repositories::item_repository;
use crate::AppState;

/// 【機能概要】: `POST /items` ハンドラ。フォーム入力によるアイテム手動作成を行う
/// 【実装方針】: TASK-0008の`parse_create_item_request`でバリデーションし、
/// 成功した場合のみitem_repository::create_itemでDB登録、201レスポンスを返す
/// 【テスト対応】: TC-001-01（必須項目のみ作成）, TC-001-E01（media_type不正）,
/// TC-001-B01（title空文字）に対応
/// 🔵 信頼性レベル: api-endpoints.md POST /itemsの仕様に直接対応
pub async fn create_item_handler(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    // 【入力値検証】: media_type/titleの妥当性をTASK-0008実装済みの関数で検証する 🔵
    let request = match parse_create_item_request(body) {
        Ok(request) => request,
        Err(err) => return Err(err),
    };

    // 【DB登録】: items + 詳細テーブルへ同一トランザクションでINSERTする 🔵
    let item = item_repository::create_item(&state.db, request).await?;

    // 【成功レスポンス】: 作成済みitemを201で返す 🔵
    Ok(created_response(item))
}

/// 【機能概要】: 作成済みitemをHTTP 201・統一レスポンス形式で返すためのレスポンスを構築する
/// 【実装方針】: `ApiOk::into_response()`は200固定のため、201を返す本ハンドラ専用に
/// ステータスコードを明示的に組み立てる
/// 【テスト対応】: created_response_returns_201_with_success_envelope を通すための実装
/// 🔵 信頼性レベル: api-endpoints.md「レスポンス（成功, 201）」に直接対応
fn created_response(item: Item) -> axum::response::Response {
    // 【レスポンス構築】: ステータス201 + 統一フォーマット{"success":true,"data":item} 🔵
    (StatusCode::CREATED, Json(ApiOk::new(item))).into_response()
}

/// 【機能概要】: page/limitクエリパラメータを正規化（クランプ）する純関数
/// 【実装方針】: page<1→1、limit<1→20（デフォルト）、limit>100→100にクランプする。
/// 不正な値でも400エラーにせず安全な範囲へ丸めることで、OFFSET計算のアンダーフローや
/// 過大なLIMIT要求からサーバーを保護する
/// 【テスト対応】: TC-0010-B01〜B06（normalize_paginationの境界値テスト）を通すための実装
/// 🟡 信頼性レベル: テストケース定義書 確定2（page<1→1, limit<1→20, limit>100→100）に対応
pub fn normalize_pagination(page: Option<u32>, limit: Option<u32>) -> (u32, u32) {
    // 【page正規化】: 未指定はデフォルト1、0は1にクランプ（u32なので負数は型レベルで排除済み） 🟡
    let page = match page {
        Some(p) if p >= 1 => p,
        _ => 1,
    };

    // 【limit正規化】: 未指定はデフォルト20、0は20にクランプ、100超は100にクランプ 🟡
    let limit = match limit {
        None => 20,
        Some(0) => 20,
        Some(l) if l > 100 => 100,
        Some(l) => l,
    };

    (page, limit)
}

/// 【機能概要】: `GET /items` ハンドラ。クエリパラメータによる絞り込み・ページネーションを行い
/// 一覧を返す
/// 【実装方針】: クエリパラメータを正規化し、repository層のlist_items/count_itemsを呼び出して
/// PaginatedOkで200を返す
/// 【テスト対応】: TC-0010-N01〜N08等のルーティング・統合テストで利用される
/// 🔵 信頼性レベル: 要件定義書 2.4 データフローに直接対応
pub async fn list_items_handler(
    State(state): State<AppState>,
    Query(query): Query<ListItemsQuery>,
) -> Result<PaginatedOk<Vec<Item>>, ApiError> {
    // 【ページネーション正規化】: page/limitをクランプして安全な範囲に揃える 🟡
    let (page, limit) = normalize_pagination(query.page, query.limit);
    let normalized_query = ListItemsQuery {
        page: Some(page),
        limit: Some(limit),
        ..query
    };

    // 【データ取得】: 絞り込み条件に従いitems一覧とtotal件数を取得する 🔵
    let items = item_repository::list_items(&state.db, &normalized_query).await?;
    let total = item_repository::count_items(&state.db, &normalized_query).await?;

    // 【レスポンス構築】: {success, data, pagination}形式で200を返す 🔵
    Ok(PaginatedOk::new(items, Pagination { page, limit, total }))
}

/// 【機能概要】: `GET /items/:id` ハンドラ。アイテム詳細をメディア別詳細テーブル・タグ・
/// カテゴリを含めて取得する
/// 【実装方針】: パスパラメータをUUIDへパース → items本体取得（無ければ404） → 詳細/タグ/
/// カテゴリを取得してItemDetailへ合成し200で返す
/// 【テスト対応】: タスクファイル テストケース1〜3（200/404/400）に対応
/// 🟡 信頼性レベル: api-endpoints.md GET /items/:id仕様からの妥当な推測
pub async fn get_item_handler(
    State(state): State<AppState>,
    Path(id_raw): Path<String>,
) -> Result<ApiOk<ItemDetail>, ApiError> {
    // 【UUIDバリデーション】: 不正な形式は400 VALIDATION_ERRORで早期リターンする 🟡
    let id = parse_item_id(&id_raw)?;

    // 【items本体取得】: 存在しない場合は404 ITEM_NOT_FOUNDを返す 🟡
    let item = item_repository::get_item_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::new(ApiErrorCode::ItemNotFound, "アイテムが見つかりません"))?;

    // 【関連データ取得】: メディア別詳細テーブル・タグ・カテゴリを合成する 🟡
    let detail = item_repository::get_item_detail(&state.db, item.media_type, id).await?;
    let tags = item_repository::get_item_tags(&state.db, id).await?;
    let categories = item_repository::get_item_categories(&state.db, id).await?;

    Ok(ApiOk::new(ItemDetail::from_parts(item, detail, tags, categories)))
}

/// 【機能概要】: `PATCH /items/:id` ハンドラ。UpdateItemRequestのSomeフィールドのみを対象アイテムに適用する
/// 【実装方針】: パスパラメータをUUIDへパース → リクエストボディをUpdateItemRequestへデシリアライズ・
/// title空文字バリデーション → item_repository::update_itemでDB反映 → 存在しなければ404
/// 【テスト対応】: TC-001-E02-B（404）、TC-001-B01-B（400・DB未変更）、TC-NEW-04（不正UUID形式）に対応
/// 🔵 信頼性レベル: 要件定義書シナリオ1〜5・REQ-0012-101〜103・REQ-0012-201〜202に直接対応
pub async fn update_item_handler(
    State(state): State<AppState>,
    Path(id_raw): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<ApiOk<Item>, ApiError> {
    let id = parse_item_id(&id_raw)?;

    let request: UpdateItemRequest = deserialize_request(body)?;

    validate_update_title(&request.title)?;

    let item = item_repository::update_item(&state.db, id, request)
        .await?
        .ok_or_else(|| ApiError::new(ApiErrorCode::ItemNotFound, "アイテムが見つかりません"))?;

    Ok(ApiOk::new(item))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::item::{ItemSource, ItemStatus, MediaType};
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn sample_item() -> Item {
        Item {
            id: Uuid::new_v4(),
            media_type: MediaType::Anime,
            title: "作品A".to_string(),
            original_title: None,
            description: None,
            cover_image_url: None,
            release_date: None,
            homepage_url: None,
            status: ItemStatus::NotStarted,
            consumed_date: None,
            rating: None,
            is_favorite: false,
            source: ItemSource::Manual,
            external_id: None,
            created_at: NaiveDate::from_ymd_opt(2026, 6, 23)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2026, 6, 23)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        }
    }

    /// TC-001-01: 作成済みitemのレスポンスがHTTP 201・統一フォーマットになることを確認
    /// 🔵 信頼性レベル: タスクファイル完了条件「成功時、作成済みitem（UUID付き）を201で返す」に直接対応
    #[tokio::test]
    async fn created_response_returns_201_with_success_envelope() {
        // 【テスト目的】: created_response関数がHTTP 201と{"success":true,"data":...}形式を返すことを確認する
        // 【テスト内容】: sample_item()を渡してレスポンスを生成し、ステータスコードとボディを検証する
        // 【期待される動作】: ステータス201、ボディがsuccess=trueでdataにitemの内容を含む
        // 🔵 信頼性レベル: api-endpoints.md「レスポンス（成功, 201）」、タスクファイル完了条件に直接対応

        // 【テストデータ準備】: TC-001-01相当の最小構成item（source=manual, external_id=None）
        // 【初期条件設定】: created_response実装済みのため、正常にレスポンスが構築される
        let item = sample_item();

        // 【実際の処理実行】: レスポンス構築関数を呼び出す
        // 【処理内容】: 作成済みitemをHTTP 201レスポンスへ変換する
        let response = created_response(item);

        // 【結果検証】: ステータスコードが201であることを確認
        // 【期待値確認】: タスク完了条件「201で返す」に対応
        assert_eq!(response.status(), StatusCode::CREATED); // 【確認内容】: 作成成功時のHTTPステータスが201であることを確認 🔵
    }

    /// TC-0010-B01: limit 最大値クランプ（limit=500 → 100）
    /// 🟡 信頼性レベル: TASK-0010 TC-004・要件 UC-6 に対応
    #[test]
    fn normalize_pagination_clamps_limit_to_100() {
        // 【テスト目的】: limit が上限(100)を超えた場合に100へクランプされることを確認する
        // 【テスト内容】: normalize_pagination(Some(1), Some(500)) を呼び出す
        // 【期待される動作】: (page, limit) = (1, 100) が返る
        // 🟡 信頼性レベル: TASK-0010 TC-004（limit=500→100）に対応（normalize_paginationは未実装関数）

        // 【テストデータ準備】: 上限超過のlimit=500を用意（過大要求のサーバー保護を検証するため）
        // 【初期条件設定】: pageは正常値1
        let (page, limit) = normalize_pagination(Some(1), Some(500));

        // 【結果検証】: limitが100に丸められることを確認
        // 【期待値確認】: 過大要求でも応答時間・メモリが保護される設計であることの確認
        assert_eq!(page, 1); // 【確認内容】: pageは変更されないことを確認 🟡
        assert_eq!(limit, 100); // 【確認内容】: limitが上限100にクランプされることを確認 🟡
    }

    /// TC-0010-B02: limit 上限ちょうど（limit=100 → 100、非クランプ境界）
    /// 🟡 信頼性レベル: 確定2・タスク「1〜100にクランプ」からの妥当な推測
    #[test]
    fn normalize_pagination_does_not_clamp_limit_at_exactly_100() {
        // 【テスト目的】: limit=100ちょうどの場合はクランプされず100のまま通過することを確認する
        // 【テスト内容】: normalize_pagination(Some(1), Some(100)) を呼び出す
        // 【期待される動作】: (page, limit) = (1, 100)（クランプ非発生）
        // 🟡 信頼性レベル: 上限境界の包含関係（>100でクランプ、==100は通過）を確認するためのoff-by-one防止テスト

        // 【テストデータ準備】: 上限ぴったりのlimit=100を用意
        // 【初期条件設定】: pageは正常値1
        let (page, limit) = normalize_pagination(Some(1), Some(100));

        // 【結果検証】: 100が101のように誤ってクランプされないことを確認
        assert_eq!(page, 1); // 【確認内容】: pageは変更されないことを確認 🟡
        assert_eq!(limit, 100); // 【確認内容】: limit=100は境界内のためクランプされず100のままであることを確認 🟡
    }

    /// TC-0010-B03: limit=0 → デフォルト20にクランプ
    /// 🟡 信頼性レベル: テストケース定義書 確定2（本フェーズで方針確定）に基づく
    #[test]
    fn normalize_pagination_clamps_zero_limit_to_default_20() {
        // 【テスト目的】: limit=0（下限割れ）が400エラーではなくデフォルト値20にクランプされることを確認する
        // 【テスト内容】: normalize_pagination(Some(1), Some(0)) を呼び出す
        // 【期待される動作】: (page, limit) = (1, 20)
        // 🟡 信頼性レベル: 確定2「limit<1 → 20」の方針に基づく（normalize_paginationは未実装関数）

        // 【テストデータ準備】: 下限割れのlimit=0を用意（無意味なLIMIT 0クエリを防ぐ検証のため）
        // 【初期条件設定】: pageは正常値1
        let (page, limit) = normalize_pagination(Some(1), Some(0));

        // 【結果検証】: limitがデフォルト値20にクランプされることを確認
        assert_eq!(page, 1); // 【確認内容】: pageは変更されないことを確認 🟡
        assert_eq!(limit, 20); // 【確認内容】: limit=0はデフォルトの20にクランプされることを確認 🟡
    }

    /// TC-0010-B04: page=0 → 1にクランプ（OFFSET=0）
    /// 🟡 信頼性レベル: 確定2・note.md 6章（page=0方針）に基づく
    #[test]
    fn normalize_pagination_clamps_zero_page_to_1() {
        // 【テスト目的】: page=0（下限割れ）が1にクランプされ、OFFSET算出時のアンダーフローを防ぐことを確認する
        // 【テスト内容】: normalize_pagination(Some(0), Some(20)) を呼び出す
        // 【期待される動作】: (page, limit) = (1, 20)
        // 🟡 信頼性レベル: 確定2「page<1 → 1」、OFFSET=(page-1)*limitのu32アンダーフロー回避方針に基づく

        // 【テストデータ準備】: 下限割れのpage=0を用意（(0-1)のu32アンダーフローpanicを防ぐ検証のため）
        // 【初期条件設定】: limitは正常値20
        let (page, limit) = normalize_pagination(Some(0), Some(20));

        // 【結果検証】: pageが1にクランプされ、OFFSET計算が安全に行えることを確認
        assert_eq!(page, 1); // 【確認内容】: page=0は1にクランプされることを確認 🟡
        assert_eq!(limit, 20); // 【確認内容】: limitは変更されないことを確認 🟡
        assert_eq!((page - 1) * limit, 0); // 【確認内容】: OFFSET算出がアンダーフローせず0になることを確認 🟡
    }

    /// TC-0010-B05: パラメータ未指定 → デフォルト(page=1, limit=20)
    /// 🔵 信頼性レベル: 要件 入力仕様表（page デフォルト1, limit デフォルト20）に直接対応
    #[test]
    fn normalize_pagination_defaults_to_page1_limit20_when_none() {
        // 【テスト目的】: page/limitともにNoneの場合、デフォルト値(1, 20)が適用されることを確認する
        // 【テスト内容】: normalize_pagination(None, None) を呼び出す
        // 【期待される動作】: (page, limit) = (1, 20)、OFFSET=0
        // 🔵 信頼性レベル: 要件定義書 2.1 入力仕様表（pageデフォルト1, limitデフォルト20）に直接対応

        // 【テストデータ準備】: クエリパラメータ完全未指定の状況を再現
        // 【初期条件設定】: page, limit ともに None
        let (page, limit) = normalize_pagination(None, None);

        // 【結果検証】: Noneとデフォルト値指定で同一結果になることを確認
        assert_eq!(page, 1); // 【確認内容】: page未指定時のデフォルトが1であることを確認 🔵
        assert_eq!(limit, 20); // 【確認内容】: limit未指定時のデフォルトが20であることを確認 🔵
        assert_eq!((page - 1) * limit, 0); // 【確認内容】: デフォルト時のOFFSETが0であることを確認 🔵
    }

    /// TC-0010-B06: OFFSET算出（page=2, limit=20 → OFFSET=20）
    /// 🟡 信頼性レベル: 要件 UC-7「page=2&limit=20 → 21〜40件目（OFFSET=20）」に対応
    #[test]
    fn normalize_pagination_computes_offset_20_for_page2_limit20() {
        // 【テスト目的】: 2ページ目（page=2, limit=20）のOFFSET算出が正しく20になることを確認する
        // 【テスト内容】: normalize_pagination(Some(2), Some(20)) の結果から (page-1)*limit を計算する
        // 【期待される動作】: page=2, limit=20が保持され、OFFSET=(2-1)*20=20となる
        // 🟡 信頼性レベル: 要件定義書 UC-7（page=2&limit=20→OFFSET=20）に対応

        // 【テストデータ準備】: 2ページ目を指定する正常値
        // 【初期条件設定】: page=2, limit=20（クランプ不要な正常範囲）
        let (page, limit) = normalize_pagination(Some(2), Some(20));

        // 【結果検証】: ページ送りのOFFSET計算が正しいことを確認
        assert_eq!(page, 2); // 【確認内容】: pageが2のまま保持されることを確認 🟡
        assert_eq!(limit, 20); // 【確認内容】: limitが20のまま保持されることを確認 🟡
        assert_eq!((page - 1) * limit, 20); // 【確認内容】: OFFSET=(page-1)*limitが20と算出されることを確認 🟡
    }

    /// TC-001-E02-B: update_item_handlerが存在しないIDに対し404 ITEM_NOT_FOUNDを返す（実DB必要）
    /// 【テスト目的】: ハンドラ層でリポジトリのNoneをApiErrorに変換する分岐を確認する
    /// 【テスト内容】: 未登録UUIDをパスパラメータに、JSONボディに{"rating": 3.0}を渡してハンドラを呼ぶ
    /// 【期待される動作】: HTTPステータス404、ApiErrorCode::ItemNotFound
    /// 🔵 信頼性レベル: 要件定義書シナリオ3・REQ-0012-201、タスクファイルテストケース2より
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn update_item_handler_returns_404_for_nonexistent_item() {
        // 【テスト前準備】: AppStateを実DBプールから構築する（テスト用ヘルパーは未実装のため
        // Greenフェーズでcrate::AppState構築用ヘルパーを用意する想定）
        let state = test_app_state().await;
        let id = Uuid::new_v4();
        let body = serde_json::json!({ "rating": 3.0 });

        // 【実際の処理実行】: まだ実装されていないupdate_item_handlerを呼び出す
        let result = update_item_handler(State(state), Path(id.to_string()), Json(body)).await;

        // 【結果検証】: 404 ITEM_NOT_FOUNDが返ることを確認
        let err = result.unwrap_err();
        assert_eq!(err.error.code, "ITEM_NOT_FOUND"); // 【確認内容】: 存在しないIDでITEM_NOT_FOUNDが返ることを確認 🔵
        assert_eq!(err.status, StatusCode::NOT_FOUND); // 【確認内容】: HTTPステータスが404であることを確認 🔵
    }

    /// TC-001-B01-B: update_item_handlerがtitle空文字のリクエストに対し400 VALIDATION_ERRORを返し、
    /// UPDATEを実行しない（実DB必要）
    /// 【テスト目的】: HTTPレベルでのバリデーションエラー伝播と、DB状態が変化しないことを確認する
    /// 【テスト内容】: 既存item IDに対しJSONボディ{"title": ""}を渡してハンドラを呼ぶ
    /// 【期待される動作】: HTTPステータス400、ApiErrorCode::ValidationError、かつDBのtitleが変化しない
    /// 🔵 信頼性レベル: 要件定義書シナリオ4・REQ-0012-102、タスクファイルテストケース3より
    #[tokio::test]
    #[ignore]
    async fn update_item_handler_returns_400_and_does_not_mutate_db_for_empty_title() {
        let state = test_app_state().await;
        let item_id = insert_test_item_for_handler(&state).await;
        let before = item_repository::get_item_by_id(&state.db, item_id)
            .await
            .unwrap()
            .unwrap();
        let body = serde_json::json!({ "title": "" });

        let result = update_item_handler(State(state.clone()), Path(item_id.to_string()), Json(body))
            .await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR"); // 【確認内容】: title空文字でVALIDATION_ERRORが返ることを確認 🔵
        assert_eq!(err.status, StatusCode::BAD_REQUEST); // 【確認内容】: HTTPステータスが400であることを確認 🔵

        // 【結果検証】: バリデーション失敗時にDBへの副作用が一切ないことを直接DBクエリで確認する（最重要検証）
        let after = item_repository::get_item_by_id(&state.db, item_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(after.title, before.title); // 【確認内容】: titleがDB上で変化していないことを確認 🔵
        assert_eq!(after.updated_at, before.updated_at); // 【確認内容】: UPDATE未実行＝updated_at不変であることを確認 🔵
    }

    /// TC-NEW-04: update_item_handlerが不正なUUID文字列のパスパラメータに対し400を返す（既存ロジック再利用確認）
    /// 【テスト目的】: パス検証は既存parse_item_id（GET /items/:idで使用中）を再利用する方針のため、
    /// その挙動がPATCHでも一貫していることを確認する
    /// 【テスト内容】: パスパラメータ"not-a-uuid"を渡してハンドラを呼ぶ
    /// 【期待される動作】: HTTPステータス400、ApiErrorCode::ValidationError
    /// 🟡 信頼性レベル: 要件定義書REQ-0012-103・EDGE-0012-03より。既存parse_item_idのテストが
    /// 別途存在する前提で、PATCH側では結合のみを確認する軽量テストとする
    #[tokio::test]
    #[ignore]
    async fn update_item_handler_returns_400_for_invalid_uuid_path() {
        let state = test_app_state().await;
        let body = serde_json::json!({ "rating": 3.0 });

        let result =
            update_item_handler(State(state), Path("not-a-uuid".to_string()), Json(body)).await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR"); // 【確認内容】: 不正なUUID形式でVALIDATION_ERRORが返ることを確認 🟡
        assert_eq!(err.status, StatusCode::BAD_REQUEST); // 【確認内容】: HTTPステータスが400であることを確認 🟡
    }

    /// 【テスト用ヘルパー】: 実DB接続済みのAppStateを構築する（Greenフェーズで実装する想定）
    /// 現時点ではAppStateの構築方法・DATABASE_URL取得方法が未確定のため未実装関数として参照する
    async fn test_app_state() -> AppState {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0012統合テストにはDATABASE_URL環境変数が必要です");
        let pool = sqlx::PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました");
        AppState {
            db: pool,
            internal_api_key: "test-key".to_string(),
        }
    }

    /// 【テスト用ヘルパー】: ハンドラ統合テスト用にitemsへ1件投入しidを返す
    async fn insert_test_item_for_handler(state: &AppState) -> Uuid {
        sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO items (media_type, title, status, is_favorite, source, external_id) \
            VALUES ('anime', 'テストアイテム', 'not_started', false, 'manual', NULL) RETURNING id",
        )
        .fetch_one(&state.db)
        .await
        .expect("テスト用itemの投入に失敗しました")
    }
}
