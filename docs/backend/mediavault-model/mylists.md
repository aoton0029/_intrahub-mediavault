# mylists

`backend/mediavault-api/migrations/20260623000002_add_relation_tables.up.sql` / `backend/mediavault-api/src/models/mylist.rs`

## DBスキーマ

### mylists

| カラム | 型 | NULL | 備考 |
|---|---|---|---|
| id | UUID PK | NOT NULL | `gen_random_uuid()` |
| name | VARCHAR(255) | NOT NULL | |
| created_at | TIMESTAMP | NOT NULL | DEFAULT CURRENT_TIMESTAMP |

### mylist_items（多対多）

| カラム | 型 | 備考 |
|---|---|---|
| mylist_id | UUID FK → mylists(id) ON DELETE CASCADE | PK(mylist_id, item_id)の一部 |
| item_id | UUID FK → items(id) ON DELETE CASCADE | PK(mylist_id, item_id)の一部 |

## Rustモデル

- `Mylist { id: Uuid, name: String, created_at: NaiveDateTime }`（`sqlx::FromRow`）
- `CreateMylistRequest { name: String }`
- `AddMylistItemRequest { item_id: Uuid }`
- `validate_mylist_name(name: &str) -> Result<(), ApiError>` — trim().is_empty()で空文字・空白のみを拒否（`category.rs`と対称）。

## 参照

エンドポイント例は [mediavault-api/mylists.md](../mediavault-api/mylists.md) を参照。
