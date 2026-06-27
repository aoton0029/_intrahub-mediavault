# TASK-0031 Steamライブラリインポート機能 TDD Greenフェーズ

**機能名**: Steamライブラリインポート機能（`POST /import/steam`）
**タスクID**: TASK-0031
**要件名**: mediavault-backend
**作成日**: 2026-06-27

---

## 1. 実装方針

Redフェーズで作成された`todo!()`を、確定済み設計判断（note.md 8章・red-phase.md 2章）に従って実装した。

### 実装した関数

1. **`validate_steam_id`**（`import/steam_import.rs`）
   - `steam_id.len() == 17 && 全文字が数値`を検証 → 不正時は`ApiErrorCode::ValidationError`（400）
   - 検証通過後は`parse::<u64>()`でu64へ変換
   - 🔵 信頼性レベル: note.md「SteamID64 形式検証」に直接対応

2. **`SteamCredentialLookup::find`**（新規追加メソッド、`import/steam_import.rs`）
   - `Pool(pool)`: `api_credential_repository::find_by_provider(pool, ApiProvider::Steam)`を呼び`api_key`を取り出す
   - `Fixed(resolver)`: 固定クロージャを即時呼び出す（DBアクセスなし）
   - 🟡 信頼性レベル: TASK-0023 `ApiCredentialLookup::find_by_provider`パターンの踏襲

3. **`steam_game_entry_to_create_item_request`**（`import/steam_import.rs`）
   - `media_type: MediaType::Game`固定
   - `title`: `name.unwrap_or_else(|| format!("Unknown (appid:{appid})"))`（確定方針#2）
   - 他フィールドは全て`None`（CreateItemRequestのデフォルト相当）

4. **`import_steam_library`**（`import/steam_import.rs`）
   - `validate_steam_id`で早期検証（外部API呼び出し前）
   - `SteamCredentialLookup::find()`でAPIキー取得、`None`なら401 `SteamApiKeyInvalid`
   - `SteamClient`を構築（`steam_api_base_url`指定時は`new_with_base_urls`、本番は`new`）
   - `get_owned_games`呼び出し。エラーは`map_steam_api_error`でマッピング
   - 各`SteamGameEntry`について`item_repository::find_existing_import`で重複チェック
     → 存在すればスキップ（集計外、確定方針#1）
     → 存在しなければ`steam_game_entry_to_create_item_request`で変換し`create_item_with_source`で登録
   - 成功時`summary.record_success()`、DB起因失敗時は配列インデックスを`row_number`として`record_failure`（確定方針#3）

5. **`map_steam_api_error`**（新規追加ヘルパー、`import/steam_import.rs`）
   - `Http{401|403}` / `Auth(_)` → `SteamApiKeyInvalid`（401）
   - その他（Timeout/Network/RateLimit/Parse/Http(その他)）→ `ExternalApiTimeout`（502）
   - 🟡 信頼性レベル: TASK-0018/0019パターン再利用からの妥当な推測

6. **`import_steam_handler`**（`handlers/import_steam.rs`）
   - `SteamCredentialLookup::Pool(state.db.clone())`を構築
   - `import_steam_library(&state.db, &credentials, None, &request.steam_id)`へ委譲
   - 成功時`Ok(Json(ApiOk::new(summary)).into_response())`

### Red期待からの軽微な修正（テストヘルパー）

Redフェーズの`unreachable_pool()`ヘルパーは`PgPool::connect(...).await`（即時接続）を使用していたが、
到達不能ホスト（`127.0.0.1:1`）への接続試行がこの開発環境では即座に拒否されず、デフォルトの
sqlxコネクトタイムアウト（30秒）まで待たされてしまい、DB非依存のはずの5件のユニットテストが
`PoolTimedOut`でpanicしていた（実装ロジック自体は正しく早期returnしており、プールへ到達する前に
完了する設計だったが、ヘルパー関数のプール構築自体がブロックしていた）。

`PgPool::connect_lazy`（非同期化不要・実際にクエリが発行されるまで接続を試行しない）へ変更することで、
意味的に同じ「到達不能プール」を表現しつつ、テストが高速に完了するよう修正した。テストの期待値（assert）
は一切変更していない。

```rust
// 変更前（Red）
async fn unreachable_pool() -> PgPool {
    PgPool::connect("postgres://invalid:invalid@127.0.0.1:1/invalid")
        .await
        .expect("到達不能プールの構築検証用接続に失敗しました")
}

// 変更後（Green）
fn unreachable_pool() -> PgPool {
    PgPool::connect_lazy("postgres://invalid:invalid@127.0.0.1:1/invalid")
        .expect("到達不能プールの構築（lazy）に失敗しました")
}
```

呼び出し側の`unreachable_pool().await`は`unreachable_pool()`（同期呼び出し）へ統一した。

---

## 2. テスト実行結果

### ビルド

```
cargo build -p mediavault-api
```
→ **成功**（エラーなし、warningなし）

### テスト実行（DB非依存）

```
cargo test -p mediavault-api steam
```

結果:
- **15 passed**: 全DB非依存ユニットテストが成功
  - `validate_steam_id_*`（6件）
  - `steam_game_entry_conversion_*`（3件）
  - `import_steam_library_returns_401_when_api_key_not_configured`
  - `import_steam_library_returns_401_when_steam_api_rejects_key`
  - `import_steam_library_returns_empty_summary_for_empty_game_list`
  - `import_steam_library_returns_502_when_steam_api_is_unreachable`
  - `import_steam_library_returns_400_for_invalid_steam_id_before_calling_steam_api`
  - `steam_api_key_invalid_returns_401_with_expected_wire_code`
- **6 ignored**: 実DB必要な統合テスト（Docker未起動のため未実行、想定通り）
  - `import_steam_library_registers_all_games_and_reports_success_count`
  - `import_steam_library_continues_processing_after_one_game_registration_fails`
  - `import_steam_library_skips_duplicate_appid_without_counting_as_failure`
  - `import_steam_with_empty_steam_id_returns_400`
  - `import_steam_without_configured_api_key_returns_401`
  - `import_steam_with_valid_key_and_games_returns_200`
- **0 failed**

### 全体テスト（`cargo test -p mediavault-api`）

196 passed / 2 failed（`services::file_storage`、env変数の並行テスト干渉による既存の不安定テスト。
単独実行では成功するため、TASK-0031の実装とは無関係。pre-existing issue） / 192 ignored

---

## 3. 品質判定

```
✅ 高品質:
- テスト結果: Steam関連15件のDB非依存テストが全て成功（cargo build/testエラーなし）
- 実装品質: シンプルかつ動作する。既存パターン（external_search.rs/import_booklog.rs）を踏襲
- リファクタ箇所: map_steam_api_errorのエラー分類の網羅性確認、import_steam_library関数の
  行数（やや長い）をRefactorフェーズで分割検討
- 機能的問題: なし
- コンパイルエラー: なし
- ファイルサイズ: steam_import.rs 約610行（800行以内）
- モック使用: 実装コード（todo!()を置き換えた本体）にモック・スタブは含まれていない
  （wiremockはテストコード内のみ）
```

---

## 4. 次のステップ

次のお勧めステップ: `/tsumiki:tdd-refactor mediavault-backend TASK-0031` でRefactorフェーズ（品質改善）を開始します。
