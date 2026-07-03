← [index](./index.md)

# Import API

## POST /import/booklog
Booklog エクスポートCSVを取り込む。行単位で成否を集計し、失敗があっても200を返す。

- **Content-Type**: `multipart/form-data`（フィールド名 `file` または `csv`）
- **成功レスポンス** (200): `ApiOk<ImportSummary>`
  ```json
  {
    "success_count": 10,
    "failure_count": 2,
    "failures": [ { "row_number": 5, "reason": "..." } ]
  }
  ```
- **エラー**: 400 `VALIDATION_ERROR`（ファイル未指定・空）

## POST /import/steam
Steam Web API 経由でユーザーのゲームライブラリをインポートする。

- **リクエストボディ** (`SteamImportRequest`): `steam_id` (必須)
- **成功レスポンス** (200): `ApiOk<ImportSummary>`
- **エラー**: 400 `VALIDATION_ERROR`, 401 `STEAM_API_KEY_INVALID`, 502 `EXTERNAL_API_TIMEOUT`
