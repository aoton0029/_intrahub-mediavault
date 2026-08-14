# MediaVault Extractor タスク概要

**作成日**: 2026-08-14
**プロジェクト期間**: Day 1 - Day 27（27営業日）
**推定工数**: 216時間
**総タスク数**: 27件

## 関連文書

- **要件定義書**: [📋 requirements.md](../spec/requirements.md)
- **ユーザストーリー**: [📖 user-stories.md](../spec/user-stories.md)
- **受け入れ基準**: [✅ acceptance-criteria.md](../spec/acceptance-criteria.md)
- **準備タスク**: [🔧 prep.md](../spec/prep.md)
- **コンテキストノート**: [📝 note.md](../spec/note.md)
- **アーキテクチャ**: [📐 architecture.md](../design/architecture.md)
- **データフロー図**: [🔄 dataflow.md](../design/dataflow.md)
- **DBスキーマ**: [🗄️ database-schema.sql](../design/database-schema.sql)
- **API仕様**: [🔌 api-endpoints.md](../design/api-endpoints.md)
- **型定義（api）**: [📝 interfaces.rs](../design/interfaces.rs)
- **型定義（worker）**: [📝 interfaces.py](../design/interfaces.py)
- **技術スタック**: [🔧 tech-stack.md](../tech-stack.md)
- **PRD**: [📄 PRD.md](../PRD.md)

> **注1**: 対象は Rust API・Python worker・MCP サーバーであり、抽出機能のUIは要件のスコープ外である（[requirements.md](../spec/requirements.md) §対象範囲）。テンプレートの「フロントエンドタスク」「UI/UX要件（ローディング・モバイル・アクセシビリティ）」は該当せず、代わりに **AIエージェント向け要件**（エラーの識別可能性・ツール説明文の明確さ）を各タスクに設けている。
>
> **注2**: mediavault-mcp のツール実装（TASK-0024・TASK-0025）も本番号空間に含む。`docs/backend/mediavault-mcp/tasks/` の TASK-0001〜0026 とは**別の番号空間**である。
>
> **注3**: 設計の Phase 6（非機能検証）はタスク化していない。CPU OCR の処理時間計測のみ TASK-0023 に含め、残りは実装完了後の運用で対応する。

## フェーズ構成

| フェーズ | 期間 | 成果物 | タスク数 | 工数 | ファイル |
|---------|------|--------|----------|------|----------|
| Phase 1 | Day 1-6 | DBスキーマ・モデル・repository・内部APIパス統一 | 6 | 48h | [TASK-0001〜0006](#phase-1-基盤構築) |
| Phase 2 | Day 7-10 | 公開API 4本 | 4 | 32h | [TASK-0007〜0010](#phase-2-公開api) |
| Phase 3 | Day 11-15 | worker内部API 5本・競合制御 | 5 | 40h | [TASK-0011〜0015](#phase-3-worker内部api) |
| Phase 4 | Day 16-23 | Python worker 一式 | 8 | 64h | [TASK-0016〜0023](#phase-4-python-worker) |
| Phase 5 | Day 24-27 | MCPツール・ドキュメント改訂 | 4 | 32h | [TASK-0024〜0027](#phase-5-mcpドキュメント) |

各フェーズは180時間以内（最大64h）に収まっている。

## タスク番号管理

**使用済みタスク番号**: TASK-0001 〜 TASK-0027
**次回開始番号**: TASK-0028（EPUB対応・動画音声の文字起こし等、将来拡張用）

## 全体進捗

- [ ] Phase 1: 基盤構築
- [ ] Phase 2: 公開API
- [ ] Phase 3: worker内部API
- [ ] Phase 4: Python worker
- [ ] Phase 5: MCP・ドキュメント

## マイルストーン

| # | マイルストーン | 時期 | 達成条件 |
|---|---|---|---|
| M1 | DBとパス規約が固まる | Day 6 | 2テーブルが適用され、内部APIが `/api/v1/internal/*` へ統一される |
| M2 | **公開APIが動く** | Day 10 | 手動INSERTした抽出結果を `GET /items/{id}/text` で読める。抽出リクエストが冪等に動く |
| M3 | **api 側が完成する** | Day 15 | PRD §8.8 の受け入れ条件**全9項目**が検証済み。worker なしで api の正しさが保証される |
| M4 | **抽出が実際に通る** | Day 23 | 実PDFの抽出がエンドツーエンドで動く。CPU OCR の処理時間が実測され、暫定値だった定数が確定する |
| M5 | 要件完了 | Day 27 | AIエージェントが抽出を自己完結で依頼・監視・読解できる。jobs 参照がリポジトリから消える |

**M3 が設計上の要点** 🔵: Phase 3 完了時点で、Python worker が1行も書かれていなくても api 側の受け入れ条件を全て検証できる。これは擬似 worker（`FakeWorker`）による検証を [TASK-0015](TASK-0015.md) に置いたためであり、api と worker の開発リスクを分離している。

---

## Phase 1: 基盤構築

**期間**: Day 1-6（48h）
**目標**: DBスキーマ・型・データアクセス層を用意し、内部APIのパス規約を統一する
**成果物**: migration 1本、models 2ファイル、repositories 2ファイル、services 1ファイル、ルーター改修

### タスク一覧

- [ ] [TASK-0001: 抽出テーブルのmigration追加](TASK-0001.md) - 8h (DIRECT) 🔵
- [ ] [TASK-0002: 抽出モデル・エラーコード・label合成の実装](TASK-0002.md) - 8h (TDD) 🔵
- [ ] [TASK-0003: item_extraction_repository の実装](TASK-0003.md) - 8h (TDD) 🔵
- [ ] [TASK-0004: item_file_text_repository の実装](TASK-0004.md) - 8h (TDD) 🔵
- [ ] [TASK-0005: ファイル参照解決サービス](TASK-0005.md) - 8h (TDD) 🟡
- [ ] [TASK-0006: 内部APIパスの `/api/v1/internal/*` への移設](TASK-0006.md) - 8h (TDD) 🔵

### 依存関係

```
TASK-0001 ──┬─> TASK-0002 ──┬─> TASK-0003
            │               └─> TASK-0004
            └─> TASK-0003
TASK-0002 ────> TASK-0005
TASK-0006（独立。TASK-0001〜0005 と並行可能）
```

**並行実行の余地**: TASK-0006 は他と依存関係がない。TASK-0003 と TASK-0004 も互いに独立。

**着手前の注意** ⚠️: [TASK-0005](TASK-0005.md) は [prep.md](../spec/prep.md) §必須「`item_files.path` の実データ分布の確認」を先に済ませる必要がある。実データの分布が設計を左右する。

---

## Phase 2: 公開API

**期間**: Day 7-10（32h）
**目標**: 抽出リソースの公開APIと全文取得APIを完成させる
**成果物**: handlers 2ファイル、統合テスト

### タスク一覧

- [ ] [TASK-0007: POST .../extraction（冪等な抽出リクエスト）](TASK-0007.md) - 8h (TDD) 🔵
- [ ] [TASK-0008: GET .../extraction と cancel の実装](TASK-0008.md) - 8h (TDD) 🔵
- [ ] [TASK-0009: GET /items/{id}/text（全文チャンク取得）](TASK-0009.md) - 8h (TDD) 🔵
- [ ] [TASK-0010: 公開APIの統合テストとエラーコード整合](TASK-0010.md) - 8h (TDD) 🔵

### 依存関係

```
TASK-0003, TASK-0005 ──> TASK-0007 ──> TASK-0008 ──┐
TASK-0004 ────────────> TASK-0009 ─────────────────┴─> TASK-0010
```

**Phase 2 完了時点でできること**: 手動でDBへ INSERT した抽出結果を、公開API経由で読める。worker がなくても Item Text API の正しさを検証できる。

---

## Phase 3: worker内部API

**期間**: Day 11-15（40h）
**目標**: worker 連携の内部APIを完成させ、競合制御を実証する
**成果物**: handlers 1ファイル、repository 拡張、競合制御テスト

### タスク一覧

- [ ] [TASK-0011: 内部API claim（排他取得とlease発行）](TASK-0011.md) - 8h (TDD) 🟡
- [ ] [TASK-0012: 内部API heartbeat](TASK-0012.md) - 8h (TDD) 🔵
- [ ] [TASK-0013: 内部API complete（同一トランザクションでの結果確定）](TASK-0013.md) - 8h (TDD) 🔵
- [ ] [TASK-0014: 内部API fail / cancelled](TASK-0014.md) - 8h (TDD) 🟡
- [ ] [TASK-0015: 競合制御の統合テスト（PRD §8.8 検証）](TASK-0015.md) - 8h (TDD) 🔵

### 依存関係

```
TASK-0003, TASK-0005, TASK-0006 ──> TASK-0011 ──┬─> TASK-0012 ──┐
TASK-0004 ──────────────────────────────────────┼─> TASK-0013 ──┼─> TASK-0015
                                                └─> TASK-0014 ──┘
```

**M3（api 側の完成）**: [TASK-0015](TASK-0015.md) 完了時点で PRD §8.8 の9項目がすべて検証済みになる。

---

## Phase 4: Python worker

**期間**: Day 16-23（64h）
**目標**: 抽出処理を実装し、エンドツーエンドで動かす
**成果物**: `extractor/` 一式（Python worker）

### タスク一覧

- [ ] [TASK-0016: extractor プロジェクト初期化と設定・ログ基盤](TASK-0016.md) - 8h (DIRECT) 🔵
- [ ] [TASK-0017: 内部APIクライアント](TASK-0017.md) - 8h (TDD) 🔵
- [ ] [TASK-0018: パス解決の安全性と形式判定](TASK-0018.md) - 8h (TDD) 🔵
- [ ] [TASK-0019: OCRエンジン境界と yomitoku 実装](TASK-0019.md) - 8h (TDD) 🔵
- [ ] [TASK-0020: PDF抽出器（テキストレイヤー優先とOCRフォールバック）](TASK-0020.md) - 8h (TDD) 🟡
- [ ] [TASK-0021: 画像抽出器・正規化・境界構築](TASK-0021.md) - 8h (TDD) 🟡
- [ ] [TASK-0022: 常駐ループと heartbeat スレッド](TASK-0022.md) - 8h (TDD) 🟡
- [ ] [TASK-0023: worker のエンドツーエンド結合テスト](TASK-0023.md) - 8h (TDD) 🔵

### 依存関係

```
TASK-0015 ──> TASK-0016 ──┬─> TASK-0017 ──┐
                          ├─> TASK-0018 ──┤
                          └─> TASK-0019 ──┼─> TASK-0020 ──┐
                                          ├─> TASK-0021 ──┼─> TASK-0022 ──> TASK-0023
                                          └───────────────┘
```

**並行実行の余地**: TASK-0017 / TASK-0018 / TASK-0019 は互いに独立。TASK-0020 と TASK-0021 も並行可能（ただし TASK-0021 の `normalize` / `build_boundaries` を TASK-0020 が使うため、TASK-0021 を先に終えるほうがスムーズ）。

**着手前の注意** ⚠️: [prep.md](../spec/prep.md) §必須「抽出対象となるファイルの実データサンプルの用意」を先に済ませる。[TASK-0020](TASK-0020.md) の OCRフォールバック閾値と [TASK-0023](TASK-0023.md) の処理時間計測に必要。

---

## Phase 5: MCP・ドキュメント

**期間**: Day 24-27（32h）
**目標**: AIエージェントから利用可能にし、ドキュメントの整合を取る
**成果物**: MCPツール4本、api/mcp ドキュメント改訂

### タスク一覧

- [ ] [TASK-0024: MCP ツール get_item_text の実装](TASK-0024.md) - 8h (TDD) 🔵
- [ ] [TASK-0025: MCP 抽出系3ツールの実装](TASK-0025.md) - 8h (TDD) 🔵
- [ ] [TASK-0026: mediavault-api 側ドキュメントの改訂](TASK-0026.md) - 8h (DIRECT) 🔵
- [ ] [TASK-0027: mediavault-mcp 側ドキュメントの改訂と最終検証](TASK-0027.md) - 8h (DIRECT) 🔵

### 依存関係

```
TASK-0023 ──> TASK-0024 ──> TASK-0025 ──┐
         └──> TASK-0026 ────────────────┴─> TASK-0027
```

---

## クリティカルパス

```
TASK-0001 → TASK-0002 → TASK-0003 → TASK-0007 → TASK-0008 → TASK-0010
  → TASK-0011 → TASK-0013 → TASK-0015 → TASK-0016 → TASK-0019
  → TASK-0020 → TASK-0022 → TASK-0023 → TASK-0024 → TASK-0025 → TASK-0027
```

**クリティカルパス工数**: 136時間（17タスク）
**並行作業可能工数**: 80時間（10タスク）

単独開発のため実質は逐次実行になるが、並行可能なタスクは着手順を入れ替えられる（例: [TASK-0006](TASK-0006.md) を最初に片付けてもよい）。

---

## 信頼性レベルサマリー

### 全タスク統計

**総タスク数**: 27件
**総項目数**: 447項目

| レベル | 件数 | 割合 |
|---|---|---|
| 🔵 青信号 | 350項目 | 78% |
| 🟡 黄信号 | 97項目 | 22% |
| 🔴 赤信号 | 0項目 | 0% |

### フェーズ別信頼性

| フェーズ | 🔵 青 | 🟡 黄 | 🔴 赤 | 合計 | 青率 |
|---------|-------|-------|-------|------|------|
| Phase 1 | 58 | 13 | 0 | 71 | 81% |
| Phase 2 | 57 | 10 | 0 | 67 | 85% |
| Phase 3 | 54 | 27 | 0 | 81 | 66% |
| Phase 4 | 121 | 39 | 0 | 160 | 75% |
| Phase 5 | 60 | 8 | 0 | 68 | 88% |

**品質評価**: ✅ 高品質（タスク粒度: 適切 / 依存関係: 完全に定義 / 実装可能性: 確実 / 🔴 ゼロ）

### 🟡 が多いタスク（要注意）

| タスク | 青率 | 主な理由 |
|---|---|---|
| [TASK-0011](TASK-0011.md) claim | 50% | sweeper の実行タイミング・取得なしのレスポンス形式など、要件が定めていない運用挙動を設計で補った |
| [TASK-0014](TASK-0014.md) fail/cancelled | 61% | PRD §8.5 の状態遷移図が `fail → queued/failed` の分岐条件を明記していない |
| [TASK-0021](TASK-0021.md) 正規化 | 64% | PRD FR-005 が正規化の項目のみ挙げ、強度を定めていない |
| [TASK-0022](TASK-0022.md) 常駐ループ | 65% | 常駐プロセスの運用挙動（停止・スレッド管理）を設計で補った |
| [TASK-0005](TASK-0005.md) file_ref | 67% | 実データの分布が未確認 |
| [TASK-0020](TASK-0020.md) PDF抽出 | 68% | 「品質基準を満たさないページ」の閾値が実測待ち |

いずれも**中核の要件は 🔵** であり、🟡 は周辺の具体化（既定値・運用挙動・テスト手法）に集中している。実装時に挙動を変えたくなった場合は、各タスクのテストを更新すれば足りる。

---

## タスクタイプ別内訳

| タイプ | 件数 | 工数 |
|---|---|---|
| TDD | 23件 | 184h |
| DIRECT | 4件 | 32h |

DIRECT は TASK-0001（migration）、TASK-0016（プロジェクト初期化）、TASK-0026・TASK-0027（ドキュメント改訂）。

---

## 着手前に必要な準備

[prep.md](../spec/prep.md) の**必須4項目**を先に済ませること。

| 準備 | ブロックするタスク |
|---|---|
| `INTERNAL_API_KEY` の払い出し | [TASK-0016](TASK-0016.md) |
| 共有ボリュームの read-only マウント構成の確定 | [TASK-0005](TASK-0005.md), [TASK-0016](TASK-0016.md) |
| `item_files.path` の実データ分布の確認 | [TASK-0005](TASK-0005.md) |
| 抽出対象の実データサンプルの用意 | [TASK-0020](TASK-0020.md), [TASK-0023](TASK-0023.md) |

---

## 実装中に確定する暫定値

設計・タスクで暫定値を入れてある定数。[TASK-0023](TASK-0023.md) の実測で確定する。

| 定数 | 暫定値 | 確定タスク |
|---|---|---|
| `max_attempts` | 3 | 運用開始後 |
| `EXTRACTOR_LEASE_SECONDS` | 300 | [TASK-0023](TASK-0023.md) 計測1 |
| `EXTRACTOR_HEARTBEAT_INTERVAL_SEC` | 30 | 同上 |
| `EXTRACTOR_JOB_TIMEOUT_SEC` | 3600 | 同上 |
| `EXTRACTOR_OCR_FALLBACK_MIN_CHARS_PER_PAGE` | 50 | [TASK-0023](TASK-0023.md) 計測3 |
| `MAX_CONTENT_CHARS` | 500万 | 蔵書の分布確認後 |
| `EXTRACTOR_MAX_FILE_BYTES` / `MAX_PAGES` | 500MB / 2000 | 同上 |

いずれも**構造には影響しない**。値の変更で対応できる。

---

## 本要件のスコープ外

| 項目 | 理由 |
|---|---|
| CI（GitHub Actions での ruff / mypy / pytest） | ヒアリングで「含めない」と決定 |
| 非機能検証（vLLM との VRAM 共存確認） | 運用判断待ち（[prep.md](../spec/prep.md) §確認事項） |
| EPUB 対応 | PRD §4.2 対象外。boundaries の構造は拡張可能にしてある（REQ-304） |
| 動画・音声の文字起こし | PRD §4.2 対象外 |
| 抽出機能のUI | [requirements.md](../spec/requirements.md) §対象範囲 |
| 要約・embedding・Knowledge Vault 書き込み | PRD §4.2 対象外 |

---

## 次のステップ

タスクを実装するには:
- 全タスクを順番に実装: `/tsumiki:kairo-implement`
- 特定タスクを実装: `/tsumiki:kairo-implement TASK-0001`
