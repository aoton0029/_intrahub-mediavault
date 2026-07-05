# tags

`backend/mediavault-api/migrations/20260623000002_add_relation_tables.up.sql` / `backend/mediavault-api/src/models/tag.rs`

## DBスキーマ

### tags

| カラム | 型 | NULL | 備考 |
|---|---|---|---|
| id | UUID PK | NOT NULL | `gen_random_uuid()` |
| name | VARCHAR(100) | NOT NULL | UNIQUE |

### item_tags（多対多）

| カラム | 型 | 備考 |
|---|---|---|
| item_id | UUID FK → items(id) ON DELETE CASCADE | PK(item_id, tag_id)の一部 |
| tag_id | UUID FK → tags(id) ON DELETE CASCADE | PK(item_id, tag_id)の一部 |

## Rustモデル

- `Tag { id: Uuid, name: String }`（`sqlx::FromRow`）
- `CreateTagRequest { name: String }`
- `validate_tag_name(name: &str) -> Result<(), ApiError>` — trim().is_empty()で空文字・空白のみを`VALIDATION_ERROR`で拒否。最大長（100文字）検証はDB制約に委ね、アプリ側では明示チェックしない。

## 参照

エンドポイント例は [mediavault-api/tags.md](../mediavault-api/tags.md) を参照。
