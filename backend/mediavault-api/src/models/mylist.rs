//! マイリストのモデル・リクエストDTO・バリデーション
//!
//! TASK-0016: マイリストCRUD実装（models/category.rsと対称な構造）

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};

/// マイリスト本体（`POST /mylists`のレスポンスで返す表現）
/// 🔵 信頼性レベル: database-schema.sqlのmylistsテーブル定義（id, name, created_at）に直接対応
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Mylist {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
}

/// `POST /mylists` リクエストDTO
/// 🔵 信頼性レベル: タスク仕様「nameを受け取り作成する」に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct CreateMylistRequest {
    pub name: String,
}

/// `POST /mylists/:id/items` リクエストDTO
/// 🔵 信頼性レベル: タスク仕様「item_idを受け取り複合キーでINSERTする」に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct AddMylistItemRequest {
    pub item_id: Uuid,
}

/// `PATCH /mylists/:id` リクエストDTO（名前変更）
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMylistRequest {
    pub name: String,
}

/// マイリスト一覧表示用（`GET /mylists`のレスポンスで返す表現）。
/// カバー画像候補（最大4件）と所属item数を付与件数として持つ。
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MylistWithCovers {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub item_count: i64,
    pub cover_urls: Vec<String>,
}

/// 【機能概要】: マイリスト名が空白のみでないことを検証する
/// 【実装方針】: validate_category_name（models/category.rs）と対称な実装
/// 🟡 信頼性レベル: validate_category_nameと同様の妥当な推測
pub fn validate_mylist_name(name: &str) -> Result<(), ApiError> {
    if name.trim().is_empty() {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "マイリスト名は空にできません",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// マイリスト作成リクエストの正常デシリアライズ
    #[test]
    fn create_mylist_request_deserializes_valid_name() {
        let value = serde_json::json!({ "name": "今期視聴予定" });

        let request: CreateMylistRequest = serde_json::from_value(value).unwrap();

        assert_eq!(request.name, "今期視聴予定");
    }

    /// マイリスト名が空文字でVALIDATION_ERRORになる
    #[test]
    fn create_mylist_with_empty_name_returns_validation_error() {
        let empty = "";
        let blank = "   ";

        let empty_result = validate_mylist_name(empty);
        let blank_result = validate_mylist_name(blank);

        assert!(empty_result.is_err());
        assert!(blank_result.is_err());
    }

    /// name未指定でのデシリアライズエラー
    #[test]
    fn create_mylist_request_missing_name_field_fails_deserialization() {
        let value = serde_json::json!({});

        let result: Result<CreateMylistRequest, _> = serde_json::from_value(value);

        assert!(result.is_err());
    }

    /// item_idを受け取るリクエストの正常デシリアライズ
    #[test]
    fn add_mylist_item_request_deserializes_valid_item_id() {
        let item_id = Uuid::new_v4();
        let value = serde_json::json!({ "item_id": item_id });

        let request: AddMylistItemRequest = serde_json::from_value(value).unwrap();

        assert_eq!(request.item_id, item_id);
    }
}
