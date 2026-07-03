# frontend-ui-compliance タスク概要

**作成日**: 2026-07-03
**プロジェクト期間**: 目安4フェーズ（1フェーズ=数日、合計16タスク）
**推定工数**: 99時間
**総タスク数**: 16件

## 関連文書

- **要件定義書**: [📋 requirements.md](../../spec/frontend-ui-compliance/requirements.md)
- **要件ヒアリング記録**: [💬 interview-record.md](../../spec/frontend-ui-compliance/interview-record.md)
- **設計文書**: [📐 architecture.md](../../design/frontend-ui-compliance/architecture.md)
- **データフロー図**: [🔄 dataflow.md](../../design/frontend-ui-compliance/dataflow.md)
- **設計ヒアリング記録**: [💬 design-interview.md](../../design/frontend-ui-compliance/design-interview.md)
- **PRD**: [ui-compliance-PRD.md](../../frontend/ui-compliance-PRD.md)
- **モックアップ共通スタイル**: [_shared.css](../../frontend/ui/_shared.css)

## フェーズ構成

| フェーズ | 目標 | タスク数 | 工数 | ファイル |
|---------|------|----------|------|----------|
| Phase 1 | デザイントークン基盤 | 2 | 14h | [TASK-0001~0002](#phase-1-デザイントークン基盤) |
| Phase 2 | 共通コンポーネント拡張 | 7 | 41h | [TASK-0003~0009](#phase-2-共通コンポーネント拡張) |
| Phase 3 | 画面別スタイル適用 | 5 | 38h | [TASK-0010~0014](#phase-3-画面別スタイル適用) |
| Phase 4 | 統合・検証 | 2 | 12h | [TASK-0015~0016](#phase-4-統合検証) |

## タスク番号管理

**使用済みタスク番号**: TASK-0001 ~ TASK-0016（本要件専用の連番。他要件 frontend-collection-ui / mediavault-backend / backend-frontend-integration とは独立採番）
**次回開始番号**: TASK-0017

## 全体進捗

- [ ] Phase 1: デザイントークン基盤
- [ ] Phase 2: 共通コンポーネント拡張
- [ ] Phase 3: 画面別スタイル適用
- [ ] Phase 4: 統合・検証

## マイルストーン

- **M1: トークン基盤完成**: `index.css`のトークン置換・Tailwind連携完了（TASK-0001, 0002）
- **M2: 共通部品完成**: Button/TagPill/EmptyState/FilterBar/Sidebar/MediaCard/RootLayoutの拡張完了（TASK-0003~0009）
- **M3: 全画面UI完成**: 5画面すべての視覚デザイン準拠完了（TASK-0010~0014）
- **M4: リリース準備完了**: アクセシビリティ検証・全体リグレッション確認完了（TASK-0015, 0016）

---

## Phase 1: デザイントークン基盤

**目標**: `_shared.css`準拠のデザイントークンを`index.css`に反映し、Tailwind/shadcnと連携させる
**成果物**: 更新済み `frontend/src/index.css`

### タスク一覧

- [x] [TASK-0001: デザイントークンの再定義（index.css）](TASK-0001.md) - 8h (TDD) 🔵
- [x] [TASK-0002: Tailwind @theme連携とshadcn oklchトークンの上書き](TASK-0002.md) - 6h (TDD) 🔵

### 依存関係

```
TASK-0001 → TASK-0002
```

---

## Phase 2: 共通コンポーネント拡張

**目標**: モックアップ準拠の共通コンポーネント（ボタン・タグピル・空状態・フィルタバー・サイドバー・メディアカード・レイアウト）を実装する
**成果物**: 更新済み `Button`, 新規 `TagPill`, 更新済み `EmptyState` / `FilterBar` / `Sidebar` / `MediaCard` / `RootLayout`

### タスク一覧

- [ ] [TASK-0003: Buttonコンポーネントのvariant拡張](TASK-0003.md) - 6h (TDD) 🔵
- [ ] [TASK-0004: TagPillコンポーネントの新規作成](TASK-0004.md) - 4h (TDD) 🔵
- [ ] [TASK-0005: EmptyStateのモックアップ準拠更新](TASK-0005.md) - 3h (TDD) 🔵
- [ ] [TASK-0006: FilterBarのモックアップ準拠更新](TASK-0006.md) - 6h (TDD) 🔵
- [ ] [TASK-0007: Sidebarのモックアップ準拠拡張](TASK-0007.md) - 8h (TDD) 🔵
- [ ] [TASK-0008: MediaCardのモックアップ準拠拡張](TASK-0008.md) - 6h (TDD) 🔵
- [ ] [TASK-0009: RootLayoutのapp-shellグリッド化](TASK-0009.md) - 6h (TDD) 🔵

### 依存関係

```
TASK-0002 → TASK-0003 → TASK-0004
TASK-0002 → TASK-0003 → TASK-0005
TASK-0002 → TASK-0003 → TASK-0006
TASK-0003 → TASK-0007
TASK-0003 → TASK-0008
TASK-0002 → TASK-0009
```

---

## Phase 3: 画面別スタイル適用

**目標**: 全5画面（全体一覧・詳細・検索追加・フォーム・設定）にモックアップ準拠のレイアウトを適用する
**成果物**: 更新済み `HomePage` / `ItemDetailPage` / `SearchAddPage` / `ItemFormPage` / `SettingsPage`

### タスク一覧

- [ ] [TASK-0010: HomePageのフィルタバー・追加ボタン統合](TASK-0010.md) - 8h (TDD) 🔵
- [ ] [TASK-0011: ItemDetailPageのパンくず・タイトルバー・ドキュメント本文](TASK-0011.md) - 8h (TDD) 🔵
- [ ] [TASK-0012: SearchAddPageの検索結果リスト更新](TASK-0012.md) - 6h (TDD) 🔵
- [ ] [TASK-0013: ItemFormPageの2カラムフォーム化](TASK-0013.md) - 8h (TDD) 🔵
- [ ] [TASK-0014: SettingsPageのタブ+パネルレイアウト化](TASK-0014.md) - 8h (TDD) 🔵

### 依存関係

```
TASK-0006, TASK-0007, TASK-0008, TASK-0009 → TASK-0010
TASK-0009, TASK-0003 → TASK-0011
TASK-0003 → TASK-0012
TASK-0003, TASK-0004 → TASK-0013
TASK-0003 → TASK-0014
```

---

## Phase 4: 統合・検証

**目標**: 全画面のアクセシビリティ・リグレッションを最終確認する
**成果物**: 検証チェックリスト、ビルド/テスト成功確認

### タスク一覧

- [ ] [TASK-0015: 全画面横断のアクセシビリティ・コントラスト比検証](TASK-0015.md) - 6h (DIRECT) 🔵
- [ ] [TASK-0016: 全体リグレッションチェックとビルド確認](TASK-0016.md) - 6h (DIRECT) 🔵

### 依存関係

```
TASK-0010, TASK-0011, TASK-0012, TASK-0013, TASK-0014 → TASK-0015 → TASK-0016
```

---

## 信頼性レベルサマリー

### 全タスク統計（タスク全体の信頼性レベル、各ファイル先頭の値）

- **総タスク数**: 16件
- 🔵 **青信号**: 16件 (100%)
- 🟡 **黄信号**: 0件 (0%)
- 🔴 **赤信号**: 0件 (0%)

すべてのタスクが要件定義書（REQ-001〜011, REQ-401/402）・設計文書（architecture.md/dataflow.md）・実装済みファイルの現況確認に基づいて定義されている。ただし各タスク内部の実装詳細項目には、既存コンポーネントとの統合方法など🟡（黄信号）の推測項目が一部含まれる（例: TASK-0002のshadcn `--accent`トークンとの名前衝突リスク、TASK-0008のMediaTypeBadgeとの統合方法）。

**品質評価**: 高品質（依存関係が明確、実装可能性が高い。一部🟡項目は実装時に確認が必要）

## クリティカルパス

```
TASK-0001 → TASK-0002 → TASK-0003 → TASK-0007 → TASK-0010 → TASK-0015 → TASK-0016
```

**クリティカルパス工数**: 8+6+6+8+8+6+6 = 48時間
**並行作業可能工数**: 51時間（Phase2の一部タスク、Phase3の一部画面タスクは並行実施可能）

## 特記事項（設計との差分）

- 設計文書（architecture.md）では`EmptyState`/`FilterBar`が「新規」と記載されているが、実際には`frontend/src/components/common/`に既存実装があるため、TASK-0005/TASK-0006は「新規作成」ではなく「モックアップ準拠への更新」として定義した。
- `TagPill`のみ実際に新規作成が必要（TASK-0004）。

## 次のステップ

タスクを実装するには:
- 全タスク順番に実装: `/tsumiki:kairo-implement`
- 特定タスクを実装: `/tsumiki:kairo-implement TASK-0001`
