# TASK-0027: POST /items/:id/files/upload（バイナリ直接アップロード） - 開発コンテキスト

## 1. 技術スタック

### 使用技術・フレームワーク
- **言語**: Rust (Edition 2024)
- **Webフレームワーク**: Axum 0.8.9
  - **Multipart処理**: `axum::extract::Multipart`（標準Axum extract）
- **非同期ランタイム**: Tokio 1.52.3（features: full）
  - **ファイルI/O**: `tokio::fs` による非同期ストリーミング書込
- **データベース**: PostgreSQL + sqlx 0.8（コンパイル時SQLチェック有効、async対応）
- **シリアライゼーション**: serde 1.0.228（derive機能）、serde_json 1.0.150
- **UUID**: uuid 1.0（v4, serde対応）
- **日時処理**: chrono 0.4（serde対応）
- **環境変数**: dotenvy 0.15
  - **用途**: ファイル保存先ベースディレクトリの設定（開発環境: 相対パス、本番: `/srv/files/pdf`, `/srv/media/photos`）
- **ロギング**: tracing 0.1、tracing-subscriber 0.3
- **HTTP ミドルウェア**: tower 0.5.3、tower-http 0.7.0（CORS機能）

### アーキテクチャパターン
- **パターン**: レイヤードアーキテクチャ（TASK-0026継続）
- **層構成**: routes → handlers → services → repositories → db/sqlx + file_storage
- **新規層**: `services/file_storage.rs`（ファイル書込・ロールバック処理を集約）
- **設計方針**:
  - ハンドラはMultipart受信・リクエストボディ解析のみ担当
  - ファイル書込はサービス層に集約し、トランザクション的一貫性を確保
  - リポジトリ層は既存TASK-0026の`item_file_repository`を再利用

### 参照元:
- `backend/mediavault-api/Cargo.toml`
- `docs/design/mediavault-backend/architecture.md`
- `docs/design/mediavault-backend/api-endpoints.md` (L307-318)
- `docs/design/mediavault-backend/dataflow.md` (L125-160)

---

## 2. 開発ルール

### プロジェクト固有のルール（TASK-0026の継続）

#### エラーハンドリング
- **エラー型**: `ApiError`（response.rs定義）を使用
- **新規エラーコード**: `FileStorageWriteFailed`（500 INTERNAL_SERVER_ERROR、REQ-019/TC-019-E01対応）
  - 必要に応じて`response.rs`のApiErrorCode enumに追加
- **DB層でのエラー処理**: sqlx::Errorを統一的に`ApiError`に変換
  - 詳細はtracing::error!でサーバーログのみに出力
- **ファイルI/O エラー処理**: std::io::Error → ApiError（FileStorageWriteFailed）への変換
  - ディスク容量不足、権限エラー、パストラバーサル試行等はサーバーログに詳細記録、クライアントには「ファイル書込に失敗しました」の一般メッセージ
- **参考実装**: `backend/mediavault-api/src/repositories/item_file_repository.rs`の`db_error()`関数パターン

#### レスポンス形式（TASK-0026の継続）
- **成功**: `{"success": true, "data": {...}}` (ApiOk構造体)
- **エラー**: `{"success": false, "error": {"code": "...", "message": "..."}}`
- **ステータスコード**: 作成時201、エラー時4xx/5xx

#### ファイル保存要件（REQ-402, REQ-104）
- **保存先ディレクトリ分岐**:
  - `file_type="pdf"` → `/srv/files/pdf`（または環境変数 `PDF_STORAGE_PATH`）
  - `file_type="photo"`等 → `/srv/media/photos`（または環境変数 `MEDIA_STORAGE_PATH`）
- **相対パス保存**: `item_files.path` には配置先ベースディレクトリからの相対パスのみを保存
  - 例: `/srv/files/pdf`配下に置いたファイル → `path = "2025-01-15/uuid-filename.pdf"`
- **一意なファイル名生成**: UUID + 元の拡張子から生成（パストラバーサル・名前衝突防止）
  - 例: `{uuid}.pdf`, `{uuid}.jpg`, `{uuid}.png`
- **環境変数設定（開発環境対応）**:
  - `PDF_STORAGE_PATH`: デフォルト `/srv/files/pdf`
  - `MEDIA_STORAGE_PATH`: デフォルト `/srv/media/photos`
  - 開発環境では`./test_files/pdf`, `./test_files/photos` 等に切替可能

#### トランザクション・ロールバック処理（EDGE-003）
- **実装パターン**: 書込 → DB登録の順（write-then-record）
- **書込失敗時**: DBレコード作成をスキップ、FILE_STORAGE_WRITE_FAILEDエラーを返す
- **DB登録失敗時**: 書き込んだファイルを削除（クリーンアップ処理）し、エラーを返す
  - リポジトリ層で`create_item_file()`失敗時、ハンドラが`tokio::fs::remove_file()`でロールバック
- **テスト時**: フェイク実装でファイルサーバー書込失敗をシミュレート

### コード規約（TASK-0026の継続）

#### モジュール構成
```
src/
  handlers/
    item_files.rs        # POST /items/:id/files/upload ハンドラ（更新）
  models/
    item_file.rs         # 既存TASK-0026（バリデーション再利用）
    ...
  repositories/
    item_file_repository.rs # 既存TASK-0026（create_item_file再利用）
    ...
  services/
    file_storage.rs      # 【新規】ファイル書込・パス生成・ロールバック処理集約
    ...
  routes/
    mod.rs               # ルーティング（既存に統合）
```

#### ファイル名命名規則
- **ハンドラ**: `item_files.rs` （TASK-0026から継続）
- **モデル**: `item_file.rs` （TASK-0026から継続）
- **サービス**: `file_storage.rs` （本タスクで新規）
- **リポジトリ**: `item_file_repository.rs` （TASK-0026から継続）

#### ドキュメントコメント規約
```rust
/// 【機能概要】: 簡潔な説明
/// 【実装方針】: どのように実装するか
/// 【テスト対応】: 対応するテストケース
/// 🔵/🟡 信頼性レベル: 根拠
pub async fn function_name() { ... }
```

#### 型定義パターン（TASK-0026から継続）
- **Enum**: `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]`
- **構造体**: `#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]`

### 参照元:
- `backend/mediavault-api/src/handlers/item_files.rs` （TASK-0026）
- `backend/mediavault-api/src/models/item_file.rs` （TASK-0026）
- `backend/mediavault-api/src/repositories/item_file_repository.rs` （TASK-0026）
- `docs/implements/mediavault-backend/TASK-0020/note.md` （開発ルール参考）

---

## 3. 関連実装

### TASK-0026 既存実装（基盤として再利用）
- **ハンドラ**: `src/handlers/item_files.rs`
  - `create_item_file_handler()`: パス指定方式、すでに実装済み
  - 本タスクで新規に`upload_item_file_handler()`を追加
- **モデル**: `src/models/item_file.rs`
  - `FileType` enum: `Pdf`, `Image`, `Other` （再利用）
  - `ItemFile` struct: レスポンス型（再利用）
  - `CreateItemFileRequest`: リクエスト検証済み型（再利用パターン）
- **リポジトリ**: `src/repositories/item_file_repository.rs`
  - `create_item_file()`: item存在確認 + INSERT ロジック（再利用）
  - `db_error()`: DB層エラーハンドリングパターン（再利用）

### Axum Multipart処理パターン（新規）
- **参考リソース**: Axum 0.8.9公式ドキュメント - Multipart extraction
- **実装パターン**:
  ```rust
  async fn upload_handler(
      Path(item_id): Path<Uuid>,
      State(state): State<AppState>,
      multipart: Multipart,
  ) -> Result<Response, ApiError> {
      let (file_bytes, file_type, label) = parse_multipart(multipart).await?;
      // ...
  }
  ```

### トランザクション / クリーンアップパターン
- **参考実装**: TASK-0020 スタッフ管理（複数テーブル同時更新時のトランザクション例）
  - パターン: `pool.begin().await` → 複数操作 → `tx.commit()`
- **本タスクでの応用**: ファイル書込 → DB登録の順で処理、DB登録失敗時に書込済みファイル削除

### 類似機能のファイル処理（参考）
- **インポート機能** (TASK-0025): multipart/form-data での大容量データ受取・パース
  - `src/models/item_import.rs` の multipart 処理パターン参考
- **APIキー管理** (TASK-0022): リクエストボディバリデーション
  - `parse_*` 関数の命名規約・エラーハンドリングパターン参考

### 参照元:
- `backend/mediavault-api/src/handlers/item_files.rs`
- `backend/mediavault-api/src/models/item_file.rs`
- `backend/mediavault-api/src/repositories/item_file_repository.rs`
- `docs/tasks/mediavault-backend/TASK-0026.md`

---

## 4. 設計文書

### API仕様（TASK-0027対応箇所）
- **エンドポイント**: `POST /items/:id/files/upload` 🔵
- **リクエスト**: `multipart/form-data`
  - **フィールド**:
    - `file` (required, binary): アップロードするバイナリファイル
    - `file_type` (required, enum): `pdf`, `image`, `other`
    - `label` (optional, string): ファイルのラベル（TASK-0026と同一）
- **レスポンス（成功, 201）**: ItemFile型（TASK-0026と同一）
- **エラーコード**:
  - `FILE_STORAGE_WRITE_FAILED` (500): ファイル書込失敗時 (TC-019-E01)
  - `ITEM_NOT_FOUND` (404): item_id が存在しない場合
  - `VALIDATION_ERROR` (400): file_type が不正値の場合

### データモデル（既存継続）
- **テーブル**: `item_files` （TASK-0026で定義済み）
  - `id` (UUID, PK)
  - `item_id` (UUID, FK → items.id)
  - `path` (VARCHAR): 相対パス（**本タスクで新たに相対パス記載ロジック追加**）
  - `label` (VARCHAR, nullable)
  - `file_type` (ENUM: pdf, image, other)
  - `calibre_book_id` (VARCHAR, nullable)：後続TASK-0028で使用
  - `created_at` (TIMESTAMP)

### ファイルサーバー構成（REQ-402/104）
```
ホストマシン
├── /srv/files/pdf/           （コンテナ側：PDF保存）
│   ├── 2025-01-15/
│   │   └── {uuid}.pdf
│   └── ...
├── /srv/media/photos/        （コンテナ側：画像保存）
│   ├── 2025-01-15/
│   │   └── {uuid}.jpg
│   └── ...

開発環境（.env指定）
├── ./test_files/pdf/
└── ./test_files/photos/
```

### 参照元:
- `docs/design/mediavault-backend/api-endpoints.md` (L307-318)
- `docs/design/mediavault-backend/dataflow.md` (L125-160)
- `docs/design/mediavault-backend/database-schema.sql` (item_files テーブル定義)
- `docs/tasks/mediavault-backend/TASK-0027.md`

---

## 5. テスト関連情報

### テストフレームワーク・設定
- **単体テスト**: `#[tokio::test]` / `#[test]`（Rust標準 + tokio-macros）
- **統合テスト**: `#[tokio::test]` + `#[ignore]` （DATABASE_URL環境変数必須）
- **テストヘルパー**: `test_pool()`, `test_app_state()` 関数（既存TASK-0026継続）
  - 用途: DB接続 + AppState構築

### 既存テストのディレクトリ構成・命名パターン
```
src/
  handlers/
    item_files.rs
      #[cfg(test)]
        mod tests:
          - post_item_file_with_existing_item_returns_201() （TC-007-01）
          - post_item_file_with_nonexistent_item_returns_404() （TC-007-02）
          - post_item_file_with_empty_path_returns_400() （TC-007-04）
          - post_item_file_with_invalid_file_type_returns_400() （TC-007-03）
  models/
    item_file.rs
      #[cfg(test)]
        mod tests:
          - create_item_file_request_deserializes_valid_fields()
          - parse_create_item_file_request_rejects_empty_path()
          - ...
  repositories/
    item_file_repository.rs
      #[cfg(test)]
        mod tests:
          - create_item_file_with_nonexistent_item_returns_item_not_found()
```

### 既存テストユーティリティ・パターン
- **DB接続テスト**: `test_pool()` 関数（item_file_repository.rs内）
  - DATABASE_URL環境変数から接続文字列取得
  - `#[ignore]` で統合テスト扱い（開発環境でのみ実行）
- **ハンドラテスト**: `test_app_state()` + `build_router()` （handlers/item_files.rs内）
  - Axum の `tower::ServiceExt` を用いたリクエスト・レスポンス検証
- **モックの使用**: wiremock 0.6 （dev-dependencies）
  - 外部APIモック（TASK-0024等で使用）、本タスクでは未使用想定

### 本タスクで新規に必要なテスト関連実装
- **ファイルサーバー書込テスト**: 一時ディレクトリ (`std::env::temp_dir()` 等) を用いた実ファイル書込テスト
  - テスト実行後のクリーンアップ: `std::fs::remove_file()`, `std::fs::remove_dir_all()`
- **Multipart解析テスト**: `axum::body::to_bytes()` + serde_json 検証
  - または既存の TASK-0025 (item_import) のmultipart テストパターン参考

### テストケース洗い出し（TASK-0027仕様より）
1. **TC-019-01**: 正常系 - バイナリアップロード（file_type=pdf）でファイル配置・パス保存 → 201
2. **TC-019-02**: file_type=photo 等で `/srv/media/photos` 配下に配置 → 201
3. **TC-019-E01 (EDGE-003)**: ファイル書込失敗時ロールバック → FILE_STORAGE_WRITE_FAILED (500)
4. **TC-019-03**: item_id不存在 → ITEM_NOT_FOUND (404)
5. **統合TC-1**: 書込→DB登録の一貫性確認（正常系）
6. **統合TC-2**: DB登録失敗時のファイルクリーンアップ確認

### 参照元:
- `backend/mediavault-api/src/handlers/item_files.rs` (tests モジュール)
- `backend/mediavault-api/src/models/item_file.rs` (tests モジュール)
- `backend/mediavault-api/src/repositories/item_file_repository.rs` (tests モジュール)
- `docs/implements/mediavault-backend/TASK-0020/staff-testcases.md` （テストケース洗い出し参考）
- `Cargo.toml` (dev-dependencies: wiremock)

---

## 6. 注意事項

### 技術的制約
- **大容量ファイル対応**: Axum のリクエストボディサイズ上限確認・調整（デフォルト 2MB程度）
  - `.layer(DefaultBodyLimit::max(/* サイズ */))` で設定可能
- **ストリーミング書込**: メモリ全展開を避けるため、tokio::fs でストリーミング処理推奨
- **並行アップロード**: Tokio の full features で複数並行処理対応

### セキュリティ・パフォーマンス要件（REQ-402, REQ-104, EDGE-003）
- **パストラバーサル防止**: クライアント指定のファイル名をそのまま使用せず、UUID + 拡張子で生成
- **ファイル名衝突防止**: UUID 一意性で保証
- **コンテナ内ファイル保持禁止**: 本体はバインドマウント先のみに保存、コンテナ内に残さない
  - 実装時: `/srv/*` 直下にのみ書込、`.` (アプリホームディレクトリ) への書込禁止
- **トランザクション一貫性**: 書込失敗時のレコード未作成、DB失敗時のファイルクリーンアップで確保

### 環境変数設定（開発環境対応）
- **必須**: `DATABASE_URL`（既存）
- **本タスク新規**: 
  - `PDF_STORAGE_PATH` （デフォルト: `/srv/files/pdf`）
  - `MEDIA_STORAGE_PATH` （デフォルト: `/srv/media/photos`）
- **開発環境**: `.env` で `./test_files/pdf`, `./test_files/photos` に切替

### 参照元:
- `docs/tasks/mediavault-backend/TASK-0027.md` (注意事項セクション)
- `docs/design/mediavault-backend/dataflow.md` (EDGE-003 データ整合性の保証)
- `docs/spec/mediavault-backend/requirements.md` (REQ-402, REQ-104)
