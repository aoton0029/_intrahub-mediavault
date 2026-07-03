← [index](./index.md)

# Item Episodes API

## POST /groups/{group_id}/episodes
- **リクエストボディ** (`CreateItemEpisodeRequest`): `episode_number` (必須), `title` / `original_title` / `air_date` / `description` (optional)
- **成功レスポンス** (201): `ApiOk<ItemEpisode>`
- **エラー**: 404 `GROUP_NOT_FOUND`, 400 `INVALID_GROUP_TYPE_FOR_EPISODES`（`volume` タイプの group には作成不可、DBトリガーでも二重チェック）, 409 `DUPLICATE_EPISODE_NUMBER`

## GET /groups/{group_id}/episodes
- **成功レスポンス** (200): `ApiOk<ItemEpisode[]>`
- **エラー**: 404 `GROUP_NOT_FOUND`
