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
- **成功レスポンス** (200): `PaginatedOk<ItemWithRefs[]>`（`ItemWithRefs` = `Item`の全フィールド + `tags: TagRef[]` + `categories: CategoryRef[]`。フロントエンドのカードUIでタグピル表示に使う）

```json
{
  "success": true,
  "data": [
    {
      "id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001",
      "media_type": "anime",
      "title": "作品A",
      "original_title": null,
      "description": "あらすじ",
      "cover_image_url": "https://img.annict.com/xxx.jpg",
      "release_date": "2023-04-01",
      "homepage_url": null,
      "status": "in_progress",
      "consumed_date": null,
      "rating": 8.5,
      "is_favorite": true,
      "source": "api",
      "external_id": "12345",
      "created_at": "2026-07-01T12:00:00",
      "updated_at": "2026-07-01T12:00:00",
      "tags": [{ "id": "5a1e...", "name": "お気に入り原作" }],
      "categories": [{ "id": "9c2f...", "name": "2026年視聴" }]
    }
  ],
  "pagination": {
    "limit": 20,
    "has_more": true,
    "next_after_created_at": "2026-07-01T12:00:00",
    "next_after_id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001"
  }
}
```

`pagination`は`{ limit: number, has_more: boolean, next_after_created_at: string | null, next_after_id: string | null }`。`has_more=false`のとき`next_after_created_at`/`next_after_id`は`null`。件数の総数（`total`）は返さない（COUNT(*)クエリを避けるため）。

## GET /items/counts-by-media-type
サイドバー表示用に、メディア種別ごとのアイテム件数を集計して返す。`/items/{id}` より前にルーティング登録。

- **認証**: 不要
- **成功レスポンス** (200): `ApiOk<MediaTypeCounts>`

```json
{
  "success": true,
  "data": {
    "anime": 42,
    "movie": 10,
    "drama": 3,
    "manga": 87,
    "novel": 15,
    "game": 21,
    "academic_book": 2,
    "paper": 1,
    "total": 181
  }
}
```

## POST /items
アイテム新規作成。

- **認証**: 不要
- **リクエストボディ** (`CreateItemRequest`):
  - `media_type` (必須, string) — `anime` / `movie` / `drama` / `manga` / `novel` / `game` / `academic_book` / `paper`
  - `title` (必須, string) — 空白のみは400
  - `original_title` / `description` / `cover_image_url` / `homepage_url` (optional, string)
  - `release_date` / `consumed_date` (optional, `"YYYY-MM-DD"`)
  - `rating` (optional, number)
  - `is_favorite` (optional, bool)
  - `details` (optional, 任意形状のJSONオブジェクト) — スキーマは強制されない自由形式の`serde_json::Value`。`media_type`との整合性チェックはサーバー側で行わない（呼び出し側の責任）。実データ上の慣例的な形は下記「[details（media_type別JSON形状）](#detailsmedia_type別json形状)」を参照
- **成功レスポンス** (201): `ApiOk<Item>`

```json
{
  "success": true,
  "data": {
    "id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001",
    "media_type": "manga",
    "title": "作品B",
    "original_title": null,
    "description": null,
    "cover_image_url": null,
    "release_date": null,
    "homepage_url": null,
    "status": "not_started",
    "consumed_date": null,
    "rating": null,
    "is_favorite": false,
    "source": "manual",
    "external_id": null,
    "created_at": "2026-07-09T09:00:00",
    "updated_at": "2026-07-09T09:00:00"
  }
}
```

- **エラー**: 400 `VALIDATION_ERROR`

## GET /items/search
外部プロバイダAPIを横断検索する（`media_type` に応じてプロバイダを自動選択）。`/items/{id}` より前にルーティング登録。検索段階では軽量な候補一覧のみを返し、詳細情報は `POST /items/import` 実行時にサーバー側で該当プロバイダから再取得する。

- **認証**: 不要
- **クエリパラメータ** (`ItemSearchQuery`): `media_type` (必須), `q` (必須, 検索語)
- **media_type → プロバイダ振り分け**:
  - `anime` → Annict（キー必須）
  - `manga` / `novel` / `academic_book` → 楽天ブックス（キー必須、`applicationId:accessKey`形式で登録）
  - `movie` / `drama` → TMDb（キー必須。`movie`はSearch Movie、`drama`はSearch TV）
  - `game` → Steamストア検索（キー不要）
  - `paper` → NDL（国立国会図書館サーチ、キー必須）
- **成功レスポンス** (200): `ApiOk<SearchResultItem[]>`

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | string | プロバイダ固有ID（`mal_id` / `annict work id` / `tmdb id` / `steam appid` / `isbn` 等、`media_type`により意味が異なる）。`POST /items/import` の `external_id` にそのまま渡す |
| `media_type` | string | リクエストと同じ`media_type`が入る |
| `provider` | string \| null | 採用プロバイダ（`annict` / `rakuten` / `tmdb` / `steam` / `ndl`）。`POST /items/import` の `provider` にそのまま渡せる |
| `title` | string | タイトル |
| `thumbnail_url` | string \| null | サムネイル画像URL |

```json
{
  "success": true,
  "data": [
    {
      "id": "12345",
      "media_type": "anime",
      "provider": "annict",
      "title": "鬼滅の刃",
      "thumbnail_url": "https://img.annict.com/xxx.jpg"
    }
  ]
}
```

- **エラー**: 422 `API_KEY_NOT_CONFIGURED`, 502 `EXTERNAL_API_TIMEOUT` / `EXTERNAL_API_ERROR`

### details（media_type別JSON形状）

`Item.details`（DBカラム）/ `ItemDetail.detail`（レスポンス）は型定義された共通スキーマを持たない自由形式のJSON（`Option<serde_json::Value>`）。手動作成時（`POST /items` で`details`省略）は`null`。`POST /items/import`経由の場合、以下の形は各`build_*_create_request`関数（`services/external_search.rs`）が実際に組み立てている内容そのもの（`media_type`/`provider`/`title`等、`Item`本体側に既にある情報は重複して含まない）。値が取得できなかったフィールドは`null`または空配列になる。

- `anime`（Annictの作品情報 + Jikan(MyAnimeList)のあらすじ・ジャンル等をマージ）:
  `episodes`, `status`, `season`, `year`, `studios[]`, `source`, `duration`, `trailer_url`, `genres[]`, `rating`, `url`, `alternative_titles[]`
- `manga` / `novel` / `academic_book`（楽天ブックス、`external_id`はISBN）:
  `authors`, `publisher`, `isbn`, `series_name`
- `paper`（NDL、`external_id`はISBN。楽天ではなくNDL経由な点に注意）:
  `authors`, `publisher`, `isbn`（`series_name`は含まない）
- `movie`（TMDb `GET /movie/{id}`）:
  `runtime_minutes`, `original_language`, `vote_count`, `collection`, `production_companies[]`, `genres[]`, `rating`
- `drama`（TMDb `GET /tv/{id}`）:
  `number_of_seasons`, `number_of_episodes`, `networks[]`, `status`, `original_language`, `first_air_date`, `last_air_date`, `genres[]`, `rating`
- `game`（Steam `appdetails`）:
  `platforms[]`, `developers[]`, `publishers[]`, `screenshots[]`, `metacritic`, `genres[]`

例（`anime`）:
```json
{
  "episodes": 26,
  "status": "Finished Airing",
  "season": "2019-spring",
  "year": 2019,
  "studios": ["ufotable"],
  "source": "Manga",
  "duration": "23 min per ep",
  "trailer_url": "https://youtube.com/watch?v=xxx",
  "genres": ["Action", "Fantasy"],
  "rating": 8.5,
  "url": "https://myanimelist.net/anime/xxxxx",
  "alternative_titles": ["Demon Slayer: Kimetsu no Yaiba"]
}
```

例（`manga`）:
```json
{
  "authors": "作者太郎",
  "publisher": "集英社",
  "isbn": "9784088xxxxxx",
  "series_name": "作品B"
}
```

## POST /items/import
`GET /items/search` の検索結果から選択した1件をIDで指定し、サーバー側で該当プロバイダから詳細情報を再取得してアイテムを作成する（`items` + `source=api`で登録）。`/items/{id}` より前にルーティング登録。

- **認証**: 不要
- **リクエストボディ** (`ImportByIdRequest`):
  - `media_type` (必須, string)
  - `provider` (optional, string) — `GET /items/search` のレスポンス要素の`provider`をそのまま渡せるが、実際のプロバイダ選択は`media_type`から自動決定されるため未使用でもよい
  - `external_id` (必須, string) — `GET /items/search` のレスポンス要素の`id`。空文字・空白のみは400
- サーバー内部フロー: `ExternalSearchService::fetch_import_details(media_type, external_id)` で対応プロバイダから詳細を再取得し `CreateItemRequest` を構築（`details`は上記「[details（media_type別JSON形状）](#detailsmedia_type別json形状)」の形）→ 重複チェック（同一`media_type`+`external_id`が既存の場合409）→ DB登録
- **成功レスポンス** (201): `ApiOk<Item>`（`POST /items`と同形。`source: "api"`, `external_id`にインポート元IDが入る）
- **エラー**: 400 `VALIDATION_ERROR`（`external_id`欠落・空文字）, 404（プロバイダ側で対象が見つからない）, 409 `ITEM_ALREADY_IMPORTED`, 422 `API_KEY_NOT_CONFIGURED`, 502 `EXTERNAL_API_TIMEOUT` / `EXTERNAL_API_ERROR`

## GET /items/{id}
アイテム詳細取得（関連情報含む）。

- **認証**: 不要
- **パスパラメータ**: `id` (uuid)
- **成功レスポンス** (200): `ApiOk<ItemDetail>`
  - `Item`の全フィールド
  - `detail`: 上記「[details（media_type別JSON形状）](#detailsmedia_type別json形状)」参照。`details`未保存のアイテムでは`null`
  - `tags: TagRef[]`（`{id, name}`）
  - `categories: CategoryRef[]`（`{id, name}`）
  - `calibre_links: CalibreWebLinkInfo[]`（`{file_id, calibre_book_id}`。`calibre_book_id`設定済みのPDF `item_files`のみ）
  - `streaming_links: ItemStreamingLink[]`（`{id, item_id, platform, url, created_at}`。`platform`は`netflix` / `amazon_prime` / `disney_plus` / `dmm_tv` / `apple_tv`）

```json
{
  "success": true,
  "data": {
    "id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001",
    "media_type": "anime",
    "title": "作品A",
    "original_title": null,
    "description": "あらすじ",
    "cover_image_url": "https://img.annict.com/xxx.jpg",
    "release_date": "2023-04-01",
    "homepage_url": null,
    "status": "in_progress",
    "consumed_date": null,
    "rating": 8.5,
    "is_favorite": true,
    "source": "api",
    "external_id": "12345",
    "created_at": "2026-07-01T12:00:00",
    "updated_at": "2026-07-01T12:00:00",
    "detail": {
      "episodes": 26,
      "status": "Finished Airing",
      "season": "2019-spring",
      "year": 2019,
      "studios": ["ufotable"],
      "source": "Manga",
      "duration": "23 min per ep",
      "trailer_url": "https://youtube.com/watch?v=xxx",
      "genres": ["Action", "Fantasy"],
      "rating": 8.5,
      "url": "https://myanimelist.net/anime/xxxxx",
      "alternative_titles": ["Demon Slayer: Kimetsu no Yaiba"]
    },
    "tags": [{ "id": "5a1e...", "name": "お気に入り原作" }],
    "categories": [{ "id": "9c2f...", "name": "2026年視聴" }],
    "calibre_links": [],
    "streaming_links": [
      { "id": "e1f2...", "item_id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001", "platform": "netflix", "url": "https://netflix.com/title/xxx", "created_at": "2026-07-02T00:00:00" }
    ]
  }
}
```

- **エラー**: 404 `ITEM_NOT_FOUND`, 400 `VALIDATION_ERROR`（UUID形式不正）

## PATCH /items/{id}
アイテム更新（部分更新）。

- **認証**: 不要
- **リクエストボディ** (`UpdateItemRequest`): 全フィールド Optional（`media_type` / `source` / `external_id` は含まれない = 変更不可）
- **成功レスポンス** (200): `ApiOk<Item>`（レスポンス形は`POST /items`と同じ）
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

```json
{
  "success": true,
  "data": {
    "id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001",
    "media_type": "anime",
    "title": "作品A",
    "status": "completed",
    "consumed_date": "2026-07-09",
    "is_favorite": true,
    "source": "api",
    "external_id": "12345",
    "created_at": "2026-07-01T12:00:00",
    "updated_at": "2026-07-09T10:00:00"
  }
}
```

- **エラー**: 404 `ITEM_NOT_FOUND`
