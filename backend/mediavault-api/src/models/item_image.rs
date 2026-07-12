//! item_images（画像URL）のモデル・リクエストDTO・バリデーション
//!
//! item_links.rs（models/item_link.rs）と対称な構造。labelを持たない最小構成。

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};

/// item_images本体（`GET/POST /items/:id/images`のレスポンスで返す表現）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemImage {
    pub id: Uuid,
    pub item_id: Uuid,
    pub url: String,
    pub created_at: NaiveDateTime,
}

/// `POST /items/:id/images` リクエストDTO
#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemImageRequest {
    pub url: String,
}

/// CreateItemImageRequestのバリデーション（url空文字拒否）を行う
pub fn parse_create_item_image_request(
    request: CreateItemImageRequest,
) -> Result<CreateItemImageRequest, ApiError> {
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
    fn create_item_image_request_deserializes_valid_url() {
        let value = serde_json::json!({ "url": "https://example.com/image.jpg" });
        let request: CreateItemImageRequest = serde_json::from_value(value).unwrap();
        assert_eq!(request.url, "https://example.com/image.jpg");
    }

    #[test]
    fn parse_create_item_image_request_rejects_empty_url() {
        let request = CreateItemImageRequest {
            url: "".to_string(),
        };

        let result = parse_create_item_image_request(request);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().error.code,
            ApiErrorCode::ValidationError.as_code_str()
        );
    }

    #[test]
    fn parse_create_item_image_request_rejects_blank_url() {
        let request = CreateItemImageRequest {
            url: "   ".to_string(),
        };

        let result = parse_create_item_image_request(request);

        assert!(result.is_err());
    }

    #[test]
    fn parse_create_item_image_request_accepts_valid_url() {
        let request = CreateItemImageRequest {
            url: "https://example.com/image.jpg".to_string(),
        };

        let result = parse_create_item_image_request(request);

        assert!(result.is_ok());
    }
}
