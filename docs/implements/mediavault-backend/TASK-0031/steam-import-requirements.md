# TASK-0031 Steamライブラリインポート機能 TDD要件定義書

**機能名**: Steamライブラリインポート機能（`POST /import/steam`）
**タスクID**: TASK-0031
**要件名**: mediavault-backend
**出力ファイル**: `docs/implements/mediavault-backend/TASK-0031/steam-import-requirements.md`
**作成日**: 2026-06-27

---

## 信頼性レベル凡例

- 🔵 **青信号**: EARS要件定義書・設計文書を参考にしてほぼ推測していない
- 🟡 **黄信号**: EARS要件定義書・設計文書から妥当な推測
- 🔴 **赤信号**: EARS要件定義書・設計文書にない推測

---

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

- 🔵 **何をする機能か**: `POST /import/steam` エンドポイントで、ユーザーが指定した `steam_id`（SteamID64）に紐づくSteamの所持ゲーム一覧を Steam Web API（`GetOwnedGames` 相当）経由で取得し、各ゲームを `items`（`media_type=game`, `source=api`）および `game_details`（`steam_appid` 等）として一括登録する機能。
- 🔵 **どのような問題を解決するか**: ユーザーが手作業で1本ずつゲームを登録する手間を排除し、Steamライブラリをワンアクションでメディアコレクションに取り込めるようにする（As a メディア管理ユーザー / So that 所持ゲームを手入力せずに一括登録できる）。
- 🔵 **想定されるユーザー**: 自身のSteamライブラリをMediaVaultで管理したいローカル利用者（As a ローカルメディア管理アプリ利用者）。
- 🔵 **システム内での位置づけ**: Phase 5「内部API・インポート」のインポート系エンドポイント群の一つ。TASK-0023で確立した `ExternalSearchService`/`api-client-lib` 連携パターンと、TASK-0009のアイテム作成ロジック、TASK-0030のインポート結果集約パターンを統合する最終タスク群の一つ。ハンドラ → usecase（import層）→ リポジトリ/外部APIクライアントの3層構成に従う。
- **参照したEARS要件**: REQ-017, EDGE-002
- **参照した設計文書**:
  - `docs/design/mediavault-backend/api-endpoints.md` 「POST /import/steam」セクション
  - `docs/design/mediavault-backend/architecture.md`（ハンドラ→usecase→リポジトリの3層構成）
  - `docs/tasks/mediavault-backend/TASK-0031.md`（タスク概要・実装詳細）

---

## 2. 入力・出力の仕様（EARS機能要件・型定義ベース）

### 入力（リクエスト）

- 🔵 **HTTPメソッド・パス**: `POST /import/steam`
- 🔵 **リクエストボディ**（JSON）:
  ```json
  { "steam_id": "76561198000000000" }
  ```
- 🔵 **Rust型**: `SteamImportRequest { steam_id: String }`
- 🔵 **入力制約**:
  - `steam_id` は SteamID64 形式（17桁の数値文字列）であること
  - 簡易検証: `steam_id.len() == 17 && steam_id.chars().all(|c| c.is_numeric())`
  - 🟡 より厳密には SteamID64 範囲 `(76561197960265728..=u64::MAX)` の検証も可能（note.md記載の推奨。MVPでは簡易検証で可）

### 外部API入力（Steam Web API呼び出し）

- 🔵 **クライアント**: `api-client-lib::clients::steam::SteamClient::get_owned_games`
- 🔵 **リクエスト型**:
  ```rust
  pub struct GetOwnedGamesRequest {
      pub steam_id: u64,                   // SteamID64
      pub include_appinfo: bool,           // ゲーム名・画像情報を含める
      pub include_played_free_games: bool, // 無料ゲームを含める
  }
  ```
- 🔵 **APIキー**: DB（`api_credentials` テーブル）に保存された `ApiProvider::Steam` のキーを `api_credential_repository::find_by_provider` で取得して使用（REQ-015関連）。

### 外部APIからの中間出力（Steamレスポンス）

- 🔵 **レスポンス型**:
  ```rust
  pub struct SteamOwnedGamesModel {
      pub game_count: u32,
      pub games: Vec<SteamGameEntry>,
  }
  pub struct SteamGameEntry {
      pub appid: u32,
      pub name: Option<String>,
      pub playtime_forever: u32,   // プレイ時間（分）
      pub img_icon_url: Option<String>,
  }
  ```

### 出力（レスポンス）

- 🔵 **成功時HTTPステータス**: `200 OK`
- 🔵 **レスポンス型**（TASK-0030と共通の `models/import.rs`）:
  ```rust
  pub struct ImportSummary {
      pub success_count: usize,
      pub failure_count: usize,
      pub failures: Vec<ImportFailure>,
  }
  ```
- 🔵 **共通レスポンス包装**: `{ "success": true, "data": { ImportSummary } }`（note.md レスポンス形式規約）
- 🔵 **エラー時形式**: `{ "success": false, "error": { "code": "...", "message": "..." } }`

### 入出力の関係性・データフロー

- 🔵 リクエストの `steam_id`（String）→ 検証 → `u64` 変換 → `GetOwnedGamesRequest` 構築 → Steam API 呼び出し → `Vec<SteamGameEntry>` 取得。
- 🔵 各 `SteamGameEntry` → `CreateItemRequest`（`media_type=game`, `source=api`, `external_id=appid`）へ変換 → `item_repository::create_item_with_source` で `items`+`game_details` 登録。
- 🔵 登録結果を集計し `ImportSummary` として返却（成功件数・失敗件数・失敗詳細）。
- **参照したEARS要件**: REQ-017
- **参照した設計文書**:
  - `docs/design/mediavault-backend/api-endpoints.md`「POST /import/steam」リクエスト例
  - `backend/api-client-lib/src/clients/steam/models.rs`（`GetOwnedGamesRequest`/`SteamOwnedGamesModel`/`SteamGameEntry`）
  - `backend/mediavault-api/src/models/import.rs`（`ImportSummary`/`ImportFailure`）
  - `backend/mediavault-api/src/repositories/item_repository.rs`（`create_item_with_source`）

---

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

### パフォーマンス要件

- 🔵 複数ゲーム登録は1回のトランザクション処理を基本とし、個別INSERTの繰り返しを避ける（note.md バッチ最適化／`INSERT ... ON CONFLICT ... DO NOTHING` パターン、TASK-0030参考）。
- 🟡 Steam Web APIのレート制限・タイムアウト時は、TASK-0018/0019で確立済みの `EXTERNAL_API_TIMEOUT`（502相当）パターンを再利用する。

### セキュリティ要件

- 🔵 Steam APIキーは環境変数ではなくDB（`api_credentials` テーブル）に保存されたものを使用する（REQ-015関連）。
- 🔵 キーが未設定または無効（Steam側が401/403相当を返す）の場合は `STEAM_API_KEY_INVALID`（401）で安全に返却し、キー内容はレスポンスに含めない。
- 🔵 入力検証で `steam_id` の形式・範囲をチェックし、不正値は `400 VALIDATION_ERROR` で拒否する。

### 互換性要件（MUST）

- 🔵 レスポンス形式は他インポート（TASK-0030 ブクログCSV）と同一の `ImportSummary` を再利用し、形式の一貫性を保つ（MUST）。
- 🔵 エラーレスポンス形式は既存の `ApiError` 統一形式に従う（MUST）。

### アーキテクチャ制約

- 🔵 ハンドラ（Axum: `State`, `Json` エクストラクタ）→ usecase（`import/steam_import.rs`）→ リポジトリ/外部APIクライアントの3層構成に従う。
- 🔵 APIキー取得は `ApiCredentialLookup` トレイト/`find_by_provider` 経由とし、テスト時はDI（固定キー注入）でDB非依存にできる構造とする（TASK-0023パターン）。
- 🔵 エラーは `ApiError` 型に統一し、HTTPステータスへ自動変換する。新規エラーコード `STEAM_API_KEY_INVALID`（401）を `ApiErrorCode` enum に追加する。

### データベース制約

- 🔵 `items` + `game_details` の複数テーブル操作は必ずトランザクション内で原子化する（note.md DB操作規約）。
- 🔵 重複防止: `(media_type=game, source=api, external_id=steam_appid)` の組み合わせで既存チェックし、存在する場合は新規作成せずスキップ（または更新）する。重複は失敗（failure）として扱わない。

### API制約

- 🔵 `POST /import/steam` は `api-endpoints.md` 定義のエラーコード `STEAM_API_KEY_INVALID`（401, TC-017-E01）を返す。
- **参照したEARS要件**: REQ-017, REQ-015（APIキー管理）, NFR（外部APIタイムアウト: TASK-0018/0019由来）
- **参照した設計文書**:
  - `docs/design/mediavault-backend/api-endpoints.md`「POST /import/steam」エラーコード
  - `backend/mediavault-api/src/models/response.rs`（`ApiError`/`ApiErrorCode`）
  - `backend/mediavault-api/src/repositories/api_credential_repository.rs`（`find_by_provider`）

---

## 4. 想定される使用例（EARS Edgeケース・データフローベース）

### 基本的な使用パターン（通常要件 REQ-017）

- 🔵 **正常系（TC-017-01）**: 有効なSteam APIキーが設定済みで、Steam APIが複数件の所持ゲームを返す。`POST /import/steam { "steam_id": "76561198000000000" }` 呼び出し → 取得件数分の `items`(`media_type=game`,`source=api`)+`game_details` を登録 → `success_count` が件数と一致する `ImportSummary` を返す。

### データフロー

- 🔵 リクエスト受信 → `steam_id` 検証 → APIキー取得 → `GetOwnedGames` 呼び出し → 各ゲーム変換・登録（重複チェック込み）→ 結果集計 → `ImportSummary` 返却。

### エッジケース

- 🟡 **プロフィール非公開（TC-017-02）**: Steam APIが空配列 `[]` を返す。エラーとせず `success_count=0, failure_count=0, failures=[]` で `200 OK` を返す（note.md／TASK-0031注意事項）。
- 🟡 **重複 steam_appid（テストケース5）**: 既に同一 `steam_appid` で `items` に登録済みのゲームを含む一覧。重複分は新規作成せずスキップ（または更新）として処理し、failure には記録しない。
- 🟡 **一部ゲーム登録失敗（TC-017-E02 / EDGE-002）**: 取得した複数件のうち1件がDB制約違反等で登録失敗。失敗した1件は `failures` に `ImportFailure` として記録し、他の正常なゲームの登録は継続する。

### エラーケース

- 🔵 **Steam APIキー無効（TC-017-E01）**: APIキー未設定、またはSteam APIが認証エラー（401/403相当）を返す → `401 Unauthorized` を `STEAM_API_KEY_INVALID` エラーコードとともに返す。
- 🟡 **steam_id 形式不正（テストケース4）**: `steam_id` が空文字または非数値文字列・桁数不正 → `400 Bad Request`（`VALIDATION_ERROR` 相当）を返す。
- 🟡 **外部APIタイムアウト**: Steam APIがタイムアウト/ネットワークエラー → `EXTERNAL_API_TIMEOUT`（502相当）を返す（TASK-0018/0019パターン再利用）。
- **参照したEARS要件**: REQ-017, EDGE-002
- **参照した受け入れ基準**: TC-017-01, TC-017-E01, TC-017-02, テストケース4, テストケース5
- **参照した設計文書**:
  - `docs/tasks/mediavault-backend/TASK-0031.md`（単体テスト要件・注意事項）
  - `docs/design/mediavault-backend/dataflow.md`（インポートフロー）

---

## 5. EARS要件・設計文書との対応関係

- **参照したユーザストーリー**: Steamライブラリ一括インポート（As a メディア管理ユーザー / I want 所持ゲームを一括取り込みたい / So that 手入力の手間を省きたい）
- **参照した機能要件**: REQ-017（Steamライブラリインポート）, REQ-015（外部APIキー管理・関連）
- **参照した非機能要件**: 外部APIタイムアウトハンドリング（NFR、TASK-0018/0019由来 `EXTERNAL_API_TIMEOUT`）、APIキーDB保存のセキュリティ要件
- **参照したEdgeケース**: EDGE-002（一部失敗時の処理継続）
- **参照した受け入れ基準**:
  - TC-017-01: 正常な所持ゲーム一覧の一括登録
  - TC-017-E01: Steam APIキー無効時の401
  - TC-017-02: プロフィール非公開時（空配列）の正常終了
  - テストケース4: steam_id 形式不正時の400
  - テストケース5: 重複 steam_appid のスキップ・更新
- **参照した設計文書**:
  - **アーキテクチャ**: `docs/design/mediavault-backend/architecture.md`（3層構成、リポジトリパターン、ApiError統一）
  - **データフロー**: `docs/design/mediavault-backend/dataflow.md`（インポートフロー）
  - **型定義**:
    - `backend/api-client-lib/src/clients/steam/models.rs`（`GetOwnedGamesRequest`/`SteamOwnedGamesModel`/`SteamGameEntry`）
    - `backend/mediavault-api/src/models/import.rs`（`ImportSummary`/`ImportFailure`）
    - `backend/mediavault-api/src/models/item.rs`（`ItemSource::Api`）
  - **データベース**: `items` / `game_details` / `api_credentials` テーブル
  - **API仕様**: `docs/design/mediavault-backend/api-endpoints.md`「POST /import/steam」

---

## 6. 実装対象ファイル

### 新規作成
- `backend/mediavault-api/src/handlers/import_steam.rs` — ハンドラ（検証・usecase呼び出し・`ImportSummary`返却）
- `backend/mediavault-api/src/import/steam_import.rs` — Steam API連携・変換・重複チェック・DB登録オーケストレーション
- `backend/mediavault-api/src/import/mod.rs` — import モジュール入口（`pub use steam_import::*`）

### 既存拡張
- `backend/mediavault-api/src/models/response.rs` — `ApiErrorCode` に `STEAM_API_KEY_INVALID`(401) 追加
- `backend/mediavault-api/src/routes/mod.rs` — `.route("/import/steam", post(import_steam_handler))` 追加
- `backend/mediavault-api/src/handlers/mod.rs` — `pub mod import_steam;` 追加

### 既存活用（変更不要）
- `backend/mediavault-api/src/models/import.rs`（`ImportSummary`/`ImportFailure`）
- `backend/mediavault-api/src/models/item.rs`（`ItemSource::Api`）
- `backend/mediavault-api/src/repositories/item_repository.rs`（`create_item_with_source`）
- `backend/mediavault-api/src/repositories/api_credential_repository.rs`（`find_by_provider`）
- `backend/api-client-lib/src/clients/steam/mod.rs`（`SteamClient`）

---

## 品質判定

```
✅ 高品質:
- 要件の曖昧さ: なし（入出力・エラー・重複・継続処理がタスク仕様/note.mdで明確）
- 入出力定義: 完全（リクエスト型・外部API型・レスポンス型すべて確定）
- 制約条件: 明確（トランザクション・重複防止・APIキー・エラーコードを定義）
- 実装可能性: 確実（前提タスクTASK-0023/0009/0030の既存パターンを再利用）
- 信頼性レベル分布: 🔵中心（基幹要件は青信号）、一部🟡（厳密範囲検証・タイムアウト・重複更新方針）
```

### 信頼性レベル集計
| カテゴリ | 🔵 | 🟡 | 🔴 |
|---|---|---|---|
| 1. 機能概要 | 4 | 0 | 0 |
| 2. 入出力 | 8 | 1 | 0 |
| 3. 制約条件 | 8 | 1 | 0 |
| 4. 使用例 | 3 | 5 | 0 |
| **合計** | **23** | **7** | **0** |

**総合評価**: 高品質（🔵が大半、🔴ゼロ）

---

## 次のステップ

次のお勧めステップ: `/tsumiki:tdd-testcases mediavault-backend TASK-0031` でテストケースの洗い出しを行います。
