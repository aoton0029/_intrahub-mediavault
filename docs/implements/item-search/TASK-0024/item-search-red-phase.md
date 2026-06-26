# TASK-0024 Redフェーズ記録: GET /items/search 外部API検索エンドポイント

**作成日**: 2026-06-26
**対象タスク**: TASK-0024
**対象テストケース**: TC-0024-N01, N04, E01, E02, E03, E04, E05, E06, B02, B03（計10件 + 集合外境界1件）

---

## 1. 作成したテストケース一覧

| ID | 配置ファイル | テスト層 | 概要 | 期待結果 |
|---|---|---|---|---|
| TC-0024-N04 | `backend/mediavault-api/src/models/item_search.rs` | ユニット | ItemSearchQuery デシリアライズ成功 | media_type=Anime, q="foo" |
| TC-0024-B03 | `backend/mediavault-api/src/models/item_search.rs` | ユニット | MediaType全8variant受理 + 集合外拒否 | 8受理 / 1拒否 |
| TC-0024-E01 | `backend/mediavault-api/src/models/response.rs` | ユニット | ApiKeyNotConfigured → 422変換 | 422 / API_KEY_NOT_CONFIGURED |
| TC-0024-E02 | `backend/mediavault-api/src/models/response.rs` | ユニット | ExternalApiError(Timeout) → 502変換 | 502 / EXTERNAL_API_TIMEOUT |
| TC-0024-E03 | `backend/mediavault-api/src/models/response.rs` | ユニット | ApiError 6variant全集約 → 502 | 502全件 |
| TC-0024-N01相当 | `backend/mediavault-api/src/handlers/items.rs` | ハンドラ統合(#[ignore]) | search_items_handler anime成功 | 200 |
| TC-0024-E01相当 | `backend/mediavault-api/src/handlers/items.rs` | ハンドラ統合(#[ignore]) | search_items_handler APIキー未設定 | 422 |
| TC-0024-E04 | `backend/mediavault-api/src/routes/mod.rs` | ルーター統合(#[ignore]) | q欠落 | 400 |
| TC-0024-E05 | `backend/mediavault-api/src/routes/mod.rs` | ルーター統合(#[ignore]) | media_type欠落 | 400 |
| TC-0024-E06 | `backend/mediavault-api/src/routes/mod.rs` | ルーター統合(#[ignore]) | media_type不正値 | 400 |
| TC-0024-B02 | `backend/mediavault-api/src/routes/mod.rs` | ルーター統合(#[ignore]) | ルート誤マッチ防止 | 非500到達 |

合計: ユニット5件（DB非依存・即時実行）+ ハンドラ/ルーター統合6件（`#[ignore]`、実DB前提）

---

## 2. 実装した/変更したファイル

- **新規**: `backend/mediavault-api/src/models/item_search.rs` — `ItemSearchQuery` DTO + 3ユニットテスト
- **追記**: `backend/mediavault-api/src/models/mod.rs` — `pub mod item_search;` 追加
- **追記**: `backend/mediavault-api/src/models/response.rs` —
  - `ApiErrorCode::ApiKeyNotConfigured`（422）/ `ApiErrorCode::ExternalApiTimeout`（502）新variant
  - `impl From<ExternalSearchError> for ApiError`
  - 3ユニットテスト（TC-0024-E01/E02/E03）
- **追記**: `backend/mediavault-api/src/handlers/items.rs` — `search_items_handler`（未実装・呼び出しのみ）への2統合テスト（`#[ignore]`）
- **追記**: `backend/mediavault-api/src/routes/mod.rs` —
  - `build_router` に `.route("/items/search", get(search_items_handler))` を `/items/:id` より前に追加
  - 4ルーター統合テスト（`#[ignore]`）

---

## 3. 期待される失敗（実行結果）

```
cargo test -p mediavault-api
```

```
error[E0432]: unresolved import `crate::handlers::items::search_items_handler`
  --> mediavault-api\src\routes\mod.rs:24:5
error[E0425]: cannot find function `search_items_handler` in this scope
   --> mediavault-api\src\handlers\items.rs:565
error[E0425]: cannot find function `search_items_handler` in this scope
   --> mediavault-api\src\handlers\items.rs:588
error: could not compile `mediavault-api` (bin "mediavault-api" test) due to 3 previous errors
```

- `cargo build -p mediavault-api` でも根本原因は同一の1エラー（`search_items_handler`未解決）のみであることを確認済み（他の新規コード — `ItemSearchQuery`、`ApiErrorCode`新variant、`From<ExternalSearchError>`実装 — からの不整合エラーは発生していない）。
- これはRedフェーズとして正しい状態: ハンドラ未実装によりコンパイル自体が失敗し、「まだ実装されていない機能」を明確に示している。

---

## 4. Greenフェーズで実装すべき内容

1. `backend/mediavault-api/src/handlers/items.rs` に `search_items_handler` を実装する。
   - `State(state): State<AppState>`, `Query(query): Query<ItemSearchQuery>` を受け取る
   - `ExternalSearchService::new(state.db.clone()).search(query.media_type, &query.q).await?` を呼ぶ
   - 成功時 `Ok(ApiOk::new(results))` を返す（型: `Result<ApiOk<Vec<ExternalSearchResult>>, ApiError>`）
   - `?` 演算子で `ExternalSearchError` → `ApiError`（今回実装した`From`変換）へ自動伝播させる
2. `backend/mediavault-api/src/routes/mod.rs` の import 文に `search_items_handler` を追加する（ルート登録は既にRedフェーズで完了済み）。
3. Green実装後、`#[ignore]` 付き統合テスト（ハンドラ4件・ルーター4件）を `docker compose up -d db` + wiremock環境で `cargo test -- --ignored` 実行し、TC-0024-N01〜N03・B01（E2E、wiremock）も追加実装する。
