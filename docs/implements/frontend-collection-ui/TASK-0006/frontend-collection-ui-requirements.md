# TASK-0006: 共通UIコンポーネント実装 - TDD要件定義書

**機能名**: 共通UIコンポーネント（MediaCard / MediaTypeBadge / FilterBar / EmptyState / ConfirmDialog）
**タスクID**: TASK-0006
**要件名**: frontend-collection-ui
**タスクタイプ**: TDD
**作成日**: 2026-06-30

---

## 信頼性レベル凡例

- 🔵 **青信号**: EARS要件定義書・設計文書を参考にしてほぼ推測していない
- 🟡 **黄信号**: EARS要件定義書・設計文書から妥当な推測
- 🔴 **赤信号**: EARS要件定義書・設計文書にない推測

---

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

- 🔵 **何をする機能か**: 複数の一覧・詳細画面で再利用する独自UIコンポーネント群を `src/components/common/` に実装する。本タスクでは以下の5コンポーネントを対象とする。
  - **MediaCard**: 一覧画面でアイテム1件をカード表示する
  - **MediaTypeBadge**: `MediaType` に応じた色・ラベルのバッジを表示する
  - **FilterBar**: 絞り込みUIの器（コンテナ）。本タスクでは枠のみ（詳細はPhase 2）
  - **EmptyState**: アイテム0件時の空状態メッセージ・追加導線を表示する
  - **ConfirmDialog**: 削除等の操作に対する確認ダイアログを表示する

- 🔵 **どのような問題を解決するか**: 各画面（HomePage / GeneralListPage / AcademicListPage / PaperListPage 等）で重複しがちなカード表示・バッジ・空状態・確認ダイアログを共通化し、UI一貫性と再利用性を担保する。これによりPhase 2以降の画面実装タスクの土台を提供する。

- 🟡 **想定されるユーザー**: MediaVaultのコレクション管理を行うエンドユーザー（コンポーネント自体は画面を経由して利用される）。直接の利用者は後続タスクを実装する開発者。

- 🔵 **システム内での位置づけ**: Feature-Sliced寄りのレイヤード構成（pages → features → components → api/hooks/types/lib）における `components/common/` レイヤー。shadcn/ui（`components/ui/`）をベースコンポーネントとして使用する独自コンポーネント層。

- **参照したEARS要件**: REQ-001（カード/リスト表示）, REQ-002/REQ-003（絞り込み）, EDGE-101（空状態）, REQ-007（削除確認の用途）
- **参照した設計文書**:
  - `docs/design/frontend-collection-ui/architecture.md`「コンポーネント粒度」
  - `docs/tasks/frontend-collection-ui/TASK-0006.md`「実装詳細」

---

## 2. 入力・出力の仕様（EARS機能要件・TypeScript型定義ベース）

### 2.1 MediaCard 🔵

- **入力（Props）**:
  ```typescript
  interface MediaCardProps {
    item: Item;                      // 🔵 interfaces.ts Item型（media_type判別共用体）
    onClick?: (item: Item) => void;  // 🟡 クリック時コールバック（設計文書に明記なし、妥当な推測）
  }
  ```
- **出力（描画）**: `item.coverImageUrl`（カバー画像）, `item.title`（タイトル）, `item.mediaType`（→ MediaTypeBadge）, `item.isFavorite`（お気に入り表示）, `item.status`（ステータス表示）を含むカードDOM。
- **入出力の関係性**: `item` props を受け取り表示要素にマッピング。`onClick` 指定時はカードクリックで `onClick(item)` を呼ぶ。
- **参照したEARS要件**: REQ-001
- **参照した設計文書**: `interfaces.ts` の `Item`（`ItemBase` + media_type別 `details`）

### 2.2 MediaTypeBadge 🔵

- **入力（Props）**:
  ```typescript
  interface MediaTypeBadgeProps {
    mediaType: MediaType;  // 🔵 8種別: anime/movie/drama/manga/novel/game/academic_book/paper
  }
  ```
- **出力（描画）**: 種別ごとのアクセントカラークラス（`getMediaTypeAccentClass()` 由来、例: `text-accent-anime`）と日本語ラベル（例: anime→「アニメ」🟡）を反映したバッジDOM。shadcn/ui の `Badge` をベースに使用。
- **入出力の関係性**: `mediaType` → アクセントクラス + ラベルへの決定的マッピング。
- **参照したEARS要件**: -（TASK-0002 アクセントカラー定義に依拠）
- **参照した設計文書**: `frontend/src/lib/media-type-accent.ts`（`getMediaTypeAccentClass()`）, `interfaces.ts` の `MediaType`

### 2.3 FilterBar（枠のみ） 🔵

- **入力（Props）**:
  ```typescript
  interface FilterBarProps {
    children?: React.ReactNode;  // 🔵 タスク指示「枠のみ、詳細はPhase2」
  }
  ```
- **出力（描画）**: `children` をラップするコンテナDOMのみ。media_type/タグ/カテゴリ/お気に入り/status の選択UIは本タスク対象外（Phase 2 / TASK-0010）。
- **入出力の関係性**: `children` をそのままレンダリング。後続フック `useSearchParamsFilter`（TASK-0008）との接続点となる器。
- **参照したEARS要件**: REQ-002/REQ-003（将来接続先、本タスクでは枠のみ）
- **参照した設計文書**: `architecture.md`「コンポーネント粒度」, TASK-0006「FilterBar（枠のみ）」

### 2.4 EmptyState 🟡

- **入力（Props）**:
  ```typescript
  interface EmptyStateProps {
    message: string;           // 🟡 EDGE-101「コレクションがありません」等のメッセージ
    actionLabel?: string;      // 🟡 追加導線ボタンのラベル（任意）
    onAction?: () => void;     // 🟡 追加導線ボタンのコールバック（任意）
  }
  ```
- **出力（描画）**: `message` を表示。`actionLabel` + `onAction` が両方指定された場合のみアクションボタンを表示。
- **入出力の関係性**: `message` 表示は必須。ボタンクリックで `onAction()` を呼ぶ。
- **参照したEARS要件**: EDGE-101（0件時の空状態と追加画面への導線）
- **参照した設計文書**: `architecture.md`「コンポーネント粒度」（コンポーネント存在は🔵、props構成は🟡推測）

### 2.5 ConfirmDialog 🟡

- **入力（Props）**:
  ```typescript
  interface ConfirmDialogProps {
    open: boolean;             // 🟡 表示/非表示の制御
    title: string;             // 🟡 ダイアログタイトル
    description?: string;      // 🟡 補足説明（任意）
    onConfirm: () => void;     // 🟡 確認時コールバック
    onCancel: () => void;      // 🟡 キャンセル時コールバック
    confirmLabel?: string;     // 🟡 確認ボタンラベル（任意・デフォルトあり）
    cancelLabel?: string;      // 🟡 キャンセルボタンラベル（任意・デフォルトあり）
  }
  ```
- **出力（描画）**: `open=true` のとき `title`/`description`/確認・キャンセルボタンを含むダイアログDOM。`open=false` のとき内容非表示。shadcn/ui の `Dialog` をベースに使用。
- **入出力の関係性**: 確認ボタン→ `onConfirm()`、キャンセルボタン→ `onCancel()`。状態（open/close）は呼び出し側または `useConfirmDialog`（TASK-0008）が管理。
- **参照したEARS要件**: REQ-007（アイテム削除の確認用途）
- **参照した設計文書**: `architecture.md`「コンポーネント粒度」（コンポーネント名は🔵、props構成は🟡推測）

---

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

- 🔵 **アーキテクチャ制約**:
  - 各コンポーネントは `src/components/common/` に配置する（`src/components/ui/` は shadcn/ui 専用）。
  - shadcn/ui のベースコンポーネント（Button / Dialog / Badge 等）を利用する。
  - media_type アクセント色は `getMediaTypeAccentClass()` 関数経由で取得する（直接ハードコードしない）。
  - CSSクラス結合は `cn()`（tailwind-merge + clsx）を利用する。条件付きスタイルは `class-variance-authority`（cva）パターンに準拠する。

- 🔵 **型安全性制約**:
  - `Item` 型は `media_type` 判別共用体として扱う。型ガード `isItemOfType<T>()` を必要に応じ利用する。
  - 各コンポーネントの props インターフェースを明示的に定義する。

- 🔵 **テスト制約**:
  - 単体テストのみ対象（Vitest + @testing-library/react + jsdom）。
  - テストファイルはコンポーネントと同ディレクトリに `*.test.tsx` で配置（例: `src/components/common/MediaCard.test.tsx`）。
  - jest-dom matchers は `src/test/setup.ts` 経由で有効化済み。
  - 統合テスト・E2Eテストは本タスク対象外（Phase 2 の画面実装タスクで実施）。

- 🟡 **パフォーマンス要件**: 本タスク固有の数値目標は設計文書に明記なし。表示専用の軽量コンポーネントとして実装する。

- 🟡 **セキュリティ要件**: 本タスク固有の要件は設計文書に明記なし（共通コンポーネントは表示専用のため該当少）。

- 🔵 **互換性要件**: React 18.3+ / TypeScript 5.7+ / Tailwind CSS v4 / React Router v7 / lucide-react に準拠する。

- **参照したEARS要件**: -（非機能の数値要件は本タスクスコープ外）
- **参照した設計文書**: `architecture.md`, `frontend/src/lib/media-type-accent.ts`, `frontend/src/lib/utils.ts`, `frontend/CLAUDE.md`

---

## 4. 想定される使用例（EARS Edgeケース・データフローベース）

### 4.1 基本的な使用パターン

- 🔵 **MediaCard**: 一覧画面が `Item` の配列をmapし、各 `Item` を `<MediaCard item={item} onClick={...} />` として描画。クリックで詳細画面へ遷移する想定。
- 🔵 **MediaTypeBadge**: MediaCard内および詳細画面で `<MediaTypeBadge mediaType={item.mediaType} />` として種別を視覚化。
- 🔵 **FilterBar**: 一覧画面上部に `<FilterBar>{filterControls}</FilterBar>` として絞り込みUIを内包（本タスクでは枠のみ）。
- 🟡 **EmptyState**: 一覧取得結果が0件のとき `<EmptyState message="コレクションがありません" actionLabel="追加" onAction={...} />` を表示。
- 🟡 **ConfirmDialog**: 削除操作時に `<ConfirmDialog open={...} title="削除しますか？" onConfirm={...} onCancel={...} />` を表示。

### 4.2 エッジケース

- 🟡 **EmptyState（actionなし）**: `actionLabel` / `onAction` 未指定時はメッセージのみ表示し、ボタンは描画しない。
- 🟡 **ConfirmDialog（open=false）**: `open=false` のときはダイアログ内容を描画しない。
- 🟡 **MediaCard（onClickなし）**: `onClick` 未指定時はクリックしてもエラーにならない（ハンドラ呼び出しなし）。
- 🟡 **MediaCard（任意項目欠落）**: `coverImageUrl` 等の任意項目が未設定でもエラーなく描画する（プレースホルダ等の扱いはGreen/Refactorで具体化）。
- 🔵 **MediaTypeBadge（8種別網羅）**: anime/movie/drama/manga/novel/game/academic_book/paper の全種別でエラーなく描画し、種別ごとに異なるアクセントクラスを付与する。

### 4.3 エラーケース

- 🟡 想定される異常入力はTypeScript型システムで概ね排除される（`mediaType` は8種別のリテラル union、`Item` は判別共用体）。実行時の不正値ハンドリングは本タスクの主目的外。

- **参照したEARS要件**: EDGE-101（0件空状態）
- **参照した設計文書**: `dataflow.md`（一覧表示フロー）, TASK-0006「単体テスト要件」

---

## 5. EARS要件・設計文書との対応関係

- **参照したユーザストーリー**: コレクション全体の一覧表示・絞り込み・削除確認（requirements.md ユーザストーリー群）
- **参照した機能要件**:
  - REQ-001: 全体一覧画面でコレクションをカード/リスト表示（→ MediaCard）
  - REQ-002 / REQ-003: media_type・タグ・カテゴリ・お気に入り・status での絞り込み（→ FilterBar、本タスクは枠のみ）
  - REQ-007: アイテム削除（→ ConfirmDialog の用途）
- **参照した非機能要件**: 本タスク固有の数値NFRは未参照（スコープ外）
- **参照したEdgeケース**:
  - EDGE-101: アイテム0件時の空状態メッセージと追加導線（→ EmptyState）
- **参照した受け入れ基準**: `acceptance-criteria.md` TC-001-01 他（フィルタ・CRUD・Empty State のテストケース定義）
- **参照した設計文書**:
  - **アーキテクチャ**: `docs/design/frontend-collection-ui/architecture.md`「コンポーネント粒度」「レイヤード構成」
  - **データフロー**: `docs/design/frontend-collection-ui/dataflow.md`（一覧表示フロー）
  - **型定義**: `docs/design/frontend-collection-ui/interfaces.ts`（`Item`, `MediaType`, `ItemStatus`, `isItemOfType`）
  - **実装参照**: `frontend/src/lib/media-type-accent.ts`, `frontend/src/components/ui/button.tsx`, `frontend/src/lib/utils.ts`
  - **テスト参照**: `frontend/src/App.test.tsx`, `frontend/src/test/setup.ts`, `frontend/vitest.config.ts`

---

## 6. 実装対象ファイル（予定）

| コンポーネント | 実装ファイル | テストファイル |
|---|---|---|
| MediaCard | `frontend/src/components/common/MediaCard.tsx` | `frontend/src/components/common/MediaCard.test.tsx` |
| MediaTypeBadge | `frontend/src/components/common/MediaTypeBadge.tsx` | `frontend/src/components/common/MediaTypeBadge.test.tsx` |
| FilterBar | `frontend/src/components/common/FilterBar.tsx` | `frontend/src/components/common/FilterBar.test.tsx` |
| EmptyState | `frontend/src/components/common/EmptyState.tsx` | `frontend/src/components/common/EmptyState.test.tsx` |
| ConfirmDialog | `frontend/src/components/common/ConfirmDialog.tsx` | `frontend/src/components/common/ConfirmDialog.test.tsx` |

※ shadcn/ui の `badge` / `dialog` が未導入の場合は `npx shadcn@latest add badge dialog` で追加する。

---

## 7. 品質判定

```
✅ 高品質:
- 要件の曖昧さ: ほぼなし（各コンポーネントの責務・props・テスト観点が明確）
- 入出力定義: 完全（5コンポーネントすべてのprops/出力を定義）
- 制約条件: 明確（配置・ベースコンポーネント・テスト方針が確定）
- 実装可能性: 確実（依存する型・ユーティリティ・テスト基盤が整備済み）
- 信頼性レベル分布:
  - 🔵 青信号: コンポーネント存在・命名・配置・MediaCard/MediaTypeBadge/FilterBar の中核仕様
  - 🟡 黄信号: EmptyState/ConfirmDialog の props 構成、MediaCardレイアウト詳細、日本語ラベル
  - 🔴 赤信号: なし
```

**総合評価**: コンポーネントの存在・役割分担・配置は architecture.md から確実（🔵）。EmptyState / ConfirmDialog の詳細 props 構成と一部表示仕様は設計文書に直接記載がなく、一般的なUIパターンからの妥当な推測（🟡）を含むが、TASK-0006 のタスク文書で props 形が明示されているため実装上の曖昧さは小さい。TDD実装に着手可能な高品質要件と判断する。

---

## 8. 次のステップ

次のお勧めステップ: `/tsumiki:tdd-testcases frontend-collection-ui TASK-0006` でテストケースの洗い出しを行います。
