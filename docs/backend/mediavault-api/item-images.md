← [index](./index.md)

# Item Images API

アイテムに複数の画像URLを紐づけて管理するAPI。手動での追加・削除に加え、`POST /items`・`POST /items/import`
実行時に外部APIレスポンスから収集した「画像URLっぽい項目」の値がサーバー側で自動的に登録される。

## GET /items/{id}/images
指定アイテムに紐づく画像URLを作成日時昇順で一覧取得する。
- **成功レスポンス** (200): `ApiOk<ItemImage[]>`

## POST /items/{id}/images
- **リクエストボディ** (`CreateItemImageRequest`): `url` (必須)
- **成功レスポンス** (201): `ApiOk<ItemImage>`
- **エラー**:
  - 404 `ITEM_NOT_FOUND`
  - 400 `VALIDATION_ERROR`(urlが空文字)

同一アイテムに同一URLが既に登録済みの場合はエラーにせず、既存レコードを返す（`ON CONFLICT`によるupsert）。

## DELETE /items/{id}/images/{image_id}
- **成功レスポンス**: 204
- **エラー**: 404 `ITEM_NOT_FOUND`

## ItemDetail拡張

`GET /items/{id}`のレスポンス(`ItemDetail`)に`images: ItemImage[]`が含まれる。

## サムネイル設定

一覧に表示した画像のうちどれをカードのサムネイルとして使うかは、新規エンドポイントを設けず既存の
`PATCH /items/{id}`（`cover_image_url`フィールド）で更新する。

## 外部APIインポート時の自動収集

`POST /items`・`POST /items/import`の`CreateItemRequest`は`additional_image_urls: string[]`フィールドを持つ。
インポート経路では、各プロバイダ（Annict/Jikan/TMDb/楽天/Steam/NDL）のレスポンスからキー名が画像を示唆する語
（`image`/`thumbnail`/`cover`/`poster`/`screenshot`/`banner`/`artwork`/`backdrop`/`capsule`等）を含み、
値が`http(s)://`で始まる文字列であるものをすべて収集し、item作成と同一トランザクションで`item_images`へ
一括登録する（重複URLは無視）。手動作成（`POST /items`）でも`additional_image_urls`を任意で渡せる。
