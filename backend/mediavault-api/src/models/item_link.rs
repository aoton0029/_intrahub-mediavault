//! item_links（参考リンク）のモデル・リクエストDTO・バリデーション
//!
//! TASK-0021: item_links CRUD実装（models/item_relation.rsと対称な構造）

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};

/// item_links本体（`POST /items/:id/links`のレスポンスで返す表現）
/// 🔵 信頼性レベル: database-schema.sqlのitem_linksテーブル定義に直接対応
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemLink {
    pub id: Uuid,
    pub item_id: Uuid,
    pub url: String,
    pub label: String,
    pub created_at: NaiveDateTime,
}

/// `POST /items/:id/links` リクエストDTO
/// 🔵 信頼性レベル: タスク仕様「url(必須), label(必須)を受け取る」に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemLinkRequest {
    pub url: String,
    pub label: String,
}

/// 【機能概要】: CreateItemLinkRequestのバリデーション（url空文字拒否）を行う
/// 🟡 信頼性レベル: タスク仕様「urlが空文字の場合、VALIDATION_ERROR（400）を返す」より
pub fn parse_create_item_link_request(
    request: CreateItemLinkRequest,
) -> Result<CreateItemLinkRequest, ApiError> {
    if request.url.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "urlは必須です",
        ));
    }
    if request.label.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "labelは必須です",
        ));
    }
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース1: CreateItemLinkRequestの正常デシリアライズ
    #[test]
    fn create_item_link_request_deserializes_valid_fields() {
        let value = serde_json::json!({
            "url": "https://example.com",
            "label": "公式サイト"
        });

        let request: CreateItemLinkRequest = serde_json::from_value(value).unwrap();

        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.label, "公式サイト");
    }

    /// テストケース4: url空文字で400
    #[test]
    fn parse_create_item_link_request_rejects_empty_url() {
        let request = CreateItemLinkRequest {
            url: "".to_string(),
            label: "公式サイト".to_string(),
        };

        let result = parse_create_item_link_request(request);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().error.code,
            ApiErrorCode::ValidationError.as_code_str()
        );
    }

    /// label空文字で400
    #[test]
    fn parse_create_item_link_request_rejects_empty_label() {
        let request = CreateItemLinkRequest {
            url: "https://example.com".to_string(),
            label: "".to_string(),
        };

        let result = parse_create_item_link_request(request);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().error.code,
            ApiErrorCode::ValidationError.as_code_str()
        );
    }

    /// 正常な入力は検証を通過する
    #[test]
    fn parse_create_item_link_request_accepts_valid_fields() {
        let request = CreateItemLinkRequest {
            url: "https://example.com".to_string(),
            label: "公式サイト".to_string(),
        };

        let result = parse_create_item_link_request(request);

        assert!(result.is_ok());
    }
}
