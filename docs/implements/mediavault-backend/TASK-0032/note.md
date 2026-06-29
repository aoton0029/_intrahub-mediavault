# TASK-0032 開発ノート: 主要フロー統合テスト実装

## 1. 技術スタック
- Rust edition 2024 / workspace resolver "3"
- axum 0.8.9（features: multipart）/ tokio 1.52.3 (full) / sqlx 0.8 (postgres, runtime-tokio, macros, chrono, uuid)
- dev-dependencies（`backend/mediavault-api/Cargo.toml`）: `wiremock = "0.6"`, `tempfile = "3"`（既存・追加不要）
- 参照元: backend/mediavault-api/Cargo.toml

## 2. 開発ルール
- `AGENTS.md` / `docs/rule/` は本リポジトリに存在しない（追加ルールなし）。
- インラインテスト（`#[cfg(test)] mod tests`）が既存方針だが、本タスクは別ファイル統合テスト（`tests/`配下）を新設する。
- 実DB必要なテストは `#[tokio::test]` + `#[ignore]`、`cargo test -- --ignored` 実行パターンを既存ルーティングテストが採用（後述）。

## 3. 関連実装
### test_app_state() ヘルパー（既存・参考にする）
- backend/mediavault-api/src/routes/mod.rs:186-196
  ```rust
  async fn test_app_state() -> AppState {
      let database_url = std::env::var("DATABASE_URL").expect(...);
      let db = sqlx::PgPool::connect(&database_url).await.expect(...);
      AppState { db, internal_api_key: String::new() }
  }
  ```
- backend/mediavault-api/src/routes/internal.rs:63-73（同様の内部API用ヘルパー）
- 内部APIキーは `AppState.internal_api_key` ではなく `std::env::var("INTERNAL_API_KEY")` を直接読む（middleware側）。テストでは `std::env::set_var("INTERNAL_API_KEY", key)` で設定する（internal.rs:75-83既存パターン）。
- **注意**: `std::env::set_var` はRust最新版でunsafe化されている可能性があるため、既存コードに倣う（internal.rs参照）。

### ExternalSearchService のテスト用DI
- backend/mediavault-api/src/services/external_search.rs:136-190
- `ApiCredentialLookup::Fixed(closure)` でDB不要のAPIキー注入。
- `with_test_base_urls()` でJikan/TMDb/OpenLibrary/NDL/IGDBの各ベースURLをwiremockの`MockServer`URLに差し替え可能。

### item_files（パス指定）
- リクエスト: `CreateItemFileRequest { path: String, label: Option<String>, file_type: FileType }`
- レスポンス: `ItemFile { id, item_id, path, label, file_type, calibre_book_id, created_at }`
- 参照元: backend/mediavault-api/src/models/item_file.rs, backend/mediavault-api/src/handlers/item_files.rs:23-42

### item_episodes（EDGE-101）
- `POST /groups/:group_id/episodes` ハンドラ内で`group_type == GroupType::Volume`なら`ApiErrorCode::InvalidGroupTypeForEpisodes`（400, `INVALID_GROUP_TYPE_FOR_EPISODES`）を返す。
- 参照元: backend/mediavault-api/src/handlers/item_episodes.rs:21-49

### API key認証ミドルウェア
- backend/mediavault-api/src/middleware/api_key_auth.rs:15-30
- `Authorization: Bearer <key>` ヘッダーを`INTERNAL_API_KEY`環境変数と比較。不一致・欠落は401 `Unauthorized`。
- 内部ルーター（`build_internal_router`）にのみ`.layer(axum::middleware::from_fn(api_key_auth))`適用（internal.rs:45）。公開ルーターには認証なし。

## 4. 設計文書
- 概要: docs/tasks/mediavault-backend/overview.md
- タスク詳細: docs/tasks/mediavault-backend/TASK-0032.md
- 要件/受け入れ基準: docs/spec/mediavault-backend/requirements.md, docs/spec/mediavault-backend/acceptance-criteria.md
- 開発ノート（既存タスクの実装詳細）: docs/spec/mediavault-backend/note.md

## 5. テスト関連情報
- 統合テスト用ディレクトリ `backend/mediavault-api/tests/` は未作成（新設が必要）。
- ルート定義一覧（実装済み・全フェーズ完了確認済み）: backend/mediavault-api/src/routes/mod.rs（全体）, backend/mediavault-api/src/routes/internal.rs
  - `POST/GET /items`, `GET /items/search`, `POST /items/import`, `GET/PATCH/DELETE /items/:id`, `PATCH /items/:id/status`
  - `POST/DELETE /tags`, `/categories`, `/mylists`, `/item-relations`
  - `POST/GET /items/:id/groups`, `POST/GET /groups/:group_id/episodes`
  - `POST /staff`, `/items/:id/staff`
  - `POST /items/:id/files`（パス指定）, `POST /items/:id/files/upload`（multipart, 100MB上限）, `PATCH /items/:id/files/:file_id/calibre-link`
  - `POST/DELETE /items/:id/links`, `/items/:id/trailers`
  - `PUT /settings/api-keys/:provider`
  - `POST /import/booklog`, `POST /import/steam`
  - `/internal/items`(POST), `/internal/items/search`(GET), `/internal/items/:id`(PATCH), `/internal/items/:id/groups`(POST), `/internal/groups/:group_id/episodes`(POST), `/internal/items/:id/files`(POST) — 全て`api_key_auth`ミドルウェア配下
- DB接続: `DATABASE_URL`環境変数（テスト専用変数は別途用意されていない。`docker-compose up -d db`前提）。
- 既存統合テストは全て`#[ignore]`属性付き、`cargo test -- --ignored`で実行。

## 6.5 完了確認（tdd-verify-complete）

- `cargo check --tests -p mediavault-api` はクリーンに成功（warning 0件、refactor後）。
- 完了条件8項目中7項目は実装済み（IT-001/002/004/005/006/007/008/009/010/011でカバー）。
- **未達1件**: 外部API検索→インポートフロー（IT-003）はテストシナリオを記述済みだが`#[ignore = "no test seam..."]`でスキップ。
  - 原因: `handlers/items.rs`の`search_items_handler`が`ExternalSearchService::new(state.db.clone())`を直接構築し、`with_test_base_urls`/`with_fixed_credentials`は`#[cfg(test)]`ガード付きでサービス自身のユニットテストからのみ参照可能（`tests/`配下の外部統合テストクレートからはリンク不可）。
  - 対応方針（フォローアップ推奨）: `services/external_search.rs`にテスト注入用の seam（例: `cfg(feature = "test-util")`での公開、または`AppState`経由のDI）を追加する小タスクを別途切り出す。本タスクは「統合テスト実装」が主目的のためプロダクションコード変更は対象外とした。
- `cargo test -- --ignored`実MWeb実行は本環境にPostgresが起動していないため未実施（CI＝TASK-0033で実行される前提）。

## 6. 注意事項
- 外部API実通信禁止。`wiremock`（既存dev-dependency）でスタブ化すること。
- カスケード削除の確認は削除後の関連テーブル直接SELECTで0件確認。
- 当初overview.mdではTASK-0021/0027/0028/0030が未完了表示だったが、実コード確認の結果ハンドラ・ルートは実装済みと判明したため、本タスクではフルスコープ（アップロード方式・calibre-link等含む）で統合テストを実装する方針とする（チェックボックスは本タスク着手前に修正済み）。
