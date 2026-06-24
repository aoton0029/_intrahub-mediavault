# TASK-0015 Refactorフェーズ記録: タグ・カテゴリCRUD実装

## 実施した改善

### 1. SQLSTATE判定ロジックの重複除去（DRY原則） 🟡
- **Before**: `tag_repository.rs`と`category_repository.rs`それぞれに`is_unique_violation`/`is_foreign_key_violation`を個別定義（完全に同一コードが2箇所に存在）
- **After**: `backend/mediavault-api/src/repositories/db_error_utils.rs`を新規作成し、両関数を集約。両リポジトリから`use crate::repositories::db_error_utils::{is_foreign_key_violation, is_unique_violation};`でインポート
- **効果**: SQLSTATEコードの判定ロジックが1箇所に集約され、将来的に他テーブル（item_relations等）でも再利用可能。単体テスト2件（`is_unique_violation_returns_false_for_non_database_error`等）も追加し、ヘルパー自体の正しさを保証

### 2. 日本語コメントの強化 🟡
- `db_error`関数のコメントに「【改善内容】」セクションを追加し、SQLSTATE判定の責任分離を明記
- `db_error_utils.rs`の各関数に【ヘルパー関数】【再利用性】【単一責任】の観点でコメントを付与

## セキュリティレビュー結果

- 🔵 **SQLインジェクション対策**: 全てのSQL文で`sqlx::query`/`query_as`の`bind()`によるプレースホルダ方式を使用しており、文字列結合による動的SQL構築は一切行っていない。タグ名・カテゴリ名・UUIDはすべてbindパラメータとして渡されるため、インジェクションの危険はない
- 🔵 **DB内部情報の漏洩防止**: `db_error`関数で内部エラー詳細を`tracing::error!`にのみ出力し、クライアントへは固定の汎用メッセージのみ返す方針を継続（item_repository.rsの既存方針と一致）
- 🟡 **入力検証**: `validate_tag_name`/`validate_category_name`で空文字・空白のみの入力を拒否。UUID形式の検証は既存`parse_item_id`を再利用しており、不正な形式は400で早期に拒否される
- 🔵 **認証・認可**: 本タスクのエンドポイントは既存ルーター構成にそのまま追加されており、既存の認証ミドルウェア（APIキー検証等、内部API向け）の適用範囲外には出ていない（`/tags`等は既存`/items`と同様の公開APIとして扱う設計を維持）
- 特筆すべき脆弱性は発見されなかった

## パフォーマンスレビュー結果

- 🔵 **計算量**: `create_tag`/`delete_tag`/`attach_tag_to_item`/`detach_tag_from_item`はいずれも単一のSQL文（O(1)のINSERT/DELETE、インデックス付きPK/UNIQUE制約による高速な制約チェック）であり、計算量上の懸念はない
- 🔵 **DB接続**: 既存`AppState`の共有`PgPool`をそのまま使い回しており、リクエストごとに新規接続を確立する非効率な実装ではない
- 🟡 **N+1懸念**: 本タスクの範囲（単発のCRUD・付与・解除）ではN+1問題は発生しない。一覧取得系のエンドポイントは本タスクのスコープ外（item一覧側でtag_id/category_idによる絞り込みはTASK-0010で対応済み）
- 重大な性能課題は発見されなかった

## テスト実行結果（リファクタリング後）

```bash
cd backend
cargo test -p mediavault-api
```

```
test result: ok. 62 passed; 0 failed; 56 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

- リファクタ前の60件 + db_error_utils.rsの新規ヘルパーテスト2件 = 62件、全て成功
- 全テストの実行時間は実質0秒（2秒を超える遅いテストは検出されなかった）
- `describe.skip`等によるテスト無効化、`testPathIgnorePatterns`等での除外は存在しない（Rustプロジェクトのため該当機能自体がない）
- 開発時生成ファイル（`debug-*`, `temp-*`, `*.bak`等）は検出されなかった

## clippyによる静的解析

```bash
cargo clippy -p mediavault-api --all-targets
```
新規追加コード（tag.rs, category.rs, tag_repository.rs, category_repository.rs, handlers/tags.rs, handlers/categories.rs, db_error_utils.rs）について警告は検出されなかった。

## ファイルサイズ

全ファイル200行未満（500行制限に対し十分余裕あり、分割不要）

## 残課題（本タスクのスコープ外として保留）

1. HTTPレベルの統合テスト（ルーター経由、`routes/mod.rs`の既存パターンに倣ったもの）は未追加。verify-completeフェーズでの判定次第で追加を検討
2. 実DB統合テスト（`#[ignore]`、19件）はdocker-compose環境での実行確認が未実施。マージ前にCIまたは手動で`cargo test -- --ignored`の実行を推奨
3. `attach_tag_to_item`の重複付与エラーが`DuplicateTagName`を流用している点は、タスク仕様L45「no-opまたは409」のうち409を選択した実装判断として維持（要件定義書に明記済み）
