//! item_images（画像URL）のモデル・リクエストDTO・バリデーション
//!
//! item_links.rs（models/item_link.rs）と対称な構造。labelの代わりに
//! kind（画像種別）・source（収集元）・sort_order（表示順）を持つ。

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};

/// 画像の種別（DBのimage_kind enumに対応）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, Default)]
#[sqlx(type_name = "image_kind", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ImageKind {
    Cover,
    Backdrop,
    Screenshot,
    Thumbnail,
    #[default]
    Other,
}

/// 画像の収集元（DBのimage_source enumに対応）。
/// 手動追加は`manual`、外部APIインポート時は取得プロバイダ名が入る。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, Default)]
#[sqlx(type_name = "image_source", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ImageSource {
    #[default]
    Manual,
    Annict,
    Jikan,
    Tmdb,
    Rakuten,
    Steam,
    Ndl,
}

/// item_images本体（`GET/POST /items/:id/images`のレスポンスで返す表現）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemImage {
    pub id: Uuid,
    pub item_id: Uuid,
    pub url: String,
    pub kind: ImageKind,
    pub source: ImageSource,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
}

/// item作成/インポート時にitem_imagesへ登録する画像1件分の入力。
///
/// `CreateItemRequest.additional_images`の要素。手動作成（`POST /items`）では
/// kind省略時は`other`、sourceは常にサーバー側で`manual`固定とする（リクエストからは受け取らない）。
/// インポート経路では`ExternalSearchService`の各`build_*_create_request`が
/// プロバイダに応じたkind/sourceを明示的に設定する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewItemImage {
    pub url: String,
    #[serde(default)]
    pub kind: ImageKind,
    #[serde(skip, default)]
    pub source: ImageSource,
}

impl NewItemImage {
    pub fn new(url: String, kind: ImageKind, source: ImageSource) -> Self {
        Self { url, kind, source }
    }
}

/// `POST /items/:id/images` リクエストDTO
#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemImageRequest {
    pub url: String,
    /// 画像種別。省略時は`other`
    #[serde(default)]
    pub kind: ImageKind,
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
        assert_eq!(request.kind, ImageKind::Other);
    }

    #[test]
    fn create_item_image_request_deserializes_explicit_kind() {
        let value =
            serde_json::json!({ "url": "https://example.com/shot.jpg", "kind": "screenshot" });
        let request: CreateItemImageRequest = serde_json::from_value(value).unwrap();
        assert_eq!(request.kind, ImageKind::Screenshot);
    }

    #[test]
    fn parse_create_item_image_request_rejects_empty_url() {
        let request = CreateItemImageRequest {
            url: "".to_string(),
            kind: ImageKind::Other,
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
            kind: ImageKind::Other,
        };

        let result = parse_create_item_image_request(request);

        assert!(result.is_err());
    }

    #[test]
    fn parse_create_item_image_request_accepts_valid_url() {
        let request = CreateItemImageRequest {
            url: "https://example.com/image.jpg".to_string(),
            kind: ImageKind::Cover,
        };

        let result = parse_create_item_image_request(request);

        assert!(result.is_ok());
    }

    #[test]
    fn new_item_image_deserializes_with_defaults_and_ignores_source() {
        // sourceは#[serde(skip)]のためリクエストから注入できず、常にManualになる
        let value = serde_json::json!({ "url": "https://example.com/a.jpg", "source": "tmdb" });
        let image: NewItemImage = serde_json::from_value(value).unwrap();
        assert_eq!(image.kind, ImageKind::Other);
        assert_eq!(image.source, ImageSource::Manual);
    }
}
