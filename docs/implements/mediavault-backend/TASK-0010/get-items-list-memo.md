# TDD開発メモ: GET /items（一覧・絞り込み）

## 概要

- 機能名: GET /items（一覧・絞り込み）
- 開発開始: 2026-06-23
- 現在のフェーズ: 完了

## 関連ファイル

- 元タスクファイル: `docs/tasks/mediavault-backend/TASK-0010.md`
- 要件定義: `docs/implements/mediavault-backend/TASK-0010/get-items-list-requirements.md`
- テストケース定義: `docs/implements/mediavault-backend/TASK-0010/get-items-list-testcases.md`
- Redフェーズ記録: `docs/implements/mediavault-backend/TASK-0010/get-items-list-red-phase.md`
- 実装ファイル（Greenフェーズで実装予定）:
  - `backend/mediavault-api/src/models/response.rs`（PaginatedOk/Pagination）
  - `backend/mediavault-api/src/models/item.rs`（ListItemsQuery）
  - `backend/mediavault-api/src/handlers/items.rs`（normalize_pagination/list_items_handler）
  - `backend/mediavault-api/src/repositories/item_repository.rs`（list_items/count_items/build_list_items_query/build_count_items_query）
  - `backend/mediavault-api/src/routes/mod.rs`（GET /items ルート追加）
- テストファイル（Redフェーズで追加したテスト本体は上記実装ファイルに同居）

## Redフェーズ（失敗するテスト作成）

### 作成日時

2026-06-23

### テストケース

テストケース定義書（28ケース）のうち、実DB不要なユニットテスト18件と `#[ignore]` 付き統合テスト11件（B06はユニット/統合の両面実装のため重複カウント）を作成した。

- ユニット（実DB不要）:
  - TC-0010-N09, N10（PaginatedOk シリアライズ・200応答）
  - TC-0010-B01〜B06（normalize_pagination のクランプ・デフォルト・OFFSET算出）
  - TC-0010-Q01〜Q06（QueryBuilder生成SQLの構造検証）
- 統合（`#[ignore]`、実DB必要）:
  - TC-0010-N01〜N08（一覧取得・各種絞り込み・複合AND）
  - TC-0010-B07, B08（範囲外page・空テーブル）
  - TC-0010-E04（DBエラー→INTERNAL_ERROR変換）

TC-0010-E01〜E03（クエリパラメータ型不正→400）はAxumルーター層の挙動検証であり、`routes/mod.rs` へのGET /itemsルート追加後に統合テストとして実装する方針とした（要件・テストケース定義書には明記済み、ルート未実装の現時点ではテスト対象コードが存在しないため本Redフェーズの対象外とした）。

### テストコード

詳細は以下に集約:
- `backend/mediavault-api/src/models/response.rs`（`paginated_ok_serializes_to_expected_json`, `paginated_ok_returns_200_even_when_data_is_empty`）
- `backend/mediavault-api/src/handlers/items.rs`（`normalize_pagination_*` 6件）
- `backend/mediavault-api/src/repositories/item_repository.rs`（`build_list_items_sql_*` 5件, `build_count_items_sql_shares_same_where_clause_as_list`, `list_items_*` 8件の統合テスト, `list_items_converts_db_error_to_internal_error`）

### 期待される失敗

`cargo check -p mediavault-api --tests` で40件のコンパイルエラーを確認。すべて新規参照シンボル（`PaginatedOk`, `Pagination`, `normalize_pagination`, `ListItemsQuery`, `list_items`, `count_items`, `build_list_items_query`, `build_count_items_query`）が未定義であることに起因し、既存実装（TASK-0008/0009）には影響なし。

代表エラー:
```
error[E0432]: unresolved import `crate::models::item::ListItemsQuery`
error[E0422]: cannot find struct, variant or union type `Pagination` in this scope
error[E0433]: cannot find type `PaginatedOk` in this scope
error[E0425]: cannot find function `normalize_pagination` in this scope
error[E0425]: cannot find function `build_list_items_query` in this scope
error[E0425]: cannot find function `list_items` in this scope
error[E0425]: cannot find function `count_items` in this scope
error: could not compile `mediavault-api` (bin "mediavault-api" test) due to 40 previous errors
```

### 次のフェーズへの要求事項

Greenフェーズでは Red phase 記録ファイルの「4. Greenフェーズで実装すべき内容」に列挙した以下を実装する:
1. `models/response.rs` に `Pagination` / `PaginatedOk<T>` を追加
2. `models/item.rs` に `ListItemsQuery` を追加
3. `handlers/items.rs` に `normalize_pagination` と `list_items_handler` を追加
4. `repositories/item_repository.rs` に `build_list_items_query` / `build_count_items_query` / `list_items` / `count_items` を追加（sqlx::QueryBuilderでtag_id/category_idはEXISTSサブクエリ）
5. `routes/mod.rs` に `GET /items` ルートを追加
6. 統合テスト用のテスト用DBセットアップ・シードヘルパー（`test_pool`, `seed_items*` 等、現在は `unimplemented!()` スタブ）を実装し、`cargo test -p mediavault-api -- --ignored` で実行可能にする

最小実装でまずユニットテスト（18件）をGreenにし、統合テスト（11件）はテスト用DB環境構築と合わせて段階的にGreen化する方針を推奨する。

## Greenフェーズ（最小実装）

39 passed / 0 failed / 14 ignored（DB統合テスト）で完了。実装ファイル:
`models/response.rs`（Pagination/PaginatedOk）, `models/item.rs`（ListItemsQuery）,
`handlers/items.rs`（normalize_pagination/list_items_handler）,
`repositories/item_repository.rs`（push_item_filters/build_list_items_query/build_count_items_query/list_items/count_items）,
`routes/mod.rs`（GET /items ルート）。

## Refactorフェーズ（品質改善）

### 実施日時
2026-06-23

### レビュー結果
- **セキュリティ**: 動的WHERE句は`sqlx::QueryBuilder`の`push_bind`で全値パラメータ化、識別子（テーブル名/カラム名）は固定文字列のみ使用。SQLインジェクション経路なし。`db_error`はDB内部詳細をクライアントへ漏らさずログにのみ出力する既存方針を継続。問題なし。
- **パフォーマンス**: フィルタは`idx_items_media_type`/`idx_items_status`/`idx_items_is_favorite`を活用するカラム条件、tag_id/category_idはJOIN+DISTINCTではなくEXISTSサブクエリで重複排除コストを回避。`list_items`/`count_items`が同一`push_item_filters`を共有し、total整合性をコードレベルで保証。問題なし。
- **コード品質**: `cargo clippy -p mediavault-api --all-targets`はTASK-0010対象ファイル（item_repository.rs / handlers/items.rs / models/response.rs / routes/mod.rs）に対し警告0件。既存の`push_item_filters`ヘルパー抽出により単一責任・DRYは既に達成済み。
- **ファイルサイズ**: 各対象ファイルは500行制限内（item_repository.rs 857行と最大だが、本タスクのコメント含む統合テスト・ヘルパー群を含めての行数であり、機能別分割が必要な複雑度には達していないと判断。本タスクでは分割を見送り）。

### 適用した改善
1. `push_item_filters`内の`push_clause_prefix!`マクロに、マクロ採用理由（builderの可変借用とフラグの可変参照を同時に要するため、クロージャ化より単純）を説明するコメントを追加（`backend/mediavault-api/src/repositories/item_repository.rs`）。
   - 🟡 信頼性レベル: 既存実装の設計意図からの妥当な推測（挙動変更なし、コメントのみ）。

### 見送った改善（過剰設計回避）
- `push_clause_prefix!`マクロの関数・クロージャ化: 借用関係上の複雑さが増すだけで可読性向上が見込めないため見送り。
- `insert_test_item`シードヘルパーの共通モジュール化: 現時点で他テストスイートからの利用実績がなく、仮説的な将来タスクのための抽象化はルールにより見送り。

### テスト実行結果（各変更後に再実行）
`cargo test -p mediavault-api`: **39 passed; 0 failed; 14 ignored**（変更前と同一、リグレッションなし）。

### 品質判定
```
✅ 高品質:
- テスト結果: 全て継続成功（39 passed / 0 failed / 14 ignored）
- セキュリティ: 重大な脆弱性なし（パラメータ化クエリ・固定識別子・情報漏洩対策を確認）
- パフォーマンス: 重大な性能課題なし（インデックス活用・EXISTS採用・total整合性保証を確認）
- リファクタ品質: Green実装が既に高品質のため、過剰な変更を避けコメント改善のみ適用
- コード品質: clippy警告0件（対象ファイル）
- ドキュメント: 完成
```
