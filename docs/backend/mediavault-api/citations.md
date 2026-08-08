← [index](./index.md)

# Citations API

作品・論文からの引用を記録する。映像作品は再生秒数、書籍・論文はページ番号や電子書籍の位置No.など、`locator_type`に応じた付加情報を保持する。

## GET /items/{id}/citations
指定アイテムに紐づく引用を作成日時昇順で一覧取得する。
- **成功レスポンス** (200): `ApiOk<Citation[]>`
- **エラー**: 404 `ITEM_NOT_FOUND`

## POST /items/{id}/citations
- **リクエストボディ** (`CreateCitationRequest`): `quote_text` (必須), `note`, `locator_type` (必須。page/timestamp/location/chapter/none), `page_number`, `timestamp_seconds`, `location_number`, `chapter`
  - `locator_type`に対応する値（例: `locator_type=page`なら`page_number`）を指定することを推奨するが、必須バリデーションはしない（未指定はnullのまま保存）
- **成功レスポンス** (201): `ApiOk<Citation>`
- **エラー**: 404 `ITEM_NOT_FOUND`, 400 `VALIDATION_ERROR`

```json
{
  "success": true,
  "data": {
    "id": "c1b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001",
    "item_id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001",
    "quote_text": "人は見たいものしか見ようとしない。",
    "note": "第3章の議論のまとめとして引用",
    "locator_type": "page",
    "page_number": 128,
    "timestamp_seconds": null,
    "location_number": null,
    "chapter": null,
    "created_at": "2026-07-01T12:00:00",
    "updated_at": "2026-07-01T12:00:00"
  }
}
```

## PATCH /citations/{citation_id}
- **リクエストボディ** (`UpdateCitationRequest`): `quote_text`, `note`, `locator_type`, `page_number`, `timestamp_seconds`, `location_number`, `chapter`（いずれも任意、指定したフィールドのみ更新）
- **成功レスポンス** (200): `ApiOk<Citation>`
- **エラー**: 404 `CITATION_NOT_FOUND`, 400 `VALIDATION_ERROR`

## DELETE /citations/{citation_id}
- **成功レスポンス**: 204
- **エラー**: 404 `CITATION_NOT_FOUND`
