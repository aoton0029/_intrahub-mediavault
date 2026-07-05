# staff / item_staff

`backend/mediavault-api/migrations/20260623000002_add_relation_tables.up.sql` / `backend/mediavault-api/src/models/staff.rs`

## DBスキーマ

### staff

| カラム | 型 | NULL | 備考 |
|---|---|---|---|
| id | UUID PK | NOT NULL | `gen_random_uuid()` |
| external_id | VARCHAR(100) | NULL | |
| name | VARCHAR(255) | NOT NULL | |
| image_url | VARCHAR(1000) | NULL | |
| created_at | TIMESTAMP | NOT NULL | DEFAULT CURRENT_TIMESTAMP |

インデックス: `idx_staff_external_id`

### item_staff（多対多、role付き）

| カラム | 型 | NULL | 備考 |
|---|---|---|---|
| id | UUID PK | NOT NULL | `gen_random_uuid()` |
| item_id | UUID FK → items(id) ON DELETE CASCADE | NOT NULL | |
| staff_id | UUID FK → staff(id) ON DELETE CASCADE | NOT NULL | |
| role | VARCHAR(100) | NOT NULL | |
| character_name | VARCHAR(255) | NULL | |

インデックス: `idx_item_staff_item_id`, `idx_item_staff_staff_id`

## Rustモデル

- `Staff { id, external_id: Option<String>, name: String, image_url: Option<String>, created_at }`（`sqlx::FromRow`）
- `ItemStaff { id, item_id, staff_id, role: String, character_name: Option<String> }`（`sqlx::FromRow`）
- `CreateStaffRequest { name, external_id, image_url }` — `parse_create_staff_request`で`name`空文字拒否 + 255文字超過を拒否。
- `CreateItemStaffRequest { staff_id, role, character_name }` — `parse_create_item_staff_request`で`role`空文字拒否 + `ROLE_MAX_LEN`(100文字)超過を拒否 + `character_name`の`CHARACTER_NAME_MAX_LEN`(255文字)超過を拒否。
- 定数: `ROLE_MAX_LEN: usize = 100`（DB `role VARCHAR(100)`と一致）, `CHARACTER_NAME_MAX_LEN: usize = 255`（DB `character_name VARCHAR(255)`と一致）。

## 参照

エンドポイント例は [mediavault-api/staff.md](../mediavault-api/staff.md) を参照。
