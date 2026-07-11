← [index](./index.md)

# Settings API

## PUT /settings/api-keys/{provider}
外部API連携キーを登録・更新（upsert）。

- **パスパラメータ**: `provider` ∈ `tmdb`, `igdb`, `ndl`, `steam`, `annict`, `rakuten`（`jikan` はキー不要のため対象外）
- **リクエストボディ** (`UpdateApiKeyRequest`): `api_key` (必須)

```json
{ "api_key": "valid-tmdb-key" }
```

- **成功レスポンス** (200): `ApiOk<ApiCredential>`

```json
{
  "success": true,
  "data": {
    "provider": "tmdb",
    "api_key": "valid-tmdb-key",
    "updated_at": "2026-07-11T12:00:00"
  }
}
```

- **エラー**: 400 `INVALID_PROVIDER`（許可されたprovider文字列と一致しない場合。大文字表記や`jikan`も対象）

```json
{
  "success": false,
  "error": {
    "code": "INVALID_PROVIDER",
    "message": "不正なproviderが指定されました"
  }
}
```
