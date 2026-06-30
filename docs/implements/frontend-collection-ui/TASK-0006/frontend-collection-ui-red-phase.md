# TASK-0006: 共通UIコンポーネント実装 - Redフェーズ記録

**機能名**: 共通UIコンポーネント（MediaCard / MediaTypeBadge / FilterBar / EmptyState / ConfirmDialog）
**タスクID**: TASK-0006
**要件名**: frontend-collection-ui
**フェーズ**: Red（失敗するテスト作成）
**作成日**: 2026-06-30

---

## 1. 作成したテストファイル一覧

| コンポーネント | テストファイル | テストケース数 |
| --- | --- | --- |
| MediaCard | `frontend/src/components/common/MediaCard.test.tsx` | 10（正常系6・異常系2・境界値2、うちit.eachで8種別反復のため実行件数は計17件） |
| MediaTypeBadge | `frontend/src/components/common/MediaTypeBadge.test.tsx` | 4（正常系2・異常系1・境界値1、境界値はit.eachで8種別反復） |
| FilterBar | `frontend/src/components/common/FilterBar.test.tsx` | 3（正常系1・異常系1・境界値1） |
| EmptyState | `frontend/src/components/common/EmptyState.test.tsx` | 6（正常系3・異常系1・境界値2） |
| ConfirmDialog | `frontend/src/components/common/ConfirmDialog.test.tsx` | 7（正常系4・異常系1・境界値2） |

**テストケース定義書対応**: `docs/implements/frontend-collection-ui/TASK-0006/frontend-collection-ui-testcases.md` の TC-MC-*, TC-MB-*, TC-FB-*, TC-ES-*, TC-CD-* 全30件を実装（要求の10件以上を達成）。

---

## 2. テスト実行結果（失敗確認）

実行コマンド:
```bash
yarn test -- src/components/common/MediaCard.test.tsx src/components/common/MediaTypeBadge.test.tsx src/components/common/FilterBar.test.tsx src/components/common/EmptyState.test.tsx src/components/common/ConfirmDialog.test.tsx
```

結果: **5 Test Files failed (5)** / no tests run（importエラーのためテスト自体が実行不能）

各ファイルで以下と同種のエラーが発生：
```
Error: Failed to resolve import "./MediaCard" from "src/components/common/MediaCard.test.tsx". Does the file exist?
```

これは想定通りの失敗である。`frontend/src/components/common/` 配下に実コンポーネントファイル（MediaCard.tsx, MediaTypeBadge.tsx, FilterBar.tsx, EmptyState.tsx, ConfirmDialog.tsx）がまだ存在しないため、Vite の import 解決でエラーとなる。Greenフェーズでこれらのファイルを実装することでテストが実行可能になり、各アサーションの成否が判定される。

---

## 3. Greenフェーズで実装すべき内容

### 3.1 MediaCard.tsx
- Props: `{ item: Item; onClick?: (item: Item) => void; }`
- `data-testid="media-card"` を持つカードコンテナ
- `item.title` を表示
- `<MediaTypeBadge mediaType={item.mediaType} />` を内包
- `coverImageUrl` を `<img src={...} />` として表示（`role="img"`、未設定時はプレースホルダで例外を起こさない）
- `isFavorite` 表示用に `data-testid="media-card-favorite"` 要素、`data-favorite={String(item.isFavorite)}` 属性
- `status` 表示用に `data-testid="media-card-status"` 要素、`data-status={item.status}` 属性
- クリック時に `onClick?.(item)` を呼ぶ（onClick未指定時は何もしない安全なガード）

### 3.2 MediaTypeBadge.tsx
- Props: `{ mediaType: MediaType; }`
- `getMediaTypeAccentClass(mediaType)` でアクセントクラスを取得し className に適用
- mediaType → 日本語ラベル変換テーブル（anime→アニメ, movie→映画, drama→ドラマ, manga→漫画, novel→小説, game→ゲーム, academic_book→専門書, paper→論文）
- 想定外の文字列が来てもクラッシュしない（`MEDIA_TYPE_ACCENT_CLASS[key]` が undefined の場合は空文字等にフォールバック）
- shadcn/ui の Badge をベースに使用（`npx shadcn@latest add badge` が必要な場合あり）

### 3.3 FilterBar.tsx
- Props: `{ children?: React.ReactNode; }`
- `data-testid="filter-bar"` を持つコンテナ要素
- children をそのまま描画（複数子・未指定いずれも安全に処理）

### 3.4 EmptyState.tsx
- Props: `{ message: string; actionLabel?: string; onAction?: () => void; }`
- `data-testid="empty-state"` を持つコンテナ要素
- `message` を表示
- `actionLabel` が指定された場合のみボタンを描画（`role="button"`、ラベルは `actionLabel`）
- ボタンクリックで `onAction?.()` を呼ぶ（onAction未指定時も安全）

### 3.5 ConfirmDialog.tsx
- Props: `{ open: boolean; title: string; description?: string; onConfirm: () => void; onCancel: () => void; confirmLabel?: string; cancelLabel?: string; }`
- shadcn/ui の Dialog をベースに使用（`npx shadcn@latest add dialog` が必要な場合あり）
- `open=false` のとき title/description 等の内容を描画しない
- `open=true` のとき title・description（指定時）を表示
- 確認ボタン（ラベル: `confirmLabel` または既定値、例: 'OK'）クリックで `onConfirm()` を呼ぶ
- キャンセルボタン（ラベル: `cancelLabel` または既定値、例: 'キャンセル'）クリックで `onCancel()` を呼ぶ
- ラベル省略時もボタンは2つ描画される

---

## 4. 信頼性レベル分布（実装したテストケース）

- 🔵 青信号: 9件 — title表示、onClick発火、MediaTypeBadge委譲、8種別網羅（MediaCard/MediaTypeBadge）、children パススルー、アクセントクラス適用
- 🟡 黄信号: 19件 — 画像/お気に入り/status表示の具体的形式、EmptyState/ConfirmDialogの挙動詳細、任意prop省略時の堅牢性
- 🔴 赤信号: 2件 — MediaTypeBadge型外値防御、ConfirmDialog多重クリック挙動（実装方針により調整可）

---

## 5. 次のステップ

次のお勧めステップ: `/tsumiki:tdd-green` でGreenフェーズ（最小実装）を開始します。
