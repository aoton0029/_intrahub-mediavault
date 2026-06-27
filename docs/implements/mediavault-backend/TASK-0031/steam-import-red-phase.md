# TASK-0031 Steamライブラリインポート機能 TDD Redフェーズ

**機能名**: Steamライブラリインポート機能（`POST /import/steam`）
**タスクID**: TASK-0031
**要件名**: mediavault-backend
**作成日**: 2026-06-27

---

## 1. 作成したテストケース一覧

### 新規作成ファイル

- `backend/mediavault-api/src/import/steam_import.rs`（usecase層・新規）
  - `validate_steam_id`（未実装・`todo!()`）
  - `SteamCredentialLookup`（DI用enum、TASK-0023 `ApiCredentialLookup`パターン踏襲）
  - `import_steam_library`（未実装・`todo!()`）
  - `steam_game_entry_to_create_item_request`（未実装・`todo!()`）
- `backend/mediavault-api/src/handlers/import_steam.rs`（ハンドラ層・新規）
  - `SteamImportRequest` DTO
  - `import_steam_handler`（未実装・`todo!()`）

### 既存拡張ファイル

- `backend/mediavault-api/src/models/response.rs`
  - `ApiErrorCode::SteamApiKeyInvalid`追加（401 `STEAM_API_KEY_INVALID`）。Greenを兼ねて実装済み（共通基盤のため）
- `backend/mediavault-api/src/import/mod.rs`
  - `pub mod steam_import;` 追加
- `backend/mediavault-api/src/handlers/mod.rs`
  - `pub mod import_steam;` 追加
- `backend/mediavault-api/src/routes/mod.rs`
  - `.route("/import/steam", axum::routing::post(import_steam_handler))` 追加

### テストケース一覧（17件）

#### steam_id 形式検証ユニットテスト（DB非依存・6件）

| テストID | テスト関数名 | 対応TC | 信頼性 |
|---|---|---|---|
| 1 | `validate_steam_id_accepts_17_digit_numeric_string` | TC-017-B01 | 🔵 |
| 2 | `validate_steam_id_rejects_empty_string` | TC-017-E03-a | 🟡 |
| 3 | `validate_steam_id_rejects_non_numeric_characters` | TC-017-E03-b | 🟡 |
| 4 | `validate_steam_id_rejects_16_digit_string` | TC-017-E03-c | 🟡 |
| 5 | `validate_steam_id_rejects_18_digit_string` | TC-017-E03-d | 🟡 |
| 6 | `validate_steam_id_handles_17_digit_max_value_without_overflow` | TC-017-B05 | 🟡 |

#### SteamGameEntry→CreateItemRequest変換ユニットテスト（DB非依存・3件）

| テストID | テスト関数名 | 対応TC | 信頼性 |
|---|---|---|---|
| 7 | `steam_game_entry_conversion_sets_media_type_game_and_title` | TC-017-N03 | 🔵 |
| 8 | `steam_game_entry_conversion_uses_fallback_title_when_name_is_none` | TC-017-B02 | 🟡 |
| 9 | `steam_game_entry_conversion_preserves_title_for_different_appid` | TC-017-N03対称 | 🔵 |

#### import_steam_library ユニット・統合テスト（8件）

| テストID | テスト関数名 | 対応TC | DB依存 | 信頼性 |
|---|---|---|---|---|
| 10 | `import_steam_library_returns_401_when_api_key_not_configured` | TC-017-E01-A | 非依存 | 🔵 |
| 11 | `import_steam_library_returns_401_when_steam_api_rejects_key` | TC-017-E01-B | 非依存 | 🔵 |
| 12 | `import_steam_library_returns_empty_summary_for_empty_game_list` | TC-017-02 | 非依存 | 🟡 |
| 13 | `import_steam_library_registers_all_games_and_reports_success_count` | TC-017-01 | 依存（`#[ignore]`） | 🔵 |
| 14 | `import_steam_library_continues_processing_after_one_game_registration_fails` | TC-017-E02 | 依存（`#[ignore]`） | 🟡 |
| 15 | `import_steam_library_skips_duplicate_appid_without_counting_as_failure` | TC-017-E05 | 依存（`#[ignore]`） | 🟡 |
| 16 | `import_steam_library_returns_502_when_steam_api_is_unreachable` | TC-017-E04 | 非依存 | 🟡 |
| 17 | `import_steam_library_returns_400_for_invalid_steam_id_before_calling_steam_api` | TC-017-E03 | 非依存 | 🟡 |

#### ハンドラ統合テスト（3件、いずれも`#[ignore]`、実DB必要）

| テストID | テスト関数名 | 対応TC | 信頼性 |
|---|---|---|---|
| 18 | `import_steam_with_empty_steam_id_returns_400` | TC-017-E03 | 🟡 |
| 19 | `import_steam_without_configured_api_key_returns_401` | TC-017-E01 | 🔵 |
| 20 | `import_steam_with_valid_key_and_games_returns_200` | TC-017-01 | 🔵 |

#### ApiErrorCode単体テスト（1件、即時グリーン）

| テストID | テスト関数名 | 対応TC | 信頼性 |
|---|---|---|---|
| 21 | `steam_api_key_invalid_returns_401_with_expected_wire_code` | TC-017-E01-CODE | 🔵 |

合計: 21件（テストケース追加目標数10件を超過）

---

## 2. 設計判断の確定事項（未確定事項#1〜#5の解消）

note.md「8. テストケース定義フェーズの確定要事項」・テストケース定義書「6. 実装時に確定が必要な未確定事項」を以下の方針で確定し、テスト期待値に反映した。

1. **重複steam_appidの集計方針**: **スキップ＝集計外**（success/failureどちらにも数えない）に確定。`import_steam_library_skips_duplicate_appid_without_counting_as_failure`で検証。
2. **name=Noneエントリの扱い**: **フォールバックタイトルで登録（success）** に確定。タイトルは`"Unknown (appid:{appid})"`形式。`steam_game_entry_conversion_uses_fallback_title_when_name_is_none`で検証。
3. **ImportFailureの識別子**: ユーザー指示通り、既存`ImportFailure.row_number: u32`を**配列インデックス（0始まり）として再利用**する方針に確定（型拡張は行わない）。
4. **ImportSummaryカウント型**: 既存実装の`u32`をそのまま使用（要件定義書の`usize`表記は採用しない）。
5. **トランザクション分離単位**: ゲーム1件ごとに独立した`create_item_with_source`呼び出しとし、1件の失敗が他に影響しない方式に確定（EDGE-002対応）。

---

## 3. ビルド・テスト実行結果

### ビルド

```
cargo build -p mediavault-api
```
→ **成功**（警告のみ、エラーなし）。`todo!()`を含む関数は型シグネチャが確定しているためコンパイルは通る。

### テスト実行

```
cargo test -p mediavault-api steam
```

結果:
- **1 passed**: `steam_api_key_invalid_returns_401_with_expected_wire_code`（ApiErrorCode単体、共通基盤のため実装済み）
- **14 failed**（期待通りのRed失敗）:
  - `validate_steam_id_*`（6件）: `not yet implemented: Greenフェーズで実装する: 17桁数値文字列検証 + u64変換`でpanic
  - `steam_game_entry_conversion_*`（3件）: `not yet implemented: ...CreateItemRequest変換`でpanic
  - `import_steam_library_*`（DB非依存5件）: `validate_steam_id`/`import_steam_library`内の`todo!()`、または`unreachable_pool()`の接続タイムアウト（`PoolTimedOut`）でpanic
- **6 ignored**: `#[ignore]`付き実DB統合テスト（DB依存テスト3件・ハンドラ統合テスト3件）は意図的に未実行

すべて**Greenフェーズでの実装待ち**を示す失敗であり、想定通りのRed状態。

---

## 4. Greenフェーズで実装すべき内容

1. **`validate_steam_id`**（`import/steam_import.rs`）
   - 17桁・全数字チェック → 不正時は`ApiError::new(ApiErrorCode::ValidationError, ...)`
   - 検証通過後は`steam_id.parse::<u64>()`でu64へ変換（17桁全域でオーバーフローしないことをTC-017-B05で保証）

2. **`steam_game_entry_to_create_item_request`**（`import/steam_import.rs`）
   - `media_type: MediaType::Game`固定
   - `title`: `name.unwrap_or_else(|| format!("Unknown (appid:{appid})"))`
   - その他フィールドは`CreateItemRequest`のデフォルト相当（`None`）

3. **`import_steam_library`**（`import/steam_import.rs`）
   - `validate_steam_id`で早期検証（外部API呼び出し前）
   - `SteamCredentialLookup`経由でAPIキー取得、`None`なら`ApiErrorCode::SteamApiKeyInvalid`（401）
   - `SteamClient::new_with_base_urls`または`new`でクライアント構築（テスト時は`steam_api_base_url`引数でURL差し替え）
   - `get_owned_games`呼び出し。Steam側401/403相当エラーは`ApiErrorCode::SteamApiKeyInvalid`へマッピング、タイムアウト/接続不能は`ApiErrorCode::ExternalApiTimeout`（502）へマッピング
   - 各`SteamGameEntry`について:
     - 既存`(media_type=Game, source=Api, external_id=appid.to_string())`の重複チェック（事前SELECT）→ 存在する場合はスキップ（集計外、インデックスを進めるのみ）
     - 新規の場合は`steam_game_entry_to_create_item_request`で変換し`create_item_with_source`で登録
     - 成功時`summary.record_success()`、DB起因失敗時は`summary.record_failure(ImportFailure::new(index as u32, "db error"))`（indexは配列インデックス、設計判断#1反映）

4. **`import_steam_handler`**（`handlers/import_steam.rs`）
   - `SteamImportRequest`から`steam_id`を取得し`import_steam_library`へ委譲
   - `?`演算子で`ApiError`を自動伝播
   - 成功時`Ok(Json(ApiOk::new(summary)).into_response())`

5. **テスト専用ベースURL注入経路の確定**
   - `import_steam_library`に`steam_api_base_url: Option<&str>`引数を追加済み（テスト用）。本番経路（ハンドラ）は`None`を渡し`SteamClient::new`を使う。
   - ハンドラ統合テスト（TC-017-01相当）でwiremockサーバーへ到達させる場合、`AppState`等を経由したテスト専用注入手段の確定が必要（Greenフェーズで`ExternalSearchService`の`with_test_base_urls`パターンを参考に設計する）。

---

## 5. 次のステップ

次のお勧めステップ: `/tsumiki:tdd-green mediavault-backend TASK-0031` でGreenフェーズ（最小実装）を開始します。
