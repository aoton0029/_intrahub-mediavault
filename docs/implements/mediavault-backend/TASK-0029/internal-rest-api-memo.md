# TDD開発メモ: internal-rest-api（TASK-0029）

## 概要

- 機能名: internal-rest-api（内部REST APIルート群 /internal/items等）
- 開発開始: 2026-06-26
- 現在のフェーズ: Red

## 関連ファイル

- 元タスクファイル: `docs/tasks/mediavault-backend/TASK-0029.md`
- 要件定義: `docs/implements/mediavault-backend/TASK-0029/internal-rest-api-requirements.md`
- テストケース定義: `docs/implements/mediavault-backend/TASK-0029/internal-rest-api-testcases.md`
- 実装ファイル（未実装・Greenフェーズ対象）: `backend/mediavault-api/src/routes/internal.rs`（`build_internal_router` 関数）
- ルーター統合ファイル（未変更・Greenフェーズ対象）: `backend/mediavault-api/src/routes/mod.rs`（`build_router()` 内で `.merge()`）
- テストファイル: `backend/mediavault-api/src/routes/internal.rs`（インライン `#[cfg(test)] mod tests`）

## Redフェーズ（失敗するテスト作成）

### 作成日時

2026-06-26

### テストケース

テストケース定義書（`internal-rest-api-testcases.md`）の20件すべてに対応する21個のテスト関数を作成した（TC-018-B04のみpage=0補正/page=abc拒否の2系統に分割）。

- 正常系8件: POST/PATCH /internal/items, GET /internal/items/search（条件付き・未指定）, groups→episodes連鎖, upsert挙動, ファイル登録, 登録→検索反映
- 異常系8件: 認証ヘッダーなし401, 誤キー401, 全ルート401網羅, 各種404（item不在・group不在）, media_type不正400
- 境界値5関数（4テストケース）: バージョンプレフィックス誤り404, search誤マッチ防止, limitクランプ, page補正/拒否

### テストコード

`backend/mediavault-api/src/routes/internal.rs` の `#[cfg(test)] mod tests` を参照（全21関数、各関数に日本語コメント【テスト目的】【テスト内容】【期待される動作】【確認内容】および信頼性レベル🔵🟡を付与済み）。

すべて `#[tokio::test]` + `#[ignore]`（実DB必要、`cargo test -p mediavault-api -- --ignored` で実行）。

主要な共通ヘルパー:
- `test_app_state()`: DATABASE_URL環境変数からPgPool接続を構築（既存 `routes/mod.rs` パターンを再利用）
- `set_internal_api_key(key)`: `unsafe { std::env::set_var("INTERNAL_API_KEY", key) }` でミドルウェア照合元をセット
- `valid_create_item_body()`: 最小有効な `CreateItemRequest` JSON文字列

### 期待される失敗

`build_internal_router` 関数が未実装のため、テストモジュールの `use crate::routes::internal::build_internal_router;` でコンパイルエラーになる。

```
error[E0432]: unresolved import `crate::routes::internal::build_internal_router`
  --> mediavault-api\src\routes\internal.rs:24:9
```

`cargo test -p mediavault-api routes::internal` で上記エラーを確認済み（Red成功）。

### 次のフェーズへの要求事項

Greenフェーズでは以下を実装する:

1. `routes/internal.rs` に `pub fn build_internal_router(state: AppState) -> Router` を実装。
   - `/internal/items`（POST: create_item_handler）
   - `/internal/items/:id`（PATCH: update_item_handler）
   - `/internal/items/search`（GET: list_items_handler相当。`:id`より前にリテラル登録し誤マッチ防止）
   - `/internal/items/:id/groups`（POST: create_item_group_handler、upsert）
   - `/internal/groups/:group_id/episodes`（POST: create_item_episode_handler、upsert）
   - `/internal/items/:id/files`（POST: create_item_file_handler、パス指定方式）
   - `.layer(axum::middleware::from_fn(api_key_auth))` を適用（環境変数照合のため `from_fn`、stateは不要）
2. `routes/mod.rs` の `build_router()` で `.merge(internal::build_internal_router(state.clone()))` 統合。`/api/v1`系とは別にバージョンプレフィックスなしでマージすること。
3. 既存ハンドラ（Phase2/Phase4実装）をそのまま再利用し、新規ロジックは最小限に留める（選択肢A方針）。
4. 実装後、`cargo build -p mediavault-api` が通り、`cargo test -p mediavault-api -- --ignored`（実DB起動済み）で全21テストがpassすることを確認する。

## Greenフェーズ（最小実装）

（未着手）

## Refactorフェーズ（品質改善）

（未着手）
