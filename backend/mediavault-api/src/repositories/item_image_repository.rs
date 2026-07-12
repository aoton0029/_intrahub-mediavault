//! item_images のDB操作
//!
//! item_link_repository.rsと対称な構造。加えて、item作成/インポート時に
//! 外部APIレスポンスから収集した画像URL群を一括登録するinsert_item_images_bulkを持つ。

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::item_image::ItemImage;
use crate::models::response::{ApiError, ApiErrorCode};

fn db_error(err: sqlx::Error) -> ApiError {
    tracing::error!("item_images repository db error: {err}");
    ApiError::new(ApiErrorCode::InternalError, "画像の登録処理に失敗しました")
}

async fn item_exists(pool: &PgPool, item_id: Uuid) -> Result<bool, ApiError> {
    let result: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM items WHERE id = $1")
        .bind(item_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    Ok(result.is_some())
}

/// item_idの存在を確認し、item_imagesへ新規画像URLレコードをINSERTする
pub async fn create_item_image(
    pool: &PgPool,
    item_id: Uuid,
    url: String,
) -> Result<ItemImage, ApiError> {
    if !item_exists(pool, item_id).await? {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたアイテムが見つかりません",
        ));
    }

    sqlx::query_as::<_, ItemImage>(
        "INSERT INTO item_images (item_id, url)
         VALUES ($1, $2)
         ON CONFLICT (item_id, url) DO UPDATE SET url = EXCLUDED.url
         RETURNING id, item_id, url, created_at",
    )
    .bind(item_id)
    .bind(&url)
    .fetch_one(pool)
    .await
    .map_err(db_error)
}

/// 指定item_idに紐づく画像URLを一覧取得する（`GET /items/:id/images`）
pub async fn list_item_images(pool: &PgPool, item_id: Uuid) -> Result<Vec<ItemImage>, ApiError> {
    sqlx::query_as(
        "SELECT id, item_id, url, created_at
         FROM item_images
         WHERE item_id = $1
         ORDER BY created_at",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await
    .map_err(db_error)
}

/// item_imagesから指定idのレコードをDELETEする（item_idとの整合性チェック含む）
pub async fn delete_item_image(
    pool: &PgPool,
    item_id: Uuid,
    image_id: Uuid,
) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM item_images WHERE id = $1 AND item_id = $2")
        .bind(image_id)
        .bind(item_id)
        .execute(pool)
        .await
        .map_err(db_error)?;

    Ok(result.rows_affected() > 0)
}

/// item作成/インポート時、外部APIレスポンスから収集した画像URL群を同一トランザクション内で
/// 一括登録する。item自体は呼び出し元で既にINSERT済みのためitem存在チェックは行わない。
/// 同一(item_id, url)は`ON CONFLICT DO NOTHING`で無害化する。
pub async fn insert_item_images_bulk(
    tx: &mut Transaction<'_, Postgres>,
    item_id: Uuid,
    urls: &[String],
) -> Result<(), ApiError> {
    for url in urls {
        sqlx::query(
            "INSERT INTO item_images (item_id, url)
             VALUES ($1, $2)
             ON CONFLICT (item_id, url) DO NOTHING",
        )
        .bind(item_id)
        .bind(url)
        .execute(&mut **tx)
        .await
        .map_err(db_error)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn test_pool() -> PgPool {
        let url =
            std::env::var("DATABASE_URL").expect("統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    #[tokio::test]
    #[ignore]
    async fn create_item_image_with_nonexistent_item_returns_item_not_found() {
        let pool = test_pool().await;

        let result = create_item_image(
            &pool,
            Uuid::new_v4(),
            "https://example.com/image.jpg".to_string(),
        )
        .await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "ITEM_NOT_FOUND");
    }

    #[tokio::test]
    #[ignore]
    async fn delete_item_image_returns_false_for_nonexistent_id() {
        let pool = test_pool().await;

        let deleted = delete_item_image(&pool, Uuid::new_v4(), Uuid::new_v4())
            .await
            .unwrap();

        assert!(!deleted);
    }
}
