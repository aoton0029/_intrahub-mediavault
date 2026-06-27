# TASK-0030: ブクログCSVインポート実装 - 開発コンテキスト

## 1. 技術スタック

### 使用技術・フレームワーク
- **言語**: Rust (Edition 2024)
- **Webフレームワーク**: Axum 0.8.9 with `multipart` feature
- **非同期ランタイム**: Tokio 1.52.3（features: full）
- **データベース**: PostgreSQL + sqlx 0.8（コンパイル時SQLチェック有効、async対応）
- **シリアライゼーション**: serde 1.0.228（derive機能）、serde_json 1.0.150
- **CSV パース**: `csv` crate（Rustの標準CSV処理ライブラリ）
- **文字コード変換**: `encoding_rs` crate（UTF-8/Shift_JISフォールバック対応用）
- **UUID**: uuid 1.0（v4, serde対応）
- **日時処理**: chrono 0.4（serde対応）
- **環境変数**: dotenvy 0.15
- **ロギング**: tracing 0.1、tracing-subscriber 0.3
- **HTTP ミドルウェア**: tower 0.5.3、tower-http 0.7.0（CORS機能）
- **テスト補助**: wiremock 0.6（HTTP模擬サーバー）、tempfile 3（一時ファイル）

### アーキテクチャパターン
- **パターン**: レイヤードアーキテクチャ + CSV専用パーサー分離設計
- **層構成**: routes → handlers → (parser) → repositories → db/sqlx
- **設計方針**: CSVパーサ部分をカラムマッピングから分離し、実サンプル確認後の調整が容易な構造を実現
  - `booklog_csv.rs` で カラムマッピング定義と行単位パース処理を実装
  - 形式不正行はスキップし、`ImportFailure` に理由を記録
  - 正常行は既存 TASK-0009 の登録ロジック経由で `items` テーブルへ登録

### 参照元
- `backend/mediavault-api/Cargo.toml`
- `docs/design/mediavault-backend/architecture.md`
- `docs/design/mediavault-backend/api-endpoints.md` (POST /import/booklog)

---

## 2. 開発ルール

### プロジェクト固有のルール

#### エラーハンドリング
- **エラー型**: `ApiError`（response.rs定義）を使用
- **エラーコード**: 統一エラーコード（VALIDATION_ERROR, INTERNAL_ERROR等）を適用
- **インポート固有**: 正常行は登録、不正行はスキップ + `ImportFailure` に記録（EDGE-002対応）
- **全行不正時**: 200で空配列を返す（例外で落ちない、TC-016-E01要件）
- **参考実装**: `backend/mediavault-api/src/repositories/item_repository.rs`の`db_error()`関数

#### Multipart アップロード処理
- **方式**: Axum の `axum::extract::Multipart`エクストラクタ を使用
- **ファイル条件**: 存在しない・空の場合は `400 Bad Request` (`VALIDATION_ERROR`)
- **文字コード**: UTF-8を前提、Shift_JIS等の可能性に対応可能な設計（`encoding_rs`利用予備）
- **参考**: 既存の `upload_item_file_handler`（`handlers/item_files.rs`）でmultipart処理例あり

#### レスポンス形式
- **成功**: `{"success": true, "data": {"success_count": N, "failure_count": M, "failures": [...]}}`
- **エラー**: `{"success": false, "error": {"code": "...", "message": "..."}}`
- **ステータスコード**: 成功時200、エラー時4xx/5xx

#### CSV行単位パーサ設計
- **1行ずつ処理**: `csv` crateのレコードイテレータで1行ずつ解析
- **必須カラム**: 作品名（title）が空・欠落時はスキップ
- **型変換失敗**: 日付・数値パース失敗時もスキップ
- **スキップ記録**: 理由文字列（例: `"title is empty"`, `"invalid date format"`）を `ImportFailure` に記録

### コード規約

#### モジュール構成
```
src/
  handlers/
    import_booklog.rs      # POST /import/booklog ハンドラ（新規）
  models/
    import.rs              # ImportSummary, ImportFailure 型定義
    response.rs            # 既存 ApiError, ApiErrorCode
  repositories/
    item_repository.rs     # 既存 create_item（再利用）
  import/ (新規ディレクトリ)
    booklog_csv.rs         # カラムマッピング + 行単位パーサ実装
    mod.rs                 # import モジュール入口
  routes/
    mod.rs                 # ルーティング定義にPOST /import/booklogを追記
```

#### ファイル名命名規則
- ハンドラファイル: `{feature}.rs` (例: `import_booklog.rs`)
- パーサモジュール: `import/booklog_csv.rs`（カラムマッピングを分離）
- テストファイル: `#[cfg(test)] mod tests` インラインに配置

#### ドキュメントコメント規約（参考実装から）
```rust
/// 【機能概要】: 簡潔な説明
/// 【実装方針】: どのように実装するか
/// 【テスト対応】: 対応するテストケース
/// 🔵/🟡 信頼性レベル: 根拠
pub async fn function_name() { ... }
```

### 参照元:
- `backend/mediavault-api/src/handlers/items.rs`
- `backend/mediavault-api/src/handlers/item_files.rs` (multipart処理例)
- `backend/mediavault-api/src/models/item.rs`
- `backend/mediavault-api/src/repositories/item_repository.rs`

---

## 3. 関連実装（参考パターン）

### 既存アイテム登録ロジックの再利用（TASK-0009）

#### Items Create（TASK-0009実装済み）
- `handlers/items.rs::create_item_handler()`
  - リクエスト検証 → `item_repository::create_item()` → レスポンス（201）
  - **ポイント**: `CreateItemRequest` の構造とバリデーションパターンを参考
  
- `models/item.rs`
  - `CreateItemRequest`: フィールド定義・デシリアライズ
  - `validate_title()`: 空文字チェック関数
  - `parse_create_item_request()`: 入力検証関数
  
- `repositories/item_repository.rs`
  - `create_item()`: トランザクション → items INSERT → detail_table INSERT
  - **ポイント**: 複数テーブル操作の トランザクション処理パターン

#### Multipart ファイルアップロード（TASK-0026参考）
- `handlers/item_files.rs::upload_item_file_handler()`
  - `axum::extract::Multipart` を受け取る実装
  - ファイルストリーム処理パターン
  - **参考点**: 本タスクのCSVファイル受け取り実装で参考にできる

### インポート結果形式（TASK-0025参考）

#### ImportItemRequest パターン
- `models/item_import.rs::ImportItemRequest`
  - 外部API結果から item を作成するDTO
  - `CreateItemRequest` への変換実装
  - **参考**: インポート専用DTO設計パターン

#### 部分更新ロジック（TASK-0012参考）
- `repositories/item_repository.rs::update_item()`
  - QueryBuilder による動的SQL生成パターン
  - 複数フィールド更新の 実装パターン

### 参照元ファイル:
- `backend/mediavault-api/src/handlers/items.rs`
- `backend/mediavault-api/src/handlers/item_files.rs`
- `backend/mediavault-api/src/models/item.rs`
- `backend/mediavault-api/src/models/item_import.rs`
- `backend/mediavault-api/src/repositories/item_repository.rs`

---

## 4. 設計文書

### API仕様

#### POST /import/booklog 🔵
- **信頼性**: 🔵 api-endpoints.md「POST /import/booklog」より
- **関連要件**: REQ-016, EDGE-002
- **説明**: ブクログのエクスポートCSVファイルをアップロード、行単位で解析・登録
- **リクエスト**: multipart/form-data（ファイルフィールド名: `file` または `csv`）
- **レスポンス（成功, 200）**: `ImportSummary`
  ```json
  {
    "success_count": 10,
    "failure_count": 2,
    "failures": [
      { "row_number": 3, "reason": "title is empty" },
      { "row_number": 7, "reason": "invalid date format" }
    ]
  }
  ```
- **エラー**:
  - `400 VALIDATION_ERROR`: ファイル未添付または0バイト
  - `500 INTERNAL_ERROR`: パース処理エラー
- **特殊要件**: 全行不正でも200で `ImportSummary` を返す（TC-016-E01）

### 仮カラムフォーマット定義（実サンプル確認待ち）🟡

**信頼性**: 🟡 prep.md「ブクログCSVのサンプルファイル準備」より実物未確認のため仮定義

仮定義カラム（後で差分調整対象）:
- `作品名` (title, **必須**): 文字列
- `感想/レビュー` (description, optional): 文字列
- `読了日` (consumed_date, optional): `YYYY-MM-DD` 形式の日付
- `評価` (rating, optional): 整数 or 小数
- `ISBN` (external_id参考情報, optional): 文字列

**実装方針**: カラムマッピングを `import/booklog_csv.rs` の `BooklogCsvRow` 構造体の `#[serde(rename = "...")]` で定義し、実サンプル入手後は **ここのマッピング定義のみを差分修正する設計** とする

### 処理フロー

1. **ファイルアップロード受け取り** (`POST /import/booklog`)
   - `Multipart` エクストラクタで CSVファイルのバイト列を取得
   - 存在確認・サイズチェック（0バイト → 400）

2. **文字コード処理**
   - UTF-8でのデコード試行
   - 失敗時は `encoding_rs` による自動変換（Shift_JIS等に対応）

3. **CSV行単位パーサ処理**
   - `csv` crate で1行ずつ読み込み
   - 各行を `BooklogCsvRow` へデシリアライズ
   - デシリアライズ失敗 → スキップ + `ImportFailure` 記録

4. **行別バリデーション**
   - 必須項目（title）の空チェック
   - 型変換（日付・数値）の検証
   - いずれかエラー → スキップ + 理由記録

5. **正常行の登録**
   - `BooklogCsvRow` を `CreateItemRequest` へ変換
   - `source` フィールドを設定（`"manual"` または専用値、設計文書確認後に決定）
   - 既存 TASK-0009 の `item_repository::create_item()` で登録

6. **レスポンス構築**
   - 成功件数・失敗件数・失敗理由を `ImportSummary` で返却
   - すべてのケースで HTTP 200

### 参照元
- `docs/design/mediavault-backend/api-endpoints.md`
- `docs/spec/mediavault-backend/acceptance-criteria.md` (TC-016-01, TC-016-E01)
- `docs/spec/mediavault-backend/prep.md` (ブクログCSVサンプル準備)

---

## 5. テスト関連情報

### テストフレームワーク・設定ファイル
- **フレームワーク**: Rust の `#[test]` / `#[tokio::test]`
- **テスト実行**: `cargo test --workspace` または `cargo test -p mediavault-api`
- **テスト環境**: Docker Compose のPostgres コンテナ（`docker compose up -d db`）
- **環境変数**: `DATABASE_URL` で接続確認
- **テスト補助**: `wiremock` 0.6（HTTP模擬）、`tempfile` 3（一時CSV ファイル作成）

### 既存テストのディレクトリ構成・命名パターン
- **場所**: 各実装ファイル末尾に `#[cfg(test)] mod tests` としてインライン配置（別ファイルなし）
- **参照例**:
  - `backend/mediavault-api/src/models/item.rs` L272-334（DTOデシリアライズテスト）
  - `backend/mediavault-api/src/handlers/items.rs` L195-350+（ハンドラユニット＋統合テスト）
  - `backend/mediavault-api/src/repositories/item_repository.rs` L815-1100（DB統合テスト）

### テストユーティリティ・モック設定
- **DB接続**: `test_pool()` ヘルパー関数（`item_repository.rs` L1076-1082）
  - `DATABASE_URL` 環境変数から接続
  - `#[tokio::test]` + `#[ignore]` で `cargo test -- --ignored` 実行対象

- **一時ファイル作成**: `tempfile` crate で一時CSVファイル生成
  - テストケース別にCSVサンプルを作成

- **CSV テストデータ**: 各テストケースで以下パターンを用意
  - 正常行のみのCSV
  - 不正行（作品名空白）を1行含むCSV
  - 全行不正なCSV
  - 空ファイル

- **エラー検証**: `unreachable_pool()` で接続不能なPgPoolを構築し、DB層エラー変換を検証

### テストケース対応マッピング

#### ユニットテスト（DB非依存、`#[test]`）
1. **CSV行パーサのユニットテスト**（import/booklog_csv.rs内）
   - 正常行デシリアライズ確認
   - 不正行エラーハンドリング確認
   - 日付フォーマット検証

2. **バリデーション関数テスト**
   - 必須項目チェック
   - 型変換エラー処理

#### 統合テスト（DB必須、`#[tokio::test]` + `#[ignore]`）
1. **TC-016-01: 正常CSVの全行登録** 🔵
   - 正常行のみのCSVを投入
   - `success_count` がCSV行数と一致確認
   - 対応する `items` レコード登録確認

2. **TC-016-02: 形式不正行のスキップ（EDGE-002）** 🔵
   - 作品名が空の行を1行含むCSVを投入
   - その行が `failures` に記録されたか確認
   - 他の正常行が登録されたか確認

3. **TC-016-E01: 全行不正でも例外にならない** 🔵
   - 全行が形式不正なCSVを投入
   - `200` が返る確認
   - `success_count=0`、`failure_count` がCSV行数と一致確認

4. **TC-016-04: 空ファイルアップロード時の400** 🟡
   - 0バイトまたは未添付のmultipartリクエスト
   - `400 Bad Request` (`VALIDATION_ERROR`) が返る確認

### テスト規約（既存方針を継続）
1. **DB非依存**: `#[test]`のみ → `cargo test -p mediavault-api`
2. **DB必須**: `#[tokio::test]` + `#[ignore]` → `cargo test -- --ignored`
3. **信頼性レベル記載**: 🔵/🟡/🔴 を各テスト関数に付与
4. **日本語コメント**: 【テスト目的】【テスト内容】【期待される動作】【確認内容】等の段落区分

### 参照元:
- `backend/mediavault-api/src/models/item.rs` L272-334
- `backend/mediavault-api/src/repositories/item_repository.rs` L815-1100
- `backend/mediavault-api/src/handlers/items.rs` L195-350+

---

## 6. 注意事項

### 技術的制約

#### カラムマッピングの分離設計 🟡
- **実施**: `booklog_csv.rs` で `BooklogCsvRow { title, description, consumed_date, rating }` 構造体を定義
- **マッピング部分**: 構造体フィールドの `#[serde(rename = "...")]` で定義
- **調整方法**: 実サンプル確認後、`rename` 値のみを差分修正（`BooklogCsvRow` の構造自体は変更不要）
- **理由**: ハンドラ・レスポンス型・スキップロジックは再利用可能にするため

#### 文字コード対応 🟡
- **前提**: UTF-8でのデコード を試行
- **フォールバック**: `encoding_rs` crate で自動変換可能な設計予備
- **実装方針**: 初段階はUTF-8のみで実装、必要に応じて後調整

#### `source` フィールド値 🟡
- **未決定**: `manual` または専用 `booklog` 値どちらを設定するか要検討
- **確認方法**: `item.rs` の既存 `ItemSource` enum値を確認
- **実装時**: 決定後に `parse_booklog_csv_row()` 内で設定

#### 全行不正時の例外回避 🔵
- **要件**: 全行が形式不正でも HTTP 200 で応答（TC-016-E01）
- **実装**: イテレータ処理中にパニックを避け、最終的に `ImportSummary` をレスポンスする
- **検証**: テスト TC-016-E01 で確認必須

### セキュリティ・パフォーマンス要件
- **ファイルサイズ制限**: 大量CSVの処理タイムアウト対策（数百行での軽い性能確認）
- **入力検証**: タイトル空文字チェック、型変換エラー処理で不正データを確実に排除
- **エラーログ**: 不正行の理由を詳細にログ（運用時の分析用）
- **トランザクション**: 行ごとのINSERTは既存repository経由で原子性確保

### 依存関係・実装順序
- **前提タスク**: TASK-0009（アイテム作成ロジック、既に完了）
  - `item_repository::create_item()` が利用可能
  - `CreateItemRequest` の構造が固定

- **後続タスク**: なし（Phase 5最終タスク群の一つ）

- **CSV crate追加**: `Cargo.toml` の `[dependencies]` に `csv = "1.3"` を追加する必要あり
  - 未検証のため、実装時に確認・追加

---

## 7. 実装チェックリスト（TDD開発用）

### Red Phase
- [ ] テストケース1～4の失敗確認（未実装なため実行不可状態）
- [ ] CSV行パーサのテストケース作成
- [ ] ハンドラテストケース作成

### Green Phase
- [ ] `handlers/import_booklog.rs` 実装
  - Multipart ファイル受け取り
  - 文字コード処理
  - CSV パーサ呼び出し
  - レスポンス構築

- [ ] `import/booklog_csv.rs` 実装
  - `BooklogCsvRow` 構造体定義（カラムマッピング含む）
  - 行単位パーサ関数
  - バリデーション関数

- [ ] `models/import.rs` 実装
  - `ImportSummary` 型定義
  - `ImportFailure` 型定義
  - Serialize/Deserialize実装

- [ ] `routes/mod.rs` 更新
  - `.route("/import/booklog", post(import_booklog_handler))` を追記
  - `/items/search`, `/items/import` より後（パスマッチ優先度）に配置

- [ ] テストケース1～4 全てがpass確認

### Refactor Phase
- [ ] CSV パーサ関数の分離・整理
- [ ] エラーメッセージの統一・改善
- [ ] ドキュメントコメント追記
- [ ] 既存repository関数の呼び出しパターン統一

### Verify Complete Phase
- [ ] 単体テスト: `cargo test -p mediavault-api`（DB非依存テスト全pass）
- [ ] 統合テスト: `cargo test -- --ignored`（実DB接続テスト全pass）
- [ ] TC-016-01/E01 の実データ流し込み確認
- [ ] 不正行の理由文字列が正確に記録されているか確認
- [ ] 数百行CSVでのタイムアウト確認

---

## 補足：プロジェクト全体の補助情報

### プロジェクト構成
- **リポジトリ**: `https://github.com/yourusername/MediaVault`
- **プロジェクトルート**: `backend/`
- **APIパッケージ**: `backend/mediavault-api/`
- **DB**: PostgreSQL (Docker Compose)
- **環境ファイル**: `.env`（DATABASE_URL, INTERNAL_API_KEY等）

### 構成ファイル・ディレクトリ
- `backend/mediavault-api/src/handlers/` - HTTP ハンドラ群
- `backend/mediavault-api/src/models/` - DTOと型定義
- `backend/mediavault-api/src/repositories/` - DB操作層
- `backend/mediavault-api/src/import/` - **本タスク新規追加** CSVパーサ
- `backend/mediavault-api/src/routes/` - ルーティング定義
- `docs/spec/mediavault-backend/` - 要件・仕様定義
- `docs/design/mediavault-backend/` - 技術設計文書
- `docs/implements/mediavault-backend/TASK-0030/` - 本タスクの開発ノート

### 既存実装との関連性
- **TASK-0009**: アイテム作成ロジック（既完了）- 本タスクで `create_item()` を再利用
- **TASK-0025**: 外部API検索結果インポート（既完了）- `ImportItemRequest` パターン参考
- **TASK-0026**: ファイルアップロード（既完了）- Multipart処理参考
- **TASK-0029**: 内部REST API ルート（既完了）- インポートは外部API対象外

### 追加ルール・ガイドライン
- **AGENTS.md**: このリポジトリに存在しない
- **docs/rule/ ディレクトリ**: このリポジトリに存在しない
- **追加ルール**: なし（既存の開発ルールに従う）

### 進行状況
- **Phase 5 状況**: Phase 5は「内部API・インポート」フェーズ
  - TASK-0029（内部REST APIルート）完了
  - TASK-0030（ブクログCSVインポート）本実装開始
  - TASK-0031（Steamライブラリインポート）は後続

### 確認待ちの外部情報
- **実サンプル**: prep.md に記載の「ブクログCSVサンプル」入手待ち
  - カラムフォーマット確定後、`BooklogCsvRow` の rename値を調整
  - 実装は仮定値で進行可能（テスト用CSVは自作）
