# MediaVault API データモデル（レスポンスstruct一覧）

各エンドポイントのレスポンスJSONに登場するstructのフィールド一覧。DBスキーマ・ER図・トリガー・インデックス・リクエストDTO（`CreateXxxRequest`等）・バリデーションの詳細は [mediavault-model/index.md](../mediavault-model/index.md) を参照。エンドポイントごとのリクエスト/レスポンス例は各カテゴリ別ドキュメント（[index.md](./index.md)参照）を参照。

## Item / ItemDetail

`GET /items`, `GET /items/:id` 等で返す。詳細: [mediavault-model/items.md](../mediavault-model/items.md)

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

## Tag / Category / Mylist

詳細: [mediavault-model/tags.md](../mediavault-model/tags.md), [categories.md](../mediavault-model/categories.md), [mylists.md](../mediavault-model/mylists.md)

| struct | フィールド |
|---|---|
| Tag | id: UUID, name: string |
| Category | id: UUID, name: string |
| Mylist | id: UUID, name: string, created_at: datetime |

## ItemRelation

`POST /item-relations`等のレスポンス。詳細: [mediavault-model/item-relations.md](../mediavault-model/item-relations.md)

| フィールド | 型 |
|---|---|
| id | UUID |
| item_id | UUID |
| related_item_id | UUID |
| relation_type | RelationType（reference/dlc） |
| created_at | datetime |

## ItemLink / ItemTrailer / ItemFile

詳細: [mediavault-model/item-links.md](../mediavault-model/item-links.md), [item-trailers.md](../mediavault-model/item-trailers.md), [item-files.md](../mediavault-model/item-files.md)

| struct | フィールド |
|---|---|
| ItemLink | id: UUID, item_id: UUID, url: string, label: string, created_at: datetime |
| ItemTrailer | id: UUID, item_id: UUID, url: string, label: string \| null, created_at: datetime |
| ItemFile | id: UUID, item_id: UUID, path: string, label: string \| null, file_type: FileType（pdf/image/other）, calibre_book_id: string \| null, created_at: datetime |

## ItemGroup / ItemEpisode

`GET /items/:id/groups`, `GET /groups/:group_id/episodes`のレスポンス。詳細: [mediavault-model/item-groups.md](../mediavault-model/item-groups.md), [item-episodes.md](../mediavault-model/item-episodes.md)

| struct | フィールド |
|---|---|
| ItemGroup | id: UUID, item_id: UUID, parent_item_id: UUID \| null, group_type: GroupType（season/volume/chapter）, group_name: string, number: number \| null, display_order: number, created_at / updated_at: datetime |
| ItemEpisode | id: UUID, group_id: UUID, episode_number: number, title: string \| null, original_title: string \| null, air_date: date \| null, description: string \| null, created_at / updated_at: datetime |

## Staff / ItemStaff

`POST /staff`, `POST /items/:id/staff`のレスポンス。詳細: [mediavault-model/staff.md](../mediavault-model/staff.md)

| struct | フィールド |
|---|---|
| Staff | id: UUID, external_id: string \| null, name: string, image_url: string \| null, created_at: datetime |
| ItemStaff | id: UUID, item_id: UUID, staff_id: UUID, role: string, character_name: string \| null |

## ApiCredential

`PUT /settings/api-keys/:provider`のレスポンス。詳細: [mediavault-model/api-credentials.md](../mediavault-model/api-credentials.md)

| フィールド | 型 |
|---|---|
| provider | ApiProvider（tmdb/igdb/ndl/steam/open_library/ani_list） |
| api_key | string |
| updated_at | datetime |

## 共通レスポンス型（`src/models/response.rs`）

| struct | フィールド |
|---|---|
| ApiOk\<T\> | success: true, data: T |
| ApiError | success: false, error: ApiErrorBody（`{ code: string, message: string }`） |
| Pagination | page: number, limit: number, total: number |
| PaginatedOk\<T\> | success: true, data: T[], pagination: Pagination |

エラーコード一覧（`ApiErrorCode`）は [index.md](./index.md#エラーコード一覧) を参照。
