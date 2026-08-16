# MediaVault API データモデル（レスポンスstruct一覧）

各エンドポイントのレスポンスJSONに登場するstructのフィールド一覧。DB定義の正本は migration、DTO・バリデーションの正本は実装を参照する。エンドポイントごとの例は各カテゴリ別ドキュメント（[index.md](./index.md)参照）を参照。

## Item / ItemDetail

`GET /items`, `GET /items/:id` 等で返す。詳細: [items.md](./items.md)

**Item**

| フィールド | 型 |
|---|---|
| id | UUID |
| media_type | MediaType（anime/movie/drama/manga/novel/game/academic_book/paper） |
| title | string |
| original_title | string \| null |
| description | string \| null |
| cover_image_url | string \| null |
| release_date | date \| null |
| homepage_url | string \| null |
| status | ItemStatus（not_started/in_progress/completed） |
| consumed_date | date \| null |
| rating | number \| null |
| is_favorite | boolean |
| source | ItemSource（api/manual） |
| external_id | string \| null |
| created_at / updated_at | datetime |

**ItemDetail**（`GET /items/:id`用。Itemの全フィールド + 以下）

| フィールド | 型 |
|---|---|
| detail | object \| null（メディア別詳細。media_typeに応じたキー集合） |
| tags | TagRef[]（`{ id, name }`） |
| categories | CategoryRef[]（`{ id, name }`） |
| calibre_links | CalibreWebLinkInfo[]（`{ file_id, calibre_book_id }`。calibre_book_id設定済みPDFのみ） |
| streaming_links | ItemStreamingLink[]（配信サービスURL。詳細は下記「ItemStreamingLink」参照） |
| images | ItemImage[]（画像URL。詳細は下記「ItemImage」参照） |
| theme_songs | ItemThemeSong[]（OP/ED等。映像作品以外は空配列。詳細は下記「ThemeSong / ItemThemeSong」参照） |

## Tag / Category / Mylist

詳細: [tags.md](./tags.md), [categories.md](./categories.md), [mylists.md](./mylists.md)

| struct | フィールド |
|---|---|
| Tag | id: UUID, name: string |
| TagWithCount | id: UUID, name: string, item_count: number（`GET /tags`用。item_tags経由の付与件数） |
| Category | id: UUID, name: string |
| CategoryWithCount | id: UUID, name: string, item_count: number（`GET /categories`用。item_categories経由の付与件数） |
| Mylist | id: UUID, name: string, created_at: datetime |

## ItemWithRefs / MediaTypeCounts

`GET /items`（一覧）・`GET /items/counts-by-media-type`のレスポンス。

| struct | フィールド |
|---|---|
| ItemWithRefs | Itemの全フィールド + tags: TagRef[] + categories: CategoryRef[]（一覧のカードUIでタグピル表示に使う） |
| MediaTypeCounts | anime, movie, drama, manga, novel, game, academic_book, paper, total: number（サイドバーのメディア種別件数表示に使う） |

## ItemRelation

`POST /item-relations`等のレスポンス。詳細: [item-relations.md](./item-relations.md)

| フィールド | 型 |
|---|---|
| id | UUID |
| item_id | UUID |
| related_item_id | UUID |
| relation_type | RelationType（adaptation/sequel/prequel/spinoff/dlc/reference、向きの意味は [item-relations.md](item-relations.md) 参照） |
| created_at | datetime |

## ItemLink / ItemTrailer / ItemFile

詳細: [item-links.md](./item-links.md), [item-trailers.md](./item-trailers.md), [item-files.md](./item-files.md)

| struct | フィールド |
|---|---|
| ItemLink | id: UUID, item_id: UUID, url: string, label: string, created_at: datetime |
| ItemTrailer | id: UUID, item_id: UUID, url: string, label: string \| null, created_at: datetime |
| ItemFile | id: UUID, item_id: UUID, path: string, label: string \| null, file_type: FileType（pdf/image/other）, calibre_book_id: string \| null, created_at: datetime |

## ItemStreamingLink

`GET/POST /items/{id}/streaming-links`のレスポンス。詳細: [item-streaming-links.md](./item-streaming-links.md)

| フィールド | 型 |
|---|---|
| id | UUID |
| item_id | UUID |
| platform | StreamingPlatform（netflix/amazon_prime/disney_plus/dmm_tv/apple_tv） |
| url | string |
| created_at | datetime |

## ItemImage

`GET/POST /items/{id}/images`のレスポンス。詳細: [item-images.md](./item-images.md)

| フィールド | 型 |
|---|---|
| id | UUID |
| item_id | UUID |
| url | string |
| created_at | datetime |

## ItemGroup / ItemEpisode

`GET /items/:id/groups`, `GET /groups/:group_id/episodes`のレスポンス。詳細: [item-groups.md](./item-groups.md), [item-episodes.md](./item-episodes.md)

| struct | フィールド |
|---|---|
| ItemGroup | id: UUID, item_id: UUID, parent_item_id: UUID \| null, group_type: GroupType（season/volume/chapter）, group_name: string, number: number \| null, display_order: number, created_at / updated_at: datetime |
| ItemEpisode | id: UUID, group_id: UUID, episode_number: number, title: string \| null, original_title: string \| null, air_date: date \| null, description: string \| null, created_at / updated_at: datetime |

## Staff / ItemStaff

`POST /staff`, `POST /items/:id/staff`のレスポンス。詳細: [staff.md](./staff.md)

| struct | フィールド |
|---|---|
| Staff | id: UUID, external_id: string \| null, name: string, image_url: string \| null, created_at: datetime |
| ItemStaff | id: UUID, item_id: UUID, staff_id: UUID, role: string, character_name: string \| null |

## ThemeSong / ItemThemeSong

`GET/POST /theme-songs`, `GET/POST /items/{id}/theme-songs`のレスポンス。曲はアイテムから独立したマスタで、`item_theme_songs`を介してアイテムと多対多に紐づく。アーティスト・作曲・作詞は正規化せず`theme_songs`の列として持つ。詳細: [theme-songs.md](./theme-songs.md)

| struct | フィールド |
|---|---|
| ThemeSong | id: UUID, title: string, artist: string \| null, composer: string \| null, lyricist: string \| null, arranger: string \| null, note: string \| null, created_at / updated_at: datetime |
| ThemeSongLink | id: UUID, theme_song_id: UUID, link_type: ThemeSongLinkType（youtube/spotify/apple_music/amazon_music/niconico/official/other）, url: string, label: string \| null, sort_order: number, created_at: datetime |
| ThemeSongWithLinks | ThemeSongの全フィールド + links: ThemeSongLink[] |
| ThemeSongDetail | ThemeSongWithLinksの全フィールド + items: ThemeSongItemRef[]（`GET /theme-songs/{id}`用。曲が使われている作品一覧） |
| ThemeSongItemRef | item_id: UUID, title: string, media_type: MediaType, theme_type: ThemeSongType |
| ItemThemeSong | id: UUID, item_id: UUID, theme_type: ThemeSongType（op/ed/insert/image/character/theme/other、意味は [theme-songs.md](./theme-songs.md) 参照）, display_order: number, created_at: datetime, theme_song: ThemeSongWithLinks |

## Cast / ItemCast

`POST /cast`, `POST /items/:id/cast`のレスポンス。声優＋役名を`staff`/`item_staff`とは別テーブル（`cast_members`/`item_cast`）で管理する。詳細: [cast.md](./cast.md)

| struct | フィールド |
|---|---|
| Cast | id: UUID, external_id: string \| null, name: string, image_url: string \| null, created_at: datetime |
| ItemCast | id: UUID, item_id: UUID, cast_id: UUID, character_name: string \| null |

## Citation

`GET/POST /items/{id}/citations`, `PATCH /citations/{id}`のレスポンス。詳細: [citations.md](./citations.md)

| フィールド | 型 |
|---|---|
| id | UUID |
| item_id | UUID |
| quote_text | string |
| note | string \| null |
| locator_type | LocatorType（page/timestamp/location/chapter/none） |
| page_number | number \| null（`locator_type=page`。書籍・論文のページ番号） |
| timestamp_seconds | number \| null（`locator_type=timestamp`。映像作品の再生秒数） |
| location_number | number \| null（`locator_type=location`。電子書籍の位置No.） |
| chapter | string \| null（`locator_type=chapter`。章・話数など） |
| created_at / updated_at | datetime |

## ApiCredential

`PUT /settings/api-keys/:provider`のレスポンス。詳細: [settings.md](./settings.md)

| フィールド | 型 |
|---|---|
| provider | ApiProvider（tmdb/igdb/ndl/steam/open_library/ani_list） |
| api_key | string |
| updated_at | datetime |

## ItemFileExtraction / ItemFileText

文字抽出の状態と現行結果。`extraction_state` は `queued`, `running`, `cancelling`, `succeeded`, `failed`, `cancelled`。

| テーブル | 主なフィールド |
|---|---|
| `item_file_extractions` | id: UUID, item_file_id: UUID (FK), state: ExtractionState, attempts / max_attempts: number, progress_current / progress_total: number, claimed_by: string \| null, lease_token: UUID \| null, lease_expires_at: datetime \| null, error: JSON \| null, created_at / updated_at: datetime |
| `item_file_texts` | id: UUID, item_file_id: UUID (FK, UNIQUE), content: string, boundaries: JSON配列 `[{start,end,label}]`, extraction_version: string, extractor: JSON object, extracted_at: datetime, created_at / updated_at: datetime |

`item_file_extractions` は active 状態に限る `item_file_id` の部分UNIQUE indexを持つ。両テーブルとも `item_files` 削除時にCASCADE削除される。

## 共通レスポンス型（`src/models/response.rs`）

| struct | フィールド |
|---|---|
| ApiOk\<T\> | success: true, data: T |
| ApiError | success: false, error: ApiErrorBody（`{ code: string, message: string }`） |
| Pagination | limit: number, has_more: boolean, next_after_created_at: string \| null, next_after_id: string \| null |
| PaginatedOk\<T\> | success: true, data: T[], pagination: Pagination |

エラーコード一覧（`ApiErrorCode`）は [index.md](./index.md#エラーコード一覧) を参照。
