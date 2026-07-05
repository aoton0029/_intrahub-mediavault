# items

`backend/mediavault-api/migrations/20260623000001_init_schema.up.sql` / `backend/mediavault-api/src/models/item.rs`

## DBスキーマ

### items（共通項目テーブル）

| カラム | 型 | NULL | 備考 |
|---|---|---|---|
| id | UUID PK | NOT NULL | `gen_random_uuid()` |
| media_type | media_type | NOT NULL | |
| title | VARCHAR(500) | NOT NULL | CHECK `title <> ''` (`chk_items_title_not_empty`) |
| original_title | VARCHAR(500) | NULL | |
| description | TEXT | NULL | |
| cover_image_url | VARCHAR(1000) | NULL | |
| release_date | DATE | NULL | |
| homepage_url | VARCHAR(1000) | NULL | |
| status | item_status | NOT NULL | DEFAULT `not_started` |
| consumed_date | DATE | NULL | |
| rating | REAL | NULL | |
| is_favorite | BOOLEAN | NOT NULL | DEFAULT false |
| source | item_source | NOT NULL | CHECK `chk_items_source_external_id`: `manual` または (`api` かつ `external_id NOT NULL`) |
| external_id | VARCHAR(255) | NULL | |
| created_at | TIMESTAMP | NOT NULL | DEFAULT CURRENT_TIMESTAMP |
| updated_at | TIMESTAMP | NOT NULL | DEFAULT CURRENT_TIMESTAMP、トリガー`trg_items_updated_at`で自動更新 |

インデックス: `idx_items_media_type`, `idx_items_status`, `idx_items_is_favorite`, `idx_items_external_id`

### メディア別詳細テーブル（`item_id`をPK兼FKとした1:1、`ON DELETE CASCADE`）

| テーブル | カラム |
|---|---|
| anime_details | item_id PK/FK, episode_count INTEGER, season_count INTEGER, studio VARCHAR(255), genre_list TEXT[] NOT NULL DEFAULT '{}', source_type VARCHAR(100), jikan_id VARCHAR(100) |
| movie_details | item_id PK/FK, runtime_minutes INTEGER, director VARCHAR(255), genre_list TEXT[] NOT NULL DEFAULT '{}', tmdb_id VARCHAR(100) |
| drama_details | item_id PK/FK, episode_count INTEGER, season_count INTEGER, network VARCHAR(255), genre_list TEXT[] NOT NULL DEFAULT '{}', tmdb_id VARCHAR(100) |
| manga_details | item_id PK/FK, volume_count INTEGER, chapter_count INTEGER, author VARCHAR(255), illustrator VARCHAR(255), magazine VARCHAR(255), jikan_id VARCHAR(100) |
| novel_details | item_id PK/FK, volume_count INTEGER, author VARCHAR(255), publisher VARCHAR(255), isbn VARCHAR(50), openlibrary_id VARCHAR(100), google_books_id VARCHAR(100) |
| game_details | item_id PK/FK, platform_list TEXT[] NOT NULL DEFAULT '{}', developer VARCHAR(255), publisher VARCHAR(255), steam_appid VARCHAR(100), igdb_id VARCHAR(100) |
| academic_book_details | item_id PK/FK, author VARCHAR(255), publisher VARCHAR(255), isbn VARCHAR(50), ndl_id VARCHAR(100), google_books_id VARCHAR(100) |
| paper_details | item_id PK/FK, doi VARCHAR(255), journal_name VARCHAR(255), volume_issue VARCHAR(100), page_range VARCHAR(100), author_list TEXT[] NOT NULL DEFAULT '{}', ndl_id VARCHAR(100) |

これら詳細テーブルにRust側の専用structは定義されておらず、ハンドラ層で`serde_json::Value`として`items`本体とは別に読み書きし、`ItemDetail.detail`にラップして返す。

## Rustモデル（`src/models/item.rs`）

### ENUM

```rust
#[sqlx(type_name = "media_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MediaType { Anime, Movie, Drama, Manga, Novel, Game, AcademicBook, Paper }

#[sqlx(type_name = "item_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus { NotStarted, InProgress, Completed }

#[sqlx(type_name = "item_source", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ItemSource { Api, Manual }
```

### struct Item（`sqlx::FromRow`）

items テーブル1行そのまま。フィールドはDBスキーマと同一（id, media_type, title, original_title, description, cover_image_url, release_date, homepage_url, status, consumed_date, rating, is_favorite, source, external_id, created_at, updated_at）。

### struct TagRef / CategoryRef（`sqlx::FromRow`）

`{ id: Uuid, name: String }`。`GET /items/:id`レスポンスでタグ/カテゴリの簡易表現として使う。

### struct ItemDetail

`Item`の全フィールド + `detail: Option<serde_json::Value>`（メディア別詳細） + `tags: Vec<TagRef>` + `categories: Vec<CategoryRef>` + `calibre_links: Vec<CalibreWebLinkInfo>`（`item_file.rs`定義、calibre_book_id設定済みPDFのみ付加）。`ItemDetail::from_parts` / `from_parts_with_calibre_links` で `Item` + 付随データから合成する。

## リクエストDTO・バリデーション

- `CreateItemRequest { media_type, title, original_title, description, cover_image_url, release_date, homepage_url, rating, is_favorite, details: Option<serde_json::Value>, consumed_date }` — `source`はサーバー側で`manual`固定、`external_id`はNULL固定（ハンドラ側で付与）。`parse_create_item_request`で`title`非空検証（`validate_title`: trim().is_empty()で拒否）。
- `ListItemsQuery { media_type, tag_id, category_id, is_favorite, status, title, page, limit }` — 一覧フィルタ用クエリDTO、全フィールドOptional。
- `UpdateItemRequest { title, original_title, description, cover_image_url, release_date, homepage_url, status, consumed_date, rating, is_favorite }` — `media_type`/`source`/`external_id`は変更不可のため除外。全フィールドNoneなら`has_any_update_field`がfalseを返しUPDATE文をスキップする設計。`validate_update_title`はtitleがSome("")の場合のみ拒否。
- `UpdateStatusRequest { status, consumed_date }` — `PATCH /items/:id/status`専用。`status`必須。

## 参照

APIレスポンスのフィールド仕様は [mediavault-api/data-model.md](../mediavault-api/data-model.md#item--itemdetail)、エンドポイント例は [mediavault-api/items.md](../mediavault-api/items.md) を参照。
