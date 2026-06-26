# TASK-0027 要件定義書: POST /items/:id/files/upload（バイナリ直接アップロード）

- **機能名**: item-file-upload（multipart/form-data バイナリ直接アップロード）
- **タスクID**: TASK-0027
- **要件名**: mediavault-backend
- **フェーズ**: Phase 4 - ファイル管理・拡張機能
- **作成日**: 2026-06-26
- **出力ファイル**: `docs/implements/mediavault-backend/TASK-0027/item-file-upload-requirements.md`

---

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

### 何をする機能か
- 🔵 `POST /items/:id/files/upload` エンドポイントを実装し、`multipart/form-data` でバイナリファイル本体を直接受け取る。
  - 根拠: `TASK-0027.md` タスク概要・完了条件 L24、`api-endpoints.md` POST /items/:id/files/upload。
- 🔵 受け取ったファイルを `file_type` に応じてファイルサーバー上のディレクトリ（`pdf` → `/srv/files/pdf`、画像系 → `/srv/media/photos`）へ書き込み、配置後の**相対パス**を `item_files.path` に保存して 201 でレコードを返す。
  - 根拠: `TASK-0027.md` 完了条件 L25-26、`note.md` L56-67。
- 🔵 ファイル本体はアプリコンテナ内に保存してはならず（REQ-402）、バインドマウントされたファイルサーバー上のディレクトリにのみ書き込む。
  - 根拠: `TASK-0027.md` L17・注意事項 L106、`note.md` L296-297。

### どのような問題を解決するか
- 🔵 TASK-0026 の「パス指定方式」（既にファイルサーバー上に存在するパスを登録）に対し、本タスクは「クライアントが保持するバイナリを直接アップロードして配置する」ユースケースを補完する。
  - 根拠: `TASK-0027.md` 依存タスク L20、`note.md` L125-135。
- 🟡 アプリコンテナにファイル本体を残さず、ストレージとアプリの責務を分離することで、コンテナの揮発性・スケールアウト時のデータ整合性問題を回避する（REQ-402）。
  - 根拠: REQ-402 からの妥当な推測。

### 想定されるユーザー
- 🟡 MediaVault のフロントエンド／API クライアント（ファイル本体をローカルに持ち、サーバーへ転送したいユーザー）。
  - 根拠: バックエンドAPIのみのプロジェクト（`TASK-0027.md` UI/UX要件 L98）からの妥当な推測。

### システム内での位置づけ
- 🔵 レイヤードアーキテクチャ（routes → handlers → services → repositories → db/file_storage）。本タスクで `services/file_storage.rs` を新設し、ファイル書込・パス生成・ロールバックを集約する。リポジトリ層は TASK-0026 の `item_file_repository` を再利用する。
  - 根拠: `note.md` L20-28・L88-94。

- **参照したEARS要件**: REQ-019/REQ-104（保存先分岐・相対パス保存）、REQ-402（コンテナ内非保存）、EDGE-003（書込/DB整合性）
- **参照した設計文書**: `api-endpoints.md`（POST /items/:id/files/upload, L307-318）、`dataflow.md`（機能5シーケンス・データ整合性の保証, L125-160）、`architecture.md`（レイヤード構成）

---

## 2. 入力・出力の仕様（EARS機能要件・Rust型定義ベース）

### 入力（HTTPリクエスト）
- 🔵 メソッド・パス: `POST /items/:id/files/upload`
  - `:id` … 対象 item の UUID（パスパラメータ）。`parse_item_id()` で検証。
- 🔵 Content-Type: `multipart/form-data`
- 🔵 multipart フィールド:
  | フィールド | 必須 | 型 | 説明 |
  |---|---|---|---|
  | `file` | 必須 | binary | アップロードするファイル本体（元ファイル名を含む） |
  | `file_type` | 必須 | enum | `pdf` / `image` / `other`（既存 `FileType` enum、`models/item_file.rs` 準拠） |
  | `label` | 任意 | string | ファイルのラベル（TASK-0026 と同一、nullable） |
  - 根拠: `note.md` L172-184、`models/item_file.rs` `FileType`/`ItemFile`。
  - 🟡 注記（要確認）: `TASK-0027.md` 本文・テストケース2 では `file_type="photo"` と記載されているが、既存 `FileType` enum（`models/item_file.rs` L16-20）は `Pdf`/`Image`/`Other` であり `photo` は存在しない。本要件では**API値は既存 enum の `image` を正**とし、`photo` は同義の表記ゆれとみなす。実装時に enum を拡張せず `image` を用いる方針（要レビュー）。

### 出力（HTTPレスポンス）
- 🔵 成功時（201 Created）: 統一レスポンス `{"success": true, "data": ItemFile}`
  - `ItemFile`（`models/item_file.rs`）: `{ id, item_id, path, label, file_type, calibre_book_id, created_at }`
  - `path` は**配置先ベースディレクトリからの相対パス**（例: `2025-01-15/{uuid}.pdf`）。絶対パスのベース部分はアプリ設定として分離する。
  - 根拠: `note.md` L60-62・L179、`response.rs` `ApiOk`、`item_files.rs` `created_response`（201固定）。
- 🔵 エラー時: 統一エラー `{"success": false, "error": {"code": "...", "message": "..."}}`（`ApiError`）
  | コード | HTTP | 発生条件 |
  |---|---|---|
  | `VALIDATION_ERROR` | 400 | `file_type` が不正値、`file` フィールド欠落等 |
  | `ITEM_NOT_FOUND` | 404 | `:id` の item が存在しない |
  | `FILE_STORAGE_WRITE_FAILED` | 500 | ファイルサーバーへの書込失敗（TC-019-E01, EDGE-003） |
  - 根拠: `TASK-0027.md` 完了条件 L27・単体テスト要件、`note.md` L180-183。
  - 🟡 注記（実装必須）: `FILE_STORAGE_WRITE_FAILED` は現状 `response.rs` の `ApiErrorCode` enum に**未定義**。本タスクで新規 variant `FileStorageWriteFailed`（`"FILE_STORAGE_WRITE_FAILED"` / 500 INTERNAL_SERVER_ERROR）を追加する必要がある。`note.md` L43 と整合。
  - 🟡 クライアントへ返すメッセージは「ファイル書込に失敗しました」等の一般メッセージとし、I/O エラー詳細（パス・権限・容量等）はサーバーログ（`tracing::error!`）にのみ出力する（情報漏洩防止）。根拠: `note.md` L47-48。

### 入出力の関係性 / データフロー
- 🔵 multipart 解析（handler） → file_type 検証 → item 存在確認 → 一意ファイル名生成 → **ファイル書込（service）** → 成功確認後に **`item_files` INSERT（repository）** → 相対パスを含む `ItemFile` を 201 で返却。
  - 根拠: `dataflow.md` 機能5シーケンス（L125-160）、`note.md` L70-72（write-then-record）。

- **参照したEARS要件**: REQ-019, REQ-104
- **参照した設計文書**: `api-endpoints.md`（L307-318）、`models/item_file.rs`（`FileType`, `ItemFile`, `CreateItemFileRequest`）、`response.rs`（`ApiOk`, `ApiError`, `ApiErrorCode`）、`database-schema.sql`（item_files）

---

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

### パフォーマンス要件
- 🟡 大容量ファイル（PDF等）のアップロードを想定し、Axum の multipart ボディサイズ上限（デフォルト 2MB 程度）を `DefaultBodyLimit::max(...)` で調整する。上限値は実装時に決定（要レビュー）。
  - 根拠: `TASK-0027.md` 注意事項 L108、`note.md` L288。
- 🟡 メモリ全展開を避けるため、`tokio::fs` によるストリーミング書込を検討する。
  - 根拠: `TASK-0027.md` L107、`note.md` L290。

### セキュリティ要件
- 🔵 パストラバーサル・名前衝突防止: クライアント指定のファイル名をそのまま使わず、サーバー側で一意なファイル名（UUID + 元拡張子、例 `{uuid}.pdf`）を生成する。
  - 根拠: `TASK-0027.md` 完了条件 L29・注意事項 L109、`note.md` L294-295。
- 🔵 コンテナ内ファイル保持禁止（REQ-402）: 書込先は `/srv/*`（または環境変数で指定したパス）のみとし、アプリホームディレクトリ（`.`）への書込は禁止。
  - 根拠: `note.md` L296-297。
- 🟡 拡張子の扱い: 元ファイル名から拡張子を抽出する際も、パス区切り文字・親ディレクトリ参照（`..`）を含む値を弾く／正規化する（要レビュー）。
  - 根拠: パストラバーサル防止方針からの妥当な推測。

### 互換性要件（MUST）
- 🔵 REQ-402: ファイル本体はアプリコンテナ内に保存しない（バインドマウント先のみ）。
- 🔵 既存レスポンス規約（`{success, data}` / `{success, error:{code,message}}`）、201 固定、`FileType` enum、`item_files` スキーマを TASK-0026 から継続する。
  - 根拠: `note.md` L51-54・L185-193。

### アーキテクチャ制約
- 🔵 ハンドラは multipart 受信・解析のみ担当。ファイル書込・パス生成・ロールバックは `services/file_storage.rs` に集約。リポジトリ層は `item_file_repository::create_item_file()` を再利用（重複実装しない）。
  - 根拠: `note.md` L24-28・L86-94・L133-135。

### データベース制約
- 🔵 `item_files` テーブル（TASK-0026 定義済み）: `id`(UUID PK), `item_id`(UUID FK→items.id), `path`(VARCHAR, 相対パス), `label`(nullable), `file_type`(ENUM pdf/image/other), `calibre_book_id`(nullable, TASK-0028 用), `created_at`(TIMESTAMP)。
  - 🔵 `item_id` は FK 制約あり。存在しない item への INSERT は `create_item_file()` 内で `ITEM_NOT_FOUND` に変換される（TASK-0026 既存挙動）。
  - 根拠: `note.md` L185-193、`item_file_repository.rs`。

### API制約 / 環境変数
- 🔵 書込先ベースディレクトリは環境変数で設定可能とする:
  - `PDF_STORAGE_PATH`（デフォルト `/srv/files/pdf`）
  - `MEDIA_STORAGE_PATH`（デフォルト `/srv/media/photos`）
  - 開発・テスト環境では `./test_files/pdf`, `./test_files/photos` 等のローカルディレクトリへ切替可能。
  - 根拠: `note.md` L64-67・L300-305、`TASK-0027.md` 注意事項 L106。

- **参照したEARS要件**: REQ-402（MUST）, REQ-104, NFR（ボディサイズ・ストリーミング）, EDGE-003
- **参照した設計文書**: `architecture.md`（レイヤード構成・services層）、`database-schema.sql`（item_files）、`api-endpoints.md`、`dataflow.md`（データ整合性の保証）

---

## 4. 想定される使用例（EARS Edgeケース・データフローベース）

### 基本的な使用パターン（通常要件）
- 🔵 **正常系1（TC-019-01）**: `file_type="pdf"` + 有効バイナリ → `/srv/files/pdf` 配下に配置、相対パスを `item_files.path` に保存、201 で `ItemFile` を返す。
  - 根拠: `TASK-0027.md` テストケース1 L65-68。
- 🔵 **正常系2（TC-019-02）**: `file_type="image"`（spec表記 `photo`）→ `/srv/media/photos` 配下に配置、201。
  - 根拠: `TASK-0027.md` テストケース2 L70-73。

### データフロー
- 🔵 multipart 受信 → 解析 → file_type 検証 → item 存在確認 → 一意名生成 → 書込 → 書込成功確認 → DB INSERT → 201。書込より先に DB INSERT を行わない（順序保証）。
  - 根拠: `TASK-0027.md` 完了条件 L28、`dataflow.md` L125-160。

### エッジケース / エラーケース
- 🟡 **EDGE-003: 書込失敗ロールバック（TC-019-E01）**: ディスク容量不足・権限エラー等で書込に失敗 → `item_files` レコードを作成せず `FILE_STORAGE_WRITE_FAILED`（500）を返す。テストはフェイク実装で書込失敗を注入する。
  - 根拠: `TASK-0027.md` テストケース3 L75-78・完了条件 L27、`note.md` L69-74。
- 🟡 **EDGE-003 対称: DB登録失敗時のファイルクリーンアップ**: 書込成功後に DB INSERT が失敗（不正 item_id 等）した場合、書き込んだファイルを `tokio::fs::remove_file()` で削除し、部分失敗状態を残さない。
  - 根拠: `TASK-0027.md` 統合テストケース2 L92-94、`note.md` L71-73・L298。
- 🔵 **item_id 不存在（TC-019-03）**: 存在しない `item_id` → `ITEM_NOT_FOUND`（404）。理想的には書込前に item 存在確認を行い、無駄な書込を避ける（仕様: 「ファイル書込は行わない」）。
  - 根拠: `TASK-0027.md` テストケース4 L80-83。
- 🟡 **file_type 不正値**: enum 外の値 → `VALIDATION_ERROR`（400）。書込は行わない。
  - 根拠: `note.md` L182、`models/item_file.rs` のデシリアライズ失敗パターンからの妥当な推測。
- 🟡 **`file` フィールド欠落 / 空ファイル**: 必須フィールド欠落は `VALIDATION_ERROR`（400）。空バイト列の扱いは実装時に決定（要レビュー）。
  - 根拠: multipart 必須フィールド仕様からの妥当な推測。
- 🟡 **統合: 書込→DB登録の一貫性（統合TC-1）**: 一時ディレクトリをファイルサーバー代替に用い、(a) 正常系でファイルと DB レコードが両方存在、(b) 書込失敗注入時にファイルも DB レコードも残らないことを確認。
  - 根拠: `TASK-0027.md` 統合テストケース1 L88-90。

- **参照したEARS要件**: EDGE-003
- **参照した設計文書**: `dataflow.md`（機能5シーケンス・データ整合性の保証）

---

## 5. EARS要件・設計文書との対応関係

- **参照したユーザストーリー**: ファイルアップロード（クライアントがバイナリを直接転送して item に紐付ける）
- **参照した機能要件**: REQ-019（バイナリアップロード）、REQ-104（保存先分岐・相対パス保存）
- **参照した非機能要件**: REQ-402（コンテナ内非保存・MUST）、ボディサイズ上限/ストリーミング（NFR、`TASK-0027.md` 注意事項）
- **参照したEdgeケース**: EDGE-003（書込/DB登録の整合性・ロールバック対称性）
- **参照した受け入れ基準**:
  - TC-019-01（正常系 pdf 配置・パス保存 201）
  - TC-019-02（image/photo → /srv/media/photos 201）
  - TC-019-E01（書込失敗 → FILE_STORAGE_WRITE_FAILED 500）
  - TC-019-03（item_id 不存在 → ITEM_NOT_FOUND 404）
  - 統合TC-1（書込↔DBレコード一貫性）／統合TC-2（DB失敗時ファイルクリーンアップ）
- **参照した設計文書**:
  - **アーキテクチャ**: `architecture.md`（routes→handlers→services→repositories、services/file_storage 新設）
  - **データフロー**: `dataflow.md`（機能5シーケンス L125-160・データ整合性の保証）
  - **型定義**: `src/models/item_file.rs`（`FileType`, `ItemFile`, `CreateItemFileRequest`）、`src/models/response.rs`（`ApiOk`, `ApiError`, `ApiErrorCode`）
  - **データベース**: `database-schema.sql`（item_files テーブル）
  - **API仕様**: `api-endpoints.md`（POST /items/:id/files/upload, L307-318）
- **既存実装（再利用対象）**:
  - `src/handlers/item_files.rs`（`create_item_file_handler` に並べて `upload_item_file_handler` を追加）
  - `src/repositories/item_file_repository.rs`（`create_item_file()` 再利用）
  - `src/models/item_file.rs`（`FileType` 検証再利用）
- **本タスクで新規追加**:
  - `src/services/file_storage.rs`（書込・一意名生成・相対パス算出・ロールバック）
  - `ApiErrorCode::FileStorageWriteFailed`（`"FILE_STORAGE_WRITE_FAILED"` / 500）を `src/models/response.rs` に追加

---

## 6. 未確定事項・実装時に確認すべき点

1. 🟡 **file_type の表記ゆれ（`photo` vs `image`）**: spec本文は `photo`、既存 enum は `image`。本要件は `image` を正とする方針。確定要レビュー。
2. 🟡 **相対パスのサブディレクトリ構成**: `note.md` の例は日付ディレクトリ（`2025-01-15/{uuid}.ext`）。日付ディレクトリを採用するか UUID 直下にするか実装時に確定。
3. 🟡 **multipart ボディサイズ上限の具体値**: `DefaultBodyLimit` の設定値（PDF 想定の上限）。
4. 🟡 **空ファイル・空 `file_type` の扱い**: 400 とするか許容するか。
5. 🟡 **item 存在確認のタイミング**: 書込前に確認して無駄な I/O を避ける（推奨）か、`create_item_file()` の FK エラーに委ねるか。仕様「item不存在時は書込を行わない」に従い前者を推奨。

---

## 品質判定

- 要件の曖昧さ: 一部あり（第6章に5項目を明示。いずれも実装方針レベルで致命的ではない）
- 入出力定義: 完全（HTTP I/O・エラーコード・型・相対パス規約を明記）
- 制約条件: 明確（REQ-402・EDGE-003・環境変数・アーキテクチャ制約を網羅）
- 実装可能性: 確実（既存基盤の再利用ポイントと新規追加点を特定済み）
- 信頼性レベル分布: 🔵 多数（概要・I/O骨子・主要制約）、🟡 中程度（ロールバック挙動・サイズ/ストリーミング・表記ゆれ）、🔴 なし

**総合評価**: 高品質（⭕）。ロールバック関連と表記ゆれは🟡中心のため、tdd-testcases / tdd-red 前に第6章の5項目を確認することを推奨。
