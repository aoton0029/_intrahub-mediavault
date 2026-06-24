//! item_episodes のDB操作
//!
//! TASK-0019: item_episodes CRUD + EDGE-101検証実装（item_group_repository.rsと対称な構造）

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::item_episode::{CreateItemEpisodeRequest, ItemEpisode};
use crate::models::item_group::GroupType;
use crate::models::response::{ApiError, ApiErrorCode};
use crate::repositories::db_error_utils::{is_foreign_key_violation, is_unique_violation};

/// 【機能概要】: sqlxのDBエラーを統一エラー型（INTERNAL_ERROR）へ変換する
/// 🟡 信頼性レベル: item_group_repository::db_errorと対称
fn db_error(err: sqlx::Error) -> ApiError {
    tracing::error!("item_episodes repository db error: {err}");
    ApiError::new(
        ApiErrorCode::InternalError,
        "話数の登録処理に失敗しました",
    )
}

/// 【機能概要】: 指定したgroup_idのgroup_typeをitem_groupsテーブルから取得する
/// 🔵 信頼性レベル: TASK-0019.md 実装詳細2（group_type=volume検証）に直接対応
pub async fn get_group_type(pool: &PgPool, group_id: Uuid) -> Result<Option<GroupType>, ApiError> {
    let result: Option<(GroupType,)> =
        sqlx::query_as("SELECT group_type FROM item_groups WHERE id = $1")
            .bind(group_id)
            .fetch_optional(pool)
            .await
            .map_err(db_error)?;

    Ok(result.map(|(group_type,)| group_type))
}

/// 【機能概要】: item_episodesへ新規話数レコードをINSERTする
/// 【実装方針】: アプリケーション層でのgroup_type=volume検証は呼び出し元（ハンドラ）の責務とし、
/// 本関数はDBトリガー（trg_check_episode_group_type）のRAISE EXCEPTIONとuq_item_episodes一意制約違反を
/// それぞれINVALID_GROUP_TYPE_FOR_EPISODES（400）・DUPLICATE_EPISODE_NUMBER（409）へマッピングする
/// 🔵 信頼性レベル: TASK-0019.md 実装詳細1/3・テストケース1/2/3に直接対応
pub async fn create_item_episode(
    pool: &PgPool,
    group_id: Uuid,
    request: CreateItemEpisodeRequest,
) -> Result<ItemEpisode, ApiError> {
    sqlx::query_as(
        "INSERT INTO item_episodes (group_id, episode_number, title, original_title, air_date, description)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, group_id, episode_number, title, original_title, air_date, description, created_at, updated_at",
    )
    .bind(group_id)
    .bind(request.episode_number)
    .bind(&request.title)
    .bind(&request.original_title)
    .bind(request.air_date)
    .bind(&request.description)
    .fetch_one(pool)
    .await
    .map_err(|err| {
        if is_unique_violation(&err) {
            ApiError::new(
                ApiErrorCode::DuplicateEpisodeNumber,
                "同一グループ内に同じepisode_numberの話数が既に存在します",
            )
        } else if is_foreign_key_violation(&err) {
            ApiError::new(
                ApiErrorCode::GroupNotFound,
                "指定されたグループが見つかりません",
            )
        } else if is_episode_group_type_trigger_violation(&err) {
            ApiError::new(
                ApiErrorCode::InvalidGroupTypeForEpisodes,
                "volume配下のグループには話数を登録できません",
            )
        } else {
            db_error(err)
        }
    })
}

/// 【機能概要】: DBトリガー`trg_check_episode_group_type`が発生させるRAISE EXCEPTIONかどうかを判定する
/// 【実装方針】: トリガーはSQLSTATEを指定していないためPostgresのデフォルト（P0001）となる。
/// アプリケーション層チェックをすり抜けた競合状態（EDGE-101の二重防御）でのみ到達する経路。
/// 🔵 信頼性レベル: TASK-0019.md 実装詳細3・注意事項「DBトリガーのRAISE EXCEPTIONメッセージからエラーコードへのマッピング」に直接対応
fn is_episode_group_type_trigger_violation(err: &sqlx::Error) -> bool {
    matches!(
        err,
        sqlx::Error::Database(db_err)
            if db_err.code().as_deref() == Some("P0001")
                && db_err.message().contains("item_episodes cannot be added to a volume-type group")
    )
}

/// 【機能概要】: group_idに紐づく話数一覧をepisode_number昇順で取得する
/// 🔵 信頼性レベル: TASK-0019.md 完了条件「episode_number順で一覧取得」・テストケース4に直接対応
pub async fn list_item_episodes(
    pool: &PgPool,
    group_id: Uuid,
) -> Result<Vec<ItemEpisode>, ApiError> {
    sqlx::query_as(
        "SELECT id, group_id, episode_number, title, original_title, air_date, description, created_at, updated_at
         FROM item_episodes
         WHERE group_id = $1
         ORDER BY episode_number ASC",
    )
    .bind(group_id)
    .fetch_all(pool)
    .await
    .map_err(db_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 【テスト用ヘルパー】: docker-compose のテスト用Postgres（DATABASE_URL環境変数）に接続する
    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0019統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    /// 存在しないgroup_idでget_group_typeがNoneを返す
    #[tokio::test]
    #[ignore]
    async fn get_group_type_returns_none_for_nonexistent_group() {
        let pool = test_pool().await;

        let group_type = get_group_type(&pool, Uuid::new_v4()).await.unwrap();

        assert!(group_type.is_none());
    }

    /// テストケース: 存在しないgroup_idでINSERTするとGROUP_NOT_FOUNDになる（外部キー制約違反）
    #[tokio::test]
    #[ignore]
    async fn create_item_episode_with_nonexistent_group_returns_group_not_found() {
        let pool = test_pool().await;
        let request = CreateItemEpisodeRequest {
            episode_number: 1,
            title: Some("第1話".to_string()),
            original_title: None,
            air_date: None,
            description: None,
        };

        let result = create_item_episode(&pool, Uuid::new_v4(), request).await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "GROUP_NOT_FOUND");
    }

    /// テストケース4: 話数が存在しないグループで一覧が空配列を返す
    #[tokio::test]
    #[ignore]
    async fn list_item_episodes_returns_empty_for_group_without_episodes() {
        let pool = test_pool().await;

        let episodes = list_item_episodes(&pool, Uuid::new_v4()).await.unwrap();

        assert!(episodes.is_empty());
    }

    /// is_episode_group_type_trigger_violationが非Databaseエラーでfalseを返す
    #[test]
    fn is_episode_group_type_trigger_violation_returns_false_for_non_database_error() {
        let err = sqlx::Error::PoolClosed;
        assert!(!is_episode_group_type_trigger_violation(&err));
    }
}
