# TDD要件定義書: TASK-0028 PATCH /items/:id/files/:file_id/calibre-link

**機能名**: calibre-link（item_files の calibre_book_id 更新 + アイテム詳細APIへのCalibre-Web遷移情報付加）
**タスクID**: TASK-0028
**要件名**: mediavault-backend
**出力ファイル**: `docs/implements/mediavault-backend/TASK-0028/calibre-link-requirements.md`
**作成日**: 2026-06-26

> **信頼性レベル凡例**
> - 🔵 **青信号**: タスク定義書・api-endpoints.md・database-schema.sql・既存実装を参照し、ほぼ推測なし
> - 🟡 **黄信号**: 上記資料からの妥当な推測
> - 🔴 **赤信号**: 資料に明確な記載がなく推測を含む

---

## 1. 機能の概要（タスク定義書・設計文書ベース）

- 🔵 **何をする機能か**: `PATCH /items/:id/files/:file_id/calibre-link` エンドポイントを実装し、`file_type=pdf` の `item_files` レコードに対して `calibre_book_id` を更新する。
- 🔵 **どのような問題を解決するか**: Calibre-Web 側で PDF の取込が完了したあと、MediaVault 側の該当 `item_files` レコードに Calibre-Web 上の書籍ID（`calibre_book_id`）を関連付け、後続でアイテム詳細から Calibre-Web へ遷移できるようにする（REQ-103）。
- 🔵 **想定されるユーザー**: Calibre-Web 取込完了後にこのエンドポイントを呼び出す内部処理・連携バッチ、および詳細APIを参照するフロントエンド。
- 🔵 **システム内での位置づけ**: バックエンドAPI（Axum + sqlx + PostgreSQL）の4層構造（Handlers → Repositories → Models）に従う。Phase 4「ファイル管理・拡張機能」の独立した拡張機能。前提タスク TASK-0026（item_files 基盤）に積み上げる。
- 🟡 **副次機能**: `calibre_book_id` 設定済みの PDF について、アイテム詳細API（GET /items/:id）のレスポンスに Calibre-Web 遷移用情報を含める（TC-020-02、URL構築方式は未確定のため変更容易な構造で実装）。

- **参照したタスク定義**: `docs/tasks/mediavault-backend/TASK-0028.md`（L16-30 タスク概要・完了条件）
- **参照した要件**: REQ-020 / REQ-103
- **参照した設計文書**: `docs/design/mediavault-backend/api-endpoints.md`（PATCH /items/:id/files/:file_id/calibre-link）、`docs/design/mediavault-backend/architecture.md`（4層構造）
- **参照した既存実装**: `backend/mediavault-api/src/models/item_file.rs`、`backend/mediavault-api/src/repositories/item_file_repository.rs`、`backend/mediavault-api/src/handlers/item_files.rs`

---

## 2. 入力・出力の仕様（API仕様・型定義ベース）

### 2.1 入力

- 🔵 **パスパラメータ**:
  - `:id`（= `item_id`）: UUID文字列。`parse_item_id(&str) -> Result<Uuid, ApiError>` で検証（不正形式は VALIDATION_ERROR / 400）。
  - `:file_id`: UUID文字列。同様にUUID形式を検証する（不正形式は VALIDATION_ERROR / 400）。🟡 既存に `parse_file_id` は無いため、`parse_item_id` と同等の検証関数を追加する想定。
- 🔵 **リクエストボディ**（`Content-Type: application/json`）:
  ```json
  { "calibre_book_id": "calibre-12345" }
  ```
  - `calibre_book_id`: 必須・非空文字列（trim後に空文字は VALIDATION_ERROR / 400）。
- 🔵 **DTO定義**: `UpdateCalibreLinkRequest { calibre_book_id: String }` を `backend/mediavault-api/src/models/item_file.rs` に追加。デシリアライズは既存の `deserialize_request(value) -> Result<T, ApiError>` パターンを使用。検証関数 `parse_update_calibre_link_request` を追加する想定。

### 2.2 出力

- 🔵 **成功（200 OK）**: 更新後の `item_files` レコードを統一レスポンス形式 `ApiOk` で返す。
  ```json
  {
    "success": true,
    "data": {
      "id": "uuid",
      "item_id": "uuid",
      "path": "/srv/files/pdf/example.pdf",
      "label": "本編PDF",
      "file_type": "pdf",
      "calibre_book_id": "calibre-12345",
      "created_at": "2026-06-26T10:00:00"
    }
  }
  ```
  既存 `ItemFile` 構造体（`models/item_file.rs` L24-33）をそのまま返却に使用できる。
- 🔵 **バリデーションエラー（400）**: `file_type != pdf`、`calibre_book_id` 空文字、UUID形式不正の場合。
  ```json
  { "success": false, "error": { "code": "VALIDATION_ERROR", "message": "..." } }
  ```
- 🟡 **NotFound（404）**: `item_id`/`file_id` が存在しない、または `item_id` と `file_id` の紐付けが不一致の場合、`FILE_NOT_FOUND` 相当を返す。
  ```json
  { "success": false, "error": { "code": "FILE_NOT_FOUND", "message": "..." } }
  ```
  > **重要な実装上の注記（🟡→要対応）**: 現状 `backend/mediavault-api/src/models/response.rs` の `ApiErrorCode` enum に `FileNotFound`（`"FILE_NOT_FOUND"` / 404）variant は**未定義**。本タスクで新規追加する必要がある（TASK-0024 の `ApiKeyNotConfigured` 追加パターン、TASK-0027 の `FileStorageWriteFailed` 追加パターンと同様）。`code_and_status()` への追加も必要。

### 2.3 入出力の関係性・データフロー

- 🔵 リクエストの `calibre_book_id` で、`:id`+`:file_id` で特定され `file_type='pdf'` の `item_files` 行を `UPDATE` し、更新後の行を `RETURNING` で取得して返す。
- 🔵 更新は冪等（同じ `calibre_book_id` で複数回呼んでも同じ結果）。
- 🟡 詳細API（GET /items/:id）側では、`calibre_book_id IS NOT NULL` かつ `file_type='pdf'` の `item_files` について Calibre-Web 遷移情報をレスポンスに付加する。

- **参照したAPI仕様**: `docs/design/mediavault-backend/api-endpoints.md`（リクエスト/レスポンス例）、note.md L105-148
- **参照した型定義**: `backend/mediavault-api/src/models/item_file.rs`（`ItemFile`, `FileType`）

---

## 3. 制約条件（非機能要件・アーキテクチャ設計ベース）

- 🔵 **アーキテクチャ制約**: 4層構造を遵守。
  - Handler: `update_calibre_link_handler`（`handlers/item_files.rs`）— パスパラメータ/ボディ検証、レスポンス整形。
  - Repository: `update_calibre_link`（`item_file_repository.rs`）— SQL実行とDBエラー変換。
  - Model: `UpdateCalibreLinkRequest` + 検証関数（`models/item_file.rs`）。
- 🔵 **データベース制約**: `item_files` テーブル（id UUID PK, item_id FK, path, label nullable, file_type enum[pdf/image/other], calibre_book_id nullable, created_at）。`calibre_book_id` は NULL 許容（参照: `database-schema.sql`、note.md L33-37, L152-155）。
- 🔵 **SQL方針**: `UPDATE item_files SET calibre_book_id = $1 WHERE id = $2 AND item_id = $3 AND file_type = 'pdf' RETURNING ...`（タスク定義書 L54）。`fetch_optional` で 0行（不存在/不一致/非pdf）の場合は `None` を返し、ハンドラ側で 404 判定。
  - 🟡 **file_type=pdf の検証順序**: テストケース2（`file_type=photo` で **VALIDATION_ERROR(400)**）とテストケース3（不存在/不一致で **404**）を区別するため、`WHERE` 句に `file_type='pdf'` を含めるだけでは両者とも0行となり区別不能。**先に対象行を取得（id+item_id一致）→存在しなければ404→存在するが file_type≠pdf なら400→pdfなら更新** という2段階方式が必要。
- 🔵 **エラーハンドリング制約**: 統一 `ApiError::new(code, message)` を使用。DBエラーは repository層の `db_error(sqlx::Error) -> ApiError`（INTERNAL_ERROR/500）でマスキングし内部情報を漏らさない（response.rs / item_file_repository.rs L13-19）。
- 🔵 **SQLインジェクション対策**: sqlx のパラメータバインドで対策済み。
- 🟡 **calibre_book_id の妥当性検証は範囲外**: Calibre-Web 側に実在するIDかは検証しない（タスク定義書 L103-104）。
- 🟡 **認証**: 本エンドポイントは現状内部API認証未対応（Phase 2 で認証スキーム実装予定。note.md L232）。
- 🟡 **詳細API遷移URL構築**: Calibre-Web 側の URL構成・認証方式が未確定のため、遷移情報は独立した小型構造体で定義し変更容易にする（タスク定義書 L58-64, L102）。本タスクでは固定テンプレート/設定値ベースで実装。
- 🔵 **パフォーマンス**: 単一行UPDATE・単一テーブル参照で軽量。特別なパフォーマンス要件なし。

- **参照した非機能要件/設計**: `docs/design/mediavault-backend/architecture.md`、`docs/design/mediavault-backend/database-schema.sql`、note.md 第6章

---

## 4. 想定される使用例（テストケース・データフローベース）

### 4.1 基本的な使用パターン（正常系）

- 🔵 **TC-020-01**: `file_type=pdf` の既存 `item_files` レコードに対し `{ "calibre_book_id": "calibre-12345" }` で PATCH → 200、`calibre_book_id` が更新され、更新後レコードが返る。
- 🟡 **TC-020-02**: `calibre_book_id` 設定済みPDFを持つアイテムで GET /items/:id → レスポンスの該当ファイル情報に `calibre_book_id`（および Calibre-Web 遷移情報）が含まれる。

### 4.2 エッジ・エラーケース

- 🔵 **E01（テストケース2）**: 対象 `item_files` の `file_type` が `pdf` 以外（例: `photo`/`image`）→ **VALIDATION_ERROR(400)**、レコードは更新されない。
- 🔵 **E02（テストケース3）**: `file_id` が存在しない、または `item_id` と `file_id` の紐付けが不一致 → **FILE_NOT_FOUND相当(404)**。
- 🟡 **E03**: `calibre_book_id` が空文字/空白のみ → VALIDATION_ERROR(400)。
- 🟡 **E04**: `:id` または `:file_id` が UUID形式でない → VALIDATION_ERROR(400)。
- 🟡 **E05**: リクエストボディが不正JSON / `calibre_book_id` キー欠落 → VALIDATION_ERROR(400)（`deserialize_request` のデシリアライズエラー）。

### 4.3 データフロー

```
Client
  → PATCH /items/:id/files/:file_id/calibre-link  { calibre_book_id }
  → handler: parse_item_id(:id) / parse_file_id(:file_id) / deserialize_request / parse_update_calibre_link_request
  → repository: 対象行取得(id+item_id) → 不存在? 404 : file_type≠pdf? 400 : UPDATE ... RETURNING
  → 200 ApiOk(ItemFile)

GET /items/:id（詳細）
  → 既存 get_item_detail + item_files 取得
  → calibre_book_id IS NOT NULL & file_type=pdf の行に Calibre-Web 遷移情報を付加
  → 200
```

- **参照したテストケース**: タスク定義書「単体テスト要件」（L66-90）、note.md 第5章（TC-020-01/02, E01/E02）
- **参照したデータフロー**: `docs/design/mediavault-backend/dataflow.md`

---

## 5. EARS要件・設計文書との対応関係

- **参照したユーザストーリー**: Calibre-Web 取込完了後の PDF 書籍ID関連付け（REQ-103）
- **参照した機能要件**: REQ-020, REQ-103
- **参照した受け入れ基準（完了条件）**:
  1. `calibre_book_id` を受け取り対象 `item_files` を更新（200で更新後レコード返却）
  2. `file_type != pdf` でエラー（VALIDATION_ERROR/400）
  3. `item_id`/`file_id` 不存在・紐付け不一致で FILE_NOT_FOUND相当(404)
  4. 詳細APIレスポンスに Calibre-Web 遷移情報付加（TC-020-02）
  5. TC-020-01・TC-020-02 を満たす
- **参照したEdgeケース**: E01（file_type不一致→400）, E02（不存在/不一致→404）
- **参照した設計文書**:
  - **API仕様**: `docs/design/mediavault-backend/api-endpoints.md`（PATCH /items/:id/files/:file_id/calibre-link）
  - **データベース**: `docs/design/mediavault-backend/database-schema.sql`（`item_files` テーブル）
  - **アーキテクチャ**: `docs/design/mediavault-backend/architecture.md`（4層構造・統一エラーハンドリング）
  - **データフロー**: `docs/design/mediavault-backend/dataflow.md`
- **参照した既存実装（前提・参考）**:
  - `backend/mediavault-api/src/models/item_file.rs`（`ItemFile`, `FileType`, `CreateItemFileRequest`, 検証関数パターン）
  - `backend/mediavault-api/src/repositories/item_file_repository.rs`（`db_error`, `item_exists`, `create_item_file`）
  - `backend/mediavault-api/src/handlers/item_files.rs`（ハンドラ/レスポンス整形パターン）
  - `backend/mediavault-api/src/models/response.rs`（`ApiErrorCode`, `ApiOk`, `code_and_status`）
  - `backend/mediavault-api/src/routes/mod.rs`（ルート登録パターン）
  - TASK-0012（PATCH 部分更新パターン）, TASK-0024/0027（ApiErrorCode 新規variant追加パターン）

---

## 6. 実装対象ファイル一覧

| ファイル | 変更内容 | 信頼性 |
|---|---|---|
| `backend/mediavault-api/src/models/item_file.rs` | `UpdateCalibreLinkRequest` DTO + `parse_update_calibre_link_request` 追加。詳細用 Calibre-Web 遷移情報の小型構造体 | 🔵 / 🟡 |
| `backend/mediavault-api/src/models/response.rs` | `ApiErrorCode::FileNotFound`（`"FILE_NOT_FOUND"`/404）variant 追加 + `code_and_status()` 対応 | 🟡（要新規追加） |
| `backend/mediavault-api/src/repositories/item_file_repository.rs` | `update_calibre_link`（対象取得→pdf検証→UPDATE RETURNING）追加 | 🔵 |
| `backend/mediavault-api/src/handlers/item_files.rs` | `update_calibre_link_handler` 追加。404/400分岐 | 🔵 |
| `backend/mediavault-api/src/handlers/items.rs` ほか詳細取得経路 | 詳細APIレスポンスへの Calibre-Web 遷移情報付加 | 🟡 |
| `backend/mediavault-api/src/routes/mod.rs` | `PATCH /items/:id/files/:file_id/calibre-link` ルート登録 | 🔵 |

---

## 7. 品質判定

```
✅ 高品質（主要部分）:
- 要件の曖昧さ: コア更新ロジック（更新/400/404）は明確
- 入出力定義: API仕様・既存ItemFile構造体により完全
- 制約条件: 4層構造・エラーハンドリング・DBスキーマが明確
- 実装可能性: 確実（前提TASK-0026の基盤あり）

⚠️ 要確認項目（実装時に決定が必要）:
1. FILE_NOT_FOUND エラーコードが未定義 → response.rs に新規追加が必要（🟡）
2. file_type検証(400)と不存在(404)の区別 → 2段階処理（取得→検証→更新）が必要（🟡）
3. 詳細API(GET /items/:id)への Calibre-Web 遷移情報付加(TC-020-02) → URL構築方式未確定。
   現状の詳細レスポンスに item_files が含まれているか要確認。独立した小型構造体で変更容易に実装（🟡）
4. :file_id 用UUID検証関数 → 既存 parse_item_id 相当を流用/追加（🟡）

信頼性レベル分布: 🔵 多数（コアCRUD）/ 🟡 数件（エラーコード追加・詳細API拡張・検証順序）/ 🔴 なし
総合評価: 高品質（コア機能は確実、TC-020-02部分とエラーコード追加は実装時に確定）
```

---

## 次のステップ

次のお勧めステップ: `/tsumiki:tdd-testcases mediavault-backend TASK-0028` でテストケースの洗い出しを行います。
