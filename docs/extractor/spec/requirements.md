# MediaVault Extractor 全文抽出 要件定義書

**作成日**: 2026-08-14
**作業規模**: フル機能開発
**対象**: `mediavault-extractor`（新規Python worker）/ `mediavault-api`（抽出専用API新設・**汎用jobs廃止**）/ `mediavault-mcp`（ツール再定義）

## 概要

MediaVaultに登録されたファイル（MVPではPDFと画像）から、非同期にプレーンテキストを抽出し、`GET /api/v1/items/{id}/text` を通じてAIエージェントへ提供する仕組みを構築する。

PRD §8 は当初これを**汎用 `jobs` テーブル + Jobs API** の上に構築する前提だったが、以下の理由により**汎用jobsを廃止し、`item_files` に従属する抽出専用リソースへ再設計する**。

1. 汎用jobsは仕様の一部と参照だけが先行しており、実装が一切存在しない（[note.md](note.md) §2）。廃止コストがゼロである。
2. MVPの `job_type` は `extract_text` 1種のみであり、`job_type` による分岐・`dedup_key` による重複防止・型なし `payload` は、単一用途に対して過剰な抽象である。
3. 抽出対象は常に `item_file` 1件であり、「1ファイルにつき未完了の抽出は最大1件」という制約をDBの部分UNIQUE indexで表現できる。これにより `dedup_key` の設計・生成・衝突検討がすべて不要になる。
4. パス（`/items/{id}/files/{file_id}/extraction`）に対象が現れるため、Itemとファイルの対応検証がルーティング段階で自然に行われる。

将来 `job_type` が2種以上必要になった時点で、その時の実需に基づいて汎用化を検討する。

## 関連文書

- **ヒアリング記録**: [💬 interview-record.md](interview-record.md)
- **ユーザストーリー**: [📖 user-stories.md](user-stories.md)
- **受け入れ基準**: [✅ acceptance-criteria.md](acceptance-criteria.md)
- **コンテキストノート**: [📝 note.md](note.md)
- **準備タスク**: [🔧 prep.md](prep.md)
- **PRD**: [PRD.md](../PRD.md)
- **技術スタック**: [tech-stack.md](../tech-stack.md)

---

## 確定した設計判断

PRD §11「要調整事項」およびヒアリングで確定した事項。詳細な経緯は [interview-record.md](interview-record.md)。

| # | 論点 | 決定 | 出典 |
|---|---|---|---|
| D-1 | 公開リソース表現 | ファイル従属の抽出リソース `/items/{id}/files/{file_id}/extraction`。`dedup_key` 廃止 | ヒアリングQ1 |
| D-2 | 公開入力 | `item_file_id`（パスパラメータ）を正本とし、ホストパスは受け取らない | PRD §11-1 |
| D-3 | 内部APIパス規約 | `/api/v1/internal/*` に統一。既存 `/internal/*` 5本も移設 | ヒアリングQ3・PRD §11-2 |
| D-4 | ファイル受け渡し | read-only 共有ボリューム。内部APIは検証済み参照を返す | ヒアリングQ2・PRD §11-3 |
| D-5 | 結果送信 | 完了時に一括送信。抽出結果保存と成功遷移を同一トランザクション | ヒアリングQ4・PRD §11-4 |
| D-6 | 境界情報 | MVPから jsonb で保存し `label` を返す | ヒアリングQ5・PRD §11-5 |
| D-7 | 自動抽出 | 行わない。明示的な抽出リクエストのみ | ヒアリングQ6 |
| D-8 | MCPツール | jobs系4ツールを廃止し extraction系3ツールへ再定義 | ヒアリングQ7 |

---

## 機能要件（EARS記法）

**【信頼性レベル凡例】**:
- 🔵 **青信号**: PRD・設計文書・ユーザヒアリングを参考にした確実な要件
- 🟡 **黄信号**: PRD・設計文書・ユーザヒアリングから妥当な推測による要件
- 🔴 **赤信号**: PRD・設計文書・ユーザヒアリングにない推測による要件

### 通常要件

#### A. mediavault-api — 公開API

- **REQ-001**: システムは `POST /api/v1/items/{id}/files/{file_id}/extraction` を提供し、指定ファイルのテキスト抽出をリクエストできなければならない 🔵 *ヒアリングQ1・PRD §8.3（jobs廃止に伴いパス変更）*
- **REQ-002**: システムは `GET /api/v1/items/{id}/files/{file_id}/extraction` を提供し、抽出の状態・進捗・エラー・最終結果メタデータを返さなければならない 🔵 *ヒアリングQ1・PRD §8.3*
- **REQ-003**: システムは `POST /api/v1/items/{id}/files/{file_id}/extraction/cancel` を提供し、キャンセルを要求できなければならない 🔵 *ヒアリングQ1・PRD §8.3*
- **REQ-004**: システムは抽出作成前に、対象ファイルの存在、指定Itemへの帰属、対応形式であること、読み取り可能であることを検証しなければならない 🔵 *PRD §8.3*
- **REQ-005**: システムは `GET /api/v1/items/{id}/text` を [item-text.md](../../backend/mediavault-api/item-text.md) の規約に従い実装しなければならない。チャンク識別子は形式によらない0起点の連番とする 🔵 *PRD §8.6・item-text.md*
- **REQ-006**: システムは `GET /api/v1/items/{id}/text` のレスポンスに `chunk_size`、`total_chunks`、`label`、`extraction_version` を含めなければならない 🔵 *PRD §8.6*
- **REQ-007**: システムは1レスポンスに全文を含めず、要求されたチャンクのみを返さなければならない 🔵 *PRD §8.6*
- **REQ-008**: システムは巨大な本文をAPIプロセスのメモリへ全件ロードせず、DB側の `SUBSTRING` でチャンクを切り出さなければならない 🔵 *item-text.md 実装上の注意*

#### B. mediavault-api — worker内部API

- **REQ-020**: システムは内部API `POST /api/v1/internal/extractions/claim` を提供し、実行可能な抽出を1件だけ排他的に払い出さなければならない 🔵 *PRD §8.4 claim*
- **REQ-021**: システムは claim 時に lease token と lease 期限を発行し、claim レスポンスへ含めなければならない 🔵 *PRD §8.4・§8.5*
- **REQ-022**: システムは claim レスポンスに、検証済みのファイル参照（worker の許可ルートから解決可能なパス）、`item_file_id`、`file_type`、ファイルサイズを含めなければならない 🔵 *ヒアリングQ2・PRD §8.4 file access*
- **REQ-023**: システムは内部API `POST /api/v1/internal/extractions/{id}/heartbeat` を提供し、lease延長、進捗更新、キャンセル要求の有無の返却を一度に行わなければならない 🔵 *PRD §8.4 heartbeat*
- **REQ-024**: システムは内部API `POST /api/v1/internal/extractions/{id}/complete` を提供し、抽出本文・境界情報・抽出メタデータを保存して抽出を `succeeded` へ遷移させなければならない 🔵 *PRD §8.4 complete*
- **REQ-025**: システムは complete 処理において、抽出結果の保存と状態遷移を**同一トランザクション**で確定しなければならない 🔵 *ヒアリングQ4・PRD §8.2*
- **REQ-026**: システムは内部API `POST /api/v1/internal/extractions/{id}/fail` を提供し、構造化エラーを保存したうえで、再試行（`queued` へ戻す）か終了（`failed`）かを判定しなければならない 🔵 *PRD §8.4 fail*
- **REQ-027**: システムは内部API `POST /api/v1/internal/extractions/{id}/cancelled` を提供し、worker の停止確認を受けて `cancelled` へ遷移させなければならない 🔵 *PRD §8.4 cancel*
- **REQ-028**: システムは内部APIの全ルートに `INTERNAL_API_KEY` によるサービス認証を要求しなければならない 🔵 *PRD §8.4・NFR-002*
- **REQ-029**: システムは内部APIのパスを `/api/v1/internal/*` へ統一し、既存の `/internal/*` 5本（items作成・検索・更新、groups upsert、episodes upsert、files登録）も同規約へ移設しなければならない 🔵 *ヒアリングQ3・PRD §11-2*

#### C. mediavault-api — データモデル

- **REQ-040**: システムは `item_file_extractions` テーブルを持ち、少なくとも `id`、`item_file_id`、`state`、`attempts`、`max_attempts`、`progress_current`、`progress_total`、`claimed_by`、`lease_token`、`lease_expires_at`、`error`、`created_at`、`updated_at` を保持しなければならない 🔵 *PRD §8.1（`job_type`・`dedup_key`・`target_item_id` を除去して再構成）*
- **REQ-041**: システムは `item_file_texts` テーブルを持ち、`item_file_id`（`item_files` へのFK・UNIQUE）、`content`、`boundaries`、`extraction_version`、`extractor`、`extracted_at` を保持しなければならない 🔵 *PRD §8.2*
- **REQ-042**: システムは `boundaries` を jsonb 配列 `[{ "start": number, "end": number, "label": string }]` として保存し、`GET /items/{id}/text` の `label` 解決に用いなければならない 🔵 *ヒアリングQ5・PRD §8.2・FR-006*
- **REQ-043**: システムは `extractor` に使用抽出方式（`embedded_text` / `ocr`）、OCRエンジン名、OCR実行方式（`cpu` / `gpu`）、モデル識別子を保持しなければならない 🔵 *PRD FR-007*
- **REQ-044**: システムは `item_file_id` に対して未完了状態（`queued` / `running` / `cancelling`）の `item_file_extractions` が最大1件であることを、部分UNIQUE indexで強制しなければならない 🔵 *ヒアリングQ1・PRD §8.1 重複禁止要求*

#### D. mediavault-extractor（worker）

- **REQ-060**: workerは内部APIをポーリングし、実行可能な抽出を排他的に claim しなければならない 🔵 *PRD FR-001*
- **REQ-061**: workerは処理対象を `item_file_id` および内部APIが返した検証済み参照からのみ決定しなければならない 🔵 *PRD FR-002*
- **REQ-062**: workerは拡張子だけでなくMIME typeまたはファイルシグネチャを併用して形式を判定しなければならない 🔵 *PRD FR-003*
- **REQ-063**: workerは抽出本文に対し、文字コード（NFKC）、改行、連続空白、制御文字の正規化を行わなければならない 🔵 *PRD FR-005*
- **REQ-064**: workerは正規化において、原文の意味を変える校正やLLMによる書き換えを行ってはならない 🔵 *PRD FR-005*
- **REQ-065**: workerは抽出完了時に、抽出本文・境界情報・`extraction_version`・抽出日時・使用抽出方式・OCR実行方式/エンジン/モデル識別子を内部APIへ報告しなければならない 🔵 *PRD FR-007*
- **REQ-066**: workerはページまたは章などの安全な区切りで、進捗報告とキャンセル要求の確認を行わなければならない 🔵 *PRD FR-008*
- **REQ-067**: workerは抽出結果を完了時に一括送信しなければならない 🔵 *ヒアリングQ4・PRD §11-4*
- **REQ-068**: workerはページ・章の文字範囲と表示用 `label` を境界情報として構築しなければならない 🔵 *ヒアリングQ5・PRD FR-006*
- **REQ-069**: workerはOCRエンジンを `OcrEngine` Protocol 境界の内側に閉じ込め、yomitoku 固有型を境界外へ公開してはならない 🔵 *PRD §12・tech-stack.md*
- **REQ-070**: workerはDBへ直接接続してはならない 🔵 *PRD §5.1*
- **REQ-071**: workerはジョブID、ファイルID、処理形式、ページ数、処理時間、終了状態、OCR実行方式、エンジン、モデル識別子を構造化ログへ記録しなければならない 🔵 *PRD NFR-004*

#### E. mediavault-mcp

- **REQ-080**: システムは `request_extraction` ツールを提供し、`item_id` と `file_id` を指定して抽出を依頼できなければならない 🔵 *ヒアリングQ7・mcp PRD §7.2（`enqueue_job` を置換）*
- **REQ-081**: システムは `get_extraction_status` ツールを提供し、抽出の状態・進捗・エラーを取得できなければならない 🔵 *ヒアリングQ7（`get_job` / `list_jobs` を置換）*
- **REQ-082**: システムは `cancel_extraction` ツールを提供し、抽出をキャンセルできなければならない 🔵 *ヒアリングQ7（`cancel_job` を置換）*
- **REQ-083**: システムは `get_item_text` が `not_extracted` を返す際、`request_extraction` による解決を促すメッセージを含めなければならない 🔵 *mastra-integration.md §エラー分類*

#### F. 既存ドキュメントの改訂

- **REQ-090**: システム設計文書から汎用jobsの記述と `jobs.md` へのリンク切れ参照をすべて除去し、抽出リソースの記述へ置き換えなければならない 🔵 *ヒアリングQ1・[note.md](note.md) §4*
- **REQ-091**: `docs/backend/mediavault-api/extraction.md` を新規作成し、`jobs.md` は作成してはならない 🔵 *ヒアリングQ1*

### 条件付き要件

- **REQ-101**: 対象ファイルに未完了の抽出が既に存在する場合、`POST .../extraction` は新規作成せず既存の抽出を `200` で返さなければならない 🔵 *ヒアリングQ1・PRD §8.1 冪等要求*
- **REQ-102**: 対象ファイルの抽出が存在しないか終端状態である場合、`POST .../extraction` は新規の抽出を作成し `201` を返さなければならない 🟡 *REQ-101 の裏返しとして妥当な推測*
- **REQ-103**: 既に成功済みの抽出結果が存在するファイルに対して再抽出が完了した場合、システムは `item_file_texts` の該当行を置き換え、`extraction_version` を更新しなければならない 🔵 *PRD FR-010・item-text.md*
- **REQ-104**: 抽出ロジックまたはチャンク境界が変わる場合、workerは `extraction_version` を変更しなければならない 🔵 *PRD FR-010*
- **REQ-105**: PDFに有効なテキストレイヤーがある場合、workerはそれを優先しなければならない 🔵 *PRD FR-004*
- **REQ-106**: PDFのページにテキストが存在しない、または品質基準を満たさない場合、workerはそのページに限りOCRを実行しなければならない 🔵 *PRD FR-004*
- **REQ-107**: 対象が画像ファイルの場合、workerはOCRを実行しなければならない 🔵 *PRD FR-004*
- **REQ-108**: 未対応形式を検出した場合、workerは再試行不可の明確なエラーとして終了しなければならない 🔵 *PRD FR-003・FR-009*
- **REQ-109**: 一時的なAPI通信失敗が発生した場合、workerは指数バックオフで再試行しなければならない 🔵 *PRD FR-009・tech-stack.md（tenacity）*
- **REQ-110**: 破損ファイル、未対応形式、恒久的なOCR失敗が発生した場合、システムは無限に再試行してはならない 🔵 *PRD FR-009*
- **REQ-111**: `attempts` が `max_attempts` に達した状態で fail を受けた場合、システムは抽出を `failed`（終端）へ遷移させなければならない 🟡 *PRD FR-009 から妥当な推測*
- **REQ-112**: `attempts` が `max_attempts` 未満で再試行可能な fail を受けた場合、システムは抽出を `queued` へ戻し `attempts` を加算しなければならない 🟡 *PRD §8.5 状態遷移図から妥当な推測*
- **REQ-113**: `EXTRACTOR_OCR_DEVICE` に `cuda` が指定され、CUDA GPUを利用できない場合、workerは起動時エラーとし、抽出をclaimしてはならない 🔵 *PRD FR-011・§8.9*
- **REQ-114**: `EXTRACTOR_OCR_DEVICE` に未知の値が指定された場合、workerは起動時エラーとしなければならない 🔵 *PRD FR-011*
- **REQ-115**: `GET /items/{id}/text` で `file_id` が省略され、抽出済みファイルが2件以上ある場合、システムは `409 AMBIGUOUS_FILE` を候補一覧とともに返し、推測で選んではならない 🔵 *item-text.md 主ファイルの解決*
- **REQ-116**: ファイルは存在するが抽出結果がない場合、システムは `422 TEXT_NOT_EXTRACTED` を返し、`404 FILE_NOT_FOUND` と区別しなければならない 🔵 *item-text.md・PRD §8.8*
- **REQ-117**: 抽出が `queued` / `running` / `failed` のいずれであっても、抽出結果が存在しなければ `GET /items/{id}/text` は `422 TEXT_NOT_EXTRACTED` を返さなければならない。本APIは抽出の状態を参照しない 🔵 *item-text.md §全文抽出との関係*
- **REQ-118**: lease が期限切れの抽出が存在する場合、システムは当該抽出を再claim可能としなければならない 🔵 *PRD §8.1・§8.8*

### 状態要件

- **REQ-201**: 抽出が `queued` にある場合、システムは claim による `running` への遷移と、キャンセル要求による `cancelled` への即時遷移のみを許可しなければならない 🔵 *PRD §8.5 状態遷移図*
- **REQ-202**: 抽出が `running` にある場合、システムはキャンセル要求を受けて `cancelling` へ遷移させ、worker の確認を待って `cancelled` へ遷移させなければならない 🔵 *PRD §8.5*
- **REQ-203**: 抽出が `succeeded` / `failed` / `cancelled`（終端状態）にある場合、システムはいかなる状態遷移も拒否しなければならない 🔵 *PRD §8.5*
- **REQ-204**: 抽出が `cancelling` または `cancelled` にある場合、システムは complete を受理して成功として確定してはならない 🔵 *PRD §8.5・FR-008*
- **REQ-205**: 終端状態の抽出に対してキャンセルが要求された場合、システムは `409 EXTRACTION_ALREADY_FINISHED` を返さなければならない 🟡 *既存 `JOB_ALREADY_FINISHED` の定義から妥当な移植*
- **REQ-206**: 抽出が `failed` または `cancelled` で終わった場合、システムは既存の成功済み抽出結果を削除してはならない 🔵 *PRD §8.2*
- **REQ-207**: worker が cancel 要求を検知した状態では、workerは成功結果を確定してはならない 🔵 *PRD FR-008・tech-stack.md 実行ループ骨子*

### オプション要件

- **REQ-301**: システムは `GET /api/v1/items/{id}/files/{file_id}/extraction` のレスポンスに、直近の失敗理由と `attempts` を含めてもよい 🟡 *運用性の観点から妥当な推測*
- **REQ-302**: workerは `EXTRACTOR_MAX_CONCURRENCY` により同時実行数を設定できてもよい。既定値は1とする 🔵 *tech-stack.md*
- **REQ-303**: システムはヘルスチェックにおいて、プロセス生存、MediaVault-api到達性、選択中のOCRバックエンド到達性を区別できるようにしてもよい 🔵 *PRD NFR-004*
- **REQ-304**: EPUB対応を後から追加できるよう、境界情報のデータ構造は章境界を表現可能な形にしてもよい 🔵 *PRD FR-006・tech-stack.md*

### 制約要件

- **REQ-401**: システムはファイル登録（`POST /items/{id}/files`）およびアップロード（`POST /items/{id}/files/upload`）時に、抽出を自動的にキューしてはならない 🔵 *ヒアリングQ6*
- **REQ-402**: システムは外部から渡された任意の絶対パス・相対パスをそのまま開いてはならない 🔵 *PRD FR-002・NFR-002*
- **REQ-403**: workerはファイルを開く前に `Path.resolve()` でsymlinkを展開し、許可ルート配下であることを検証しなければならない 🔵 *PRD NFR-002・tech-stack.md*
- **REQ-404**: システムは共有ボリュームを read-only でマウントしなければならない 🔵 *PRD NFR-002・ヒアリングQ2*
- **REQ-405**: システムはAPIキー、抽出本文全体、画像、個人情報をログへ出力してはならない 🔵 *PRD NFR-002・NFR-004*
- **REQ-406**: 一般公開APIから claim、heartbeat、complete、fail、cancelled を実行できてはならない 🔵 *PRD §8.4*
- **REQ-407**: complete / fail / cancelled は claim 時に発行された lease token を要求し、一致しない場合は拒否しなければならない 🔵 *PRD §8.5*
- **REQ-408**: システムは抽出本文とエラー内容に保存サイズ上限を設けなければならない 🔵 *PRD §8.7*
- **REQ-409**: Itemまたはファイル削除時の抽出・抽出結果の扱いを、FK制約と業務処理で一貫させなければならない 🔵 *PRD §8.7*
- **REQ-410**: MVPの抽出対象形式は `pdf` と `image` に限定する。`video` / `audio` / `archive` / `other` は `UNSUPPORTED_FILE_TYPE` とする 🟡 *PRD §4.1・§4.2（動画・音声の文字起こしは対象外）と item-files.md の file_type 分類から妥当な推測*
- **REQ-411**: OCR実行方式は worker 起動時に確定し、処理中の抽出に対して動的に変更してはならない 🔵 *PRD FR-011*
- **REQ-412**: システムは yomitoku による暗黙のCPUフォールバックに依存してはならない 🔵 *PRD FR-011*
- **REQ-413**: `GET /items/{id}/text` のチャンク `index` は形式によらない0起点の連番でなければならず、`boundaries` の導入によってこの規約を変更してはならない 🔵 *item-text.md・intrahub-mastra REQ-006*
- **REQ-414**: 同一の `(file_id, extraction_version, chunk_size)` に対して `index` と本文の対応は不変でなければならない 🔵 *item-text.md*

---

## 非機能要件

### パフォーマンス

- **NFR-001**: `GET /api/v1/items/{id}/text` は本文サイズによらず、要求チャンクのみをDB側で切り出して返し、レスポンスタイムが本文全長に比例しないこと 🔵 *item-text.md 実装上の注意*
- **NFR-002**: Extractor の同時実行数の初期値は1とする。スループットは重視しない 🔵 *tech-stack.md・PRD NFR-003*
- **NFR-003**: CPU実行時のOCR処理時間は実データで計測し、計測結果に基づいて `EXTRACTOR_JOB_TIMEOUT_SEC` の初期値を確定する。PRDに数値目標がないため、本要件では固定値を定めない 🔵 *tech-stack.md §GPU制約*
- **NFR-004**: claim のポーリングがAPIおよびDBへ過度な負荷をかけないこと。`EXTRACTOR_POLL_INTERVAL_SEC` で調整可能とする 🟡 *tech-stack.md の環境変数定義から妥当な推測*

### セキュリティ

- **NFR-101**: 内部APIは `INTERNAL_API_KEY` による認証を必須とし、未設定・不一致は `401 UNAUTHORIZED` を返す 🔵 *PRD NFR-002・index.md*
- **NFR-102**: 対象ファイルの共有ボリュームは read-only でマウントする 🔵 *PRD NFR-002*
- **NFR-103**: symlink を含め、許可されたルート外のファイルを読み取らない。判定は resolve 後に行い、判定前にファイルを開かない 🔵 *PRD NFR-002・tech-stack.md*
- **NFR-104**: ログのマスキング処理は structlog の processor 1箇所に集約し、各所で書き分けない 🔵 *tech-stack.md*
- **NFR-105**: worker は公開HTTP APIを一切提供しない 🔵 *PRD §4.2*
- **NFR-106**: worker の依存関係にDBドライバを含めないことで、「DBへ直接接続しない」を依存レベルで強制する 🔵 *tech-stack.md*

### 分離・可用性

- **NFR-201**: Extractor の停止、再起動、OCR失敗によって mediavault-api と mediavault-mcp が停止してはならない 🔵 *PRD NFR-001*
- **NFR-202**: worker が異常終了して lease を更新できない場合、期限経過後に抽出を再取得可能とする 🔵 *PRD §8.1*
- **NFR-203**: 抽出障害や高負荷が MediaVault の検索・更新を妨げてはならない 🔵 *PRD §2*

### GPU共存性

- **NFR-301**: MVPは `EXTRACTOR_OCR_DEVICE=cpu` を既定とし、vLLM と同居しても VRAM 競合で落ちない状態を保つ 🔵 *PRD NFR-003・tech-stack.md*
- **NFR-302**: `cuda` で運用する場合は、vLLM を停止するか `--gpu-memory-utilization` を下げ、実機で両サービスの起動と同時推論の成功を確認する。VRAM不足時の自動切り替えは行わない 🔵 *PRD NFR-003*
- **NFR-303**: GPU使用量、空きVRAM、OOM、処理時間を観測し、同居時の性能低下を評価可能にする 🔵 *PRD NFR-003*

### 可観測性

- **NFR-401**: 抽出ID、ファイルID、処理形式、ページ数、処理時間、終了状態、OCR実行方式、エンジン、モデル識別子を構造化ログへ記録する 🔵 *PRD NFR-004*
- **NFR-402**: 待機中の抽出数、待機時間、成功率、処理時間、lease切れ回数を観測可能にする 🔵 *PRD §8.7*
- **NFR-403**: ヘルスチェックで、プロセスの生存、MediaVault-api到達性、選択中のOCRバックエンド到達性を区別できるようにする 🔵 *PRD NFR-004*

### ユーザビリティ

- **NFR-501**: `TEXT_NOT_EXTRACTED` と `FILE_NOT_FOUND` を区別する。前者は「抽出を依頼すれば解決する」、後者は「そもそも材料が無い」ことを表す 🔵 *item-text.md*
- **NFR-502**: `AMBIGUOUS_FILE` のレスポンスには候補一覧を含め、クライアントが `file_id` を指定して再試行できるようにする 🔵 *item-text.md*
- **NFR-503**: 抽出のエラーは構造化し、AIエージェントが「復旧待ち」「再依頼で解決」「材料が無い」を区別できるようにする 🔵 *api-tool-mapping.md §エラー分類*

### 再現性

- **NFR-601**: 同一ファイル＋同一 `extraction_version` から同等の本文が再生成できること 🔵 *tech-stack.md 品質基準*
- **NFR-602**: 抽出コアロジックのテストカバレッジ 80%以上、Ruff エラーゼロ、mypy `--strict` を通すこと 🔵 *tech-stack.md 品質基準*

---

## API仕様サマリー

### 公開API（`/api/v1`）

| Method | Path | 説明 | 主なレスポンス |
|---|---|---|---|
| POST | `/items/{id}/files/{file_id}/extraction` | 抽出リクエスト（冪等） | 201（新規）/ 200（未完了の既存を返却） |
| GET | `/items/{id}/files/{file_id}/extraction` | 状態・進捗・エラー取得 | 200 `ApiOk<Extraction>` |
| POST | `/items/{id}/files/{file_id}/extraction/cancel` | キャンセル要求 | 200 `ApiOk<Extraction>` |
| GET | `/items/{id}/text` | 抽出済み全文のチャンク取得 | 200 `ApiOk<ItemText>` |

**廃止するエンドポイント（未実装のため実体なし）**: `GET /api/v1/jobs`、`GET /api/v1/jobs/{id}`、`POST /api/v1/jobs/{id}/cancel`、`POST /internal/jobs`

### 内部API（`/api/v1/internal`、`api_key_auth`）

| Method | Path | 説明 |
|---|---|---|
| POST | `/extractions/claim` | 排他claim + lease token 発行 + 検証済みファイル参照の返却 |
| POST | `/extractions/{id}/heartbeat` | lease延長・進捗更新・キャンセル要求の取得 |
| POST | `/extractions/{id}/complete` | 本文+境界情報+メタデータ保存 → `succeeded`（同一トランザクション） |
| POST | `/extractions/{id}/fail` | 構造化エラー保存 → `queued` 再投入 or `failed` |
| POST | `/extractions/{id}/cancelled` | worker のキャンセル確認 → `cancelled` |

既存の `/internal/*` 5本も `/api/v1/internal/*` へ移設する（REQ-029）。

### 状態遷移

```text
queued ──claim───────> running ──complete───> succeeded
   │                      ├──fail───────────> queued（attempts < max）/ failed
   │                      └──cancelled確認──> cancelled
   └──cancel要求────────────────────────────> cancelled

running ──cancel要求──> cancelling ──worker確認──> cancelled
```

`succeeded` / `failed` / `cancelled` は終端状態。

### 追加・削除するエラーコード

| コード | HTTP | 条件 | 操作 |
|---|---|---|---|
| `EXTRACTION_NOT_FOUND` | 404 | 指定ファイルに抽出が存在しない | 追加 |
| `EXTRACTION_ALREADY_FINISHED` | 409 | 終端状態の抽出をキャンセルしようとした | 追加 |
| `UNSUPPORTED_FILE_TYPE` | 422 | 抽出非対応の `file_type` | 追加 |
| `INVALID_LEASE_TOKEN` | 409 | lease token 不一致、または lease 期限切れ後の complete/fail | 追加 |
| `TEXT_NOT_EXTRACTED` | 422 | 既存定義のまま（**未実装**表記を解除） | 更新 |
| `AMBIGUOUS_FILE` | 409 | 既存定義のまま（**未実装**表記を解除） | 更新 |
| `JOB_NOT_FOUND` | 404 | — | **削除** |
| `JOB_ALREADY_FINISHED` | 409 | — | **削除** |

---

## Edgeケース

### エラー処理

- **EDGE-001**: 2台の worker が同時に claim を発行した場合、同一の抽出を両方が取得してはならない（`FOR UPDATE SKIP LOCKED` 等で担保）🔵 *PRD §8.8*
- **EDGE-002**: lease 期限切れ後に旧 worker から complete が届いた場合、システムは `INVALID_LEASE_TOKEN` で拒否し、再claimされた新しい試行の結果を優先しなければならない 🔵 *PRD §8.5・§8.8*
- **EDGE-003**: worker がキャンセル検知後に complete を送ってしまった場合、APIは `cancelling` 状態を根拠に受理を拒否しなければならない 🔵 *PRD §8.5*
- **EDGE-004**: `item_files.path` が指すファイルがボリューム上に存在しない場合、workerは再試行不可のエラーとして fail しなければならない 🟡 *PRD FR-009 から妥当な推測*
- **EDGE-005**: 拡張子とファイルシグネチャが不一致の場合、workerは明示的なエラーとして扱わなければならない 🔵 *tech-stack.md §形式判定*
- **EDGE-006**: 抽出中に対象 `item_file` が削除された場合、システムは抽出を終端状態へ整合的に遷移させなければならない 🟡 *PRD §8.7 から妥当な推測*
- **EDGE-007**: OCR が全ページで失敗した場合、部分結果を成功として確定してはならない 🟡 *PRD FR-009 から妥当な推測*
- **EDGE-008**: heartbeat が API 到達不能で連続失敗した場合、workerは処理を中断し、lease 失効による再claimに委ねなければならない 🟡 *tech-stack.md 実行ループから妥当な推測*
- **EDGE-009**: 抽出本文が保存サイズ上限を超えた場合、システムは complete を拒否し、再試行不可のエラーとして記録しなければならない 🟡 *PRD §8.7 から妥当な推測*
- **EDGE-010**: 未完了の抽出が存在する状態で cancel が二重に要求された場合、システムは冪等に扱い、既に `cancelling` なら現状を返さなければならない 🟡 *状態遷移図から妥当な推測*

### 境界値

- **EDGE-101**: `chunk_index` が `total_chunks` 以上の場合、`400 VALIDATION_ERROR` を返す 🔵 *item-text.md*
- **EDGE-102**: `chunk_size` の既定は `4000`、最大は `20000`。範囲外は `400 VALIDATION_ERROR` 🔵 *item-text.md*
- **EDGE-103**: `total_chunks` は `ceil(char_length(content) / chunk_size)` で算出し、バイト長ではなく**文字数**で数える 🔵 *item-text.md*
- **EDGE-104**: 末尾チャンクは `chunk_size` 未満になりうる 🔵 *item-text.md*
- **EDGE-105**: 抽出結果が空文字列（テキストが1文字も取れなかった）の場合、`TEXT_NOT_EXTRACTED`（未抽出）とは区別し、`total_chunks: 0` の抽出済みとして扱う 🟡 *item-text.md「テキストが空ではなく、まだ抽出していないことを表す」の記述から妥当な推測*
- **EDGE-106**: `EXTRACTOR_MAX_FILE_BYTES` / `EXTRACTOR_MAX_PAGES` を超えるファイルは、開く前に検証して再試行不可のエラーとする 🔵 *tech-stack.md §セキュリティ*
- **EDGE-107**: `boundaries` の各要素は `start <= end` かつ本文長を超えないこと。ページ境界が本文長と一致しない場合は保存を拒否する 🟡 *REQ-042 の整合性要求から妥当な推測*

---

## 信頼性レベル分布（requirements.md）

要件項目（REQ / NFR / EDGE）125件の内訳。

| レベル | 件数 | 割合 |
|---|---|---|
| 🔵 青信号 | 110 | 88% |
| 🟡 黄信号 | 15 | 12% |
| 🔴 赤信号 | 0 | 0% |

**品質評価**: ✅ 高品質（要件の曖昧さなし / 入出力定義ほぼ完全 / 制約条件明確 / 実装可能性確実）

🟡 が残る主な領域は、PRD に明記がなく妥当な推測で補った再試行判定の細部・エッジケース・MVP対象形式の限定であり、いずれも設計フェーズで確定可能な粒度である。

---

## トレーサビリティ

| PRD 出典 | 対応要件 |
|---|---|
| FR-001 ジョブ取得 | REQ-020, REQ-021, REQ-060, EDGE-001 |
| FR-002 対象ファイル検証 | REQ-004, REQ-061, REQ-402 |
| FR-003 形式判定 | REQ-062, REQ-108, REQ-410, EDGE-005 |
| FR-004 テキスト抽出 | REQ-105, REQ-106, REQ-107 |
| FR-005 正規化 | REQ-063, REQ-064 |
| FR-006 出典位置 | REQ-042, REQ-068, REQ-304, REQ-413 |
| FR-007 結果報告 | REQ-043, REQ-065 |
| FR-008 進捗とキャンセル | REQ-023, REQ-066, REQ-204, REQ-207 |
| FR-009 再試行 | REQ-026, REQ-109〜112, EDGE-004, EDGE-007 |
| FR-010 再抽出 | REQ-103, REQ-104 |
| FR-011 OCR実行方式 | REQ-069, REQ-113, REQ-114, REQ-411, REQ-412 |
| NFR-001 分離 | NFR-201, NFR-203 |
| NFR-002 セキュリティ | REQ-028, REQ-402〜405, NFR-101〜106 |
| NFR-003 GPU共存性 | NFR-301〜303, REQ-302 |
| NFR-004 可観測性 | REQ-071, REQ-303, NFR-401, NFR-403 |
| §8.1 ジョブデータモデル | REQ-040, REQ-044, REQ-118, NFR-202 |
| §8.2 抽出結果データモデル | REQ-041〜043, REQ-025, REQ-206 |
| §8.3 ジョブ投入・参照API | REQ-001〜004 |
| §8.4 worker内部API | REQ-020〜028, REQ-406 |
| §8.5 状態遷移と競合制御 | REQ-201〜207, REQ-407, EDGE-002, EDGE-003 |
| §8.6 Item Text API | REQ-005〜008, REQ-115〜117 |
| §8.7 API側の安全性と運用 | REQ-408, REQ-409, NFR-402, EDGE-009 |
| §8.8 api受け入れ条件 | [acceptance-criteria.md](acceptance-criteria.md) §PRD受け入れ条件マッピング |
| §8.9 OCRデバイス受け入れ条件 | 同上 |
| §11-1〜5 要調整事項 | D-2, D-3, D-4, D-5, D-6 |
| §12 技術選定 | REQ-069 |
