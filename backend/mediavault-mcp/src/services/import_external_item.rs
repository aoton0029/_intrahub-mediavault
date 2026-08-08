//! Service層: `import_external_item`
//!
//! TASK-0017: import_external_item ツールの実装
//!
//! **最初の書き込み系ツール**。以降の書き込み系ツールが踏襲するパターン
//! （annotation・冪等性・結果の構造）をここで確立する。

use serde::Serialize;

use crate::api::client::ApiClient;
use crate::api::error::ApiClientError;
use crate::api::models::{Item, ItemWithRefs, MediaType};
use crate::result::outcome::{Outcome, classify_api_error};
use crate::result::summary::ItemSummary;
use crate::tools::import_external_item::{ImportExternalItemParams, ImportExternalItemResult};

/// api の409冪等化での重複判定コード 🔵 REQ-112
const ITEM_ALREADY_IMPORTED: &str = "ITEM_ALREADY_IMPORTED";

/// 既存Item探索の上限ページ数（1ページ = [SCAN_PAGE_LIMIT] 件、計100件まで）。
///
/// 🟡 Intent: タスクファイル実装項目3「推奨: 走査するページ数に上限（例: 5ページ=100件）を
///    設ける」より。api は409レスポンスに既存Itemの ID を含まないため MCP 側で探す必要がある。
const SCAN_MAX_PAGES: u32 = 5;
const SCAN_PAGE_LIMIT: u32 = 20;

/// `POST /items/import` のリクエストボディ。
///
/// 🔵 Intent: `items.md` `ImportByIdRequest` より。
#[derive(Debug, Clone, Serialize)]
struct ImportByIdRequest<'a> {
    media_type: MediaType,
    external_id: &'a str,
    provider: Option<&'a str>,
}

/// `import_external_item` の本体。
///
/// 🔵 Intent: mcp-tools.md 4. のとおり、api が対応プロバイダから詳細を再取得し
///    `CreateItemRequest` を構築して登録する。MCP は詳細情報を組み立てない。
///    **書き込みは自動リトライしない**（architecture.md 可用性より）。
pub async fn import(api: &ApiClient, params: ImportExternalItemParams) -> ImportExternalItemResult {
    let body = ImportByIdRequest {
        media_type: params.media_type,
        external_id: &params.external_id,
        provider: params.provider.as_deref(),
    };

    match api.post::<_, Item>("/api/v1/items/import", &body).await {
        Ok(item) => ImportExternalItemResult {
            outcome: Outcome::Success,
            item: Some(ItemSummary::from(&item)),
            already_existed: false,
            error: None,
        },
        Err(ApiClientError::Api {
            code, status: 409, ..
        }) if code == ITEM_ALREADY_IMPORTED => {
            // 🔵 Intent: REQ-112「重複を作らず既存Itemを返す」。エラーにはしない。
            let existing = find_existing_item(api, params.media_type, &params.external_id).await;
            ImportExternalItemResult {
                outcome: Outcome::Success,
                item: existing.map(|item| ItemSummary::from(&item)),
                already_existed: true,
                error: None,
            }
        }
        Err(err) => {
            let (outcome, error) = classify_api_error(&err);
            ImportExternalItemResult::early_return(outcome, Some(error))
        }
    }
}

/// `GET /items?media_type=X` を走査して `external_id` が一致する既存Itemを探す。
///
/// 🟡 Intent: タスクファイル「案A」。ページ数上限に達しても見つからなければ `None` を返し、
///    呼び出し元は `item: null` + `already_existed: true` として扱う（エラーにしない）。
///    途中で api 呼び出しが失敗した場合も同様に `None` として扱う
///    （「重複がないこと自体は成功」なので登録失敗として扱わない）。
async fn find_existing_item(
    api: &ApiClient,
    media_type: MediaType,
    external_id: &str,
) -> Option<ItemWithRefs> {
    let mut after: Option<(chrono::NaiveDateTime, uuid::Uuid)> = None;

    for _ in 0..SCAN_MAX_PAGES {
        let mut query = vec![
            ("media_type", as_query_string(media_type)),
            ("limit", SCAN_PAGE_LIMIT.to_string()),
        ];
        if let Some((created_at, id)) = &after {
            query.push((
                "after_created_at",
                created_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
            ));
            query.push(("after_id", id.to_string()));
        }

        let response = api.get::<Vec<ItemWithRefs>>("/api/v1/items", &query).await;
        let response = match response {
            Ok(response) => response,
            Err(_) => return None,
        };

        if let Some(found) = response
            .data
            .into_iter()
            .find(|item| item.external_id.as_deref() == Some(external_id))
        {
            return Some(found);
        }

        match response.pagination {
            Some(pagination) if pagination.has_more => {
                match (pagination.next_after_created_at, pagination.next_after_id) {
                    (Some(created_at), Some(id)) => after = Some((created_at, id)),
                    _ => return None,
                }
            }
            _ => return None,
        }
    }

    None
}

/// `#[serde(rename_all = "snake_case")]` な enum をクエリ文字列へ変換する。
fn as_query_string<T: serde::Serialize>(value: T) -> String {
    match serde_json::to_value(value).expect("enum は常にシリアライズ可能") {
        serde_json::Value::String(s) => s,
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース2: 409 → 冪等化コードの判定
    #[test]
    fn already_imported_code_matches_api_error() {
        let err = ApiClientError::Api {
            code: "ITEM_ALREADY_IMPORTED".to_string(),
            message: "既にインポート済みです".to_string(),
            status: 409,
        };
        let matched = matches!(
            &err,
            ApiClientError::Api { code, status: 409, .. } if code == ITEM_ALREADY_IMPORTED
        );
        assert!(matched);
    }

    #[test]
    fn as_query_string_converts_snake_case_enum() {
        assert_eq!(as_query_string(MediaType::Anime), "anime");
    }
}
