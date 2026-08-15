# mediavault-mcp タスク概要

**作成日**: 2026-08-07
**プロジェクト期間**: Day 1 - Day 26（26営業日）
**推定工数**: 208時間
**総タスク数**: 26件

## 関連文書

- **要件定義書**: [📋 requirements.md](../spec/requirements.md)
- **ユーザストーリー**: [📖 user-stories.md](../spec/user-stories.md)
- **受け入れ基準**: [✅ acceptance-criteria.md](../spec/acceptance-criteria.md)
- **準備タスク**: [🔧 prep.md](../spec/prep.md)
- **アーキテクチャ**: [📐 architecture.md](../design/architecture.md)
- **データフロー図**: [🔄 dataflow.md](../design/dataflow.md)
- **型定義**: [📝 interfaces.rs](../design/interfaces.rs)
- **MCPツール仕様**: [🔌 mcp-tools.md](../design/mcp-tools.md)
- **技術スタック**: [🔧 tech-stack.md](../tech-stack.md)
- **PRD**: [📄 PRD.md](../PRD.md)
- **コンテキストノート**: [📝 note.md](../spec/note.md)

> **注**: 対象は MCP サーバーであり UI を持たない。テンプレートの「フロントエンドタスク」「UI/UX要件（ローディング・モバイル・アクセシビリティ）」は該当せず、代わりに **AIエージェント向け要件**（ツール説明文・スキーマの可読性、NFR-201 / NFR-202）を各タスクに設けている。
>
> DBスキーマ設計タスクも存在しない。MCP は PostgreSQL へ直接アクセスしない（REQ-140 / NFR-303）。

## フェーズ構成

| フェーズ | 期間 | 成果物 | タスク数 | 工数 | ファイル |
|---------|------|--------|----------|------|----------|
| Phase 0 | Day 1-4 | MediaVault-api の先行改修 | 4 | 32h | [TASK-0001~0004](#phase-0-mediavault-api-先行改修) |
| Phase 1 | Day 5-11 | MCP基盤・認証・ApiClient・health | 7 | 56h | [TASK-0005~0011](#phase-1-mcp基盤構築) |
| Phase 2 | Day 12-16 | 読み取りツール5件 | 5 | 40h | [TASK-0012~0016](#phase-2-読み取りツール) |
| Phase 3 | Day 17-22 | 書き込みツール6件 | 6 | 48h | [TASK-0017~0022](#phase-3-書き込みツール) |
| Phase 4 | Day 23-26 | 横断検証・CI・ドキュメント | 4 | 32h | [TASK-0023~0026](#phase-4-横断検証仕上げ) |

各フェーズは180時間以内（最大56h）に収まっている。

## タスク番号管理

**使用済みタスク番号**: TASK-0001 ~ TASK-0026
**次回開始番号**: TASK-0027（第2段階: stdio 用）

抽出関連のタスクは `docs/extractor/tasks/` の別番号空間で管理する。同ディレクトリの TASK-0024 / TASK-0025 で `get_item_text` と抽出系3ツールは実装済みであり、本タスク群の TASK-0027 以降では stdio のみを扱う。

## 全体進捗

- [ ] Phase 0: MediaVault-api 先行改修
- [ ] Phase 1: MCP基盤構築
- [ ] Phase 2: 読み取りツール
- [ ] Phase 3: 書き込みツール
- [ ] Phase 4: 横断検証・仕上げ

## マイルストーン

| # | マイルストーン | 時期 | 達成条件 |
|---|---|---|---|
| M1 | api 側の前提が揃う | Day 4 | PREP-01〜04 完了。MCP 実装のブロッカーが解消 |
| M2 | **MCPが実際に繋がる** | Day 11 | Claude Code から `health` ツールが呼べる。MVP の実利用に到達する最初の地点 |
| M3 | 読み取りが揃う | Day 16 | US-01 / US-02 / US-09 が実行できる |
| M4 | 全ツール完成 | Day 22 | US-01〜US-09 のすべてが実行できる |
| M5 | MVP完了 | Day 26 | PRD §7.1 の完了条件5項目を達成 |

---

## Phase 0: MediaVault-api 先行改修

**期間**: Day 1-4
**目標**: MCP 実装のブロッカーとなる api 側の機能を先に整える
**成果物**: `relation_type` 6種別、別名・原題検索、該当件数、`GET /collection/overview`

> **MCP 実装とは独立して着手できる**。4タスクはすべて並行実行可能。

### タスク一覧

- [ ] [TASK-0001: relation_type ENUM の6種別拡張](TASK-0001.md) - 8h (TDD) 🔵
- [ ] [TASK-0002: GET /items の別名・原題検索対応](TASK-0002.md) - 8h (TDD) 🟡
- [ ] [TASK-0003: 検索結果の該当件数の返却](TASK-0003.md) - 8h (TDD) 🔵
- [ ] [TASK-0004: GET /api/v1/collection/overview の新設](TASK-0004.md) - 8h (TDD) 🔵

### 依存関係

```
TASK-0001 （独立）
TASK-0002 → TASK-0003   ※同一クエリ構築箇所を触るため
TASK-0004 （独立）
```

---

## Phase 1: MCP基盤構築

**期間**: Day 5-11
**目標**: MCP サーバーが起動し、認証され、最初のツールが動く状態にする
**成果物**: クレート、Config、認証、ApiClient、共通結果型、health ツール、Docker統合

### タスク一覧

- [ ] [TASK-0005: mediavault-mcp クレート作成と workspace 統合](TASK-0005.md) - 8h (DIRECT) 🔵
- [ ] [TASK-0006: Config と起動時検証](TASK-0006.md) - 8h (TDD) 🟡
- [ ] [TASK-0007: Bearer認証ミドルウェアと /healthz](TASK-0007.md) - 8h (TDD) 🔵
- [ ] [TASK-0008: ApiClient層の実装](TASK-0008.md) - 8h (TDD) 🔵
- [ ] [TASK-0009: 共通結果型と rmcp サーバー骨格](TASK-0009.md) - 8h (TDD) 🔵
- [ ] [TASK-0010: health ツールの実装](TASK-0010.md) - 8h (TDD) 🔵
- [ ] [TASK-0011: Dockerfile と docker-compose 統合](TASK-0011.md) - 8h (DIRECT) 🔵

### 依存関係

```
TASK-0005 → TASK-0006 → TASK-0007 ┐
                     → TASK-0008 ┴→ TASK-0009 → TASK-0010 → TASK-0011
```

TASK-0007 と TASK-0008 は並行可能。

---

## Phase 2: 読み取りツール

**期間**: Day 12-16
**目標**: 所蔵確認・作品理解・外部検索・統計を AI から利用できるようにする
**成果物**: `search_library`, `get_item_context`, `search_external_catalog`, `collection_overview`

### タスク一覧

- [ ] [TASK-0012: ItemSummary 縮約と名前→ID解決サービス](TASK-0012.md) - 8h (TDD) 🔵
- [ ] [TASK-0013: search_library ツールの実装](TASK-0013.md) - 8h (TDD) 🔵
- [ ] [TASK-0014: get_item_context ツールの実装](TASK-0014.md) - 8h (TDD) 🔵
- [ ] [TASK-0015: search_external_catalog ツールの実装](TASK-0015.md) - 8h (TDD) 🔵
- [ ] [TASK-0016: collection_overview ツールの実装](TASK-0016.md) - 8h (TDD) 🔵

### 依存関係

```
TASK-0010 → TASK-0012 ┬→ TASK-0013 → TASK-0014
                      ├→ TASK-0015
                      └→ TASK-0016
TASK-0002, TASK-0003 → TASK-0013   ※api側の前提
TASK-0004 → TASK-0016              ※api側の前提
```

TASK-0013 / TASK-0015 / TASK-0016 は並行可能。

---

## Phase 3: 書き込みツール

**期間**: Day 17-22
**目標**: 登録・記録・整理・関連付けを AI から実行できるようにする
**成果物**: `import_external_item`, `create_item`, `update_consumption`, `organize_item`, `relate_items`, `add_access_link`

### タスク一覧

- [ ] [TASK-0017: import_external_item ツールの実装](TASK-0017.md) - 8h (TDD) 🔵
- [ ] [TASK-0018: create_item ツールの実装](TASK-0018.md) - 8h (TDD) 🔵
- [ ] [TASK-0019: update_consumption ツールの実装](TASK-0019.md) - 8h (TDD) 🔵
- [ ] [TASK-0020: organize_item ツールの実装](TASK-0020.md) - 8h (TDD) 🔵
- [ ] [TASK-0021: relate_items ツールの実装](TASK-0021.md) - 8h (TDD) 🔵
- [ ] [TASK-0022: add_access_link ツールの実装](TASK-0022.md) - 8h (TDD) 🟡

### 依存関係

```
TASK-0015 → TASK-0017 ┬→ TASK-0018 → TASK-0020
                      ├→ TASK-0019
                      ├→ TASK-0021
                      └→ TASK-0022
TASK-0012 → TASK-0018, TASK-0020   ※名前解決
TASK-0001 → TASK-0021              ※api側の前提
```

TASK-0017（最初の書き込みツール）でパターンを確立し、以降は並行可能。

---

## Phase 4: 横断検証・仕上げ

**期間**: Day 23-26
**目標**: 個別タスクでは担保できない横断的な品質を保証し、MVP を完成させる
**成果物**: 安全性テスト、エラー一貫性テスト、調整済みツール説明文、CI、運用ドキュメント

### タスク一覧

- [ ] [TASK-0023: 安全性の横断検証](TASK-0023.md) - 8h (TDD) 🔵
- [ ] [TASK-0024: エラー透過とエッジケースの網羅テスト](TASK-0024.md) - 8h (TDD) 🔵
- [ ] [TASK-0025: ツール説明文とスキーマのAI可読性調整](TASK-0025.md) - 8h (TDD) 🔵
- [ ] [TASK-0026: CI と運用ドキュメント](TASK-0026.md) - 8h (DIRECT) 🔵

### 依存関係

```
TASK-0022 → TASK-0023 ┬→ TASK-0024 ┐
                      └→ TASK-0025 ┴→ TASK-0026
TASK-0011 → TASK-0026
```

---

## 信頼性レベルサマリー

### 全タスク統計

- **総タスク数**: 26件
- 🔵 **青信号**: 23件 (88%)
- 🟡 **黄信号**: 3件 (12%)
- 🔴 **赤信号**: 0件 (0%)

🟡 のタスク: TASK-0002（別名検索、`details` の実データ形状が未確認）、TASK-0006（Config、既定値の判断が多い）、TASK-0022（add_access_link、api 仕様の確認事項が残る）

### フェーズ別信頼性（タスク内の項目単位）

| フェーズ | 🔵 青 | 🟡 黄 | 🔴 赤 | 合計 | 🔵率 |
|---------|-------|-------|-------|------|------|
| Phase 0 | 44 | 23 | 0 | 67 | 66% |
| Phase 1 | 76 | 44 | 0 | 120 | 63% |
| Phase 2 | 93 | 45 | 0 | 138 | 67% |
| Phase 3 | 132 | 56 | 0 | 188 | 70% |
| Phase 4 | 68 | 41 | 0 | 109 | 62% |
| **合計** | **413** | **209** | **0** | **622** | **66%** |

**品質評価**: ✅ **高品質**

🔴（要件・設計に根拠のない推測）が0件である点が重要。🟡 の大半は「要件は確定しているが実装手段の選択が残る」項目であり、`/tsumiki:tdd-requirements` の段階で api 仕様書やコードを確認すれば確定する。

## クリティカルパス

```
TASK-0002 → TASK-0003 → TASK-0013 → TASK-0014
     ↑
TASK-0005 → TASK-0006 → TASK-0008 → TASK-0009 → TASK-0010 → TASK-0012 → TASK-0013
                                                                      ↓
TASK-0015 → TASK-0017 → TASK-0018 → TASK-0020 → TASK-0023 → TASK-0025 → TASK-0026
```

**クリティカルパス工数**: 約120時間（15日）
**並行作業可能工数**: 約88時間（11日）

Phase 0 を Phase 1 と並行させれば、実質22日程度まで短縮できる（api 改修と MCP 基盤は独立しているため）。

## 未解決の前提条件

実装着手前に確認・決定が必要な項目（[prep.md](../spec/prep.md) 参照）:

| 項目 | 影響するタスク | 状態 |
|---|---|---|
| PREP-05: `MCP_AUTH_TOKEN` の生成・配布 | TASK-0006, TASK-0011 | 未着手 |
| PREP-06: 外部プロバイダAPIキーの設定 | TASK-0015, TASK-0017（手動確認時）| 未着手 |
| PREP-09: `limit` 超過時の挙動 | TASK-0013, TASK-0016 | **設計で「拒否」に決定**。要ユーザー確認 |
| PREP-10: `rating` の許容範囲 | TASK-0019 | 未解決（api 仕様に記載なし）|
| PREP-11: リバースプロキシ経路での公開範囲 | TASK-0011 | 未解決 |
| `rmcp` 3.1 の実 API 形状 | TASK-0005, TASK-0009 | TASK-0005 で確認する |

> **TASK-0005 の確認結果次第で設計決定 D-01（構造化結果への統一）が成立しない可能性がある**。その場合は実装を止めて設計へ差し戻すこと（TASK-0009 注意事項）。

## 第2段階（MVP範囲外）

PRD §7.2 のうち、本タスク群の TASK-0027 以降で扱う未実装項目は stdio のみである。全文・抽出ツールは `docs/extractor/tasks/` の別番号空間で実装済み。

| 機能 | 対応要件 | 前提 |
|---|---|---|
| stdio トランスポート | REQ-902 | Tool層・Service層は変更不要 |

## 次のステップ

タスクを実装するには:

- 全タスク順番に実装: `/tsumiki:kairo-implement`
- 特定タスクを実装: `/tsumiki:kairo-implement TASK-0001`

**推奨**: Phase 0（TASK-0001〜0004）から着手する。MCP 実装とは独立しており、これを片付けないと Phase 2・3 でブロックされる。
