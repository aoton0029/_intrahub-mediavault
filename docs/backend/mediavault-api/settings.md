← [index](./index.md)

# Settings API

## PUT /settings/api-keys/{provider}
外部API連携キーを登録・更新（upsert）。

- **パスパラメータ**: `provider` ∈ `tmdb`, `igdb`, `ndl`, `steam`, `open_library`, `ani_list`（`jikan` はキー不要のため対象外）
- **リクエストボディ** (`UpdateApiKeyRequest`): `api_key` (必須)
- **成功レスポンス** (200): `ApiOk<ApiCredential>`
- **エラー**: 400 `INVALID_PROVIDER`
