//! 設定関連ハンドラ（外部APIキー管理）
//!
//! TASK-0022: api_credentials（外部APIキー管理）CRUD実装
//!
//! 🔵 信頼性レベル: 要件定義書REQ-0022-01〜06・データフロー（第2章）より

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::models::api_credential::{parse_api_provider, ApiCredential, UpdateApiKeyRequest};
use crate::models::item::deserialize_request;
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::api_credential_repository;
use crate::AppState;

/// `PUT /settings/api-keys/:provider` ハンドラ
///
/// 【機能概要】: パスパラメータproviderをApiProviderに変換し、リクエストボディのapi_keyでupsertする
/// 【実装方針】: provider変換失敗時はDBアクセス前にInvalidProvider（400）を返す。
/// これによりprovider不正時は不要なDB接続・クエリが発生しない（早期リターン）
/// 【セキュリティ補足】: api_keyの平文はレスポンスボディに含まれる（仕様上の許容、REQ-015/NFR-202で
/// 暗号化は対象外と明記）が、サーバーログには出力しない設計（repository層db_error参照）
/// 🔵 信頼性レベル: 要件定義書「データフロー」L53・REQ-0022-01/05/101/102より
pub async fn update_api_key_handler(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<axum::response::Response, ApiError> {
    let request: UpdateApiKeyRequest = deserialize_request(body)?;

    let provider = parse_api_provider(&provider).ok_or_else(|| {
        ApiError::new(
            ApiErrorCode::InvalidProvider,
            "不正なproviderが指定されました",
        )
    })?;

    let credential =
        api_credential_repository::upsert_api_credential(&state.db, provider, request.api_key)
            .await?;

    Ok(ok_response(credential))
}

fn ok_response(credential: ApiCredential) -> axum::response::Response {
    (StatusCode::OK, Json(ApiOk::new(credential))).into_response()
}

#[cfg(test)]
mod tests {
    // 本ファイル内のユニットテストは無し（provider変換・DTOデシリアライズはmodels/api_credential.rsで検証済み）。
    // ハンドラのHTTPレベル検証（TC-015-01-C・TC-015-02-B・TC-NEW-02/04）はroutes/mod.rsの統合テストに配置する
    // （note.md「テスト規約」・既存test_app_state()パターンに合流させる方針）。
}
