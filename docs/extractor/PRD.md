# MediaVault Extractor PRD（草案）

> ステータス: 草案  
> 作成日: 2026-08-14  
> 更新日: 2026-08-14  
> 本書は要求と責務境界を定める。OCRライブラリ、内部API、DBスキーマの詳細は後続設計で決定する。

## 1. 概要

MediaVault Extractorは、MediaVaultに登録されたファイルからテキストを非同期に抽出するPython製workerである。

PDF、EPUB、画像などの形式差とOCR処理を吸収し、抽出結果をMediaVault-apiへ返す。自身ではユーザー向けAPI、MCPツール、要約、embedding、Knowledge Noteを提供しない。

```text
intrahub-mastra
       │ MCP
       ▼
mediavault-mcp
       │ REST
       ▼
mediavault-api ── jobs / 抽出結果
       ▲
       │ 内部API
       ▼
mediavault-extractor（Python worker）
       └─ yomitoku OCR（CPU / CUDA GPU）
```

関連文書:

- [Extraction API](../backend/mediavault-api/extraction.md)
- [Item Text API](../mediavault-api/item-text.md)
- [MediaVault-mcpとMastraの連携設計](../mediavault-mcp/design/mastra-integration.md)

## 2. 背景と課題

- OCR・PDF・画像処理のPython依存をRust製APIへ組み込むと、イメージサイズと保守負担が増える。
- OCRはCPU・メモリ消費と処理時間が大きく、同期HTTP処理に適さない。
- GPUを必要とするOCRモデルをExtractorと同居させると、GPU割り当て、VRAM管理、モデル配布の責務がworkerへ混在する。
- `intrahub-mastra`の全文調査には、形式によらない安定したチャンク参照が必要である。
- API処理と抽出処理を分離し、抽出障害や高負荷がMediaVaultの検索・更新を妨げない構成が必要である。

## 3. 目的

- 登録済みファイルから検索・AI利用可能なプレーンテキストを抽出する。
- テキストレイヤーがある文書は通常抽出し、必要なページだけOCRへフォールバックする。
- 長時間処理をジョブとして実行し、進捗、失敗、再試行、キャンセルを扱えるようにする。
- 抽出エンジンの変更がMediaVault-api、MCP、Mastraへ波及しない境界を作る。
- 抽出元ファイルと抽出結果の対応、および再抽出による版の変化を追跡可能にする。

## 4. 対象範囲

### 4.1 MVP

- Python製の常駐ポーリングworker
- PDFの埋め込みテキスト抽出
- PDF内画像および画像ファイルのOCR
- テキストの正規化
- 進捗・結果・エラーのMediaVault-apiへの報告
- キャンセル要求の確認
- 抽出処理バージョンの記録

### 4.2 対象外

- 要約、翻訳、タグ推定、固有表現抽出
- embedding生成とベクトル検索
- Knowledge Vaultへの書き込み
- 動画・音声の文字起こし
- UIおよび一般公開HTTP API
- MediaVaultデータベースの直接操作
- 任意のホストパスを指定した抽出

## 5. 責務境界

### 5.1 MediaVault Extractor

- MediaVault-apiから抽出ジョブを取得する。
- APIが許可したファイルだけを読み取る。
- ファイル形式を判定し、適切な抽出器を選ぶ。
- 通常抽出とOCRを実行する。
- 抽出本文、区切り情報、処理バージョン、OCR実行方式、進捗、エラーをAPIへ返す。
- DBへ直接接続しない。

### 5.2 MediaVault-api

- ジョブと状態遷移の正本を所有する。
- `item_file_id`、対象Item、実ファイルの対応を検証する。
- workerへ安全なファイル参照を提供する。
- 抽出結果を永続化し、`GET /api/v1/items/{id}/text`で公開する。
- 重複ジョブ、再試行回数、キャンセルを管理する。

### 5.3 MediaVault-mcp / intrahub-mastra

- MCPはジョブ投入・状態確認・全文取得を目的単位のツールとして提供する。
- Mastraは抽出済みチャンクを取得して調査・執筆へ利用する。
- Extractorを直接呼び出さない。

## 6. 機能要求

### FR-001: ジョブ取得

workerは内部APIをポーリングし、未取得の`extract_text`ジョブを排他的にclaimできなければならない。同じジョブを複数workerが同時実行してはならない。

### FR-002: 対象ファイルの検証

workerが処理対象として受け取る識別子は原則`item_file_id`とする。外部から渡された任意の絶対パス・相対パスをそのまま開いてはならない。

### FR-003: 形式判定

拡張子だけでなくMIME typeまたはファイルシグネチャを併用し、対応する抽出器を選ぶ必要がある。未対応形式は明確なエラーとして終了する。

### FR-004: テキスト抽出

- PDFに有効なテキストレイヤーがある場合は、それを優先する。
- テキストが存在しない、または品質基準を満たさないページだけOCRを実行する。
- 画像ファイルはOCR対象とする。

### FR-005: 正規化

抽出結果に対して、文字コード、改行、連続空白、制御文字を正規化する。ただし原文の意味を変える校正やLLMによる書き換えは行わない。

### FR-006: 出典位置

可能な場合、文字範囲とPDFページまたはEPUB章の対応を保持する。形式固有の位置は表示用`label`に使用し、MCPへ公開するチャンク識別子は0起点の連番とする。

### FR-007: 結果報告

成功時は少なくとも次をMediaVault-apiへ報告しなければならない。

- `item_file_id`
- 抽出本文
- 形式固有の区切り情報
- `extraction_version`
- 抽出日時
- 使用した抽出方式（embedded text / OCR）
- OCRを使用した場合の実行方式（`cpu` / `gpu`）、エンジン、モデル識別子

### FR-008: 進捗とキャンセル

ページまたは章などの安全な区切りで進捗とキャンセル要求を確認する。キャンセル後に成功結果を確定してはならない。

### FR-009: 再試行

一時的なAPI通信失敗は再試行可能とし、破損ファイル、未対応形式、恒久的なOCR失敗は無限再試行しない。最終的な再試行判断と上限はMediaVault-apiのジョブ管理に従う。

### FR-010: 再抽出

同じファイルを再抽出できなければならない。抽出ロジックまたはチャンク境界が変わる場合は`extraction_version`を変更し、保存済み出典参照が古くなったことを検出可能にする。

### FR-011: OCR実行方式の切り替え

- OCRエンジンはMVPではyomitokuを使用する。
- OCR実行デバイスは環境変数`EXTRACTOR_OCR_DEVICE`で`cpu`または`cuda`を選択できなければならない。外部へ報告する実行方式はそれぞれ`cpu`、`gpu`とする。
- 既定値は`cpu`とする。CPU実行ではyomitokuの軽量モデルを使用し、通常モデルとの差異と精度・速度を実データで評価する。
- 実行方式はworker起動時に確定し、処理中のジョブに対して動的に変更しない。変更は設定更新後のworker再起動で反映する。
- 未知の実行方式、または`cuda`指定時にCUDA GPUを利用できない場合は起動時エラーとし、ジョブ取得を開始してはならない。yomitokuによる暗黙のCPUフォールバックに依存しない。
- 実行方式による差異をOCRエンジン境界の内側へ閉じ込め、PDF・画像抽出、正規化、結果報告の処理は共通化する。

## 7. 非機能要求

### NFR-001: 分離

Extractorの停止、再起動、OCR失敗によってMediaVault-apiとmediavault-mcpが停止してはならない。

### NFR-002: セキュリティ

- 内部APIは`INTERNAL_API_KEY`等で認証する。
- 対象ファイルの共有ボリュームは原則read-onlyでmountする。
- ログへAPIキー、本文全体、個人情報を不用意に出力しない。
- シンボリックリンクを含め、許可されたルート外のファイルを読まない。

### NFR-003: GPU共存性

- vLLMとExtractorへのGPUデバイス割り当ては排他的ロックではないため、同じGPUを両コンテナから参照できる。ただし、同時実行には両プロセスのモデル、CUDAコンテキスト、KV cache等が収まる空きVRAMが必要である。
- 現行vLLM設定は`--gpu-memory-utilization 0.90`であるため、ExtractorはCPU実行を標準運用とし、GPU同時利用可能とはみなさない。
- Extractorを`cuda`で運用する場合は、vLLMを停止するか、vLLMのGPUメモリ使用上限を下げ、実機で両サービスの起動と同時推論が成功することを確認する。VRAM不足時の自動切り替えは行わない。
- GPU使用量、空きVRAM、OOM、処理時間を観測し、同居時の性能低下を評価可能にする。

### NFR-004: 可観測性

ジョブID、ファイルID、処理形式、ページ数、処理時間、終了状態、OCR実行方式、エンジン、モデル識別子を構造化ログへ記録する。ヘルスチェックでプロセスの生存、MediaVault-api到達性、選択中のOCRバックエンド到達性を区別できるようにする。APIキー、画像、抽出本文はログへ出力しない。

## 8. MediaVault-api拡張要求

Extractorの導入には、worker本体に加えてMediaVault-apiの抽出管理・worker連携・抽出結果公開を拡張する必要がある。抽出と結果の正本はMediaVault-apiおよびPostgreSQLとし、Extractorは内部API以外から状態を変更しない。

### 8.1 抽出データモデル

MediaVault-apiの`item_file_extractions`テーブルに、ファイル単位の抽出状態を保持する。

| フィールド | 内容 |
|---|---|
| `id` | 抽出ID |
| `state` | `queued` / `running` / `succeeded` / `failed` / `cancelling` / `cancelled` |
| `item_file_id` | 抽出対象ファイル |
| `attempts` / `max_attempts` | 試行回数と上限 |
| `progress_current` / `progress_total` | 進捗 |
| `claimed_by` / `lease_expires_at` | workerの排他取得と回収に使用 |
| `error` | 構造化された失敗情報 |
| `created_at` / `updated_at` | 作成・更新日時 |

同じ`item_file_id`のactiveな抽出は、`state IN ('queued','running','cancelling')`を条件とする部分UNIQUE indexで1件に制限する。workerがleaseを更新できない場合、期限経過後に抽出を再取得可能にする。

### 8.2 抽出結果データモデル

MediaVault-apiに抽出本文を保持する`item_file_texts`相当のテーブルを追加する。

| フィールド | 内容 |
|---|---|
| `item_file_id` | `item_files`へのFK。同一ファイルの現行結果は1件 |
| `content` | 正規化済み全文 |
| `boundaries` | MVPから保存するページ・章等の文字範囲と表示ラベル。jsonb `[{start, end, label}]` |
| `extraction_version` | 抽出ロジックと境界の版 |
| `extractor` | 使用方式とエンジン情報 |
| `extracted_at` | 抽出完了日時 |

抽出成功と抽出結果の置換は、同一トランザクションで確定する。失敗・キャンセル時に既存の成功結果を削除してはならない。

### 8.3 抽出要求・参照API

ファイルを親に持つ [Extraction API](../backend/mediavault-api/extraction.md) を提供する。

- `POST /api/v1/items/{id}/files/{file_id}/extraction`: 冪等な抽出要求
- `GET /api/v1/items/{id}/files/{file_id}/extraction`: 最新状態・進捗の取得
- `POST /api/v1/items/{id}/files/{file_id}/extraction/cancel`: キャンセル要求

公開入力はパスパラメータの`item_file_id`を正本とする。APIは対象ファイルの存在、Itemとの対応、対応形式、読み取り可能性を検証してから抽出を作成する。

### 8.4 worker内部API

MediaVault-apiは、Extractor専用に認証された `/api/v1/internal/extractions/*` APIを提供する。

| 操作 | 要求 |
|---|---|
| claim | 次の実行可能な抽出を排他的に取得し、leaseを設定する |
| heartbeat | lease延長、進捗更新、キャンセル要求の取得を行う |
| file access | `item_file_id`から検証済みファイル参照またはストリームを取得する |
| complete | 抽出本文とメタデータを一括保存し、抽出を`succeeded`へ遷移する |
| fail | 構造化エラーを保存し、再試行または`failed`を決定する |
| cancel | workerが処理停止を確認し、`cancelled`へ遷移する |

内部APIには`INTERNAL_API_KEY`または同等のサービス認証を必須とする。一般公開APIからclaim、heartbeat、結果確定を実行できてはならない。

### 8.5 状態遷移と競合制御

MediaVault-apiは次の状態遷移を検証し、不正な逆行や終端状態の上書きを拒否する。

```text
queued ──claim──> running ──complete──> succeeded
   │                  ├────fail──────> queued / failed
   │                  └──cancel確認──> cancelled
   └──cancel────────> cancelled

running ──cancel要求──> cancelling ──worker確認──> cancelled
```

- `succeeded` / `failed` / `cancelled`は終端状態とする。
- complete/failにはclaim時のlease token等を要求し、別workerや古い試行による上書きを防ぐ。
- APIはキャンセル済み抽出の結果を成功として確定してはならない。

### 8.6 Item Text API

[Item Text API](../mediavault-api/item-text.md)に従い、`GET /api/v1/items/{id}/text`を実装する。

- `file_id`を明示して対象ファイルを選択できる。
- `chunk_index`は形式によらない0起点の連番とする。
- `chunk_size`、`total_chunks`、`label`、`extraction_version`を返す。
- `FILE_NOT_FOUND`、`TEXT_NOT_EXTRACTED`、`AMBIGUOUS_FILE`を区別する。
- 1レスポンスに全文を含めず、要求されたチャンクだけを返す。
- 巨大な本文をAPIプロセスのメモリへ全件ロードせずに切り出せるようにする。

### 8.7 API側の安全性と運用

- ファイル実体の解決時に許可ルート内であることを検証する。
- 内部APIの認証キーとファイル本文をログへ出力しない。
- 抽出本文と抽出エラーには保存サイズ上限を設ける。
- 抽出数、待機時間、成功率、処理時間、lease切れ回数を観測可能にする。
- Itemまたはファイル削除時の抽出・抽出結果の扱いをFKと業務処理で一貫させる。

### 8.8 MediaVault-apiの受け入れ条件

- [ ] 同じ対象への未完了抽出が冪等化される。
- [ ] 2台のworkerが同じ抽出を同時にclaimできない。
- [ ] lease切れ抽出を安全に再取得できる。
- [ ] 古いworkerからのcompleteが拒否される。
- [ ] キャンセル要求がworkerへ伝わり、終端状態へ遷移する。
- [ ] 抽出結果保存と抽出成功が不整合にならない。
- [ ] Item Text APIがチャンクと`extraction_version`を返す。
- [ ] 未抽出、ファイル不在、複数候補を異なるエラーとして返す。
- [ ] 内部APIを無認証で呼び出せない。

### 8.9 ExtractorのOCRデバイス受け入れ条件

- [ ] 設定省略時にyomitokuがCPUで実行される。
- [ ] `EXTRACTOR_OCR_DEVICE=cpu`と`cuda`の各設定で、結果メタデータへそれぞれ`cpu`と`gpu`が記録される。
- [ ] `cuda`指定時にGPUを利用できなければ、ジョブをclaimする前に起動エラーとなる。
- [ ] CPU/GPUの切り替えにworker再起動が必要であり、実行中ジョブのデバイスが途中で変わらない。
- [ ] vLLM稼働中にGPUモードを許可する場合、VRAM上限を調整した構成で両サービスの起動と同時推論を確認できる。


## 11. 決定事項

1. 公開入力はパスパラメータの`item_file_id`だけとし、ホストパスはAPI側で解決する。✅
2. 内部ルートは`/api/v1/internal/*`へ統一し、旧`/internal/*`は提供しない。✅
3. workerはread-only共有ボリュームからファイルを読む。✅
4. 抽出結果はcomplete時に一括送信する。✅
5. 境界情報はjsonb `[{start, end, label}]`として保存する。✅

## 12. 技術選定
1. MVPのOCRライブラリには[yomitoku](https://github.com/kotaro-kinoshita/yomitoku)を使用する。
2. [ndlocr-lite](https://github.com/ndl-lab/ndlocr-lite)は将来の差し替え候補とし、OCRエンジン境界の外へyomitoku固有型を公開しない。
