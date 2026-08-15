← [index](./index.md)

# 内部API（`/api/v1/internal/*`）

すべてのルートに `api_key_auth` ミドルウェアが適用される。`Authorization` ヘッダに `INTERNAL_API_KEY` の値（生値または `Bearer <key>`）が必要。未設定・不一致は `401 UNAUTHORIZED`。旧 `/internal/*` パスは提供しない。

| Method | Path | 説明 |
|---|---|---|
| POST | /api/v1/internal/items | アイテム新規作成（公開APIと同一ハンドラ） |
| GET | /api/v1/internal/items/search | アイテム検索 |
| PATCH | /api/v1/internal/items/{id} | アイテム更新 |
| POST | /api/v1/internal/items/{id}/groups | グループの upsert |
| POST | /api/v1/internal/groups/{group_id}/episodes | エピソードの upsert |
| POST | /api/v1/internal/items/{id}/files | ファイル情報登録 |
| POST | /api/v1/internal/extractions/claim | 実行可能な抽出を排他的に取得 |
| POST | /api/v1/internal/extractions/{id}/heartbeat | lease延長・進捗更新・キャンセル確認 |
| POST | /api/v1/internal/extractions/{id}/complete | 本文を保存して成功を確定 |
| POST | /api/v1/internal/extractions/{id}/fail | 構造化エラーを報告 |
| POST | /api/v1/internal/extractions/{id}/cancelled | キャンセル完了を報告 |

既存6ルートのリクエスト・レスポンスは対応する公開APIと同じ。groups と episodes は一意キーに一致する行を更新し、なければ作成する。新規作成は `201`、更新は `200`。

worker用5ルートのリクエスト・レスポンス、lease、状態遷移の詳細は [extraction.md](./extraction.md#worker-内部api) を参照。
