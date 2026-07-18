//! バックアップ（エクスポート/インポート）用DTO
//!
//! GET /backup/export・POST /backup/import で使う、テーブル列と1:1対応の
//! 行DTOとバージョン付きエンベロープ。API応答用モデルとは独立させ、
//! バックアップファイル形式をAPIの変更から切り離す。

use std::collections::BTreeMap;

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::item::{ItemSource, ItemStatus, MediaType};
use crate::models::item_file::FileType;
use crate::models::item_group::GroupType;
use crate::models::item_image::{ImageKind, ImageSource};
use crate::models::item_relation::RelationType;
use crate::models::item_streaming_link::StreamingPlatform;

/// バックアップファイル形式のバージョン。互換性を壊す変更時にインクリメントする
pub const SCHEMA_VERSION: u32 = 1;

/// バックアップファイル全体のエンベロープ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFile {
    pub schema_version: u32,
    pub exported_at: NaiveDateTime,
    pub data: BackupData,
}

/// 全対象テーブルの行データ。配列が欠落したファイルも受理できるよう全て default
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackupData {
    #[serde(default)]
    pub items: Vec<BackupItem>,
    #[serde(default)]
    pub tags: Vec<BackupTag>,
    #[serde(default)]
    pub item_tags: Vec<BackupItemTag>,
    #[serde(default)]
    pub categories: Vec<BackupCategory>,
    #[serde(default)]
    pub item_categories: Vec<BackupItemCategory>,
    #[serde(default)]
    pub mylists: Vec<BackupMylist>,
    #[serde(default)]
    pub mylist_items: Vec<BackupMylistItem>,
    #[serde(default)]
    pub item_relations: Vec<BackupItemRelation>,
    #[serde(default)]
    pub item_links: Vec<BackupItemLink>,
    #[serde(default)]
    pub item_trailers: Vec<BackupItemTrailer>,
    #[serde(default)]
    pub item_streaming_links: Vec<BackupItemStreamingLink>,
    #[serde(default)]
    pub item_images: Vec<BackupItemImage>,
    #[serde(default)]
    pub item_files: Vec<BackupItemFile>,
    #[serde(default)]
    pub item_groups: Vec<BackupItemGroup>,
    #[serde(default)]
    pub item_episodes: Vec<BackupItemEpisode>,
    #[serde(default)]
    pub staff: Vec<BackupStaff>,
    #[serde(default)]
    pub item_staff: Vec<BackupItemStaff>,
    #[serde(default)]
    pub cast_members: Vec<BackupCastMember>,
    #[serde(default)]
    pub item_cast: Vec<BackupItemCast>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupItem {
    pub id: Uuid,
    pub media_type: MediaType,
    pub title: String,
    pub original_title: Option<String>,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub homepage_url: Option<String>,
    pub status: ItemStatus,
    pub consumed_date: Option<NaiveDate>,
    pub rating: Option<f32>,
    pub is_favorite: bool,
    pub source: ItemSource,
    pub external_id: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupTag {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupItemTag {
    pub item_id: Uuid,
    pub tag_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupCategory {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupItemCategory {
    pub item_id: Uuid,
    pub category_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupMylist {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupMylistItem {
    pub mylist_id: Uuid,
    pub item_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupItemRelation {
    pub id: Uuid,
    pub item_id: Uuid,
    pub related_item_id: Uuid,
    pub relation_type: RelationType,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupItemLink {
    pub id: Uuid,
    pub item_id: Uuid,
    pub url: String,
    pub label: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupItemTrailer {
    pub id: Uuid,
    pub item_id: Uuid,
    pub url: String,
    pub label: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupItemStreamingLink {
    pub id: Uuid,
    pub item_id: Uuid,
    pub platform: StreamingPlatform,
    pub url: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupItemImage {
    pub id: Uuid,
    pub item_id: Uuid,
    pub url: String,
    pub kind: ImageKind,
    pub source: ImageSource,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupItemFile {
    pub id: Uuid,
    pub item_id: Uuid,
    pub path: String,
    pub label: Option<String>,
    pub file_type: FileType,
    pub calibre_book_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupItemGroup {
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupItemEpisode {
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupStaff {
    pub id: Uuid,
    pub external_id: Option<String>,
    pub name: String,
    pub image_url: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupItemStaff {
    pub id: Uuid,
    pub item_id: Uuid,
    pub staff_id: Uuid,
    pub role: String,
    pub character_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupCastMember {
    pub id: Uuid,
    pub external_id: Option<String>,
    pub name: String,
    pub image_url: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupItemCast {
    pub id: Uuid,
    pub item_id: Uuid,
    pub cast_id: Uuid,
    pub character_name: Option<String>,
}

/// テーブルごとのインポート結果件数
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct TableImportCount {
    pub inserted: u32,
    /// 既存レコードとPK/一意制約が衝突しスキップされた件数
    pub skipped: u32,
}

/// POST /backup/import のレスポンスボディ（ApiOkでラップされる）
#[derive(Debug, Clone, Default, Serialize)]
pub struct BackupImportReport {
    pub tables: BTreeMap<String, TableImportCount>,
    pub total_inserted: u32,
    pub total_skipped: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// エンベロープがJSONへラウンドトリップできる
    #[test]
    fn backup_file_round_trips_through_json() {
        let file = BackupFile {
            schema_version: SCHEMA_VERSION,
            exported_at: chrono::NaiveDate::from_ymd_opt(2026, 7, 18)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap(),
            data: BackupData {
                items: vec![BackupItem {
                    id: Uuid::nil(),
                    media_type: MediaType::Anime,
                    title: "テスト".to_string(),
                    original_title: None,
                    description: None,
                    cover_image_url: None,
                    release_date: None,
                    homepage_url: None,
                    status: ItemStatus::NotStarted,
                    consumed_date: None,
                    rating: None,
                    is_favorite: false,
                    source: ItemSource::Manual,
                    external_id: None,
                    details: Some(serde_json::json!({"episodes": 12})),
                    created_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap(),
                    updated_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 1)
                        .unwrap()
                        .and_hms_opt(0, 0, 0)
                        .unwrap(),
                }],
                ..BackupData::default()
            },
        };

        let json = serde_json::to_value(&file).unwrap();
        assert_eq!(json["schema_version"], SCHEMA_VERSION);
        assert_eq!(json["data"]["items"][0]["media_type"], "anime");

        let restored: BackupFile = serde_json::from_value(json).unwrap();
        assert_eq!(restored.data.items.len(), 1);
        assert_eq!(restored.data.items[0].title, "テスト");
    }

    /// テーブル配列が欠落したファイルでも空Vecとしてデシリアライズできる
    #[test]
    fn backup_data_missing_arrays_default_to_empty() {
        let json = serde_json::json!({
            "schema_version": 1,
            "exported_at": "2026-07-18T12:00:00",
            "data": { "items": [] }
        });

        let file: BackupFile = serde_json::from_value(json).unwrap();
        assert!(file.data.tags.is_empty());
        assert!(file.data.item_episodes.is_empty());
    }

    /// 不正なenum値を含むファイルはデシリアライズに失敗する
    #[test]
    fn backup_data_with_invalid_enum_fails_to_deserialize() {
        let json = serde_json::json!({
            "schema_version": 1,
            "exported_at": "2026-07-18T12:00:00",
            "data": {
                "tags": [],
                "item_streaming_links": [{
                    "id": "00000000-0000-0000-0000-000000000000",
                    "item_id": "00000000-0000-0000-0000-000000000000",
                    "platform": "not_a_platform",
                    "url": "https://example.com",
                    "created_at": "2026-01-01T00:00:00"
                }]
            }
        });

        assert!(serde_json::from_value::<BackupFile>(json).is_err());
    }
}
