# MediaVault Extractor API エンドポイント仕様

**作成日**: 2026-08-14
**関連設計**: [architecture.md](architecture.md)
**関連要件定義**: [requirements.md](../spec/requirements.md)

**【信頼性レベル凡例】**:
- 🔵 **青信号**: EARS要件定義書・設計文書・既存API仕様を参考にした確実な定義
- 🟡 **黄信号**: EARS要件定義書・設計文書・既存API仕様から妥当な推測による定義
- 🔴 **赤信号**: EARS要件定義書・設計文書・既存API仕様にない推測による定義

---

## 共通仕様

### ベースURL 🔵

**信頼性**: 🔵 *[index.md](../../backend/mediavault-api/index.md) §基本方針・設計ヒアリングQ3より*

| 区分 | パス | 認証 |
|---|---|---|
| 公開API | `/api/v1` | **なし**（単一ユーザー・セルフホスト） |
| 内部API | `/api/v1/internal` | `INTERNAL_API_KEY`（`api_key_auth`） |

内部APIのパスは本設計で `/internal/*` から `/api/v1/internal/*` へ移設する（[architecture.md](architecture.md) D-6）。既存の内部API 5本も同時に移る。旧パスは残さない。

### 認証（内部APIのみ）🔵

**信頼性**: 🔵 *既存 `src/middleware/api_key_auth.rs:15` の実測より*

```http
Authorization: <INTERNAL_API_KEY>
```
または
```http
Authorization: Bearer <INTERNAL_API_KEY>
```

既存ミドルウェアは両形式を受理する。未設定・不一致は `401 UNAUTHORIZED`。

### 成功レスポンス 🔵

**信頼性**: 🔵 *既存 `src/models/response.rs:14` の実測より*

```json
{ "success": true, "data": { } }
```

`ApiOk<T>` の `IntoResponse` は常に `200`。`201` を返す場合はハンドラで `(StatusCode::CREATED, Json(ApiOk::new(x)))` を明示する（既存 `src/handlers/item_files.rs:46` と同じ）。

### エラーレスポンス 🔵

**信頼性**: 🔵 *既存 `src/models/response.rs:37` の実測より*

```json
{ "success": false, "error": { "code": "ERROR_CODE", "message": "..." } }
```

`GET /items/{id}/text` の `AMBIGUOUS_FILE` のみ `candidates` を追加する拡張を持つ（[item-text.md](../../backend/mediavault-api/item-text.md) 既定）。

### 追加・削除するエラーコード 🔵

**信頼性**: 🔵 *[requirements.md](../spec/requirements.md) §追加・削除するエラーコードより*

`src/models/response.rs` の `ApiErrorCode` enum と `code_and_status()` へ以下を追加する。

| Rust variant | code | HTTP | 条件 |
|---|---|---|---|
| `ExtractionNotFound` | `EXTRACTION_NOT_FOUND` | 404 | 指定ファイルに抽出が1件も存在しない |
| `ExtractionAlreadyFinished` | `EXTRACTION_ALREADY_FINISHED` | 409 | 終端状態の抽出をキャンセルしようとした |
| `UnsupportedFileType` | `UNSUPPORTED_FILE_TYPE` | 422 | 抽出非対応の `file_type` |
| `InvalidLeaseToken` | `INVALID_LEASE_TOKEN` | 409 | lease token 不一致、または lease 失効後の complete/fail |
| `TextNotExtracted` | `TEXT_NOT_EXTRACTED` | 422 | ファイルは存在するが抽出結果がない |
| `AmbiguousFile` | `AMBIGUOUS_FILE` | 409 | `file_id` 省略時に抽出済みファイルが2件以上 |

`JOB_NOT_FOUND` / `JOB_ALREADY_FINISHED` は**追加しない**（未実装のため実体がなく、[index.md](../../backend/mediavault-api/index.md) のエラーコード表からのみ削除する）。

既存の `FILE_NOT_FOUND`（404）・`ITEM_NOT_FOUND`（404）・`VALIDATION_ERROR`（400）・`UNAUTHORIZED`（401）・`INTERNAL_ERROR`（500）はそのまま再利用する。

---

## 公開API

### POST /items/{id}/files/{file_id}/extraction 🔵

**信頼性**: 🔵 *REQ-001・REQ-004・REQ-101・REQ-102・TC-001-01〜B01より*

**関連要件**: REQ-001, REQ-004, REQ-044, REQ-101, REQ-102, REQ-410

**説明**: 対象ファイルのテキスト抽出をリクエストする。冪等。

**パスパラメータ**: `id`（item UUID）, `file_id`（item_file UUID）

**リクエストボディ**: なし。抽出対象は `file_id` から一意に決まる（REQ-402: ホストパスは受け取らない）

**処理順序**（この順序が `422` と `404` の区別を決めるため重要）:
1. `id` / `file_id` の UUID 形式検証 → 不正なら `400 VALIDATION_ERROR`
2. `item_files` から `id = file_id AND item_id = {id}` で取得 → 0件なら `404 FILE_NOT_FOUND`
3. `file_type` が `pdf` / `image` か → それ以外なら `422 UNSUPPORTED_FILE_TYPE`
4. `path` から `FileRef` を解決し、実体の存在と読み取り可能性を確認 → 不可なら `422 UNPROCESSABLE_ENTITY`
5. `INSERT INTO item_file_extractions` → `23505`（部分UNIQUE違反）なら既存の未完了行を返して `200`

**成功レスポンス（201 - 新規作成）**:
```json
{
  "success": true,
  "data": {
    "id": "e1a2b3c4-0000-0000-0000-000000000001",
    "item_file_id": "f1a2b3c4-1e3e-4c9a-9c3e-2f6b1a2a0002",
    "state": "queued",
    "attempts": 0,
    "max_attempts": 3,
    "progress_current": 0,
    "progress_total": null,
    "error": null,
    "created_at": "2026-08-14T09:30:00",
    "updated_at": "2026-08-14T09:30:00"
  }
}
```

**成功レスポンス（200 - 未完了の既存を返却）**: 同じ形。`state` は `queued` / `running` / `cancelling` のいずれか。

**エラー**: `400 VALIDATION_ERROR`, `404 ITEM_NOT_FOUND`, `404 FILE_NOT_FOUND`, `422 UNSUPPORTED_FILE_TYPE`, `422 UNPROCESSABLE_ENTITY`

**冪等性の実現** 🔵 *REQ-044・[architecture.md](architecture.md) D-1より*:
`dedup_key` は使わない。部分UNIQUE index `uq_item_file_extractions_active` に INSERT を投げ、`is_unique_violation`（既存 `db_error_utils`）で検出したら既存行を SELECT して `200` で返す。並列 POST でも DB が直列化するため、未完了行は必ず1件に収束する（TC-001-B01）。

---

### GET /items/{id}/files/{file_id}/extraction 🔵

**信頼性**: 🔵 *REQ-002・REQ-301・TC-002-01/02より*

**関連要件**: REQ-002, REQ-301

**説明**: 抽出の状態・進捗・エラーを取得する。履歴のうち**最新1件**（`created_at DESC LIMIT 1`）を返す（[architecture.md](architecture.md) D-2）。

**成功レスポンス（200）**:
```json
{
  "success": true,
  "data": {
    "id": "e1a2b3c4-0000-0000-0000-000000000001",
    "item_file_id": "f1a2b3c4-1e3e-4c9a-9c3e-2f6b1a2a0002",
    "state": "running",
    "attempts": 1,
    "max_attempts": 3,
    "progress_current": 3,
    "progress_total": 10,
    "error": null,
    "created_at": "2026-08-14T09:30:00",
    "updated_at": "2026-08-14T09:32:10"
  }
}
```

**失敗した抽出の例**:
```json
{
  "success": true,
  "data": {
    "state": "failed",
    "attempts": 3,
    "max_attempts": 3,
    "error": {
      "kind": "unsupported_format",
      "message": "拡張子とファイルシグネチャが一致しません",
      "retryable": false
    }
  }
}
```

**エラー**: `400 VALIDATION_ERROR`, `404 ITEM_NOT_FOUND`, `404 FILE_NOT_FOUND`, `404 EXTRACTION_NOT_FOUND`（抽出が一度も作られていない）

`lease_token` / `lease_expires_at` / `claimed_by` は**公開レスポンスに含めない** 🟡 *内部の排他制御情報であり外部へ出す必要がないため（要件に明記はない）*

---

### POST /items/{id}/files/{file_id}/extraction/cancel 🔵

**信頼性**: 🔵 *REQ-003・REQ-201・REQ-202・REQ-205・TC-003-01〜E03より*

**関連要件**: REQ-003, REQ-201, REQ-202, REQ-205, EDGE-010

**説明**: 最新の抽出にキャンセルを要求する。

**状態別の挙動**:

| 現在の state | 遷移後 | HTTP | 備考 |
|---|---|---|---|
| `queued` | `cancelled` | 200 | worker がまだ触っていないため即座に終端へ（REQ-201） |
| `running` | `cancelling` | 200 | worker の確認を待つ。即 `cancelled` にはしない（REQ-202） |
| `cancelling` | `cancelling` | 200 | 冪等。現在の状態をそのまま返す（EDGE-010） |
| `succeeded` / `failed` / `cancelled` | 変化なし | **409** | `EXTRACTION_ALREADY_FINISHED`（REQ-205） |

**成功レスポンス（200）**: `GET .../extraction` と同じ形

**エラー**: `400 VALIDATION_ERROR`, `404 ITEM_NOT_FOUND`, `404 FILE_NOT_FOUND`, `404 EXTRACTION_NOT_FOUND`, `409 EXTRACTION_ALREADY_FINISHED`

---

### GET /items/{id}/text 🔵

**信頼性**: 🔵 *REQ-005〜008・REQ-115〜117・[item-text.md](../../backend/mediavault-api/item-text.md) より*

**関連要件**: REQ-005, REQ-006, REQ-007, REQ-008, REQ-115, REQ-116, REQ-117, REQ-413, REQ-414

**説明**: 抽出済み全文の指定チャンクを取得する。既存の [item-text.md](../../backend/mediavault-api/item-text.md) 設計を実装する。本設計での変更点は `label` の扱いのみ（下記）。

**クエリパラメータ**:

| 名前 | 型 | 必須 | 既定 | 説明 |
|---|---|---|---|---|
| `file_id` | uuid | | | 省略時は主ファイルを解決（下記） |
| `chunk_index` | number | | `0` | 0起点 |
| `chunk_size` | number | | `4000` | 最大 `20000` |

**成功レスポンス（200）**:
```json
{
  "success": true,
  "data": {
    "item_id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001",
    "file_id": "f1a2b3c4-1e3e-4c9a-9c3e-2f6b1a2a0002",
    "extracted_at": "2026-08-14T09:45:00",
    "extraction_version": "pdf-v1",
    "chunk": {
      "index": 0,
      "size": 4000,
      "total_chunks": 12,
      "label": "p.1-3",
      "text": "本文の先頭4000文字..."
    }
  }
}
```

**`label` の解決** 🔵 *設計ヒアリングQ2・[architecture.md](architecture.md) D-5より*:

item-text.md の初期案は「MVPでは `label` を常に `null`」だったが、本設計では `boundaries` を MVP から保存するため実際のラベルを返す。`chunk_size=4000` は通常数ページ分に相当するため、**範囲表記**を採用する。

```text
チャンク文字範囲 [start, end) と交差する全境界を取る
  → 1件なら           label = その境界の label        （例 "p.9"）
  → 2件以上なら       label = "{先頭}-{末尾の数値部}"  （例 "p.1-3"）
  → 0件（境界情報なし）label = null
```

`index` は形式非依存の0起点連番のままで、ページ情報は `label` にのみ現れる（REQ-413）。

**主ファイルの解決**（`file_id` 省略時）🔵 *item-text.md §主ファイルの解決より*:

| 抽出済みファイル数 | 挙動 |
|---|---|
| 1件 | そのファイルを対象 |
| 0件（ファイル自体が無い） | `404 FILE_NOT_FOUND` |
| 0件（ファイルはあるが未抽出） | `422 TEXT_NOT_EXTRACTED` |
| 2件以上 | `409 AMBIGUOUS_FILE`（候補一覧付き）。**推測で選ばない** |

**エラー**: `400 VALIDATION_ERROR`, `404 ITEM_NOT_FOUND`, `404 FILE_NOT_FOUND`, `409 AMBIGUOUS_FILE`, `422 TEXT_NOT_EXTRACTED`

**抽出の状態を参照しない** 🔵 *REQ-117・TC-005-E04/E05より*: 本APIは `item_file_extractions` を見ない。`item_file_texts` に行があるかどうかだけで判定する。抽出が `running` でも `failed` でも、結果がなければ `422 TEXT_NOT_EXTRACTED` を返す。

---

## 内部API（worker 専用）

すべて `/api/v1/internal` 配下。`api_key_auth` ミドルウェアが適用される。公開ルーターには登録しない（REQ-406）。

### POST /internal/extractions/claim 🔵

**信頼性**: 🔵 *REQ-020〜022・EDGE-001・TC-020-01〜B02より*

**関連要件**: REQ-020, REQ-021, REQ-022, REQ-118, EDGE-001

**説明**: 実行可能な抽出を1件だけ排他的に取得し、lease を設定する。

**リクエスト**:
```json
{ "worker_id": "extractor-1", "lease_seconds": 300 }
```

**成功レスポンス（200 - 取得あり）**:
```json
{
  "success": true,
  "data": {
    "extraction_id": "e1a2b3c4-0000-0000-0000-000000000001",
    "item_file_id": "f1a2b3c4-1e3e-4c9a-9c3e-2f6b1a2a0002",
    "item_id": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001",
    "file_type": "pdf",
    "size_bytes": 12345678,
    "attempts": 1,
    "lease_token": "9f8e7d6c-0000-0000-0000-00000000000a",
    "lease_expires_at": "2026-08-14T09:35:00",
    "file_ref": {
      "root": "storage",
      "relative_path": "b6b6f9a0-1e3e-4c9a-9c3e-2f6b1a2a0001/3c2b1a09.pdf"
    }
  }
}
```

**成功レスポンス（200 - 取得なし）** 🟡 *[tech-stack.md](../tech-stack.md) 実行ループ `if job is None: sleep(...)` から妥当な推測*:
```json
{ "success": true, "data": null }
```
`204` ではなく `data: null` とする理由は、`ApiOk<Option<T>>` で既存のレスポンス型をそのまま使え、worker 側も同じデシリアライズ経路で扱えるためである。

**`file_ref` の形式** 🔵 *設計ヒアリングQ4（要件定義フェーズ）・[architecture.md](architecture.md) D-3より*:

| `root` | 解決元 | worker 側マウント |
|---|---|---|
| `"storage"` | `item_files.path` が相対パス（アップロード経路） | `EXTRACTOR_STORAGE_ROOT` |
| `"library"` | `item_files.path` が絶対パス（リンク経路） | `EXTRACTOR_LIBRARY_ROOT` |

api は worker のマウントパスを知らない。`relative_path` に `..` を含む値は api 側でも拒否する（REQ-402）。

**排他制御** 🔵 *EDGE-001・TC-020-B01より*: `FOR UPDATE SKIP LOCKED LIMIT 1`。2台の worker が同時に呼んでもブロックせず別々の行を取る。`state='queued'` に加え、lease 切れの `running` / `cancelling` も対象に含めることで worker 異常終了からの復旧を兼ねる（REQ-118・TC-020-B02）。

**エラー**: `401 UNAUTHORIZED`

---

### POST /internal/extractions/{id}/heartbeat 🔵

**信頼性**: 🔵 *REQ-023・REQ-066・TC-023-01〜03より*

**関連要件**: REQ-023, REQ-066, REQ-202

**説明**: lease 延長・進捗更新・キャンセル要求の取得を1リクエストで行う。

**リクエスト**:
```json
{
  "lease_token": "9f8e7d6c-0000-0000-0000-00000000000a",
  "progress_current": 5,
  "progress_total": 10,
  "lease_seconds": 300
}
```

**成功レスポンス（200）**:
```json
{
  "success": true,
  "data": {
    "state": "cancelling",
    "cancel_requested": true,
    "lease_expires_at": "2026-08-14T09:40:00"
  }
}
```

`cancel_requested` が `true` になるのは `state = "cancelling"` のときのみ。worker はこれを検知したらページ境界で処理を中断し、`cancelled` エンドポイントを呼ぶ（REQ-207）。

**エラー**: `401 UNAUTHORIZED`, `404 EXTRACTION_NOT_FOUND`, `409 INVALID_LEASE_TOKEN`

---

### POST /internal/extractions/{id}/complete 🔵

**信頼性**: 🔵 *REQ-024・REQ-025・REQ-065・TC-024-01〜B01より*

**関連要件**: REQ-024, REQ-025, REQ-065, REQ-103, REQ-204, REQ-407, REQ-408

**説明**: 抽出本文・境界情報・メタデータを保存し、抽出を `succeeded` へ遷移させる。**保存と遷移は同一トランザクション**（[architecture.md](architecture.md) D-4）。

**リクエスト**:
```json
{
  "lease_token": "9f8e7d6c-0000-0000-0000-00000000000a",
  "content": "正規化済みの全文...",
  "boundaries": [
    { "start": 0, "end": 1200, "label": "p.1" },
    { "start": 1200, "end": 2900, "label": "p.2" }
  ],
  "extraction_version": "pdf-v1",
  "extracted_at": "2026-08-14T09:45:00",
  "extractor": {
    "method": "mixed",
    "embedded_text_pages": 7,
    "ocr_pages": 3,
    "ocr": { "engine": "yomitoku", "device": "cpu", "model": "yomitoku-lite-v1" }
  }
}
```

**検証順序**:
1. `lease_token` 照合 + `state = 'running'` 確認（`FOR UPDATE` で行ロック）→ 不一致・`cancelling`・終端なら `409 INVALID_LEASE_TOKEN`
2. `content` のサイズ上限 → 超過なら `422 UNPROCESSABLE_ENTITY`（EDGE-009・REQ-408）
3. `boundaries` の整合性（`start <= end`、`end <= char_length(content)`）→ 不正なら `400 VALIDATION_ERROR`（EDGE-107）
4. `item_file_texts` を UPSERT（再抽出時は置き換え。REQ-103）
5. `item_file_extractions` を `succeeded` へ

**成功レスポンス（200）**: `GET .../extraction` と同じ形（`state: "succeeded"`）

**エラー**: `400 VALIDATION_ERROR`, `401 UNAUTHORIZED`, `404 EXTRACTION_NOT_FOUND`, `409 INVALID_LEASE_TOKEN`, `422 UNPROCESSABLE_ENTITY`

**キャンセル済みへの complete が拒否される仕組み** 🔵 *REQ-204・TC-003-E02より*: ステップ1の `WHERE state = 'running'` により、`cancelling` の抽出は0行となり `409` になる。「キャンセル後に成功結果を確定してはならない」がクエリ条件そのもので担保される。

---

### POST /internal/extractions/{id}/fail 🔵

**信頼性**: 🔵 *REQ-026・REQ-110〜112・TC-026-01〜03より*

**関連要件**: REQ-026, REQ-110, REQ-111, REQ-112, REQ-206

**説明**: 構造化エラーを保存し、再試行するか終了するかを判定する。

**リクエスト**:
```json
{
  "lease_token": "9f8e7d6c-0000-0000-0000-00000000000a",
  "error": {
    "kind": "ocr_failed",
    "message": "全ページのOCRに失敗しました",
    "retryable": false
  }
}
```

**遷移判定** 🟡 *REQ-111・REQ-112・PRD §8.5 状態遷移図から妥当な推測*:

| `retryable` | `attempts` vs `max_attempts` | 遷移後 |
|---|---|---|
| `false` | — | `failed`（終端）。無限再試行しない（REQ-110） |
| `true` | `attempts < max_attempts` | `queued`（lease をクリアして再投入） |
| `true` | `attempts >= max_attempts` | `failed`（終端） |

`error.kind` の想定値 🟡: `unsupported_format` / `corrupt_file` / `file_not_found` / `size_limit_exceeded` / `ocr_failed` / `api_unreachable` / `lease_expired` / `internal`

**既存の成功結果は削除しない** 🔵 *REQ-206・TC-026-04より*: fail 処理は `item_file_texts` に一切触れない。再抽出が失敗しても `GET /items/{id}/text` は従来の結果を返し続ける。

**成功レスポンス（200）**: `GET .../extraction` と同じ形

**エラー**: `401 UNAUTHORIZED`, `404 EXTRACTION_NOT_FOUND`, `409 INVALID_LEASE_TOKEN`

---

### POST /internal/extractions/{id}/cancelled 🔵

**信頼性**: 🔵 *REQ-027・REQ-202・TC-003-02より*

**関連要件**: REQ-027, REQ-202, REQ-207

**説明**: worker がキャンセル要求を受けて処理を停止したことを確認し、`cancelled`（終端）へ遷移させる。

**リクエスト**:
```json
{ "lease_token": "9f8e7d6c-0000-0000-0000-00000000000a" }
```

**成功レスポンス（200）**: `GET .../extraction` と同じ形（`state: "cancelled"`）

**エラー**: `401 UNAUTHORIZED`, `404 EXTRACTION_NOT_FOUND`, `409 INVALID_LEASE_TOKEN`

抽出結果は保存されない。部分結果が成功として確定することはない（REQ-204・REQ-207）。

---

## 既存内部API 5本の移設 🔵

**信頼性**: 🔵 *REQ-029・設計ヒアリングQ3・既存 `src/routes/internal.rs:28` の実測より*

パス以外の仕様変更はない。リクエスト/レスポンスは [internal-api.md](../../backend/mediavault-api/internal-api.md) の既存記述どおり。

| 旧パス | 新パス |
|---|---|
| `POST /internal/items` | `POST /api/v1/internal/items` |
| `GET /internal/items/search` | `GET /api/v1/internal/items/search` |
| `PATCH /internal/items/{id}` | `PATCH /api/v1/internal/items/{id}` |
| `POST /internal/items/{id}/groups` | `POST /api/v1/internal/items/{id}/groups` |
| `POST /internal/groups/{group_id}/episodes` | `POST /api/v1/internal/groups/{group_id}/episodes` |
| `POST /internal/items/{id}/files` | `POST /api/v1/internal/items/{id}/files` |
| ~~`POST /internal/jobs`~~ | **削除**（未実装のため実体なし） |

**実装上の変更点** 🔵: `src/routes/internal.rs` のパス文字列は変更しない（`/internal/items` のまま）。`src/main.rs` で内部ルーターを公開ルーターへ `merge` してから `/api/v1` 配下へ `nest` することで、結果的に `/api/v1/internal/*` になる。`merge` は Router ごとのレイヤーを保持するため、`api_key_auth` は内部ルートにのみ適用され続ける。

---

## mediavault-mcp ツール 🔵

**信頼性**: 🔵 *REQ-080〜083・ヒアリングQ7（要件定義フェーズ）より*

| 旧ツール | 新ツール | 呼び出す api |
|---|---|---|
| `enqueue_job` | `request_extraction(item_id, file_id)` | `POST /api/v1/items/{id}/files/{fid}/extraction` |
| `get_job` / `list_jobs` | `get_extraction_status(item_id, file_id)` | `GET /api/v1/items/{id}/files/{fid}/extraction` |
| `cancel_job` | `cancel_extraction(item_id, file_id)` | `POST /api/v1/items/{id}/files/{fid}/extraction/cancel` |

`get_job` と `list_jobs` を1本に統合した理由は、抽出対象が常に `(item_id, file_id)` で一意に決まり、ID を別途扱う必要がなくなったためである。

**重要**: 新ツールはすべて**公開API**を呼ぶ。旧 `enqueue_job` は `POST /internal/jobs`（内部API）を呼ぶ設計だったが、抽出リクエストは公開APIで提供するため、mcp が `INTERNAL_API_KEY` を使う必要はなくなる。

`get_item_text` が `not_extracted` を返す際は、`request_extraction` による解決を促すメッセージを含める（REQ-083・NFR-503）。

---

## エンドポイント一覧（index.md への反映内容）🔵

**信頼性**: 🔵 *REQ-090・[note.md](../spec/note.md) §4より*

### 追加する行

| Method | Path | 説明 | 詳細 |
|--------|------|------|------|
| POST | /items/{id}/files/{file_id}/extraction | テキスト抽出リクエスト | extraction.md |
| GET | /items/{id}/files/{file_id}/extraction | 抽出の状態・進捗取得 | extraction.md |
| POST | /items/{id}/files/{file_id}/extraction/cancel | 抽出キャンセル | extraction.md |

`GET /items/{id}/text` は既存行から **未実装** 表記を外す。

### 削除する行

| Method | Path |
|---|---|
| GET | /jobs |
| GET | /jobs/{id} |
| POST | /jobs/{id}/cancel |
| POST | /internal/jobs（internal-api.md 側） |

目次からも `jobs.md` の行を除去し、`extraction.md` を追加する。`jobs.md` は作成しない（REQ-091）。

---

## レート制限 🔵

**信頼性**: 🔵 *既存 API に実装がないことの実測より*

導入しない。単一ユーザー・セルフホストであり、既存 API にもレート制限機構は存在しない。抽出の負荷制御は worker 側の `EXTRACTOR_MAX_CONCURRENCY`（既定1）で行う。

## CORS 🔵

**信頼性**: 🔵 *既存 `src/main.rs:56` の実測より*

`CORS_ALLOWED_ORIGIN` 環境変数（既定 `http://localhost`）。抽出APIも公開APIの一部として同じ CORS 設定が適用される。内部APIは同一 Router に merge されるため CORS レイヤーの対象になるが、ブラウザから呼ぶことはないため実害はない 🟡。

---

## 関連文書

- **アーキテクチャ**: [architecture.md](architecture.md)
- **データフロー**: [dataflow.md](dataflow.md)
- **DBスキーマ**: [database-schema.sql](database-schema.sql)
- **型定義（api）**: [interfaces.rs](interfaces.rs)
- **型定義（worker）**: [interfaces.py](interfaces.py)
- **要件定義**: [requirements.md](../spec/requirements.md)
- **受け入れ基準**: [acceptance-criteria.md](../spec/acceptance-criteria.md)

## 信頼性レベルサマリー

エンドポイント・仕様項目 24件の内訳。

| レベル | 件数 | 割合 |
|---|---|---|
| 🔵 青信号 | 20件 | 83% |
| 🟡 黄信号 | 4件 | 17% |
| 🔴 赤信号 | 0件 | 0% |

**品質評価**: ✅ 高品質

🟡 の内訳: claim の「取得なし」レスポンス形式、`lease_token` を公開レスポンスから除外する判断、fail の遷移判定表、`error.kind` の想定値、内部APIへのCORS適用。いずれも要件に反しない範囲の具体化。
