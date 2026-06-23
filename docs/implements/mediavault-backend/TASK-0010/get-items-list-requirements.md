# TDD要件定義書: GET /items（一覧・絞り込み）

- **機能名**: GET /items（一覧・絞り込み）
- **タスクID**: TASK-0010
- **要件名**: mediavault-backend
- **フェーズ**: Phase 2 - コアCRUD実装
- **作成日**: 2026-06-23

**【信頼性レベル凡例】**:
- 🔵 **青信号**: EARS要件定義書・設計文書を参考にしてほぼ推測していない場合
- 🟡 **黄信号**: EARS要件定義書・設計文書から妥当な推測の場合
- 🔴 **赤信号**: EARS要件定義書・設計文書にない推測の場合

---

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

- 🔵 **何をする機能か**: 登録済みアイテム（items）の一覧を取得し、`media_type` / `tag_id` / `category_id` / `is_favorite` / `status` による絞り込みと、`page` / `limit` によるページネーションを行う `GET /items` エンドポイントを提供する。
  - *参照元: `docs/tasks/mediavault-backend/TASK-0010.md` タスク概要, `docs/design/mediavault-backend/api-endpoints.md` GET /items（L63-71）*
- 🔵 **どのような問題を解決するか**: ユーザーが蓄積したメディアコレクションを横断的に閲覧・検索したいというニーズに対し、種別やタグ等の条件で目的のアイテムへ素早く到達できるようにする（REQ-001「一覧・絞り込み」, user-stories 1.4）。
  - *参照元: TASK-0010.md 信頼性レベル注記（REQ-001, user-stories 1.4）*
- 🔵 **想定されるユーザー**: 単一ユーザー（セルフホスト前提、利用者向けエンドポイントはユーザー認証を持たない）。
  - *参照元: `docs/design/mediavault-backend/api-endpoints.md` 認証（REQ-401, 単一ユーザー前提）*
- 🔵 **システム内での位置づけ**: Layered Architecture の `routes → handlers → repositories → DB` フローに従う読み取り系API。`handlers/items.rs` がクエリパラメータ抽出・正規化を行い、`repositories/item_repository.rs` が動的WHERE句を組み立ててDBから取得する。
  - *参照元: `docs/implements/mediavault-backend/TASK-0010/note.md` 1.技術スタック, `docs/design/mediavault-backend/architecture.md`*

- **参照したEARS要件**: REQ-001
- **参照した設計文書**: `api-endpoints.md`（GET /items, ページネーション）, `architecture.md`（Layered Architecture）

---

## 2. 入力・出力の仕様（EARS機能要件・型定義ベース）

### 2.1 入力（クエリパラメータ）

🔵 すべて optional。Axum の `Query` extractor で `ListItemsQuery` 構造体（`models/item.rs` に新規追加）として受け取る。
*参照元: `api-endpoints.md` GET /items クエリパラメータ, TASK-0010.md 実装詳細1, note.md 4.設計文書*

| パラメータ | 型 | 制約・デフォルト | 備考 |
|---|---|---|---|
| `media_type` | `Option<MediaType>` | enum値（anime/movie/drama/manga/novel/game/academic_book/paper） | 🔵 不正値はデシリアライズエラー→400 |
| `tag_id` | `Option<Uuid>` | UUID形式 | 🔵 指定時は `item_tags` 中間テーブルで絞り込み |
| `category_id` | `Option<Uuid>` | UUID形式 | 🔵 指定時は `item_categories` 中間テーブルで絞り込み |
| `is_favorite` | `Option<bool>` | true/false | 🔵 `idx_items_is_favorite` 活用 |
| `status` | `Option<ItemStatus>` | enum値（not_started/in_progress/completed） | 🔵 `idx_items_status` 活用 |
| `page` | `Option<u32>` | デフォルト 1 | 🟡 1未満は実装フェーズで方針確定（下記制約参照） |
| `limit` | `Option<u32>` | デフォルト 20、最大 100（1〜100にクランプ） | 🔵 |

- 🔵 **各フィルタ条件はAND結合**で適用される（指定された条件のみ追加）。
  - *参照元: TASK-0010.md 完了条件, api-endpoints.md*

### 2.2 出力（成功レスポンス）

- 🔵 **HTTPステータス**: 200 OK
- 🔵 **形式**: 統一フォーマット `{ "success": true, "data": [...], "pagination": {...} }`
  - `data`: `Item` 配列（`models/item.rs` の既存 `Item` 構造体をそのままシリアライズ）
  - `pagination`: `{ "page": <適用page>, "limit": <適用limit>, "total": <総件数> }`
  - *参照元: api-endpoints.md ページネーション（L47-55）, TASK-0010.md 完了条件, note.md 4.設計文書*

```json
{
  "success": true,
  "data": [
    {
      "id": "…", "media_type": "anime", "title": "作品A",
      "original_title": null, "description": null, "cover_image_url": null,
      "release_date": null, "homepage_url": null, "status": "not_started",
      "consumed_date": null, "rating": null, "is_favorite": false,
      "source": "manual", "external_id": null,
      "created_at": "…", "updated_at": "…"
    }
  ],
  "pagination": { "page": 1, "limit": 20, "total": 100 }
}
```

> 🟡 **補足**: 既存 `ApiOk<T>` は `{ success, data }` のみで `pagination` を持たないため、本タスクで `pagination` を含む一覧専用レスポンス型（例: `ApiList<T>` または `ListItemsResponse`）を新規定義するか、ハンドラで明示的に JSON を組み立てる必要がある。型の最終決定はテストケース／設計フェーズで確定する。
> *参照元: `models/response.rs`（ApiOk定義）, note.md 6.注意事項*

### 2.3 出力（エラーレスポンス）

- 🔵 不正なクエリパラメータ値（例: `page=abc`, `media_type=invalid`）は Axum のデシリアライズエラーとして **400 Bad Request** を返す。
  - *参照元: TASK-0010.md 注意事項, note.md 6.注意事項*
- 🟡 DBエラー時は既存 `db_error()` パターンに準じ `INTERNAL_ERROR`（500）を返す。
  - *参照元: `repositories/item_repository.rs` db_error関数, note.md 2.開発ルール*

### 2.4 データフロー

🔵 `GET /items` → `list_items_handler`（クエリ抽出・limit/pageの正規化）→ `item_repository::list_items`（動的WHERE句＋LIMIT/OFFSETでデータ取得）＋ `item_repository::count_items`（同条件でCOUNT(*)）→ `{ data, pagination }` 構築 → 200。
*参照元: note.md 1.技術スタック・3.関連実装, architecture.md*

- **参照したEARS要件**: REQ-001
- **参照した設計文書**: `api-endpoints.md`（GET /items, ページネーション）, `models/item.rs`（Item, MediaType, ItemStatus）, `models/response.rs`（ApiOk）

---

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

- 🟡 **パフォーマンス要件**: 数千件規模で1秒以内に応答すること（NFR-002）。`idx_items_media_type`, `idx_items_status`, `idx_items_is_favorite` のインデックスを活用するクエリ形状にすること。
  - *参照元: note.md 6.注意事項（パフォーマンス要件）, TASK-0010.md 完了条件, database-schema.sql L70-73*
- 🔵 **セキュリティ要件（SQLインジェクション対策）**: 動的WHERE句構築には `sqlx::QueryBuilder` を用い、値はすべて `bind()` でパラメータ化する。テーブル名・カラム名等の識別子は固定文字列のみ使用し、外部入力を直接埋め込まない。
  - *参照元: note.md 6.注意事項（セキュリティ）, TASK-0010.md 実装詳細2*
- 🔵 **認証制約**: 本エンドポイントはユーザー認証・APIキー認証を要求しない（APIキー認証は `/internal/*` のみ）。
  - *参照元: api-endpoints.md 認証, note.md 6.注意事項（セキュリティ）*
- 🔵 **アーキテクチャ制約**: Layered Architecture（routes → handlers → repositories → DB）を維持し、DBアクセスは repositories 層に集約する。ハンドラ署名は `async fn handler(State(state): State<AppState>, ...) -> Result<impl IntoResponse, ApiError>` に準拠。
  - *参照元: note.md 2.開発ルール（実装パターン）, architecture.md*
- 🔵 **データベース制約**:
  - `items` テーブル（id, media_type, title, status, is_favorite 等）を主対象とする。
  - `tag_id` 指定時は `item_tags(item_id, tag_id)`、`category_id` 指定時は `item_categories(item_id, category_id)` 中間テーブルを JOIN またはサブクエリ（`EXISTS`/`IN`）で絞り込む。多対多のため重複排除（DISTINCT/EXISTS等）に留意する。
  - *参照元: database-schema.sql, note.md 4・6.注意事項*
- 🔵 **API制約**:
  - クエリパラメータはすべて optional。
  - 各フィルタは AND 結合。
  - `limit` は 1〜100 にクランプ、`OFFSET = (page - 1) * limit`、`total` は同条件 `COUNT(*)`。
  - *参照元: api-endpoints.md GET /items・ページネーション, TASK-0010.md 完了条件*
- 🟡 **境界値の方針（要設計確定）**:
  - `limit > 100` → 100にクランプ（🔵 明示）。
  - `limit = 0` / `page = 0` 等の不正値・下限の扱いは TASK-0010 で明示されておらず、テストケース／設計フェーズで方針確定が必要。
  - *参照元: note.md 6.注意事項（既知の検討項目）*

- **参照したEARS要件**: REQ-001, NFR-002
- **参照した設計文書**: `architecture.md`, `database-schema.sql`（items, item_tags, item_categories, インデックス）, `api-endpoints.md`

---

## 4. 想定される使用例（Edgeケース・データフローベース）

### 4.1 基本的な使用パターン

- 🔵 **UC-1（絞り込みなし）**: `GET /items` → デフォルト `page=1, limit=20`、全件のうち先頭20件 + `pagination.total`=全件数を返す。（TC-001）
- 🔵 **UC-2（media_type絞り込み）**: `GET /items?media_type=anime` → anime種別のitemのみ返す。（TC-002）
- 🟡 **UC-3（複数条件のAND絞り込み）**: `GET /items?media_type=anime&is_favorite=true` → 両条件を満たすitemのみ返す。（TC-003）
- 🟡 **UC-4（tag_id絞り込み）**: `GET /items?tag_id=<uuid>` → `item_tags` 経由で当該タグを持つitemのみ返す（実DB統合テスト対象）。
- 🟡 **UC-5（category_id絞り込み）**: `GET /items?category_id=<uuid>` → `item_categories` 経由で当該カテゴリのitemのみ返す（実DB統合テスト対象）。
  - *参照元: TASK-0010.md 単体テスト要件・統合テスト要件*

### 4.2 ページネーション・境界値

- 🟡 **UC-6（limit最大クランプ）**: `GET /items?limit=500` → 実際に適用される `limit` は 100。レスポンス `pagination.limit` も 100。（TC-004）
- 🟡 **UC-7（page指定）**: `GET /items?page=2&limit=20` → 21〜40件目を返す（`OFFSET=20`）。
- 🟡 **UC-8（範囲外page）**: 件数を超えるpage指定時は空配列 + 正しい `total` を返す。
  - *参照元: TASK-0010.md テストケース4, note.md 6.注意事項*

### 4.3 エラーケース

- 🔵 **EC-1（不正なクエリ値）**: `GET /items?page=abc` または `media_type=invalid` → 400 Bad Request（Axumデシリアライズエラー）。
- 🟡 **EC-2（DBエラー）**: DB接続障害等 → 500 `INTERNAL_ERROR`（詳細はサーバーログのみ、クライアントへは汎用メッセージ）。
  - *参照元: TASK-0010.md 注意事項, note.md 6.注意事項*

- **参照したEARS要件**: REQ-001
- **参照した設計文書**: `api-endpoints.md`（GET /items）, `database-schema.sql`（item_tags, item_categories）

---

## 5. EARS要件・設計文書との対応関係

- **参照したユーザストーリー**: user-stories 1.4（コレクション一覧・絞り込み）
- **参照した機能要件**: REQ-001（一覧・絞り込み）
- **参照した非機能要件**: NFR-002（数千件規模で1秒以内応答）
- **参照したEdgeケース**: limit上限クランプ、範囲外page、不正クエリ値（TASK-0010 注意事項由来）
- **参照した受け入れ基準（TASK-0010 完了条件）**:
  - 全クエリパラメータをoptionalで受け付ける
  - 各フィルタはAND結合
  - page=1デフォルト、limit=20デフォルト・最大100
  - レスポンスが `{ "success": true, "data": [...], "pagination": {...} }` 形式
  - `idx_items_media_type` / `idx_items_status` / `idx_items_is_favorite` を活用するクエリ
- **参照した単体テスト要件**: TC-001（絞り込みなし）, TC-002（media_type）, TC-003（AND絞り込み）, TC-004（limitクランプ）
- **参照した設計文書**:
  - **アーキテクチャ**: `docs/design/mediavault-backend/architecture.md`（Layered Architecture, インデックス活用）
  - **データフロー**: routes → handlers → repositories → DB（note.md 1.技術スタック）
  - **型定義**: `backend/mediavault-api/src/models/item.rs`（Item, MediaType, ItemStatus）, `backend/mediavault-api/src/models/response.rs`（ApiOk, ApiError）
  - **データベース**: `docs/design/mediavault-backend/database-schema.sql`（items L45-73, item_tags, item_categories, idx_items_*）
  - **API仕様**: `docs/design/mediavault-backend/api-endpoints.md`（GET /items L63-71, ページネーション L47-55）

### 実装対象ファイル（相対パス）

- `backend/mediavault-api/src/models/item.rs` — `ListItemsQuery` 構造体追加（＋必要に応じ一覧レスポンス型）
- `backend/mediavault-api/src/handlers/items.rs` — `list_items_handler` 追加
- `backend/mediavault-api/src/repositories/item_repository.rs` — `list_items` / `count_items` 関数追加
- `backend/mediavault-api/src/routes/mod.rs` — `.route("/items", get(...))` 追加

---

## 6. 品質判定

```
✅ 高品質:
- 要件の曖昧さ: ほぼなし（limit=0/page=0境界の方針のみ設計フェーズで確定 → 🟡として明示済み）
- 入出力定義: 完全（クエリパラメータ表・成功/エラーレスポンス・データフローを定義）
- 制約条件: 明確（パフォーマンス/セキュリティ/アーキテクチャ/DB/API制約を列挙）
- 実装可能性: 確実（依存TASK-0008/0009の既存パターンに準拠、追加ファイル・関数を特定済み）
- 信頼性レベル分布: 🔵中心（主要要件はすべて設計文書・タスク・既存実装に裏付け）
```

**信頼性レベル分布の概況**:
- 🔵 青信号: 機能概要・入力仕様・主要制約・基本ユースケース・エラーケース（大多数）
- 🟡 黄信号: pagination専用レスポンス型の確定方法、limit=0/page=0境界、複数条件AND・tag/category絞り込みの細部
- 🔴 赤信号: なし

**未確定事項（テストケース／設計フェーズで確定）**:
1. `pagination` を含む一覧レスポンス型の定義方法（新規型 vs ハンドラでJSON組み立て）
2. `limit = 0` / `page = 0` など下限境界値の扱い
3. tag_id / category_id 絞り込みのSQLパターン（JOIN + DISTINCT vs EXISTS サブクエリ）
