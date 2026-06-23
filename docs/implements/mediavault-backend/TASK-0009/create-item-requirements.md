# TASK-0009: POST /items（手動作成）要件定義

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

- 🔵 何をする機能か: フォーム入力（JSONリクエスト）のみでアイテムを新規作成する `POST /items` エンドポイントを実装する。`source=manual`, `external_id=NULL` でitemsテーブルへ登録し、media_typeに対応する詳細テーブル（anime_details等）へも同一トランザクションでINSERTする。
- 🔵 どのような問題を解決するか: 外部API検索結果に依存しないアイテム登録手段を提供する（As a ユーザー / So that 外部APIに存在しない作品や手動で記録したい作品も登録できる）
- 🔵 想定されるユーザー: MediaVaultの利用者（個人ライブラリ管理者）
- 🔵 システム内での位置づけ: items共通CRUDの作成エンドポイント。Phase 2のコアCRUD実装の起点であり、TASK-0010〜0014（GET一覧/詳細/PATCH/DELETE/status更新）の前提となる
- **参照したEARS要件**: REQ-003（フォーム入力のみでアイテム新規作成）
- **参照した設計文書**: `docs/design/mediavault-backend/api-endpoints.md`（POST /items セクション）, `docs/design/mediavault-backend/architecture.md`

## 2. 入力・出力の仕様（EARS機能要件・型定義ベース）

- 🔵 入力パラメータ: `CreateItemRequest`（`backend/mediavault-api/src/models/item.rs`定義済み、TASK-0008）
  - `media_type: MediaType`（必須、enum: anime/movie/drama/manga/novel/game/academic_book/paper）
  - `title: String`（必須、空白のみは不可）
  - `original_title, description, cover_image_url, release_date, homepage_url: Option<...>`（任意）
  - `rating: Option<f32>`, `is_favorite: Option<bool>`（任意、未指定時はDBデフォルト: false）
  - `details: Option<serde_json::Value>`（任意、media_typeに応じた詳細テーブルカラムをJSONで指定。未指定時は全カラムNULL/デフォルト`'{}'`でINSERT）
- 🔵 出力値: 成功時 `{ "success": true, "data": <作成済みItem> }` をHTTP 201で返す（`Item`構造体相当のJSON、UUID付き）
- 🟡 出力の詳細フォーマット（詳細テーブルの内容を含めるか）: api-endpoints.mdには「作成済みitem（UUID付き）」のみ記載され詳細フィールドの扱いは明記がないため、`items`テーブルの内容のみを返す方針とする（妥当な推測）
- 🔵 入出力の関係性: リクエストの`media_type`によって振り分け先の詳細テーブルが決まる（1:1関連）。`source`/`external_id`はリクエストに含まれず、ハンドラ側で固定値（`Manual`/`None`）を付与する
- **参照したEARS要件**: REQ-003
- **参照した設計文書**: `backend/mediavault-api/src/models/item.rs`（CreateItemRequest, Item）, `docs/design/mediavault-backend/database-schema.sql`（items, 各*_detailsテーブル）

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

- 🔵 データベース制約: `items.title`は`NOT NULL`、`items.source`は`NOT NULL`。各詳細テーブルは`item_id UUID PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE`の1:1関連。`genre_list`/`platform_list`/`author_list`等の配列カラムは`NOT NULL DEFAULT '{}'`
- 🔵 トランザクション制約: itemsテーブルへのINSERTと詳細テーブルへのINSERTは同一`sqlx::Transaction`内で実行し、いずれかが失敗した場合は両方ロールバックする
- 🔵 バリデーション制約: `media_type`が不正な値の場合（デシリアライズ失敗）、`title`が空文字・空白のみの場合は`VALIDATION_ERROR`（400）を返す。既存の`parse_create_item_request`/`validate_title`（TASK-0008実装済み）を流用する
- 🟡 レスポンスステータス制約: 既存の`ApiOk<T>::into_response()`は`StatusCode::OK`(200)固定のため、201を返すには本タスクのハンドラ内で`(StatusCode::CREATED, Json(ApiOk::new(item)))`を明示的に構築する必要がある（既存共通型のそのままの利用はできない点に注意）
- 🔵 アーキテクチャ制約: ルーティングは`routes/mod.rs`の`build_router`に`.route("/items", post(...))`を追加。ハンドラは`handlers/items.rs`（新規）、DB処理は新設の`repositories/item_repository.rs`に分離する
- **参照したEARS要件**: REQ-003
- **参照した設計文書**: `docs/design/mediavault-backend/database-schema.sql`, `backend/mediavault-api/src/models/response.rs`, `backend/mediavault-api/src/routes/mod.rs`

## 4. 想定される使用例（EARS Edgeケース・データフローベース）

- 🔵 基本的な使用パターン（TC-001-01）: `media_type="anime"`, `title="作品A"`のみ指定 → 201、`source=manual`, `external_id=null`のitemが返る
- 🔵 エラーケース（TC-001-E01）: `media_type="invalid"` → 400 `VALIDATION_ERROR`
- 🔵 境界値ケース（TC-001-B01）: `title=""` → 400 `VALIDATION_ERROR`
- 🟡 エッジケース: `title`が空白のみ（`"   "`）の場合も`validate_title`により400となる（TASK-0008の既存テストで確認済みの挙動を流用）
- 🟡 統合テストケース: 実DB（PostgreSQL）に対して`POST /items`実行後、`items`テーブルおよび対応する詳細テーブルにレコードが存在することを確認する（トランザクション整合性の確認は妥当な推測）
- **参照したEARS要件**: REQ-003、関連EDGE要件は本タスクに直接該当するものなし
- **参照した設計文書**: `docs/tasks/mediavault-backend/TASK-0009.md`（単体テスト要件・統合テスト要件セクション）

## 5. EARS要件・設計文書との対応関係

- **参照した機能要件**: REQ-003
- **参照した受け入れ基準/テストケース**: TC-001-01（必須項目のみで作成）, TC-001-E01（media_type不正で400）, TC-001-B01（title空文字で400）
- **参照した設計文書**:
  - **アーキテクチャ**: `docs/design/mediavault-backend/architecture.md`
  - **データベース**: `docs/design/mediavault-backend/database-schema.sql`（items, anime_details〜paper_details、L45-160）
  - **API仕様**: `docs/design/mediavault-backend/api-endpoints.md`（POST /items, L85-101）
  - **既存実装（TASK-0008）**: `backend/mediavault-api/src/models/item.rs`
  - **既存実装（TASK-0005）**: `backend/mediavault-api/src/models/response.rs`
  - **既存実装（TASK-0007）**: `backend/mediavault-api/src/main.rs`, `backend/mediavault-api/src/routes/mod.rs`, `backend/mediavault-api/src/db/mod.rs`

---

## 品質判定

✅ **高品質**
- 要件の曖昧さ: なし（タスクファイルに完了条件・テストケースが明記）
- 入出力定義: 完全（既存のCreateItemRequest/Item構造体が確定済み）
- 制約条件: 明確（DBスキーマ・トランザクション要件が明記）
- 実装可能性: 確実（依存タスクTASK-0008が完了済みで土台が整っている）
- 信頼性レベル: 🔵が大多数、🟡は2件（レスポンス詳細フィールドの扱い、201構築方法）のみ
