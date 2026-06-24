# TASK-0015 要件定義書: タグ・カテゴリCRUD実装

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

- 🔵 **何をする機能か**: タグ・カテゴリの作成・削除、およびアイテムへの付与・解除を行うCRUD API一式を実装する。
- 🔵 **どのような問題を解決するか**: 利用者が自分なりの分類体系（タグ・カテゴリ）でコレクションを整理できるようにする（user-stories.md 2.1「そうすることで自分なりの分類体系でコレクションを整理できる」）。
- 🔵 **想定されるユーザー**: コレクション管理を行う利用者（個人ユーザー）。
- 🟡 **システム内での位置づけ**: items CRUD（TASK-0008〜0014）に続くPhase 2コアCRUDの一部。タグ/カテゴリはitemsとは独立した多対多の関連エンティティであり、items本体のテーブルとは別テーブル（`tags`, `categories`, `item_tags`, `item_categories`）で管理される。
- **参照したEARS要件**: REQ-004（タグ・カテゴリの作成、アイテムへの付与・削除APIを提供）
- **参照した設計文書**: docs/design/mediavault-backend/api-endpoints.md L236-248（タグ・カテゴリ・マイリスト・関連付けセクション）, docs/spec/mediavault-backend/user-stories.md L98-112（ストーリー2.1）

## 2. 入力・出力の仕様

### POST /tags 🔵
- **入力**: `{ "name": string }`（VARCHAR(100)、必須、空文字不可）
- **出力（成功）**: 201, `{ success: true, data: { id: UUID, name: string } }`
- **出力（重複名）**: タグ名一意制約違反エラー（後述の制約条件参照）
- **参照したEARS要件**: REQ-004
- **参照した設計文書**: docs/design/mediavault-backend/database-schema.sql（tagsテーブル: `id UUID PRIMARY KEY DEFAULT gen_random_uuid(), name VARCHAR(100) NOT NULL UNIQUE`）

### DELETE /tags/:id 🔵
- **入力**: パスパラメータ `id`（UUID）
- **出力（成功）**: 204 No Content
- **出力（存在しない）**: 404
- 削除時、`item_tags`の関連レコードは`ON DELETE CASCADE`により自動削除される（アプリ側で個別削除処理は不要）
- **参照した設計文書**: docs/design/mediavault-backend/database-schema.sql（item_tagsテーブルFK制約 `ON DELETE CASCADE`）

### POST /items/:id/tags/:tag_id 🟡
- **入力**: パスパラメータ `id`（item UUID）, `tag_id`（tag UUID）。リクエストボディなし。
- **出力（成功）**: 201（`item_tags`への複合キーINSERT）
- **出力（既に付与済み）**: 複合PK制約違反 → no-opまたは409（タスク仕様に「既存の場合はno-opまたは409」と記載、本実装ではエラーを返す方針とする＝409）
- **出力（item/tag不存在）**: FK制約違反 → 404（事前存在チェックまたはFK違反ハンドリングで対応）
- **参照したEARS要件**: REQ-004「付与・削除」（エンドポイント形式は仕様未記載のため推測 🟡）
- **参照した設計文書**: docs/tasks/mediavault-backend/TASK-0015.md L27, item_tagsテーブル定義

### DELETE /items/:id/tags/:tag_id 🟡
- **入力**: パスパラメータ `id`（item UUID）, `tag_id`（tag UUID）
- **出力（成功）**: 204
- **出力（関連レコード不存在）**: 404
- **参照したEARS要件**: REQ-004
- **参照した設計文書**: docs/tasks/mediavault-backend/TASK-0015.md L28

### POST /categories ・ DELETE /categories/:id 🔵
- タグと完全に同一パターン（テーブル名・カラム名のみ異なる）
- **参照した設計文書**: docs/design/mediavault-backend/database-schema.sql（categoriesテーブル）

### POST /items/:id/categories/:category_id ・ DELETE /items/:id/categories/:category_id 🟡
- タグの付与/解除と完全に同一パターン
- **参照したEARS要件**: REQ-004「item_categoriesへの付与・削除エンドポイントが実装される」（タスク完了条件より、エンドポイント形式自体はAPI仕様に明記なしのため推測 🟡）
- **参照した設計文書**: docs/tasks/mediavault-backend/TASK-0015.md L30

## 3. 制約条件

- 🔵 **データベース制約**: `tags.name`, `categories.name` は `VARCHAR(100) NOT NULL UNIQUE`。重複作成時はPostgreSQLユニーク制約違反（SQLSTATE `23505`）が発生する。
- 🔵 **データベース制約**: `item_tags`, `item_categories` は複合PK（`item_id, tag_id` / `item_id, category_id`）かつ両FKに`ON DELETE CASCADE`を持つ。重複付与は複合PK制約違反（SQLSTATE `23505`）。
- 🟡 **エラーコード制約**: 既存`ApiErrorCode`（ValidationError/Unauthorized/ItemNotFound/UnprocessableEntity/InternalError/ExternalApiError）には一意制約違反専用のコードがない。タスク仕様の注意事項（TASK-0015.md L94）「タグ・カテゴリ名の一意制約違反時のエラーコードは共通エラー型に新規追加が必要（例: DUPLICATE_TAG_NAME, DUPLICATE_CATEGORY_NAME）」に従い、`ApiErrorCode`へ新規バリアントを追加し409 Conflictにマッピングする。
- 🔵 **セキュリティ要件**: 既存ハンドラと同様、DBエラーの内部情報（SQLメッセージ等）はクライアントに返さず、`tracing::error!`でログのみ行う（item_repository.rsの`db_error`パターンを踏襲）。
- 🔵 **アーキテクチャ制約**: 既存の`AppState`（DBプール保持）、`ApiOk<T>`/`ApiError`レスポンス型、ルーター構成（`routes/mod.rs`）にタグ・カテゴリ用のモデル/ハンドラ/リポジトリファイルを新規追加する形で統合する。
- **参照したEARS要件**: REQ-004
- **参照した設計文書**: docs/design/mediavault-backend/database-schema.sql, docs/tasks/mediavault-backend/TASK-0015.md（注意事項）, backend/mediavault-api/src/models/response.rs

## 4. 想定される使用例

### 基本パターン 🔵
1. `POST /tags { "name": "お気に入り" }` → 201でタグ作成
2. `POST /items/{item_id}/tags/{tag_id}` → アイテムにタグ付与
3. `DELETE /items/{item_id}/tags/{tag_id}` → タグ解除
4. `DELETE /tags/{tag_id}` → タグ削除（関連item_tagsもカスケード削除）

カテゴリも同様のフロー。

### エッジケース・エラーケース 🟡
- 既存タグと同名で`POST /tags`→ 一意制約違反エラー（テストケース2、TASK-0015.md L64-67に明記）
- 存在しないタグIDで`DELETE /tags/:id`→ 404（仕様には明記なし、既存`DELETE /items/:id`の404パターンから妥当な推測）
- 存在しないitem/tagの組み合わせで`POST /items/:id/tags/:tag_id`→ FK制約違反、404相当（タスク仕様に明記なし、妥当な推測）

- **参照したEARS要件**: REQ-004（acceptance-criteria.mdにはタグ・カテゴリ専用のEDGE-XXXは存在しない）
- **参照した設計文書**: docs/tasks/mediavault-backend/TASK-0015.md（単体テスト要件セクション、テストケース1〜4）

## 5. EARS要件・設計文書との対応関係

- **参照したユーザストーリー**: ストーリー2.1「タグ・カテゴリを管理する」（user-stories.md L98-112）
- **参照した機能要件**: REQ-004
- **参照した非機能要件**: 該当する専用NFRはなし（既存のエラーハンドリング・ログ方針NFRを継承）
- **参照したEdgeケース**: 専用EDGE-XXXはなし（TASK-0015.md内のテストケース1〜4が実質的なエッジケース定義）
- **参照した受け入れ基準**: docs/tasks/mediavault-backend/TASK-0015.md「単体テスト要件」テストケース1〜4、「統合テスト要件」（タグ削除時のカスケード削除確認）
- **参照した設計文書**:
  - **アーキテクチャ**: backend/mediavault-api/src/routes/mod.rs（既存ルーター構成）
  - **データフロー**: 該当する専用dataflow.md記載なし（items CRUDと同型のシンプルなCRUDフロー）
  - **型定義**: なし（item.rsのDTOパターンを参考に新規定義）
  - **データベース**: docs/design/mediavault-backend/database-schema.sql（tags, categories, item_tags, item_categoriesテーブル）
  - **API仕様**: docs/design/mediavault-backend/api-endpoints.md L236-248
