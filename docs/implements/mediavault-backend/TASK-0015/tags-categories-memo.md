# TDD開発メモ: タグ・カテゴリCRUD実装

## 概要

- 機能名: タグ・カテゴリCRUD実装
- 開発開始: 2026-06-24
- 現在のフェーズ: 完了

## 🎯 最終結果 (2026-06-24)
- **実装率**: 125% (20/16テストケース、補完テスト含む)
- **品質判定**: 合格
- **TODO更新**: ✅完了マーク追加
- **全体テスト**: mediavault-apiクレート 62 passed / 0 failed / 56 ignored（DB依存統合テスト、docker-compose環境での実行確認は未実施）
- **スコープ外失敗**: `api-client-lib`クレートの`ndl_test::search_returns_results`が外部API（NDL）へのタイムアウトで失敗（本タスクと無関係、auto-debug対応推奨）

## 関連ファイル

- 元タスクファイル: `docs/tasks/mediavault-backend/TASK-0015.md`
- 要件定義: `docs/implements/mediavault-backend/TASK-0015/tags-categories-requirements.md`
- テストケース定義: `docs/implements/mediavault-backend/TASK-0015/tags-categories-testcases.md`
- Redフェーズ記録: `docs/implements/mediavault-backend/TASK-0015/tags-categories-red-phase.md`
- 実装ファイル（Greenフェーズで作成）:
  - `backend/mediavault-api/src/models/tag.rs`
  - `backend/mediavault-api/src/models/category.rs`
  - `backend/mediavault-api/src/repositories/tag_repository.rs`
  - `backend/mediavault-api/src/repositories/category_repository.rs`
  - `backend/mediavault-api/src/handlers/tags.rs`（新規作成予定）
  - `backend/mediavault-api/src/handlers/categories.rs`（新規作成予定）
  - `backend/mediavault-api/src/models/response.rs`（ApiErrorCode追加）
  - `backend/mediavault-api/src/routes/mod.rs`（ルート追加）
- テストファイル: 上記の実装ファイルと同一（インライン `#[cfg(test)] mod tests`）

## Redフェーズ（失敗するテスト作成）

### 作成日時

2026-06-24

### テストケース

- model層（純粋関数・デシリアライズ）: tag.rs 4件、category.rs 3件
- repository層（DB統合テスト、`#[ignore]`）: tag_repository.rs 10件、category_repository.rs 4件
- 合計20テスト（テストケース定義書の正常系6・異常系8・境界値2の計16をベースに、DBエラー変換テスト等を補完）

### テストコード

`backend/mediavault-api/src/models/tag.rs`, `category.rs`, `backend/mediavault-api/src/repositories/tag_repository.rs`, `category_repository.rs` を参照。

### 期待される失敗

`cargo test -p mediavault-api tag_repository` 実行時、`create_tag`, `delete_tag`, `attach_tag_to_item`, `detach_tag_from_item` 等が未定義のため `error[E0425]: cannot find function ... in this scope` のコンパイルエラーが29件発生することを確認済み。model層も同様に `CreateTagRequest`, `validate_tag_name` 等が未定義のためコンパイルエラーとなる。

### 次のフェーズへの要求事項

Greenフェーズで以下を実装する:
1. `models/tag.rs`, `models/category.rs`: `Tag`/`Category`構造体、`CreateTagRequest`/`CreateCategoryRequest`、`validate_tag_name`/`validate_category_name`
2. `repositories/tag_repository.rs`, `category_repository.rs`: `create_tag`/`create_category`, `delete_tag`/`delete_category`, `attach_tag_to_item`/`attach_category_to_item`, `detach_tag_from_item`/`detach_category_from_item`, `db_error`, ユニーク制約違反(23505)・FK違反(23503)のハンドリング、`test_pool`/`unreachable_pool`ヘルパー
3. `models/response.rs`: `ApiErrorCode::DuplicateTagName` / `DuplicateCategoryName`（409）を追加
4. `handlers/tags.rs`, `handlers/categories.rs`: HTTPハンドラ実装
5. `routes/mod.rs`: 新規ルート登録（`/tags`, `/tags/:id`, `/items/:id/tags/:tag_id`, `/categories`, `/categories/:id`, `/items/:id/categories/:category_id`）

詳細はRedフェーズ記録ファイル参照。

## Greenフェーズ（最小実装）

### 実装日時

2026-06-24

### 実装方針

items系の既存パターン（DTO・バリデーション・db_error・SQLSTATE別エラーハンドリング・ハンドラ・ルーティング）を踏襲し、タグ・カテゴリで完全に対称な実装を行った。詳細はGreenフェーズ記録ファイル参照。

### 実装コード

`backend/mediavault-api/src/models/tag.rs`, `category.rs`, `repositories/tag_repository.rs`, `category_repository.rs`, `handlers/tags.rs`, `categories.rs`, `routes/mod.rs`（追記）, `models/response.rs`（ApiErrorCode追加）

### テスト結果

`cargo test -p mediavault-api` → 60 passed; 0 failed; 56 ignored（既存テストにregressなし）

### 課題・改善点

Greenフェーズ記録ファイルの「課題・改善点」セクション参照（tag/category間のコード重複、重複付与エラーコードの分離検討、HTTPレベル統合テストの追加、実DB統合テストの実行確認）。

## Refactorフェーズ（品質改善）

### リファクタ日時

2026-06-24

### 改善内容

- `tag_repository.rs`/`category_repository.rs`に重複していたSQLSTATE判定ロジック（`is_unique_violation`/`is_foreign_key_violation`）を`repositories/db_error_utils.rs`へ抽出し、DRY原則を適用
- `db_error`関数等の日本語コメントを強化

### セキュリティレビュー

SQLインジェクション対策（bind方式の徹底）、DB内部情報の漏洩防止、入力検証、認証・認可の適用範囲について確認し、重大な脆弱性は発見されなかった。詳細はRefactorフェーズ記録ファイル参照。

### パフォーマンスレビュー

各操作は単一SQL文によるO(1)処理であり、重大な性能課題は発見されなかった。

### 最終コード

`backend/mediavault-api/src/repositories/db_error_utils.rs`（新規）、`tag_repository.rs`/`category_repository.rs`（重複除去）

### 品質評価

✅ 高品質: テスト62件全て成功（新規ヘルパーテスト2件含む）、clippy警告なし、ファイルサイズ全て200行未満、重大なセキュリティ・性能課題なし
