← [index](./index.md)

# Jobs API

`MediaVault-worker` へ委譲する非同期ジョブの登録・進捗確認・キャンセル。ジョブキュー全体の設計（`jobs` テーブル定義・状態遷移・リトライ方針・worker 側プロトコル）は [../../basic-design/05_job-queue.md](../../basic-design/05_job-queue.md) を参照。

**未実装。** 本ドキュメントは設計仕様であり、`jobs` テーブル・ハンドラともにまだ存在しない。

## 認証の方針

| Path | 認証 |
|---|---|
| `POST /internal/jobs` | 内部API（`INTERNAL_API_KEY`） |
| `GET /api/v1/jobs/{id}` / `GET /api/v1/jobs` / `POST /api/v1/jobs/{id}/cancel` | 公開API（無認証） |

登録（enqueue）のみ内部APIに置く。公開APIは無認証のため、重い処理を任意のクライアントから無制限に積めるのを避ける。パイプラインジョブ（`extract_text`/`index`/`resolve_links`）は api が業務処理と同一トランザクション内で直接 INSERT するため、公開の enqueue エンドポイントは不要。

---

## データ型

### `Job`

| フィールド | 型 | 説明 |
|---|---|---|
| `id` | uuid | ジョブID |
| `job_type` | `job_type` | `extract_text` / `index` / `resolve_links` |
| `state` | `job_state` | `queued` / `running` / `succeeded` / `failed` / `cancelling` / `cancelled` |
| `payload` | object | job_type ごとのパラメータ（下記） |
| `result` | object \| null | 成功時の結果。終端前は `null` |
| `error` | string \| null | 失敗理由。成功時は `null` |
| `progress_current` | number | 処理済み件数 |
| `progress_total` | number \| null | 全体件数。事前に確定しないジョブでは `null` |
| `attempts` | number | 実行試行回数 |
| `max_attempts` | number | 打ち切りまでの試行上限（既定 3） |
| `target_item_id` | uuid \| null | 対象 item |
| `created_at` / `updated_at` | timestamp | |

`state` が `succeeded` / `failed` / `cancelled` のいずれかであれば終端。クライアントは終端到達をもってポーリングを停止する。

### `payload` スキーマ（job_type 別）

| job_type | payload |
|---|---|
| `extract_text` | `{ "item_file_id": uuid, "path": string }` |
| `index` | `{ "item_id": uuid }` |
| `resolve_links` | `{ "item_id": uuid, "hints": { "title": string, "media_type": media_type } }` |

ナレッジ生成のジョブ種別は持たない（[basic-design/04_jobs-and-agent-integration.md](../../basic-design/04_jobs-and-agent-integration.md#knowledgehubとの責務分界)）。

---

## POST /internal/jobs

ジョブを登録する。`MediaVault-mcp` の `enqueue_job` ツールの対応API。

- **リクエストボディ** (`CreateJobRequest`)

  | フィールド | 必須 | 説明 |
  |---|---|---|
  | `job_type` | ○ | 上記 enum のいずれか |
  | `payload` | ○ | `job_type` に対応する形。形が合わなければ 400 |
  | `target_item_id` | | 対象 item。`payload` に `item_id` を含む場合は省略時にそこから補完する |
  | `dedup_key` | | 省略時は `<job_type>:<対象の主キー>` を自動生成。`null` を明示すると重複抑止なし |
  | `max_attempts` | | 既定 3 |

  ```json
  {
    "job_type": "resolve_links",
    "payload": {
      "item_id": "b2b5c1a0-0000-0000-0000-000000000000",
      "hints": { "title": "とある作品", "media_type": "anime" }
    }
  }
  ```

- **成功レスポンス** (201): `ApiOk<Job>`
- **重複時** (200): `ApiOk<Job>` — 同一 `dedup_key` の未終了ジョブが既に存在する場合、新規作成せず**既存ジョブをそのまま返す**（冪等）。`201` か `200` かで新規/既存を判別できる
- **エラー**: 400 `VALIDATION_ERROR`（`job_type` 不正、`payload` の形が `job_type` と不一致）, 401 `UNAUTHORIZED`, 404 `ITEM_NOT_FOUND`（`target_item_id` が存在しない）

## GET /api/v1/jobs/{id}

ジョブの状態・進捗・結果を取得する。フロントエンドのポーリング先。

- **成功レスポンス** (200): `ApiOk<Job>`

  ```json
  {
    "success": true,
    "data": {
      "id": "...",
      "job_type": "extract_text",
      "state": "running",
      "payload": { "item_file_id": "...", "path": "documents/foo.pdf" },
      "result": null,
      "error": null,
      "progress_current": 12,
      "progress_total": 48,
      "attempts": 1,
      "max_attempts": 3,
      "target_item_id": "...",
      "created_at": "2026-07-31T10:00:00",
      "updated_at": "2026-07-31T10:00:07"
    }
  }
  ```

- **エラー**: 400 `VALIDATION_ERROR`（UUID形式不正）, 404 `JOB_NOT_FOUND`

## GET /api/v1/jobs

ジョブ一覧を取得する。item 詳細画面で「この item に対して進行中の処理」を表示する用途。

- **クエリパラメータ**

  | 名前 | 説明 |
  |---|---|
  | `target_item_id` | item で絞り込む |
  | `state` | 状態で絞り込む。カンマ区切りで複数可（例 `queued,running,cancelling`） |
  | `job_type` | 種別で絞り込む |
  | `limit` / `after_created_at` / `after_id` | keyset ページネーション（共通仕様、[index.md](./index.md)） |

- **成功レスポンス** (200): `PaginatedOk<Job[]>`（`created_at` 降順）
- **エラー**: 400 `VALIDATION_ERROR`

## POST /api/v1/jobs/{id}/cancel

ジョブのキャンセルを要求する。既存の `POST /import/booklog/jobs/{job_id}/cancel` と同じ二段構えの挙動。

- `state='queued'` → その場で `cancelled` に遷移し、`cancelled` の `Job` を返す
- `state='running'` → `cancelling` に遷移。worker が次のチェックポイントで観測して `cancelled` に落とす
- 終端状態（`succeeded`/`failed`/`cancelled`）→ 409

- **成功レスポンス** (200): `ApiOk<Job>`（更新後の状態）
- **エラー**: 404 `JOB_NOT_FOUND`, 409 `JOB_ALREADY_FINISHED`

---

## 追加エラーコード

[index.md](./index.md) のエラーコード一覧に以下を追加する。

| コード | HTTPステータス | 説明 |
|---|---|---|
| JOB_NOT_FOUND | 404 | 指定した job が存在しない |
| JOB_ALREADY_FINISHED | 409 | 終端状態のジョブをキャンセルしようとした |
