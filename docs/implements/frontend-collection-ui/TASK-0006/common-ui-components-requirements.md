# TASK-0006: 共通UIコンポーネント実装 - TDD要件定義書

**機能名**: 共通UIコンポーネント実装（common-ui-components）
**タスクID**: TASK-0006
**要件名**: frontend-collection-ui
**タスクタイプ**: TDD
**フェーズ**: Phase 1 - 基盤構築
**作成日**: 2026-06-30

## 信頼性レベル凡例

- 🔵 **青信号**: EARS要件定義書・設計文書を参考にしてほぼ推測していない
- 🟡 **黄信号**: EARS要件定義書・設計文書から妥当な推測
- 🔴 **赤信号**: EARS要件定義書・設計文書にない推測

---

## 1. 機能の概要（EARS要件定義書・設計文書ベース）

- 🔵 **何をする機能か**: `src/components/common/` に、複数画面で再利用する独自UIコンポーネント5種（`MediaCard`, `MediaTypeBadge`, `FilterBar`（枠のみ）, `EmptyState`, `ConfirmDialog`）を実装する。shadcn/ui（Button, Dialog, Badge 等、`src/components/ui/`）を基底コンポーネントとして利用する。
  - 参照: `docs/design/frontend-collection-ui/architecture.md`「コンポーネント粒度」, `docs/tasks/frontend-collection-ui/TASK-0006.md`

- 🔵 **どのような問題を解決するか**: 一覧・詳細・管理など複数画面に共通して現れるUIパターン（アイテムのカード表示、メディア種別バッジ、絞り込みバーの器、空状態表示、削除等の確認ダイアログ）を共通部品として切り出し、画面実装タスク（Phase 2以降）で再利用できるようにすることで、UIの一貫性と実装効率を担保する。
  - 参照: overview.md「Phase 1 成果物: 共通UIコンポーネント」, architecture.md「Component 粒度分割: ui（shadcn/ui）+ common（再利用独自コンポーネント）」

- 🔵 **想定されるユーザー**: 直接的には後続の画面実装タスク（HomePage/GeneralListPage/AcademicListPage/PaperListPage 他）を担当する開発者。間接的には、これらの画面を通じてコレクションを閲覧・管理するエンドユーザー（個人のメディアコレクション管理者）。
  - 参照: requirements.md REQ-001「全体一覧画面でコレクション全体をカード/リスト表示」

- 🔵 **システム内での位置づけ**: Feature-Sliced 寄りレイヤード構成（pages → features → components → api/hooks/types/lib）の `components/common` レイヤー。`lib/media-type-accent.ts` の `getMediaTypeAccentClass()` と `lib/utils.ts` の `cn()` に依存し、`types`（Item, MediaType）を入力として受け取る純粋な表示層コンポーネント群。
  - 参照: architecture.md「Feature-Sliced 寄りレイヤード構成」, note.md「3. 関連実装」

- **参照したEARS要件**: REQ-001, REQ-002, REQ-007, EDGE-101
- **参照した設計文書**: architecture.md「コンポーネント粒度」, interfaces.ts（Item, MediaType）, media-type-accent.ts

---

## 2. 入力・出力の仕様（EARS機能要件・TypeScript型定義ベース）

各コンポーネントは React 関数コンポーネントであり、入力 = props、出力 = レンダリング結果（JSX）およびコールバック呼び出し。

### 2.1 MediaCard 🔵

- 🔵 **入力（props）**:
  ```typescript
  interface MediaCardProps {
    item: Item;                       // 判別共用体（media_typeで8種別）。interfaces.ts Item型
    onClick?: (item: Item) => void;   // 任意。カードクリック時に当該itemを渡して呼ばれる
  }
  ```
  - `item.title`: string（必須）
  - `item.coverImageUrl`: string（任意。ISO/URL）
  - `item.mediaType`: MediaType（必須、判別子）
  - `item.isFavorite`: boolean（必須）
  - `item.status`: ItemStatus（'not_started' | 'in_progress' | 'completed'）
- 🔵 **出力**: `title`, `coverImageUrl`（画像）, `mediaType`（→`MediaTypeBadge`）, `isFavorite`, `status` を表示するカード。`onClick` 指定時はカードクリックで `onClick(item)` を呼ぶ。
- 🟡 **入出力の関係性**: カード内の具体的レイアウト（画像位置・お気に入りアイコンの種類・status表示形式）は設計文書に明記がないため妥当な推測。`isFavorite`／`status` の視覚表現は実装時に決定。
- 参照: interfaces.ts ItemBase（title, coverImageUrl, isFavorite, status, mediaType）, TASK-0006「実装詳細 > MediaCard」

### 2.2 MediaTypeBadge 🔵

- 🔵 **入力（props）**:
  ```typescript
  interface MediaTypeBadgeProps {
    mediaType: MediaType;  // 'anime'|'movie'|'drama'|'manga'|'novel'|'game'|'academic_book'|'paper'
  }
  ```
- 🔵 **出力**: 当該 `mediaType` に対応するアクセントカラークラス（`getMediaTypeAccentClass(mediaType)` の戻り値、例: `text-accent-anime`）を適用したバッジ。日本語ラベルを表示。
- 🟡 **入出力の関係性（ラベル）**: 8種別の日本語ラベル（例: anime→「アニメ」, movie→「映画」, drama→「ドラマ」, manga→「マンガ」, novel→「小説」, game→「ゲーム」, academic_book→「専門書」, paper→「論文」）は設計文書に明記がないため妥当な推測。
- 🔵 **データフロー**: `mediaType` → `getMediaTypeAccentClass()` → CSSクラス → バッジ表示。アクセント色定義は `frontend/src/lib/media-type-accent.ts` に既存。
- 参照: interfaces.ts MediaType（8種別）, media-type-accent.ts（`MEDIA_TYPE_ACCENT_CLASS` の8キー）, TASK-0002（アクセントカラー）

### 2.3 FilterBar（枠のみ） 🔵

- 🔵 **入力（props）**:
  ```typescript
  interface FilterBarProps {
    children?: React.ReactNode;  // 任意。バー内に差し込む要素
  }
  ```
- 🔵 **出力**: `children` をそのまま内包するコンテナ（バーの器）。本タスクでは絞り込みUIの詳細（media_type/タグ/カテゴリ/お気に入り/status の選択UI）は実装しない。
- 🔵 **入出力の関係性**: 後続フック `useSearchParamsFilter`（TASK-0008）および詳細UI（TASK-0010, Phase 2）と接続するための器のみ提供。
- 参照: TASK-0006「実装詳細 > FilterBar（枠のみ）」, overview.md「TASK-0010: FilterBarコンポーネント詳細実装」

### 2.4 EmptyState 🟡

- 🟡 **入力（props）**:
  ```typescript
  interface EmptyStateProps {
    message: string;            // 必須。空状態メッセージ（例:「コレクションがありません」）
    actionLabel?: string;       // 任意。アクションボタンのラベル
    onAction?: () => void;      // 任意。アクションボタンクリック時のコールバック
  }
  ```
- 🟡 **出力**: `message` を表示。`actionLabel` 指定時はアクションボタンを表示し、クリックで `onAction` を呼ぶ。
- 🟡 **入出力の関係性**: `actionLabel` と `onAction` はペアで機能する想定（導線ボタン）。props構成自体は EDGE-101 から妥当な推測。
- 参照: requirements.md EDGE-101「アイテムが0件の場合、空状態として『コレクションがありません』等のメッセージと追加画面への導線を表示」, TASK-0006「実装詳細 > EmptyState」

### 2.5 ConfirmDialog 🟡

- 🟡 **入力（props）**:
  ```typescript
  interface ConfirmDialogProps {
    open: boolean;             // 必須。表示/非表示制御
    title: string;            // 必須。ダイアログタイトル
    description?: string;      // 任意。本文
    onConfirm: () => void;    // 必須。確認ボタンクリック時
    onCancel: () => void;     // 必須。キャンセルボタンクリック時
    confirmLabel?: string;    // 任意。確認ボタンラベル（デフォルト想定あり）
    cancelLabel?: string;     // 任意。キャンセルボタンラベル（デフォルト想定あり）
  }
  ```
- 🟡 **出力**: `open=true` のとき `title`/`description` と確認/キャンセルボタンを表示。`open=false` のとき非表示。確認ボタンで `onConfirm`、キャンセルボタンで `onCancel` を呼ぶ。
- 🟡 **入出力の関係性**: 状態管理（open/close）は呼び出し側または `useConfirmDialog`（TASK-0008）と組み合わせる前提の制御コンポーネント（controlled）。`confirmLabel`/`cancelLabel` 未指定時のデフォルト文言は実装時に決定（妥当な推測）。
- 参照: TASK-0006「実装詳細 > ConfirmDialog」, requirements.md REQ-007（アイテム削除の確認用途）

- **参照したEARS要件**: REQ-001, REQ-002, REQ-007, EDGE-101
- **参照した設計文書**: interfaces.ts（Item, ItemBase, MediaType, ItemStatus）, media-type-accent.ts

---

## 3. 制約条件（EARS非機能要件・アーキテクチャ設計ベース）

- 🔵 **アーキテクチャ制約**:
  - 各コンポーネントは `frontend/src/components/common/` に配置する（`src/components/ui/` は shadcn/ui 専用）。
  - 基底コンポーネントは shadcn/ui（Button/Dialog/Badge 等）を使用する。`ConfirmDialog` は shadcn/ui の `Dialog` をベースにする。
  - 条件付きスタイルは `class-variance-authority`（cva）+ `tailwind-merge`（`cn()`）で指定する。
  - media_type のアクセント色は `getMediaTypeAccentClass()`（`src/lib/media-type-accent.ts`）から取得する（色値をハードコードしない）。
  - 参照: architecture.md「コンポーネント粒度」, note.md「2. 開発ルール」「8. 注意事項・制約」, button.tsx（参考パターン）

- 🔵 **型安全性制約**:
  - props インターフェースを明示的に定義する。
  - `Item` 型は `media_type` 判別共用体として扱う。型ガード `isItemOfType<T>()` を必要に応じて利用する。
  - 参照: interfaces.ts（Item 判別共用体）, note.md「8. 注意事項・制約 > 型安全性」

- 🔵 **テスト制約**:
  - Vitest + @testing-library/react + @testing-library/jest-dom を使用する。テスト環境は jsdom。
  - テストファイルはコンポーネントと同一ディレクトリに `*.test.tsx` で配置する（例: `src/components/common/MediaCard.test.tsx`）。
  - 単体テストのみ対象。統合テストは Phase 2 の画面実装タスク内で実施。E2E は対象外。
  - 参照: note.md「2. 開発ルール > テスト駆動開発」「5. テスト関連情報」, vitest.config.ts, src/test/setup.ts

- 🟡 **パフォーマンス要件**: 本タスク固有の数値目標は設計文書に明記なし。表示専用の軽量コンポーネントであるため、不要な再レンダリングを避ける一般的配慮にとどめる（妥当な推測）。

- 🟡 **セキュリティ要件**: 本タスク固有の要件は設計文書に明記なし。`coverImageUrl` 等の外部URLは表示のみで、`dangerouslySetInnerHTML` 等のXSSリスク要素を導入しない（妥当な推測）。

- 🟡 **アクセシビリティ制約**: A11y の本格対応は Phase 6（TASK-0034）。本タスクでは shadcn/ui（Radix UI ベース）の標準的なアクセシビリティ（Dialog のフォーカストラップ・ロール等）に準拠する範囲とする（妥当な推測）。

- **参照したEARS要件**: REQ-001, REQ-002, REQ-007, EDGE-101
- **参照した設計文書**: architecture.md, interfaces.ts, vitest.config.ts, button.tsx, media-type-accent.ts

---

## 4. 想定される使用例（EARS Edgeケース・データフローベース）

### 4.1 基本的な使用パターン 🔵

- 🔵 **MediaCard**: 一覧画面（HomePage 等）でアイテム配列を `map` し、各 `item` を `<MediaCard item={item} onClick={handleSelect} />` として表示。クリックで詳細画面へ遷移する導線を提供（後続タスク側）。
- 🔵 **MediaTypeBadge**: `MediaCard` 内および一覧/詳細画面で `<MediaTypeBadge mediaType={item.mediaType} />` として種別を色付きバッジ表示。
- 🔵 **FilterBar**: 一覧画面で `<FilterBar>{/* TASK-0010で詳細フィルタUI */}</FilterBar>` として絞り込みUIの器を配置。
- 🟡 **EmptyState**: 一覧取得結果が0件のとき `<EmptyState message="コレクションがありません" actionLabel="アイテムを追加" onAction={goToAdd} />` を表示。
- 🟡 **ConfirmDialog**: 削除操作時に `<ConfirmDialog open={isOpen} title="削除しますか？" onConfirm={doDelete} onCancel={close} />` を表示。

### 4.2 データフロー 🔵

- 🔵 **MediaTypeBadge**: `mediaType` → `getMediaTypeAccentClass(mediaType)` → CSSクラス適用 → 色付きバッジ描画。
- 🟡 **MediaCard**: `item`（API 由来の Item）→ ItemBase フィールド抽出 → 各表示要素へマッピング（`mediaType` は `MediaTypeBadge` へ委譲）。
- 参照: dataflow.md（一覧表示フロー）, media-type-accent.ts

### 4.3 エッジケース 🟡

- 🟡 **EDGE-101（0件）**: 一覧0件時に `EmptyState` を表示（基本パターン4.1に対応）。
- 🟡 **MediaCard: coverImageUrl 欠落**: `coverImageUrl` が undefined のアイテムでも、プレースホルダ等でエラーなく描画する（妥当な推測）。
- 🟡 **MediaTypeBadge: 8種別網羅**: 8種別すべて（academic_book/paper 含む）でエラーなく描画し、それぞれ異なるアクセントクラスを適用する（`MEDIA_TYPE_ACCENT_CLASS` の8キーに対応、🔵）。
- 🟡 **FilterBar: children 未指定**: `children` 省略時もコンテナのみ描画されエラーにならない（妥当な推測）。
- 🟡 **EmptyState: action 未指定**: `actionLabel`/`onAction` 省略時はメッセージのみ表示し、ボタンを描画しない。
- 🟡 **ConfirmDialog: open=false**: ダイアログ内容を描画しない（非表示）。

### 4.4 エラーケース 🟡

- 🟡 本タスクのコンポーネントは表示専用であり、API/非同期エラー処理は持たない。props 由来の入力欠落（任意項目）に対して描画を破綻させないことがエラー耐性の主眼（妥当な推測）。

- **参照したEARS要件**: EDGE-101, REQ-001, REQ-007
- **参照した設計文書**: dataflow.md（一覧表示フロー）, interfaces.ts, media-type-accent.ts

---

## 5. EARS要件・設計文書との対応関係

- **参照したユーザストーリー**: コレクション閲覧者（一覧でアイテムを把握したい）、コレクション管理者（削除等の操作を確認付きで安全に行いたい）
- **参照した機能要件**:
  - REQ-001: 全体一覧画面でコレクション全体をカード/リスト表示（→ MediaCard）
  - REQ-002: media_type・タグ・カテゴリ・お気に入り・status での絞り込み（→ FilterBar 枠のみ）
  - REQ-007: アイテム削除（→ ConfirmDialog の用途）
- **参照した非機能要件**: 本タスク固有の NFR は設計文書に明記なし（パフォーマンス/セキュリティ/A11y は一般配慮および後続 Phase 6 対応、🟡）
- **参照したEdgeケース**: EDGE-101（アイテム0件 → EmptyState）
- **参照した受け入れ基準**: acceptance-criteria.md（TC-001-01 他: 一覧表示・絞り込み・CRUD・Empty State）※本タスクは部品単体テストが対象
- **参照した設計文書**:
  - **アーキテクチャ**: architecture.md「コンポーネント粒度」「Feature-Sliced 寄りレイヤード構成」
  - **データフロー**: dataflow.md（一覧表示フロー、MediaTypeBadge 色適用フロー）
  - **型定義**: interfaces.ts（Item 判別共用体, ItemBase, MediaType, ItemStatus）
  - **データベース**: 本タスクは表示層のため直接対応なし（DB制約は backend 側）
  - **API仕様**: 本タスクは表示層のため直接対応なし（apiClient は TASK-0005）
  - **既存実装**: media-type-accent.ts（`getMediaTypeAccentClass`）, button.tsx（cva/cn 参考パターン）, App.test.tsx（テスト参考パターン）

---

## 6. 品質判定

```
✅ 高品質:
- 要件の曖昧さ: なし（5コンポーネントの存在・役割・propsインターフェースは設計文書／タスクノートで確定）
- 入出力定義: 完全（各コンポーネントの props 型と表示・コールバックを明示）
- 制約条件: 明確（配置先・基底ライブラリ・スタイル管理・テスト方針が確定）
- 実装可能性: 確実（依存する media-type-accent.ts / utils.ts / shadcn Button は既存。Dialog/Badge は shadcn 追加で導入可能）
```

### 信頼性レベル分布

- 🔵 青信号: 機能概要、MediaCard/MediaTypeBadge/FilterBar の存在・props、アーキテクチャ/型/テスト制約、データフロー（多数）
- 🟡 黄信号: MediaCard レイアウト詳細、MediaTypeBadge 日本語ラベル、EmptyState/ConfirmDialog の props 構成、NFR・A11y、エッジ/エラーケースの一部
- 🔴 赤信号: なし

**総合評価**: 高品質。コンポーネントの存在・役割分担は architecture.md「コンポーネント粒度」から確実（🔵）。各 props の細部（日本語ラベル、カードレイアウト、ダイアログ既定文言）は一般的UIパターンからの妥当な推測（🟡）で、実装時に確定すればよく、TDD を進める上での阻害要因はない。

---

## 7. 次のステップ

次のお勧めステップ: `/tsumiki:tdd-testcases frontend-collection-ui TASK-0006` でテストケースの洗い出しを行います。
