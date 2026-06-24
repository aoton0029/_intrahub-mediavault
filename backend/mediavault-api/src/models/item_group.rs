//! item_groups（シーズン/巻/章）のモデル・リクエストDTO
//!
//! TASK-0018: item_groups CRUD実装（models/item_relation.rsと対称な構造）

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// グループ種別（`season`/`volume`/`chapter`）
/// 🔵 信頼性レベル: database-schema.sqlのgroup_type ENUM定義に直接対応
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "group_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum GroupType {
    Season,
    Volume,
    Chapter,
}

/// item_groups本体（`POST/GET /items/:id/groups`のレスポンスで返す表現）
/// 🔵 信頼性レベル: database-schema.sqlのitem_groupsテーブル定義に直接対応
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemGroup {
    pub id: Uuid,
    pub item_id: Uuid,
    pub parent_item_id: Option<Uuid>,
    pub group_type: GroupType,
    pub group_name: String,
    pub number: Option<i32>,
    pub display_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// `POST /items/:id/groups` リクエストDTO
/// 🔵 信頼性レベル: api-endpoints.md POST /items/:id/groups リクエスト例に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemGroupRequest {
    pub group_type: GroupType,
    pub group_name: String,
    pub number: Option<i32>,
    #[serde(default)]
    pub display_order: i32,
    pub parent_item_id: Option<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース1: CreateItemGroupRequestの正常デシリアライズ（display_order未指定時は0）
    #[test]
    fn create_item_group_request_deserializes_with_default_display_order() {
        let value = serde_json::json!({
            "group_type": "season",
            "group_name": "シーズン1",
            "number": 1
        });

        let request: CreateItemGroupRequest = serde_json::from_value(value).unwrap();

        assert_eq!(request.group_type, GroupType::Season);
        assert_eq!(request.group_name, "シーズン1");
        assert_eq!(request.number, Some(1));
        assert_eq!(request.display_order, 0);
        assert_eq!(request.parent_item_id, None);
    }

    /// テストケース2: 巻グループ作成リクエストの正常デシリアライズ
    #[test]
    fn create_item_group_request_deserializes_volume_type() {
        let value = serde_json::json!({
            "group_type": "volume",
            "group_name": "第1巻",
            "number": 1
        });

        let request: CreateItemGroupRequest = serde_json::from_value(value).unwrap();

        assert_eq!(request.group_type, GroupType::Volume);
    }

    /// テストケース3: group_typeが不正値の場合はデシリアライズエラーになる
    #[test]
    fn create_item_group_request_with_invalid_group_type_fails_deserialization() {
        let value = serde_json::json!({
            "group_type": "invalid",
            "group_name": "不正グループ"
        });

        let result: Result<CreateItemGroupRequest, _> = serde_json::from_value(value);

        assert!(result.is_err());
    }

    /// parent_item_idを指定した入れ子構造リクエストの正常デシリアライズ
    #[test]
    fn create_item_group_request_deserializes_with_parent_item_id() {
        let parent_id = Uuid::new_v4();
        let value = serde_json::json!({
            "group_type": "chapter",
            "group_name": "第1章",
            "parent_item_id": parent_id
        });

        let request: CreateItemGroupRequest = serde_json::from_value(value).unwrap();

        assert_eq!(request.parent_item_id, Some(parent_id));
    }
}
