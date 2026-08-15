← [index](./index.md)

# Extraction API

ファイル単位の文字抽出要求、状態・進捗、キャンセルと、worker専用の処理APIを定義する。公開APIは認証不要、内部APIは `INTERNAL_API_KEY` が必須。

## 公開レスポンス

`Extraction` は `id`, `item_file_id`, `state`, `attempts`, `max_attempts`, `progress_current`, `progress_total`, `error`, `created_at`, `updated_at` を返す。`state` は `queued | running | cancelling | succeeded | failed | cancelled`。`error` は `{ kind, message, retryable }` または `null`。lease token、期限、worker ID は公開しない。

```json
{
  "success": true,
  "data": {
    "id": "e1a2b3c4-0000-0000-0000-000000000001",
    "item_file_id": "f1a2b3c4-0000-0000-0000-000000000002",
    "state": "queued", "attempts": 0, "max_attempts": 3,
    "progress_current": 0, "progress_total": null, "error": null,
    "created_at": "2026-08-15T09:30:00", "updated_at": "2026-08-15T09:30:00"
  }
}
```

## POST /items/{id}/files/{file_id}/extraction

抽出を要求する。ボディはない。UUID、item、fileの帰属、`file_type`（`pdf` / `image`）、許可ルート内の実体を順に検証してから作成する。

- 新規作成: `201` と `Extraction`
- 同じファイルに active な抽出がある: `200` と既存の `Extraction`
- エラー: `400 VALIDATION_ERROR`, `404 ITEM_NOT_FOUND`, `404 FILE_NOT_FOUND`, `422 UNSUPPORTED_FILE_TYPE`, `422 UNPROCESSABLE_ENTITY`

冪等性は `item_file_extractions(item_file_id) WHERE state IN ('queued','running','cancelling')` の部分UNIQUE indexで保証する。並列要求も未完了行1件へ収束する。

## GET /items/{id}/files/{file_id}/extraction

対象ファイルの抽出履歴から最新1件を返す。抽出がなければ `404 EXTRACTION_NOT_FOUND`。ほかに `400 VALIDATION_ERROR`, `404 ITEM_NOT_FOUND`, `404 FILE_NOT_FOUND` を返す。

## POST /items/{id}/files/{file_id}/extraction/cancel

最新の抽出をキャンセルする。`queued` は直ちに `cancelled`、`running` は `cancelling`、`cancelling` はそのまま返す。終端状態は `409 EXTRACTION_ALREADY_FINISHED`。抽出がなければ `404 EXTRACTION_NOT_FOUND`。

## 状態遷移

```text
queued ──claim──> running ──complete──> succeeded
   │                  ├──fail(retryable)──> queued / failed
   │                  └──cancel要求──> cancelling ──worker確認──> cancelled
   └──cancel──> cancelled
```

`succeeded`, `failed`, `cancelled` は終端状態。complete/fail/cancelled は claim 時の lease token を要求し、古いworkerからの更新を拒否する。

## Worker 内部API

すべて `/api/v1/internal` 配下で、`Authorization: Bearer <INTERNAL_API_KEY>`（生のキーも可）が必要。

### POST /extractions/claim

リクエストは `{ "worker_id": "extractor-1", "lease_seconds": 300 }`。取得対象がなければ `{ "success": true, "data": null }`。取得時は `extraction_id`, `item_file_id`, `item_id`, `file_type`, `size_bytes`, `attempts`, `lease_token`, `lease_expires_at`, `file_ref: { root, relative_path }` を返す。`FOR UPDATE SKIP LOCKED` で排他取得し、lease切れも回収する。

### POST /extractions/{id}/heartbeat

リクエストは `lease_token`、任意の `progress_current`, `progress_total`, `lease_seconds`。`state`, `cancel_requested`, `lease_expires_at` を返す。`cancelling` のときだけ `cancel_requested=true`。

### POST /extractions/{id}/complete

`lease_token`, `content`, `boundaries: [{start,end,label}]`, `extraction_version`, `extracted_at`, `extractor` を一括送信する。本文・境界を検証し、`item_file_texts` のUPSERTと `succeeded` 遷移を同一トランザクションで確定する。

### POST /extractions/{id}/fail

`{ "lease_token": "...", "error": { "kind": "ocr_failed", "message": "...", "retryable": false } }` を送る。再試行可能かつ試行上限未満なら `queued`、それ以外は `failed`。

### POST /extractions/{id}/cancelled

`{ "lease_token": "..." }` を送る。`cancelling` を `cancelled` に確定する。

内部APIの共通エラーは `401 UNAUTHORIZED`, `404 EXTRACTION_NOT_FOUND`, `409 INVALID_LEASE_TOKEN`。入力不正は `400 VALIDATION_ERROR`、本文上限超過等は `422 UNPROCESSABLE_ENTITY`。

## Item Text APIとの関係

成功時に保存された `item_file_texts` は [GET /items/{id}/text](./item-text.md) からチャンク単位で取得する。Item Text APIは抽出状態ではなく保存済み結果の有無で応答し、進捗は本APIの `GET .../extraction` で確認する。
