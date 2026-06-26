# TASK-0029: 内部REST APIルート群実装（/internal/items等） - 開発コンテキスト

## 1. 技術スタック

### 使用技術・フレームワーク
- **言語**: Rust (Edition 2024)
- **Webフレームワーク**: Axum 0.8.9
- **非同期ランタイム**: Tokio 1.52.3（features: full）
- **データベース**: PostgreSQL + sqlx 0.8（コンパイル時SQLチェック有効、async対応）
- **シリアライゼーション**: serde 1.0.228（derive機能）、serde_json 1.0.150
- **UUID**: uuid 1.0（v4, serde対応）
- **日時処理**: chrono 0.4（serde対応）
- **環境変数**: dotenvy 0.15
- **ロギング**: tracing 0.1、tracing-subscriber 0.3
- **HTTP ミドルウェア**: tower 0.5.3、tower-http 0.7.0（CORS機能）

### アーキテクチャパターン
- **パターン**: レイヤードアーキテクチャ
- **層構成**: routes → handlers → (services) → repositories → db/sqlx
- **設計方針**: DB操作をrepository層に集約し、sqlxのコンパイル時チェック活用。ハンドラから直接SQLを書かない。
- **内部API設計**: `/internal` プレフィックスを持つ専用ルーター。TASK-0006のAPIキー検証ミドルウェア（`api_key_auth`）をLayerで適用。利用者向けルーター（`/api/v1`）とは`Router::merge`で統合。

### 参照元
- `backend/mediavault-api/Cargo.toml`
- `docs/design/mediavault-backend/architecture.md`

---

## 2. 開発ルール

### プロジェクト固有のルール

#### エラーハンドリング
- **エラー型**: `ApiError`（response.rs定義）を使用
- **エラーコード**: 統一エラーコード（ITEM_NOT_FOUND, VALIDATION_ERROR, INTERNAL_ERROR等）を適用
- **DB層でのエラー処理**: sqlx::Errorを統一的に`ApiError`に変換する
  - クライアントへはDB内部情報を含めない（セキュリティ対策）
  - 詳細はtracing::error!でサーバーログのみに出力
- **参考実装**: `backend/mediavault-api/src/repositories/item_repository.rs`の`db_error()`関数

#### APIキー認証
- **方式**: `Authorization: Bearer {INTERNAL_API_KEY}` ヘッダー検証
- **実装済み**: TASK-0006で実装済みの`api_key_auth`ミドルウェア（`middleware/api_key_auth.rs`）
- **未設定・不一致**: `401 Unauthorized`を返す（TC-018-E01）
- **内部API のみ必須**: `/api/v1` エンドポイントはAPIキー検証なし、`/internal` エンドポイントのみ検証ミドルウェア適用

#### レスポンス形式
- **成功**: `{"success": true, "data": {...}}` (ApiOk構造体)
- **ページネーション**: pagination フィールド付き
- **エラー**: `{"success": false, "error": {"code": "...", "message": "..."}}`
- **ステータスコード**: 作成時201, 削除時204, 更新時200, エラー時4xx/5xx

#### 入力検証とバリデーション
- **場所**: ハンドラの関数内でリクエストボディを検証（serde deserialize + 追加チェック）
- **パターン**: `parse_*` 関数を別途実装し、結果を`Result`で返す（早期リターン可能にする）
- **例**: `parse_create_item_request()` → CreateItemRequest構造体の検証済み取得
- **内部API向けの拡張**: `/internal/items/:id/groups` のupsert振る舞いなど、既存の`parse_*`関数を呼び出して検証する

#### トランザクション処理
- **使用**: sqlx::Transaction<Postgres>
- **パターン**: `pool.begin().await` → sqlx操作 → tx.commit().await
- **用途**: 複数テーブルへの同一アトミック操作（例: items作成 + 詳細テーブル作成）

### コード規約

#### モジュール構成
```
src/
  handlers/
    items.rs           # items関連（POST /items, PATCH /items/:id, GET /items）
    item_files.rs      # files関連（POST /items/:id/files）
    groups.rs          # groups関連（POST /items/:id/groups）
    episodes.rs        # episodes関連（POST /internal/groups/:group_id/episodes）
    internal_*.rs      # 内部API専用ハンドラ（新規）
  models/
    item.rs            # CreateItemRequest, UpdateItemRequest, ItemStatusRequest等
    item_file.rs       # ItemFileRequest等
    response.rs        # ApiError, ApiErrorCode等
  repositories/
    item_repository.rs # items CRUD
    item_file_repository.rs # item_files CRUD
    item_group_repository.rs # item_groups CRUD
    item_episode_repository.rs # item_episodes CRUD
  routes/
    mod.rs             # ルーティング定義（内部API ルーター設定を含む）
  services/
    # 既存サービス層は未使用（必要に応じて拡張）
  middleware/
    api_key_auth.rs    # APIキー検証（TASK-0006実装済み）
  db/
    # DB接続プール設定
```

#### ファイル名命名規則
- モジュールファイル: `{entity}.rs` (例: `items.rs`, `groups.rs`)
- 集約: `mod.rs` で各モジュールをpub use
- テストファイル: `{entity}_tests.rs` または `#[cfg(test)] mod tests`

#### ドキュメントコメント規約（参考実装から）
```rust
/// 【機能概要】: 簡潔な説明
/// 【実装方針】: どのように実装するか
/// 【テスト対応】: 対応するテストケース
/// 🔵/🟡 信頼性レベル: 根拠
pub async fn function_name() { ... }
```

#### 型定義パターン
- **Enum**: `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]`
  - `#[sqlx(type_name = "enum_name", rename_all = "snake_case")]`
  - `#[serde(rename_all = "snake_case")]`
- **構造体**: `#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]`

#### リクエスト/レスポンス構造体
- **リクエスト**: `CreateXxxRequest`, `UpdateXxxRequest`, `DeleteXxxRequest`
  - Deserializeのみ必要（serde::Deserialize）
  - バリデーション関数を別途実装（parseXxx）
- **レスポンス**: Xxx（models/xxx.rs）
  - Serializeが必須（serde::Serialize）
  - sqlx::FromRowで直接SELECT結果をマッピング可能にする

### 参照元:
- `backend/mediavault-api/src/handlers/items.rs`
- `backend/mediavault-api/src/models/item.rs`
- `backend/mediavault-api/src/repositories/item_repository.rs`
- `backend/mediavault-api/src/middleware/api_key_auth.rs` (TASK-0006)

---

## 3. 関連実装（参考パターン）

### Phase 2（TASK-0009～0013）実装済みCRUDハンドラ・リポジトリの再利用パターン

#### Items CRUD（TASK-0009/0010実装済み）
- `handlers/items.rs`
  - `create_item_handler()`: リクエスト検証 → repository呼び出し → レスポンス（201）
  - `list_items_handler()`: ページネーション処理 + フィルタ → repository呼び出し
  - `get_item_handler()`: 単一取得 → 404判定
  - `update_item_handler()`: 部分更新 → 404判定（TASK-0012）
  - `delete_item_handler()`: 削除 + カスケード削除（TASK-0013）
  - `normalize_pagination()`: page/limitの正規化
  - `created_response()`: ステータスコード201を明示的に返す
  
- `models/item.rs`
  - `Item`: `sqlx::FromRow`でDB結果を直接マップ
  - `CreateItemRequest`: リクエストDTO
  - `UpdateItemRequest`: 部分更新用DTO（TASK-0012）
  - `ItemStatusRequest`: ステータス更新（TASK-0014）
  - `parse_create_item_request()`: 入力検証関数
  - `parse_update_item_request()`: 部分更新バリデーション（TASK-0012）

- `repositories/item_repository.rs`
  - `create_item()`: トランザクション開始 → items INSERT → detail_table INSERT → コミット
  - `list_items()`: QueryBuilder活用で動的WHERE句構築
  - `get_item_by_id()`: 単一取得
  - `update_item()`: 動的UPDATE（TASK-0012）
  - `delete_item()`: カスケード削除（TASK-0013）
  - `detail_table_name()`: media_typeから詳細テーブル名を解決
  - `db_error()`: sqlx::Error → ApiError変換

#### Groups/Episodes CRUD（TASK-0018/0019実装済み）
- `models/item.rs` 内または新規 `item_groups.rs`
  - `ItemGroup`: グループモデル
  - `CreateItemGroupRequest`: グループ作成リクエスト
  
- `repositories/item_group_repository.rs`
  - `create_or_update_group()`: upsert処理（既存グループなら更新、なければ新規作成）
  - `get_group_by_id()`
  - `delete_group()`

- 同様に `item_episodes.rs` / `item_episode_repository.rs`

#### File Registration（TASK-0026実装済み）
- `models/item_file.rs`
  - `ItemFile`: ファイルモデル
  - `CreateItemFileRequest`: パス指定方式のリクエスト
  
- `repositories/item_file_repository.rs`
  - `register_file()`: ファイルサーバー上のパス指定で登録
  - 相対パスのみをDB保持（バイナリはファイルサーバーHDD管理）

### 内部API固有パターン（本タスクで新規実装）

#### ハンドラレイアウト
- `/internal` プレフィックス用の新規ハンドラファイル: `handlers/internal_items.rs`, `handlers/internal_groups.rs`, `handlers/internal_episodes.rs`, `handlers/internal_files.rs`
- または既存ハンドラを共通化し、内部ルーターでマウント

#### ルーター設定（routes/mod.rs）
- 新規関数: `build_internal_router(state: AppState) -> Router`
- ミドルウェア適用: `.layer(axum::middleware::from_fn_with_state(state.clone(), api_key_auth))`
- 既存ルーターと統合: `app.merge(build_internal_router(state))`

#### Upsert処理（Groups/Episodes）
- 既存グループ・エピソードが存在する場合は更新
- 存在しない場合は新規作成
- 複数テーブルへの操作はトランザクション単位で保証

#### パス指定方式ファイル登録
- TASK-0026実装済みの「既存パス指定方式」をそのまま再利用
- バイナリ直接アップロードは提供しない（内部APIは`/items/:id/files` のパス指定のみ）

### 参照元ファイル:
- `backend/mediavault-api/src/handlers/items.rs`
- `backend/mediavault-api/src/models/item.rs`
- `backend/mediavault-api/src/repositories/item_repository.rs`
- `backend/mediavault-api/src/routes/mod.rs`
- `backend/mediavault-api/src/middleware/api_key_auth.rs` (TASK-0006)

---

## 4. 設計文書

### API仕様（内部REST API）

#### ルーター設定
- **プレフィックス**: `/internal`
- **認証**: TASK-0006のAPIキー検証ミドルウェア（`Authorization: Bearer {INTERNAL_API_KEY}`）を適用
- **バージョニング**: `/internal` 直下にマウント（`/api/v1` のようなバージョンプレフィックスなし）

#### POST /internal/items 🔵
- **信頼性**: 🔵 api-endpoints.md「POST /internal/items」より
- **説明**: アイテム新規登録（手動 or 外部API取得結果のインポート相当）
- **リクエスト**: `CreateItemRequest`（既存DTO再利用）
- **レスポンス**: 作成済みitem（201 Created）
- **エラー**: 入力検証エラー → `VALIDATION_ERROR`（400）
- **実装方針**: TASK-0009の`create_item_handler`をそのまま再利用するか、サービス層関数を共通化して両ハンドラから呼び出す

#### PATCH /internal/items/:id 🔵
- **信頼性**: 🔵 api-endpoints.md「PATCH /internal/items/:id」より
- **説明**: 既存アイテムのメタデータ部分更新
- **リクエスト**: `UpdateItemRequest`（既存DTO再利用）
- **レスポンス**: 更新後item（200 OK）
- **エラー**: 存在しないitem_id → `ITEM_NOT_FOUND`（404）、入力エラー → `VALIDATION_ERROR`（400）
- **実装方針**: TASK-0012の`update_item_handler`をそのまま再利用するか、共通化

#### GET /internal/items/search 🔵
- **信頼性**: 🔵 api-endpoints.md「GET /internal/items/search」より
- **説明**: タイトル・media_type・タグ・external_id 条件での検索
- **クエリパラメータ**: `title`, `media_type`, `tag_ids`, `external_id`, `page`, `limit`（すべてoptional）
- **レスポンス**: item配列 + ページネーション（200 OK）
- **実装方針**: TASK-0010の`list_items_handler`で既に実装済みの検索ロジックを再利用

#### POST /internal/items/:id/groups 🔵
- **信頼性**: 🔵 api-endpoints.md「グループ/エピソードの登録・更新」より
- **説明**: シーズン/巻/章グループの登録・更新（upsert）
- **リクエスト**: `CreateItemGroupRequest`（TASK-0018実装）
- **レスポンス**: 作成/更新済みグループ（201 Created or 200 OK）
- **エラー**: 存在しないitem_id → `ITEM_NOT_FOUND`（404）
- **実装方針**: TASK-0018のハンドラを再利用、upsert振る舞いを適用

#### POST /internal/groups/:group_id/episodes 🔵
- **信頼性**: 🔵 api-endpoints.md「グループ/エピソード」より
- **説明**: エピソードの登録・更新（upsert）
- **リクエスト**: `CreateItemEpisodeRequest`（TASK-0019実装）
- **レスポンス**: 作成/更新済みエピソード（201 Created or 200 OK）
- **エラー**: 存在しないgroup_id → 404
- **実装方針**: TASK-0019のハンドラを再利用

#### POST /internal/items/:id/files 🔵
- **信頼性**: 🔵 api-endpoints.md「ファイルサーバー上のパスを紐付け登録」より
- **説明**: ファイルサーバー上のパスを指定してファイル登録（相対パス方式のみ）
- **リクエスト**: `CreateItemFileRequest`（TASK-0026実装）
- **レスポンス**: 登録済みファイル（201 Created）
- **エラー**: 存在しないitem_id → `ITEM_NOT_FOUND`（404）
- **実装方針**: TASK-0026のハンドラをそのまま再利用

### テスト対応

#### TC-018-01: APIキー検証ミドルウェア適用確認 🔵
- **Given**: 正しい`INTERNAL_API_KEY`を`Authorization: Bearer`ヘッダーに設定
- **When**: `POST /internal/items`を呼び出し
- **Then**: ミドルウェアを通過し、ハンドラが実行されて`201 Created`相当が返る

#### TC-018-E01: APIキー不一致での401 🔵
- **Given**: 誤った値または未設定の`Authorization`ヘッダー
- **When**: `/internal/*`配下の任意のエンドポイントを呼び出し
- **Then**: `401 Unauthorized`が返り、ハンドラ本体は実行されない

#### TC-018-E02: 存在しないitem_idでの404 🔵
- **Given**: 正しいAPIキー + 存在しない`item_id`
- **When**: `PATCH /internal/items/:id`または`POST /internal/items/:id/groups`を呼び出し
- **Then**: `404 Not Found`が返る

#### TC-018-04: 検索クエリパラメータ未指定時の全件取得 🟡
- **Given**: 正しいAPIキー + クエリパラメータなし
- **When**: `GET /internal/items/search`を呼び出し
- **Then**: ページネーション付き全件一覧が返る

### 参照元
- `docs/design/mediavault-backend/api-endpoints.md`
- `docs/spec/mediavault-backend/acceptance-criteria.md`

---

## 5. テスト関連情報

### テストフレームワーク・設定ファイル
- **フレームワーク**: Rust の `#[test]` / `#[tokio::test]`
- **テスト実行**: `cargo test --workspace` または `cargo test -p mediavault-api`
- **テスト環境**: Docker Compose のPostgres コンテナ（`docker compose up -d db`）
- **環境変数**: `DATABASE_URL` で接続確認

### 既存テストのディレクトリ構成・命名パターン
- **場所**: 各実装ファイル末尾に `#[cfg(test)] mod tests` としてインライン配置（別ファイルなし）
- **参照例**:
  - `backend/mediavault-api/src/models/item.rs` L272-334（DTOデシリアライズテスト）
  - `backend/mediavault-api/src/repositories/item_repository.rs` L815-1100（DB統合テスト）
  - `backend/mediavault-api/src/handlers/items.rs` L195-350+ （ハンドラユニット＋統合テスト）
  - `backend/mediavault-api/src/routes/mod.rs` L144-180 （ルーティング統合テスト）

### テストユーティリティ・モック設定
- **DB接続**: `test_pool()` ヘルパー関数（`item_repository.rs` L1076-1082）
  - `DATABASE_URL`環境変数から接続
  - `#[tokio::test]` + `#[ignore]` で `cargo test -- --ignored` 実行対象

- **テストデータ挿入**: `insert_test_item()`, `insert_test_category()` 等のヘルパー（各リポジトリ）
  - 直接INSERT文で既知データをセットアップ
  - クリーンアップは明記されていない（テストDB使い捨て前提）

- **エラー検証**: `unreachable_pool()` で接続不能なPgPoolを構築し、DB層エラー変換を検証

- **SQL生成テスト**: `QueryBuilder.sql()` で動的SQL文字列をassert（実DB不要）

- **ハンドラレベル**: `test_app_state()` ヘルパー（`routes/mod.rs` L144-154）
  - `AppState { db, internal_api_key }`を構築
  - テスト用Postgres接続で実行

### テスト規約（既存方針を継続）
1. **DB非依存**: `#[test]`のみ → `cargo test -p mediavault-api`
2. **DB必須**: `#[tokio::test]` + `#[ignore]` → `cargo test -- --ignored`
3. **信頼性レベル記載**: 🔵/🟡/🔴 を各テスト関数に付与
4. **日本語コメント**: 【テスト目的】【テスト内容】【期待される動作】【確認内容】等の段落区分

### E2Eテスト設定（UIタスクの場合）
- **対象外**: MediaVaultバックエンド API のみのため、UI E2Eテストなし
- **代替**: ハンドラレベルの統合テスト（routes/mod.rs 内 `test_app_state()` パターン）で外部API連携確認

### 参照元:
- `backend/mediavault-api/src/models/item.rs` L272-334
- `backend/mediavault-api/src/repositories/item_repository.rs` L815-1100
- `backend/mediavault-api/src/handlers/items.rs` L195-350+
- `backend/mediavault-api/src/routes/mod.rs` L144-180

---

## 6. 注意事項

### 技術的制約

#### ハンドラ共通化の設計選択肢
- **選択肢A**: `/api/v1` と `/internal` で同一ハンドラ関数を共用（推奨）
  - `routes/mod.rs` 内で両プレフィックスから同一ハンドラをマウント
  - 実装重複を避ける
  - デメリット: 将来、内部API固有のロジック（例: 特別なバリデーション）が必要になった際に分岐が複雑化する可能性

- **選択肢B**: `/internal` 専用ハンドラを新規作成（代替案）
  - `handlers/internal_items.rs` 等で新規定義
  - 内部API固有ロジックの追加が容易
  - デメリット: 初期段階で重複コード発生

**実装方針**: 当面は選択肢Aで進め、必要に応じて分岐を検討すること

#### APIキー検証ミドルウェアの適用方法
- TASK-0006で実装済みの `api_key_auth` ミドルウェア（`middleware/api_key_auth.rs`）を利用
- `Router` に `.layer(axum::middleware::from_fn_with_state(...))` で適用
- `AppState` に `internal_api_key` フィールド保持済み（`main.rs` L18-19）

#### URLプレフィックス間違い注意
- `/internal` ルーターはバージョンプレフィックス（`/api/v1`）を持たない
- `build_router()` でマウント時、`/internal` を直接ルートに登録（`/api/v1/internal` にしない）

#### Upsert処理の一貫性保証
- グループ・エピソードで既存・新規を自動判定するupdate_or_insertロジックを実装
- 複数テーブル操作はトランザクションで原子性確保
- 例: `POST /internal/items/:id/groups` で既存グループなら更新、なければINSERT

### セキュリティ・パフォーマンス要件
- **APIキー未設定・不一致**: 確実に `401 Unauthorized` を返す（TC-018-E01確認）
- **ハンドラ本体実行禁止**: ミドルウェアで401応答後、ハンドラ関数は実行されないこと
- **入力検証**: 既存の`parse_*`関数を再利用して、API層での二重検証は避ける
- **リストAPI**: ページネーション（page/limit）で大量件数応答を防止
- **トランザクション**: 失敗時の自動ロールバックで一貫性保証

### 既存実装との互換性
- **Phase2/Phase4ハンドラの再利用**: 既存実装（TASK-0009～0013, TASK-0018, TASK-0019, TASK-0026）をそのまま呼び出す
- **DB層の変更不要**: リポジトリ層は既に完成、新規実装はハンドラ・ルーティング層のみ
- **リクエストDTOの再利用**: `CreateItemRequest`, `UpdateItemRequest` 等は既存定義を活用

---

## 7. 実装チェックリスト（TDD開発用）

### Red Phase
- [ ] テストケース1～4の失敗確認（未実装なため実行不可状態）

### Green Phase
- [ ] `/internal` ルーター生成関数実装
- [ ] APIキー検証ミドルウェア適用
- [ ] `POST /internal/items` ハンドラマウント（既存create_item_handler再利用）
- [ ] `PATCH /internal/items/:id` ハンドラマウント（既存update_item_handler再利用）
- [ ] `GET /internal/items/search` ハンドラマウント（既存list_items_handler再利用）
- [ ] `POST /internal/items/:id/groups` ハンドラマウント（既存orロジック再利用）
- [ ] `POST /internal/groups/:group_id/episodes` ハンドラマウント
- [ ] `POST /internal/items/:id/files` ハンドラマウント（既存再利用）
- [ ] `main.rs` でルーターマージ
- [ ] テストケース1～4 の全てがpass

### Refactor Phase
- [ ] ハンドラ重複排除（可能な限り既存を活用）
- [ ] エラーレスポンス統一確認
- [ ] トランザクション処理の一貫性確認
- [ ] ドキュメントコメント追記

### Verify Complete Phase
- [ ] すべてのテストケース実行確認
- [ ] APIキー検証が正確に機能することを確認
- [ ] 存在しないリソースに対して404を返すことを確認
- [ ] ページネーション処理が正確に機能することを確認
- [ ] 統合テスト（実DB接続）でのE2E動作確認

---

## 補足：プロジェクト全体の補助情報
- **AGENTS.md**: このリポジトリに存在しない
- **docs/rule/ ディレクトリ**: このリポジトリに存在しない
- **追加ルール**: なし（既存の開発ルールに従う）
- **進行状況**: Phase 3完了（TASK-0025完了済み）、Phase 5開始前のため、Phase2/3の実装をそのまま再利用可能
