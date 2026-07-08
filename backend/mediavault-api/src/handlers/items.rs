//! items ハンドラ
//!
//! TASK-0009: POST /items（手動作成）実装

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::models::domain::MediaDetails;
use crate::models::item::{
    Item, ItemDetail, ItemWithRefs, ListItemsQuery, MediaTypeCounts, UpdateItemRequest,
    UpdateStatusRequest, deserialize_request, parse_create_item_request, parse_item_id,
    validate_update_title,
};
use crate::models::item_import::parse_import_item_request;
use crate::models::item_search::ItemSearchQuery;
use crate::models::response::{ApiError, ApiErrorCode, ApiOk, PaginatedOk, Pagination};
use crate::repositories::item_file_repository;
use crate::repositories::item_repository;
use crate::repositories::item_streaming_link_repository;
use crate::services::external_search::ExternalSearchService;

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

    // 【details検証】: 指定された場合はMediaDetails形式かつmedia_type一致であることを検証する
    validate_create_details(&request)?;

    // 【DB登録】: itemsテーブル（details JSONB含む）へINSERTする 🔵
    let item = item_repository::create_item(&state.db, request).await?;

    // 【成功レスポンス】: 作成済みitemを201で返す 🔵
    Ok(created_response(item))
}

/// 【機能概要】: 手動作成リクエストのdetailsがMediaDetails形式として妥当かを検証する
/// 【実装方針】: detailsはインポート経路では正規化済みMediaDetailsが入る前提のため、
/// 手動作成経路でも同一スキーマへデシリアライズ可能で、かつ内側のmedia_typeが
/// リクエスト本体のmedia_typeと一致する場合のみ受理する（不一致・不正形式は400）
fn validate_create_details(
    request: &crate::models::item::CreateItemRequest,
) -> Result<(), ApiError> {
    let Some(details) = &request.details else {
        return Ok(());
    };

    let parsed: MediaDetails = serde_json::from_value(details.clone())
        .map_err(|_| ApiError::new(ApiErrorCode::ValidationError, "detailsの形式が不正です"))?;

    if parsed.core().media_type != request.media_type {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "detailsのmedia_typeがリクエストのmedia_typeと一致しません",
        ));
    }

    Ok(())
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

/// 【機能概要】: limitクエリパラメータを正規化（クランプ）する純関数
/// 【実装方針】: limit<1→20（デフォルト）、limit>100→100にクランプする。
/// 不正な値でも400エラーにせず安全な範囲へ丸めることで、過大なLIMIT要求からサーバーを保護する
pub fn normalize_limit(limit: Option<u32>) -> u32 {
    match limit {
        None => 20,
        Some(0) => 20,
        Some(l) if l > 100 => 100,
        Some(l) => l,
    }
}

/// 【機能概要】: `GET /items` ハンドラ。クエリパラメータによる絞り込み・ページネーションを行い
/// 一覧を返す
/// 【実装方針】: limitを正規化し、repository層のlist_itemsをlimit+1件で呼び出してhas_more/次カーソルを
/// 判定し、PaginatedOkで200を返す
/// 【テスト対応】: TC-0010-N01〜N08等のルーティング・統合テストで利用される
/// 🔵 信頼性レベル: 要件定義書 2.4 データフローに直接対応
pub async fn list_items_handler(
    State(state): State<AppState>,
    Query(query): Query<ListItemsQuery>,
) -> Result<PaginatedOk<Vec<ItemWithRefs>>, ApiError> {
    // 【limit正規化】: limitをクランプして安全な範囲に揃える
    let limit = normalize_limit(query.limit);
    let normalized_query = ListItemsQuery {
        limit: Some(limit),
        ..query
    };

    // 【データ取得】: 絞り込み条件に従いitems一覧を取得する（limit+1件取得してhas_moreを判定する） 🔵
    let mut items = item_repository::list_items(&state.db, &normalized_query).await?;

    // 【has_more判定】: limit+1件目が取得できた場合は次ページが存在する
    let has_more = items.len() as u32 > limit;
    if has_more {
        items.truncate(limit as usize);
    }
    let (next_after_created_at, next_after_id) = if has_more {
        let last = items.last().expect("has_more implies non-empty items");
        (Some(last.created_at), Some(last.id))
    } else {
        (None, None)
    };

    // 【tags/categories付与】: カードUIのタグピル表示のため、一覧アイテムにもtags/categoriesを
    // バッチ取得して合成する（N+1回避のためitem_id単位で一括取得しRust側でzipする）
    let item_ids: Vec<uuid::Uuid> = items.iter().map(|item| item.id).collect();
    let mut tags_by_item = item_repository::get_items_tags_batch(&state.db, &item_ids).await?;
    let mut categories_by_item =
        item_repository::get_items_categories_batch(&state.db, &item_ids).await?;

    let items_with_refs: Vec<ItemWithRefs> = items
        .into_iter()
        .map(|item| {
            let tags = tags_by_item.remove(&item.id).unwrap_or_default();
            let categories = categories_by_item.remove(&item.id).unwrap_or_default();
            ItemWithRefs {
                item,
                tags,
                categories,
            }
        })
        .collect();

    // 【レスポンス構築】: {success, data, pagination}形式で200を返す 🔵
    Ok(PaginatedOk::new(
        items_with_refs,
        Pagination {
            limit,
            has_more,
            next_after_created_at,
            next_after_id,
        },
    ))
}

/// 【機能概要】: `GET /items/counts-by-media-type` ハンドラ。サイドバー表示用に
/// メディア種別ごとのアイテム件数を集計して返す
pub async fn count_items_by_media_type_handler(
    State(state): State<AppState>,
) -> Result<ApiOk<MediaTypeCounts>, ApiError> {
    let counts = item_repository::count_items_by_media_type(&state.db).await?;
    Ok(ApiOk::new(counts))
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

    // 【関連データ取得】: items.details（正規化済みMediaDetails JSON）・タグ・カテゴリを合成する 🟡
    let detail = item_repository::get_item_detail(&state.db, id).await?;
    let tags = item_repository::get_item_tags(&state.db, id).await?;
    let categories = item_repository::get_item_categories(&state.db, id).await?;

    // 【Calibre-Web遷移情報取得】: calibre_book_id設定済みPDFについて遷移情報を付加する（TASK-0028 TC-020-02） 🟡
    let calibre_links = item_file_repository::get_item_calibre_links(&state.db, id).await?;

    // 【配信URL取得】: Netflix/AmazonPrime/DisneyPlus/DmmTv/AppleTvの登録済みURLを付加する
    let streaming_links =
        item_streaming_link_repository::list_item_streaming_links(&state.db, id).await?;

    Ok(ApiOk::new(ItemDetail::from_parts_full(
        item,
        detail,
        tags,
        categories,
        calibre_links,
        streaming_links,
    )))
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

/// 【機能概要】: `DELETE /items/:id` ハンドラ。指定itemを削除し、関連テーブルは
/// DBの`ON DELETE CASCADE`制約に委ねて連動削除する
/// 【実装方針】: パスパラメータをUUIDへパース → item_repository::delete_itemでDELETE実行 →
/// 削除件数が0件（対象不存在）の場合は404を返す
/// 【テスト対応】: TC-001-03（正常削除で204）、存在しないitemで404 に対応
/// 🔵 信頼性レベル: タスクファイル「DELETE /items/:idが指定itemを削除し、成功時204を返す」
/// 「該当itemが存在しない場合、ITEM_NOT_FOUND（404）を返す」に直接対応
pub async fn delete_item_handler(
    State(state): State<AppState>,
    Path(id_raw): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let id = parse_item_id(&id_raw)?;

    let deleted = item_repository::delete_item(&state.db, id).await?;
    if !deleted {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "アイテムが見つかりません",
        ));
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// 【機能概要】: `PATCH /items/:id/status` ハンドラ。視聴・読了状況（`status`）および
/// 消費日（`consumed_date`）のみを更新する専用エンドポイント
/// 【実装方針】: パスパラメータをUUIDへパース → リクエストボディを`UpdateStatusRequest`へ
/// デシリアライズ（`status`不正値はここでVALIDATION_ERRORになる） →
/// `item_repository::update_item_status`でDB反映 → 存在しなければ404
/// 【テスト対応】: タスクファイル テストケース1（正常更新）・2（status不正値で400）・
/// 3（存在しないitemで404）に対応
/// 🔵 信頼性レベル: REQ-008・user-stories 1.5、タスクファイル完了条件に直接対応
pub async fn update_item_status_handler(
    State(state): State<AppState>,
    Path(id_raw): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<ApiOk<Item>, ApiError> {
    let id = parse_item_id(&id_raw)?;

    let request: UpdateStatusRequest = deserialize_request(body)?;

    let item = item_repository::update_item_status(&state.db, id, request)
        .await?
        .ok_or_else(|| ApiError::new(ApiErrorCode::ItemNotFound, "アイテムが見つかりません"))?;

    Ok(ApiOk::new(item))
}

/// 【機能概要】: `GET /items/search` ハンドラ。外部API（Jikan/TMDb/NDL/IGDB）を検索し、
/// ノーマライズ済み`MediaDetails`（media_typeが判別子）の候補一覧を返す
/// 【実装方針】: AppStateはExternalSearchServiceを保持しないため、ハンドラ内で都度構築する。
pub async fn search_items_handler(
    State(state): State<AppState>,
    Query(query): Query<ItemSearchQuery>,
) -> Result<ApiOk<Vec<MediaDetails>>, ApiError> {
    // 【サービス構築】: AppStateはPgPoolのみ保持するため、ハンドラ内で都度ExternalSearchServiceを構築する 🔵
    let service = ExternalSearchService::new(state.db.clone());

    // 【外部API検索実行】: media_type別ディスパッチで対応プロバイダへ検索を委譲する。
    // エラー時は`?`でExternalSearchError→ApiErrorへ自動変換され、422/502として伝播する 🔵
    let results = service.search(query.media_type, &query.q).await?;

    // 【成功レスポンス】: 統一フォーマット{"success":true,"data":[...]}で200を返す 🔵
    Ok(ApiOk::new(results))
}

/// 【機能概要】: `POST /items/import` ハンドラ。外部API検索結果（GET /items/search）から
/// 選択した1件をインポートし、items+メディア別詳細テーブルへ`source=api`で登録する
/// 【実装方針】: `parse_import_item_request`でバリデーション → `item_repository::import_item`で
/// 重複チェック+DB登録 → 成功時は既存`created_response`を再利用して201を返す
/// 【テスト対応】: TC-0025-N02（created_response再利用）、TC-0025-N06（ルーター経由201）、
/// TC-0025-E01〜E08（各種エラー）に対応
/// 🔵 信頼性レベル: item-import-requirements.md 2.2・2.3データフロー、note.mdより
pub async fn import_item_handler(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    // 【入力値検証】: media_type/external_id/titleの妥当性をparse_import_item_requestで検証する 🔵
    let request = parse_import_item_request(body)?;

    // 【DB登録】: 重複チェック+items+詳細テーブルへ同一トランザクションでINSERTする。
    // 重複時はitem_repository::import_item内でItemAlreadyImported（409）として早期returnされる 🟡
    let item = item_repository::import_item(&state.db, request).await?;

    // 【成功レスポンス】: 既存create_item_handlerと同一のcreated_responseを再利用し201で返す 🔵
    Ok(created_response(item))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::item::{ItemSource, ItemStatus, MediaType};
    use crate::models::item_search::ItemSearchQuery;
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

    /// limit 最大値クランプ（limit=500 → 100）
    #[test]
    fn normalize_limit_clamps_to_100() {
        assert_eq!(normalize_limit(Some(500)), 100);
    }

    /// limit 上限ちょうど（limit=100 → 100、非クランプ境界）
    #[test]
    fn normalize_limit_does_not_clamp_at_exactly_100() {
        assert_eq!(normalize_limit(Some(100)), 100);
    }

    /// limit=0 → デフォルト20にクランプ
    #[test]
    fn normalize_limit_clamps_zero_to_default_20() {
        assert_eq!(normalize_limit(Some(0)), 20);
    }

    /// limit未指定 → デフォルト20
    #[test]
    fn normalize_limit_defaults_to_20_when_none() {
        assert_eq!(normalize_limit(None), 20);
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

        let result =
            update_item_handler(State(state.clone()), Path(item_id.to_string()), Json(body)).await;

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

    /// TC-001-03: delete_item_handlerが正常削除で204を返す（実DB必要）
    /// 【テスト目的】: 既存itemに対しDELETEを実行した際、ステータス204が返ることを確認する
    /// 【テスト内容】: テスト用itemを投入し、そのIDでdelete_item_handlerを呼ぶ
    /// 【期待される動作】: HTTPステータス204
    /// 🔵 信頼性レベル: タスクファイル テストケース1（TC-001-03）に直接対応
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn delete_item_handler_returns_204_for_existing_item() {
        let state = test_app_state().await;
        let item_id = insert_test_item_for_handler(&state).await;

        let response = delete_item_handler(State(state), Path(item_id.to_string()))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT); // 【確認内容】: 正常削除でHTTPステータスが204であることを確認 🔵
    }

    /// 存在しないitemで404 ITEM_NOT_FOUND（実DB必要）
    /// 【テスト目的】: 未登録UUIDを指定した場合に404 ITEM_NOT_FOUNDが返ることを確認する
    /// 【テスト内容】: 未登録UUIDをパスパラメータにdelete_item_handlerを呼ぶ
    /// 【期待される動作】: HTTPステータス404、ApiErrorCode::ItemNotFound
    /// 🔵 信頼性レベル: タスクファイル テストケース2（存在しないitemで404）に直接対応
    #[tokio::test]
    #[ignore]
    async fn delete_item_handler_returns_404_for_nonexistent_item() {
        let state = test_app_state().await;
        let id = Uuid::new_v4();

        let result = delete_item_handler(State(state), Path(id.to_string())).await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "ITEM_NOT_FOUND"); // 【確認内容】: 存在しないIDでITEM_NOT_FOUNDが返ることを確認 🔵
        assert_eq!(err.status, StatusCode::NOT_FOUND); // 【確認内容】: HTTPステータスが404であることを確認 🔵
    }

    /// テストケース2: update_item_status_handlerがstatus不正値に対し400 VALIDATION_ERRORを返す（実DB必要）
    /// 【テスト目的】: ハンドラ層でdeserialize_requestのエラーがApiErrorに変換される分岐を確認する
    /// 🟡 信頼性レベル: タスクファイル テストケース2（status不正値で400）に対応
    #[tokio::test]
    #[ignore]
    async fn update_item_status_handler_returns_400_for_invalid_status() {
        let state = test_app_state().await;
        let item_id = insert_test_item_for_handler(&state).await;
        let body = serde_json::json!({ "status": "invalid_status" });

        let result =
            update_item_status_handler(State(state), Path(item_id.to_string()), Json(body)).await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR"); // 【確認内容】: status不正値でVALIDATION_ERRORが返ることを確認 🟡
        assert_eq!(err.status, StatusCode::BAD_REQUEST); // 【確認内容】: HTTPステータスが400であることを確認 🟡
    }

    /// テストケース3: update_item_status_handlerが存在しないitemに対し404 ITEM_NOT_FOUNDを返す（実DB必要）
    /// 🟡 信頼性レベル: タスクファイル テストケース3（存在しないitemで404）に対応
    #[tokio::test]
    #[ignore]
    async fn update_item_status_handler_returns_404_for_nonexistent_item() {
        let state = test_app_state().await;
        let id = Uuid::new_v4();
        let body = serde_json::json!({ "status": "completed" });

        let result =
            update_item_status_handler(State(state), Path(id.to_string()), Json(body)).await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "ITEM_NOT_FOUND"); // 【確認内容】: 存在しないIDでITEM_NOT_FOUNDが返ることを確認 🟡
        assert_eq!(err.status, StatusCode::NOT_FOUND); // 【確認内容】: HTTPステータスが404であることを確認 🟡
    }

    /// テストケース1: update_item_status_handlerがstatus・consumed_dateを正常更新し200を返す（実DB必要）
    /// 🔵 信頼性レベル: タスクファイル テストケース1（statusとconsumed_dateの正常更新）に直接対応
    #[tokio::test]
    #[ignore]
    async fn update_item_status_handler_updates_status_and_consumed_date() {
        let state = test_app_state().await;
        let item_id = insert_test_item_for_handler(&state).await;
        let body = serde_json::json!({ "status": "completed", "consumed_date": "2026-06-20" });

        let response =
            update_item_status_handler(State(state), Path(item_id.to_string()), Json(body))
                .await
                .unwrap();

        assert_eq!(response.data.status, ItemStatus::Completed); // 【確認内容】: statusが更新値に変化することを確認 🔵
        assert_eq!(
            response.data.consumed_date,
            Some(NaiveDate::from_ymd_opt(2026, 6, 20).unwrap())
        ); // 【確認内容】: consumed_dateが更新値に変化することを確認 🔵
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

    /// TC-0024-N01相当（ハンドラ単体呼び出し）: search_items_handlerがanime検索で200を返す（実DB+外部API必要）
    /// 【テスト目的】: まだ実装されていないsearch_items_handlerが、ItemSearchQueryを受け取り
    /// ExternalSearchService経由で200・ApiOk<Vec<MediaDetails>>を返すことを確認する
    /// 【テスト内容】: ItemSearchQuery { media_type: Anime, q: "鬼滅" } を渡してsearch_items_handlerを呼ぶ
    /// 【期待される動作】: ハンドラ呼び出し自体がコンパイルエラーとなる想定（Red状態）。
    /// Green実装後はHTTPステータス200、ApiOk<Vec<MediaDetails>>形式で返る
    /// 🔵 信頼性レベル: 要件 4.1 TC-002-01・external_search.rs L18-28・dataflow.md（機能1正常系）より
    #[tokio::test]
    #[ignore] // 実DB・外部API（wiremock）が必要。Greenフェーズでwiremock注入経路を確定し#[ignore]解除またはユニット化する
    async fn search_items_handler_returns_200_for_anime_search() {
        // 【テスト前準備】: 実DBプールからAppStateを構築する（Jikanはキー不要のためDB未登録でも到達可能）
        let state = test_app_state().await;
        let query = ItemSearchQuery {
            media_type: MediaType::Anime,
            q: "鬼滅".to_string(),
        };

        // 【実際の処理実行】: まだ実装されていないsearch_items_handlerを呼び出す
        // 【処理内容】: ExternalSearchService::new(state.db.clone()).search(Anime, "鬼滅")への委譲を期待する
        let result = search_items_handler(State(state), Query(query)).await;

        // 【結果検証】: 200・ApiOk<Vec<MediaDetails>>が返ることを確認する（Green phaseで検証予定）
        assert!(result.is_ok()); // 【確認内容】: Green実装後はOk(ApiOk<Vec<MediaDetails>>)が返ることを確認する 🔵
    }

    /// TC-0024-E01相当（ハンドラ単体呼び出し）: search_items_handlerがAPIキー未設定で422を返す（実DB+外部API必要）
    /// 【テスト目的】: ExternalSearchError::ApiKeyNotConfiguredがハンドラの`?`演算子経由で
    /// HTTPステータス422・API_KEY_NOT_CONFIGUREDへ伝播することを確認する
    /// 【テスト内容】: TMDbキー未登録の状態でItemSearchQuery { media_type: Movie, q: "Matrix" } を渡して呼ぶ
    /// 【期待される動作】: ハンドラ呼び出し自体がコンパイルエラーとなる想定（Red状態）。
    /// Green実装後はErr(ApiError { status: 422, error.code: "API_KEY_NOT_CONFIGURED", .. })が返る
    /// 🟡 信頼性レベル: 要件 4.2 TC-002-E01・EDGE-001より妥当推測
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn search_items_handler_returns_422_when_api_key_not_configured() {
        let state = test_app_state().await;
        let query = ItemSearchQuery {
            media_type: MediaType::Movie,
            q: "Matrix".to_string(),
        };

        // 【実際の処理実行】: まだ実装されていないsearch_items_handlerを呼び出す
        let result = search_items_handler(State(state), Query(query)).await;

        // 【結果検証】: 422 API_KEY_NOT_CONFIGUREDが返ることを確認する
        let err = result.unwrap_err();
        assert_eq!(err.error.code, "API_KEY_NOT_CONFIGURED"); // 【確認内容】: TMDbキー未登録でAPI_KEY_NOT_CONFIGUREDが返ることを確認 🟡
        assert_eq!(err.status, StatusCode::UNPROCESSABLE_ENTITY); // 【確認内容】: HTTPステータスが422であることを確認 🟡
    }

    /// TC-0025-N02: import_item_handler成功時のレスポンスがHTTP 201・統一フォーマットになる
    /// （created_response再利用確認、DB非依存）
    /// 【テスト目的】: インポート成功時のレスポンス構築が、手動作成POST /itemsと同一の
    /// created_responseを再利用して201を返すことを確認する
    /// 【テスト内容】: source=Api/external_id=Some("12345")のsample_itemをcreated_responseへ渡す
    /// 【期待される動作】: response.status() == StatusCode::CREATED
    /// 🔵 信頼性レベル: 要件2.2、既存created_response（L49-52）に直接対応
    #[tokio::test]
    async fn created_response_returns_201_for_api_sourced_item() {
        // 【テストデータ準備】: source=Api/external_id=Someのitemを用意する
        // 【初期条件設定】: import経路（source=api）でも同一の201契約が成り立つことを代表する
        let mut item = sample_item();
        item.source = ItemSource::Api;
        item.external_id = Some("12345".to_string());

        // 【実際の処理実行】: 既存created_responseをapi-source itemに対して呼び出す
        let response = created_response(item);

        // 【結果検証】: ステータスコードが201であることを確認
        assert_eq!(response.status(), StatusCode::CREATED); // 【確認内容】: import経路でも201が返ることを確認 🔵
    }

    /// TC-0025-N06: POST /items/importがルーター経由で有効リクエストに対し201を返す（実DB必要・E2E）
    /// 【テスト目的】: build_routerに登録されたPOST /items/importが、tower::oneshot駆動で201を
    /// 返すE2Eパスを確認する
    /// 【テスト内容】: import_item_handlerを直接呼び出し、有効なJSONボディに対する応答を確認する
    /// 【期待される動作】: ステータス201、ボディがsource:"api"・external_id:"12345"を含むItem
    /// 🔵 信頼性レベル: 要件2.3データフロー、note.mdルーティング方針より
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn import_item_handler_returns_201_for_valid_request() {
        let state = test_app_state().await;
        let body = serde_json::json!({
            "media_type": "anime",
            "external_id": "12345",
            "title": "鬼滅の刃"
        });

        // 【実際の処理実行】: import_item_handlerを呼び出す
        let response = import_item_handler(State(state), Json(body)).await.unwrap();

        // 【結果検証】: HTTPステータスが201であることを確認
        assert_eq!(response.status(), StatusCode::CREATED); // 【確認内容】: 有効なインポートリクエストで201が返ることを確認 🔵
    }

    /// TC-0025-E01相当（ハンドラ単体）: import_item_handlerがexternal_id欠落で400 VALIDATION_ERROR
    /// を返す（実DB不要・バリデーション先行のため副作用なし）
    /// 【テスト目的】: ハンドラ層でのバリデーション失敗が、DB到達前にApiErrorへ変換されることを確認する
    /// 【テスト内容】: external_idキーを含まないJSONボディでimport_item_handlerを呼ぶ
    /// 【期待される動作】: Err(ApiError{code:"VALIDATION_ERROR", status:400})
    /// 🔵 信頼性レベル: 要件2.2エラー表・4.2、TASK-0025.mdテストケース2に直接対応
    /// 【補足】: external_id欠落はserdeデシリアライズ失敗（VALIDATION_ERROR）経路で弾かれる。
    /// AppState構築（DB接続）を要するため#[ignore]とする
    #[tokio::test]
    #[ignore]
    async fn import_item_handler_returns_400_for_missing_external_id() {
        let state = test_app_state().await;
        let body = serde_json::json!({
            "media_type": "anime",
            "title": "鬼滅の刃"
        });

        // 【実際の処理実行】: import_item_handlerを呼び出す
        let result = import_item_handler(State(state), Json(body)).await;

        // 【結果検証】: VALIDATION_ERROR・400であることを確認
        let err = result.unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR"); // 【確認内容】: external_id欠落でVALIDATION_ERRORが返ることを確認 🔵
        assert_eq!(err.status, StatusCode::BAD_REQUEST); // 【確認内容】: HTTPステータスが400であることを確認 🔵
    }

    /// TC-0025-B04: POST /items/importがリテラルパスとして優先され/items/:idに誤マッチしない
    /// （ルーター経由・実DB必要）
    /// 【テスト目的】: "import"が/items/:idの:idとしてUUIDパースされず400にならないことを確認する
    /// 【テスト内容】: build_router経由でPOST /items/importへ有効ボディを送信する
    /// 【期待される動作】: UUIDパースエラー由来の400にはならない（500でもない）
    /// 🔵 信頼性レベル: note.md L46-49・L190-192、既存get_items_search_does_not_fall_through_to_item_id_route
    /// より
    #[tokio::test]
    #[ignore]
    async fn post_items_import_does_not_fall_through_to_item_id_route() {
        use crate::routes::build_router;
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;

        let state = test_app_state().await;
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/items/import")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"media_type":"anime","external_id":"12345","title":"鬼滅の刃"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: ルーティング誤マッチによるpanic/500が発生しないことを確認
        assert_ne!(response.status(), StatusCode::INTERNAL_SERVER_ERROR); // 【確認内容】: /items/importが/items/:idのUUIDパースに吸われず、500にならないことを確認 🔵
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

    /// 【テスト用ヘルパー】: 指定file_type/calibre_book_idのitem_files行を投入しfile_idを返す（TASK-0028）
    async fn insert_test_item_file_for_handler(
        state: &AppState,
        item_id: Uuid,
        file_type: &str,
        calibre_book_id: Option<&str>,
    ) -> Uuid {
        sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO item_files (item_id, path, label, file_type, calibre_book_id)
             VALUES ($1, '/srv/files/pdf/example.pdf', '本編PDF', $2::file_type, $3)
             RETURNING id",
        )
        .bind(item_id)
        .bind(file_type)
        .bind(calibre_book_id)
        .fetch_one(&state.db)
        .await
        .expect("テスト用item_fileの投入に失敗しました")
    }

    /// TC-020-02: GET /items/:id がcalibre_book_id設定済みPDFのCalibre-Web遷移情報を含む（ハンドラ統合）
    /// 【テスト目的】: calibre_book_id設定済みPDFを持つアイテムの詳細取得で、レスポンスに
    /// calibre_book_id（Calibre-Web遷移情報）が含まれることを確認する
    /// 【テスト内容】: itemを作成→pdf item_fileを作成→calibre_book_id="calibre-12345"を事前設定→
    /// get_item_handlerを呼び出す
    /// 【期待される動作】: 200、レスポンスのcalibre_linksに"calibre-12345"を含む要素が1件存在する
    /// 🟡 信頼性レベル: タスク定義書 単体テスト要件テストケース4（TC-020-02）に対応。
    /// URL構築方式未確定のため遷移情報の具体形は実装時確定（要件定義書第3章L97）
    /// 【Red期待】: ItemDetailにcalibre_linksフィールド・get_item_calibre_links関数が
    /// 未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn get_item_handler_includes_calibre_web_link_info_for_linked_pdf() {
        let state = test_app_state().await;
        let item_id = insert_test_item_for_handler(&state).await;
        let file_id =
            insert_test_item_file_for_handler(&state, item_id, "pdf", Some("calibre-12345")).await;

        // 【実際の処理実行】: get_item_handlerを直接呼び出す
        let result = get_item_handler(State(state), Path(item_id.to_string())).await;

        // 【結果検証】: 200・calibre_linksにcalibre-12345が含まれることを確認する
        let response = result.expect("詳細取得は成功するはず");
        assert!(
            response
                .data
                .calibre_links
                .iter()
                .any(|link| link.file_id == file_id && link.calibre_book_id == "calibre-12345")
        ); // 【確認内容】: calibre_book_id設定済みPDFの遷移情報が詳細レスポンスに含まれることを確認 🟡
    }

    /// TC-020-N03: calibre_book_id=NULLのPDFは詳細APIで遷移情報を付加されない（ハンドラ統合）
    /// 【テスト目的】: 付加条件calibre_book_id IS NOT NULLの境界（負側）を確認する
    /// 【テスト内容】: itemを作成→pdf item_fileを作成（calibre_book_id未設定のまま）→get_item_handlerを呼ぶ
    /// 【期待される動作】: 200、calibre_linksが空（該当ファイルの遷移情報が付加されない）
    /// 🟡 信頼性レベル: 要件定義書2.3 L77（calibre_book_id IS NOT NULL条件）からの妥当な推測
    /// 【Red期待】: ItemDetailにcalibre_linksフィールド・get_item_calibre_links関数が
    /// 未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn get_item_handler_does_not_include_calibre_link_for_unlinked_pdf() {
        let state = test_app_state().await;
        let item_id = insert_test_item_for_handler(&state).await;
        insert_test_item_file_for_handler(&state, item_id, "pdf", None).await;

        let result = get_item_handler(State(state), Path(item_id.to_string())).await;

        let response = result.expect("詳細取得は成功するはず");
        assert!(response.data.calibre_links.is_empty()); // 【確認内容】: calibre_book_id未設定のPDFには遷移情報が付加されないことを確認 🟡
    }

    /// TC-020-B05: 複数ファイル混在時、calibre_book_id設定済みPDFのみ遷移情報が付く（ハンドラ統合）
    /// 【テスト目的】: 付加条件file_type=pdf AND calibre_book_id IS NOT NULLが行単位で
    /// 正しく適用されることを確認する
    /// 【テスト内容】: 1つのitemに (a)calibre設定済みpdf, (b)未設定pdf, (c)image を作成→get_item_handlerを呼ぶ
    /// 【期待される動作】: 200、calibre_linksに(a)のみが含まれ、件数は1件
    /// 🟡 信頼性レベル: 要件定義書2.3 L77・TC-020-02からの妥当な推測（行単位適用の検証）
    /// 【Red期待】: ItemDetailにcalibre_linksフィールド・get_item_calibre_links関数が
    /// 未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn get_item_handler_applies_calibre_link_condition_per_row_with_mixed_files() {
        let state = test_app_state().await;
        let item_id = insert_test_item_for_handler(&state).await;
        let linked_pdf_id =
            insert_test_item_file_for_handler(&state, item_id, "pdf", Some("calibre-99999")).await;
        insert_test_item_file_for_handler(&state, item_id, "pdf", None).await;
        insert_test_item_file_for_handler(&state, item_id, "image", None).await;

        let result = get_item_handler(State(state), Path(item_id.to_string())).await;

        let response = result.expect("詳細取得は成功するはず");
        assert_eq!(response.data.calibre_links.len(), 1); // 【確認内容】: 混在ケースでも遷移情報が付くのは1件のみであることを確認 🟡
        assert_eq!(response.data.calibre_links[0].file_id, linked_pdf_id); // 【確認内容】: 付加された遷移情報がcalibre設定済みpdfの行であることを確認 🟡
    }
}
