# TASK-0025 TDD開発コンテキストノート: POST /items/import 実装

**タスク**: TASK-0025 - POST /items/import エンドポイント実装
**プロジェクト**: mediavault-backend
**作成日**: 2026-06-26
**関連要件**: docs/tasks/mediavault-backend/TASK-0025.md

---

## 1. 技術スタック

### 言語・フレームワーク
- **言語**: Rust (edition 2024)
- **Webフレームワーク**: Axum 0.8.9
- **非同期ランタイム**: Tokio 1.52.3
- **データベース**: PostgreSQL (sqlx 0.8)
- **JSON処理**: serde_json 1.0.150
- **参照元**: backend/mediavault-api/Cargo.toml

### アーキテクチャパターン
- **レイヤード構造**: routes → handlers → services → repositories → db
- **特徴**: 単一ユーザー・小規模運用前提。CQRSやマイクロサービス不採用
- **参照元**: docs/design/mediavault-backend/architecture.md L20-28

### トランザクション処理
- **既存パターン**: sqlx::Transaction 使用（TASK-0009のitems作成で実装済み）
- **原子性確保**: items + メディア別詳細テーブル同時INSERT
- **参照元**: backend/mediavault-api/src/repositories/item_repository.rs L51-94

---

## 2. 開発ルール

### コーディング規約
- **ハンドラ設計**: Axum extractors を活用（Query, Json, State, Path等）
- **エラーハンドリング**: `Result<T, ApiError>` 統一。`?` 演算子で自動伝播・panic防止
- **DTO定義**: serde::Deserialize + Serialize で自動導出
- **テスト配置**: ハンドラ・モデルファイル内に `#[cfg(test)] mod tests` でインライン配置

### エラー処理方針
- **APIレスポンス形式統一**: 成功=`ApiOk<T>`、失敗=`ApiError` をそのまま `IntoResponse`
- **エラーコード文字列**: ワイヤーコードは snake_case（`VALIDATION_ERROR`等）
- **情報漏洩防止**: DB内部情報・外部API生エラー詳細をクライアントへ返さない
- **参照元**: docs/design/mediavault-backend/architecture.md L37-39

### ルーティング方針
- **登録順序**: リテラルパス（`/items/import`）を動的パス（`/items/:id`）より前に登録して誤マッチ防止
- **配置**: 全エンドポイントが `routes/mod.rs` 内 `build_router` 関数に `.route(...)` でフラット列挙
- **参照元**: docs/design/mediavault-backend/api-endpoints.md 共通仕様

### テスト規約
- **ユニットテスト**: DB非依存の変換・デシリアライズ検証
- **ルーター統合**: `build_router(state)` + `tower::ServiceExt::oneshot` で駆動
- **E2E統合**: `#[ignore]` 付与で `cargo test -- --ignored` 実行（実DB前提）
- **参照元**: TASK-0024 note.md テスト規約セクション

---

## 3. 関連実装

### TASK-0009: POST /items（手動作成）既存実装
- **ハンドラファイル**: backend/mediavault-api/src/handlers/items.rs L27-42 (create_item_handler)
- **リポジトリファイル**: backend/mediavault-api/src/repositories/item_repository.rs L51-94 (create_item)
- **パターン**:
  - ハンドラで CreateItemRequest を検証し、リポジトリの create_item を呼び出す
  - リポジトリで sqlx::Transaction を開始
  - items テーブルへ INSERT → detail_table_name() で解決したテーブルへ INSERT → commit
  - source=manual, external_id=NULL 固定

### TASK-0024: GET /items/search（外部API検索）既存実装
- **サービスファイル**: backend/mediavault-api/src/services/external_search.rs
- **DTO型**:
  - `ExternalSearchResult`: media_type, provider (Option), external_id, title, raw_data 等
  - `ExternalSearchError`: ApiKeyNotConfigured(ApiProvider) | ExternalApiError(...)
- **特性**: PgPool を直接構築・所有のためモック化困難
- **参照元**: TASK-0024 note.md 関連実装セクション

### 既存ハンドラパターン
- **ファイル**: backend/mediavault-api/src/handlers/items.rs
- **実装例**: `create_item_handler` (L27-42), `list_items_handler` (L84-100)
- **パターン**:
  - `State(state): State<AppState>` で依存注入
  - `Json<Dto>` でボディ抽出
  - 成功→`ApiOk::new(data)` またはステータスコード指定で構築
  - 失敗→`Err(ApiError::...)` で伝播
  - HTTP 201 作成時は `created_response(item)` 関数で構築 (L49-52)

### MediaType enum
- **ファイル**: backend/mediavault-api/src/models/item.rs L12-24
- **Variant**: anime, movie, drama, manga, novel, game, academic_book, paper（8個）
- **デシリアライズ**: snake_case 文字列として既に Deserialize 実装済み

### Item構造体
- **ファイル**: backend/mediavault-api/src/models/item.rs L46-64
- **external_id フィールド**: `pub external_id: Option<String>`（NULL許容）
- **source フィールド**: `pub source: ItemSource` (Api | Manual)
- **特徴**: sqlx::FromRow で自動実装済み

### ItemSource enum
- **ファイル**: backend/mediavault-api/src/models/item.rs L37-43
- **Variant**: Api, Manual
- **デシリアライズ**: snake_case で既に実装済み

---

## 4. 設計文書

### API仕様（外部API検索・インポート）
- **ファイル**: docs/design/mediavault-backend/api-endpoints.md
- **対象**: POST /items/import エンドポイント定義（TASK-0025）
- **リクエスト契約**: media_type, external_id（必須）, title, description等
  - external_id は GET /items/search 結果から取得した値
- **成功レスポンス**: 201、ApiOk<Item> で作成済み item を返却
- **エラーコード**: 
  - 400 VALIDATION_ERROR（external_id欠落・不正値）
  - 409 ITEM_ALREADY_IMPORTED（重複登録時、設計判断により決定予定）

### アーキテクチャ（レイヤード構造）
- **ファイル**: docs/design/mediavault-backend/architecture.md
- **関連セクション**: コンポーネント構成 L30-46
- **設計判断**: routes → handlers → services → repositories の単方向依存

### データフロー（機能2: 外部API検索結果からのインポート）
- **ファイル**: docs/design/mediavault-backend/dataflow.md
- **フロー**: クライアント → POST /items/import → ハンドラ → リポジトリ (create_item) → items+詳細テーブル INSERT → Item返却

### 要件定義書
- **ファイル**: docs/spec/mediavault-backend/requirements.md
- **対象要件**: REQ-002（外部API検索結果からアイテム新規作成）
- **関連状態要件**: REQ-201b（source=api の場合、external_id を保持）

### テストケース定義書（TASK-0025）
- **ファイル**: docs/tasks/mediavault-backend/TASK-0025.md L76-100
- **テストケース数**: 4個
  - TC-002-03: 検索結果からitem作成（正常系）
  - external_id欠落時に400
  - 重複external_idのインポート（仕様確定待ち）
  - TASK-0009ロジックとの一貫性

---

## 5. テスト関連情報

### テストフレームワーク
- **テスト実行**: `cargo test -p mediavault-api`（ユニット・ルーター）、`cargo test -- --ignored`（統合）
- **非同期対応**: `#[tokio::test]` マクロで async テスト関数対応
- **ルーター駆動**: `tower::ServiceExt::oneshot(request)` で単一リクエスト駆動
- **外部APIモック**: `wiremock` 0.6（HTTP mock server）で既知応答返却
- **参照元**: backend/mediavault-api/Cargo.toml (wiremock = "0.6")

### 既存テストパターン（routes/mod.rs）
- **ファイル**: backend/mediavault-api/src/routes/mod.rs（`#[cfg(test)] mod tests` セクション）
- **初期化**: `test_app_state()` で実 DB コネクション取得（`.env` + DATABASE_URL 前提）
- **ルーター構築**: `build_router(state)` で全ルート登録済みルーターを生成
- **例**: GET /items?media_type=invalid でステータス 400 確認パターン

### 統合テスト実行環境前提
- **Database setup**: `docker compose up -d db`（postgres コンテナ起動）
- **環境変数**: `.env` に `DATABASE_URL=postgresql://...` 設定
- **外部API**: wiremock + ベースURL注入による HTTP モック
- **参照元**: backend/CLAUDE.md

### テストコメント規約（日本語指針）
- **テスト目的**: 「【テスト目的】: 〜を確認する」で要件対応を明示
- **テスト内容**: 「【テスト内容】: 〜を実行する」で具体処理を記述
- **期待される動作**: 「【期待される動作】: 〜であること」で期待値を記述
- **信頼性レベル**: 「🔵/🟡 信頼性レベル: 〜より」で根拠を明示
- **参照元**: TASK-0024 note.md テスト関連情報セクション

---

## 6. 注意事項

### 実装上の制約
1. **TASK-0009の create_item 再利用方針**:
   - 既存: `create_item(CreateItemRequest)` は source=manual, external_id=NULL で固定
   - TASK-0025: source=api, external_id=Some(...) を設定する必要がある
   - **設計判断**: CreateItemRequest またはリポジトリレベルで拡張するか、サービス層で中間型を導入するか決定が必要
   - **推奨**: repository::create_item_internal(pool, media_type, ..., source, external_id) 等の内部関数化 or CreateItemInput中間型導入

2. **外部_id必須バリデーション**:
   - ハンドラで external_id が欠落または空文字の場合は 400 VALIDATION_ERROR を返す
   - 参照元: TASK-0025.md 実装詳細 L60-63

3. **重複登録チェック設計待ち** (🟡 信頼性):
   - 同一 media_type + external_id の組み合わせが既に存在する場合の挙動は未決定
   - TASK-0025.md L65-68 に基づき、実装前にプロダクトオーナーへ確認推奨
   - 本タスク開始前に確定させることが望ましい

4. **routes/mod.rs への登録**:
   - リテラルパス `/items/import` を `/items/:id` より前に登録
   - ルーター構築時点で既に `/items/search` が存在（TASK-0024）のため登録順序に注意

### 要件上の重要ポイント
- **トランザクション一貫性**: TASK-0009 のトランザクション処理を再利用し、source/external_id のみ異なる形で items+詳細テーブルを同時作成する
- **APIキー検証不要**: POST /items/import は内部REST APIではなく、ユーザー向けエンドポイント（認証なし）
- **meta_data詳細化**: TASK-0025.md L45 の `details: serde_json::Value` をメディア別詳細テーブルの各カラムに反映するかは、TASK-0009 の既存実装状況に依存（現状は item_id のみINSERTする実装）

### セキュリティ・パフォーマンス
- **情報漏洩防止**: エラーレスポンスに DB 内部情報を含めない（既存 ApiError 汎用メッセージ方針踏襲）
- **SQL インジェクション防止**: detail_table_name() の固定文字列 match のため、SQLインジェクション の危険なし
- **コネクションプール**: 単一ユーザー・セルフホスト想定で小規模コネクションプール（既存設定踏襲）

---

## 7. ファイル一覧（相対パス）

### 実装対象
- `backend/mediavault-api/src/models/` - ImportItemRequest DTO 新規定義（item_import.rs または item.rs に追記）
- `backend/mediavault-api/src/handlers/items.rs` - import_item ハンドラ実装 + テスト
- `backend/mediavault-api/src/repositories/item_repository.rs` - 再利用可能な create_item 設計確定・拡張
- `backend/mediavault-api/src/routes/mod.rs` - POST /items/import ルート登録

### 参照文書
- `docs/tasks/mediavault-backend/TASK-0025.md` - タスク定義書
- `docs/spec/mediavault-backend/requirements.md` - 統合要件定義（REQ-002）
- `docs/design/mediavault-backend/api-endpoints.md` - API 仕様
- `docs/design/mediavault-backend/architecture.md` - アーキテクチャ
- `docs/design/mediavault-backend/dataflow.md` - データフロー

### 既存参照コード
- `backend/mediavault-api/src/handlers/items.rs` - 既存ハンドラパターン（create_item_handler等）
- `backend/mediavault-api/src/repositories/item_repository.rs` - トランザクション処理パターン（create_item関数）
- `backend/mediavault-api/src/models/item.rs` - Item, ItemSource, MediaType定義
- `backend/mediavault-api/src/models/response.rs` - ApiError, ApiErrorCode定義
- `backend/mediavault-api/src/routes/mod.rs` - ルーター実装＆テストパターン
- `backend/mediavault-api/src/main.rs` - AppState 定義

---

## 8. 次ステップ

### tdd-red 着手前確定事項
1. **TASK-0009 create_item 拡張方針**: 
   - Option A: CreateItemRequest に source/external_id フィールド追加（ハンドラで制御）
   - Option B: repository 層に内部関数 create_item_with_source(source, external_id) を追加
   - Option C: サービス層の中間型 CreateItemInput を導入
   - **推奨**: Option B（既存 create_item は変更無し、新タスクで新関数追加）

2. **重複チェック方針確定**:
   - テストケース3（TASK-0025.md L89-93）の「重複時挙動」を確定
   - 409 ITEM_ALREADY_IMPORTED を返すか、既存レコード返却か、許容するか

3. **ImportItemRequest DTO 配置**:
   - item_import.rs（新規）か item.rs（追記）か決定

4. **ApiErrorCode 拡張（必要に応じて）**:
   - ITEM_ALREADY_IMPORTED 追加が必要な場合は models/response.rs に variant 追加

5. **wiremock 確認**: 
   - backend/mediavault-api/Cargo.toml に wiremock 0.6 が既に記載済み確認 ✓

### 実装順序推奨
1. ImportItemRequest DTO 定義
2. TASK-0009 create_item 拡張方針確定 → リポジトリ層で新関数追加 or 既存関数修正
3. ApiErrorCode 拡張（重複チェック用エラーコード追加、必要に応じて）
4. import_item ハンドラ実装
5. ルート登録（routes/mod.rs）
6. テスト実装（各ファイルのインライン + routes テスト）
7. 統合テスト実装（#[ignore] 付与）

---

## 9. 信頼性評価サマリー

| 項目 | 🔵確実 | 🟡推測 | 🔴未確定 | 合計 |
|---|---|---|---|---|
| 実装パターン | 4 | 2 | 1 | 7 |
| テスト要件 | 2 | 2 | 0 | 4 |

**全体評価**: 中高品質
- ✓ TASK-0009 の実装パターンが確定済み（再利用可能）
- ✓ 既存ハンドラ・リポジトリパターンが確立済み
- 🟡 create_item 拡張方式の設計判断待ち
- 🟡 重複チェック方針の要件確定待ち
- **推奨アクション**: tdd-red 前に上記 2 点を確定

