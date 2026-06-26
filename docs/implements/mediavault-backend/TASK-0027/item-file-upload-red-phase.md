# TASK-0027 Redフェーズ記録: POST /items/:id/files/upload（バイナリ直接アップロード）

- **機能名**: item-file-upload
- **タスクID**: TASK-0027
- **要件名**: mediavault-backend
- **作成日**: 2026-06-26

## 1. 設計決定（テストケース定義書 第8章の未確定事項の確定）

Red着手前に以下を確定した。

1. `file_type="photo"` は既存 `FileType::Image` の表記ゆれとして扱う（enum拡張なし）。画像ディレクトリ（`MEDIA_STORAGE_PATH`、デフォルト `/srv/media/photos`）に配置する。
2. `FileType::Other` も画像ディレクトリ（`MEDIA_STORAGE_PATH`）に配置する（pdf以外を集約）。
3. `ApiErrorCode::FileStorageWriteFailed`（`"FILE_STORAGE_WRITE_FAILED"` / 500）を `src/models/response.rs` に追加した（実装済み・テスト済み）。
4. 書込失敗注入は `FileWriter` トレイトで行う。本番実装は `TokioFileWriter`、テストでは常に失敗する `FailingFileWriter` を注入する。
5. ボディサイズ上限（`DefaultBodyLimit`）はルーター層で対応する前提とし、本Redフェーズでは個別テスト化しない。
6. 空（0バイト）ファイルアップロードは許容する（拒否しない）。

## 2. 作成・変更したファイル

- `backend/mediavault-api/Cargo.toml`: `tempfile = "3"` をdev-dependenciesへ追加
- `backend/mediavault-api/src/models/response.rs`: `ApiErrorCode::FileStorageWriteFailed` variant追加（500・`FILE_STORAGE_WRITE_FAILED`）+ 単体テスト1件追加
- `backend/mediavault-api/src/services/file_storage.rs`（新規）: `resolve_base_dir`/`generate_object_name`/`FileWriter`トレイト/`TokioFileWriter`/`FailingFileWriter`/`store_file`/`cleanup_file`のスタブ（`todo!()`）+ 単体テスト11件
- `backend/mediavault-api/src/services/mod.rs`: `pub mod file_storage;` 追加
- `backend/mediavault-api/src/handlers/item_files.rs`: ハンドラ統合テスト9件追加（`multipart_body()`/`multipart_body_without_file()`/`insert_test_item()`ヘルパー含む）。`upload_item_file_handler`・`/items/:id/files/upload` ルートは未実装

## 3. 作成したテストケース一覧

### サービス単体（`src/services/file_storage.rs`、DB不要・`#[test]`）

| テスト名 | 対応テストケースID | 結果 |
|---|---|---|
| `generate_object_name_returns_uuid_with_original_extension` | TC-019-U04 | FAILED（`todo!()`） |
| `generate_object_name_does_not_collide_on_repeated_calls` | TC-019-U04 | FAILED（`todo!()`） |
| `generate_object_name_handles_filename_without_extension` | TC-019-B03 | FAILED（`todo!()`） |
| `generate_object_name_neutralizes_path_traversal_in_filename` | TC-019-E06 | FAILED（`todo!()`） |
| `store_file_returns_relative_path_under_base_dir` | TC-019-U05 | FAILED（`todo!()`） |
| `store_file_with_failing_writer_returns_file_storage_write_failed_error` | TC-019-E01 | FAILED（`todo!()`） |
| `resolve_base_dir_for_image_uses_media_storage_path` | TC-019-02 | ok（resolve_base_dirは実装済み） |
| `resolve_base_dir_for_other_uses_media_storage_path` | TC-019-B05 | ok |
| `resolve_base_dir_for_pdf_uses_pdf_storage_path` | TC-019-01 | ok |
| `resolve_base_dir_default_does_not_point_to_current_directory` | TC-019-E07 | ok |
| `cleanup_file_removes_previously_written_file` | TC-019-E02 / IT-019-02 | FAILED（`todo!()`） |

### ハンドラ統合（`src/handlers/item_files.rs`、`#[tokio::test]` + `#[ignore]`、`DATABASE_URL`必須）

| テスト名 | 対応テストケースID |
|---|---|
| `post_item_file_upload_with_pdf_returns_201_and_relative_path` | TC-019-01 |
| `post_item_file_upload_with_image_returns_201` | TC-019-02 |
| `post_item_file_upload_without_label_returns_201` | TC-019-N03 |
| `post_item_file_upload_with_nonexistent_item_returns_404_and_no_file_written` | TC-019-03 |
| `post_item_file_upload_with_write_failure_returns_500_and_creates_no_record` | TC-019-E01 |
| `post_item_file_upload_with_invalid_file_type_returns_400` | TC-019-E03 |
| `post_item_file_upload_without_file_field_returns_400` | TC-019-E04 |
| `post_item_file_upload_with_invalid_uuid_path_returns_400` | TC-019-E05 |
| `post_item_file_upload_with_empty_file_returns_201` | TC-019-B01 |

合計: 11（サービス単体）+ 9（ハンドラ統合・`#[ignore]`）= 20件（既存3件の応答コードテストを含む`models/response.rs`は別途1件追加）。

## 4. 実行結果

```
cargo check --tests        # 成功（コンパイルエラーなし、警告のみ）
cargo test --bin mediavault-api services::file_storage
# running 11 tests
# 4 passed（resolve_base_dir系：実装済みロジック）
# 7 failed（generate_object_name/store_file/cleanup_file 経由のtodo!()パニック）

cargo test --bin mediavault-api models::response
# running 16 tests; 16 passed（FileStorageWriteFailedの500/コードマッピングも合格）
```

ハンドラ統合テスト（`#[ignore]`）は `DATABASE_URL` 未設定のため通常実行されないが、`cargo test --bin mediavault-api -- --list` でコンパイル・登録されていることを確認済み。Green実装後に `cargo test -- --ignored` で実DBに対して実行する。

## 5. 期待される失敗内容

- `services::file_storage::tests::*`: `generate_object_name`/`store_file`/`cleanup_file` が `todo!()` のため `panic` で失敗（Red期待）。
- `handlers::item_files::tests::post_item_file_upload_*`: `/items/:id/files/upload` ルートが `routes::build_router` に未登録のため、実行時は404（ルート不在）または`upload_item_file_handler`未実装により期待ステータス（201/400/404/500）と不一致になる想定（`#[ignore]`のためDATABASE_URL設定時に確認）。

## 6. Greenフェーズで実装すべき内容

1. `src/services/file_storage.rs`:
   - `generate_object_name()`: UUID v4 + 安全な拡張子抽出（`..`/`/`/`\`除去）の実装
   - `TokioFileWriter::write/remove`: `tokio::fs`（または`std::fs`）による実書込・削除
   - `store_file()`: `resolve_base_dir()` → `generate_object_name()` → `FileWriter::write()` → 失敗時`ApiErrorCode::FileStorageWriteFailed`変換 → 相対パス算出
   - `cleanup_file()`: `FileWriter::remove()` 呼び出し
2. `src/handlers/item_files.rs`:
   - `upload_item_file_handler`: `axum::extract::Multipart` でfile/file_type/label受信 → file_type検証 → item存在確認（書込前）→ `file_storage::store_file()` → 成功時`item_file_repository::create_item_file()` → 失敗時ファイルクリーンアップ → 201/400/404/500マッピング
3. `src/routes/mod.rs`: `/items/:id/files/upload` を `axum::routing::post(upload_item_file_handler)` で登録し、`DefaultBodyLimit::max(...)` をルートまたはアプリ全体に設定
4. `src/services/mod.rs`: 既に`file_storage`を公開済み（変更不要）
