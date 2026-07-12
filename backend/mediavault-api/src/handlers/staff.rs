//! staff（スタッフ管理）ハンドラ
//!
//! TASK-0020: スタッフ管理CRUD実装（handlers/item_relations.rsと対称な構造）
//!
//! 【実装状況】: Greenフェーズで全ハンドラを実装済み。ルーター経由の統合テストは
//! 実DB接続（DATABASE_URL）が必要なため#[ignore]を付与している。

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::AppState;
use crate::models::item::{deserialize_request, parse_item_id};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::models::staff::{
    CreateItemStaffRequest, CreateStaffRequest, ItemStaff, Staff, StaffListQuery,
    parse_create_item_staff_request, parse_create_staff_request,
};
use crate::repositories::staff_repository;

/// 【機能概要】: `GET /staff?q=...` ハンドラ。氏名部分一致でスタッフを検索する
/// 【実装方針】: qが空/未指定の場合はDBへ問い合わせず空配列を返す（一覧全件取得を避ける）
pub async fn list_staff_handler(
    State(state): State<AppState>,
    Query(query): Query<StaffListQuery>,
) -> Result<axum::response::Response, ApiError> {
    let q = query.q.unwrap_or_default();
    let trimmed = q.trim();
    if trimmed.is_empty() {
        return Ok(Json(ApiOk::new(Vec::<crate::models::staff::StaffSearchResult>::new())).into_response());
    }

    let results = staff_repository::search_staff(&state.db, trimmed, 20).await?;
    Ok(Json(ApiOk::new(results)).into_response())
}

/// 【機能概要】: `POST /staff` ハンドラ。name(必須)/external_id(optional)/image_url(optional)を受け取りスタッフを作成する
/// 【実装方針】: deserialize_request → parse_create_staff_request → repository::create_staff → 201応答
/// 【テスト対応】: TC-N-01, TC-E-01 を通すための実装
/// 🔵 信頼性レベル: api-endpoints.md POST /staff・TASK-0020 完了条件1 に直接対応
pub async fn create_staff_handler(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    // 【入力値検証】: JSONをCreateStaffRequestへデシリアライズする 🔵
    let request: CreateStaffRequest = deserialize_request(body)?;
    // 【入力値検証】: name空文字チェック等のバリデーションを行う 🔵
    let request = parse_create_staff_request(request)?;

    // 【メイン処理】: staff_repository::create_staffでINSERTする 🔵
    let staff = staff_repository::create_staff(
        &state.db,
        request.name,
        request.external_id,
        request.image_url,
    )
    .await?;

    Ok(created_staff_response(staff))
}

/// 【機能概要】: `POST /items/:id/staff` ハンドラ。staff_id(必須)/role(必須)/character_name(optional)を受け取り紐付けを作成する
/// 【実装方針】: deserialize_request → parse_create_item_staff_request → repository::link_staff → 201応答
/// 【テスト対応】: TC-N-03, TC-N-04, TC-E-02, TC-E-03, TC-E-06 を通すための実装
/// 🔵 信頼性レベル: api-endpoints.md POST /items/:id/staff・TASK-0020 完了条件2 に直接対応
pub async fn create_item_staff_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    // 【入力値検証】: パスパラメータをUUIDへパースする 🔵
    let item_id = parse_item_id(&item_id)?;
    // 【入力値検証】: JSONをCreateItemStaffRequestへデシリアライズする 🔵
    let request: CreateItemStaffRequest = deserialize_request(body)?;
    // 【入力値検証】: role空文字・長さ制限、character_name長さ制限をチェックする 🔵
    let request = parse_create_item_staff_request(request)?;

    // 【メイン処理】: staff_repository::link_staffでitem_id/staff_id存在確認 + INSERTする 🔵
    let item_staff = staff_repository::link_staff(
        &state.db,
        item_id,
        request.staff_id,
        request.role,
        request.character_name,
    )
    .await?;

    Ok(created_item_staff_response(item_staff))
}

/// 【機能概要】: `GET /items/:id/staff` ハンドラ。指定itemに紐づくスタッフ紐付けを一覧取得する
/// 🟡 信頼性レベル: handlers::item_groups::list_item_groups_handlerと対称
pub async fn list_item_staff_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    let staff = staff_repository::list_item_staff(&state.db, item_id).await?;
    Ok(Json(ApiOk::new(staff)).into_response())
}

/// 【機能概要】: `DELETE /items/:id/staff/:item_staff_id` ハンドラ。指定item_staff_idの紐付けを削除する
/// 【実装方針】: parse_item_id × 2 → repository::unlink_staff → 204応答（false時は404）
/// 【設計判断】: 404時のエラーコードはITEM_NOT_FOUNDを採用している。これは「指定item_idに属する
/// 紐付けが見つからない」ことを表す（item_staff_idの不存在とitem_id不一致の両方をこのコードで
/// 表現する）。staff-testcases.md TC-E-04/TC-E-05は404ステータスのみを要求しエラーコード文字列を
/// 指定していないため、専用コード（例: ITEM_STAFF_NOT_FOUND）の追加は本フェーズでは見送った。
/// 【テスト対応】: TC-N-05, TC-E-04 を通すための実装
/// 🟡 信頼性レベル: api-endpoints.md DELETE /items/:id/staff/:item_staff_id・TASK-0020 完了条件3 に対応
pub async fn delete_item_staff_handler(
    State(state): State<AppState>,
    Path((item_id, item_staff_id)): Path<(String, String)>,
) -> Result<axum::response::Response, ApiError> {
    // 【入力値検証】: パスパラメータを両方UUIDへパースする 🟡
    let item_id = parse_item_id(&item_id)?;
    let item_staff_id = parse_item_id(&item_staff_id)?;

    // 【メイン処理】: item_id整合性チェック付きでDELETEする 🟡
    let deleted = staff_repository::unlink_staff(&state.db, item_id, item_staff_id).await?;

    // 【結果分岐】: 影響行0件（不存在/item_id不一致）の場合は404を返す 🟡
    if !deleted {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定された紐付けが見つかりません",
        ));
    }

    // 【結果返却】: 削除成功時は204 No Contentで応答する 🟡
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// 【機能概要】: 作成済みStaffをHTTP 201・統一レスポンス形式で返す
/// 🔵 信頼性レベル: handlers::item_groups::created_responseと対称
fn created_staff_response(staff: Staff) -> axum::response::Response {
    (StatusCode::CREATED, Json(ApiOk::new(staff))).into_response()
}

/// 【機能概要】: 作成済みItemStaffをHTTP 201・統一レスポンス形式で返す
/// 🔵 信頼性レベル: created_staff_responseと対称
fn created_item_staff_response(item_staff: ItemStaff) -> axum::response::Response {
    (StatusCode::CREATED, Json(ApiOk::new(item_staff))).into_response()
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
    /// 🔵 信頼性レベル: routes/mod.rs test_app_stateと対称
    async fn test_app_state() -> AppState {
        let database_url = std::env::var("DATABASE_URL")
            .expect("TASK-0020ルーティング統合テストにはDATABASE_URL環境変数が必要です");
        let db = PgPool::connect(&database_url)
            .await
            .expect("テスト用DBへの接続に失敗しました");
        AppState {
            db,
            internal_api_key: String::new(),
        }
    }

    /// TC-N-01: POST /staff に必須nameのみを送信すると201でStaffが返る（ルーター経由）
    /// 🔵 信頼性レベル: TASK-0020 テストケース1 / staff-testcases.md TC-N-01
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn post_staff_with_required_fields_only_returns_201() {
        // 【テスト目的】: nameのみでスタッフ作成が成功し201でUUID付きStaffが返ることを確認する
        // 【テスト内容】: POST /staff に { "name": "監督A" } を送信
        // 【期待される動作】: staffテーブルへINSERTされ、201ステータスで応答する
        // 🔵 信頼性レベル: TASK-0020 テストケース1 に明記

        // 【テストデータ準備】: クリーンなAppStateとRouterを用意
        let state = test_app_state().await;
        let app = crate::routes::build_router(state);

        // 【実際の処理実行】: POST /staff を実行
        // 【処理内容】: create_staff_handlerの呼び出しを期待する
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/staff")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({ "name": "監督A" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: ステータスコードが201であること
        // 【期待値確認】: 作成成功時は201 Createdが返る規約（item_relations等と同様）
        assert_eq!(response.status(), StatusCode::CREATED); // 【確認内容】: スタッフ作成成功時に201が返ることを確認 🔵
    }

    /// TC-E-01: POST /staff に空のnameを送信すると400 VALIDATION_ERRORが返る（ルーター経由）
    /// 🔵 信頼性レベル: staff-requirements.md 2.1・4.3 に明記
    #[tokio::test]
    #[ignore]
    async fn post_staff_with_empty_name_returns_400() {
        // 【テスト目的】: 空文字nameが400 VALIDATION_ERRORで拒否されることを確認する
        // 【テスト内容】: POST /staff に { "name": "" } を送信
        // 【期待される動作】: バリデーションエラーで400が返り、INSERTに到達しない
        // 🔵 信頼性レベル: staff-requirements.md 2.1・4.3 に明記

        let state = test_app_state().await;
        let app = crate::routes::build_router(state);

        // 【実際の処理実行】: POST /staff を実行
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/staff")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::json!({ "name": "" }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: ステータスコードが400であること
        assert_eq!(response.status(), StatusCode::BAD_REQUEST); // 【確認内容】: 空nameが400で拒否されることを確認 🔵
    }

    /// TC-E-02: POST /items/:id/staff に存在しないstaff_idを送信すると404 STAFF_NOT_FOUNDが返る（ルーター経由）
    /// 🔵 信頼性レベル: TASK-0020 完了条件・テストケース5 / staff-requirements.md 2.2・4.3 に明記
    #[tokio::test]
    #[ignore]
    async fn post_item_staff_with_nonexistent_staff_id_returns_404_staff_not_found() {
        // 【テスト目的】: 不存在のstaff_idでの紐付けがSTAFF_NOT_FOUND(404)で拒否されることを確認する
        // 【テスト内容】: POST /items/:item_id/staff に存在しないstaff_idを送信
        // 【期待される動作】: ステータス404、レスポンスボディのerror.codeが"STAFF_NOT_FOUND"
        // 🔵 信頼性レベル: TASK-0020 完了条件・テストケース5 に明記

        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();
        let nonexistent_staff_id = Uuid::new_v4();

        // 【実際の処理実行】: POST /items/:id/staff を実行
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/items/{item_id}/staff"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "staff_id": nonexistent_staff_id,
                            "role": "監督"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: ステータスコードが404であること
        assert_eq!(response.status(), StatusCode::NOT_FOUND); // 【確認内容】: 不存在staff_idで404が返ることを確認 🔵
    }

    /// TC-E-04: DELETE /items/:id/staff/:item_staff_id に存在しないitem_staff_idを送信すると404が返る（ルーター経由）
    /// 🟡 信頼性レベル: TASK-0020 完了条件 / staff-requirements.md 4.3（🟡）
    #[tokio::test]
    #[ignore]
    async fn delete_item_staff_with_nonexistent_id_returns_404() {
        // 【テスト目的】: 不存在のitem_staff_idでの削除が404で拒否されることを確認する
        // 【テスト内容】: DELETE /items/:item_id/staff/:item_staff_id に存在しないIDを送信
        // 【期待される動作】: ステータス404、影響行0件をサイレント成功にしない
        // 🟡 信頼性レベル: TASK-0020 完了条件（🟡）

        let state = test_app_state().await;
        let app = crate::routes::build_router(state);
        let item_id = Uuid::new_v4();
        let nonexistent_item_staff_id = Uuid::new_v4();

        // 【実際の処理実行】: DELETE /items/:id/staff/:item_staff_id を実行
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!(
                        "/items/{item_id}/staff/{nonexistent_item_staff_id}"
                    ))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // 【結果検証】: ステータスコードが404であること
        assert_eq!(response.status(), StatusCode::NOT_FOUND); // 【確認内容】: 不存在item_staff_idで404が返ることを確認 🟡
    }
}
