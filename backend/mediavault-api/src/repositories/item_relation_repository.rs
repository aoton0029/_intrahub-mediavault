//! item_relations のDB操作
//!
//! TASK-0017: item_relations（関連付け・DLC）CRUD実装（mylist_repository.rsと対称な構造）

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::item_relation::{ItemRelation, RelationType};
use crate::models::response::{ApiError, ApiErrorCode};
use crate::repositories::db_error_utils::{is_foreign_key_violation, is_unique_violation};

/// 【機能概要】: sqlxのDBエラーを統一エラー型（INTERNAL_ERROR）へ変換する
/// 🟡 信頼性レベル: mylist_repository::db_errorと対称
fn db_error(err: sqlx::Error) -> ApiError {
    tracing::error!("item_relations repository db error: {err}");
    ApiError::new(
        ApiErrorCode::InternalError,
        "関連付けの登録処理に失敗しました",
    )
}

/// 【機能概要】: item_relationsへ新規関連付けレコードをINSERTする
/// 【実装方針】: 一意制約違反（uq_item_relations）はDUPLICATE_RELATION、
/// 外部キー制約違反（存在しないitem_id/related_item_id）はITEM_NOT_FOUNDへマッピングする
/// 🔵 信頼性レベル: タスク仕様テストケース1・完了条件「同一組み合わせの一意制約違反対応」に直接対応
pub async fn create_item_relation(
    pool: &PgPool,
    item_id: Uuid,
    related_item_id: Uuid,
    relation_type: RelationType,
) -> Result<ItemRelation, ApiError> {
    sqlx::query_as(
        "INSERT INTO item_relations (item_id, related_item_id, relation_type)
         VALUES ($1, $2, $3)
         RETURNING id, item_id, related_item_id, relation_type, created_at",
    )
    .bind(item_id)
    .bind(related_item_id)
    .bind(relation_type)
    .fetch_one(pool)
    .await
    .map_err(|err| {
        if is_unique_violation(&err) {
            ApiError::new(
                ApiErrorCode::DuplicateRelation,
                "指定された関連付けは既に登録されています",
            )
        } else if is_foreign_key_violation(&err) {
            ApiError::new(
                ApiErrorCode::ItemNotFound,
                "指定されたアイテムが見つかりません",
            )
        } else {
            db_error(err)
        }
    })
}

/// 【機能概要】: item_relationsから指定IDのレコードをDELETEする
/// 🔵 信頼性レベル: タスク仕様テストケース4に直接対応
pub async fn delete_item_relation(pool: &PgPool, id: Uuid) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM item_relations WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(db_error)?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    /// 【テスト用ヘルパー】: docker-compose のテスト用Postgres（DATABASE_URL環境変数）に接続する
    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0017統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    /// テストケース1: 存在しないitem_idでINSERTするとITEM_NOT_FOUNDになる（外部キー制約違反）
    #[tokio::test]
    #[ignore]
    async fn create_item_relation_with_nonexistent_item_returns_item_not_found() {
        let pool = test_pool().await;

        let result = create_item_relation(
            &pool,
            Uuid::new_v4(),
            Uuid::new_v4(),
            RelationType::Dlc,
        )
        .await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "ITEM_NOT_FOUND");
    }

    /// テストケース4: 存在しないidでのDELETEはfalseを返す
    #[tokio::test]
    #[ignore]
    async fn delete_item_relation_returns_false_for_nonexistent_id() {
        let pool = test_pool().await;

        let deleted = delete_item_relation(&pool, Uuid::new_v4()).await.unwrap();

        assert!(!deleted);
    }
}
