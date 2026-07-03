← [index](./index.md)

# Health API

## GET /health
DBへの疎通確認込みのヘルスチェック。

- **認証**: 不要
- **成功レスポンス** (200): `{"success":true,"data":{"status":"ok"}}`
- **エラー**: 500 `INTERNAL_ERROR`（DB接続失敗時）
