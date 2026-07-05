# item_trailers

`backend/mediavault-api/migrations/20260623000002_add_relation_tables.up.sql` / `backend/mediavault-api/src/models/item_trailer.rs`

## DBスキーマ

### item_trailers

| カラム | 型 | NULL | 備考 |
|---|---|---|---|
| id | UUID PK | NOT NULL | `gen_random_uuid()` |
| item_id | UUID FK → items(id) ON DELETE CASCADE | NOT NULL | |
| url | VARCHAR(1000) | NOT NULL | |
| label | VARCHAR(255) | NULL | |
| created_at | TIMESTAMP | NOT NULL | DEFAULT CURRENT_TIMESTAMP |

インデックス: `idx_item_trailers_item_id`

## Rustモデル（`item_link.rs`と対称構造。labelはoptional）

- `ItemTrailer { id, item_id, url: String, label: Option<String>, created_at }`（`sqlx::FromRow`）
- `CreateItemTrailerRequest { url, label: Option<String> }`
- `parse_create_item_trailer_request(request) -> Result<CreateItemTrailerRequest, ApiError>` — `url`が空文字（trim後）の場合`VALIDATION_ERROR`。

## 参照

エンドポイント例は [mediavault-api/item-trailers.md](../mediavault-api/item-trailers.md) を参照。
