//! cast（キャスト管理）のモデル・リクエストDTO・バリデーション
//!
//! staffとは別テーブル（cast_members/item_cast）で管理するキャスト（声優＋役名）専用のモデル。
//! models/staff.rsと対称な構造（role列を持たない点のみ異なる）。

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};

/// cast本体（`POST /cast`のレスポンスで返す表現）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Cast {
    pub id: Uuid,
    pub external_id: Option<String>,
    pub name: String,
    pub image_url: Option<String>,
    pub created_at: NaiveDateTime,
}

/// item_cast本体（`POST /items/:id/cast`のレスポンスで返す表現）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemCast {
    pub id: Uuid,
    pub item_id: Uuid,
    pub cast_id: Uuid,
    pub character_name: Option<String>,
}

/// item_cast一覧取得（`GET /items/:id/cast`）専用の表現。cast_membersテーブルとJOINしてname/image_urlを含める
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemCastWithName {
    pub id: Uuid,
    pub item_id: Uuid,
    pub cast_id: Uuid,
    pub character_name: Option<String>,
    pub cast_name: String,
    pub cast_image_url: Option<String>,
}

/// `POST /cast` リクエストDTO
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCastRequest {
    pub name: String,
    pub external_id: Option<String>,
    pub image_url: Option<String>,
}

/// `POST /items/:id/cast` リクエストDTO
#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemCastRequest {
    pub cast_id: Uuid,
    pub character_name: Option<String>,
}

/// `GET /cast` クエリパラメータDTO（氏名部分一致検索）
#[derive(Debug, Clone, Deserialize)]
pub struct CastListQuery {
    pub q: Option<String>,
}

/// `GET /cast?q=...` レスポンス表現。既存キャスト検索モーダルで
/// 紐付け作品数を併記するため、cast_membersテーブルの列にlinked_item_countを加える
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CastSearchResult {
    pub id: Uuid,
    pub external_id: Option<String>,
    pub name: String,
    pub image_url: Option<String>,
    pub created_at: NaiveDateTime,
    pub linked_item_count: i64,
}

/// character_name列の最大長（VARCHAR(255)）
pub const CHARACTER_NAME_MAX_LEN: usize = 255;

/// CreateCastRequestのバリデーション（name空文字拒否）を行う
pub fn parse_create_cast_request(
    request: CreateCastRequest,
) -> Result<CreateCastRequest, ApiError> {
    if request.name.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "nameは必須です",
        ));
    }

    if request.name.chars().count() > 255 {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "nameは255文字以内で指定してください",
        ));
    }

    Ok(request)
}

/// CreateItemCastRequestのバリデーション（character_name長さ制限）を行う
pub fn parse_create_item_cast_request(
    request: CreateItemCastRequest,
) -> Result<CreateItemCastRequest, ApiError> {
    if let Some(character_name) = &request.character_name
        && character_name.chars().count() > CHARACTER_NAME_MAX_LEN
    {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            format!("character_nameは{CHARACTER_NAME_MAX_LEN}文字以内で指定してください"),
        ));
    }

    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_create_cast_request_accepts_valid_name() {
        let req = CreateCastRequest {
            name: "声優A".to_string(),
            external_id: None,
            image_url: None,
        };

        let result = parse_create_cast_request(req);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "声優A");
    }

    #[test]
    fn parse_create_cast_request_rejects_empty_name() {
        let req = CreateCastRequest {
            name: "".into(),
            external_id: None,
            image_url: None,
        };

        let result = parse_create_cast_request(req);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().error.code,
            ApiErrorCode::ValidationError.as_code_str()
        );
    }

    #[test]
    fn parse_create_item_cast_request_accepts_character_name_at_max_length() {
        let character_name_255 = "c".repeat(CHARACTER_NAME_MAX_LEN);
        let req = CreateItemCastRequest {
            cast_id: Uuid::new_v4(),
            character_name: Some(character_name_255.clone()),
        };

        let result = parse_create_item_cast_request(req);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().character_name.unwrap().len(),
            CHARACTER_NAME_MAX_LEN
        );
    }

    #[test]
    fn parse_create_item_cast_request_rejects_character_name_exceeding_max_length() {
        let character_name_256 = "c".repeat(CHARACTER_NAME_MAX_LEN + 1);
        let req = CreateItemCastRequest {
            cast_id: Uuid::new_v4(),
            character_name: Some(character_name_256),
        };

        let result = parse_create_item_cast_request(req);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().error.code,
            ApiErrorCode::ValidationError.as_code_str()
        );
    }
}
