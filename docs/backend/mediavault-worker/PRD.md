# MediaVault-worker

## 概要
MediaVaultの非同期ジョブ実行部分のPRD。`jobs` テーブルをポーリングし、ファイル処理・検索インデックス更新・外部連携リンク解決などのパイプラインジョブと、KnowledgeHub側エージェントが依頼するナレッジ生成ジョブを実行する。
全体構想は[ルートPRD](../../PRD.md)を参照。バックエンド側は[backend/PRD.md](../PRD.md)、基本設計全体は[basic-design/00_overview.md](../../basic-design/00_overview.md)を参照。

## 技術スタック
| 要素 | 技術 |
|------|------|
| ワーカー実装 | Rust |
| データベース | PostgreSQL（`jobs`テーブルをポーリング） |
| ファイルアクセス | `/srv/anime`・`/srv/live-action`・`/srv/manga`（ro） |
| デプロイ | Docker |

## 設計方針
- workerが停止してもメタデータ層（一覧/検索/詳細）は動作し続ける（部分的縮退運転）。
- ナレッジ生成（要約/wiki/embeddingの「どう作るか」）は既定ではworkerの責務としない。エージェント側で生成された結果を格納するのが基本であり、worker自身が生成ロジックを持つのはオプション（LLMエンドポイント自前運用時のみ）。

## ジョブ種別

### パイプラインジョブ（自動・エージェント非関与）
`MediaVault-api` がファイル登録時に自動でenqueueする。

| ジョブ種別 | トリガー | 内容 |
|---|---|---|
| `extract_text` | ファイル登録（PDF等） | テキスト抽出（全文検索インデックス用） |
| `index` | メタデータ変更・`extract_text`完了 | 検索インデックスの更新 |
| `resolve_links` | ファイル登録・視聴リンク未解決時 | Jellyfin/Calibre-Web APIを呼び出し `item_links` へ登録 |

### エージェント駆動ジョブ（判断はエージェント、実行のみworker）
`MediaVault-mcp` の `enqueue_job` 経由でKnowledgeHub側エージェントがenqueueする。

| ジョブ種別 | 内容 |
|---|---|
| `wiki` | 要約/wikiページ生成 → `knowledge` へ格納 |
| `embed` | embedding生成 → `knowledge` へ格納 |

## 依存関係
| 依存 | 用途 | 備考 |
|---|---|---|
| PostgreSQL | ジョブキュー・メタデータ更新 | 必須 |
| 検索バックエンド（Postgres FTS / Meilisearch） | `index`ジョブ | 必須 |
| `/srv/anime`・`/srv/live-action`・`/srv/manga`（ro） | ファイル処理 | 必須 |
| LLMエンドポイント（`LLM_BASE_URL`/`LLM_API_KEY`） | `wiki`/`embed`をworker自身が実行する場合のみ | オプション（既定はKnowledgeHub側エージェントに委譲） |

## ジョブキュー設計
`jobs` テーブル定義、api からの enqueue 契約（業務処理と同一トランザクション内の INSERT）、worker のジョブ取得プロトコル（`FOR UPDATE SKIP LOCKED`）、進捗報告、キャンセル、クラッシュ回復、副作用の書き戻し先は [basic-design/05_job-queue.md](../../basic-design/05_job-queue.md) に定義済み。

## リトライ方針
ジョブは状態（state）・リトライ回数を `jobs` テーブルで管理し、失敗時は再試行する。試行上限は既定 3 回、バックオフは `30秒 × 2^(attempts-1)`。リトライ不能な失敗（対象ファイル不在・payload 破損など）は再試行せず即 `failed` とする。詳細は [basic-design/05_job-queue.md](../../basic-design/05_job-queue.md) の「3-3. 完了・失敗・リトライ」を参照。

## やらなくていいこと
- `wiki`/`embed`の生成ロジックを既定でworker内に実装すること（既定ではKnowledgeHub側エージェントに委譲し、workerは結果格納・ジョブ実行のみを担う）
- ユーザー向けのAPI/UIを提供すること（workerは非公開のバックグラウンド処理のみを担い、外部からの直接アクセスは想定しない）
