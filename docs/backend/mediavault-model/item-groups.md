# item_groups

`backend/mediavault-api/migrations/20260623000002_add_relation_tables.up.sql` / `backend/mediavault-api/src/models/item_group.rs`

## DBスキーマ

### item_groups（シーズン/巻/章、入れ子構造可）

| カラム | 型 | NULL | 備考 |
|---|---|---|---|
| id | UUID PK | NOT NULL | `gen_random_uuid()` |
| item_id | UUID FK → items(id) ON DELETE CASCADE | NOT NULL | |
| parent_item_id | UUID FK → items(id) ON DELETE CASCADE | NULL | 階層グルーピング用。自身のitemではなく別itemを指す |
| group_type | group_type | NOT NULL | season / volume / chapter |
| group_name | VARCHAR(255) | NOT NULL | |
| number | INTEGER | NULL | |
| display_order | INTEGER | NOT NULL | DEFAULT 0 |
| created_at / updated_at | TIMESTAMP | NOT NULL | `updated_at`はトリガー`trg_item_groups_updated_at`で自動更新 |

インデックス: `idx_item_groups_item_id`, `idx_item_groups_parent_item_id`

## Rustモデル

```rust
#[sqlx(type_name = "group_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum GroupType { Season, Volume, Chapter }
```

- `ItemGroup { id, item_id, parent_item_id: Option<Uuid>, group_type, group_name, number: Option<i32>, display_order: i32, created_at, updated_at }`（`sqlx::FromRow`）
- `CreateItemGroupRequest { group_type, group_name, number: Option<i32>, display_order: i32 (#[serde(default)]), parent_item_id: Option<Uuid> }` — `display_order`省略時は0。

## 参照

`item_groups`配下のエピソードは [item-episodes.md](./item-episodes.md) を参照。エンドポイント例は [mediavault-api/item-groups.md](../mediavault-api/item-groups.md) を参照。
