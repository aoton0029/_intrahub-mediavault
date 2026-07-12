//! cast / item_cast のDB操作
//!
//! staff_repository.rsと対称な構造（roleを持たない点のみ異なる）。

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::cast::{Cast, CastSearchResult, ItemCast, ItemCastWithName};
use crate::models::response::{ApiError, ApiErrorCode};

fn db_error(err: sqlx::Error) -> ApiError {
    tracing::error!("cast repository db error: {err}");
    ApiError::new(
        ApiErrorCode::InternalError,
        "キャストの登録処理に失敗しました",
    )
}

/// 指定したitem_idがitemsテーブルに存在するか確認する
pub async fn item_exists(pool: &PgPool, item_id: Uuid) -> Result<bool, ApiError> {
    let result: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM items WHERE id = $1")
        .bind(item_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    Ok(result.is_some())
}

/// 指定したcast_idがcast_membersテーブルに存在するか確認する
pub async fn cast_exists(pool: &PgPool, cast_id: Uuid) -> Result<bool, ApiError> {
    let result: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM cast_members WHERE id = $1")
        .bind(cast_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    Ok(result.is_some())
}

/// 氏名部分一致（ILIKE）でcastを検索し、紐付け作品数（linked_item_count）を併記して返す
pub async fn search_cast(
    pool: &PgPool,
    query: &str,
    limit: i64,
) -> Result<Vec<CastSearchResult>, ApiError> {
    sqlx::query_as::<_, CastSearchResult>(
        "SELECT c.id, c.external_id, c.name, c.image_url, c.created_at,
                COUNT(i.id) AS linked_item_count
         FROM cast_members c
         LEFT JOIN item_cast i ON i.cast_id = c.id
         WHERE c.name ILIKE $1
         GROUP BY c.id
         ORDER BY c.name
         LIMIT $2",
    )
    .bind(format!("%{query}%"))
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(db_error)
}

/// cast_membersテーブルへ新規キャストレコードをINSERTする
pub async fn create_cast(
    pool: &PgPool,
    name: String,
    external_id: Option<String>,
    image_url: Option<String>,
) -> Result<Cast, ApiError> {
    sqlx::query_as::<_, Cast>(
        "INSERT INTO cast_members (external_id, name, image_url)
         VALUES ($1, $2, $3)
         RETURNING id, external_id, name, image_url, created_at",
    )
    .bind(&external_id)
    .bind(&name)
    .bind(&image_url)
    .fetch_one(pool)
    .await
    .map_err(db_error)
}

/// external_id一致の既存castを返すか、無ければ新規作成する
pub async fn find_or_create_cast_by_external_id(
    pool: &PgPool,
    external_id: Option<String>,
    name: String,
    image_url: Option<String>,
) -> Result<Cast, ApiError> {
    if let Some(ext_id) = &external_id {
        let existing: Option<Cast> = sqlx::query_as(
            "SELECT id, external_id, name, image_url, created_at FROM cast_members WHERE external_id = $1",
        )
        .bind(ext_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

        if let Some(cast) = existing {
            return Ok(cast);
        }
    }

    create_cast(pool, name, external_id, image_url).await
}

/// item_id/cast_idの存在を確認し、item_castへ新規紐付けレコードをINSERTする
pub async fn link_cast(
    pool: &PgPool,
    item_id: Uuid,
    cast_id: Uuid,
    character_name: Option<String>,
) -> Result<ItemCast, ApiError> {
    if !item_exists(pool, item_id).await? {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたアイテムが見つかりません",
        ));
    }

    if !cast_exists(pool, cast_id).await? {
        return Err(ApiError::new(
            ApiErrorCode::CastNotFound,
            "指定されたキャストが見つかりません",
        ));
    }

    sqlx::query_as::<_, ItemCast>(
        "INSERT INTO item_cast (item_id, cast_id, character_name)
         VALUES ($1, $2, $3)
         RETURNING id, item_id, cast_id, character_name",
    )
    .bind(item_id)
    .bind(cast_id)
    .bind(&character_name)
    .fetch_one(pool)
    .await
    .map_err(db_error)
}

/// 指定item_idに紐づくキャスト紐付けを一覧取得する（`GET /items/:id/cast`）
pub async fn list_item_cast(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<Vec<ItemCastWithName>, ApiError> {
    sqlx::query_as(
        "SELECT item_cast.id, item_cast.item_id, item_cast.cast_id, item_cast.character_name,
                cast_members.name AS cast_name, cast_members.image_url AS cast_image_url
         FROM item_cast
         JOIN cast_members ON cast_members.id = item_cast.cast_id
         WHERE item_cast.item_id = $1",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await
    .map_err(db_error)
}

/// item_castから指定idのレコードをDELETEする（item_idとの整合性チェック含む）
pub async fn unlink_cast(
    pool: &PgPool,
    item_id: Uuid,
    item_cast_id: Uuid,
) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM item_cast WHERE id = $1 AND item_id = $2")
        .bind(item_cast_id)
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
        let url = std::env::var("DATABASE_URL")
            .expect("cast_repository統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn create_cast_with_required_fields_only_returns_cast_with_null_optionals() {
        let pool = test_pool().await;

        let result = create_cast(&pool, "声優A".to_string(), None, None).await;

        let cast = result.expect("create_castは成功するはず");
        assert_eq!(cast.name, "声優A");
        assert!(cast.external_id.is_none());
        assert!(cast.image_url.is_none());
    }

    #[tokio::test]
    #[ignore]
    async fn link_cast_with_character_name_persists_character_name() {
        let pool = test_pool().await;
        let item_id = Uuid::new_v4();
        let cast_id = Uuid::new_v4();

        let result = link_cast(&pool, item_id, cast_id, Some("主人公".to_string())).await;

        let item_cast = result.expect("link_castは成功するはず");
        assert_eq!(item_cast.character_name, Some("主人公".to_string()));
    }

    #[tokio::test]
    #[ignore]
    async fn link_cast_with_nonexistent_cast_id_returns_cast_not_found() {
        let pool = test_pool().await;
        let item_id = Uuid::new_v4();
        let nonexistent_cast_id = Uuid::new_v4();

        let result = link_cast(&pool, item_id, nonexistent_cast_id, None).await;

        let err = result.expect_err("不存在cast_idはエラーになるはず");
        assert_eq!(err.error.code, "CAST_NOT_FOUND");
    }

    #[tokio::test]
    #[ignore]
    async fn deleting_cast_cascades_to_item_cast_records() {
        let pool = test_pool().await;
        let cast = create_cast(&pool, "カスケード確認用".to_string(), None, None)
            .await
            .expect("create_castは成功するはず");
        let item_id = Uuid::new_v4();
        let _item_cast = link_cast(&pool, item_id, cast.id, None)
            .await
            .expect("link_castは成功するはず");

        sqlx::query("DELETE FROM cast_members WHERE id = $1")
            .bind(cast.id)
            .execute(&pool)
            .await
            .expect("cast削除に失敗");

        let remaining: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM item_cast WHERE cast_id = $1")
                .bind(cast.id)
                .fetch_one(&pool)
                .await
                .expect("COUNT取得に失敗");
        assert_eq!(remaining, 0);
    }
}
