# item_relations

`backend/mediavault-api/migrations/20260623000002_add_relation_tables.up.sql` / `backend/mediavault-api/src/models/item_relation.rs`

## DBスキーマ

### item_relations

| カラム | 型 | NULL | 備考 |
|---|---|---|---|
| id | UUID PK | NOT NULL | `gen_random_uuid()` |
| item_id | UUID FK → items(id) ON DELETE CASCADE | NOT NULL | |
| related_item_id | UUID FK → items(id) ON DELETE CASCADE | NOT NULL | |
| relation_type | relation_type | NOT NULL | reference / dlc |
| created_at | TIMESTAMP | NOT NULL | DEFAULT CURRENT_TIMESTAMP |

制約: `chk_item_relations_not_self` CHECK `item_id <> related_item_id`、`uq_item_relations` UNIQUE(item_id, related_item_id, relation_type)

インデックス: `idx_item_relations_item_id`, `idx_item_relations_related_item_id`

`items`への自己参照（M:N）: `item_id`/`related_item_id`双方が`items(id)`を参照する。

## Rustモデル

```rust
#[sqlx(type_name = "relation_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum RelationType { Reference, Dlc }
```

- `ItemRelation { id: Uuid, item_id: Uuid, related_item_id: Uuid, relation_type: RelationType, created_at: NaiveDateTime }`（`sqlx::FromRow`）
- `CreateItemRelationRequest { item_id: Uuid, related_item_id: Uuid, relation_type: RelationType }`
- `validate_not_self_reference(request) -> Result<(), ApiError>` — `item_id == related_item_id`を`VALIDATION_ERROR`で拒否。DB側`chk_item_relations_not_self`制約に対するアプリ層の第一防衛線。

## 参照

エンドポイント例は [mediavault-api/item-relations.md](../mediavault-api/item-relations.md) を参照。
