//! collection ハンドラ
//!
//! TASK-0004: GET /api/v1/collection/overview の新設

use axum::extract::{Query, State};

use crate::AppState;
use crate::models::collection::{
    CollectionOverview, CollectionOverviewQuery, validate_recent_limit,
};
use crate::models::response::{ApiError, ApiOk};
use crate::services::collection_service;

/// 【機能概要】: `GET /collection/overview` ハンドラ。コレクション全体の統計
/// （media_type別/status別件数、お気に入り件数、総件数、最近追加・更新されたItem）を
/// 1回のリクエストで返す
/// 【実装方針】: `recent_limit`を検証後、`collection_service::get_collection_overview`へ委譲する。
/// 集計ロジック自体はrepository/serviceに切り出し、ハンドラは薄く保つ
/// 🔵 信頼性レベル: TASK-0004完了条件・prep.md PREP-04に直接対応
pub async fn get_collection_overview_handler(
    State(state): State<AppState>,
    Query(query): Query<CollectionOverviewQuery>,
) -> Result<ApiOk<CollectionOverview>, ApiError> {
    let recent_limit = validate_recent_limit(query.recent_limit)?;
    let overview = collection_service::get_collection_overview(&state.db, recent_limit).await?;
    Ok(ApiOk::new(overview))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppState;
    use axum::http::StatusCode;
    use uuid::Uuid;

    async fn test_app_state() -> AppState {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0004統合テストにはDATABASE_URL環境変数が必要です");
        let pool = sqlx::PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました");
        AppState {
            db: pool,
            internal_api_key: "test-key".to_string(),
        }
    }

    async fn insert_test_item(
        state: &AppState,
        media_type: &str,
        status: &str,
        is_favorite: bool,
    ) -> Uuid {
        sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO items (media_type, title, status, is_favorite, source, external_id) \
            VALUES ($1, 'collection-overview-test', $2, $3, 'manual', NULL) RETURNING id",
        )
        .bind(media_type)
        .bind(status)
        .bind(is_favorite)
        .fetch_one(&state.db)
        .await
        .unwrap()
    }

    /// テストケース6: recent_limit=0 は400 VALIDATION_ERRORを返す（実DB必要）
    /// 🔵 信頼性レベル: TASK-0004完了条件・単体テスト要件テストケース6より
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn get_collection_overview_handler_returns_400_for_recent_limit_zero() {
        let state = test_app_state().await;
        let query = CollectionOverviewQuery {
            recent_limit: Some(0),
        };

        let result = get_collection_overview_handler(State(state), Query(query)).await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    /// テストケース6: recent_limit=51 は400 VALIDATION_ERRORを返す（実DB必要）
    /// 🔵 信頼性レベル: TASK-0004完了条件・単体テスト要件テストケース6より
    #[tokio::test]
    #[ignore]
    async fn get_collection_overview_handler_returns_400_for_recent_limit_over_max() {
        let state = test_app_state().await;
        let query = CollectionOverviewQuery {
            recent_limit: Some(51),
        };

        let result = get_collection_overview_handler(State(state), Query(query)).await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
    }

    /// テストケース1・2: 新規投入したitem分だけtotal_items/favorite_count/by_media_type/by_status
    /// が増分し、recently_added/recently_updatedにも反映されることを確認する（実DB必要）
    /// 【テスト内容】: anime(favorite,completed)を2件、manga(not_favorite,not_started)を1件投入し、
    /// 投入前後の差分を検証する（他テストと共有DBのため絶対値ではなく差分で検証）
    /// 🔵 信頼性レベル: TASK-0004完了条件・単体テスト要件テストケース1/2より
    #[tokio::test]
    #[ignore]
    async fn get_collection_overview_handler_reflects_inserted_items() {
        let state = test_app_state().await;

        let before = get_collection_overview_handler(
            State(state.clone()),
            Query(CollectionOverviewQuery { recent_limit: None }),
        )
        .await
        .unwrap()
        .data;

        insert_test_item(&state, "anime", "completed", true).await;
        insert_test_item(&state, "anime", "completed", true).await;
        insert_test_item(&state, "manga", "not_started", false).await;

        let after = get_collection_overview_handler(
            State(state.clone()),
            Query(CollectionOverviewQuery { recent_limit: None }),
        )
        .await
        .unwrap()
        .data;

        assert_eq!(after.total_items, before.total_items + 3);
        assert_eq!(after.favorite_count, before.favorite_count + 2);

        let anime_before = before
            .by_media_type
            .iter()
            .find(|entry| entry.key == "anime")
            .unwrap()
            .count;
        let anime_after = after
            .by_media_type
            .iter()
            .find(|entry| entry.key == "anime")
            .unwrap()
            .count;
        assert_eq!(anime_after, anime_before + 2);

        let completed_before = before
            .by_status
            .iter()
            .find(|entry| entry.key == "completed")
            .unwrap()
            .count;
        let completed_after = after
            .by_status
            .iter()
            .find(|entry| entry.key == "completed")
            .unwrap()
            .count;
        assert_eq!(completed_after, completed_before + 2);
    }

    /// テストケース5: recent_limit未指定時は既定10件が返る（実DB必要）
    /// 🔵 信頼性レベル: TASK-0004完了条件「既定10」より
    #[tokio::test]
    #[ignore]
    async fn get_collection_overview_handler_returns_up_to_10_recent_items_by_default() {
        let state = test_app_state().await;
        for _ in 0..12 {
            insert_test_item(&state, "manga", "not_started", false).await;
        }

        let overview = get_collection_overview_handler(
            State(state),
            Query(CollectionOverviewQuery { recent_limit: None }),
        )
        .await
        .unwrap()
        .data;

        assert_eq!(overview.recently_added.len(), 10);
        assert_eq!(overview.recently_updated.len(), 10);
    }

    /// テストケース7: 空コレクションでも200・0件/空配列で返る（実DB・空DB前提のため参考実装。
    /// 通常は他テストと共有DBのため0件保証はできず、非負であることのみ確認する）
    /// 🟡 信頼性レベル: 完了条件・境界ケースとしての妥当な推測
    #[tokio::test]
    #[ignore]
    async fn get_collection_overview_handler_returns_200_without_error_when_no_matching_rows() {
        let state = test_app_state().await;

        let result = get_collection_overview_handler(
            State(state),
            Query(CollectionOverviewQuery {
                recent_limit: Some(5),
            }),
        )
        .await;

        assert!(result.is_ok());
        let overview = result.unwrap().data;
        assert_eq!(overview.by_media_type.len(), 8);
        assert_eq!(overview.by_status.len(), 3);
    }
}
