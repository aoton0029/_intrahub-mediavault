//! バックアップのエクスポート/インポートハンドラ
//!
//! GET /backup/export: 全データのバージョン付きJSONをダウンロード形式で返す。
//! ボディはエンベロープそのもの（ApiOkラップなし）とし、保存したファイルを
//! そのままPOST /backup/importへ再アップロードできる対称形にする。
//! POST /backup/import: バックアップJSONをマージインポートし、件数レポートを返す。

use axum::Json;
use axum::extract::State;
use axum::http::header;
use axum::response::IntoResponse;

use crate::AppState;
use crate::models::backup::{BackupFile, BackupImportReport};
use crate::models::response::{ApiError, ApiOk};
use crate::services::backup_service;

/// GET /backup/export
pub async fn export_backup_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let backup = backup_service::export_backup(&state.db).await?;
    let filename = format!(
        "mediavault-backup-{}.json",
        chrono::Utc::now().format("%Y%m%d%H%M%S")
    );
    Ok((
        [(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )],
        Json(backup),
    ))
}

/// POST /backup/import
pub async fn import_backup_handler(
    State(state): State<AppState>,
    Json(backup): Json<BackupFile>,
) -> Result<ApiOk<BackupImportReport>, ApiError> {
    let report = backup_service::import_backup(&state.db, backup).await?;
    Ok(ApiOk::new(report))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::AppState;
    use crate::models::backup::BackupFile;
    use crate::routes::build_router;

    /// docker-composeのテスト用Postgres（DATABASE_URL環境変数）へ接続する。
    /// 本モジュールの統合テストはすべて#[ignore]（cargo test -- --ignored で実行）
    async fn test_app_state() -> AppState {
        let database_url = std::env::var("DATABASE_URL")
            .expect("バックアップ統合テストにはDATABASE_URLが必要です");
        let db = sqlx::PgPool::connect(&database_url)
            .await
            .expect("テスト用DBへの接続に失敗しました");
        AppState {
            db,
            internal_api_key: String::new(),
        }
    }

    async fn read_body_json(response: axum::response::Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    /// エクスポート→同一ファイルの再インポートで全行skippedになる（ラウンドトリップ）
    #[tokio::test]
    #[ignore]
    async fn export_then_reimport_skips_all_existing_rows() {
        let state = test_app_state().await;

        // シード: item + tag + item_tag
        let item_id: Uuid = sqlx::query_scalar(
            "INSERT INTO items (media_type, title, source) VALUES ('anime', 'バックアップテスト', 'manual') RETURNING id",
        )
        .fetch_one(&state.db)
        .await
        .unwrap();
        let tag_id: Uuid = sqlx::query_scalar(
            "INSERT INTO tags (name) VALUES ('backup-test-tag') ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id",
        )
        .fetch_one(&state.db)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO item_tags (item_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(item_id)
        .bind(tag_id)
        .execute(&state.db)
        .await
        .unwrap();

        // エクスポート
        let app = build_router(state.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/backup/export")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let disposition = response
            .headers()
            .get(axum::http::header::CONTENT_DISPOSITION)
            .expect("Content-Dispositionヘッダが必要")
            .to_str()
            .unwrap()
            .to_string();
        assert!(disposition.contains("attachment"));
        assert!(disposition.contains("mediavault-backup-"));

        let exported = read_body_json(response).await;
        let backup: BackupFile = serde_json::from_value(exported.clone()).unwrap();
        assert!(backup.data.items.iter().any(|i| i.id == item_id));
        assert!(backup.data.item_tags.iter().any(|t| t.item_id == item_id));

        // 同一ファイルを再インポート → 全行skipped
        let app = build_router(state.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/backup/import")
                    .header("content-type", "application/json")
                    .body(Body::from(exported.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let report = read_body_json(response).await;
        assert_eq!(report["success"], true);
        assert_eq!(report["data"]["total_inserted"], 0);
        assert!(report["data"]["total_skipped"].as_u64().unwrap() >= 3);

        // 後始末（tagはUNIQUE nameのため削除）
        sqlx::query("DELETE FROM items WHERE id = $1")
            .bind(item_id)
            .execute(&state.db)
            .await
            .unwrap();
        sqlx::query("DELETE FROM tags WHERE id = $1")
            .bind(tag_id)
            .execute(&state.db)
            .await
            .unwrap();
    }

    /// 参照先itemが存在しないitem_tagsを含むファイル → 400・DB無変化（ロールバック）
    #[tokio::test]
    #[ignore]
    async fn import_with_dangling_reference_returns_400_and_rolls_back() {
        let state = test_app_state().await;
        let orphan_item = Uuid::new_v4();
        let orphan_tag = Uuid::new_v4();
        let new_tag_id = Uuid::new_v4();

        // 新規tag（本来insertされる行）+ danglingなitem_tags → 全体ロールバックされるはず
        let body = serde_json::json!({
            "schema_version": 1,
            "exported_at": "2026-07-18T00:00:00",
            "data": {
                "tags": [{"id": new_tag_id, "name": "backup-rollback-tag"}],
                "item_tags": [{"item_id": orphan_item, "tag_id": orphan_tag}]
            }
        });

        let app = build_router(state.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/backup/import")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // ロールバック確認: tagsに新規行が残っていない
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tags WHERE id = $1")
            .bind(new_tag_id)
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    /// schema_version不一致 → 400 UNSUPPORTED_BACKUP_VERSION
    #[tokio::test]
    #[ignore]
    async fn import_with_unsupported_version_returns_400() {
        let state = test_app_state().await;
        let body = serde_json::json!({
            "schema_version": 999,
            "exported_at": "2026-07-18T00:00:00",
            "data": {}
        });

        let app = build_router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/backup/import")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let json = read_body_json(response).await;
        assert_eq!(json["error"]["code"], "UNSUPPORTED_BACKUP_VERSION");
    }
}
