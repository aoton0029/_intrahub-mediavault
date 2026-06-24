# TASK-0020 要件定義書: スタッフ管理CRUD実装

**タスクID**: TASK-0020
**機能名**: staff（スタッフ管理CRUD）
**要件名**: mediavault-backend
**出力ファイル**: docs/implements/mediavault-backend/TASK-0020/staff-requirements.md

---

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

- 🔵 **何をする機能か**: スタッフ（`staff`）の作成と、アイテム（item）へのスタッフ紐付け（`item_staff`、`role`・`character_name`付き）のCRUDを提供する。具体的には `POST /staff`、`POST /items/:id/staff`、`DELETE /items/:id/staff/:item_staff_id` の3エンドポイントを実装する。
- 🔵 **どのような問題を解決するか**: 利用者が作品（item）に対して監督・声優などのスタッフ情報を登録・関連付けし、役割やキャラクター名を含めて管理できるようにする（REQ-009「スタッフの追加、役割の付与、作品への紐付けを行うAPIを提供」）。
- 🔵 **想定されるユーザー**: コレクション（メディア作品）管理を行う個人ユーザー（user-stories 4.1）。
- 🟡 **システム内での位置づけ**: items CRUD（TASK-0008〜0014）に続くPhase 2「コアCRUD実装」の一部。`staff`はitemsと独立したエンティティで、`item_staff`がitemとstaffを多対多に紐付ける関連テーブル（role/character_name属性付き）として機能する。tags/categories（TASK-0015）と同様の関連エンティティパターンに属する。
- **参照したEARS要件**: REQ-009
- **参照した設計文書**: docs/design/mediavault-backend/api-endpoints.md（staff関連エンドポイント）, docs/design/mediavault-backend/architecture.md（レイヤードアーキテクチャ）, docs/spec/mediavault-backend/requirements.md（REQ-009）

---

## 2. 入力・出力の仕様（EARS機能要件・型定義ベース）

### 2.1 POST /staff 🔵
- **入力**: `{ "name": string(必須), "external_id": string|null(optional), "image_url": string|null(optional) }`
  - `name`: `VARCHAR(255) NOT NULL`、空文字不可
  - `external_id`: `VARCHAR(100)` NULL許容（外部API重複防止用、本フェーズでは保持のみ・バリデーション不要）
  - `image_url`: `VARCHAR(1000)` NULL許容
- **出力（成功）**: 201, `{ success: true, data: { id: UUID, external_id, name, image_url, created_at } }`
- **出力（バリデーション失敗）**: 400 `VALIDATION_ERROR`（name空など）
- **参照したEARS要件**: REQ-009
- **参照した設計文書**: database-schema.sql（staffテーブル）, api-endpoints.md（POST /staff）

### 2.2 POST /items/:id/staff 🔵
- **入力**: パスパラメータ `id`（item UUID）、ボディ `{ "staff_id": UUID(必須), "role": string(必須), "character_name": string|null(optional) }`
  - `role`: `VARCHAR(100) NOT NULL`、空文字不可、上限100文字
  - `character_name`: `VARCHAR(255)` NULL許容（声優役などキャラ名が必要な場合のみ）、上限255文字
- **出力（成功）**: 201, `{ success: true, data: { id: UUID, item_id, staff_id, role, character_name } }`
- **出力（staff不存在）**: 404 `STAFF_NOT_FOUND`
- **出力（item不存在）**: 404 `ITEM_NOT_FOUND`
- **出力（バリデーション失敗）**: 400 `VALIDATION_ERROR`（role空・UUID不正など）
- **参照したEARS要件**: REQ-009
- **参照した設計文書**: api-endpoints.md（POST /items/:id/staff、リクエスト例 `{ "staff_id": "...", "role": "監督", "character_name": null }`）, database-schema.sql（item_staffテーブル）

### 2.3 DELETE /items/:id/staff/:item_staff_id 🟡
- **入力**: パスパラメータ `id`（item UUID）, `item_staff_id`（item_staff.id UUID）
- **出力（成功）**: 204 No Content
- **出力（item_staff不存在 または item_idに属さない）**: 404
- **参照したEARS要件**: REQ-009（「紐付け」から削除エンドポイントを妥当推測、api-endpoints.mdでも🟡）
- **参照した設計文書**: TASK-0020.md 実装詳細3, api-endpoints.md

### 2.4 データフロー（共通）
- routes → handlers（リクエスト検証・存在確認）→ repositories（sqlx query）→ PostgreSQL。
- ハンドラから直接SQLを書かず、DB操作はrepository層へ集約する。🔵

---

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

- 🔵 **データベース制約**:
  - `staff(id UUID PK DEFAULT gen_random_uuid(), external_id VARCHAR(100), name VARCHAR(255) NOT NULL, image_url VARCHAR(1000), created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP)`。`idx_staff_external_id`。
  - `item_staff(id UUID PK, item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE, staff_id UUID NOT NULL REFERENCES staff(id) ON DELETE CASCADE, role VARCHAR(100) NOT NULL, character_name VARCHAR(255))`。`idx_item_staff_item_id`, `idx_item_staff_staff_id`。
  - `item_id`/`staff_id`はFK制約あり。不存在のIDはDB側でintegrity error（SQLSTATE `23503`）となるが、より詳細なエラーメッセージのためアプリ側で事前存在確認も行う（両重チェック推奨）。
- 🟡 **エラーコード制約**: 既存`ApiErrorCode`（response.rs）には`STAFF_NOT_FOUND`が存在しない。TASK-0020完了条件「存在しないstaff_idを指定した場合STAFF_NOT_FOUND（404）を返す」に従い、`ApiErrorCode`に`StaffNotFound`バリアント（`STAFF_NOT_FOUND` / 404）を新規追加する。`ItemNotFound`は既存を流用。
- 🔵 **セキュリティ要件**: DBエラーの内部情報（SQLメッセージ等）はクライアントへ返さず、`tracing::error!`でサーバーログのみに出力する（item_repository.rsの`db_error`パターンを踏襲）。SQLインジェクションはsqlxマクロ（`query!`/`query_as!`）で自動防止。
- 🔵 **入力検証制約**: `role`上限100文字、`character_name`上限255文字、`name`は空文字不可。ハンドラ内の`parse_*`関数で検証し`Result`で早期リターン。
- 🔵 **アーキテクチャ制約**: 既存の`AppState`（DBプール保持）、`ApiOk<T>`/`ApiError`レスポンス型、`routes/mod.rs`構成に対し、`models/staff.rs`・`repositories/staff_repository.rs`・`handlers/staff.rs`を新規追加して統合する。
- 🔵 **技術スタック制約**: Rust(Edition 2024) / Axum 0.8 / sqlx 0.8（コンパイル時SQLチェック）/ Tokio / uuid v4 / chrono。
- **参照したEARS要件**: REQ-009
- **参照した設計文書**: database-schema.sql, architecture.md, backend/mediavault-api/src/models/response.rs, item_repository.rs（db_error）

---

## 4. 想定される使用例（Edgeケース・データフローベース）

### 4.1 基本的な使用パターン 🔵
- スタッフを `POST /staff` で作成 → 返却された`id`を使い `POST /items/:id/staff` でitemへ紐付け → 不要になれば `DELETE /items/:id/staff/:item_staff_id` で解除。

### 4.2 正常系ケース
- 🔵 **スタッフ作成（必須のみ）**: `{ "name": "監督A" }` → 201 + UUID付きStaff。
- 🔵 **itemへの紐付け（監督役）**: 既存item・既存staff, `{ "staff_id": "...", "role": "監督" }` → 201, item_staffにレコード作成、`character_name`はnull。
- 🟡 **character_name付きの紐付け（声優役）**: `{ "staff_id": "...", "role": "声優", "character_name": "主人公" }` → 201, character_nameが正しく保存される。
- 🟡 **紐付け削除**: 既存item_staffレコードに対し DELETE → 204, レコード削除。

### 4.3 エッジ・エラーケース
- 🟡 **存在しないstaff_idで紐付け**: 不存在staff UUIDで `POST /items/:id/staff` → 404 `STAFF_NOT_FOUND`。
- 🟡 **存在しないitem_idで紐付け**: 不存在item UUIDで `POST /items/:id/staff` → 404 `ITEM_NOT_FOUND`。
- 🟡 **存在しないitem_staff_idで削除**: 不存在item_staff UUIDで DELETE → 404。
- 🟡 **item_idに属さないitem_staff_idで削除**: item_staffは存在するが指定item_idと不一致 → 404（整合性チェック）。
- 🟡 **カスケード削除（統合テスト）**: staff削除時に関連する`item_staff`が`ON DELETE CASCADE`で自動削除されることを実DBで確認する。
- 🟡 **role空文字 / 不正UUID**: 400 `VALIDATION_ERROR`。

### 4.4 注意事項
- 🔵 `staff.external_id`は外部API（AniList等）由来のスタッフ重複登録防止用。本フェーズでは外部API連携対象外のため、フィールド保持のみ（バリデーション不要）。
- **参照したEARS要件**: REQ-009
- **参照した設計文書**: TASK-0020.md（単体テスト要件・統合テスト要件）, database-schema.sql（ON DELETE CASCADE）

---

## 5. EARS要件・設計文書との対応関係

- **参照したユーザストーリー**: user-stories 4.1（スタッフ・役割の管理）
- **参照した機能要件**: REQ-009（staffの追加、roleの付与、作品への紐付けAPI提供）
- **参照した非機能要件**: セキュリティ（DB内部情報の秘匿、SQLインジェクション防止）、入力検証（長さ制限）、パフォーマンス（item_staffのindex活用）
- **参照したEdgeケース**: STAFF_NOT_FOUND、ITEM_NOT_FOUND、item_staff不存在/不一致による404、role/character_name長さ超過、カスケード削除
- **参照した受け入れ基準**: TASK-0020.md 完了条件5項目・単体テスト要件5ケース・統合テスト要件1ケース
- **参照した設計文書**:
  - **アーキテクチャ**: architecture.md（routes → handlers → repositories → db レイヤード構成）
  - **データフロー**: HTTPリクエスト → ハンドラ検証 → repository sqlx → PostgreSQL
  - **型定義**: 新規 `Staff` / `ItemStaff`（sqlx::FromRow）, `CreateStaffRequest` / `CreateItemStaffRequest`（serde::Deserialize）
  - **データベース**: database-schema.sql（`staff`, `item_staff`テーブル）
  - **API仕様**: api-endpoints.md（POST /staff, POST /items/:id/staff, DELETE /items/:id/staff/:item_staff_id）

---

## 品質判定

| 評価項目 | 状態 |
|---|---|
| 要件の曖昧さ | なし（DELETEのitem_id整合性チェックのみ🟡推測だがタスク仕様に明記あり） |
| 入出力定義 | 完全（3エンドポイントの入力・出力・エラーを定義） |
| 制約条件 | 明確（DB制約・新規エラーコード・セキュリティ・検証） |
| 実装可能性 | 確実（既存items/tags CRUDパターンを踏襲可能） |
| 信頼性レベル分布 | 🔵 多数（概要・POST系・DB制約）、🟡 一部（DELETE仕様・エッジケース）、🔴 なし |

**総合評価**: ✅ 高品質
