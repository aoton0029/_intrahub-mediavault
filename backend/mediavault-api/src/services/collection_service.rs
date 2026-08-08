//! コレクション統計集計ロジック
//!
//! TASK-0004: GET /api/v1/collection/overview の新設。
//! `GET /items/counts-by-media-type` との重複を避けるため、既存の
//! `item_repository::count_items_by_media_type` をそのまま再利用する。

use std::collections::HashMap;

use sqlx::PgPool;

use crate::models::collection::{CollectionOverview, CountEntry, RecentItemsOrder};
use crate::models::item::{ItemStatus, ItemWithRefs, MediaType};
use crate::models::response::ApiError;
use crate::repositories::item_repository;

/// メディア種別の固定順（`MediaTypeCounts`のフィールド順に揃える）
const MEDIA_TYPE_ORDER: [MediaType; 8] = [
    MediaType::Anime,
    MediaType::Movie,
    MediaType::Drama,
    MediaType::Manga,
    MediaType::Novel,
    MediaType::Game,
    MediaType::AcademicBook,
    MediaType::Paper,
];

/// status の固定順
const STATUS_ORDER: [ItemStatus; 3] = [
    ItemStatus::NotStarted,
    ItemStatus::InProgress,
    ItemStatus::Completed,
];

fn media_type_key(media_type: MediaType) -> &'static str {
    match media_type {
        MediaType::Anime => "anime",
        MediaType::Movie => "movie",
        MediaType::Drama => "drama",
        MediaType::Manga => "manga",
        MediaType::Novel => "novel",
        MediaType::Game => "game",
        MediaType::AcademicBook => "academic_book",
        MediaType::Paper => "paper",
    }
}

fn status_key(status: ItemStatus) -> &'static str {
    match status {
        ItemStatus::NotStarted => "not_started",
        ItemStatus::InProgress => "in_progress",
        ItemStatus::Completed => "completed",
    }
}

/// 【機能概要】: `GET /collection/overview`が返す集計結果を組み立てる
/// 【実装方針】: 件数系（総件数/お気に入り/media_type別/status別）と、最近追加・更新された
/// item一覧を並行して取得し、`CollectionOverview`へ集約する。件数0のmedia_type/statusも
/// `by_media_type`/`by_status`に0件エントリとして含める（完了条件「空コレクションは0・空配列」）
pub async fn get_collection_overview(
    pool: &PgPool,
    recent_limit: u32,
) -> Result<CollectionOverview, ApiError> {
    let (totals, media_type_counts, status_counts, recently_added, recently_updated) = tokio::try_join!(
        item_repository::count_collection_totals(pool),
        item_repository::count_items_by_media_type(pool),
        item_repository::count_items_by_status(pool),
        item_repository::list_recent_items(pool, recent_limit, RecentItemsOrder::CreatedAt),
        item_repository::list_recent_items(pool, recent_limit, RecentItemsOrder::UpdatedAt),
    )?;
    let (total_items, favorite_count) = totals;

    let by_media_type = MEDIA_TYPE_ORDER
        .into_iter()
        .map(|media_type| {
            let count = match media_type {
                MediaType::Anime => media_type_counts.anime,
                MediaType::Movie => media_type_counts.movie,
                MediaType::Drama => media_type_counts.drama,
                MediaType::Manga => media_type_counts.manga,
                MediaType::Novel => media_type_counts.novel,
                MediaType::Game => media_type_counts.game,
                MediaType::AcademicBook => media_type_counts.academic_book,
                MediaType::Paper => media_type_counts.paper,
            };
            CountEntry {
                key: media_type_key(media_type).to_string(),
                count,
            }
        })
        .collect();

    let by_status = STATUS_ORDER
        .into_iter()
        .map(|status| {
            let count = status_counts
                .iter()
                .find(|(row_status, _)| *row_status == status)
                .map(|(_, count)| *count)
                .unwrap_or(0);
            CountEntry {
                key: status_key(status).to_string(),
                count,
            }
        })
        .collect();

    // 【tags/categories付与】: GET /items一覧と同様、N+1回避のためitem_id単位でバッチ取得する
    let item_ids: Vec<uuid::Uuid> = recently_added
        .iter()
        .chain(recently_updated.iter())
        .map(|item| item.id)
        .collect();
    let mut tags_by_item = item_repository::get_items_tags_batch(pool, &item_ids).await?;
    let mut categories_by_item =
        item_repository::get_items_categories_batch(pool, &item_ids).await?;

    let with_refs =
        |items: Vec<crate::models::item::Item>,
         tags_by_item: &mut HashMap<uuid::Uuid, Vec<crate::models::item::TagRef>>,
         categories_by_item: &mut HashMap<uuid::Uuid, Vec<crate::models::item::CategoryRef>>|
         -> Vec<ItemWithRefs> {
            items
                .into_iter()
                .map(|item| {
                    let tags = tags_by_item.get(&item.id).cloned().unwrap_or_default();
                    let categories = categories_by_item
                        .get(&item.id)
                        .cloned()
                        .unwrap_or_default();
                    ItemWithRefs {
                        item,
                        tags,
                        categories,
                    }
                })
                .collect()
        };

    let recently_added = with_refs(recently_added, &mut tags_by_item, &mut categories_by_item);
    let recently_updated = with_refs(recently_updated, &mut tags_by_item, &mut categories_by_item);

    Ok(CollectionOverview {
        total_items,
        favorite_count,
        by_media_type,
        by_status,
        recently_added,
        recently_updated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// media_type/statusの固定順キーが完了条件のキー名と一致することを確認する
    #[test]
    fn media_type_and_status_keys_match_wire_format() {
        assert_eq!(media_type_key(MediaType::Anime), "anime");
        assert_eq!(media_type_key(MediaType::AcademicBook), "academic_book");
        assert_eq!(status_key(ItemStatus::NotStarted), "not_started");
        assert_eq!(status_key(ItemStatus::InProgress), "in_progress");
        assert_eq!(status_key(ItemStatus::Completed), "completed");
    }
}
