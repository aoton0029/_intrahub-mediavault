//! `item_file_texts` のDB操作と主ファイル解決。

use chrono::NaiveDateTime;
use serde_json::Value as JsonValue;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::models::item_extraction::CompleteRequest;
use crate::models::item_file_text::AmbiguousFileCandidate;
use crate::models::response::{ApiError, ApiErrorCode};

/// DB側で切り出した本文チャンクと、そのメタデータ。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ChunkRow {
    pub extraction_version: String,
    pub extracted_at: NaiveDateTime,
    pub boundaries: JsonValue,
    pub extractor: JsonValue,
    pub total_chunks: i64,
    pub chunk_text: String,
}

#[derive(Debug)]
pub enum PrimaryFileResolution {
    Single(Uuid),
    NoFiles,
    NoneExtracted,
    Ambiguous(Vec<AmbiguousFileCandidate>),
}

fn db_error(err: sqlx::Error) -> ApiError {
    tracing::error!(error = %err, "item_file_texts repository db error");
    ApiError::new(
        ApiErrorCode::InternalError,
        "抽出テキストの取得処理に失敗しました",
    )
}

fn serialization_error(err: serde_json::Error) -> ApiError {
    tracing::error!(error = %err, "item_file_texts metadata serialization error");
    ApiError::new(
        ApiErrorCode::InternalError,
        "抽出テキストの保存処理に失敗しました",
    )
}

/// 本文全体をアプリへロードせず、指定範囲だけをDB側で切り出す。
pub async fn fetch_chunk(
    pool: &PgPool,
    chunk_index: i64,
    chunk_size: i64,
    item_file_id: Uuid,
) -> Result<Option<ChunkRow>, ApiError> {
    sqlx::query_as::<_, ChunkRow>(
        "SELECT
             t.extraction_version,
             t.extracted_at,
             t.boundaries,
             t.extractor,
             CEIL(CHAR_LENGTH(t.content)::numeric / $2)::bigint AS total_chunks,
             SUBSTRING(
                 t.content FROM (($1 * $2 + 1)::integer) FOR ($2::integer)
             ) AS chunk_text
         FROM item_file_texts t
         WHERE t.item_file_id = $3",
    )
    .bind(chunk_index)
    .bind(chunk_size)
    .bind(item_file_id)
    .fetch_optional(pool)
    .await
    .map_err(db_error)
}

/// complete処理と同じトランザクション内で抽出結果を保存・置換する。
pub async fn upsert_text(
    connection: &mut PgConnection,
    item_file_id: Uuid,
    request: &CompleteRequest,
) -> Result<(), ApiError> {
    let boundaries = serde_json::to_value(&request.boundaries).map_err(serialization_error)?;
    let extractor = serde_json::to_value(&request.extractor).map_err(serialization_error)?;

    sqlx::query(
        "INSERT INTO item_file_texts
             (item_file_id, content, boundaries, extraction_version, extractor, extracted_at)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (item_file_id) DO UPDATE SET
             content = EXCLUDED.content,
             boundaries = EXCLUDED.boundaries,
             extraction_version = EXCLUDED.extraction_version,
             extractor = EXCLUDED.extractor,
             extracted_at = EXCLUDED.extracted_at",
    )
    .bind(item_file_id)
    .bind(&request.content)
    .bind(boundaries)
    .bind(&request.extraction_version)
    .bind(extractor)
    .bind(request.extracted_at)
    .execute(connection)
    .await
    .map_err(db_error)?;

    Ok(())
}

/// 明示されたファイルに抽出結果が存在するかを確認する。
/// 抽出ジョブの状態は参照しない。
pub async fn text_exists(pool: &PgPool, item_file_id: Uuid) -> Result<bool, ApiError> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(
             SELECT 1 FROM item_file_texts WHERE item_file_id = $1
         )",
    )
    .bind(item_file_id)
    .fetch_one(pool)
    .await
    .map_err(db_error)?;

    Ok(exists)
}

/// itemに紐づく抽出済みファイルを、候補数に応じて解決する。
pub async fn resolve_primary_file(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<PrimaryFileResolution, ApiError> {
    let candidates =
        sqlx::query_as::<_, (Uuid, Option<String>, crate::models::item_file::FileType)>(
            "SELECT f.id, f.label, f.file_type
         FROM item_files f
         INNER JOIN item_file_texts t ON t.item_file_id = f.id
         WHERE f.item_id = $1
         ORDER BY f.created_at, f.id",
        )
        .bind(item_id)
        .fetch_all(pool)
        .await
        .map_err(db_error)?;

    match candidates.as_slice() {
        [(file_id, _, _)] => Ok(PrimaryFileResolution::Single(*file_id)),
        [_, _, ..] => Ok(PrimaryFileResolution::Ambiguous(
            candidates
                .into_iter()
                .map(|(file_id, label, file_type)| AmbiguousFileCandidate {
                    file_id,
                    label,
                    file_type,
                })
                .collect(),
        )),
        [] => {
            let file_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM item_files WHERE item_id = $1")
                    .bind(item_id)
                    .fetch_one(pool)
                    .await
                    .map_err(db_error)?;
            if file_count == 0 {
                Ok(PrimaryFileResolution::NoFiles)
            } else {
                Ok(PrimaryFileResolution::NoneExtracted)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::item_extraction::{ExtractionMethod, ExtractorMetadata};
    use crate::models::item_file::FileType;
    use crate::models::item_file_text::TextBoundary;

    #[test]
    fn db_error_hides_database_details() {
        let error = db_error(sqlx::Error::PoolClosed);

        assert_eq!(error.error.code, "INTERNAL_ERROR");
        assert_eq!(error.error.message, "抽出テキストの取得処理に失敗しました");
        assert!(!error.error.message.contains("PoolClosed"));
    }

    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0004統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    async fn insert_test_item(pool: &PgPool) -> Uuid {
        sqlx::query_scalar(
            "INSERT INTO items (media_type, title, status, is_favorite, source)
             VALUES ('novel', 'TASK-0004 repository test', 'not_started', false, 'manual')
             RETURNING id",
        )
        .fetch_one(pool)
        .await
        .expect("テスト用item作成に失敗しました")
    }

    async fn insert_test_file(
        pool: &PgPool,
        item_id: Uuid,
        label: Option<&str>,
        file_type: FileType,
    ) -> Uuid {
        sqlx::query_scalar(
            "INSERT INTO item_files (item_id, path, label, file_type)
             VALUES ($1, $2, $3, $4)
             RETURNING id",
        )
        .bind(item_id)
        .bind(format!("/tmp/task-0004-{}.dat", Uuid::new_v4()))
        .bind(label)
        .bind(file_type)
        .fetch_one(pool)
        .await
        .expect("テスト用item_file作成に失敗しました")
    }

    async fn delete_test_item(pool: &PgPool, item_id: Uuid) {
        sqlx::query("DELETE FROM items WHERE id = $1")
            .bind(item_id)
            .execute(pool)
            .await
            .expect("テストデータ削除に失敗しました");
    }

    fn complete_request(content: String, version: &str) -> CompleteRequest {
        CompleteRequest {
            lease_token: Uuid::new_v4(),
            content,
            boundaries: vec![TextBoundary {
                start: 0,
                end: 0,
                label: "p.1".to_string(),
            }],
            extraction_version: version.to_string(),
            extracted_at: NaiveDateTime::default(),
            extractor: ExtractorMetadata {
                method: ExtractionMethod::EmbeddedText,
                embedded_text_pages: 1,
                ocr_pages: 0,
                ocr: None,
            },
        }
    }

    async fn save_text(pool: &PgPool, file_id: Uuid, content: String, version: &str) {
        let mut connection = pool.acquire().await.unwrap();
        upsert_text(
            &mut connection,
            file_id,
            &complete_request(content, version),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn fetch_chunk_returns_first_four_thousand_characters() {
        let pool = test_pool().await;
        let item_id = insert_test_item(&pool).await;
        let file_id = insert_test_file(&pool, item_id, None, FileType::Pdf).await;
        let content = "a".repeat(48_000);
        save_text(&pool, file_id, content, "pdf-v1").await;

        let row = fetch_chunk(&pool, 0, 4_000, file_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.total_chunks, 12);
        assert_eq!(row.chunk_text.chars().count(), 4_000);
        assert!(row.chunk_text.chars().all(|character| character == 'a'));
        delete_test_item(&pool, item_id).await;
    }

    #[tokio::test]
    #[ignore]
    async fn fetch_chunk_counts_japanese_characters_not_bytes() {
        let pool = test_pool().await;
        let item_id = insert_test_item(&pool).await;
        let file_id = insert_test_file(&pool, item_id, None, FileType::Pdf).await;
        save_text(&pool, file_id, "日".repeat(8_000), "pdf-v1").await;

        let row = fetch_chunk(&pool, 0, 4_000, file_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.total_chunks, 2);
        assert_eq!(row.chunk_text.chars().count(), 4_000);
        delete_test_item(&pool, item_id).await;
    }

    #[tokio::test]
    #[ignore]
    async fn fetch_chunk_returns_short_final_chunk() {
        let pool = test_pool().await;
        let item_id = insert_test_item(&pool).await;
        let file_id = insert_test_file(&pool, item_id, None, FileType::Pdf).await;
        save_text(&pool, file_id, "x".repeat(4_500), "pdf-v1").await;

        let row = fetch_chunk(&pool, 1, 4_000, file_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.total_chunks, 2);
        assert_eq!(row.chunk_text.chars().count(), 500);
        delete_test_item(&pool, item_id).await;
    }

    #[tokio::test]
    #[ignore]
    async fn fetch_chunk_distinguishes_empty_text_from_missing_text() {
        let pool = test_pool().await;
        let item_id = insert_test_item(&pool).await;
        let file_id = insert_test_file(&pool, item_id, None, FileType::Pdf).await;
        save_text(&pool, file_id, String::new(), "pdf-v1").await;

        let row = fetch_chunk(&pool, 0, 4_000, file_id)
            .await
            .unwrap()
            .expect("空文字でも保存済み行は返るはず");
        assert_eq!(row.total_chunks, 0);
        assert!(row.chunk_text.is_empty());
        assert!(text_exists(&pool, file_id).await.unwrap());
        delete_test_item(&pool, item_id).await;
    }

    #[tokio::test]
    #[ignore]
    async fn upsert_text_replaces_existing_row() {
        let pool = test_pool().await;
        let item_id = insert_test_item(&pool).await;
        let file_id = insert_test_file(&pool, item_id, None, FileType::Pdf).await;
        save_text(&pool, file_id, "old".to_string(), "pdf-v1").await;
        save_text(&pool, file_id, "new-content".to_string(), "pdf-v2").await;

        let row = fetch_chunk(&pool, 0, 4_000, file_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.extraction_version, "pdf-v2");
        assert_eq!(row.chunk_text, "new-content");
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM item_file_texts WHERE item_file_id = $1")
                .bind(file_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 1);
        delete_test_item(&pool, item_id).await;
    }

    #[tokio::test]
    #[ignore]
    async fn resolve_primary_file_handles_all_candidate_counts() {
        let pool = test_pool().await;

        let no_files_item = insert_test_item(&pool).await;
        assert!(matches!(
            resolve_primary_file(&pool, no_files_item).await.unwrap(),
            PrimaryFileResolution::NoFiles
        ));

        let none_extracted_item = insert_test_item(&pool).await;
        insert_test_file(&pool, none_extracted_item, None, FileType::Pdf).await;
        insert_test_file(&pool, none_extracted_item, None, FileType::Image).await;
        assert!(matches!(
            resolve_primary_file(&pool, none_extracted_item)
                .await
                .unwrap(),
            PrimaryFileResolution::NoneExtracted
        ));

        let single_item = insert_test_item(&pool).await;
        let single_file = insert_test_file(&pool, single_item, Some("本文"), FileType::Pdf).await;
        save_text(&pool, single_file, "text".to_string(), "pdf-v1").await;
        assert!(matches!(
            resolve_primary_file(&pool, single_item).await.unwrap(),
            PrimaryFileResolution::Single(id) if id == single_file
        ));

        let ambiguous_item = insert_test_item(&pool).await;
        let first = insert_test_file(&pool, ambiguous_item, Some("PDF"), FileType::Pdf).await;
        let second = insert_test_file(&pool, ambiguous_item, Some("画像"), FileType::Image).await;
        save_text(&pool, first, "pdf".to_string(), "pdf-v1").await;
        save_text(&pool, second, "image".to_string(), "image-v1").await;
        let PrimaryFileResolution::Ambiguous(candidates) =
            resolve_primary_file(&pool, ambiguous_item).await.unwrap()
        else {
            panic!("抽出済み2件はAmbiguousになるはず");
        };
        assert_eq!(candidates.len(), 2);
        assert!(candidates.iter().any(|candidate| {
            candidate.file_id == first
                && candidate.label.as_deref() == Some("PDF")
                && candidate.file_type == FileType::Pdf
        }));
        assert!(candidates.iter().any(|candidate| {
            candidate.file_id == second
                && candidate.label.as_deref() == Some("画像")
                && candidate.file_type == FileType::Image
        }));

        for item_id in [
            no_files_item,
            none_extracted_item,
            single_item,
            ambiguous_item,
        ] {
            delete_test_item(&pool, item_id).await;
        }
    }

    #[tokio::test]
    #[ignore]
    async fn fetch_chunk_handles_maximum_sized_content() {
        let pool = test_pool().await;
        let item_id = insert_test_item(&pool).await;
        let file_id = insert_test_file(&pool, item_id, None, FileType::Pdf).await;
        save_text(&pool, file_id, "z".repeat(5_000_000), "pdf-v1").await;

        let row = fetch_chunk(&pool, 0, 4_000, file_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.chunk_text.chars().count(), 4_000);
        assert_eq!(row.total_chunks, 1_250);
        delete_test_item(&pool, item_id).await;
    }
}
