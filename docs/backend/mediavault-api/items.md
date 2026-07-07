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
- **リクエストボディ** (`CreateItemRequest`): 共通フィールド + `details` (optional)。`details` を指定する場合は `MediaDetails`（下記参照）として妥当で、かつ内側の `media_type` がリクエスト本体の `media_type` と一致する必要がある（不正形式・不一致は 400）
- **成功レスポンス** (201): `ApiOk<Item>`
- **エラー**: 400 `VALIDATION_ERROR`

## GET /items/search
外部プロバイダAPIを横断検索する（`media_type` に応じてプロバイダを自動選択）。`/items/{id}` より前にルーティング登録。

- **認証**: 不要
- **クエリパラメータ** (`ItemSearchQuery`): `media_type` (必須), `q` (必須, 検索語)
- **成功レスポンス** (200): `ApiOk<MediaDetails[]>`
- **エラー**: 422 `API_KEY_NOT_CONFIGURED`, 502 `EXTERNAL_API_TIMEOUT` / `EXTERNAL_API_ERROR`

### MediaDetails（ノーマライズ済み詳細モデル）

各要素はプロバイダ生データ（旧 `raw_data`）を含まない、正規化済みのフラットな JSON オブジェクト。`media_type` が判別子を兼ねる。

**共通フィールド（MediaCore、全 media_type 共通）**:

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `media_type` | string | `anime` / `manga` / `movie` / `drama` / `game` / `novel` / `academic_book` / `paper`（判別子） |
| `provider` | string \| null | 採用プロバイダ（`tmdb` / `igdb` / `ndl` 等）。Jikan（キー不要）は `null` |
| `external_id` | string | プロバイダ固有 ID（mal_id / tmdb id 等の元値） |
| `title` | string | タイトル |
| `original_title` | string \| null | 原題 |
| `alternative_titles` | string[] | 別題（英題・シノニム等） |
| `description` | string \| null | あらすじ・概要 |
| `release_date` | string \| null | リリース日（精度はプロバイダ依存: 年のみ〜完全日付） |
| `image_url` | string \| null | 代表画像の完全 URL |
| `genres` | string[] | ジャンル名。**TMDb 検索結果は genre 名を含まないため movie/drama では空配列** |
| `rating` | number \| null | 0–10 に正規化した評価値 |
| `url` | string \| null | プロバイダ側の作品ページ URL |

**media_type 別の拡張フィールド**:

- `anime`: `episodes`, `status`, `season`, `year`, `studios[]`, `source`, `duration`, `trailer_url`
- `manga`: `chapters`, `volumes`, `status`, `authors[]`, `serializations[]`
- `movie`: `runtime_minutes`, `original_language`, `vote_count`, `collection`, `production_companies[]`
- `drama`: `number_of_seasons`, `number_of_episodes`, `networks[]`, `status`, `original_language`, `first_air_date`, `last_air_date`
- `game`: `platforms[]`, `developers[]`, `publishers[]`, `screenshots[]`, `metacritic`, `steam_appid`, `storyline`
- `novel` / `academic_book` / `paper`（書誌共通形状）: `authors[]`, `publisher`, `isbn`, `page_count`, `physical_format`

## POST /items/import
外部検索結果からアイテムをインポートして作成する。`/items/{id}` より前にルーティング登録。

- **認証**: 不要
- **リクエストボディ** (`MediaDetails`): `GET /items/search` が返す検索結果要素と同形（上記参照）。必須は `media_type` / `external_id` / `title`。`image_url`→`cover_image_url`、`url`→`homepage_url`、`release_date`（文字列。年のみは1月1日として解釈、解釈不能は無視）としてアイテムへマッピングされ、ノーマライズ済み JSON 全体が `items.details`（JSONB）へ永続化される（`GET /items/{id}` の `detail` として返る）
- **成功レスポンス** (201): `ApiOk<Item>`
- **エラー**: 409 `ITEM_ALREADY_IMPORTED`, 400 `VALIDATION_ERROR`

## GET /items/{id}
アイテム詳細取得（関連情報含む）。

- **認証**: 不要
- **パスパラメータ**: `id` (uuid)
- **成功レスポンス** (200): `ApiOk<ItemDetail>`
  - `detail`: `MediaDetails | null` — インポート/作成時に保存されたノーマライズ済み JSON（`GET /items/search` の要素と同形。共通フィールド＋media_type 別拡張フィールドは上記「MediaDetails」参照）。`details` 未保存のアイテム（手動作成で `details` 省略・details 永続化導入前のインポート）では `null`
  - その他: `Item` の全フィールド + `tags: TagRef[]` + `categories: CategoryRef[]` + `calibre_links: CalibreWebLinkInfo[]`
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
