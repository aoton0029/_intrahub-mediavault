# item_links

`backend/mediavault-api/migrations/20260623000002_add_relation_tables.up.sql` / `backend/mediavault-api/src/models/item_link.rs`

## DBスキーマ

### item_links

| カラム | 型 | NULL | 備考 |
|---|---|---|---|
| id | UUID PK | NOT NULL | `gen_random_uuid()` |
| item_id | UUID FK → items(id) ON DELETE CASCADE | NOT NULL | |
| url | VARCHAR(1000) | NOT NULL | |
| label | VARCHAR(255) | NOT NULL | |
| created_at | TIMESTAMP | NOT NULL | DEFAULT CURRENT_TIMESTAMP |

インデックス: `idx_item_links_item_id`

## Rustモデル

- `ItemLink { id, item_id, url: String, label: String, created_at }`（`sqlx::FromRow`）
- `CreateItemLinkRequest { url, label }` — 両方必須。
- `parse_create_item_link_request(request) -> Result<CreateItemLinkRequest, ApiError>` — `url`/`label`いずれも空文字（trim後）は`VALIDATION_ERROR`で拒否。

## 参照

エンドポイント例は [mediavault-api/item-links.md](../mediavault-api/item-links.md) を参照。
