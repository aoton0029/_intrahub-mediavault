# item_streaming_links

`backend/mediavault-api/migrations/20260707000001_add_item_streaming_links.up.sql` / `backend/mediavault-api/src/models/item_streaming_link.rs`

## DBスキーマ

### streaming_platform (enum)

`netflix`, `amazon_prime`, `disney_plus`, `dmm_tv`, `apple_tv` の固定5種類。

### item_streaming_links

| カラム | 型 | NULL | 備考 |
|---|---|---|---|
| id | UUID PK | NOT NULL | `gen_random_uuid()` |
| item_id | UUID FK → items(id) ON DELETE CASCADE | NOT NULL | |
| platform | streaming_platform | NOT NULL | |
| url | VARCHAR(1000) | NOT NULL | |
| created_at | TIMESTAMP | NOT NULL | DEFAULT CURRENT_TIMESTAMP |

制約: `UNIQUE (item_id, platform)`(1アイテムにつき1プラットフォーム1件まで)
インデックス: `idx_item_streaming_links_item_id`

## Rustモデル

- `StreamingPlatform` enum(`sqlx::Type`, `#[sqlx(type_name="streaming_platform", rename_all="snake_case")]`): `Netflix, AmazonPrime, DisneyPlus, DmmTv, AppleTv`
- `ItemStreamingLink { id, item_id, platform: StreamingPlatform, url: String, created_at }`（`sqlx::FromRow`）
- `CreateItemStreamingLinkRequest { platform, url }` — 両方必須。
- `parse_create_item_streaming_link_request(request) -> Result<CreateItemStreamingLinkRequest, ApiError>` — `url`が空文字（trim後）の場合`VALIDATION_ERROR`で拒否。
- 同一`(item_id, platform)`の重複登録はDB側のUNIQUE制約違反(SQLSTATE 23505)を検知し、`DuplicateStreamingLink`（409 `DUPLICATE_STREAMING_LINK`）へ変換する。

## 参照

エンドポイント例は [mediavault-api/item-streaming-links.md](../mediavault-api/item-streaming-links.md) を参照。
