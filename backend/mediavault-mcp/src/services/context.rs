//! Service層: `get_item_context`
//!
//! TASK-0014: get_item_context ツールの実装

use uuid::Uuid;

use crate::api::client::ApiClient;
use crate::api::error::ApiClientError;
use crate::api::models::{
    ItemCast, ItemDetail, ItemFile, ItemGroup, ItemLink, ItemRelation, ItemStaff, ItemTrailer,
    Mylist,
};
use crate::result::Section;
use crate::result::outcome::{Outcome, classify_api_error};
use crate::result::summary::NamedRef;
use crate::tools::get_item_context::{
    CreditView, FileView, GroupView, ItemContextResult, ItemDetailView, LinkView,
    RelationDirection, RelationView, StreamingLinkView,
};

/// `get_item_context` の本体。
///
/// 🔵 Intent: 設計決定 D-05・タスクファイル実装項目2/3 より。`GET /items/{id}` を先に
///    取得し、成功した場合のみ残り8本を `futures::join!` で並列取得する。
///    `try_join!` は使わない（1つの失敗が他のセクションを握りつぶすため REQ-021 を壊す）。
pub async fn get_context(api: &ApiClient, item_id: Uuid) -> ItemContextResult {
    let item_path = format!("/api/v1/items/{item_id}");
    let item_detail = match api.get::<ItemDetail>(&item_path, &[]).await {
        Ok(response) => response.data,
        Err(err) => {
            let (outcome, error) = classify_api_error(&err);
            return ItemContextResult::early_return(outcome, Some(error));
        }
    };

    let relations_path = format!("{item_path}/relations");
    let mylists_path = format!("{item_path}/mylists");
    let groups_path = format!("{item_path}/groups");
    let cast_path = format!("{item_path}/cast");
    let staff_path = format!("{item_path}/staff");
    let files_path = format!("{item_path}/files");
    let links_path = format!("{item_path}/links");
    let trailers_path = format!("{item_path}/trailers");

    let (relations, mylists, groups, cast, staff, files, links, trailers) = futures::join!(
        api.get::<Vec<ItemRelation>>(&relations_path, &[]),
        api.get::<Vec<Mylist>>(&mylists_path, &[]),
        api.get::<Vec<ItemGroup>>(&groups_path, &[]),
        api.get::<Vec<ItemCast>>(&cast_path, &[]),
        api.get::<Vec<ItemStaff>>(&staff_path, &[]),
        api.get::<Vec<ItemFile>>(&files_path, &[]),
        api.get::<Vec<ItemLink>>(&links_path, &[]),
        api.get::<Vec<ItemTrailer>>(&trailers_path, &[]),
    );

    let relations_section = Section::from_result(unwrap_data(relations).map(|items| {
        items
            .into_iter()
            .map(|relation| relation_to_view(relation, item_id))
            .collect()
    }));
    let mylists_section = Section::from_result(unwrap_data(mylists).map(|items| {
        items
            .into_iter()
            .map(|mylist| NamedRef {
                id: mylist.id,
                name: mylist.name,
            })
            .collect()
    }));
    let groups_section = Section::from_result(unwrap_data(groups).map(|items| {
        items
            .into_iter()
            .map(|group| GroupView {
                group_id: group.id,
                name: group.group_name,
                group_type: group.group_type,
                number: group.number,
            })
            .collect()
    }));
    let cast_section = Section::from_result(unwrap_data(cast).map(|items| {
        items
            .into_iter()
            .map(|item| CreditView {
                person_id: item.cast_id,
                role: item.character_name,
            })
            .collect()
    }));
    let staff_section = Section::from_result(unwrap_data(staff).map(|items| {
        items
            .into_iter()
            .map(|item| CreditView {
                person_id: item.staff_id,
                role: Some(item.role),
            })
            .collect()
    }));
    let files_section = Section::from_result(unwrap_data(files).map(|items| {
        items
            .into_iter()
            .map(|file| FileView {
                file_id: file.id,
                path: file.path,
                file_kind: file.file_type,
            })
            .collect()
    }));
    let links_section = Section::from_result(unwrap_data(links).map(|items| {
        items
            .into_iter()
            .map(|link| LinkView {
                link_id: link.id,
                url: link.url,
                label: Some(link.label),
            })
            .collect()
    }));
    let trailers_section = Section::from_result(unwrap_data(trailers).map(|items| {
        items
            .into_iter()
            .map(|trailer| LinkView {
                link_id: trailer.id,
                url: trailer.url,
                label: trailer.label,
            })
            .collect()
    }));

    let has_failure = is_failed(&relations_section)
        || is_failed(&mylists_section)
        || is_failed(&groups_section)
        || is_failed(&cast_section)
        || is_failed(&staff_section)
        || is_failed(&files_section)
        || is_failed(&links_section)
        || is_failed(&trailers_section);
    let outcome = determine_outcome(has_failure);

    ItemContextResult {
        outcome,
        item: Some(to_item_detail_view(item_detail)),
        relations: relations_section,
        mylists: mylists_section,
        groups: groups_section,
        cast: cast_section,
        staff: staff_section,
        files: files_section,
        links: links_section,
        trailers: trailers_section,
        error: None,
    }
}

/// `ApiResponse<Vec<T>>` から `pagination` を捨てて `Vec<T>` のみ取り出す。
fn unwrap_data<T>(
    result: Result<crate::api::client::ApiResponse<Vec<T>>, ApiClientError>,
) -> Result<Vec<T>, ApiClientError> {
    result.map(|response| response.data)
}

fn is_failed<T>(section: &Section<T>) -> bool {
    matches!(section, Section::Failed { .. })
}

/// 8本すべて成功なら `Success`、1件以上 `Failed` なら `Partial`。
///
/// 🔵 Intent: タスクファイル実装項目5の表より。
fn determine_outcome(has_failure: bool) -> Outcome {
    if has_failure {
        Outcome::Partial
    } else {
        Outcome::Success
    }
}

/// `GET /items/{id}/relations` の各行に、対象 Item から見た関係の向きを付ける。
///
/// 🔵 Intent: REQ-072・タスクファイル実装項目6より。対象 Item が `item_id` 側なら
///    `outgoing`、`related_item_id` 側なら `incoming`。
fn relation_to_view(relation: ItemRelation, item_id: Uuid) -> RelationView {
    let (direction, related_item_id) = if relation.item_id == item_id {
        (RelationDirection::Outgoing, relation.related_item_id)
    } else {
        (RelationDirection::Incoming, relation.item_id)
    };
    RelationView {
        relation_id: relation.id,
        relation_type: relation.relation_type,
        direction,
        related_item_id,
    }
}

/// `ItemDetail`（api レスポンス）を `ItemDetailView`（ツール結果）へ変換する。
///
/// 🔵 Intent: `detail` は加工せずそのまま渡す（タスクファイル「detail の扱い」より）。
fn to_item_detail_view(detail: ItemDetail) -> ItemDetailView {
    ItemDetailView {
        item_id: detail.id,
        title: detail.title,
        original_title: detail.original_title,
        media_type: detail.media_type,
        description: detail.description,
        release_date: detail.release_date,
        homepage_url: detail.homepage_url,
        status: detail.status,
        consumed_date: detail.consumed_date,
        rating: detail.rating,
        is_favorite: detail.is_favorite,
        external_id: detail.external_id,
        tags: detail
            .tags
            .into_iter()
            .map(|tag| NamedRef {
                id: tag.id,
                name: tag.name,
            })
            .collect(),
        categories: detail
            .categories
            .into_iter()
            .map(|category| NamedRef {
                id: category.id,
                name: category.name,
            })
            .collect(),
        streaming_links: detail
            .streaming_links
            .into_iter()
            .map(|link| StreamingLinkView {
                link_id: link.id,
                platform: link.platform,
                url: link.url,
            })
            .collect(),
        detail: detail.detail,
        created_at: detail.created_at,
        updated_at: detail.updated_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models::RelationType;
    use crate::result::outcome::ToolError;

    /// テストケース1: 空配列が Empty になる（`Section::from_result` の再確認）
    #[test]
    fn empty_vec_becomes_empty_section() {
        let result: Result<Vec<i32>, ApiClientError> = Ok(vec![]);
        let section = Section::from_result(result);
        assert!(matches!(section, Section::Empty));
    }

    /// テストケース2: 関係の向き判定（対象 Item が item_id 側 = outgoing）
    #[test]
    fn relation_is_outgoing_when_item_is_origin() {
        let item_id = Uuid::new_v4();
        let related_id = Uuid::new_v4();
        let relation = ItemRelation {
            id: Uuid::new_v4(),
            item_id,
            related_item_id: related_id,
            relation_type: RelationType::Sequel,
        };

        let view = relation_to_view(relation, item_id);

        assert_eq!(view.direction, RelationDirection::Outgoing);
        assert_eq!(view.related_item_id, related_id);
    }

    /// テストケース2: 関係の向き判定（対象 Item が related_item_id 側 = incoming）
    #[test]
    fn relation_is_incoming_when_item_is_target() {
        let item_id = Uuid::new_v4();
        let origin_id = Uuid::new_v4();
        let relation = ItemRelation {
            id: Uuid::new_v4(),
            item_id: origin_id,
            related_item_id: item_id,
            relation_type: RelationType::Adaptation,
        };

        let view = relation_to_view(relation, item_id);

        assert_eq!(view.direction, RelationDirection::Incoming);
        assert_eq!(view.related_item_id, origin_id);
    }

    fn loaded(count: usize) -> Section<i32> {
        Section::Loaded {
            items: vec![0; count],
        }
    }

    fn failed() -> Section<i32> {
        Section::Failed {
            error: ToolError {
                code: "MCP_API_UNREACHABLE".to_string(),
                message: "timeout".to_string(),
                retriable: true,
            },
        }
    }

    /// テストケース3: outcome の決定ロジック（全成功）
    #[test]
    fn outcome_is_success_when_all_sections_succeed() {
        let sections = [loaded(1), Section::Empty, loaded(0)];
        let has_failure = sections.iter().any(is_failed);
        assert_eq!(determine_outcome(has_failure), Outcome::Success);
    }

    /// テストケース3: outcome の決定ロジック（1件失敗）
    #[test]
    fn outcome_is_partial_when_one_section_fails() {
        let sections = [loaded(1), failed(), Section::Empty];
        let has_failure = sections.iter().any(is_failed);
        assert_eq!(determine_outcome(has_failure), Outcome::Partial);
    }

    /// テストケース3: outcome の決定ロジック（全失敗）
    #[test]
    fn outcome_is_partial_when_all_sections_fail() {
        let sections = [failed(), failed(), failed()];
        let has_failure = sections.iter().any(is_failed);
        assert_eq!(determine_outcome(has_failure), Outcome::Partial);
    }
}
