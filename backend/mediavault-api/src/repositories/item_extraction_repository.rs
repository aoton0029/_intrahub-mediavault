//! `item_file_extractions` の公開API向けDB操作。

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::item_extraction::ItemFileExtraction;
use crate::models::response::{ApiError, ApiErrorCode};
use crate::repositories::db_error_utils::is_unique_violation;

#[derive(Debug)]
pub enum CreateOutcome {
    Created(ItemFileExtraction),
    Existing(ItemFileExtraction),
}

fn db_error(err: sqlx::Error) -> ApiError {
    tracing::error!(error = %err, "item_file_extractions repository db error");
    ApiError::new(ApiErrorCode::InternalError, "抽出処理に失敗しました")
}

fn extraction_not_found() -> ApiError {
    ApiError::new(
        ApiErrorCode::ExtractionNotFound,
        "指定されたファイルの抽出処理が見つかりません",
    )
}

/// 未完了の抽出がなければ作成し、あればその行を返す。
///
/// 先行SELECTは行わず、部分UNIQUE制約へ直接INSERTして競合をDBに直列化させる。
pub async fn create_extraction(
    pool: &PgPool,
    item_file_id: Uuid,
) -> Result<CreateOutcome, ApiError> {
    let inserted = sqlx::query_as::<_, ItemFileExtraction>(
        "INSERT INTO item_file_extractions (item_file_id)
         VALUES ($1)
         RETURNING *",
    )
    .bind(item_file_id)
    .fetch_one(pool)
    .await;

    match inserted {
        Ok(row) => Ok(CreateOutcome::Created(row)),
        Err(err) if is_unique_violation(&err) => {
            let existing = find_active_by_file(pool, item_file_id).await?;
            Ok(CreateOutcome::Existing(existing))
        }
        Err(err) => Err(db_error(err)),
    }
}

/// 同一ファイルの抽出履歴から最新の1件を取得する。
pub async fn find_latest_by_file(
    pool: &PgPool,
    item_file_id: Uuid,
) -> Result<ItemFileExtraction, ApiError> {
    sqlx::query_as::<_, ItemFileExtraction>(
        "SELECT * FROM item_file_extractions
         WHERE item_file_id = $1
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(item_file_id)
    .fetch_optional(pool)
    .await
    .map_err(db_error)?
    .ok_or_else(extraction_not_found)
}

/// 部分UNIQUE制約の対象となる未完了行を取得する。
pub async fn find_active_by_file(
    pool: &PgPool,
    item_file_id: Uuid,
) -> Result<ItemFileExtraction, ApiError> {
    sqlx::query_as::<_, ItemFileExtraction>(
        "SELECT * FROM item_file_extractions
         WHERE item_file_id = $1
           AND state IN ('queued', 'running', 'cancelling')",
    )
    .bind(item_file_id)
    .fetch_optional(pool)
    .await
    .map_err(db_error)?
    .ok_or_else(extraction_not_found)
}

/// activeな抽出へキャンセルを要求する。
///
/// queuedは即時cancelled、runningはcancelling、cancellingはそのままとする。
/// 呼び出し側は同一ファイルの最新行IDを渡す。
pub async fn request_cancel(
    pool: &PgPool,
    extraction_id: Uuid,
    item_file_id: Uuid,
) -> Result<ItemFileExtraction, ApiError> {
    let updated = sqlx::query_as::<_, ItemFileExtraction>(
        "UPDATE item_file_extractions
         SET state = CASE state
                 WHEN 'queued' THEN 'cancelled'::extraction_state
                 WHEN 'running' THEN 'cancelling'::extraction_state
                 ELSE state
             END,
             lease_token = CASE WHEN state = 'queued' THEN NULL ELSE lease_token END,
             lease_expires_at = CASE WHEN state = 'queued' THEN NULL ELSE lease_expires_at END
         WHERE id = $1
           AND item_file_id = $2
           AND state IN ('queued', 'running', 'cancelling')
         RETURNING *",
    )
    .bind(extraction_id)
    .bind(item_file_id)
    .fetch_optional(pool)
    .await
    .map_err(db_error)?;

    if let Some(row) = updated {
        return Ok(row);
    }

    let latest = find_latest_by_file(pool, item_file_id).await?;
    if latest.state.is_terminal() {
        Err(ApiError::new(
            ApiErrorCode::ExtractionAlreadyFinished,
            "抽出処理は既に終了しています",
        ))
    } else {
        // 最新active行と異なるIDが渡された場合。公開APIからは到達しないが、
        // 終端済みと誤って報告せず内部エラーとして扱う。
        Err(ApiError::new(
            ApiErrorCode::InternalError,
            "抽出処理のキャンセルに失敗しました",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::item_extraction::ExtractionState;

    #[test]
    fn db_error_hides_database_details() {
        let error = db_error(sqlx::Error::PoolClosed);

        assert_eq!(error.error.code, "INTERNAL_ERROR");
        assert_eq!(error.error.message, "抽出処理に失敗しました");
        assert!(!error.error.message.contains("PoolClosed"));
    }

    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0003統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    async fn insert_test_file(pool: &PgPool) -> (Uuid, Uuid) {
        let item_id: Uuid = sqlx::query_scalar(
            "INSERT INTO items (media_type, title, status, is_favorite, source)
             VALUES ('novel', 'TASK-0003 repository test', 'not_started', false, 'manual')
             RETURNING id",
        )
        .fetch_one(pool)
        .await
        .expect("テスト用item作成に失敗しました");
        let file_id: Uuid = sqlx::query_scalar(
            "INSERT INTO item_files (item_id, path, file_type)
             VALUES ($1, $2, 'pdf')
             RETURNING id",
        )
        .bind(item_id)
        .bind(format!("/tmp/task-0003-{item_id}.pdf"))
        .fetch_one(pool)
        .await
        .expect("テスト用item_file作成に失敗しました");
        (item_id, file_id)
    }

    async fn delete_test_item(pool: &PgPool, item_id: Uuid) {
        sqlx::query("DELETE FROM items WHERE id = $1")
            .bind(item_id)
            .execute(pool)
            .await
            .expect("テストデータ削除に失敗しました");
    }

    async fn set_state(pool: &PgPool, extraction_id: Uuid, state: ExtractionState) {
        match state {
            ExtractionState::Running | ExtractionState::Cancelling => {
                sqlx::query(
                    "UPDATE item_file_extractions
                     SET state = $1, lease_token = gen_random_uuid(),
                         lease_expires_at = CURRENT_TIMESTAMP + INTERVAL '1 minute'
                     WHERE id = $2",
                )
                .bind(state)
                .bind(extraction_id)
                .execute(pool)
                .await
                .expect("active状態への更新に失敗しました");
            }
            _ => {
                sqlx::query(
                    "UPDATE item_file_extractions
                     SET state = $1, lease_token = NULL, lease_expires_at = NULL
                     WHERE id = $2",
                )
                .bind(state)
                .bind(extraction_id)
                .execute(pool)
                .await
                .expect("終端状態への更新に失敗しました");
            }
        }
    }

    #[tokio::test]
    #[ignore]
    async fn create_extraction_creates_queued_row() {
        let pool = test_pool().await;
        let (item_id, file_id) = insert_test_file(&pool).await;

        let CreateOutcome::Created(row) = create_extraction(&pool, file_id)
            .await
            .expect("新規抽出を作成できるはず")
        else {
            panic!("初回作成がExistingになりました");
        };
        assert_eq!(row.state, ExtractionState::Queued);
        assert_eq!(row.attempts, 0);

        delete_test_item(&pool, item_id).await;
    }

    #[tokio::test]
    #[ignore]
    async fn create_extraction_returns_existing_for_queued_and_running() {
        let pool = test_pool().await;
        let (item_id, file_id) = insert_test_file(&pool).await;
        let CreateOutcome::Created(created) = create_extraction(&pool, file_id).await.unwrap()
        else {
            panic!("初回作成がExistingになりました");
        };

        let CreateOutcome::Existing(queued) = create_extraction(&pool, file_id).await.unwrap()
        else {
            panic!("queuedへの再作成がCreatedになりました");
        };
        assert_eq!(queued.id, created.id);

        set_state(&pool, created.id, ExtractionState::Running).await;
        let CreateOutcome::Existing(running) = create_extraction(&pool, file_id).await.unwrap()
        else {
            panic!("runningへの再作成がCreatedになりました");
        };
        assert_eq!(running.id, created.id);

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM item_file_extractions WHERE item_file_id = $1",
        )
        .bind(file_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 1);
        delete_test_item(&pool, item_id).await;
    }

    #[tokio::test]
    #[ignore]
    async fn terminal_history_allows_new_extraction() {
        let pool = test_pool().await;
        let (item_id, file_id) = insert_test_file(&pool).await;
        let CreateOutcome::Created(first) = create_extraction(&pool, file_id).await.unwrap() else {
            panic!("初回作成がExistingになりました");
        };
        set_state(&pool, first.id, ExtractionState::Succeeded).await;

        let CreateOutcome::Created(second) = create_extraction(&pool, file_id).await.unwrap()
        else {
            panic!("終端状態後の作成がExistingになりました");
        };
        assert_ne!(first.id, second.id);

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM item_file_extractions WHERE item_file_id = $1",
        )
        .bind(file_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 2);
        delete_test_item(&pool, item_id).await;
    }

    #[tokio::test]
    #[ignore]
    async fn concurrent_creation_converges_to_one_active_row() {
        let pool = test_pool().await;
        let (item_id, file_id) = insert_test_file(&pool).await;

        let (left, right) = tokio::join!(
            create_extraction(&pool, file_id),
            create_extraction(&pool, file_id)
        );
        let outcomes = [left.unwrap(), right.unwrap()];
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(outcome, CreateOutcome::Created(_)))
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(outcome, CreateOutcome::Existing(_)))
                .count(),
            1
        );

        let active_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM item_file_extractions
             WHERE item_file_id = $1 AND state IN ('queued', 'running', 'cancelling')",
        )
        .bind(file_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(active_count, 1);
        delete_test_item(&pool, item_id).await;
    }

    #[tokio::test]
    #[ignore]
    async fn find_latest_by_file_returns_newest_history() {
        let pool = test_pool().await;
        let (item_id, file_id) = insert_test_file(&pool).await;
        let CreateOutcome::Created(first) = create_extraction(&pool, file_id).await.unwrap() else {
            panic!("初回作成がExistingになりました");
        };
        set_state(&pool, first.id, ExtractionState::Succeeded).await;
        sqlx::query(
            "UPDATE item_file_extractions
             SET created_at = CURRENT_TIMESTAMP - INTERVAL '1 second'
             WHERE id = $1",
        )
        .bind(first.id)
        .execute(&pool)
        .await
        .unwrap();
        let CreateOutcome::Created(second) = create_extraction(&pool, file_id).await.unwrap()
        else {
            panic!("2件目の作成がExistingになりました");
        };

        let latest = find_latest_by_file(&pool, file_id).await.unwrap();
        assert_eq!(latest.id, second.id);
        assert_eq!(latest.state, ExtractionState::Queued);
        delete_test_item(&pool, item_id).await;
    }

    #[tokio::test]
    #[ignore]
    async fn request_cancel_applies_state_specific_transitions() {
        let pool = test_pool().await;

        for (initial, expected) in [
            (ExtractionState::Queued, ExtractionState::Cancelled),
            (ExtractionState::Running, ExtractionState::Cancelling),
            (ExtractionState::Cancelling, ExtractionState::Cancelling),
        ] {
            let (item_id, file_id) = insert_test_file(&pool).await;
            let CreateOutcome::Created(row) = create_extraction(&pool, file_id).await.unwrap()
            else {
                panic!("初回作成がExistingになりました");
            };
            set_state(&pool, row.id, initial).await;

            let cancelled = request_cancel(&pool, row.id, file_id).await.unwrap();
            assert_eq!(cancelled.state, expected);
            delete_test_item(&pool, item_id).await;
        }

        let (item_id, file_id) = insert_test_file(&pool).await;
        let CreateOutcome::Created(row) = create_extraction(&pool, file_id).await.unwrap() else {
            panic!("初回作成がExistingになりました");
        };
        set_state(&pool, row.id, ExtractionState::Succeeded).await;
        let error = request_cancel(&pool, row.id, file_id).await.unwrap_err();
        assert_eq!(error.error.code, "EXTRACTION_ALREADY_FINISHED");
        delete_test_item(&pool, item_id).await;
    }
}
