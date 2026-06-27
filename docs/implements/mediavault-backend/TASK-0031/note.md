# TASK-0031 TDD開発コンテキストノート

Steamライブラリインポート機能（`POST /import/steam`）の実装に必要な開発コンテキスト情報を記載します。

---

## 1. 技術スタック

### 使用技術・フレームワーク
- **言語・ランタイム**: Rust 1.70+
- **Webフレームワーク**: Axum 0.6+（非同期ハンドラ）
- **データベース**: PostgreSQL + sqlx（クエリビルダ）
- **非同期実行時**: Tokio
- **HTTPクライアント**: reqwest（Steam Web API連携）
- **テストフレームワーク**: cargo test（インラインテスト）+ tokio::test（非同期テスト）
- **HTTPモック**: wiremock 0.6（Steam Web APIモック）

### アーキテクチャパターン
- **ハンドラ**: Axum エクストラクタ（Path, State, Json）→ usecase関数呼び出し
- **リポジトリパターン**: `item_repository`で永続層を抽象化
- **DI（Dependency Injection）**: `ApiCredentialLookup`トレイトでAPIキー取得を外部化（テスト用Mock対応）
- **エラーハンドリング**: `ApiError`型で統一、HTTP ステータスコードへの自動変換

### 参照元
- `backend/mediavault-api/src/main.rs`（Axumアプリ設定）
- `backend/mediavault-api/Cargo.toml`（依存クレート一覧）
- `backend/api-client-lib/Cargo.toml`（Steam クライアント）

---

## 2. 開発ルール

### プロジェクト固有ルール
1. **エラーハンドリング規約**
   - すべてのエラーは`ApiError`型で統一（response.rs）
   - HTTP ステータスコードは`ApiError`の`status`フィールドで定義
   - 新規エラーコードは`ApiErrorCode`enum に追加（TASK-0031では STEAM_API_KEY_INVALID を410追加予定）
   - エラーレスポンス形式: `{ "success": false, "error": { "code": "...", "message": "..." } }`

2. **データベース操作**
   - 複数テーブル操作（items + game_details）は必ずトランザクション処理
   - sqlx の QueryBuilder を使用して動的クエリ構築
   - エラー発生時は tx.rollback()で自動的に行われる（Result パターン）

3. **レスポンス形式**
   - 成功時: `{ "success": true, "data": {...} }`
   - エラー時: `{ "success": false, "error": { "code": "...", "message": "..." } }`
   - インポート系: `ImportSummary { success_count, failure_count, failures }`

4. **ドキュメント規約**
   - コード内で【機能概要】【実装方針】【テスト対応】の3段落を含む
   - 信頼性レベルを 🔵（完全実装済み）/ 🟡（実装中・不確定）で記載

5. **テストルール**
   - インラインテスト（`#[cfg(test)] mod tests`）を実装単位で記載
   - DB非依存テストは`test_pool()`、DB依存は`unreachable_pool()`を使用
   - 形式不正データ・API失敗時の境界値テストは必須（EDGE-002対応）

### 参照元
- `backend/mediavault-api/src/models/response.rs`（ApiError・ApiErrorCode定義）
- `docs/tasks/mediavault-backend/TASK-0030.md`（類似実装のドキュメント規約）
- `backend/mediavault-api/src/handlers/import_booklog.rs`（規約の実装例）

---

## 3. 関連実装

### 類似機能：TASK-0030（ブクログCSVインポート）
- **ハンドラ**: `backend/mediavault-api/src/handlers/import_booklog.rs`
  - multipart ファイル受け取り方式
  - 形式不正行をスキップ（ImportFailure に記録）
  - ImportSummary 返却（全行不正でも200ステータス）
  
- **リポジトリ呼び出し**: `item_repository::create_item_with_source(pool, request, source, external_id)`
  - items テーブル + detail テーブル（book_details）への自動トランザクション

- **再利用可能な型**:
  - `models/import.rs`: `ImportSummary`, `ImportFailure`
  - `models/item.rs`: `ItemSource::Manual`, `ItemSource::Api`

### ExternalSearchService パターン（TASK-0023）
- **APIキー取得**: `ApiCredentialLookup::Pool(pool)`でDB上のプロバイダ別キーを取得
- **エラー変換**: `ExternalSearchError` → `ApiError`への変換（例: ApiKeyNotConfigured→401）
- **テスト用DI**: `ApiCredentialLookup::Fixed`で固定キーを注入（DB非依存テスト）

### アイテム作成ロジック（TASK-0009）
- **関数**: `item_repository::create_item_with_source`
  - items テーブル INSERT → 自動採番 item_id
  - detail テーブル（game_details）に item_id を外部キーで登録
  - トランザクション内で原子化
  
- **パラメータ**:
  - `request: CreateItemRequest` → タイトル・メディアタイプ・その他メタデータ
  - `source: ItemSource::Api` → アイテムソースを API として記録
  - `external_id: Some(appid)` → Steam app_id を外部識別子として記録

### 参照元
- `backend/mediavault-api/src/handlers/import_booklog.rs`
- `backend/mediavault-api/src/repositories/item_repository.rs`
- `backend/mediavault-api/src/services/external_search.rs`
- `docs/implements/mediavault-backend/TASK-0030/note.md`

---

## 4. 設計文書

### TASK-0031 仕様
- **ファイル**: `docs/tasks/mediavault-backend/TASK-0031.md`
- **エンドポイント**: `POST /import/steam`
- **入力**: `{ "steam_id": "76561198000000000" }`（SteamID64形式）
- **出力**: `ImportSummary { success_count, failure_count, failures }`
- **完了条件**:
  1. steam_id 空・形式不正 → 400 (VALIDATION_ERROR)
  2. Steam APIキー未設定・無効 → 401 (STEAM_API_KEY_INVALID)
  3. 取得した所持ゲーム一覧を items + game_details として一括登録
  4. 既に同一 steam_appid で登録済み → 重複スキップ
  5. 一部ゲーム情報取得失敗時も他のゲーム登録継続（EDGE-002）
  6. ImportSummary を返却

### API クライアント（api-client-lib）
- **ファイル**: `backend/api-client-lib/src/clients/steam/mod.rs`
- **メソッド**: `SteamClient::get_owned_games(req: GetOwnedGamesRequest) → ApiResponse<SteamOwnedGamesModel>`
- **リクエスト**:
  ```rust
  pub struct GetOwnedGamesRequest {
      pub steam_id: u64,                  // SteamID64（例: 76561198000000000）
      pub include_appinfo: bool,          // ゲーム名・画像情報を含めるか
      pub include_played_free_games: bool // 無料ゲームを含めるか
  }
  ```
- **レスポンス**:
  ```rust
  pub struct SteamOwnedGamesModel {
      pub game_count: u32,
      pub games: Vec<SteamGameEntry>,
  }
  
  pub struct SteamGameEntry {
      pub appid: u32,
      pub name: Option<String>,
      pub playtime_forever: u32,  // プレイ時間（分）
      pub img_icon_url: Option<String>,
  }
  ```

### APIクレデンシャル管理
- **ファイル**: `backend/mediavault-api/src/repositories/api_credential_repository.rs`
- **メソッド**: `find_by_provider(pool: &PgPool, provider: ApiProvider) → Result<Option<ApiCredential>, ApiError>`
- **プロバイダ**: `ApiProvider::Steam`で Steamキーを検索

### 参照元
- `docs/tasks/mediavault-backend/TASK-0031.md`
- `backend/api-client-lib/src/clients/steam/models.rs`
- `backend/mediavault-api/src/repositories/api_credential_repository.rs`

---

## 5. テスト関連情報

### テストフレームワーク・設定

**テスト実行方法**:
- DB非依存ユニットテスト: `cargo test -p mediavault-api`
- DB依存統合テスト: `DATABASE_URL=postgres://... cargo test -- --ignored`
- PostgreSQL コンテナ: `docker compose up -d db`（プロジェクトルート）

**テスト属性**:
- `#[tokio::test]` - 非同期テスト関数
- `#[ignore]` - DB依存テスト（明示的に実行）
- `#[cfg(test)]` - テスト時のみコンパイル

### 既存テストのディレクトリ構成・命名パターン

**テスト配置**: ハンドラ・サービス・リポジトリの同じファイル内にインラインテストを記載

**命名規約**（TASK-0030から）:
- テスト関数: `test_xxx_success` / `test_xxx_validation_error` / `test_xxx_api_error`
- テストモジュール: `mod tests { ... }`

**TASK-0030テストケース例**（参照パターン）:
- TC-021-01: 正常な CSV インポート（複数行）
- TC-021-E01: ファイルが空（0 バイト）時の 400 エラー
- TC-021-E02: CSV 形式不正な行をスキップ、他の行は登録（EDGE-002対応）
- TC-021-02: ItemSource::Manual として登録確認

### テストユーティリティ・モック設定

**HTTPモック（Steam API）**: wiremock 0.6
```rust
use wiremock::{Mock, MockServer, ResponseTemplate};

let mock_server = MockServer::start().await;
Mock::given(...)
    .respond_with(ResponseTemplate::new(200).set_body_json(...))
    .mount(&mock_server)
    .await;
```

**DB プール ヘルパー**:
- `test_pool()` - テスト用DBプール（テスト実行時にクリアされる）
- `unreachable_pool()` - DB非依存テスト用（パニック時に実行される）

**multipart テストボディ**（参考: TASK-0030）:
- boundaries・form-data 形式の手動構築（text/plain の JSON を送信）

### テスト対応予定（実装時に詳細化）

**ユニットテスト**: SteamID64 形式検証
- `test_steam_id_validation_valid` - 有効形式
- `test_steam_id_validation_invalid` - 無効形式（短すぎる・非数値）
- `test_steam_id_validation_empty` - 空文字列

**統合テスト**: エンドツーエンド
- TC-017-01: 正常な所持ゲーム一覧の一括登録（複数ゲーム）
- TC-017-E01: Steam APIキー無効時の 401 エラー
- TC-017-E02: steam_id 形式不正時の 400 エラー
- TC-017-E03: 一部ゲーム登録失敗時のスキップ継続（EDGE-002）
- TC-017-02: プロフィール非公開時（APIが空配列返却）の正常終了

### 参照元
- `backend/mediavault-api/src/handlers/import_booklog.rs`（テスト実装例）
- `docs/tasks/mediavault-backend/TASK-0030.md`（テストケース定義）
- `Cargo.toml`（wiremock・tokio 依存）

---

## 6. 実装ファイル一覧

### 新規作成ファイル
```
backend/mediavault-api/src/handlers/import_steam.rs
    - POST /import/steam ハンドラ関数
    - リクエスト検証（steam_id 形式チェック）
    - usecase 呼び出し
    - ImportSummary レスポンス返却

backend/mediavault-api/src/import/steam_import.rs
    - Steam Web API 連携ロジック
    - GetOwnedGames リクエスト構築
    - SteamGameEntry → CreateItemRequest 変換
    - 重複チェック・DB登録の orchestra

backend/mediavault-api/src/import/mod.rs
    - import モジュール入口
    - pub use steam_import::*
```

### 既存ファイル拡張
```
backend/mediavault-api/src/models/response.rs
    - ApiErrorCode enum に STEAM_API_KEY_INVALID (401) を追加

backend/mediavault-api/src/routes/mod.rs
    - .route("/import/steam", post(import_steam_handler)) を追加

backend/mediavault-api/src/handlers/mod.rs
    - pub mod import_steam; を追加
```

### 既存活用（変更不要）
```
backend/mediavault-api/src/models/import.rs
    - ImportSummary, ImportFailure（再利用）

backend/mediavault-api/src/models/item.rs
    - ItemSource::Api（再利用）

backend/mediavault-api/src/repositories/item_repository.rs
    - create_item_with_source()（再利用）

backend/mediavault-api/src/repositories/api_credential_repository.rs
    - find_by_provider()（APIキー取得）

backend/api-client-lib/src/clients/steam/mod.rs
    - SteamClient（既に実装）
```

---

## 7. 注意事項

### 技術的制約

1. **SteamID64 形式検証**
   - 17桁の数値文字列（例: 76561198000000000）
   - 簡易チェック: `steam_id.len() == 17 && steam_id.chars().all(|c| c.is_numeric())`
   - より厳密: SteamID64範囲 `(76561197960265728..=u64::MAX)` の検証

2. **重複防止（TASK-0009・TASK-0030）**
   - 既存: `(media_type=game, source=api, external_id=steam_appid)` で検索
   - 存在時: スキップして next ゲームへ
   - EDGE-002対応: 重複は failure ではなく、記録しない

3. **プロフィール非公開**
   - Steam Web API は空配列 `[]` を返却（エラーではない）
   - レスポンス: `success_count=0, failure_count=0, failures=[]`
   - HTTP 200 (正常終了)

4. **API レート制限・タイムアウト**
   - 既存パターン（TASK-0018/0019）: EXTERNAL_API_TIMEOUT (502)
   - wiremock でのテスト: 遅延レスポンス対応

5. **ItemSource 定義**
   - `backend/mediavault-api/src/models/item_import.rs` で確認
   - `ItemSource::Api` が既に定義済み

6. **consumed_date（拡張予約地）**
   - TASK-0030 で `create_item_with_source` が拡張済み
   - `playtime_forever`（分）をメタデータとして活用可能（将来）

### セキュリティ・パフォーマンス要件

1. **APIキー管理**
   - Steam API キーは環境変数ではなく DB に暗号化保存（既存: api_credentials テーブル）
   - キー漏洩時: 401 (STEAM_API_KEY_INVALID) で安全に返却

2. **バッチ処理の最適化**
   - 複数ゲーム登録時に `INSERT ... ON CONFLICT ... DO NOTHING` パターン（TASK-0030参考）
   - 個別 INSERT より 1回のトランザクション処理

3. **入力検証**
   - steam_id: 形式・範囲チェック → 400 (VALIDATION_ERROR)
   - API レスポンス: ゲーム appid・名前の null チェック

### 参照元
- `docs/tasks/mediavault-backend/TASK-0031.md`（仕様の「注意事項」セクション）
- `docs/tasks/mediavault-backend/TASK-0030.md`（EDGE-002定義）
- `docs/tasks/mediavault-backend/TASK-0009.md`（アイテム作成ロジック）

---

## 8. テストケース定義フェーズの確定要事項（tdd-testcases由来）

`steam-import-testcases.md` 作成時に判明した、Red/Green フェーズで方針確定が必要な論点：

1. **重複 steam_appid の集計方針（TC-017-E05）**: スキップ（集計外）か更新（successカウント）か。TASK-0031完了条件は「スキップまたは更新」併記。
2. **name=None エントリの扱い（TC-017-B02）**: フォールバックタイトルで登録（success）か、failures に記録か。
3. **ImportFailure の識別子**: 既存 `ImportFailure { row_number: u32, reason: String }` はCSV行番号前提。Steam では「行番号」がないため appid 流用 / インデックス流用 / 型拡張のいずれかを確定。
4. **ImportSummary カウント型**: 既存実装は `u32`（要件定義書記載の `usize` ではない）。実装は既存 `u32` に合わせる。
5. **トランザクション分離単位**: EDGE-002（部分失敗継続）と「1回のトランザクション」推奨の両立。件数分の独立トランザクション方式が有力。

---

## 開発準備の確認リスト

- [ ] `TASK-0031.md` を読み込み、完了条件を理解
- [ ] TASK-0030 の `import_booklog.rs` を参考実装として確認
- [ ] `api-client-lib` の `SteamClient::get_owned_games` 仕様を確認
- [ ] `api_credential_repository::find_by_provider` の使用方法を確認
- [ ] `item_repository::create_item_with_source` の引数・戻り値を確認
- [ ] `ApiErrorCode` 列挙型に `STEAM_API_KEY_INVALID` を追加予定
- [ ] テストフレームワーク（tokio::test・wiremock）の使用準備
- [ ] ハンドラ・usecase・リポジトリの 3層構成を確認

---

**作成日**: 2026-06-27
**作成対象**: TASK-0031（Steamライブラリインポート機能）
**前提完了タスク**: TASK-0023（ExternalSearchService）、TASK-0009（アイテム作成ロジック）
