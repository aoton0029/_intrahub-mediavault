//! citations のDB操作

use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::models::citation::{Citation, UpdateCitationRequest, has_any_update_field};
use crate::models::response::{ApiError, ApiErrorCode};

const RETURNING_COLUMNS: &str = "id, item_id, quote_text, note, locator_type, page_number, \
    timestamp_seconds, location_number, chapter, created_at, updated_at";

/// sqlxのDBエラーを統一エラー型（INTERNAL_ERROR）へ変換する
fn db_error(err: sqlx::Error) -> ApiError {
    tracing::error!("citations repository db error: {err}");
    ApiError::new(ApiErrorCode::InternalError, "引用の登録処理に失敗しました")
}

/// 指定したitem_idがitemsテーブルに存在するか確認する
async fn item_exists(pool: &PgPool, item_id: Uuid) -> Result<bool, ApiError> {
    let result: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM items WHERE id = $1")
        .bind(item_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    Ok(result.is_some())
}

/// item_idの存在を確認し、citationsへ新規レコードをINSERTする
#[allow(clippy::too_many_arguments)]
pub async fn create_citation(
    pool: &PgPool,
    item_id: Uuid,
    quote_text: String,
    note: Option<String>,
    locator_type: crate::models::citation::LocatorType,
    page_number: Option<i32>,
    timestamp_seconds: Option<i32>,
    location_number: Option<i32>,
    chapter: Option<String>,
) -> Result<Citation, ApiError> {
    if !item_exists(pool, item_id).await? {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたアイテムが見つかりません",
        ));
    }

    let query = format!(
        "INSERT INTO citations (item_id, quote_text, note, locator_type, page_number, \
        timestamp_seconds, location_number, chapter) \
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
        RETURNING {RETURNING_COLUMNS}"
    );

    sqlx::query_as::<_, Citation>(&query)
        .bind(item_id)
        .bind(&quote_text)
        .bind(&note)
        .bind(locator_type)
        .bind(page_number)
        .bind(timestamp_seconds)
        .bind(location_number)
        .bind(&chapter)
        .fetch_one(pool)
        .await
        .map_err(db_error)
}

/// 指定item_idに紐づく引用を作成日時昇順で一覧取得する（`GET /items/:id/citations`）
pub async fn list_citations(pool: &PgPool, item_id: Uuid) -> Result<Vec<Citation>, ApiError> {
    let query =
        format!("SELECT {RETURNING_COLUMNS} FROM citations WHERE item_id = $1 ORDER BY created_at");

    sqlx::query_as(&query)
        .bind(item_id)
        .fetch_all(pool)
        .await
        .map_err(db_error)
}

/// UpdateCitationRequestからSET句を動的に構築する（SET対象が1件も無い場合はNoneを返す）
#[allow(unused_assignments)]
fn build_update_citation_query(
    request: &UpdateCitationRequest,
) -> Option<QueryBuilder<'_, Postgres>> {
    if !has_any_update_field(request) {
        return None;
    }

    let mut builder: QueryBuilder<'_, Postgres> = QueryBuilder::new("UPDATE citations SET ");
    let mut has_condition = false;

    macro_rules! push_set_separator {
        () => {
            if has_condition {
                builder.push(", ");
            } else {
                has_condition = true;
            }
        };
    }

    if let Some(quote_text) = &request.quote_text {
        push_set_separator!();
        builder.push("quote_text = ");
        builder.push_bind(quote_text.clone());
    }
    if let Some(note) = &request.note {
        push_set_separator!();
        builder.push("note = ");
        builder.push_bind(note.clone());
    }
    if let Some(locator_type) = request.locator_type {
        push_set_separator!();
        builder.push("locator_type = ");
        builder.push_bind(locator_type);
    }
    if let Some(page_number) = request.page_number {
        push_set_separator!();
        builder.push("page_number = ");
        builder.push_bind(page_number);
    }
    if let Some(timestamp_seconds) = request.timestamp_seconds {
        push_set_separator!();
        builder.push("timestamp_seconds = ");
        builder.push_bind(timestamp_seconds);
    }
    if let Some(location_number) = request.location_number {
        push_set_separator!();
        builder.push("location_number = ");
        builder.push_bind(location_number);
    }
    if let Some(chapter) = &request.chapter {
        push_set_separator!();
        builder.push("chapter = ");
        builder.push_bind(chapter.clone());
    }

    Some(builder)
}

/// 指定idのcitationを部分更新する（`PATCH /citations/:id`）
pub async fn update_citation(
    pool: &PgPool,
    id: Uuid,
    request: UpdateCitationRequest,
) -> Result<Option<Citation>, ApiError> {
    let Some(mut builder) = build_update_citation_query(&request) else {
        let query = format!("SELECT {RETURNING_COLUMNS} FROM citations WHERE id = $1");
        return sqlx::query_as(&query)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(db_error);
    };

    builder.push(" WHERE id = ");
    builder.push_bind(id);
    builder.push(" RETURNING ");
    builder.push(RETURNING_COLUMNS);

    let citation: Option<Citation> = builder
        .build_query_as()
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    Ok(citation)
}

/// citationsから指定idのレコードをDELETEする
pub async fn delete_citation(pool: &PgPool, id: Uuid) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM citations WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(db_error)?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::citation::LocatorType;
    use sqlx::PgPool;

    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("citations統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    #[tokio::test]
    #[ignore]
    async fn create_citation_with_nonexistent_item_returns_item_not_found() {
        let pool = test_pool().await;

        let result = create_citation(
            &pool,
            Uuid::new_v4(),
            "引用文".to_string(),
            None,
            LocatorType::None,
            None,
            None,
            None,
            None,
        )
        .await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "ITEM_NOT_FOUND");
    }

    #[tokio::test]
    #[ignore]
    async fn delete_citation_returns_false_for_nonexistent_id() {
        let pool = test_pool().await;

        let deleted = delete_citation(&pool, Uuid::new_v4()).await.unwrap();

        assert!(!deleted);
    }

    #[tokio::test]
    #[ignore]
    async fn update_citation_returns_none_for_nonexistent_id() {
        let pool = test_pool().await;

        let request = UpdateCitationRequest {
            quote_text: None,
            note: None,
            locator_type: None,
            page_number: Some(10),
            timestamp_seconds: None,
            location_number: None,
            chapter: None,
        };

        let result = update_citation(&pool, Uuid::new_v4(), request)
            .await
            .unwrap();

        assert!(result.is_none());
    }
}
