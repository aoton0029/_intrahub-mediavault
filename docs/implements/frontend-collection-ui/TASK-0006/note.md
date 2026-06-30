# TASK-0006: 共通UIコンポーネント実装 - 開発コンテキスト

**作成日**: 2026-06-30  
**タスクID**: TASK-0006  
**タスクタイプ**: TDD  
**推定工数**: 8時間  
**フェーズ**: Phase 1 - 基盤構築

## 1. 技術スタック

### フロントエンド全体
- **React**: 18.3+ / TypeScript 5.7+ / Vite 6
- **UI ライブラリ**: Tailwind CSS 4 + shadcn/ui
- **フォーム**: react-hook-form + zod
- **サーバー状態管理**: TanStack Query 5
- **UI 状態管理**: React内蔵 useState / useContext
- **ルーティング**: React Router v7
- **通知UI**: sonner (toast)
- **アイコン**: lucide-react

### テスト関連
- **ユニットテスト**: Vitest
- **テストライブラリ**: @testing-library/react, @testing-library/jest-dom
- **E2Eテスト**: Playwright
- **テスト環境**: jsdom

参照: `docs/spec/frontend-collection-ui/note.md`, `frontend/CLAUDE.md`

## 2. 開発ルール

### コンポーネント設計パターン
- **Atomic的粒度分割**: コンポーネントは `src/components/ui/` (shadcn/ui ベース) と `src/components/common/` (独自コンポーネント) に分離する
- **判別共用体パターン**: Item型は `media_type` による判別共用体で実装 (TypeScript型安全性向上)
- **型ガード関数**: `isItemOfType<T>()` で media_type チェック
- **CSS クラス管理**: `class-variance-authority` (cva) + `tailwind-merge` で条件付きスタイル指定

参照: `docs/design/frontend-collection-ui/architecture.md` 「コンポーネント粒度」

### テスト駆動開発（TDD）
- **フェーズ**: Red → Green → Refactor → Verify-Complete
- **単体テストフレームワーク**: Vitest + Testing Library
- **テストファイル配置**: コンポーネント同じディレクトリに `*.test.tsx` (例: `src/components/common/MediaCard.test.tsx`)
- **テスト環境初期化**: `src/test/setup.ts` で `@testing-library/jest-dom/vitest` をインポート

## 3. 関連実装

### 既存参考パターン

#### Button コンポーネント (shadcn/ui)
- **ファイル**: `src/components/ui/button.tsx`
- **特徴**: 
  - cva（class-variance-authority）で variant/size バリエーション管理
  - cn() ユーティリティ (tailwind-merge + clsx) でクラス結合
  - Radix UI Slot コンポーネント対応（`asChild` prop）
  - 型安全な props 継承（React.ComponentProps<"button">）

#### メディアタイプアクセント色管理
- **ファイル**: `src/components/lib/media-type-accent.ts`
- **機能**: 8種別の media_type ごとに対応する CSS クラスを返す
  ```typescript
  export function getMediaTypeAccentClass(mediaType: MediaType): string
  ```
- **使用例**: `MediaTypeBadge` で `text-accent-anime` 等を動的適用

参照: `frontend/src/lib/media-type-accent.ts`

### テスト参考パターン
- **ファイル**: `src/App.test.tsx`
- **パターン**: `describe` + `it` テスト構造、`render()` + `screen` Query でアサーション
- **セットアップ**: Vitest globals + jsdom + setup.ts で jest-dom matchers 有効化

## 4. 設計文書

### アーキテクチャ
- **参照**: `docs/design/frontend-collection-ui/architecture.md`
- **重点**:
  - Feature-Sliced 寄り レイヤード構成（pages → features → components → api/hooks/types/lib）
  - Component 粒度分割: ui（shadcn/ui）+ common（再利用独自コンポーネント）
  - バックエンド連携は fetch ラップの `apiClient` + TanStack Query フック

### 型定義
- **ファイル**: `docs/design/frontend-collection-ui/interfaces.ts`
- **重点**:
  - `Item` 型は `media_type` での判別共用体（8種別）
  - `MediaType`: 'anime' | 'movie' | 'drama' | 'manga' | 'novel' | 'game' | 'academic_book' | 'paper'
  - 統一API レスポンス: `ApiOkResponse<T>` | `ApiErrorResponse`
  - `ApiClientError` 例外クラス: エラーハンドリング用

### 要件定義
- **ファイル**: `docs/spec/frontend-collection-ui/requirements.md`
- **関連要件**:
  - REQ-001: 全体一覧画面でコレクション全体をカード/リスト表示
  - REQ-002: media_type・タグ・カテゴリ・お気に入い・status での絞り込み
  - EDGE-101: アイテム 0 件の場合、空状態メッセージと追加導線表示

### 受け入れ基準
- **ファイル**: `docs/spec/frontend-collection-ui/acceptance-criteria.md`
- **TC-001-01 他**: 絞り込み、CRUD、フィルタ、Empty State のテストケース定義

参照: `docs/design/frontend-collection-ui/architecture.md`, `docs/design/frontend-collection-ui/interfaces.ts`, `docs/spec/frontend-collection-ui/requirements.md`, `docs/spec/frontend-collection-ui/acceptance-criteria.md`

## 5. テスト関連情報

### ユニットテスト設定
- **設定ファイル**: `frontend/vitest.config.ts`
  - 環境: jsdom
  - globals: true
  - setupFiles: `./src/test/setup.ts`
  - 除外: node_modules, tests/e2e
- **セットアップファイル**: `frontend/src/test/setup.ts`
  - `@testing-library/jest-dom/vitest` インポート（jest-dom matchers 有効化）

### E2Eテスト設定
- **設定ファイル**: `frontend/playwright.config.ts`
- **テストディレクトリ**: `./tests/e2e`
- **ベースURL**: `http://localhost:5173`
- **ブラウザ**: chromium

### テストユーティリティ・パターン
- **Vitest**: describe, it, expect グローバル関数使用可（globals: true）
- **Testing Library**: `render()`, `screen`, `fireEvent`, `userEvent` パターン
- **アサーション**: jest-dom matchers (toBeInTheDocument, etc.)

### 既存テストサンプル
- **ファイル**: `frontend/src/App.test.tsx`
- **パターン**:
  ```typescript
  describe('App', () => {
    it('renders the home page at the root route', () => {
      render(<App />)
      expect(screen.getByText('HomePage')).toBeInTheDocument()
    })
  })
  ```

### テスト実行コマンド
- `yarn test`: ユニットテスト 1 回実行
- `yarn test:watch`: ウォッチモード
- `yarn test:e2e`: E2Eテスト（Playwright）

参照: `frontend/vitest.config.ts`, `frontend/playwright.config.ts`, `frontend/src/test/setup.ts`, `frontend/CLAUDE.md`

## 6. 実装対象コンポーネント

### MediaCard 🔵
**信頼性**: 🔵 *architecture.md「コンポーネント粒度」、requirements.md REQ-001 より*

**役割**: 一覧画面（HomePage/GeneralListPage/AcademicListPage/PaperListPage）でアイテム1件をカード表示  
**Props インターフェース**:
```typescript
interface MediaCardProps {
  item: Item;
  onClick?: (item: Item) => void;
}
```

**表示項目**: coverImageUrl, title, mediaType（→ MediaTypeBadge）, isFavorite, status

**テスト対象**:
- item.title がレンダリング結果に表示される
- mediaType に応じて MediaTypeBadge が表示される
- onClick が渡された場合、カードクリックで呼び出される

### MediaTypeBadge 🔵
**信頼性**: 🔵 *architecture.md「コンポーネント粒度」、TASK-0002 media_type 別アクセントカラー定義 より*

**役割**: MediaType を受け取り、対応する色・ラベルでバッジ表示  
**Props インターフェース**:
```typescript
interface MediaTypeBadgeProps {
  mediaType: MediaType;
}
```

**対応 8 種別**: anime, movie, drama, manga, novel, game, academic_book, paper  
**日本語ラベル**: anime→「アニメ」等（設計文書に明記なし、妥当な推測）

**テスト対象**:
- 8 種別すべてのエラーなくレンダリング
- mediaType に応じて異なる CSS クラス（アクセントカラー）が適用

### FilterBar 🔵
**信頼性**: 🔵 *architecture.md「コンポーネント粒度」、タスク指示「枠のみ、詳細は Phase 2」 より*

**役割**: 絞り込みUI の器（コンテナコンポーネント）。詳細実装は Phase 2 (TASK-0010) で実施  
**Props インターフェース**:
```typescript
interface FilterBarProps {
  children?: React.ReactNode;
}
```

**テスト対象**:
- children が正しくレンダリングされる

### EmptyState 🟡
**信頼性**: 🟡 *requirements.md EDGE-101「コレクションがありません」メッセージ、architecture.md 明記、具体的 props は推測*

**役割**: アイテム 0 件時の空状態表示  
**Props インターフェース**:
```typescript
interface EmptyStateProps {
  message: string;
  actionLabel?: string;
  onAction?: () => void;
}
```

**テスト対象**:
- message が表示される
- actionLabel + onAction が渡された場合、ボタンクリックで onAction が呼ばれる

### ConfirmDialog 🟡
**信頼性**: 🟡 *architecture.md「コンポーネント粒度」に名前明記、具体的 props は推測*

**役割**: アイテム削除・マイリスト削除等の確認ダイアログ  
**Props インターフェース**:
```typescript
interface ConfirmDialogProps {
  open: boolean;
  title: string;
  description?: string;
  onConfirm: () => void;
  onCancel: () => void;
  confirmLabel?: string;
  cancelLabel?: string;
}
```

**ベース**: shadcn/ui の Dialog  
**状態管理**: 呼び出し側または useConfirmDialog (TASK-0008) と組み合わせ想定

**テスト対象**:
- open=true で内容が表示、open=false で非表示
- 確認ボタンクリックで onConfirm 呼ばれる
- キャンセルボタンクリックで onCancel 呼ばれる

参照: `docs/tasks/frontend-collection-ui/TASK-0006.md` 「実装詳細」

## 7. 実装手順（TDD フロー）

1. **Red フェーズ**: `/tsumiki:tdd-red` → テストケース作成（失敗確認）
2. **Green フェーズ**: `/tsumiki:tdd-green` → テストを通す実装
3. **Refactor フェーズ**: `/tsumiki:tdd-refactor` → コード品質改善
4. **Verify-Complete フェーズ**: `/tsumiki:tdd-verify-complete` → テスト完全性確認

## 8. 注意事項・制約

### アーキテクチャ制約
- 各コンポーネントは `src/components/common/` に配置（`src/components/ui/` は shadcn/ui 専用）
- shadcn/ui コンポーネント（Button, Dialog, Badge 等）をベースに使用
- media_type アクセント色は `getMediaTypeAccentClass()` 関数で取得

### テスト制約
- 単体テストのみ対象。統合テストは Phase 2 の画面実装タスク内で実施
- E2E テストは対象外（該当する場合は Playwright で別途定義）

### 型安全性
- Item 型は media_type 判別共用体で実装
- props インターフェースを明示的に定義

参照: `docs/tasks/frontend-collection-ui/TASK-0006.md` 「完了条件」「信頼性レベルサマリー」

## 9. ファイルパス一覧

### 設計・要件関連
- `docs/spec/frontend-collection-ui/note.md` - 技術スタック・実装状況
- `docs/spec/frontend-collection-ui/requirements.md` - 要件定義（REQ-001 他）
- `docs/spec/frontend-collection-ui/acceptance-criteria.md` - 受け入れ基準・テストケース
- `docs/design/frontend-collection-ui/architecture.md` - アーキテクチャ設計
- `docs/design/frontend-collection-ui/interfaces.ts` - 型定義（Item, MediaType 他）
- `docs/design/frontend-collection-ui/dataflow.md` - データフロー図
- `docs/tasks/frontend-collection-ui/TASK-0006.md` - タスク詳細

### フロントエンド実装
- `frontend/src/components/ui/button.tsx` - Button コンポーネント（参考パターン）
- `frontend/src/components/common/` - 実装対象ディレクトリ
- `frontend/src/lib/media-type-accent.ts` - media_type アクセント色管理
- `frontend/src/lib/utils.ts` - cn() ユーティリティ
- `frontend/src/test/setup.ts` - テスト環境初期化
- `frontend/src/App.test.tsx` - テスト参考パターン

### テスト・設定
- `frontend/vitest.config.ts` - Vitest 設定
- `frontend/playwright.config.ts` - Playwright 設定
- `frontend/package.json` - 依存関係・スクリプト
- `frontend/CLAUDE.md` - 開発コマンド

## 10. 関連タスク

### 依存タスク（前提）
- `TASK-0001`: プロジェクト初期設定（Vite + TypeScript）
- `TASK-0002`: Tailwind + shadcn/ui セットアップ

### 後続タスク
- `TASK-0007` 以降: Phase 2 画面実装（MediaCard/FilterBar を組み合わせた統合テスト）
- `TASK-0008`: useSearchParamsFilter, useConfirmDialog フック実装
- `TASK-0010`: FilterBar 詳細UI 実装（Phase 2）

参照: `docs/tasks/frontend-collection-ui/TASK-0006.md` 「依存タスク」

---

**最後更新**: 2026-06-30  
**作成者**: Claude Code Agent
