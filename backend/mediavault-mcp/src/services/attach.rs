//! Service層: タグ・カテゴリ・マイリストの付与を共通化するヘルパ
//!
//! TASK-0020: organize_item と TASK-0018 create_item の双方が使う共通ロジック
//! （タスクファイル「注意事項」: 付与ロジックが重複するため共通の Service へ切り出す）。

use std::collections::HashSet;

use serde::Serialize;
use uuid::Uuid;

use crate::api::client::ApiClient;
use crate::api::error::ApiClientError;
use crate::api::models::{CategoryWithCount, TagWithCount};
use crate::result::operation::OperationResult;
use crate::result::outcome::classify_api_error;
use crate::services::resolve::{MasterResolver, Resolution};

/// `POST /tags` が返す 409 の重複コード 🔵 tags.md より。
const DUPLICATE_TAG_NAME: &str = "DUPLICATE_TAG_NAME";
/// `POST /categories` が返す 409 の重複コード 🔵 categories.md より。
const DUPLICATE_CATEGORY_NAME: &str = "DUPLICATE_CATEGORY_NAME";

#[derive(Debug, Clone, Serialize)]
struct CreateNamedRequest<'a> {
    name: &'a str,
}

#[derive(Debug, Clone, Serialize)]
struct AddMylistItemRequest {
    item_id: Uuid,
}

/// `POST /tags` / `POST /categories` / `POST /mylists` が返す、名前解決に必要な最小限の形。
#[derive(Debug, Clone, serde::Deserialize)]
struct CreatedNamedEntity {
    id: Uuid,
}

/// タグを名前で付与する。
///
/// 🔵 Intent: タスクファイル「実装項目3」の4分岐（既に付与済み/マスタに存在/
///    マスタに不在+作成しない/マスタに不在+作成する）をここに集約する。
///    `already_attached` が空集合なら「常に未付与」として扱われる
///    （`create_item` はItem新規作成直後のため、この呼び出し方をする）。
pub async fn apply_tag(
    api: &ApiClient,
    resolver: &mut MasterResolver<'_>,
    item_id: Uuid,
    name: &str,
    create_if_missing: bool,
    already_attached: &HashSet<Uuid>,
) -> OperationResult {
    let resolution = match resolver.resolve_tag(name).await {
        Ok(resolution) => resolution,
        Err(err) => return failed_result(name, err),
    };

    let (tag_id, created_new) = match resolution {
        Resolution::Found(named_ref) => (named_ref.id, false),
        Resolution::NotFound { available, .. } => {
            if !create_if_missing {
                return OperationResult::NotResolved {
                    requested_name: name.to_string(),
                    available_names: available,
                };
            }
            match create_tag(api, name).await {
                Ok(id) => (id, true),
                Err(err) => return failed_result(name, err),
            }
        }
    };

    if already_attached.contains(&tag_id) {
        return OperationResult::AlreadyApplied {
            target_id: tag_id,
            target_name: name.to_string(),
        };
    }

    let attach = api
        .post_empty_no_content(&format!("/api/v1/items/{item_id}/tags/{tag_id}"))
        .await;
    match attach {
        Ok(()) => OperationResult::Applied {
            target_id: tag_id,
            target_name: name.to_string(),
            created_new,
        },
        Err(err) => failed_result(name, err),
    }
}

/// カテゴリを名前で付与する。タグと同形。
pub async fn apply_category(
    api: &ApiClient,
    resolver: &mut MasterResolver<'_>,
    item_id: Uuid,
    name: &str,
    create_if_missing: bool,
    already_attached: &HashSet<Uuid>,
) -> OperationResult {
    let resolution = match resolver.resolve_category(name).await {
        Ok(resolution) => resolution,
        Err(err) => return failed_result(name, err),
    };

    let (category_id, created_new) = match resolution {
        Resolution::Found(named_ref) => (named_ref.id, false),
        Resolution::NotFound { available, .. } => {
            if !create_if_missing {
                return OperationResult::NotResolved {
                    requested_name: name.to_string(),
                    available_names: available,
                };
            }
            match create_category(api, name).await {
                Ok(id) => (id, true),
                Err(err) => return failed_result(name, err),
            }
        }
    };

    if already_attached.contains(&category_id) {
        return OperationResult::AlreadyApplied {
            target_id: category_id,
            target_name: name.to_string(),
        };
    }

    let attach = api
        .post_empty_no_content(&format!("/api/v1/items/{item_id}/categories/{category_id}"))
        .await;
    match attach {
        Ok(()) => OperationResult::Applied {
            target_id: category_id,
            target_name: name.to_string(),
            created_new,
        },
        Err(err) => failed_result(name, err),
    }
}

/// マイリストを名前で付与する。
///
/// 🔵 Intent: mylists.md より、付与エンドポイントの向きだけ他2種と逆
///    （`POST /mylists/{id}/items`、body に `item_id`）。
pub async fn apply_mylist(
    api: &ApiClient,
    resolver: &mut MasterResolver<'_>,
    item_id: Uuid,
    name: &str,
    create_if_missing: bool,
    already_attached: &HashSet<Uuid>,
) -> OperationResult {
    let resolution = match resolver.resolve_mylist(name).await {
        Ok(resolution) => resolution,
        Err(err) => return failed_result(name, err),
    };

    let (mylist_id, created_new) = match resolution {
        Resolution::Found(named_ref) => (named_ref.id, false),
        Resolution::NotFound { available, .. } => {
            if !create_if_missing {
                return OperationResult::NotResolved {
                    requested_name: name.to_string(),
                    available_names: available,
                };
            }
            match create_mylist(api, name).await {
                Ok(id) => (id, true),
                Err(err) => return failed_result(name, err),
            }
        }
    };

    if already_attached.contains(&mylist_id) {
        return OperationResult::AlreadyApplied {
            target_id: mylist_id,
            target_name: name.to_string(),
        };
    }

    let body = AddMylistItemRequest { item_id };
    let add = api
        .post_no_content(&format!("/api/v1/mylists/{mylist_id}/items"), &body)
        .await;
    match add {
        Ok(()) => OperationResult::Applied {
            target_id: mylist_id,
            target_name: name.to_string(),
            created_new,
        },
        Err(err) => failed_result(name, err),
    }
}

async fn create_tag(api: &ApiClient, name: &str) -> Result<Uuid, ApiClientError> {
    let body = CreateNamedRequest { name };
    match api
        .post::<_, CreatedNamedEntity>("/api/v1/tags", &body)
        .await
    {
        Ok(entity) => Ok(entity.id),
        // 🟡 Intent: タスクファイル「実装項目5」。別経路で同時に作られた場合、
        //    マスタを再取得して ID を得てから付与へ進む。エラーにしない。
        Err(ApiClientError::Api {
            code, status: 409, ..
        }) if code == DUPLICATE_TAG_NAME => find_tag_id_by_name(api, name).await,
        Err(err) => Err(err),
    }
}

async fn create_category(api: &ApiClient, name: &str) -> Result<Uuid, ApiClientError> {
    let body = CreateNamedRequest { name };
    match api
        .post::<_, CreatedNamedEntity>("/api/v1/categories", &body)
        .await
    {
        Ok(entity) => Ok(entity.id),
        Err(ApiClientError::Api {
            code, status: 409, ..
        }) if code == DUPLICATE_CATEGORY_NAME => find_category_id_by_name(api, name).await,
        Err(err) => Err(err),
    }
}

async fn create_mylist(api: &ApiClient, name: &str) -> Result<Uuid, ApiClientError> {
    let body = CreateNamedRequest { name };
    api.post::<_, CreatedNamedEntity>("/api/v1/mylists", &body)
        .await
        .map(|entity| entity.id)
}

async fn find_tag_id_by_name(api: &ApiClient, name: &str) -> Result<Uuid, ApiClientError> {
    let response = api.get::<Vec<TagWithCount>>("/api/v1/tags", &[]).await?;
    response
        .data
        .into_iter()
        .find(|tag| tag.name == name)
        .map(|tag| tag.id)
        .ok_or_else(|| ApiClientError::Api {
            code: DUPLICATE_TAG_NAME.to_string(),
            message: "作成競合後にタグが見つかりませんでした".to_string(),
            status: 409,
        })
}

async fn find_category_id_by_name(api: &ApiClient, name: &str) -> Result<Uuid, ApiClientError> {
    let response = api
        .get::<Vec<CategoryWithCount>>("/api/v1/categories", &[])
        .await?;
    response
        .data
        .into_iter()
        .find(|category| category.name == name)
        .map(|category| category.id)
        .ok_or_else(|| ApiClientError::Api {
            code: DUPLICATE_CATEGORY_NAME.to_string(),
            message: "作成競合後にカテゴリが見つかりませんでした".to_string(),
            status: 409,
        })
}

/// `ApiClientError` から `OperationResult::Failed` を組み立てる短縮ヘルパ。
fn failed_result(requested_name: &str, err: ApiClientError) -> OperationResult {
    let (_, error) = classify_api_error(&err);
    OperationResult::Failed {
        requested_name: requested_name.to_string(),
        error,
    }
}
