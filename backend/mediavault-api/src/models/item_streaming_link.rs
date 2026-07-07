//! item_streaming_links（配信サービスURL）のモデル・リクエストDTO・バリデーション
//!
//! item_link.rsと対称な構造。プラットフォームは固定5種類のenumとし、
//! 1アイテムにつき1プラットフォーム1件（UNIQUE(item_id, platform)）とする。

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};

/// 配信プラットフォーム種別（固定5種類）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "streaming_platform", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum StreamingPlatform {
    Netflix,
    AmazonPrime,
    DisneyPlus,
    DmmTv,
    AppleTv,
}

/// item_streaming_links本体（`POST /items/:id/streaming-links`のレスポンスで返す表現）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemStreamingLink {
    pub id: Uuid,
    pub item_id: Uuid,
    pub platform: StreamingPlatform,
    pub url: String,
    pub created_at: NaiveDateTime,
}

/// `POST /items/:id/streaming-links` リクエストDTO
#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemStreamingLinkRequest {
    pub platform: StreamingPlatform,
    pub url: String,
}

/// 【機能概要】: CreateItemStreamingLinkRequestのバリデーション（url空文字拒否）を行う
pub fn parse_create_item_streaming_link_request(
    request: CreateItemStreamingLinkRequest,
) -> Result<CreateItemStreamingLinkRequest, ApiError> {
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

    #[test]
    fn create_item_streaming_link_request_deserializes_valid_fields() {
        let value = serde_json::json!({
            "platform": "netflix",
            "url": "https://www.netflix.com/title/12345"
        });

        let request: CreateItemStreamingLinkRequest = serde_json::from_value(value).unwrap();

        assert_eq!(request.platform, StreamingPlatform::Netflix);
        assert_eq!(request.url, "https://www.netflix.com/title/12345");
    }

    #[test]
    fn parse_create_item_streaming_link_request_rejects_empty_url() {
        let request = CreateItemStreamingLinkRequest {
            platform: StreamingPlatform::AppleTv,
            url: "".to_string(),
        };

        let result = parse_create_item_streaming_link_request(request);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().error.code,
            ApiErrorCode::ValidationError.as_code_str()
        );
    }

    #[test]
    fn parse_create_item_streaming_link_request_accepts_valid_fields() {
        let request = CreateItemStreamingLinkRequest {
            platform: StreamingPlatform::DisneyPlus,
            url: "https://www.disneyplus.com/xxx".to_string(),
        };

        let result = parse_create_item_streaming_link_request(request);

        assert!(result.is_ok());
    }
}
