//! `ItemSummary` への縮約
//!
//! TASK-0012: ItemSummary 縮約と名前→ID解決サービス

use chrono::Datelike;
use serde::Serialize;
use uuid::Uuid;

use crate::api::models::{Item, ItemStatus, ItemWithRefs, MediaType};

/// 一覧系ツールが返す Item の要約表現。
///
/// 🔵 Intent: 設計決定 D-04（ヒアリング2026-08-07 Q4）より。`description` / `details` 等を
///    含めずレスポンスサイズを抑える。詳細は `get_item_context`（TASK-0014）で取得する。
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ItemSummary {
    pub item_id: Uuid,
    pub title: String,
    pub original_title: Option<String>,
    pub media_type: MediaType,
    pub release_year: Option<i32>,
    pub status: ItemStatus,
    pub rating: Option<f32>,
    pub is_favorite: bool,
    pub tags: Vec<String>,
}

/// 参照用の軽量な名前付きID。タグ・カテゴリ・マイリスト等に共通。
///
/// 🔵 Intent: interfaces.rs `NamedRef`・既存 api の TagRef / CategoryRef パターンより。
///    名前解決サービス（`services::resolve`）の解決結果としても使う。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct NamedRef {
    pub id: Uuid,
    pub name: String,
}

/// `release_date` から年のみ抽出する。
///
/// 🟡 Intent: D-04。フル日付より読みやすい表現として年のみ残す。
fn release_year(release_date: Option<chrono::NaiveDate>) -> Option<i32> {
    release_date.map(|date| date.year())
}

impl From<&ItemWithRefs> for ItemSummary {
    /// 🔵 Intent: `ItemWithRefs.tags` の `name` のみを残し ID を落とす（完了条件）。
    fn from(item: &ItemWithRefs) -> Self {
        ItemSummary {
            item_id: item.id,
            title: item.title.clone(),
            original_title: item.original_title.clone(),
            media_type: item.media_type,
            release_year: release_year(item.release_date),
            status: item.status,
            rating: item.rating,
            is_favorite: item.is_favorite,
            tags: item.tags.iter().map(|tag| tag.name.clone()).collect(),
        }
    }
}

impl From<&Item> for ItemSummary {
    /// 🔵 Intent: `create_item` / `import_external_item` が返す `Item` は tags を
    ///    持たないため、空配列にする（タスクファイル「`ItemWithRefs` を持たない場面」）。
    fn from(item: &Item) -> Self {
        ItemSummary {
            item_id: item.id,
            title: item.title.clone(),
            original_title: item.original_title.clone(),
            media_type: item.media_type,
            release_year: release_year(item.release_date),
            status: item.status,
            rating: item.rating,
            is_favorite: item.is_favorite,
            tags: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models::ApiNamedRef;

    fn sample_item_with_refs() -> ItemWithRefs {
        ItemWithRefs {
            id: Uuid::nil(),
            media_type: MediaType::Anime,
            title: "作品A".to_string(),
            original_title: None,
            release_date: Some(chrono::NaiveDate::from_ymd_opt(2023, 4, 1).unwrap()),
            status: ItemStatus::InProgress,
            rating: Some(8.5),
            is_favorite: true,
            external_id: None,
            tags: vec![ApiNamedRef {
                id: Uuid::nil(),
                name: "お気に入り原作".to_string(),
            }],
            categories: vec![],
        }
    }

    /// テストケース1: ItemWithRefs から ItemSummary への変換
    #[test]
    fn converts_item_with_refs_to_summary() {
        let item = sample_item_with_refs();
        let summary = ItemSummary::from(&item);

        assert_eq!(summary.release_year, Some(2023));
        assert_eq!(summary.tags, vec!["お気に入り原作".to_string()]);
    }

    /// テストケース2: release_date が null
    #[test]
    fn release_year_is_none_when_release_date_is_null() {
        let mut item = sample_item_with_refs();
        item.release_date = None;
        let summary = ItemSummary::from(&item);

        assert_eq!(summary.release_year, None);
    }

    /// テストケース3: description が含まれない（そもそも ItemSummary にフィールドがない）
    #[test]
    fn serialized_json_has_no_description_field() {
        let item = sample_item_with_refs();
        let summary = ItemSummary::from(&item);
        let value = serde_json::to_value(&summary).unwrap();

        assert!(value.get("description").is_none());
        assert!(value.get("details").is_none());
    }

    /// Item（tags なし）から ItemSummary への変換は tags が空配列になる
    #[test]
    fn converts_item_without_refs_with_empty_tags() {
        let item = Item {
            id: Uuid::nil(),
            media_type: MediaType::Manga,
            title: "作品B".to_string(),
            original_title: None,
            release_date: None,
            status: ItemStatus::NotStarted,
            rating: None,
            is_favorite: false,
        };
        let summary = ItemSummary::from(&item);

        assert!(summary.tags.is_empty());
    }
}
