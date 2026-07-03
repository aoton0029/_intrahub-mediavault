← [index](./index.md)

# Item Files API

## POST /items/{id}/files
ファイルパス情報のみ登録（実体アップロードなし）。
- **リクエストボディ** (`CreateItemFileRequest`): `path` (必須), `label` (optional), `file_type` (必須)
- **成功レスポンス** (201): `ApiOk<ItemFile>`
- **エラー**: 404 `ITEM_NOT_FOUND`, 400 `VALIDATION_ERROR`

## POST /items/{id}/files/upload
実ファイルをアップロードして保存。ボディサイズ上限は本エンドポイントのみ100MBに拡張（`DefaultBodyLimit::max`）。
- **Content-Type**: `multipart/form-data`（`file`, `file_type`, `label` optional）
- **成功レスポンス** (201): `ApiOk<ItemFile>`
- **エラー**: 404 `ITEM_NOT_FOUND`, 400 `VALIDATION_ERROR`, 500 `FILE_STORAGE_WRITE_FAILED`

## PATCH /items/{id}/files/{file_id}/calibre-link
PDFファイルとCalibre書籍IDを紐付ける。
- **リクエストボディ** (`UpdateCalibreLinkRequest`): `calibre_book_id` (必須)
- **成功レスポンス** (200): `ApiOk<ItemFile>`
- **エラー**: 404 `FILE_NOT_FOUND`, 400 `VALIDATION_ERROR`（対象が pdf 以外の file_type、または id 不正）
