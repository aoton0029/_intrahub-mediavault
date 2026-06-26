# TASK-0029: 内部REST APIルート群実装（/internal/items等） - Redフェーズ記録

- **機能名**: internal-rest-api（内部REST APIルート群）
- **タスクID**: TASK-0029
- **フェーズ**: Red（失敗するテスト作成）
- **作成日**: 2026-06-26

## 1. 作成したファイル

| ファイル | 内容 |
|---|---|
| `backend/mediavault-api/src/routes/internal.rs`（新規） | `/internal` 専用ルーターのスケルトン（`build_internal_router` は未実装）。インラインテストモジュール `#[cfg(test)] mod tests` に20件中19件のテストケースを実装。 |
| `backend/mediavault-api/src/routes/mod.rs`（変更） | `pub mod internal;` を追加し、新規モジュールをクレートに登録。 |

## 2. 作成したテストケース一覧（20件中19件実装。TC-018-B04は a/b に分割し2関数で実装）

### 正常系（8件）
- `post_internal_items_with_valid_key_returns_201`（TC-018-01）🔵
- `patch_internal_items_id_with_valid_key_updates_and_returns_200`（TC-018-02）🔵
- `get_internal_items_search_with_filters_returns_200_with_pagination`（TC-018-03）🔵
- `get_internal_items_search_without_query_returns_200_with_default_pagination`（TC-018-04）🟡
- `groups_then_episodes_upsert_chain_succeeds`（TC-018-05）🔵
- `posting_same_group_twice_upserts_instead_of_duplicating`（TC-018-06）🔵
- `post_internal_items_id_files_with_valid_path_returns_201`（TC-018-07）🔵
- `created_item_is_searchable_via_internal_search`（TC-018-08）🟡

### 異常系（8件）
- `post_internal_items_without_auth_header_returns_401`（TC-018-E01a）🔵
- `post_internal_items_with_wrong_key_returns_401`（TC-018-E01b）🔵
- `all_internal_routes_return_401_without_auth`（TC-018-E01c）🔵
- `patch_internal_items_id_with_nonexistent_id_returns_404`（TC-018-E02a）🔵
- `post_internal_items_id_groups_with_nonexistent_item_returns_404`（TC-018-E02b）🔵
- `post_internal_groups_group_id_episodes_with_nonexistent_group_returns_404`（TC-018-E02c）🔵
- `post_internal_items_id_files_with_nonexistent_item_returns_404`（TC-018-E02d）🔵
- `post_internal_items_with_invalid_media_type_returns_400`（TC-018-E03）🔵

### 境界値（4件、TC-018-B04を2関数化）
- `post_api_v1_internal_items_returns_404_not_found`（TC-018-B01）🔵
- `get_internal_items_search_does_not_fall_through_to_item_id_route`（TC-018-B02）🔵
- `get_internal_items_search_with_limit_over_100_clamps_to_100`（TC-018-B03）🟡
- `get_internal_items_search_with_page_zero_normalizes_to_1`（TC-018-B04a）🟡
- `get_internal_items_search_with_non_numeric_page_returns_400`（TC-018-B04b）🟡

合計21関数（テストケース定義書20件をすべてカバー。TC-018-B04のみ正常補正/異常拒否の2系統に分割）。

## 3. テスト実行結果（Red確認）

```
cargo test -p mediavault-api routes::internal
```

```
error[E0432]: unresolved import `crate::routes::internal::build_internal_router`
  --> mediavault-api\src\routes\internal.rs:24:9
   |
24 |     use crate::routes::internal::build_internal_router;
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `build_internal_router` in `routes::internal`

error: could not compile `mediavault-api` (bin "mediavault-api" test) due to 1 previous error
```

期待通りコンパイルエラーで失敗することを確認した（Redフェーズの目的を達成）。

## 4. Greenフェーズで実装すべき内容

1. `backend/mediavault-api/src/routes/internal.rs` に `pub fn build_internal_router(state: AppState) -> Router` を実装する。
   - `Router::new()` に以下をマウント:
     - `/internal/items`: POST（`create_item_handler`）, PATCH（経路は `/internal/items/:id`）
     - `/internal/items/:id`: PATCH（`update_item_handler`）
     - `/internal/items/search`: GET（`list_items_handler` または `search`相当のロジック再利用。要件上は既存`list_items_handler`の検索ロジック）
     - `/internal/items/:id/groups`: POST（`create_item_group_handler`）
     - `/internal/groups/:group_id/episodes`: POST（`create_item_episode_handler`）
     - `/internal/items/:id/files`: POST（`create_item_file_handler`）
   - `.layer(axum::middleware::from_fn(api_key_auth))` を適用する（`from_fn_with_state` ではなく `from_fn`。`api_key_auth` は環境変数を直接読むためstateを必要としない）。
   - `.with_state(state)` を最後に適用する。
2. `backend/mediavault-api/src/routes/mod.rs` の `build_router()` 内で `.merge(internal::build_internal_router(state.clone()))` のように統合する（`/internal` はバージョンプレフィックスを持たないため、`/api/v1` 系ルートと並列にマージすること。`.nest("/api/v1", ...)` 配下に入れないよう注意）。
3. `GET /internal/items/search` のクエリパラメータ未指定時の全件返却・limit上限100クランプ・page下限1補正は、既存 `list_items_handler` / `normalize_pagination` のロジックを再利用すれば自動的に満たされる想定。
4. 上記実装後、`cargo test -p mediavault-api -- --ignored` で本ファイルの全テストがpassすることを確認する（実DB必要、`docker compose up -d db` 起動済みであること）。

## 5. 注意事項

- `api_key_auth` ミドルウェアは `std::env::var("INTERNAL_API_KEY")` を照合元とするため、`AppState.internal_api_key` の値は本テストでは使用していない（`test_app_state()` で空文字を設定している）。
- 全テストは `#[ignore]` 付き（実DB必要）。`cargo test -p mediavault-api -- --ignored` で実行する。
- 現時点では `cargo build` 自体が失敗するため、`#[ignore]` テストの実行確認はGreenフェーズ実装後に行う。
