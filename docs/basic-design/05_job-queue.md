# MediaVault 基本設計 — ジョブキュー（backend ⇄ worker 連携）

← [00_overview.md](00_overview.md) / [04_jobs-and-agent-integration.md](04_jobs-and-agent-integration.md)

本ページは「`MediaVault-api` が `MediaVault-worker` へ**何を・どういう形で**投げ、worker が**どう受け取って結果を書き戻し**、それを**クライアント/フロントへどう反映するか**」を実装着手可能な粒度で定める。ジョブ種別の一覧と責務分界は [04_jobs-and-agent-integration.md](04_jobs-and-agent-integration.md)、worker 側の全体像は [../backend/mediavault-worker/PRD.md](../backend/mediavault-worker/PRD.md) を参照。

## 現状（本設計の適用前）

- `jobs` テーブルは未定義。マイグレーションは `20260623000001_init_schema.up.sql` の 1 本のみ。
- `POST /api/v1/jobs` 系エンドポイントは未実装。worker クレート・`Dockerfile.worker` も不在（`docker-compose.yml` に「未実装」コメントあり）。
- 唯一の非同期処理はブクログCSVインポートで、`tokio::spawn` + プロセス内 `HashMap`（`services/import_job_store.rs`）。**api プロセス再起動で消失する**。
- 進捗通知は `SettingsPage.tsx` の手書き sleep ループ 1 箇所のみ。SSE/WebSocket は無い。

本設計はこのギャップを埋めるためのもので、既存インポートジョブの状態機械（`Running`/`Cancelling`/`Cancelled`/`Completed`、`models/import.rs`）と語彙を意図的に揃えている。

---

## 全体像

```
[api] ── 業務処理と同一トランザクションで INSERT ──▶ [jobs テーブル]
                                                        │ ポーリング (FOR UPDATE SKIP LOCKED)
                                                        ▼
                                                    [worker]
                                                        │ 副作用を DB へ直接書き戻し
                                                        │ + jobs の state/progress/result を UPDATE
                                                        ▼
[frontend] ◀── GET /api/v1/jobs/{id} をポーリング (TanStack Query refetchInterval) ── [api]
```

**方式選択の根拠**: ブローカー（Redis/RabbitMQ）は導入しない。単一ユーザー・セルフホスト・ミニPC 1台という前提でジョブ流量が小さく、PostgreSQL は既に必須依存であるため、運用対象を増やさない DB-as-queue が妥当。結果反映も SSE ではなく**ポーリングに統一**する（nginx のバッファリング設定・接続管理・再接続処理を持ち込まずに済み、worker 停止時の縮退挙動が自明になるため）。

---

## 1. `jobs` テーブル

既存 `init_schema.up.sql` の作法（`UUID PRIMARY KEY DEFAULT gen_random_uuid()`、`TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP`、`update_updated_at_column()` トリガ、`idx_<table>_<column>` 命名）に合わせる。新規マイグレーションファイルとして追加する。

```sql
CREATE TYPE job_type AS ENUM (
    'extract_text', 'index', 'resolve_links', 'wiki', 'embed'
);

CREATE TYPE job_state AS ENUM (
    'queued', 'running', 'succeeded', 'failed', 'cancelling', 'cancelled'
);

CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type job_type NOT NULL,
    state job_state NOT NULL DEFAULT 'queued',
    payload JSONB NOT NULL,
    result JSONB,
    error TEXT,
    progress_current INTEGER NOT NULL DEFAULT 0,
    progress_total INTEGER,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    run_after TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    locked_by VARCHAR(255),
    locked_at TIMESTAMP,
    target_item_id UUID REFERENCES items(id) ON DELETE CASCADE,
    dedup_key VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT chk_jobs_progress CHECK (
        progress_total IS NULL OR progress_current <= progress_total
    )
);

-- worker のポーリング用。未実行ジョブのみを対象にする部分インデックス
CREATE INDEX idx_jobs_queued ON jobs(run_after, created_at) WHERE state = 'queued';
-- クラッシュ回復（reaper）用
CREATE INDEX idx_jobs_running_locked_at ON jobs(locked_at) WHERE state = 'running';
-- item 詳細画面で「この item に紐づく処理中ジョブ」を引く用
CREATE INDEX idx_jobs_target_item_id ON jobs(target_item_id);
-- 未終了ジョブに限った重複 enqueue の抑止
CREATE UNIQUE INDEX uq_jobs_dedup_key_active ON jobs(dedup_key)
    WHERE dedup_key IS NOT NULL AND state IN ('queued', 'running', 'cancelling');

CREATE TRIGGER trg_jobs_updated_at BEFORE UPDATE ON jobs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

### 状態遷移

```
queued ──(worker がロック取得)──▶ running ──▶ succeeded
   ▲                                 │
   │                                 ├──▶ failed        (attempts >= max_attempts)
   │                                 │
   └──(失敗リトライ / reaper 回収)────┘
                                     │
cancelling ◀──(POST /jobs/{id}/cancel)┘
   │
   └──(worker がチェックポイントで観測)──▶ cancelled
```

- `queued` / `running` / `cancelling` が**進行中**、`succeeded` / `failed` / `cancelled` が**終端**。
- `queued` 状態でのキャンセルは worker を介さず api がその場で `cancelled` にできる。
- `succeeded`/`failed`/`cancelled` になった行は削除しない（履歴として残す）。保持期間の定期削除は将来検討事項とする。

---

## 2. 投げ方（enqueue の契約）

### 2-1. api → worker は「関数呼び出し」ではなく「同一トランザクション内の INSERT」

パイプラインジョブは、**それを引き起こした業務処理と同一の DB トランザクションで `jobs` に INSERT する**。

例: `POST /api/v1/items/{id}/files`（ファイル登録）の場合

```
BEGIN
  INSERT INTO item_files (...) RETURNING id
  INSERT INTO jobs (job_type, payload, target_item_id, dedup_key)
       VALUES ('extract_text', $payload, $item_id, 'extract_text:' || $item_file_id)
COMMIT
```

こうすることで、

- ファイル登録が成功したのにジョブが投入されない（取りこぼし）
- ジョブは投入されたのにファイル登録がロールバックされた（孤児ジョブ）

の両方が構造的に起きない。transactional outbox と同じ考え方であり、DB がそのままキューであるためアウトボックス表を別に持つ必要がない。

**api は worker の完了を待たない。** enqueue した時点でリクエストは返す（`202 Accepted` 相当だが、既存の共通レスポンス形式に合わせ `201` + `ApiOk<JobResponse>` で `job_id` を返す）。

### 2-2. job_type ごとの payload スキーマ

`payload` は `job_type` によって形が変わる JSONB。api / worker 双方で serde の tagged enum（`#[serde(tag = "job_type", content = "payload")]` 相当）として型付けし、`payload` の妥当性は enqueue 時に検証する。

| job_type | enqueue する主体 | トリガー | payload |
|---|---|---|---|
| `extract_text` | api | `POST /items/{id}/files`・`/files/upload` で `file_type='pdf'` のファイルが登録されたとき | `{ "item_file_id": uuid, "path": string }` |
| `index` | api / worker | items のメタデータ更新時、および `extract_text` 完了時（worker が後続を enqueue） | `{ "item_id": uuid }` |
| `resolve_links` | api | ファイル登録時、および視聴リンク未解決の item を検出したとき | `{ "item_id": uuid, "hints": { "title": string, "media_type": media_type } }` |
| `wiki` | mcp（エージェント） | `enqueue_job` ツール | `{ "item_id": uuid, "source": "extracted_text" \| "description" }` |
| `embed` | mcp（エージェント） | `enqueue_job` ツール | `{ "item_id": uuid, "source": "wiki" \| "description" }` |

`wiki`/`embed` の**生成ロジックは既定では worker に実装しない**（[04](04_jobs-and-agent-integration.md) の責務分界に従う）。worker は KnowledgeHub 側エージェントが生成した結果を `knowledge` に格納する経路と、大量件数のバッチ実行の枠だけを提供する。

### 2-3. 冪等性

`dedup_key` により、**未終了（`queued`/`running`/`cancelling`）の同一ジョブが既にある場合は新規 enqueue を握りつぶす**。

- 命名規約: `<job_type>:<対象の主キー>`（例 `extract_text:{item_file_id}`、`index:{item_id}`）
- 実装は `INSERT ... ON CONFLICT DO NOTHING` + `RETURNING id`。何も返らなかった場合は既存ジョブの id を引き直して返す。
- 終端状態の行は部分ユニークインデックスの対象外なので、再実行したいときは同じ `dedup_key` で再 enqueue できる。

`dedup_key` を `NULL` にすれば重複抑止なしで積める（`wiki` の手動再生成など、意図的に何度も走らせたいケース）。

---

## 3. 受け取り方（worker 側プロトコル）

### 3-1. ジョブ取得

worker は N 秒間隔（既定 5 秒、`WORKER_POLL_INTERVAL_SECS`）でポーリングし、以下のクエリでロックと状態遷移を 1 ステートメントで行う。

```sql
UPDATE jobs
   SET state = 'running',
       locked_by = $1,          -- worker のインスタンス識別子（hostname + pid）
       locked_at = CURRENT_TIMESTAMP,
       attempts = attempts + 1
 WHERE id IN (
     SELECT id FROM jobs
      WHERE state = 'queued'
        AND run_after <= CURRENT_TIMESTAMP
      ORDER BY created_at
      FOR UPDATE SKIP LOCKED
      LIMIT $2
 )
RETURNING *;
```

`FOR UPDATE SKIP LOCKED` により、worker を複数プロセス／複数コンテナに増やしても同じジョブを二重実行しない。`LIMIT` は同時実行数（既定 1、`WORKER_CONCURRENCY`）。

### 3-2. 進捗報告

worker は処理中に `progress_current` / `progress_total` を直接 UPDATE する。更新頻度は「1 秒に 1 回程度」を上限とし、細かいループごとに UPDATE しない（DB への書き込み過多を避けるため）。

```sql
UPDATE jobs SET progress_current = $1, progress_total = $2 WHERE id = $3;
```

### 3-3. 完了・失敗・リトライ

| 結果 | 更新内容 |
|---|---|
| 成功 | `state='succeeded'`, `result=$json`, `error=NULL`, `locked_by=NULL`, `locked_at=NULL` |
| 失敗（リトライ可） | `state='queued'`, `error=$msg`, `run_after = CURRENT_TIMESTAMP + backoff`, `locked_by=NULL`, `locked_at=NULL` |
| 失敗（打ち切り） | `state='failed'`, `error=$msg`, `locked_by=NULL`, `locked_at=NULL` |

**リトライ方針**（PRD で「実装時に定める」とされていた箇所をここで確定する）:

- `max_attempts` 既定 **3**
- バックオフ = **`30 秒 × 2^(attempts-1)`**（30s → 60s → 120s）。ジッタは加えない（単一 worker のためサンダリングハード問題が起きない）
- `attempts >= max_attempts` に達したら `failed`。自動での無限リトライはしない
- **リトライ不能な失敗は即 `failed`**（対象ファイルが存在しない、payload が壊れている等、再実行しても結果が変わらないもの）。ネットワーク・外部 API・一時的 I/O エラーのみリトライ対象とする

`failed` になったジョブの再実行はユーザー操作（同じ `dedup_key` での再 enqueue）に委ねる。

### 3-4. クラッシュ回復（reaper）

worker が処理中に落ちると `running` のまま残る。worker は起動時と定期（既定 60 秒毎）に以下を実行する。

```sql
UPDATE jobs
   SET state = 'queued', locked_by = NULL, locked_at = NULL
 WHERE state = 'running'
   AND locked_at < CURRENT_TIMESTAMP - INTERVAL '10 minutes';
```

しきい値（`WORKER_LOCK_TIMEOUT_SECS`、既定 600）は、最も長いジョブの想定実行時間より十分長く取る。長時間ジョブは 3-2 の進捗更新に合わせて `locked_at` も更新（ハートビート）し、誤回収を防ぐ。

### 3-5. キャンセル

既存インポートジョブの `Cancelling` → `Cancelled` と同じ二段構えにする。

1. `POST /api/v1/jobs/{id}/cancel` を受けた api が `state='queued'` なら即 `cancelled`、`state='running'` なら `cancelling` に更新する
2. worker はジョブ処理中のチェックポイント（進捗更新のタイミング）で自分のジョブの `state` を読み、`cancelling` なら処理を打ち切って `cancelled` に落とす
3. 終端状態のジョブへの cancel は `409` を返す

### 3-6. 副作用の書き戻し先

**worker は api を経由せず DB へ直接書き込む。** これは「書き込みは `/api` に一本化する」という [03_api-design.md](03_api-design.md) の原則に対する明示的な例外であり、その根拠は:

- worker は MediaVault の内部コンポーネントであり、外部クライアント（web/mcp）とは信頼境界が異なる
- ジョブ状態の更新（`jobs` の UPDATE）と副作用の書き込みを**同一トランザクションに収める**必要がある。api 経由にすると「副作用は書けたがジョブ完了マークに失敗した」状態が生じ、リトライ時に二重適用される

| job_type | 書き戻し先 |
|---|---|
| `extract_text` | 抽出テキストの格納先カラム（下記「未決事項」参照） |
| `index` | 検索インデックス（Postgres FTS の tsvector カラム、または Meilisearch） |
| `resolve_links` | `item_links` に INSERT（Jellyfin/Calibre-Web の URL） |
| `wiki` / `embed` | `knowledge` テーブル |

`jobs` の更新と副作用の書き込みは 1 トランザクションで COMMIT する。したがって各ジョブハンドラは**冪等に**実装する（同じジョブが 2 回走っても `item_links` が重複しない等）。

---

## 4. クライアント/フロントへの反映

### 4-1. API（詳細は [../backend/mediavault-api/jobs.md](../backend/mediavault-api/jobs.md)）

| Method | Path | 用途 |
|---|---|---|
| POST | `/internal/jobs` | 明示的な enqueue（mcp の `enqueue_job` 用）。内部APIなので `INTERNAL_API_KEY` 認証 |
| GET | `/api/v1/jobs/{id}` | 状態・進捗・結果・エラーの取得（フロントのポーリング先） |
| GET | `/api/v1/jobs?target_item_id=&state=&job_type=` | item に紐づくジョブ一覧 |
| POST | `/api/v1/jobs/{id}/cancel` | キャンセル要求 |

enqueue を `/internal` に置くのは、公開API（`/api/v1`）が無認証であり、任意のクライアントが重い処理を無制限に積めるのを避けるため。パイプラインジョブは api が内部的に INSERT するので公開の enqueue エンドポイントは不要。

### 4-2. フロントエンド（ポーリング）

SSE/WebSocket は使わず、**TanStack Query の `refetchInterval` に統一する**。

```ts
// hooks/useJob.ts（新規）
export function useJob(jobId: string | null) {
  return useQuery({
    queryKey: ["job", jobId],
    queryFn: () => fetchJob(jobId!),
    enabled: jobId !== null,
    // 進行中のみ 1.5 秒間隔、終端状態に達したら false を返してポーリングを止める
    refetchInterval: (query) =>
      isTerminal(query.state.data?.state) ? false : 1500,
  });
}
```

- 終端到達時に対象 item のクエリキーを `invalidateQueries` し、詳細/一覧画面へ結果を反映する
- 画面遷移でポーリングは自然に止まる（`useQuery` のアンマウント）。ジョブ自体は動き続け、戻ってくれば `jobs` テーブルから現在の状態を取り直せる — これはプロセス内 `HashMap` 方式にはできなかった性質
- エラー時は `error` フィールドをトースト/インライン表示する

### 4-3. 既存ブクログインポートの扱い

現行の `SettingsPage.tsx` の手書き `sleep` ループと `useSettingsData.ts` の `startBooklogImport` / `fetchBooklogImportJob` / `cancelBooklogImportJob` は、上記 `useJob` と同じ形に寄せられる。ただし**ブクログインポート自体を `jobs` テーブルへ移すのは本設計のスコープ外とし、当面は併存させる**。理由:

- ブクログインポートは multipart アップロードされた CSV の中身をジョブに持ち回る必要があり、`payload JSONB` にそのまま載らない（別途ファイルの一時保存先の設計が要る）
- 現行実装は動作しており、api プロセス内で完結する短時間ジョブなので、永続化の便益が相対的に小さい

移行するとしても `jobs` 基盤が稼働した後の別タスクとする。その際は `job_type` に `import_booklog` を追加し、CSV を `/data` の一時領域に保存して `payload` にはパスのみを持たせる。

### 4-4. worker 停止時の縮退

worker が停止していても:

- ジョブは `queued` のまま滞留し、失われない
- フロントは「処理待ち」を表示し続ける（ポーリングはユーザーが画面を離れれば止まる）
- 一覧/検索/詳細などメタデータ層は通常どおり動作する（PRD の「部分的縮退運転」方針）

worker 復帰時に滞留分が順次処理される。

---

## 5. 未決事項（実装着手前に決める必要があるもの）

- **`extract_text` の格納先**: 現行スキーマの `item_files` に本文テキスト用カラムが無い。`item_files.extracted_text TEXT` を追加するか、`item_texts` 別表にするか未決。全文検索の設計（Postgres FTS の tsvector をどこに置くか）と併せて決める。
- **`item_links.kind`**: [02_data-model.md](02_data-model.md) は `item_links.kind` = `jellyfin`/`calibre`/`url` としているが、実スキーマの `item_links` は `url` と `label` のみで `kind` カラムが無い。`resolve_links` の実装前にどちらかへ揃える必要がある。
- **`knowledge` テーブル**: 未定義。`wiki`/`embed` の実装は `knowledge` のスキーマ確定が前提。
- **検索バックエンド**: Postgres FTS か Meilisearch か未決（`index` ジョブの中身が決まらない）。

依存の小ささから、**実装は `resolve_links` を最初の 1 種として着手するのが妥当**（新規テーブル・新規カラムを必要とせず、`item_links` への INSERT のみで完結する）。

---

## 関連ドキュメント

- [04_jobs-and-agent-integration.md](04_jobs-and-agent-integration.md) — ジョブ種別とKnowledgeHubとの責務分界
- [02_data-model.md](02_data-model.md) — `jobs` を含むデータモデル全体
- [03_api-design.md](03_api-design.md) — API全体方針（worker のDB直接書き込みは本ページ 3-6 の例外）
- [../backend/mediavault-worker/PRD.md](../backend/mediavault-worker/PRD.md)
- [../backend/mediavault-api/jobs.md](../backend/mediavault-api/jobs.md) — エンドポイント詳細仕様
