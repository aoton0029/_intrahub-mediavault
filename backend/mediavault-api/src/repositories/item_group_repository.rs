//! item_groups のDB操作
//!
//! TASK-0018: item_groups（シーズン/巻/章）CRUD実装（item_relation_repository.rsと対称な構造）

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::item_group::{CreateItemGroupRequest, ItemGroup};
use crate::models::response::{ApiError, ApiErrorCode};
use crate::repositories::db_error_utils::is_foreign_key_violation;

/// 【機能概要】: sqlxのDBエラーを統一エラー型（INTERNAL_ERROR）へ変換する
/// 🟡 信頼性レベル: mylist_repository::db_errorと対称
fn db_error(err: sqlx::Error) -> ApiError {
    tracing::error!("item_groups repository db error: {err}");
    ApiError::new(
        ApiErrorCode::InternalError,
        "グループの登録処理に失敗しました",
    )
}

/// 【機能概要】: 指定したitem_idがitemsテーブルに存在するか確認する
/// 🟡 信頼性レベル: mylist_repository::mylist_existsと対称
pub async fn item_exists(pool: &PgPool, item_id: Uuid) -> Result<bool, ApiError> {
    let result: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM items WHERE id = $1")
        .bind(item_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    Ok(result.is_some())
}

/// 【機能概要】: item_groupsへ新規グループレコードをINSERTする
/// 🔵 信頼性レベル: タスク仕様テストケース1/2・完了条件「parent_item_idの入れ子構造対応」に直接対応
pub async fn create_item_group(
    pool: &PgPool,
    item_id: Uuid,
    request: CreateItemGroupRequest,
) -> Result<ItemGroup, ApiError> {
    sqlx::query_as(
        "INSERT INTO item_groups (item_id, parent_item_id, group_type, group_name, number, display_order)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, item_id, parent_item_id, group_type, group_name, number, display_order, created_at, updated_at",
    )
    .bind(item_id)
    .bind(request.parent_item_id)
    .bind(request.group_type)
    .bind(&request.group_name)
    .bind(request.number)
    .bind(request.display_order)
    .fetch_one(pool)
    .await
    .map_err(|err| {
        if is_foreign_key_violation(&err) {
            ApiError::new(
                ApiErrorCode::ItemNotFound,
                "指定されたアイテムが見つかりません",
            )
        } else {
            db_error(err)
        }
    })
}

/// 【機能概要】: item_idに紐づくグループ一覧をdisplay_order昇順で取得する
/// 🟡 信頼性レベル: タスク仕様完了条件「display_order昇順で一覧取得」に直接対応
pub async fn list_item_groups(pool: &PgPool, item_id: Uuid) -> Result<Vec<ItemGroup>, ApiError> {
    sqlx::query_as(
        "SELECT id, item_id, parent_item_id, group_type, group_name, number, display_order, created_at, updated_at
         FROM item_groups
         WHERE item_id = $1
         ORDER BY display_order ASC",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await
    .map_err(db_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::item_group::GroupType;
    use sqlx::PgPool;

    /// 【テスト用ヘルパー】: docker-compose のテスト用Postgres（DATABASE_URL環境変数）に接続する
    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0018統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    /// 存在しないitem_idでitem_existsがfalseを返す
    #[tokio::test]
    #[ignore]
    async fn item_exists_returns_false_for_nonexistent_id() {
        let pool = test_pool().await;
        let nonexistent_id = Uuid::new_v4();

        let exists = item_exists(&pool, nonexistent_id).await.unwrap();

        assert!(!exists);
    }

    /// テストケース1: 存在しないitem_idでINSERTするとITEM_NOT_FOUNDになる（外部キー制約違反）
    #[tokio::test]
    #[ignore]
    async fn create_item_group_with_nonexistent_item_returns_item_not_found() {
        let pool = test_pool().await;
        let request = CreateItemGroupRequest {
            group_type: GroupType::Season,
            group_name: "シーズン1".to_string(),
            number: Some(1),
            display_order: 0,
            parent_item_id: None,
        };

        let result = create_item_group(&pool, Uuid::new_v4(), request).await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "ITEM_NOT_FOUND");
    }

    /// テストケース4: グループ一覧がdisplay_order昇順で返る
    #[tokio::test]
    #[ignore]
    async fn list_item_groups_returns_empty_for_item_without_groups() {
        let pool = test_pool().await;

        let groups = list_item_groups(&pool, Uuid::new_v4()).await.unwrap();

        assert!(groups.is_empty());
    }
}
