//! item_episodes（season/chapter配下の話数）のモデル・リクエストDTO
//!
//! TASK-0019: item_episodes CRUD + EDGE-101検証実装（models/item_group.rsと対称な構造）

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// item_episodes本体（`POST/GET /groups/:group_id/episodes`のレスポンスで返す表現）
/// 🔵 信頼性レベル: database-schema.sqlのitem_episodesテーブル定義に直接対応
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemEpisode {
    pub id: Uuid,
    pub group_id: Uuid,
    pub episode_number: i32,
    pub title: Option<String>,
    pub original_title: Option<String>,
    pub air_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// `POST /groups/:group_id/episodes` リクエストDTO
/// 🔵 信頼性レベル: TASK-0019.md 実装詳細1（CreateItemEpisodeRequest DTO定義）に直接対応
#[derive(Debug, Clone, Deserialize)]
pub struct CreateItemEpisodeRequest {
    pub episode_number: i32,
    pub title: Option<String>,
    pub original_title: Option<String>,
    pub air_date: Option<NaiveDate>,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース1: CreateItemEpisodeRequestの正常デシリアライズ（episode_numberのみ必須）
    /// 🔵 信頼性レベル: api-endpoints.md POST /groups/:group_id/episodes リクエスト例に直接対応
    #[test]
    fn create_item_episode_request_deserializes_with_required_field_only() {
        let value = serde_json::json!({ "episode_number": 1 });

        let request: CreateItemEpisodeRequest = serde_json::from_value(value).unwrap();

        assert_eq!(request.episode_number, 1);
        assert_eq!(request.title, None);
        assert_eq!(request.original_title, None);
        assert_eq!(request.air_date, None);
        assert_eq!(request.description, None);
    }

    /// テストケース2: CreateItemEpisodeRequestの全フィールド指定時の正常デシリアライズ
    /// 🔵 信頼性レベル: api-endpoints.md POST /groups/:group_id/episodes リクエスト例に直接対応
    #[test]
    fn create_item_episode_request_deserializes_with_all_fields() {
        let value = serde_json::json!({
            "episode_number": 1,
            "title": "第1話",
            "original_title": "Episode 1",
            "air_date": "2026-01-05",
            "description": "あらすじ"
        });

        let request: CreateItemEpisodeRequest = serde_json::from_value(value).unwrap();

        assert_eq!(request.episode_number, 1);
        assert_eq!(request.title, Some("第1話".to_string()));
        assert_eq!(request.original_title, Some("Episode 1".to_string()));
        assert_eq!(
            request.air_date,
            Some(NaiveDate::from_ymd_opt(2026, 1, 5).unwrap())
        );
        assert_eq!(request.description, Some("あらすじ".to_string()));
    }

    /// テストケース3: episode_numberが欠落している場合はデシリアライズエラーになる
    /// 🔵 信頼性レベル: episode_numberが必須フィールドであることに直接対応
    #[test]
    fn create_item_episode_request_without_episode_number_fails_deserialization() {
        let value = serde_json::json!({ "title": "第1話" });

        let result: Result<CreateItemEpisodeRequest, _> = serde_json::from_value(value);

        assert!(result.is_err());
    }
}
