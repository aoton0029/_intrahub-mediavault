# TDD開発メモ: item-file-upload

## 概要

- 機能名: item-file-upload（POST /items/:id/files/upload、multipart/form-dataバイナリ直接アップロード）
- 開発開始: 2026-06-26
- 現在のフェーズ: Red

## 関連ファイル

- 元タスクファイル: `docs/tasks/mediavault-backend/TASK-0027.md`
- 要件定義: `docs/implements/mediavault-backend/TASK-0027/item-file-upload-requirements.md`
- テストケース定義: `docs/implements/mediavault-backend/TASK-0027/item-file-upload-testcases.md`
- 実装ファイル（未実装・スタブのみ）:
  - `backend/mediavault-api/src/services/file_storage.rs`（新規、`todo!()`スタブ）
  - `backend/mediavault-api/src/handlers/item_files.rs`（`upload_item_file_handler`は未追加）
  - `backend/mediavault-api/src/routes/mod.rs`（`/items/:id/files/upload`ルート未追加）
- テストファイル:
  - `backend/mediavault-api/src/services/file_storage.rs`（`#[cfg(test)] mod tests`、11件）
  - `backend/mediavault-api/src/handlers/item_files.rs`（`#[cfg(test)] mod tests`、9件追加）
  - `backend/mediavault-api/src/models/response.rs`（`FileStorageWriteFailed`マッピングテスト1件追加）

## Redフェーズ（失敗するテスト作成）

### 作成日時

2026-06-26

### テストケース概要

テストケース定義書（22ケース）のうち、サービス単体11件＋ハンドラ統合9件＝20件を実装（うち4件のresolve_base_dir系は実装済みロジックのためok、残りは`todo!()`によりFAILED。ハンドラ統合9件は`#[ignore]`でDB必須・未実行）。

設計決定（テストケース定義書 第8章を本フェーズで確定）:
1. `photo`は`FileType::Image`の同義語（enum拡張なし、画像ディレクトリへ配置）
2. `FileType::Other`も画像ディレクトリへ集約
3. `ApiErrorCode::FileStorageWriteFailed`（500/`FILE_STORAGE_WRITE_FAILED`）を新規追加・実装済み
4. 書込失敗注入は`FileWriter`トレイト（`TokioFileWriter`/`FailingFileWriter`）で実現
5. ボディサイズ上限は本フェーズでは未テスト（ルーター層で対応予定）
6. 空ファイルは許容方針（201）

### テストコード

詳細は `backend/mediavault-api/src/services/file_storage.rs`・`backend/mediavault-api/src/handlers/item_files.rs`・`backend/mediavault-api/src/models/response.rs` を参照（本ファイルには差分の全文は転記せず、ファイルパスのみ記録する）。

### 期待される失敗

- `cargo test --bin mediavault-api services::file_storage`: 11件中7件が`todo!()`パニックで失敗（Red期待どおり）。`resolve_base_dir`系4件のみ実装済みロジックのためok。
- `cargo check --tests`: コンパイル自体は成功（警告のみ）。ハンドラ統合テストは`#[ignore]`のため通常実行はスキップされ、`/items/:id/files/upload`ルート未登録・`upload_item_file_handler`未実装の状態でも今はコンパイルエラーにならない（ルーターのオプショナルなパスにアクセスする形のテストのため、実行時に404/期待コード不一致として失敗する設計）。

### 次のフェーズへの要求事項（Green）

1. `src/services/file_storage.rs`の`generate_object_name`/`store_file`/`cleanup_file`/`TokioFileWriter`の`todo!()`を実装に置き換える
2. `src/handlers/item_files.rs`に`upload_item_file_handler`（`axum::extract::Multipart`受信→検証→service呼出→repository呼出→201/400/404/500マッピング）を追加する
3. `src/routes/mod.rs`に`/items/:id/files/upload`ルートを追加し、`DefaultBodyLimit`を設定する
4. Green完了後、`DATABASE_URL`を設定して`cargo test -- --ignored`でハンドラ統合9件を実行し、全てpassすることを確認する
