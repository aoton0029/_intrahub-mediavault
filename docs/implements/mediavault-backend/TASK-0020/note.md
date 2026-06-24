# TASK-0020: スタッフ管理CRUD実装 - 開発コンテキスト

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

### 参照元: 
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

#### レスポンス形式
- **成功**: `{"success": true, "data": {...}}` (ApiOk構造体)
- **ページネーション**: pagination フィールド付き
- **エラー**: `{"success": false, "error": {"code": "...", "message": "..."}}`
- **ステータスコード**: 作成時201, 削除時204, エラー時4xx/5xx

#### 入力検証とバリデーション
- **場所**: ハンドラの関数内でリクエストボディを検証（serde deserialize + 追加チェック）
- **パターン**: `parse_*` 関数を別途実装し、結果を`Result`で返す（早期リターン可能にする）
- **例**: `parse_create_item_request()` → CreateItemRequest構造体の検証済み取得

#### トランザクション処理
- **使用**: sqlx::Transaction<Postgres>
- **パターン**: `pool.begin().await` → sqlx操作 → tx.commit().await
- **用途**: 複数テーブルへの同一アトミック操作（例: items作成 + 詳細テーブル作成）

### コード規約

#### モジュール構成
```
src/
  handlers/     # ハンドラ層（HTTP処理）
  models/       # リクエスト/レスポンス DTO・バリデーション
  repositories/ # DB操作層（sqlx query）
  routes/       # ルーティング定義
  services/     # （将来拡張用）
  middleware/   # ミドルウェア
  db/           # DB設定
```

#### ファイル名命名規則
- モジュールファイル: `{entity}.rs` (例: `items.rs`, `staff.rs`)
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

---

## 3. 関連実装（参考パターン）

### Items CRUD（TASK-0009実装済み）の参考パターン

#### ハンドラ層（handlers/items.rs）
- `create_item_handler()`: リクエスト検証 → repository呼び出し → レスポンス
- `normalize_pagination()`: page/limitの正規化（無効値を安全な値へクランプ）
- `created_response()`: ステータスコード201を明示的に返す

#### モデル層（models/item.rs）
- `Item`: `sqlx::FromRow`でDB結果を直接マップ
- `CreateItemRequest`: リクエストDTO、source/external_idはハンドラで固定値を付与
- `parse_create_item_request()`: media_type/title等のバリデーション関数

#### リポジトリ層（repositories/item_repository.rs）
- `create_item()`: トランザクション開始 → items INSERT → detail_table INSERT → コミット
- `detail_table_name()`: media_typeから詳細テーブル名を解決
- `db_error()`: sqlx::Error → ApiError変換

### 他の関連実装
- **tags** (TASK-0012実装予定): タグ管理（カテゴリ同様の多対多設計）
- **categories** (TASK-0012実装予定): カテゴリ管理（簡易CRUD）

### 参照元ファイル:
- `backend/mediavault-api/src/handlers/items.rs`
- `backend/mediavault-api/src/models/item.rs`
- `backend/mediavault-api/src/repositories/item_repository.rs`
- `backend/mediavault-api/src/routes/mod.rs`

---

## 4. 設計文書

### API仕様（staff関連）

#### POST /staff
- **信頼性**: 🔵 REQ-009・user-stories 4.1より
- **入力**: `name` (必須), `external_id` (optional), `image_url` (optional)
- **出力**: `Staff { id, name, external_id, image_url, created_at }` (201)
- **仕様書**: `docs/design/mediavault-backend/api-endpoints.md`

#### POST /items/:id/staff
- **信頼性**: 🔵 REQ-009より
- **入力**: `staff_id` (必須UUID), `role` (必須), `character_name` (optional)
- **出力**: `ItemStaff { id, item_id, staff_id, role, character_name }` (201)
- **エラー**: staff_idが存在しない場合 → STAFF_NOT_FOUND (404)
- **仕様書**: `docs/design/mediavault-backend/api-endpoints.md`

#### DELETE /items/:id/staff/:item_staff_id
- **信頼性**: 🟡 REQ-009「紐付け」から妥当な推測
- **入力**: item_id (パス), item_staff_id (パス)
- **出力**: 204 No Content（成功）
- **エラー**: item_staff_idが存在しない または item_idに属さない場合 → 404
- **仕様書**: `docs/design/mediavault-backend/api-endpoints.md`

### データベース設計

#### staff テーブル
```sql
CREATE TABLE staff (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id VARCHAR(100),              -- 重複登録防止用（AniList等）
    name VARCHAR(255) NOT NULL,
    image_url VARCHAR(1000),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_staff_external_id ON staff(external_id);
```

#### item_staff テーブル
```sql
CREATE TABLE item_staff (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    staff_id UUID NOT NULL REFERENCES staff(id) ON DELETE CASCADE,
    role VARCHAR(100) NOT NULL,            -- 監督, 声優など
    character_name VARCHAR(255)            -- 声優の場合のキャラ名
);
CREATE INDEX idx_item_staff_item_id ON item_staff(item_id);
CREATE INDEX idx_item_staff_staff_id ON item_staff(staff_id);
```

### 関連要件定義
- **REQ-009**: システムはスタッフ（staff）の追加、役割（role）の付与、作品への紐付け（item_staff）を行うAPIを提供しなければならない

### 参照元:
- `docs/design/mediavault-backend/api-endpoints.md`
- `docs/design/mediavault-backend/database-schema.sql`
- `docs/spec/mediavault-backend/requirements.md` (REQ-009)

---

## 5. テスト関連情報

### テストフレームワーク・設定

#### テストランナー
- **フレームワーク**: Rust標準テストシステム（#[test]）
- **非同期テスト**: #[tokio::test]
- **実行方法**: `cargo test`

#### テスト構成
- **ユニットテスト**: 関数単位のロジックテスト（バリデーション等）
- **統合テスト**: ハンドラ → repository → DBまでのE2Eテスト

### 既存テストパターン（TASK-0009から）

#### テスト用補助関数
- `parse_create_item_request()`: バリデーション結果の確認
- `normalize_pagination()`: 境界値テスト（page<1→1, limit>100→100）

#### テストケースの命名規則
- 正常系: `test_create_item_with_required_fields_only`
- 異常系: `test_create_item_with_invalid_media_type_returns_error`
- 境界値: `test_normalize_pagination_clamps_limit_to_100_max`

#### テストディレクトリ構成
```
backend/mediavault-api/
  src/
    handlers/items.rs
    models/item.rs
    repositories/item_repository.rs
  tests/
    integration_tests.rs     # E2Eテスト
```

### テスト用設定ファイル
- `.env.test`: テスト用DB接続文字列
- Cargo.toml: [dev-dependencies]にテストツール（必要に応じて追加）

### 実装時の注意点

#### 単体テスト（models層）
- `parse_create_staff_request()`: name検証（空文字, 長さ制限等）
- `parse_create_item_staff_request()`: role/character_name検証

#### 統合テスト（handlers・repository）
- テスト用DBコンテナ起動（docker-compose.test.yml等）
- トランザクションロールバック（各テスト後にDELETEで清掃）

### 参照元:
- `backend/mediavault-api/src/handlers/items.rs` (テストコメント参照)
- `backend/mediavault-api/src/models/item.rs` (バリデーション関数)
- `backend/mediavault-api/tests/` (既存テストパターン)

---

## 6. 注意事項

### 技術的制約

#### DB制約
- `staff.external_id`: 外部API重複登録防止用だが、本フェーズでは外部API連携なし
  - フィールド保持のみ（バリデーション不要）
- `item_staff.character_name`: NULLable（声優役の場合のみ使用）

#### トランザクション
- `POST /items/:id/staff`時のitem_idとstaff_idの存在確認は、
  ForeignKey制約とアプリケーションレベルの両重チェック推奨
  - FK制約: DB側で自動的に拒否（integrity error）
  - アプリ側: 先にitem/staffを検索し、存在確認後INSERT（より詳細なエラーメッセージ）

### セキュリティ・パフォーマンス要件

#### セキュリティ
- **入力検証**: role/character_nameの長さ制限（role: 100, character_name: 255を上限とする）
- **SQLインジェクション**: sqlx マクロ（query!, query_as!）で自動防止済み

#### パフォーマンス
- **インデックス**: item_staff.item_id / staff_idにインデックス済み
  - DELETE時のitem_id確認クエリが高速化
- **N+1対策**: 紐付け一覧取得時（将来API追加時）はJOIN推奨

### 実装順序

#### 推奨フェーズ
1. **Red フェーズ**: テストケース定義（create_staff, create_item_staff, delete_item_staff）
2. **Green フェーズ**: ハンドラ・モデル・リポジトリ実装（テスト通過）
3. **Refactor フェーズ**: エラーハンドリング改善、コード整理

#### ファイル作成順
1. `models/staff.rs`: Staff, ItemStaff 構造体 + バリデーション関数
2. `repositories/staff_repository.rs`: create_staff, link_staff, unlink_staff
3. `handlers/staff.rs`: create_staff_handler, create_item_staff_handler, delete_item_staff_handler
4. `routes/mod.rs`: /staff, /items/:id/staff ルート追加

### 参照元:
- `docs/tasks/mediavault-backend/TASK-0020.md`
- `docs/design/mediavault-backend/architecture.md`

---

## 関連ファイル一覧（相対パス）

### 設計・要件定義
- `docs/spec/mediavault-backend/requirements.md` (REQ-009)
- `docs/design/mediavault-backend/architecture.md`
- `docs/design/mediavault-backend/api-endpoints.md` (staff endpoints)
- `docs/design/mediavault-backend/database-schema.sql` (staff tables)
- `docs/tasks/mediavault-backend/TASK-0020.md`

### 実装参考（Items CRUD）
- `backend/mediavault-api/src/handlers/items.rs`
- `backend/mediavault-api/src/models/item.rs`
- `backend/mediavault-api/src/repositories/item_repository.rs`
- `backend/mediavault-api/src/routes/mod.rs`

### プロジェクト設定
- `backend/mediavault-api/Cargo.toml`
- `backend/mediavault-api/src/main.rs`
- `backend/mediavault-api/.env` (DB接続)

---

**作成日**: 2026-06-24  
**対応タスク**: TASK-0020: スタッフ管理CRUD実装  
**開発段階**: 準備フェーズ（TDD開始前）
