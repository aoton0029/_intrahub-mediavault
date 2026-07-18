//! バックアップのエクスポート/インポートロジック
//!
//! export_backup: 単一トランザクション内の全テーブルSELECTで一貫スナップショットを取り、
//! バージョン付きエンベロープ（BackupFile）を組み立てる。
//! import_backup: schema_versionを検証し、単一トランザクション内でFK順にINSERTする。
//! 既存レコードとのPK/一意制約衝突は行単位でスキップ（ON CONFLICT DO NOTHING）し、
//! FK違反・トリガー違反等の構造的な不整合は全体をロールバックして400を返す。

use sqlx::PgPool;

use crate::models::backup::{
    BackupData, BackupFile, BackupImportReport, SCHEMA_VERSION, TableImportCount,
};
use crate::models::response::{ApiError, ApiErrorCode};
use crate::repositories::backup_repository as repo;
use crate::repositories::db_error_utils::is_foreign_key_violation;

/// エクスポート/インポート共通のDB内部エラー変換（詳細はログのみに出す）
fn internal_db_error(err: sqlx::Error) -> ApiError {
    tracing::error!("backup db error: {err}");
    ApiError::new(
        ApiErrorCode::InternalError,
        "バックアップ処理に失敗しました",
    )
}

/// インポート中のINSERTエラーを分類する。
/// FK違反（親レコードがDBにもバックアップにも存在しない）と、
/// トリガー・CHECK制約違反（P0001/23514）は不正なバックアップファイルとして400、
/// それ以外は500へマップする
fn map_import_db_error(table: &'static str, err: sqlx::Error) -> ApiError {
    if is_foreign_key_violation(&err) {
        return ApiError::new(
            ApiErrorCode::ValidationError,
            format!("バックアップファイルが不正です（{table}: 参照先レコードが存在しません）"),
        );
    }
    if let sqlx::Error::Database(db_err) = &err {
        // P0001: RAISE EXCEPTION（volumeグループへのepisode等）、23514: CHECK制約違反
        if matches!(db_err.code().as_deref(), Some("P0001") | Some("23514")) {
            return ApiError::new(
                ApiErrorCode::ValidationError,
                format!("バックアップファイルが不正です（{table}: 制約違反）"),
            );
        }
    }
    internal_db_error(err)
}

/// 全対象テーブルをSELECTしてバックアップエンベロープを構築する
pub async fn export_backup(pool: &PgPool) -> Result<BackupFile, ApiError> {
    let mut tx = pool.begin().await.map_err(internal_db_error)?;
    let conn = tx.as_mut();

    let data = BackupData {
        items: repo::fetch_all_items(conn)
            .await
            .map_err(internal_db_error)?,
        tags: repo::fetch_all_tags(conn)
            .await
            .map_err(internal_db_error)?,
        item_tags: repo::fetch_all_item_tags(conn)
            .await
            .map_err(internal_db_error)?,
        categories: repo::fetch_all_categories(conn)
            .await
            .map_err(internal_db_error)?,
        item_categories: repo::fetch_all_item_categories(conn)
            .await
            .map_err(internal_db_error)?,
        mylists: repo::fetch_all_mylists(conn)
            .await
            .map_err(internal_db_error)?,
        mylist_items: repo::fetch_all_mylist_items(conn)
            .await
            .map_err(internal_db_error)?,
        item_relations: repo::fetch_all_item_relations(conn)
            .await
            .map_err(internal_db_error)?,
        item_links: repo::fetch_all_item_links(conn)
            .await
            .map_err(internal_db_error)?,
        item_trailers: repo::fetch_all_item_trailers(conn)
            .await
            .map_err(internal_db_error)?,
        item_streaming_links: repo::fetch_all_item_streaming_links(conn)
            .await
            .map_err(internal_db_error)?,
        item_images: repo::fetch_all_item_images(conn)
            .await
            .map_err(internal_db_error)?,
        item_files: repo::fetch_all_item_files(conn)
            .await
            .map_err(internal_db_error)?,
        item_groups: repo::fetch_all_item_groups(conn)
            .await
            .map_err(internal_db_error)?,
        item_episodes: repo::fetch_all_item_episodes(conn)
            .await
            .map_err(internal_db_error)?,
        staff: repo::fetch_all_staff(conn)
            .await
            .map_err(internal_db_error)?,
        item_staff: repo::fetch_all_item_staff(conn)
            .await
            .map_err(internal_db_error)?,
        cast_members: repo::fetch_all_cast_members(conn)
            .await
            .map_err(internal_db_error)?,
        item_cast: repo::fetch_all_item_cast(conn)
            .await
            .map_err(internal_db_error)?,
    };

    tx.commit().await.map_err(internal_db_error)?;

    Ok(BackupFile {
        schema_version: SCHEMA_VERSION,
        exported_at: chrono::Utc::now().naive_utc(),
        data,
    })
}

/// schema_versionが対応バージョンかを検証する
pub fn validate_version(backup: &BackupFile) -> Result<(), ApiError> {
    if backup.schema_version != SCHEMA_VERSION {
        return Err(ApiError::new(
            ApiErrorCode::UnsupportedBackupVersion,
            format!(
                "未対応のバックアップバージョンです（file: {}, supported: {SCHEMA_VERSION}）",
                backup.schema_version
            ),
        ));
    }
    Ok(())
}

/// バックアップファイルを単一トランザクションでマージインポートする
pub async fn import_backup(
    pool: &PgPool,
    backup: BackupFile,
) -> Result<BackupImportReport, ApiError> {
    validate_version(&backup)?;

    let mut tx = pool.begin().await.map_err(internal_db_error)?;
    let mut report = BackupImportReport::default();
    let data = backup.data;

    /// 1テーブル分の行をINSERTし、inserted/skippedをレポートへ集計する
    macro_rules! import_table {
        ($name:literal, $rows:expr, $insert:path) => {{
            let mut count = TableImportCount::default();
            for row in &$rows {
                let inserted = $insert(tx.as_mut(), row)
                    .await
                    .map_err(|err| map_import_db_error($name, err))?;
                if inserted {
                    count.inserted += 1;
                } else {
                    count.skipped += 1;
                }
            }
            report.total_inserted += count.inserted;
            report.total_skipped += count.skipped;
            report.tables.insert($name.to_string(), count);
        }};
    }

    // FK制約を満たす順序でインポートする（親→子）
    import_table!("items", data.items, repo::insert_item);
    import_table!("tags", data.tags, repo::insert_tag);
    import_table!("categories", data.categories, repo::insert_category);
    import_table!("mylists", data.mylists, repo::insert_mylist);
    import_table!("staff", data.staff, repo::insert_staff);
    import_table!("cast_members", data.cast_members, repo::insert_cast_member);
    import_table!("item_tags", data.item_tags, repo::insert_item_tag);
    import_table!(
        "item_categories",
        data.item_categories,
        repo::insert_item_category
    );
    import_table!("mylist_items", data.mylist_items, repo::insert_mylist_item);
    import_table!(
        "item_relations",
        data.item_relations,
        repo::insert_item_relation
    );
    import_table!("item_links", data.item_links, repo::insert_item_link);
    import_table!(
        "item_trailers",
        data.item_trailers,
        repo::insert_item_trailer
    );
    import_table!(
        "item_streaming_links",
        data.item_streaming_links,
        repo::insert_item_streaming_link
    );
    import_table!("item_images", data.item_images, repo::insert_item_image);
    import_table!("item_files", data.item_files, repo::insert_item_file);
    import_table!("item_staff", data.item_staff, repo::insert_item_staff);
    import_table!("item_cast", data.item_cast, repo::insert_item_cast);
    import_table!("item_groups", data.item_groups, repo::insert_item_group);
    import_table!(
        "item_episodes",
        data.item_episodes,
        repo::insert_item_episode
    );

    tx.commit().await.map_err(internal_db_error)?;

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::backup::BackupData;

    fn backup_with_version(version: u32) -> BackupFile {
        BackupFile {
            schema_version: version,
            exported_at: chrono::Utc::now().naive_utc(),
            data: BackupData::default(),
        }
    }

    /// 対応バージョンは検証を通過する
    #[test]
    fn validate_version_accepts_supported_version() {
        assert!(validate_version(&backup_with_version(SCHEMA_VERSION)).is_ok());
    }

    /// 未対応バージョンはUNSUPPORTED_BACKUP_VERSION（400）で拒否される
    #[test]
    fn validate_version_rejects_unsupported_version() {
        let err = validate_version(&backup_with_version(SCHEMA_VERSION + 1)).unwrap_err();
        assert_eq!(err.error.code, "UNSUPPORTED_BACKUP_VERSION");
        assert_eq!(err.status, axum::http::StatusCode::BAD_REQUEST);
    }
}
