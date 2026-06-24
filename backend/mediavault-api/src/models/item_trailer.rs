//! item_trailers（トレーラー動画リンク）のモデル・リクエストDTO・バリデーション
//!
//! TASK-0021: item_trailers CRUD実装（models/item_link.rsと対称な構造。labelはoptional）

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};

/// item_trailers本体（`POST /items/:id/trailers`のレスポンスで返す表現）
/// 🔵 信頼性レベル: database-schema.sqlのitem_trailersテーブル定義に直接対応
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemTrailer {
    pub id: Uuid,
    pub item_id: Uuid,
    pub url: String,
    pub label: Option<String>,
    pub created_at: NaiveDateTime,
}

/// `POST /items/:id/trailers` リクエストDTO
/// 🔵 信頼性レベル: タスク仕様「url(必須), label(optional)を受け取る」に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemTrailerRequest {
    pub url: String,
    pub label: Option<String>,
}

/// 【機能概要】: CreateItemTrailerRequestのバリデーション（url空文字拒否）を行う
/// 🟡 信頼性レベル: タスク仕様「urlが空文字の場合、VALIDATION_ERROR（400）を返す」より
pub fn parse_create_item_trailer_request(
    request: CreateItemTrailerRequest,
) -> Result<CreateItemTrailerRequest, ApiError> {
    if request.url.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "urlは必須です",
        ));
    }
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース3: CreateItemTrailerRequestの正常デシリアライズ（labelなし）
    #[test]
    fn create_item_trailer_request_deserializes_without_label() {
        let value = serde_json::json!({
            "url": "https://youtube.com/xxx"
        });

        let request: CreateItemTrailerRequest = serde_json::from_value(value).unwrap();

        assert_eq!(request.url, "https://youtube.com/xxx");
        assert!(request.label.is_none());
    }

    /// labelを指定した場合は保持される
    #[test]
    fn create_item_trailer_request_deserializes_with_label() {
        let value = serde_json::json!({
            "url": "https://youtube.com/xxx",
            "label": "予告編"
        });

        let request: CreateItemTrailerRequest = serde_json::from_value(value).unwrap();

        assert_eq!(request.label, Some("予告編".to_string()));
    }

    /// url空文字で400
    #[test]
    fn parse_create_item_trailer_request_rejects_empty_url() {
        let request = CreateItemTrailerRequest {
            url: "".to_string(),
            label: None,
        };

        let result = parse_create_item_trailer_request(request);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().error.code,
            ApiErrorCode::ValidationError.as_code_str()
        );
    }

    /// 正常な入力は検証を通過する
    #[test]
    fn parse_create_item_trailer_request_accepts_valid_url() {
        let request = CreateItemTrailerRequest {
            url: "https://youtube.com/xxx".to_string(),
            label: None,
        };

        let result = parse_create_item_trailer_request(request);

        assert!(result.is_ok());
    }
}
