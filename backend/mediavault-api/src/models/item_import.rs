//! POST /items/import 用リクエストDTO・バリデーション
//!
//! TASK-0025: POST /items/import（外部検索結果からのアイテムインポート）実装
//!
//! 【models/domain全廃止に伴うリファクタ】: 旧`ImportItemRequest`（フル`MediaDetails`を
//! `CreateItemRequest`へ1:1マッピングするだけの中間DTO）は廃止した。フロントエンドは
//! `GET /items/search`が返す軽量`SearchResultItem`のid/media_type/providerのみを
//! `POST /items/import`へ送り返し、サーバー側で該当プロバイダへ詳細を再取得したうえで
//! `CreateItemRequest`を直接構築する（`services::external_search`の
//! `ExternalSearchService::fetch_import_details`が担う）。

use crate::models::api_credential::ApiProvider;
use crate::models::item::MediaType;
use crate::models::response::{ApiError, ApiErrorCode};

/// `POST /items/import` のリクエストボディ
///
/// 【機能概要】: 外部検索結果から選択した1件のインポートを要求するための最小DTO。
/// `external_id`はAPI起源アイテムの必須項目（DB CHECK制約 chk_items_source_external_id対象）。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ImportByIdRequest {
    pub media_type: MediaType,
    pub provider: Option<ApiProvider>,
    /// 外部API上の一意ID。空文字・欠落は400 VALIDATION_ERROR対象（必須・非Option）
    pub external_id: String,
}

/// `external_id`が空文字・空白のみでないことを検証する。
///
/// 【機能概要】: ImportByIdRequest.external_idの必須バリデーション。
/// trim().is_empty()基準で既存`validate_title`と判定基準を揃える。
/// 🔵 信頼性レベル: item-import-requirements.md 2.1「空文字・欠落は400」、
/// item-import-testcases.md TC-0025-E01〜E03より
pub fn validate_external_id(external_id: &str) -> Result<(), ApiError> {
    if external_id.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "external_idは空にできません",
        ));
    }
    Ok(())
}

/// JSON値を`ImportByIdRequest`としてデシリアライズし、バリデーションを行う。
///
/// media_type不正・external_id欠落や空文字は400 VALIDATION_ERRORへ変換する。
pub fn parse_import_by_id_request(value: serde_json::Value) -> Result<ImportByIdRequest, ApiError> {
    let request: ImportByIdRequest = crate::models::item::deserialize_request(value)?;
    validate_external_id(&request.external_id)?;
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    /// TC-0025-N01: ImportByIdRequestが必須項目（media_type/external_id）のみで正常にデシリアライズされる
    #[test]
    fn import_by_id_request_deserializes_minimal_fields() {
        let value = serde_json::json!({
            "media_type": "anime",
            "external_id": "12345"
        });

        let request = parse_import_by_id_request(value).unwrap();
        assert_eq!(request.media_type, MediaType::Anime);
        assert_eq!(request.external_id, "12345");
        assert_eq!(request.provider, None);
    }

    /// providerが指定された場合はSome(ApiProvider)へデシリアライズされる
    #[test]
    fn import_by_id_request_deserializes_with_provider() {
        let value = serde_json::json!({
            "media_type": "movie",
            "provider": "tmdb",
            "external_id": "603"
        });

        let request = parse_import_by_id_request(value).unwrap();
        assert_eq!(request.media_type, MediaType::Movie);
        assert_eq!(request.provider, Some(ApiProvider::Tmdb));
        assert_eq!(request.external_id, "603");
    }

    /// TC-0025-E01: external_id欠落 → 400 VALIDATION_ERROR
    /// 【補足】: external_idはString型（非Option）のためserdeデシリアライズ自体が失敗し
    /// VALIDATION_ERRORになる
    #[test]
    fn import_by_id_request_missing_external_id_returns_validation_error() {
        let value = serde_json::json!({ "media_type": "anime" });

        let err = parse_import_by_id_request(value).unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    /// TC-0025-E02: external_id空文字 → 400 VALIDATION_ERROR
    #[test]
    fn import_by_id_request_empty_external_id_returns_validation_error() {
        let value = serde_json::json!({
            "media_type": "anime",
            "external_id": ""
        });

        let err = parse_import_by_id_request(value).unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    /// TC-0025-E03: external_id空白のみ（"   "） → 400 VALIDATION_ERROR
    #[test]
    fn import_by_id_request_blank_external_id_returns_validation_error() {
        let value = serde_json::json!({
            "media_type": "anime",
            "external_id": "   "
        });

        let err = parse_import_by_id_request(value).unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    /// TC-0025-E04: media_type不正値 → 400 VALIDATION_ERROR
    #[test]
    fn import_by_id_request_invalid_media_type_returns_validation_error() {
        let value = serde_json::json!({
            "media_type": "invalid_type",
            "external_id": "12345"
        });

        let err = parse_import_by_id_request(value).unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    /// validate_external_idの単体動作確認（trim().is_empty()基準）
    #[test]
    fn validate_external_id_rejects_blank_and_accepts_nonblank() {
        assert!(validate_external_id("").is_err());
        assert!(validate_external_id("   ").is_err());
        assert!(validate_external_id("12345").is_ok());
    }

    // 【カバレッジ移動メモ】: 旧`ImportItemRequest`のMediaDetailsコア項目マッピング
    // （image_url→cover_image_url、release_date文字列→NaiveDate等）のテストは、
    // 変換ロジック自体が`services::external_search`のプロバイダ別
    // `build_*_create_request`関数へ移動したため、当該関数群のテスト
    // （services/external_search.rs内）へ移設した。
}
