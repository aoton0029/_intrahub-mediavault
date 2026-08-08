# MediaVault 基本設計 — API設計方針

← [00_overview.md](00_overview.md)

本ページは `MediaVault-api` が提供する単一 `/api` の全体方針と、各クライアント（web/mcp/worker）からの利用パターンを整理する。エンドポイント個々の詳細（パラメータ・レスポンス例・エラー）は [../backend/mediavault-api/index.md](../backend/mediavault-api/index.md) を参照。

## 設計方針

- **単一エンドポイント**: すべてのデータ操作は `MediaVault-api` の `/api` を経由する。`MediaVault-web` も `MediaVault-mcp` もDBへ直接アクセスしない。
- **書き込みの一本化**: データ変更経路は `/api` のみ。
- **ナレッジを扱わない**: 要約/wiki/embeddingは生成も格納もAPIの責務ではない。正本はKnowledgeHub Vaultにあり、`/api` は材料提供とジョブ登録に限定する（[04_jobs-and-agent-integration.md](04_jobs-and-agent-integration.md#knowledgehubとの責務分界)）。

## カテゴリ別エンドポイント一覧

| カテゴリ | 例 | 役割 |
|---|---|---|
| メタデータ | `/api/items`, `/api/tags`, `/api/categories` 等 | CRUD、タグ/リンク/ファイルの関連付け |
| 検索 | `GET /api/search?q=` | `items` に対する検索。バックエンドはPostgres FTS（既定）またはMeilisearch |
| 視聴リンク | `GET /api/items/{id}/links` | `item_links` の取得（Jellyfin/Calibre-Webへの導線） |
| ファイル | `GET/PUT/DELETE /api/files/*`（WebDAV含む） | `/data` の閲覧/アップロード/配信 |
| ジョブ | `POST /internal/jobs`, `GET /api/v1/jobs/{id}` | ジョブ登録（内部API）/進捗確認（[05_job-queue.md](05_job-queue.md)、[jobs.md](../backend/mediavault-api/jobs.md)） |
| 監視 | `GET /api/health`, `/metrics` | ヘルスチェック/Prometheusメトリクス |

詳細は各リソースドキュメント（[items.md](../backend/mediavault-api/items.md)、[item-links.md](../backend/mediavault-api/item-links.md) 等、[index.md](../backend/mediavault-api/index.md) の一覧を参照）。

## クライアント別利用パターン

| クライアント | 用途 | 呼び出すAPIカテゴリ |
|---|---|---|
| MediaVault-web | 一覧/検索/詳細/登録/編集UI | メタデータ、検索、視聴リンク、ファイル |
| MediaVault-mcp | AIエージェントへのツール提供 | メタデータ（取得系・追記/更新系）、検索、視聴リンク、ジョブ登録 |
| MediaVault-worker | ジョブ実行結果の反映 | **DB直接**（ジョブ状態更新と副作用を同一トランザクションに収める必要があるため、`/api` 一本化の明示的な例外。[05_job-queue.md](05_job-queue.md) 3-6） |
| Jellyfin | メタデータ参照（プラグイン経由、任意） | メタデータ（読み取り専用） |

## 関連ドキュメント

- [../backend/mediavault-api/index.md](../backend/mediavault-api/index.md)（ベースURL/認証/共通レスポンス形式/エラーコード）
- [02_data-model.md](02_data-model.md)
- [04_jobs-and-agent-integration.md](04_jobs-and-agent-integration.md)
- [../backend/mediavault-mcp/PRD.md](../backend/mediavault-mcp/PRD.md)
