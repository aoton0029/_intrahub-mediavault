//! Service層: `organize_item`
//!
//! TASK-0020: organize_item ツールの実装
//!
//! 設計決定 D-03（🔵 ヒアリング2026-08-07 Q2・dataflow.md 機能4）: 「事前取得 + 差分適用」で
//! 冪等性を担保する。`POST .../tags/{tag_id}` 等の 409 を握る方式は、それらのエンドポイントが
//! 409 を返す保証がドキュメント上ないため採らない。
//!
//! **ロールバックしない**（REQ-141）。途中で失敗しても、作成済み・付与済みのものは取り消さない。

use std::collections::HashSet;

use uuid::Uuid;

use crate::api::client::ApiClient;
use crate::api::models::{ItemDetail, Mylist};
use crate::result::operation::{OperationResult, aggregate_outcome};
use crate::result::outcome::classify_api_error;
use crate::services::attach;
use crate::services::resolve::MasterResolver;
use crate::tools::organize_item::{OrganizeItemParams, OrganizeItemResult};

/// `organize_item` の本体。
///
/// 🔵 Intent: mcp-tools.md 7. / dataflow.md 機能4 より。事前取得（Item 詳細・既存マイリスト）
///    自体が失敗したら以降を一切実行せず `error`/`not_found` で終了する。
pub async fn organize(api: &ApiClient, params: OrganizeItemParams) -> OrganizeItemResult {
    let detail = match api
        .get::<ItemDetail>(&format!("/api/v1/items/{}", params.item_id), &[])
        .await
    {
        Ok(response) => response.data,
        Err(err) => {
            let (outcome, error) = classify_api_error(&err);
            return OrganizeItemResult::early_return(params.item_id, outcome, error);
        }
    };

    // 🟡 Intent: 実装項目2「マスタ取得は指定がある種別のみ」と同じ考え方を、
    //    既存マイリスト取得（`GET /items/{id}/mylists`）にも適用し、無駄な呼び出しを避ける。
    let attached_mylists: HashSet<Uuid> = if params.mylists.is_empty() {
        HashSet::new()
    } else {
        match api
            .get::<Vec<Mylist>>(&format!("/api/v1/items/{}/mylists", params.item_id), &[])
            .await
        {
            Ok(response) => response.data.into_iter().map(|mylist| mylist.id).collect(),
            Err(err) => {
                let (outcome, error) = classify_api_error(&err);
                return OrganizeItemResult::early_return(params.item_id, outcome, error);
            }
        }
    };

    let attached_tags: HashSet<Uuid> = detail.tags.iter().map(|tag| tag.id).collect();
    let attached_categories: HashSet<Uuid> = detail
        .categories
        .iter()
        .map(|category| category.id)
        .collect();

    let mut resolver = MasterResolver::new(api);

    // 🟡 Intent: dataflow.md 機能4 ステップ3「並列にすると部分失敗時の『未処理』の意味が
    //    曖昧になる」より、種別内・種別間ともに指定順に逐次実行する。
    let mut tags = Vec::with_capacity(params.tags.len());
    for name in &params.tags {
        tags.push(
            attach::apply_tag(
                api,
                &mut resolver,
                params.item_id,
                name,
                params.create_if_missing,
                &attached_tags,
            )
            .await,
        );
    }

    let mut categories = Vec::with_capacity(params.categories.len());
    for name in &params.categories {
        categories.push(
            attach::apply_category(
                api,
                &mut resolver,
                params.item_id,
                name,
                params.create_if_missing,
                &attached_categories,
            )
            .await,
        );
    }

    let mut mylists = Vec::with_capacity(params.mylists.len());
    for name in &params.mylists {
        mylists.push(
            attach::apply_mylist(
                api,
                &mut resolver,
                params.item_id,
                name,
                params.create_if_missing,
                &attached_mylists,
            )
            .await,
        );
    }

    let mut all_results: Vec<OperationResult> = Vec::new();
    all_results.extend(tags.iter().cloned());
    all_results.extend(categories.iter().cloned());
    all_results.extend(mylists.iter().cloned());

    // 🟡 Intent: 実装項目7の表「指定が空（3種すべて空配列）| success」。
    //    `aggregate_outcome` は空リストを Success として扱うため追加分岐は不要。
    let outcome = aggregate_outcome(&all_results);

    OrganizeItemResult {
        outcome,
        item_id: params.item_id,
        title: Some(detail.title),
        tags,
        categories,
        mylists,
        error: None,
    }
}
