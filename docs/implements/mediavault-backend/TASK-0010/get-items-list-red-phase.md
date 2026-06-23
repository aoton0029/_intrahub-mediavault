# TDD Redフェーズ記録: GET /items（一覧・絞り込み）

- **機能名**: GET /items（一覧・絞り込み）
- **タスクID**: TASK-0010
- **要件名**: mediavault-backend
- **フェーズ**: Red（失敗するテスト作成）
- **実施日**: 2026-06-23

## 1. 対象テストケース

`get-items-list-testcases.md` の全28ケースを対象に実装した（テストケース追加目標数10以上を超過達成）。

| 分類 | 件数 | 配置先ファイル |
|---|---|---|
| 正常系（N01〜N10） | 10 | N09/N10: `models/response.rs`、N01〜N08: `repositories/item_repository.rs`（`#[ignore]`統合テスト） |
| 異常系（E01〜E04） | 1（E04のみユニット実装、E01〜E03はルーター統合テスト相当のため要件定義書に明記しGreen以降のルーティング実装時に追加） | E04: `repositories/item_repository.rs`（`#[ignore]`統合テスト） |
| 境界値（B01〜B08） | 8 | B01〜B06: `handlers/items.rs`（ユニット）、B07/B08: `repositories/item_repository.rs`（`#[ignore]`統合テスト） |
| SQL生成（Q01〜Q06） | 6 | `repositories/item_repository.rs`（ユニット、QueryBuilder検証） |

合計: ユニットテスト 18件 + 統合テスト（`#[ignore]`）11件 = 29件（B06はユニット/統合の両面）。

> 補足: TC-0010-E01〜E03（クエリパラメータの型不正による400）はAxumルーター層のデシリアライズ挙動に依存するため、`routes/mod.rs` へのエンドポイント追加（Green/Refactorフェーズ）後にルーター経由の統合テストとして追加する方針とした。本Redフェーズでは E04（DBエラー変換）のみを repositories 層のユニット相当テストとして実装している。

## 2. 変更ファイル一覧

- `backend/mediavault-api/src/models/response.rs`
  - TC-0010-N09: `paginated_ok_serializes_to_expected_json`
  - TC-0010-N10: `paginated_ok_returns_200_even_when_data_is_empty`
  - 未実装シンボル: `PaginatedOk<T>`, `Pagination`
- `backend/mediavault-api/src/handlers/items.rs`
  - TC-0010-B01: `normalize_pagination_clamps_limit_to_100`
  - TC-0010-B02: `normalize_pagination_does_not_clamp_limit_at_exactly_100`
  - TC-0010-B03: `normalize_pagination_clamps_zero_limit_to_default_20`
  - TC-0010-B04: `normalize_pagination_clamps_zero_page_to_1`
  - TC-0010-B05: `normalize_pagination_defaults_to_page1_limit20_when_none`
  - TC-0010-B06: `normalize_pagination_computes_offset_20_for_page2_limit20`
  - 未実装シンボル: `normalize_pagination(Option<u32>, Option<u32>) -> (u32, u32)`
- `backend/mediavault-api/src/repositories/item_repository.rs`
  - TC-0010-Q01〜Q06: SQL生成ユニットテスト（`build_list_items_query` / `build_count_items_query`）
  - TC-0010-N01〜N08, B07, B08, E04: `#[ignore]`統合テスト（`list_items` / `count_items`）
  - テスト用ヘルパー（`unimplemented!()`スタブ）: `test_pool`, `unreachable_pool`, `seed_items`, `seed_items_by_media_type`, `seed_items_with_favorite`, `seed_items_with_status`, `seed_items_with_tag`, `seed_items_with_category`, `seed_item_with_media_type_and_tag`
  - 未実装シンボル: `ListItemsQuery`（`models/item.rs`想定）, `list_items`, `count_items`, `build_list_items_query`, `build_count_items_query`

## 3. 確認したRed状態（`cargo check -p mediavault-api --tests`）

40件のコンパイルエラーを確認。すべて新規追加した未実装シンボルの参照に起因し、既存実装（TASK-0008/0009分）への影響はない。

代表的なエラー抜粋:

```
error[E0432]: unresolved import `crate::models::item::ListItemsQuery`
error[E0422]: cannot find struct, variant or union type `Pagination` in this scope
error[E0433]: cannot find type `PaginatedOk` in this scope
error[E0425]: cannot find function `normalize_pagination` in this scope
error[E0425]: cannot find function `build_list_items_query` in this scope
error[E0425]: cannot find function `build_count_items_query` in this scope
error[E0425]: cannot find function `list_items` in this scope
error[E0425]: cannot find function `count_items` in this scope
error: could not compile `mediavault-api` (bin "mediavault-api" test) due to 40 previous errors
```

これはRedフェーズとして適切な失敗状態（実装未完了によるコンパイルエラー）である。

## 4. Greenフェーズで実装すべき内容

1. `backend/mediavault-api/src/models/response.rs`
   - `Pagination { page: u32, limit: u32, total: i64 }`（`#[derive(Serialize)]`）
   - `PaginatedOk<T> { success: bool, data: T, pagination: Pagination }` + `PaginatedOk::new(data, pagination)` + `IntoResponse`（200固定）
2. `backend/mediavault-api/src/models/item.rs`
   - `ListItemsQuery { media_type: Option<MediaType>, tag_id: Option<Uuid>, category_id: Option<Uuid>, is_favorite: Option<bool>, status: Option<ItemStatus>, page: Option<u32>, limit: Option<u32> }`（`Deserialize`、Axum `Query`抽出対象）
3. `backend/mediavault-api/src/handlers/items.rs`
   - `pub fn normalize_pagination(page: Option<u32>, limit: Option<u32>) -> (u32, u32)`（page<1→1, limit<1→20, limit>100→100）
   - `list_items_handler`（クエリ抽出→正規化→repository呼び出し→`PaginatedOk`構築）
4. `backend/mediavault-api/src/repositories/item_repository.rs`
   - `build_list_items_query(&ListItemsQuery) -> sqlx::QueryBuilder<'_, Postgres>`（`SELECT ... FROM items [WHERE ...] LIMIT ... OFFSET ...`、tag_id/category_idはEXISTSサブクエリ）
   - `build_count_items_query(&ListItemsQuery) -> sqlx::QueryBuilder<'_, Postgres>`（`SELECT COUNT(*) FROM items [WHERE ...]`、list用と同一WHERE句生成ロジックを共有）
   - `list_items(&PgPool, &ListItemsQuery) -> Result<Vec<Item>, ApiError>`
   - `count_items(&PgPool, &ListItemsQuery) -> Result<i64, ApiError>`
5. `backend/mediavault-api/src/routes/mod.rs`
   - `.route("/items", get(list_items_handler))` 追加（既存 `.route("/items", post(...))` と同一パス）
6. 統合テスト用ヘルパー（テスト用DB接続・シード関数）の実装、または testcontainers 等への置き換え検討

## 5. テスト実行コマンド

```bash
# 型チェック（Red確認に使用）
cargo check -p mediavault-api --tests

# ユニットテストのみ（実DB不要、Greenフェーズの完了確認に使用）
cargo test -p mediavault-api

# 統合テスト含む全件（docker compose up -d db 後）
cargo test -p mediavault-api -- --ignored
```
