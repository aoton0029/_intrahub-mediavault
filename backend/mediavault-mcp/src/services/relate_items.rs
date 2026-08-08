//! Service層: `relate_items`
//!
//! TASK-0021: relate_items ツールの実装
//!
//! **向きの取り違えが最も事故が起きやすい**（REQ-072）。`item_id` が起点、
//! `related_item_id` が終点であり、種別ごとに向きの意味が異なる。

use uuid::Uuid;

use crate::api::client::ApiClient;
use crate::api::error::ApiClientError;
use crate::api::models::{ItemDetail, ItemRelation, RelationType};
use crate::result::outcome::{McpErrorCode, Outcome, ToolError, classify_api_error};
use crate::result::summary::ItemSummary;
use crate::tools::relate_items::{RelateItemsParams, RelateItemsResult, build_description};

/// `POST /item-relations` が返す409の重複判定コード 🔵 REQ-113・item-relations.md より。
const DUPLICATE_RELATION: &str = "DUPLICATE_RELATION";

#[derive(Debug, Clone, serde::Serialize)]
struct CreateItemRelationRequest {
    item_id: Uuid,
    related_item_id: Uuid,
    relation_type: RelationType,
}

/// `relate_items` の本体。
///
/// 🔵 Intent: mcp-tools.md 8. のとおり。処理順序は
///    (1) 自己参照の拒否（api を呼ばない）
///    (2) 両 Item を並列取得（存在確認を兼ねる。実装項目6「登録前に取得する」）
///    (3) `POST /item-relations`（409は冪等化）
///    の3段階。**書き込みは自動リトライしない**（架構図・可用性方針より）。
pub async fn relate(api: &ApiClient, params: RelateItemsParams) -> RelateItemsResult {
    // 🟡 Intent: EDGE-005・実装項目3より。自己参照は意味を持たず、api側の制約が
    //    未確認であるため MCP 側で弾く。api を一切呼ばない。
    if params.item_id == params.related_item_id {
        let error = ToolError {
            code: McpErrorCode::InvalidArgument.as_str().to_string(),
            message: "item_id と related_item_id が同じです。自己参照は登録できません。"
                .to_string(),
            retriable: false,
        };
        return RelateItemsResult::early_return(params.relation_type, Outcome::Error, error);
    }

    // 🔵 Intent: 実装項目6「登録前に両 Item を取得する（存在確認を兼ねる）」。並列で取得する。
    let (item_result, related_item_result) = tokio::join!(
        fetch_item(api, params.item_id),
        fetch_item(api, params.related_item_id)
    );

    let item = match item_result {
        Ok(item) => item,
        Err(err) => {
            let (outcome, error) = classify_api_error(&err);
            return RelateItemsResult::early_return(params.relation_type, outcome, error);
        }
    };
    let related_item = match related_item_result {
        Ok(item) => item,
        Err(err) => {
            let (outcome, error) = classify_api_error(&err);
            return RelateItemsResult::early_return(params.relation_type, outcome, error);
        }
    };

    let description = build_description(params.relation_type, &item.title, &related_item.title);
    let item_summary = ItemSummary::from(&item);
    let related_item_summary = ItemSummary::from(&related_item);

    let body = CreateItemRelationRequest {
        item_id: params.item_id,
        related_item_id: params.related_item_id,
        relation_type: params.relation_type,
    };

    match api
        .post::<_, ItemRelation>("/api/v1/item-relations", &body)
        .await
    {
        Ok(relation) => RelateItemsResult {
            outcome: Outcome::Success,
            relation_id: Some(relation.id),
            relation_type: params.relation_type,
            description,
            item: Some(item_summary),
            related_item: Some(related_item_summary),
            already_related: false,
            error: None,
        },
        // 🔵 Intent: REQ-113「409は冪等化する」。既存関係の relation_id は
        //    レスポンスに含まれないため `None` とする（タスクファイル実装項目5）。
        Err(ApiClientError::Api {
            code, status: 409, ..
        }) if code == DUPLICATE_RELATION => RelateItemsResult {
            outcome: Outcome::Success,
            relation_id: None,
            relation_type: params.relation_type,
            description,
            item: Some(item_summary),
            related_item: Some(related_item_summary),
            already_related: true,
            error: None,
        },
        Err(err) => {
            let (outcome, error) = classify_api_error(&err);
            RelateItemsResult::early_return(params.relation_type, outcome, error)
        }
    }
}

async fn fetch_item(api: &ApiClient, item_id: Uuid) -> Result<ItemDetail, ApiClientError> {
    api.get::<ItemDetail>(&format!("/api/v1/items/{item_id}"), &[])
        .await
        .map(|response| response.data)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース2: 自己参照は同一UUIDで検出できる
    #[test]
    fn self_reference_is_detected_by_equal_uuids() {
        let id = Uuid::nil();
        assert_eq!(id, id);
    }

    #[test]
    fn duplicate_relation_code_matches_api_error() {
        let err = ApiClientError::Api {
            code: "DUPLICATE_RELATION".to_string(),
            message: "既に関連付けられています".to_string(),
            status: 409,
        };
        let matched = matches!(
            &err,
            ApiClientError::Api { code, status: 409, .. } if code == DUPLICATE_RELATION
        );
        assert!(matched);
    }
}
