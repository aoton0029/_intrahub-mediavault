← [index](./index.md)

# Item Streaming Links API

映像作品(Movie/Drama/Anime等)の配信サービスURLを登録するAPI。プラットフォームは固定5種類
(`netflix`, `amazon_prime`, `disney_plus`, `dmm_tv`, `apple_tv`)のみで、1アイテムにつき
1プラットフォーム1件までとする。

## GET /items/{id}/streaming-links
指定アイテムに紐づく配信URLを作成日時昇順で一覧取得する。
- **成功レスポンス** (200): `ApiOk<ItemStreamingLink[]>`

## POST /items/{id}/streaming-links
- **リクエストボディ** (`CreateItemStreamingLinkRequest`): `platform` (必須。`netflix`/`amazon_prime`/`disney_plus`/`dmm_tv`/`apple_tv`のいずれか), `url` (必須)
- **成功レスポンス** (201): `ApiOk<ItemStreamingLink>`
- **エラー**:
  - 404 `ITEM_NOT_FOUND`
  - 400 `VALIDATION_ERROR`(urlが空文字)
  - 409 `DUPLICATE_STREAMING_LINK`(同一アイテムに同一プラットフォームが登録済み)

## DELETE /items/{id}/streaming-links/{link_id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`

## ItemDetail拡張

`GET /items/{id}`のレスポンス(`ItemDetail`)に`streaming_links: ItemStreamingLink[]`が含まれる。
