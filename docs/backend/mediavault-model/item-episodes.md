# item_episodes

`backend/mediavault-api/migrations/20260623000002_add_relation_tables.up.sql` / `backend/mediavault-api/src/models/item_episode.rs`

## DBスキーマ

### item_episodes（season/chapter配下のみ使用、volumeには追加不可）

| カラム | 型 | NULL | 備考 |
|---|---|---|---|
| id | UUID PK | NOT NULL | `gen_random_uuid()` |
| group_id | UUID FK → item_groups(id) ON DELETE CASCADE | NOT NULL | |
| episode_number | INTEGER | NOT NULL | UNIQUE(group_id, episode_number)（`uq_item_episodes`） |
| title | VARCHAR(500) | NULL | |
| original_title | VARCHAR(500) | NULL | |
| air_date | DATE | NULL | |
| description | TEXT | NULL | |
| created_at / updated_at | TIMESTAMP | NOT NULL | `updated_at`はトリガー`trg_item_episodes_updated_at`で自動更新 |

インデックス: `idx_item_episodes_group_id`

### トリガー: `trg_check_episode_group_type`

`check_episode_group_type()`関数がINSERT/UPDATE前に発火し、対象`group_id`の`item_groups.group_type`が`volume`の場合は例外を投げてエピソード追加を拒否する（`INVALID_GROUP_TYPE_FOR_EPISODES`エラーの元になるDB層の防衛線。アプリケーション層のハンドラでも同等の検証を行い二重に保証する）。

## Rustモデル

- `ItemEpisode { id, group_id, episode_number: i32, title: Option<String>, original_title: Option<String>, air_date: Option<NaiveDate>, description: Option<String>, created_at, updated_at }`（`sqlx::FromRow`）
- `CreateItemEpisodeRequest { episode_number: i32, title, original_title, air_date, description }` — `episode_number`のみ必須。

## 参照

エンドポイント例は [mediavault-api/item-episodes.md](../mediavault-api/item-episodes.md) を参照。
