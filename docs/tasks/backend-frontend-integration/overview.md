# backend-frontend-integration タスク概要

**作成日**: 2026-07-02
**プロジェクト期間**: 単一フェーズ・数日程度（1日単位換算で3日相当）
**推定工数**: 18時間
**総タスク数**: 7件

## 関連文書

- **要件定義書**: [📋 requirements.md](../spec/backend-frontend-integration/requirements.md)
- **設計文書**: [📐 architecture.md](../design/backend-frontend-integration/architecture.md)
- **データフロー図**: [🔄 dataflow.md](../design/backend-frontend-integration/dataflow.md)
- **受け入れ基準**: [✅ acceptance-criteria.md](../spec/backend-frontend-integration/acceptance-criteria.md)
- **コンテキストノート**: [📝 note.md](../spec/backend-frontend-integration/note.md)
- **PRD**: [📄 PRD-integration.md](../PRD-integration.md)

> 本要件は既存backend/frontendのインフラ結合作業が中心であり、新規API・DBスキーマを追加しないため、`interfaces.ts`・`database-schema.sql`・`api-endpoints.md` は設計文書として生成されていない（architecture.md参照）。それに伴い全体規模が小さいため、フェーズは1つに集約している。

## フェーズ構成

| フェーズ | 期間目安 | 成果物 | タスク数 | 工数 | ファイル |
|---------|------|--------|----------|------|----------|
| Phase 1 | 3日 | 統合docker-compose・frontend配信基盤・結合テスト | 7 | 18h | [TASK-0001~0007](#phase-1-バックエンドフロントエンド結合基盤) |

## タスク番号管理

**使用済みタスク番号**: TASK-0001 ~ TASK-0007
**次回開始番号**: TASK-0008

## 全体進捗

- [ ] Phase 1: バックエンド・フロントエンド結合基盤

## マイルストーン

- **M1: フロントエンド配信基盤完成**: `frontend/Dockerfile`・`frontend/nginx.conf`・`apiClient`相対パス化が完了（TASK-0001〜0003）
- **M2: 統合環境構築完成**: ルート`docker-compose.yml`・`.env.example`が完成し `docker compose up` で起動可能（TASK-0004〜0005）
- **M3: 結合確認完了**: 受け入れ基準の全テストケースがパスし、ドキュメント化まで完了（TASK-0006〜0007）

---

## Phase 1: バックエンド・フロントエンド結合基盤

**期間**: 3日
**目標**: 3コンテナ構成のローカル結合環境を構築し、`docker compose up` 一発で結合テストが行える状態にする
**成果物**: `frontend/Dockerfile`, `frontend/nginx.conf`, ルート`docker-compose.yml`, ルート`.env.example`, `apiClient`相対パス化, 結合テスト結果, README追記

### タスク一覧

- [ ] [TASK-0001: frontend/Dockerfile 新規作成](TASK-0001.md) - 3h (DIRECT) 🔵
- [ ] [TASK-0002: frontend/nginx.conf 新規作成](TASK-0002.md) - 3h (DIRECT) 🔵
- [ ] [TASK-0003: apiClient のデフォルトBASE_URLを相対パス化](TASK-0003.md) - 2h (TDD) 🔵
- [ ] [TASK-0004: ルート統合用 docker-compose.yml 新規作成](TASK-0004.md) - 4h (DIRECT) 🔵
- [ ] [TASK-0005: ルート .env.example 整備](TASK-0005.md) - 1h (DIRECT) 🔵
- [ ] [TASK-0006: 統合環境の起動・疎通・分離結合テスト](TASK-0006.md) - 4h (TDD) 🔵
- [ ] [TASK-0007: README等への統合起動手順ドキュメント追記](TASK-0007.md) - 1h (DIRECT) 🟡

### 依存関係

```
TASK-0001 ─┐
TASK-0002 ─┼─→ TASK-0004 ─→ TASK-0006 ─→ TASK-0007
TASK-0005 ─┘
TASK-0003 ─────────────────→ TASK-0006
```

- TASK-0001（frontend/Dockerfile）、TASK-0002（nginx.conf）、TASK-0003（apiClient相対パス化）、TASK-0005（.env.example）は並行実行可能（相互依存なし）
- TASK-0004はTASK-0001・0002・0005の完了後に着手（Dockerfile/nginx.conf/環境変数を参照するため）
- TASK-0006はTASK-0004・0003の完了後に着手
- TASK-0007はTASK-0006完了後

---

## 信頼性レベルサマリー

### 全タスク統計

- **総タスク数**: 7件
- 🔵 **青信号**: 6件 (86%)
- 🟡 **黄信号**: 1件 (14%)
- 🔴 **赤信号**: 0件 (0%)

### フェーズ別信頼性

| フェーズ | 🔵 青 | 🟡 黄 | 🔴 赤 | 合計 |
|---------|-------|-------|-------|------|
| Phase 1 | 6 | 1 | 0 | 7 |

**品質評価**: 高品質

## クリティカルパス

```
TASK-0001/0002/0005（並行） → TASK-0004 → TASK-0006 → TASK-0007
```

**クリティカルパス工数**: 4h（TASK-0004最大値） + 4h（TASK-0006） + 1h（TASK-0007） + 並行区間の最大値3h（TASK-0001） = 12時間
**並行作業可能工数**: TASK-0001(3h) + TASK-0002(3h) + TASK-0003(2h) + TASK-0005(1h) = 9時間分が並行実行対象（うちクリティカルパス上はTASK-0001の3hのみ計上）

## 次のステップ

タスクを実装するには:
- 全タスク順番に実装: `/tsumiki:kairo-implement`
- 特定タスクを実装: `/tsumiki:kairo-implement TASK-0001`
