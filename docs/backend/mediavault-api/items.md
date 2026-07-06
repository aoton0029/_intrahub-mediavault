← [index](./index.md)

# Items API

## GET /items
アイテム一覧取得。フィルタ・keyset（カーソル）ページネーション対応。

- **認証**: 不要
- **クエリパラメータ** (`ListItemsQuery`):
  - `media_type` (string, optional)
  - `tag_id` (uuid, optional)
  - `category_id` (uuid, optional)
  - `is_favorite` (bool, optional)
  - `status` (string, optional)
  - `title` (string, optional) — 部分一致検索
  - `limit` (u32, optional, default 20, max 100)
  - `after_created_at` (string, optional, NaiveDateTime形式 例: `"2026-07-01T12:00:00"`) — 前回レスポンスの`pagination.next_after_created_at`をそのまま渡す
  - `after_id` (uuid, optional) — 前回レスポンスの`pagination.next_after_id`をそのまま渡す
  - 先頭ページを取得する場合は`after_created_at`/`after_id`を両方省略する。片方のみ指定された場合は無効なカーソルとして無視され、先頭ページとして扱われる（400にはならない）
- **成功レスポンス** (200): `PaginatedOk<ItemWithRefs[]>`（`ItemWithRefs` = `Item`の全フィールド + `tags: TagRef[]` + `categories: CategoryRef[]`。フロントエンドのカードUIでタグピル表示に使う）。
  `pagination`は`{ limit: number, has_more: boolean, next_after_created_at: string | null, next_after_id: string | null }`。`has_more=false`のとき`next_after_created_at`/`next_after_id`は`null`。件数の総数（`total`）は返さない（COUNT(*)クエリを避けるため）

## GET /items/counts-by-media-type
サイドバー表示用に、メディア種別ごとのアイテム件数を集計して返す。`/items/{id}` より前にルーティング登録。

- **認証**: 不要
- **成功レスポンス** (200): `ApiOk<MediaTypeCounts>`（`MediaTypeCounts = { anime, movie, drama, manga, novel, game, academic_book, paper, total: number }`）

## GET /items/counts-by-status
ホームダッシュボード表示用に、進行状況・お気に入りごとのアイテム件数を集計して返す。`/items/{id}` より前にルーティング登録。

- **認証**: 不要
- **成功レスポンス** (200): `ApiOk<ItemStatusCounts>`（`ItemStatusCounts = { not_started, in_progress, completed, favorite, total: number }`）。`favorite`は`status`の値に関わらず`is_favorite = true`のアイテム数を表す独立した集計値

## POST /items
アイテム新規作成。

- **認証**: 不要
- **リクエストボディ** (`CreateItemRequest`): 共通フィールド + `media_type` 別詳細フィールド
- **成功レスポンス** (201): `ApiOk<Item>`
- **エラー**: 400 `VALIDATION_ERROR`

## GET /items/search
外部プロバイダAPIを横断検索する（`media_type` に応じてプロバイダを自動選択）。`/items/{id}` より前にルーティング登録。

- **認証**: 不要
- **クエリパラメータ** (`ItemSearchQuery`): `media_type` (必須), `q` (必須, 検索語)
- **成功レスポンス** (200): `ApiOk<ExternalSearchResult[]>`
- **エラー**: 422 `API_KEY_NOT_CONFIGURED`, 502 `EXTERNAL_API_TIMEOUT` / `EXTERNAL_API_ERROR`

## POST /items/import
外部検索結果からアイテムをインポートして作成する。`/items/{id}` より前にルーティング登録。

- **認証**: 不要
- **リクエストボディ** (`ImportItemRequest`): 外部ID・media_type等
- **成功レスポンス** (201): `ApiOk<Item>`
- **エラー**: 409 `ITEM_ALREADY_IMPORTED`, 400 `VALIDATION_ERROR`

## GET /items/{id}
アイテム詳細取得（関連情報含む）。

- **認証**: 不要
- **パスパラメータ**: `id` (uuid)
- **成功レスポンス** (200): `ApiOk<ItemDetail>`
- **エラー**: 404 `ITEM_NOT_FOUND`, 400 `VALIDATION_ERROR`（UUID形式不正）

## PATCH /items/{id}
アイテム更新（部分更新）。

- **認証**: 不要
- **リクエストボディ** (`UpdateItemRequest`): 全フィールド Optional
- **成功レスポンス** (200): `ApiOk<Item>`
- **エラー**: 404 `ITEM_NOT_FOUND`, 400 `VALIDATION_ERROR`

## DELETE /items/{id}
アイテム削除。

- **認証**: 不要
- **成功レスポンス**: 204 No Content
- **エラー**: 404 `ITEM_NOT_FOUND`

## PATCH /items/{id}/status
ステータス更新（視聴済み・読了などの状態遷移）。

- **認証**: 不要
- **リクエストボディ** (`UpdateStatusRequest`): `status` (必須), `consumed_date` (optional)
- **成功レスポンス** (200): `ApiOk<Item>`
- **エラー**: 404 `ITEM_NOT_FOUND`
