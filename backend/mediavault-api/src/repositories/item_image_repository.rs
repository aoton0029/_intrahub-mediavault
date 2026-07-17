//! item_images のDB操作
//!
//! item_link_repository.rsと対称な構造。加えて、item作成/インポート時に
//! 外部APIレスポンスから収集した画像URL群を一括登録するinsert_item_images_bulkを持つ。

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::item_image::{ImageKind, ItemImage, NewItemImage};
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
    kind: ImageKind,
) -> Result<ItemImage, ApiError> {
    if !item_exists(pool, item_id).await? {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたアイテムが見つかりません",
        ));
    }

    // 手動追加のためsourceは'manual'固定。既存URLとの衝突時はkindのみ更新する
    sqlx::query_as::<_, ItemImage>(
        "INSERT INTO item_images (item_id, url, kind, source)
         VALUES ($1, $2, $3, 'manual')
         ON CONFLICT (item_id, url) DO UPDATE SET kind = EXCLUDED.kind
         RETURNING id, item_id, url, kind, source, sort_order, created_at",
    )
    .bind(item_id)
    .bind(&url)
    .bind(kind)
    .fetch_one(pool)
    .await
    .map_err(db_error)
}

/// 指定item_idに紐づく画像URLを一覧取得する（`GET /items/:id/images`）
pub async fn list_item_images(pool: &PgPool, item_id: Uuid) -> Result<Vec<ItemImage>, ApiError> {
    sqlx::query_as(
        "SELECT id, item_id, url, kind, source, sort_order, created_at
         FROM item_images
         WHERE item_id = $1
         ORDER BY sort_order, created_at",
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

/// item作成/インポート時、外部APIレスポンスから明示抽出した画像群を同一トランザクション内で
/// 一括登録する。item自体は呼び出し元で既にINSERT済みのためitem存在チェックは行わない。
/// 同一(item_id, url)は`ON CONFLICT DO NOTHING`で無害化する。
/// sort_orderには配列内の並び順（0始まり）を採番する。
pub async fn insert_item_images_bulk(
    tx: &mut Transaction<'_, Postgres>,
    item_id: Uuid,
    images: &[NewItemImage],
) -> Result<(), ApiError> {
    for (index, image) in images.iter().enumerate() {
        sqlx::query(
            "INSERT INTO item_images (item_id, url, kind, source, sort_order)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (item_id, url) DO NOTHING",
        )
        .bind(item_id)
        .bind(&image.url)
        .bind(image.kind)
        .bind(image.source)
        .bind(index as i32)
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
            ImageKind::Other,
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
