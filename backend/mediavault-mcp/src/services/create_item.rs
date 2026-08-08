//! Service層: `create_item`
//!
//! TASK-0018: create_item ツールの実装
//!
//! 実行順序（🔵 dataflow.md 機能5・EDGE-004 より）:
//! 1. `POST /api/v1/items` ← 必須。失敗したら即 `error` で終了し `item: null`
//! 2. 以降を順に実行（失敗しても続行）: files → links → tags → mylists
//!
//! **ロールバックしない**（REQ-141）。関連付けが失敗しても作成済み Item を削除しない。

use serde::Serialize;
use url::Url;
use uuid::Uuid;

use crate::api::client::ApiClient;
use crate::api::error::ApiClientError;
use crate::api::models::{Item, ItemFile, ItemLink, MediaType};
use crate::result::operation::{OperationResult, aggregate_outcome};
use crate::result::outcome::{McpErrorCode, Outcome, ToolError, classify_api_error};
use crate::result::summary::ItemSummary;
use crate::services::resolve::{MasterResolver, Resolution};
use crate::tools::create_item::{CreateItemParams, CreateItemResult};

/// `POST /items` のリクエストボディ。
///
/// 🔵 Intent: `items.md` `CreateItemRequest` より。`file_paths`/`urls`/`tags`/`mylists`/
///    `create_if_missing` は Item 作成後の別APIで処理するため含めない。
#[derive(Debug, Clone, Serialize)]
struct CreateItemRequest<'a> {
    media_type: MediaType,
    title: &'a str,
    original_title: Option<&'a str>,
    description: Option<&'a str>,
    release_date: Option<chrono::NaiveDate>,
    homepage_url: Option<&'a str>,
    rating: Option<f32>,
    is_favorite: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
struct CreateItemFileRequest<'a> {
    path: &'a str,
}

/// 🟡 Intent: `item-links.md`「`label` 必須」だが `CreateItemParams.urls` は
///    ラベルを持たない設計（interfaces.rs）。url 文字列自体をラベルとして使う。
#[derive(Debug, Clone, Serialize)]
struct CreateItemLinkRequest<'a> {
    url: &'a str,
    label: &'a str,
}

#[derive(Debug, Clone, Serialize)]
struct AddMylistItemRequest {
    item_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
struct CreateTagRequest<'a> {
    name: &'a str,
}

#[derive(Debug, Clone, Serialize)]
struct CreateMylistRequest<'a> {
    name: &'a str,
}

/// `POST /tags` / `POST /mylists` が返す、名前解決に必要な最小限の形。
#[derive(Debug, Clone, serde::Deserialize)]
struct CreatedNamedEntity {
    id: Uuid,
}

/// `create_item` の本体。
///
/// 🔵 Intent: mcp-tools.md 5. / dataflow.md 機能5 より。Item 作成が失敗したら
///    後続を一切実行せず `error` で終了する。成功していれば、以降どれだけ
///    失敗しても最終的な `outcome` は `partial` 止まりにする（`error` にしない）。
pub async fn create(api: &ApiClient, params: CreateItemParams) -> CreateItemResult {
    let body = CreateItemRequest {
        media_type: params.media_type,
        title: &params.title,
        original_title: params.original_title.as_deref(),
        description: params.description.as_deref(),
        release_date: params.release_date,
        homepage_url: params.homepage_url.as_deref(),
        rating: params.rating,
        is_favorite: params.is_favorite,
    };

    let item = match api.post::<_, Item>("/api/v1/items", &body).await {
        Ok(item) => item,
        Err(err) => {
            let (outcome, error) = classify_api_error(&err);
            return CreateItemResult::creation_failed(outcome, error);
        }
    };
    let item_id = item.id;
    let summary = ItemSummary::from(&item);

    let files = apply_files(api, item_id, &params.file_paths).await;
    let links = apply_links(api, item_id, &params.urls).await;

    let mut resolver = MasterResolver::new(api);
    let tags = apply_tags(
        api,
        &mut resolver,
        item_id,
        &params.tags,
        params.create_if_missing,
    )
    .await;
    let mylists = apply_mylists(
        api,
        &mut resolver,
        item_id,
        &params.mylists,
        params.create_if_missing,
    )
    .await;

    let mut all_results: Vec<OperationResult> = Vec::new();
    all_results.extend(files.iter().cloned());
    all_results.extend(links.iter().cloned());
    all_results.extend(tags.iter().cloned());
    all_results.extend(mylists.iter().cloned());

    // 🔵 Intent: REQ-114・タスクファイル実装項目3「Item 作成に成功していれば
    //    最低でも partial（error にはしない）」。集約結果が Error でも Item は
    //    作成済みなので Partial へ引き下げる。
    let outcome = match aggregate_outcome(&all_results) {
        Outcome::Error => Outcome::Partial,
        other => other,
    };

    CreateItemResult {
        outcome,
        item: Some(summary),
        tags,
        mylists,
        files,
        links,
        error: None,
    }
}

/// URL が `http`/`https` スキームであることを確認する。
///
/// 🔵 Intent: REQ-116（`add_access_link` と同一規約）より。
fn validate_url(raw: &str) -> Result<(), ToolError> {
    match Url::parse(raw) {
        Ok(url) if url.scheme() == "http" || url.scheme() == "https" => Ok(()),
        _ => Err(ToolError {
            code: McpErrorCode::InvalidArgument.as_str().to_string(),
            message: format!("不正なURLです: {raw}"),
            retriable: false,
        }),
    }
}

async fn apply_files(
    api: &ApiClient,
    item_id: Uuid,
    file_paths: &[String],
) -> Vec<OperationResult> {
    let mut results = Vec::with_capacity(file_paths.len());
    for path in file_paths {
        let body = CreateItemFileRequest { path };
        let outcome = api
            .post::<_, ItemFile>(&format!("/api/v1/items/{item_id}/files"), &body)
            .await;
        results.push(match outcome {
            Ok(file) => OperationResult::Applied {
                target_id: file.id,
                target_name: path.clone(),
                created_new: true,
            },
            Err(err) => failed_result(path.clone(), err),
        });
    }
    results
}

async fn apply_links(api: &ApiClient, item_id: Uuid, urls: &[String]) -> Vec<OperationResult> {
    let mut results = Vec::with_capacity(urls.len());
    for raw_url in urls {
        if let Err(error) = validate_url(raw_url) {
            results.push(OperationResult::Failed {
                requested_name: raw_url.clone(),
                error,
            });
            continue;
        }

        let body = CreateItemLinkRequest {
            url: raw_url,
            label: raw_url,
        };
        let outcome = api
            .post::<_, ItemLink>(&format!("/api/v1/items/{item_id}/links"), &body)
            .await;
        results.push(match outcome {
            Ok(link) => OperationResult::Applied {
                target_id: link.id,
                target_name: raw_url.clone(),
                created_new: true,
            },
            Err(err) => failed_result(raw_url.clone(), err),
        });
    }
    results
}

async fn apply_tags(
    api: &ApiClient,
    resolver: &mut MasterResolver<'_>,
    item_id: Uuid,
    names: &[String],
    create_if_missing: bool,
) -> Vec<OperationResult> {
    let mut results = Vec::with_capacity(names.len());
    for name in names {
        let resolution = match resolver.resolve_tag(name).await {
            Ok(resolution) => resolution,
            Err(err) => {
                results.push(failed_result(name.clone(), err));
                continue;
            }
        };

        let (tag_id, created_new) = match resolution {
            Resolution::Found(named_ref) => (named_ref.id, false),
            Resolution::NotFound { available, .. } => {
                if !create_if_missing {
                    results.push(OperationResult::NotResolved {
                        requested_name: name.clone(),
                        available_names: available,
                    });
                    continue;
                }
                match create_tag(api, name).await {
                    Ok(id) => (id, true),
                    Err(err) => {
                        results.push(failed_result(name.clone(), err));
                        continue;
                    }
                }
            }
        };

        let attach = api
            .post_empty_no_content(&format!("/api/v1/items/{item_id}/tags/{tag_id}"))
            .await;
        results.push(match attach {
            Ok(()) => OperationResult::Applied {
                target_id: tag_id,
                target_name: name.clone(),
                created_new,
            },
            Err(err) => failed_result(name.clone(), err),
        });
    }
    results
}

async fn apply_mylists(
    api: &ApiClient,
    resolver: &mut MasterResolver<'_>,
    item_id: Uuid,
    names: &[String],
    create_if_missing: bool,
) -> Vec<OperationResult> {
    let mut results = Vec::with_capacity(names.len());
    for name in names {
        let resolution = match resolver.resolve_mylist(name).await {
            Ok(resolution) => resolution,
            Err(err) => {
                results.push(failed_result(name.clone(), err));
                continue;
            }
        };

        let (mylist_id, created_new) = match resolution {
            Resolution::Found(named_ref) => (named_ref.id, false),
            Resolution::NotFound { available, .. } => {
                if !create_if_missing {
                    results.push(OperationResult::NotResolved {
                        requested_name: name.clone(),
                        available_names: available,
                    });
                    continue;
                }
                match create_mylist(api, name).await {
                    Ok(id) => (id, true),
                    Err(err) => {
                        results.push(failed_result(name.clone(), err));
                        continue;
                    }
                }
            }
        };

        let body = AddMylistItemRequest { item_id };
        let add = api
            .post_no_content(&format!("/api/v1/mylists/{mylist_id}/items"), &body)
            .await;
        results.push(match add {
            Ok(()) => OperationResult::Applied {
                target_id: mylist_id,
                target_name: name.clone(),
                created_new,
            },
            Err(err) => failed_result(name.clone(), err),
        });
    }
    results
}

async fn create_tag(api: &ApiClient, name: &str) -> Result<Uuid, ApiClientError> {
    let body = CreateTagRequest { name };
    api.post::<_, CreatedNamedEntity>("/api/v1/tags", &body)
        .await
        .map(|entity| entity.id)
}

async fn create_mylist(api: &ApiClient, name: &str) -> Result<Uuid, ApiClientError> {
    let body = CreateMylistRequest { name };
    api.post::<_, CreatedNamedEntity>("/api/v1/mylists", &body)
        .await
        .map(|entity| entity.id)
}

/// `ApiClientError` から `OperationResult::Failed` を組み立てる短縮ヘルパ。
fn failed_result(requested_name: String, err: ApiClientError) -> OperationResult {
    let (_, error) = classify_api_error(&err);
    OperationResult::Failed {
        requested_name,
        error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テストケース2: URL の検証（http/https 以外を拒否）
    #[test]
    fn validate_url_rejects_non_http_scheme() {
        assert!(validate_url("not-a-url").is_err());
        assert!(validate_url("ftp://x").is_err());
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://example.com").is_ok());
    }
}
