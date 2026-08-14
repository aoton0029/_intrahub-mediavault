# MediaVault Extractor コンテキストノート

**要件名**: extractor-text-extraction（Extractor全文抽出 / jobs廃止・抽出専用API再設計）
**作成日**: 2026-08-14
**作業規模**: フル機能開発

---

## 1. スコープ

本要件は3コンポーネントを対象とする。

| コンポーネント | 対象範囲 |
|---|---|
| `mediavault-extractor` | 新規。Python製常駐ポーリングworker。PDF/画像のテキスト抽出とOCR |
| `mediavault-api` | 拡張。**汎用jobsを廃止**し、`item_files` に従属する抽出リソースAPI・worker内部API・抽出結果テーブルを新設 |
| `mediavault-mcp` | 改訂。jobs系4ツール（`enqueue_job` / `get_job` / `list_jobs` / `cancel_job`）を抽出系3ツールへ再定義 |

---

## 2. 前提となる調査結果（2026-08-14 時点）

**汎用jobsは「参照だけが存在し、実体が一切ない」状態である。**

| 確認項目 | 結果 |
|---|---|
| `docs/backend/mediavault-api/jobs.md` | **存在しない**。index.md / internal-api.md / item-text.md / mediavault-mcp設計書3本 から参照されているリンク切れ |
| `jobs` テーブル | 未定義。migration は `20260623000001_init_schema` の1本のみ |
| jobs ハンドラ | `backend/mediavault-api/src/handlers/` に存在しない |
| MCP jobs ツール | `backend/mediavault-mcp/src/tools/` に存在しない |
| `item_file_texts` テーブル | 未定義。`GET /items/{id}/text` も未実装 |

したがって jobs 廃止は**既存実装への破壊的変更を伴わない**。作業はドキュメント改訂と新規実装のみである。

なお `/import/booklog/jobs/*` は Booklog インポート専用の別機構であり、本要件の対象外。廃止しない。

---

## 3. 技術スタック

### mediavault-api（既存）
- Rust / Axum / sqlx / PostgreSQL
- 公開API `/api/v1/*`（認証なし・単一ユーザーセルフホスト）
- 内部API `/internal/*`（`api_key_auth` ミドルウェア、`INTERNAL_API_KEY`）
- レスポンス形式: `ApiOk<T>` / `PaginatedOk<T>` / `ApiError`（[index.md](../../backend/mediavault-api/index.md)）

### mediavault-extractor（新規）
- Python 3.12 / uv / Docker
- 同期ループ（asyncio不使用）、heartbeat のみ daemon thread 1本
- httpx + tenacity / pypdfium2 / puremagic / Pillow / yomitoku
- structlog（JSON）/ pydantic-settings / pytest / Ruff / mypy --strict
- 詳細: [tech-stack.md](../tech-stack.md)

### mediavault-mcp（既存）
- Rust。MediaVault-api を経由してのみ読み書きし、データを所有しない

---

## 4. 関連ファイル

### 参照した設計文書
| ファイル | 内容 |
|---|---|
| [docs/extractor/PRD.md](../PRD.md) | 本要件の一次資料。FR-001〜011、NFR-001〜004、§8 API拡張要求 |
| [docs/extractor/tech-stack.md](../tech-stack.md) | worker側の技術選定と未決事項 |
| [docs/backend/mediavault-api/index.md](../../backend/mediavault-api/index.md) | エンドポイント一覧・エラーコード表・共通レスポンス形式 |
| [docs/backend/mediavault-api/internal-api.md](../../backend/mediavault-api/internal-api.md) | 既存内部API 5本とパス規約 |
| [docs/backend/mediavault-api/item-text.md](../../backend/mediavault-api/item-text.md) | チャンク規約・`extraction_version`・エラー使い分け |
| [docs/backend/mediavault-api/item-files.md](../../backend/mediavault-api/item-files.md) | `path` の2経路（リンク=絶対パス / アップロード=相対パス）、`file_type` 自動分類 |
| [docs/backend/mediavault-mcp/design/api-tool-mapping.md](../../backend/mediavault-mcp/design/api-tool-mapping.md) | §2.10 jobs、§内部APIキー判定 |
| [docs/backend/mediavault-mcp/design/mastra-integration.md](../../backend/mediavault-mcp/design/mastra-integration.md) | `get_item_text` / `not_extracted` の扱い |

### 改訂が必要なファイル（本要件の成果として次フェーズで実施）
| ファイル | 改訂内容 |
|---|---|
| `docs/backend/mediavault-api/index.md` | `/jobs` 3行削除 → extraction 3本追加、エラーコード表更新、目次から `jobs.md` 除去 |
| `docs/backend/mediavault-api/internal-api.md` | `POST /internal/jobs` 削除、パス規約を `/api/v1/internal/*` へ、worker API 5本追加 |
| `docs/backend/mediavault-api/item-text.md` | `extract_text` ジョブ記述を extraction リソースへ、`label` 常時null記述を撤回 |
| `docs/backend/mediavault-api/data-model.md` | `item_file_extractions` / `item_file_texts` 追加 |
| `docs/backend/mediavault-api/extraction.md` | **新規作成**（jobs.md は作らない） |
| `docs/extractor/PRD.md` | §8 全体、§11 要調整事項1〜5 を決定事項へ |
| `docs/extractor/tech-stack.md` | §「このファイルで決めていないこと」の3項目を解決済みへ |
| `docs/backend/mediavault-mcp/design/mcp-tools.md` | jobs系4ツール → extraction系3ツール |
| `docs/backend/mediavault-mcp/design/api-tool-mapping.md` | §2.10 差し替え、§内部API記述更新 |
| `docs/backend/mediavault-mcp/design/mastra-integration.md` | `enqueue_job` 参照を `request_extraction` へ |
| `docs/backend/mediavault-mcp/PRD.md` | §7.2 ツール表、§api依存表 |
| `docs/backend/mediavault-mcp/spec/requirements.md` | REQ-901 の再定義 |

### 存在しない参照先（改訂時に除去する）
- `docs/backend/mediavault-api/jobs.md` — リンク切れ
- `docs/basic-design/04_jobs-and-agent-integration.md` — `docs/basic-design/` ディレクトリごと存在しない

---

## 5. 開発ルール

- **プロジェクトルート基準の相対パス**で記述する。絶対パスを書かない
- ドキュメント内の各項目に信頼性レベル 🔵🟡🔴 と出典を付す
- API のエラーコードは [index.md](../../backend/mediavault-api/index.md) の共通表に必ず追加する
- Extractor は **DBへ直接接続しない**（依存関係にDBドライバを含めないことで強制）
- ログに `INTERNAL_API_KEY`・抽出本文・画像・個人情報を出力しない

---

## 6. 注意事項

1. **`item_files.path` の二重性**: リンク経路は絶対パス、アップロード経路は `STORAGE_ROOT` からの相対パス。内部APIが worker へ返す参照はこの差を吸収し、worker 側の許可ルート（`/library` / `/srv/mediavault`）で解決可能な形にする必要がある。
2. **`file_type` は6値**（pdf / image / video / audio / archive / other）。実装済みの自動分類に対し、MVP の抽出対象は pdf と image のみ。
3. **GPU は空いていない**: vLLM が `--gpu-memory-utilization 0.90` で常時予約。CPU実行が標準運用。
4. **チャンク index は形式非依存の0起点連番**。これは intrahub-mastra の出典参照仕様に由来する不変の規約であり、boundaries 導入によって崩してはならない。
5. **キャンセル後に成功を確定してはならない**。worker とAPIの両側で担保する。

---

## 7. 関連文書

- **要件定義書**: [requirements.md](requirements.md)
- **ヒアリング記録**: [interview-record.md](interview-record.md)
- **ユーザストーリー**: [user-stories.md](user-stories.md)
- **受け入れ基準**: [acceptance-criteria.md](acceptance-criteria.md)
- **準備タスク**: [prep.md](prep.md)
