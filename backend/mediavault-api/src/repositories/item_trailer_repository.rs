//! item_trailers のDB操作
//!
//! TASK-0021: item_trailers（トレーラー動画リンク）CRUD実装（item_link_repository.rsと対称な構造）

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::item_trailer::ItemTrailer;
use crate::models::response::{ApiError, ApiErrorCode};

/// 【機能概要】: sqlxのDBエラーを統一エラー型（INTERNAL_ERROR）へ変換する
/// 🟡 信頼性レベル: item_link_repository::db_errorと対称
fn db_error(err: sqlx::Error) -> ApiError {
    tracing::error!("item_trailers repository db error: {err}");
    ApiError::new(
        ApiErrorCode::InternalError,
        "トレーラーの登録処理に失敗しました",
    )
}

/// 【機能概要】: 指定したitem_idがitemsテーブルに存在するか確認する
/// 🔵 信頼性レベル: item_link_repository::item_existsと対称
async fn item_exists(pool: &PgPool, item_id: Uuid) -> Result<bool, ApiError> {
    let result: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM items WHERE id = $1")
        .bind(item_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    Ok(result.is_some())
}

/// 【機能概要】: item_idの存在を確認し、item_trailersへ新規トレーラーレコードをINSERTする
/// 🔵 信頼性レベル: タスク仕様完了条件「該当itemが存在しない場合、ITEM_NOT_FOUND（404）を返す」に直接対応
pub async fn create_item_trailer(
    pool: &PgPool,
    item_id: Uuid,
    url: String,
    label: Option<String>,
) -> Result<ItemTrailer, ApiError> {
    if !item_exists(pool, item_id).await? {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたアイテムが見つかりません",
        ));
    }

    sqlx::query_as::<_, ItemTrailer>(
        "INSERT INTO item_trailers (item_id, url, label)
         VALUES ($1, $2, $3)
         RETURNING id, item_id, url, label, created_at",
    )
    .bind(item_id)
    .bind(&url)
    .bind(&label)
    .fetch_one(pool)
    .await
    .map_err(db_error)
}

/// 【機能概要】: 指定item_idに紐づくトレーラーを一覧取得する（`GET /items/:id/trailers`）
/// 🟡 信頼性レベル: item_group_repository::list_item_groupsと対称のリスト取得パターン
pub async fn list_item_trailers(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<Vec<ItemTrailer>, ApiError> {
    sqlx::query_as(
        "SELECT id, item_id, url, label, created_at
         FROM item_trailers
         WHERE item_id = $1
         ORDER BY created_at",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await
    .map_err(db_error)
}

/// 【機能概要】: item_trailersから指定idのレコードをDELETEする（item_idとの整合性チェック含む）
/// 🟡 信頼性レベル: item_link_repository::delete_item_linkと対称
pub async fn delete_item_trailer(
    pool: &PgPool,
    item_id: Uuid,
    trailer_id: Uuid,
) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM item_trailers WHERE id = $1 AND item_id = $2")
        .bind(trailer_id)
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

    /// 【テスト用ヘルパー】: docker-compose のテスト用Postgres（DATABASE_URL環境変数）に接続する
    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0021統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    /// テストケース: 存在しないitem_idでINSERTするとITEM_NOT_FOUNDになる
    #[tokio::test]
    #[ignore]
    async fn create_item_trailer_with_nonexistent_item_returns_item_not_found() {
        let pool = test_pool().await;

        let result = create_item_trailer(
            &pool,
            Uuid::new_v4(),
            "https://youtube.com/xxx".to_string(),
            None,
        )
        .await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "ITEM_NOT_FOUND");
    }

    /// テストケース: 存在しないtrailer_idでのDELETEはfalseを返す
    #[tokio::test]
    #[ignore]
    async fn delete_item_trailer_returns_false_for_nonexistent_id() {
        let pool = test_pool().await;

        let deleted = delete_item_trailer(&pool, Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap();

        assert!(!deleted);
    }
}
