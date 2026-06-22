# frontend-collection-ui タスク概要

**作成日**: 2026-06-22
**推定工数**: 204時間（約26人日、6フェーズ）
**総タスク数**: 35件

## 関連文書

- **要件定義書**: [📋 requirements.md](../../spec/frontend-collection-ui/requirements.md)
- **設計文書**: [📐 architecture.md](../../design/frontend-collection-ui/architecture.md)
- **データフロー図**: [🔄 dataflow.md](../../design/frontend-collection-ui/dataflow.md)
- **型定義**: [📝 interfaces.ts](../../design/frontend-collection-ui/interfaces.ts)
- **コンテキストノート**: [📝 note.md](../../spec/frontend-collection-ui/note.md)
- **バックエンドAPI仕様（利用先）**: [docs/design/mediavault-backend/api-endpoints.md](../../design/mediavault-backend/api-endpoints.md)

## フェーズ構成

| フェーズ | 目標 | タスク数 | 工数 | ファイル |
|---------|------|----------|------|----------|
| Phase 1 | 基盤構築（プロジェクト設定・API層・共通コンポーネント） | 8 | 36h | [TASK-0001~0008](#phase-1-基盤構築) |
| Phase 2 | 一覧・検索（全体/グループ別一覧、外部API検索追加） | 8 | 44h | [TASK-0009~0016](#phase-2-一覧検索) |
| Phase 3 | 詳細・グループ・関連（詳細画面、シーズン/巻、関連付け、リンク/ファイル） | 5 | 36h | [TASK-0017~0021](#phase-3-詳細グループ関連) |
| Phase 4 | フォーム（手動追加・編集、メディア別フォーム） | 5 | 30h | [TASK-0022~0026](#phase-4-フォーム) |
| Phase 5 | 管理画面（タグ/カテゴリ・マイリスト・スタッフ・設定） | 6 | 34h | [TASK-0027~0032](#phase-5-管理画面) |
| Phase 6 | 統合・品質保証（ルーティング最終調整・A11y・E2E） | 3 | 20h | [TASK-0033~0035](#phase-6-統合品質保証) |

## タスク番号管理

**使用済みタスク番号**: TASK-0001 ~ TASK-0035
**次回開始番号**: TASK-0036

## 全体進捗

- [ ] Phase 1: 基盤構築
- [ ] Phase 2: 一覧・検索
- [ ] Phase 3: 詳細・グループ・関連
- [ ] Phase 4: フォーム
- [ ] Phase 5: 管理画面
- [ ] Phase 6: 統合・品質保証

## マイルストーン

- **M1: 基盤完成**: ルーティング・apiClient・共通コンポーネント・デザイントークン整備完了（Phase 1完了時点）
- **M2: 閲覧・検索完成**: 全体/グループ別一覧、外部API検索からの追加が完成（Phase 2完了時点）
- **M3: 詳細・編集完成**: 詳細画面、シーズン/巻管理、手動追加・編集フォームが完成（Phase 3・4完了時点）
- **M4: 管理機能完成**: タグ/カテゴリ・マイリスト・スタッフ・設定（APIキー/インポート）完成（Phase 5完了時点）
- **M5: リリース準備完了**: ルーティング最終調整・アクセシビリティ・主要E2Eテスト完了（Phase 6完了時点）

---

## Phase 1: 基盤構築

**目標**: プロジェクトの依存関係・デザイントークン・ルーティング基盤・API層・共通コンポーネントを整備する
**成果物**: ビルド可能なSPAスケルトン、apiClient、共通UIコンポーネント

### タスク一覧

- [ ] [TASK-0001: 依存パッケージ追加とプロジェクト設定](TASK-0001.md) - 4h (DIRECT) 🔵
- [ ] [TASK-0002: Tailwindデザイントークン設定](TASK-0002.md) - 4h (DIRECT) 🔴
- [ ] [TASK-0003: ディレクトリ構造とルーティング基盤構築](TASK-0003.md) - 4h (DIRECT) 🟡
- [ ] [TASK-0004: 型定義ファイル配置](TASK-0004.md) - 2h (DIRECT) 🔵
- [ ] [TASK-0005: apiClient実装](TASK-0005.md) - 6h (TDD) 🔵
- [ ] [TASK-0006: 共通UIコンポーネント実装](TASK-0006.md) - 8h (TDD) 🔵
- [ ] [TASK-0007: グローバルナビゲーション実装](TASK-0007.md) - 4h (TDD) 🔵
- [ ] [TASK-0008: 汎用フック実装](TASK-0008.md) - 4h (TDD) 🟡

### 依存関係

```
TASK-0001 → TASK-0002, TASK-0003, TASK-0004, TASK-0005
TASK-0001, TASK-0002 → TASK-0006
TASK-0003 → TASK-0007, TASK-0008
TASK-0004, TASK-0005 → Phase 2以降の api/* フック実装
```

**注意**: TASK-0002（デザイントークン）の参照元CSS（`docs/frontend/ui/01_components.html`、`_shared.css`）はリポジトリ上で削除済みのため具体的な色値は🔴推測。実装前にユーザー確認を推奨。

---

## Phase 2: 一覧・検索

**目標**: 全体一覧・グループ別一覧・絞り込み・外部API検索からの追加を実装する
**成果物**: HomePage/GeneralListPage/AcademicListPage/PaperListPage/SearchAddPage

### タスク一覧

- [ ] [TASK-0009: items APIフック実装](TASK-0009.md) - 6h (TDD) 🔵
- [ ] [TASK-0010: FilterBarコンポーネント詳細実装](TASK-0010.md) - 6h (TDD) 🔵
- [ ] [TASK-0011: HomePage(全体一覧)実装](TASK-0011.md) - 6h (TDD) 🔵
- [ ] [TASK-0012: GeneralListPage実装](TASK-0012.md) - 4h (TDD) 🔵
- [ ] [TASK-0013: AcademicListPage実装](TASK-0013.md) - 4h (TDD) 🔵
- [ ] [TASK-0014: PaperListPage実装](TASK-0014.md) - 4h (TDD) 🔵
- [ ] [TASK-0015: 外部API検索フック実装](TASK-0015.md) - 6h (TDD) 🔵
- [ ] [TASK-0016: SearchAddPage実装](TASK-0016.md) - 8h (TDD) 🔵

### 依存関係

```
TASK-0005 → TASK-0009, TASK-0015
TASK-0006, TASK-0008 → TASK-0010
TASK-0009, TASK-0010, TASK-0007 → TASK-0011
TASK-0011 → TASK-0012, TASK-0013, TASK-0014
TASK-0015, TASK-0006 → TASK-0016
```

---

## Phase 3: 詳細・グループ・関連

**目標**: 詳細画面・シーズン/巻管理・関連付け・リンク/ファイル/トレーラー管理を実装する
**成果物**: ItemDetailPage、GroupSection、relations/links-files機能

### タスク一覧

- [ ] [TASK-0017: ItemDetailPage基本実装](TASK-0017.md) - 8h (TDD) 🔵
- [ ] [TASK-0018: groups/episodes APIフック・GroupSection実装](TASK-0018.md) - 8h (TDD) 🔵
- [ ] [TASK-0019: 関連付け(relations)管理UI実装](TASK-0019.md) - 6h (TDD) 🔵
- [ ] [TASK-0020: リンク・ファイル・トレーラー管理UI実装](TASK-0020.md) - 8h (TDD) 🔵
- [ ] [TASK-0021: ファイルアップロード機能実装](TASK-0021.md) - 6h (TDD) 🟡

### 依存関係

```
TASK-0009, TASK-0006 → TASK-0017
TASK-0017 → TASK-0018, TASK-0019, TASK-0020
TASK-0020 → TASK-0021
```

---

## Phase 4: フォーム

**目標**: 手動追加・編集フォームをメディアグループ別に実装する
**成果物**: ItemFormPage、メディア別フォーム部品、zodスキーマ

### タスク一覧

- [ ] [TASK-0022: zodスキーマ・共通フォーム部品実装](TASK-0022.md) - 6h (TDD) 🔵
- [ ] [TASK-0023: 一般メディア用フォーム実装](TASK-0023.md) - 8h (TDD) 🔵
- [ ] [TASK-0024: 学術書・専門書用フォーム実装](TASK-0024.md) - 4h (TDD) 🔵
- [ ] [TASK-0025: 論文・文献用フォーム実装](TASK-0025.md) - 4h (TDD) 🔵
- [ ] [TASK-0026: ItemFormPage統合実装](TASK-0026.md) - 8h (TDD) 🔵

### 依存関係

```
TASK-0004 → TASK-0022
TASK-0022 → TASK-0023, TASK-0024, TASK-0025
TASK-0023, TASK-0024, TASK-0025 → TASK-0026
```

---

## Phase 5: 管理画面

**目標**: タグ/カテゴリ・マイリスト・スタッフ・設定（APIキー/インポート/エクスポート）画面を実装する
**成果物**: TagsCategoriesPage、MyListsPage、StaffPage、SettingsPage

### タスク一覧

- [ ] [TASK-0027: タグ/カテゴリ管理ページ実装](TASK-0027.md) - 6h (TDD) 🔵
- [ ] [TASK-0028: マイリスト管理ページ実装](TASK-0028.md) - 6h (TDD) 🔵
- [ ] [TASK-0029: スタッフ管理ページ実装](TASK-0029.md) - 6h (TDD) 🔵
- [ ] [TASK-0030: 設定ページ(APIキー管理)実装](TASK-0030.md) - 6h (TDD) 🔵
- [ ] [TASK-0031: 設定ページ(インポート機能)実装](TASK-0031.md) - 8h (TDD) 🔵
- [ ] [TASK-0032: 設定ページ(エクスポート未実装ボタン)実装](TASK-0032.md) - 2h (DIRECT) 🟡

### 依存関係

```
TASK-0005 → TASK-0027, TASK-0028, TASK-0029, TASK-0030
TASK-0030 → TASK-0031, TASK-0032
```

---

## Phase 6: 統合・品質保証

**目標**: 画面遷移の最終調整、アクセシビリティ・レスポンシブ対応、主要フローのE2Eテストを実施する
**成果物**: 調整済みルーティング、A11y対応、Playwright E2Eテスト

### タスク一覧

- [ ] [TASK-0033: 画面遷移統合・ルーティング最終調整](TASK-0033.md) - 4h (DIRECT) 🔵
- [ ] [TASK-0034: アクセシビリティ・レスポンシブ対応](TASK-0034.md) - 8h (TDD) 🔵
- [ ] [TASK-0035: E2E主要フローテスト整備](TASK-0035.md) - 8h (TDD) 🔵

### 依存関係

```
Phase2〜5全画面実装完了 → TASK-0033, TASK-0034
TASK-0033, TASK-0034 → TASK-0035
```

---

## 信頼性レベルサマリー

### 全タスク統計（全35ファイルの記載項目集計、概算）

- 🔵 **青信号**: 約375件 (59%)
- 🟡 **黄信号**: 約222件 (35%)
- 🔴 **赤信号**: 約36件 (6%)

**品質評価**: 高品質（要件定義書・設計文書・バックエンドAPI仕様との対応が明確なタスクが大半。デザイントークンの具体値（TASK-0002）はソースファイル削除のため🔴、UI実装パターンの一部詳細は🟡推測）

### タスクレベルの信頼性（タスク全体評価が🔵以外のもの）

| タスク | 信頼性 | 理由 |
|---|---|---|
| TASK-0002 | 🔴 | デザイントークン参照元CSSがリポジトリから削除済みのため色値は推測 |
| TASK-0003 | 🟡 | architecture.mdの「13画面」表記とルーティング表のパス数に差異あり |
| TASK-0008 | 🟡 | 汎用フックの実装詳細は設計文書から妥当な推測 |
| TASK-0021 | 🟡 | アップロード進捗表示の具体実装はNFR-203から妥当な推測 |
| TASK-0032 | 🟡 | REQ-023はスコープ対象外要件のため実装方針は推測含む |

## クリティカルパス

```
TASK-0001 → TASK-0005 → TASK-0009 → TASK-0011 → TASK-0017 → TASK-0018
  → TASK-0033 → TASK-0034 → TASK-0035
```

または並行するフォーム系列:

```
TASK-0001 → TASK-0004 → TASK-0022 → TASK-0023/0024/0025 → TASK-0026 → TASK-0033
```

**並行作業可能**: Phase 2（一覧系）とPhase 4（フォーム系）はPhase 1完了後、Phase 3はTASK-0009完了後にそれぞれ並行着手可能。Phase 5（管理画面）はTASK-0005完了後すぐに着手可能で他フェーズと並行できる。

## 次のステップ

タスクを実装するには:
- 全タスク順番に実装: `/tsumiki:kairo-implement`
- 特定タスクを実装: `/tsumiki:kairo-implement TASK-0001`
