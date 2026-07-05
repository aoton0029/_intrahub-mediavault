# categories

`backend/mediavault-api/migrations/20260623000002_add_relation_tables.up.sql` / `backend/mediavault-api/src/models/category.rs`

## DBスキーマ

### categories

| カラム | 型 | NULL | 備考 |
|---|---|---|---|
| id | UUID PK | NOT NULL | `gen_random_uuid()` |
| name | VARCHAR(100) | NOT NULL | UNIQUE |

### item_categories（多対多）

| カラム | 型 | 備考 |
|---|---|---|
| item_id | UUID FK → items(id) ON DELETE CASCADE | PK(item_id, category_id)の一部 |
| category_id | UUID FK → categories(id) ON DELETE CASCADE | PK(item_id, category_id)の一部 |

## Rustモデル（`tag.rs`と対称構造）

- `Category { id: Uuid, name: String }`（`sqlx::FromRow`）
- `CreateCategoryRequest { name: String }`
- `validate_category_name(name: &str) -> Result<(), ApiError>` — `validate_tag_name`と同様のtrim().is_empty()判定。

## 参照

エンドポイント例は [mediavault-api/categories.md](../mediavault-api/categories.md) を参照。
