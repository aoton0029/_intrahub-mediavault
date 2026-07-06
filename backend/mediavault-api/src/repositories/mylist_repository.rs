//! マイリスト・mylist_items のDB操作
//!
//! TASK-0016: マイリストCRUD実装（category_repository.rsと対称な構造）

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::mylist::Mylist;
use crate::models::response::{ApiError, ApiErrorCode};
use crate::repositories::db_error_utils::is_foreign_key_violation;

/// 【機能概要】: sqlxのDBエラーを統一エラー型（INTERNAL_ERROR）へ変換する
/// 🟡 信頼性レベル: category_repository::db_errorと対称
fn db_error(err: sqlx::Error) -> ApiError {
    tracing::error!("mylists repository db error: {err}");
    ApiError::new(
        ApiErrorCode::InternalError,
        "マイリストの登録処理に失敗しました",
    )
}

/// 【機能概要】: 新規マイリストをmylistsテーブルへINSERTする
/// 🔵 信頼性レベル: タスク仕様テストケース1に直接対応
pub async fn create_mylist(pool: &PgPool, name: String) -> Result<Mylist, ApiError> {
    sqlx::query_as("INSERT INTO mylists (name) VALUES ($1) RETURNING id, name, created_at")
        .bind(&name)
        .fetch_one(pool)
        .await
        .map_err(db_error)
}

/// 【機能概要】: 全マイリストを作成日時順に一覧取得する（`GET /mylists`）
/// 🟡 信頼性レベル: category_repository::list_categories_with_countsと対称なリスト取得パターン
pub async fn list_mylists(pool: &PgPool) -> Result<Vec<Mylist>, ApiError> {
    sqlx::query_as("SELECT id, name, created_at FROM mylists ORDER BY created_at")
        .fetch_all(pool)
        .await
        .map_err(db_error)
}

/// 【機能概要】: 指定したitem_idが所属する全マイリストを一覧取得する（`GET /items/:id/mylists`）
/// 🟡 信頼性レベル: mylist_itemsの複合PK(mylist_id, item_id)からのJOINによる逆引き
pub async fn list_mylists_for_item(pool: &PgPool, item_id: Uuid) -> Result<Vec<Mylist>, ApiError> {
    sqlx::query_as(
        "SELECT m.id, m.name, m.created_at FROM mylists m
         JOIN mylist_items mi ON mi.mylist_id = m.id
         WHERE mi.item_id = $1
         ORDER BY m.created_at",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await
    .map_err(db_error)
}

/// 【機能概要】: 指定したmylist_idがmylistsテーブルに存在するか確認する
/// 🟡 信頼性レベル: タスク仕様「事前に存在チェックを行いMYLIST_NOT_FOUNDを返す」に対応
pub async fn mylist_exists(pool: &PgPool, mylist_id: Uuid) -> Result<bool, ApiError> {
    let result: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM mylists WHERE id = $1")
        .bind(mylist_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    Ok(result.is_some())
}

/// 【機能概要】: mylist_itemsへ複合キー（mylist_id, item_id）のレコードをINSERTする
/// 🔵 信頼性レベル: タスク仕様テストケース2に直接対応
pub async fn add_item_to_mylist(
    pool: &PgPool,
    mylist_id: Uuid,
    item_id: Uuid,
) -> Result<(), ApiError> {
    sqlx::query("INSERT INTO mylist_items (mylist_id, item_id) VALUES ($1, $2)")
        .bind(mylist_id)
        .bind(item_id)
        .execute(pool)
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
        })?;

    Ok(())
}

/// 【機能概要】: mylist_itemsから複合キー（mylist_id, item_id）のレコードをDELETEする
/// 🔵 信頼性レベル: タスク仕様テストケース3に直接対応
pub async fn remove_item_from_mylist(
    pool: &PgPool,
    mylist_id: Uuid,
    item_id: Uuid,
) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM mylist_items WHERE mylist_id = $1 AND item_id = $2")
        .bind(mylist_id)
        .bind(item_id)
        .execute(pool)
        .await
        .map_err(db_error)?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use uuid::Uuid;

    /// 【テスト用ヘルパー】: docker-compose のテスト用Postgres（DATABASE_URL環境変数）に接続する
    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0016統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    /// テストケース1: マイリスト作成がDBへINSERTされ、UUID付きのMylistを返す
    #[tokio::test]
    #[ignore]
    async fn create_mylist_inserts_and_returns_mylist() {
        let pool = test_pool().await;

        let mylist = create_mylist(&pool, "今期視聴予定".to_string())
            .await
            .unwrap();

        assert_eq!(mylist.name, "今期視聴予定");
    }

    /// 存在しないmylist_idでmylist_existsがfalseを返す
    #[tokio::test]
    #[ignore]
    async fn mylist_exists_returns_false_for_nonexistent_id() {
        let pool = test_pool().await;
        let nonexistent_id = Uuid::new_v4();

        let exists = mylist_exists(&pool, nonexistent_id).await.unwrap();

        assert!(!exists);
    }

    /// テストケース3: マイリストへのitem追加・削除
    #[tokio::test]
    #[ignore]
    async fn add_and_remove_item_from_mylist() {
        let pool = test_pool().await;
        let mylist_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();

        add_item_to_mylist(&pool, mylist_id, item_id).await.unwrap();
        let removed = remove_item_from_mylist(&pool, mylist_id, item_id)
            .await
            .unwrap();

        assert!(removed);
    }
}
