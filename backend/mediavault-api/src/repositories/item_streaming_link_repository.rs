//! item_streaming_links のDB操作
//!
//! item_link_repository.rsと対称な構造。UNIQUE(item_id, platform)違反時は
//! DuplicateStreamingLink（409）へ変換する。

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::item_streaming_link::{ItemStreamingLink, StreamingPlatform};
use crate::models::response::{ApiError, ApiErrorCode};
use crate::repositories::db_error_utils::is_unique_violation;

fn db_error(err: sqlx::Error) -> ApiError {
    tracing::error!("item_streaming_links repository db error: {err}");
    ApiError::new(
        ApiErrorCode::InternalError,
        "配信URLの登録処理に失敗しました",
    )
}

async fn item_exists(pool: &PgPool, item_id: Uuid) -> Result<bool, ApiError> {
    let result: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM items WHERE id = $1")
        .bind(item_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    Ok(result.is_some())
}

/// item_idの存在を確認し、item_streaming_linksへ新規レコードをINSERTする。
/// 同一(item_id, platform)が既に存在する場合はDuplicateStreamingLink（409）を返す。
pub async fn create_item_streaming_link(
    pool: &PgPool,
    item_id: Uuid,
    platform: StreamingPlatform,
    url: String,
) -> Result<ItemStreamingLink, ApiError> {
    if !item_exists(pool, item_id).await? {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたアイテムが見つかりません",
        ));
    }

    sqlx::query_as::<_, ItemStreamingLink>(
        "INSERT INTO item_streaming_links (item_id, platform, url)
         VALUES ($1, $2, $3)
         RETURNING id, item_id, platform, url, created_at",
    )
    .bind(item_id)
    .bind(platform)
    .bind(&url)
    .fetch_one(pool)
    .await
    .map_err(|err| {
        if is_unique_violation(&err) {
            ApiError::new(
                ApiErrorCode::DuplicateStreamingLink,
                "このプラットフォームの配信URLは既に登録されています",
            )
        } else {
            db_error(err)
        }
    })
}

/// 指定item_idに紐づく配信URLを一覧取得する（`GET /items/:id/streaming-links`）
pub async fn list_item_streaming_links(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<Vec<ItemStreamingLink>, ApiError> {
    sqlx::query_as(
        "SELECT id, item_id, platform, url, created_at
         FROM item_streaming_links
         WHERE item_id = $1
         ORDER BY created_at",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await
    .map_err(db_error)
}

/// item_streaming_linksから指定idのレコードをDELETEする（item_idとの整合性チェック含む）
pub async fn delete_item_streaming_link(
    pool: &PgPool,
    item_id: Uuid,
    link_id: Uuid,
) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM item_streaming_links WHERE id = $1 AND item_id = $2")
        .bind(link_id)
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

    async fn test_pool() -> PgPool {
        let url =
            std::env::var("DATABASE_URL").expect("統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    #[tokio::test]
    #[ignore]
    async fn create_item_streaming_link_with_nonexistent_item_returns_item_not_found() {
        let pool = test_pool().await;

        let result = create_item_streaming_link(
            &pool,
            Uuid::new_v4(),
            StreamingPlatform::Netflix,
            "https://www.netflix.com/title/12345".to_string(),
        )
        .await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "ITEM_NOT_FOUND");
    }

    #[tokio::test]
    #[ignore]
    async fn delete_item_streaming_link_returns_false_for_nonexistent_id() {
        let pool = test_pool().await;

        let deleted = delete_item_streaming_link(&pool, Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap();

        assert!(!deleted);
    }
}
