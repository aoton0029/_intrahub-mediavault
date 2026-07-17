← [index](./index.md)

# Item Images API

アイテムに複数の画像を紐づけて管理するAPI。手動での追加・削除に加え、`POST /items`・`POST /items/import`
実行時に外部APIレスポンスからプロバイダ別に明示抽出した画像がサーバー側で自動的に登録される。

## ItemImage

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | |
| `item_id` | UUID | |
| `url` | string | 画像URL |
| `kind` | enum | `cover` / `backdrop` / `screenshot` / `thumbnail` / `other` |
| `source` | enum | 収集元。`manual`（手動追加）/ `annict` / `jikan` / `tmdb` / `rakuten` / `steam` / `ndl` |
| `sort_order` | int | 表示順（自動収集時は抽出順、手動追加は0） |
| `created_at` | timestamp | |

## GET /items/{id}/images
指定アイテムに紐づく画像を`sort_order`昇順（同順位は作成日時昇順）で一覧取得する。
- **成功レスポンス** (200): `ApiOk<ItemImage[]>`

## POST /items/{id}/images
- **リクエストボディ** (`CreateItemImageRequest`): `url` (必須)、`kind` (任意、省略時`other`)
- **成功レスポンス** (201): `ApiOk<ItemImage>`
- **エラー**:
  - 404 `ITEM_NOT_FOUND`
  - 400 `VALIDATION_ERROR`(urlが空文字)

手動追加のため`source`は常に`manual`。同一アイテムに同一URLが既に登録済みの場合はエラーにせず、
`kind`のみ更新して既存レコードを返す（`ON CONFLICT`によるupsert）。

## DELETE /items/{id}/images/{image_id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`

## ItemDetail拡張

`GET /items/{id}`のレスポンス(`ItemDetail`)に`images: ItemImage[]`が含まれる。

## サムネイル設定

一覧に表示した画像のうちどれをカードのサムネイルとして使うかは、新規エンドポイントを設けず既存の
`PATCH /items/{id}`（`cover_image_url`フィールド）で更新する。

## 外部APIインポート時の自動収集

`POST /items`・`POST /items/import`の`CreateItemRequest`は`additional_images: {url, kind}[]`フィールドを持つ
（手動作成時は任意入力で`source=manual`固定。`source`はリクエストから指定できない）。

インポート経路では、旧実装の「キー名が画像を示唆する語を含むURLの全収集」ヒューリスティックを廃止し、
各プロバイダのレスポンス構造に基づいて以下を明示抽出する（同一画像のサイズ・フォーマット違いは保存しない）。
item作成と同一トランザクションで`item_images`へ一括登録され、重複URLは無視される。

| media_type | 抽出内容 |
| --- | --- |
| anime | Annict `images.recommended_url`・`images.facebook.og_image_url`（cover）＋ Jikan `images.jpg.large_image_url`（cover） |
| movie / drama | TMDb `poster_path`（cover）＋ `backdrop_path`（backdrop） |
| manga / novel / academic_book (楽天) | `largeImageUrl`（無ければmedium→small）1枚のみ（cover） |
| game | Steam `header_image`（cover）＋ `screenshots[].path_full`（screenshot） |
| paper (NDL) | `thumbnail_url`（cover） |
